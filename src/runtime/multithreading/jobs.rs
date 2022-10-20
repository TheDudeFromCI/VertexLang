//! An implementation of the jobs system used by Vertex.
//!
//! The job system is a multi-threaded task pooling system that is used to
//! evaluate nodes within the graph in order of dependencies. This allows
//! multiple parts of the graph tree to be evaluated in parallel.


use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};


/// An executable function that can be passed into a job.
///
/// This function takes no input arguments, and outputs a list of new jobs that
/// were created inside of this job that need to be waited on. If this list of
/// child jobs is empty, the job is marked as finished immediately. If there are
/// jobs within the returned job handle list, the parent job is marked as
/// hibernating until all children jobs have finished.
///
/// If the parent job does not require a child job to finish before considering
/// itself finished, then it should not be included in the returned job handle
/// list.
///
/// Note that this returned job handle list should only include jobs that were
/// spawned from within the job itself.
pub trait JobExec: Fn() -> Vec<JobHandle> + Send + Sync {}
impl<Func: Fn() -> Vec<JobHandle> + Send + Sync> JobExec for Func {}


/// Contains meta data about a job waiting to be executed.
///
/// The generic type, Func, is the function type that is called by the job
/// system.
#[derive(Clone)]
pub struct JobHandle {
    scheduler_uid: u32,
    job_id:        usize,
    job:           Arc<dyn JobExec>,
}


impl JobHandle {
    /// Gets the scheduler uid this job was created with.
    pub fn get_scheduler_uid(&self) -> u32 {
        self.scheduler_uid
    }


    /// Gets the id of this job.
    pub fn get_job_id(&self) -> usize {
        self.job_id
    }


    /// Gets a copy of the job function pointer.
    pub fn get_job(&self) -> Arc<dyn JobExec> {
        self.job.clone()
    }
}


impl PartialEq<JobHandle> for JobHandle {
    fn eq(&self, other: &JobHandle) -> bool {
        self.scheduler_uid == other.scheduler_uid && self.job_id == other.job_id
    }
}


impl Debug for JobHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "JobHandle<{}:{}>", self.scheduler_uid, self.job_id)
    }
}


/// A job instance within the job scheduler that has dependencies that need to
/// be resolved before it can be added to the main queue.
struct SleepingJob {
    handle:       JobHandle,
    dependencies: Vec<usize>,
}


/// A hibernating job is a job that has already been executed, but has created
/// nested jobs and will not be marked as finished until those nested jobs have
/// been finished.
struct HibernatingJob {
    handle:       JobHandle,
    dependencies: Vec<usize>,
}


/// A job queue for retrieving jobs within the queue asynchronously.
///
/// The generic type, Func, is the function type that is called by the job
/// system.
#[derive(Clone)]
pub struct JobQueue {
    receiver: Arc<Mutex<Receiver<JobHandle>>>,
}


impl JobQueue {
    /// Creates a new job queue instance from the given receiver.
    fn new(rec: Receiver<JobHandle>) -> Self {
        JobQueue {
            receiver: Arc::new(Mutex::new(rec)),
        }
    }

    /// Gets the next job in the queue.
    ///
    /// This method will block the thread and wait until a new job becomes
    /// available.
    pub fn next(&self) -> JobHandle {
        self.receiver.lock().unwrap().recv().unwrap()
    }
}


struct PendingJobs {
    sender: Arc<Mutex<Option<Sender<JobHandle>>>>,
}


impl PendingJobs {
    /// Creates a new pending jobs queue instance from the given sender.
    fn new(send: Sender<JobHandle>) -> Self {
        PendingJobs {
            sender: Arc::new(Mutex::new(Some(send))),
        }
    }


    /// Sends the given job handle to a the jobs queue.
    ///
    /// This function panics if the pending jobs queue has already been killed.
    fn send(&self, handle: JobHandle) {
        self.sender.lock().unwrap().as_ref().unwrap().send(handle).unwrap();
    }


    /// Deallocates the sender mpsc channel, allowing the queue to be dropped
    /// and causing all job queues to panic.
    ///
    /// This will effectively terminate all listening worker threads.
    fn kill(&self) {
        *self.sender.lock().unwrap() = None;
    }
}


/// A self-maintained container for creating and scheduling jobs that can be
/// executed from external worker threads.
///
/// It is highly recommended to store this object behind an `Arc<Mutex<..>>`
/// block in order to pass references to the worker threads. (See
/// [`JobScheduler::into_async`]) This will allow for workers to tell the job
/// scheduler when they have finished their task and update the sleeping jobs'
/// dependencies lists accordingly.
///
/// The generic type, Func, is the function type that is called by the job
/// system.
pub struct JobScheduler {
    scheduler_uid:      u32,
    min_job_id:         usize,
    cur_job_id:         usize,
    buffer:             Vec<usize>,
    sleeping_jobs:      Vec<SleepingJob>,
    hibernating_jobs:   Vec<HibernatingJob>,
    push_notifications: HashMap<usize, Sender<()>>,
    pending_jobs:       PendingJobs,
    job_queue:          JobQueue,
}


static SCHEDULER_UID: AtomicU32 = AtomicU32::new(0);


impl JobScheduler {
    /// Creates a new job scheduler instance.
    pub fn new() -> Self {
        let uid = SCHEDULER_UID.fetch_add(1, Ordering::SeqCst);
        let (sender, receiver) = channel();
        JobScheduler {
            scheduler_uid:      uid,
            min_job_id:         0,
            cur_job_id:         1,
            buffer:             vec![],
            sleeping_jobs:      vec![],
            hibernating_jobs:   vec![],
            push_notifications: HashMap::new(),
            pending_jobs:       PendingJobs::new(sender),
            job_queue:          JobQueue::new(receiver),
        }
    }


    /// Wraps this job scheduler in an `Arc<Mutex<..>>` block to make working
    /// with this job scheduler across worker threads easier.
    pub fn into_async(self) -> AsyncJobScheduler {
        AsyncJobScheduler {
            scheduler: Arc::new(Mutex::new(self)),
        }
    }


    /// Creates a notification receiver for the given job.
    ///
    /// This receiver will receive an empty `()` object when the job has
    /// finished being executed. This notification is triggered once and then
    /// the channel is discarded. If the job has already finished executing
    /// before this method is called, then `None` is returned and no job
    /// notifications are set up.
    fn build_job_notify_channel(&mut self, job: &JobHandle) -> Option<Receiver<()>> {
        if self.is_done(job) {
            return None;
        }

        let (sender, receiver) = channel();
        self.push_notifications.insert(job.job_id, sender);
        Some(receiver)
    }
}


impl Scheduler for JobScheduler {
    fn get_scheduler_uid(&self) -> u32 {
        self.scheduler_uid
    }


    fn get_queue(&self) -> JobQueue {
        self.job_queue.clone()
    }


    fn get_finished_jobs(&self) -> usize {
        self.min_job_id + self.buffer.len()
    }


    fn new_job(&mut self, dependencies: Vec<JobHandle>, job: impl JobExec + 'static) -> JobHandle {
        if dependencies.iter().any(|j| j.scheduler_uid != self.scheduler_uid) {
            panic!("Tried to use job dependencies from another job system");
        }

        let dependencies: Vec<usize> = dependencies
            .into_iter()
            .map(|j| j.job_id)
            .filter(|id| *id > self.min_job_id)
            .filter(|id| !self.buffer.contains(id))
            .collect();

        let job_id = self.cur_job_id;
        self.cur_job_id += 1;

        let job = JobHandle {
            scheduler_uid: self.scheduler_uid,
            job_id,
            job: Arc::new(job),
        };

        if dependencies.is_empty() {
            self.pending_jobs.send(job.clone());
        } else {
            let sleeping_job = SleepingJob {
                handle: job.clone(),
                dependencies,
            };

            self.sleeping_jobs.push(sleeping_job);
        }

        job
    }


    fn hibernate(&mut self, job: JobHandle, dependencies: Vec<JobHandle>) {
        if job.scheduler_uid != self.scheduler_uid {
            panic!("Tried to hibernate a job from another job system");
        }

        if self.hibernating_jobs.iter().any(|j| j.handle.job_id == job.job_id) {
            panic!("Job {} already hibernating", job.job_id);
        }

        if self.sleeping_jobs.iter().any(|j| j.handle.job_id == job.job_id) {
            panic!("Job {} has not yet been queued", job.job_id);
        }

        let dependencies: Vec<usize> = dependencies
            .into_iter()
            .map(|j| j.job_id)
            .filter(|id| *id > self.min_job_id)
            .filter(|id| !self.buffer.contains(id))
            .collect();

        if dependencies.is_empty() {
            self.finish_job(job);
            return;
        }

        let hibernator = HibernatingJob {
            handle: job,
            dependencies,
        };

        self.hibernating_jobs.push(hibernator);
    }


    fn finish_job(&mut self, job: JobHandle) {
        if job.scheduler_uid != self.scheduler_uid {
            panic!("Tried to finish a job from another job system");
        }

        if self.sleeping_jobs.iter().any(|j| j.handle.job_id == job.job_id) {
            panic!("Job {} has not yet been queued", job.job_id);
        }

        let job_id = job.job_id;

        // Remove job from sleeping jobs' dependencies
        for job in &mut self.sleeping_jobs {
            let dep_index = job.dependencies.iter().position(|id| *id == job_id);
            match dep_index {
                Some(index) => {
                    job.dependencies.remove(index);
                    if job.dependencies.is_empty() {
                        let job = job.handle.clone();
                        self.pending_jobs.send(job);
                    }
                },
                None => continue,
            };
        }
        self.sleeping_jobs.retain(|j| !j.dependencies.is_empty());

        // Remove job from hibernating jobs' dependencies
        let mut finished_jobs = vec![];
        for job in &mut self.hibernating_jobs {
            let dep_index = job.dependencies.iter().position(|id| *id == job_id);
            match dep_index {
                Some(index) => {
                    job.dependencies.remove(index);
                    if job.dependencies.is_empty() {
                        finished_jobs.push(job.handle.clone());
                    }
                },
                None => continue,
            };
        }
        self.hibernating_jobs.retain(|j| !j.dependencies.is_empty());

        // Cleanup finished job buffer
        self.buffer.push(job_id);
        loop {
            let next_min_id = self.min_job_id + 1;

            match self.buffer.iter().position(|id| *id <= next_min_id) {
                Some(index) => self.buffer.remove(index),
                None => break,
            };

            self.min_job_id = next_min_id;
        }

        // Push all jobs that finished hibernating
        for job in finished_jobs {
            self.finish_job(job);
        }

        // Trigger push notifications for finished job, if needed.
        if let Some(sender) = self.push_notifications.remove(&job.job_id) {
            sender.send(()).unwrap()
        }
    }


    fn is_done(&self, job: &JobHandle) -> bool {
        if job.scheduler_uid != self.scheduler_uid {
            panic!("Tried to finish a job from another job system");
        }

        job.job_id <= self.min_job_id || self.buffer.contains(&job.job_id)
    }


    fn terminate_workers(&self) {
        self.pending_jobs.kill();
    }
}


impl Default for JobScheduler {
    fn default() -> Self {
        Self::new()
    }
}


/// Provides an `Arc<Mutex<...>>` wrapper around the job scheduler, to make
/// using it from multiple threads easier.
#[derive(Clone)]
pub struct AsyncJobScheduler {
    scheduler: Arc<Mutex<JobScheduler>>,
}


impl AsyncJobScheduler {
    /// Blocks the current thread until the target job finishes.
    ///
    /// If the job has already finished, this method returns immediately. Note
    /// that this function should not be called from within a job, as no other
    /// jobs will be able to use the worker that the job is running on until
    /// this job finishes. This also has a potential chance to cause a deadlock
    /// in certain situations, (such as having only one worker thread). It is
    /// recommended to use hibernate instead for this use case.
    pub fn wait_for_job(&self, job: &JobHandle) {
        let receiver = self.scheduler.lock().unwrap().build_job_notify_channel(job);
        if let Some(receiver) = receiver {
            receiver.recv().unwrap();
        }
    }
}


impl Scheduler for AsyncJobScheduler {
    /// A synchronized wrapper for [`JobScheduler::get_scheduler_uid`].
    fn get_scheduler_uid(&self) -> u32 {
        self.scheduler.lock().unwrap().get_scheduler_uid()
    }


    /// A synchronized wrapper for [`JobScheduler::get_finished_jobs`].
    fn get_finished_jobs(&self) -> usize {
        self.scheduler.lock().unwrap().get_finished_jobs()
    }


    /// A synchronized wrapper for [`JobScheduler::get_queue`].
    fn get_queue(&self) -> JobQueue {
        self.scheduler.lock().unwrap().get_queue()
    }


    /// A synchronized wrapper for [`JobScheduler::new_job`].
    fn new_job(&mut self, dependencies: Vec<JobHandle>, job: impl JobExec + 'static) -> JobHandle {
        self.scheduler.lock().unwrap().new_job(dependencies, job)
    }


    /// A synchronized wrapper for [`JobScheduler::finish_job`].
    fn finish_job(&mut self, job: JobHandle) {
        self.scheduler.lock().unwrap().finish_job(job)
    }


    /// A synchronized wrapper for [`JobScheduler::hibernate`].
    fn hibernate(&mut self, job: JobHandle, dependencies: Vec<JobHandle>) {
        self.scheduler.lock().unwrap().hibernate(job, dependencies)
    }


    /// A synchronized wrapper for [`JobScheduler::is_done`].
    fn is_done(&self, job: &JobHandle) -> bool {
        self.scheduler.lock().unwrap().is_done(job)
    }


    /// A synchronized wrapper for [`JobScheduler::terminate_workers`].
    fn terminate_workers(&self) {
        self.scheduler.lock().unwrap().terminate_workers()
    }
}


/// Structs with this trait are capable of managing job systems. Namely,
/// creating new jobs, handling job dependencies, and executing jobs.
///
/// Jobs are asynchronous tasks that are executed in order to process data. Jobs
/// may be chained together to ensure that data is handled as efficiently as
/// possible. Jobs that require data from other jobs are processed in serial
/// while jobs that don't require data from other jobs are processes in
/// parallel.
///
/// The generic type, Func, is the function type that is called by the job
/// system.
pub trait Scheduler {
    /// Gets the uid for this scheduler instance.
    fn get_scheduler_uid(&self) -> u32;


    /// Creates a new job queue to retrieve jobs from.
    ///
    /// This job queue should be passed into the worker thread and called from
    /// within a loop. This queue will automatically wait for new jobs to be
    /// added to the queue, blocking the thread until they become available.
    fn get_queue(&self) -> JobQueue;


    /// Gets the total number of jobs that have been completed.
    ///
    /// Note that hibernating jobs do not count as completed until all of their
    /// child jobs have also been completed.
    fn get_finished_jobs(&self) -> usize;


    /// Creates a new job instance using the specified list of dependencies.
    ///
    /// This method will determine the job id based on an internal state value
    /// and will adjust the dependencies list so that it can be assured that
    /// all jobs that this new job handle depends on have been completed.
    ///
    /// This method will `panic!()` if attempting to use job handlers that were
    /// created by other job schedulers.
    fn new_job(&mut self, dependencies: Vec<JobHandle>, job: impl JobExec + 'static) -> JobHandle;


    /// Marks a job to begin hibernating until the listed job dependencies have
    /// all finished.
    ///
    /// A hibernating job is a job that has already been executed, but has
    /// internally created several new nested jobs. The parent job will not be
    /// finished until the child jobs have all finished.
    ///
    /// If a job creates a child job during execution and should not be marked
    /// as finished until that child job finishes, this method should be used
    /// instead of [`Self::finish_job`] with a list of all child jobs.
    fn hibernate(&mut self, job: JobHandle, dependencies: Vec<JobHandle>);


    /// Marks the given job as finished. This will automatically update sleeping
    /// jobs and pushes them to the job queue if their dependencies have been
    /// satisfied.
    ///
    /// This method will `panic!()` if attempting to use a job handler that was
    /// created by another job scheduler.
    fn finish_job(&mut self, job: JobHandle);


    /// Checks whether or not the given job has finished executing.
    fn is_done(&self, job: &JobHandle) -> bool;


    /// Manually triggers all worker threads to be terminated by dropped the
    /// sender mpsc channel and causing all job queues to panic.
    fn terminate_workers(&self);
}


#[cfg(test)]
mod test {
    use super::*;
    use ntest::timeout;
    use std::{thread, time};


    #[test]
    fn create_job_with_deps() {
        let blank = Vec::new;

        let mut scheduler = JobScheduler::new();
        let job1 = scheduler.new_job(vec![], blank);
        let job2 = scheduler.new_job(vec![job1.clone()], blank);
        let job3 = scheduler.new_job(vec![], blank);

        let queue = scheduler.get_queue();
        assert_eq!(queue.next(), job1);
        assert_eq!(queue.next(), job3);

        scheduler.finish_job(job1);
        assert_eq!(queue.next(), job2);
    }


    #[test]
    #[should_panic]
    fn finish_job_wrong_scheduler() {
        let blank = Vec::new;

        let mut sch1 = JobScheduler::new();
        let mut sch2 = JobScheduler::new();

        let job = sch1.new_job(vec![], blank);
        sch2.finish_job(job);
    }


    #[test]
    #[should_panic]
    fn deps_between_schedulers() {
        let blank = Vec::new;

        let mut sch1 = JobScheduler::new();
        let mut sch2 = JobScheduler::new();

        let a = sch1.new_job(vec![], blank);
        sch2.new_job(vec![a], blank);
    }


    #[test]
    fn hibernate_job() {
        let blank = Vec::new;

        let mut sch = JobScheduler::new();
        let queue = sch.get_queue();

        let job1 = sch.new_job(vec![], blank);
        let job2 = sch.new_job(vec![job1.clone()], blank);
        let job3 = sch.new_job(vec![], blank);

        assert_eq!(queue.next(), job1);
        sch.hibernate(job1, vec![job3.clone()]);

        assert_eq!(queue.next(), job3);
        sch.finish_job(job3);

        assert_eq!(queue.next(), job2);
    }


    #[test]
    fn deps_finished_auto_run() {
        let blank = Vec::new;

        let mut sch = JobScheduler::new();
        let queue = sch.get_queue();

        let job1 = sch.new_job(vec![], blank);
        sch.finish_job(queue.next());

        let job2 = sch.new_job(vec![job1], blank);
        assert_eq!(queue.next(), job2);
    }


    #[test]
    #[should_panic]
    fn finish_before_queue() {
        let blank = Vec::new;

        let mut sch = JobScheduler::new();
        let job1 = sch.new_job(vec![], blank);
        let job2 = sch.new_job(vec![job1], blank);

        sch.finish_job(job2);
    }


    #[test]
    #[timeout(1000)]
    fn wait_for_job() {
        let answer_slot = Arc::new(Mutex::new(0));
        let answer_borrow = answer_slot.clone();

        let sleep = move || {
            *answer_borrow.lock().unwrap() = 13;
            thread::sleep(time::Duration::from_millis(100));
            vec![]
        };


        let mut sch = JobScheduler::new().into_async();

        let mut sch_worker = sch.clone();
        thread::spawn(move || {
            let queue = sch_worker.get_queue();
            let handle = queue.next();
            handle.get_job()();
            sch_worker.finish_job(handle);
            thread::sleep(time::Duration::from_millis(5000));
        });

        let job = sch.new_job(vec![], sleep);
        sch.wait_for_job(&job);

        assert_eq!(*answer_slot.lock().unwrap(), 13);
    }
}
