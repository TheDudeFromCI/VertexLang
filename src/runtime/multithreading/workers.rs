use super::jobs::{AsyncJobScheduler, JobHandle, Scheduler};
use std::thread;
use std::thread::JoinHandle;


/// Builds and starts an indicated number of worker threads for the given job
/// scheduler.
///
/// Generally, the number of worker threads should be set to the number of
/// available CPU cores.
pub fn build_workers(scheduler: &AsyncJobScheduler, threads: u32) -> Vec<JoinHandle<()>> {
    let mut join_handles = vec![];
    for _ in 0..threads {
        let mut sch = scheduler.clone();
        let queue = scheduler.get_queue();
        let join_handle = thread::spawn(move || {
            loop {
                let handle = queue.next();
                let children: Vec<JobHandle> = handle.get_job()();

                if children.is_empty() {
                    sch.finish_job(handle);
                } else {
                    sch.hibernate(handle, children);
                }
            }
        });

        join_handles.push(join_handle);
    }

    join_handles
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::multithreading::JobScheduler;
    use ntest::assert_true;
    use std::time::Duration;


    #[test]
    fn terminate_workers() {
        let sch = JobScheduler::new().into_async();
        let workers = build_workers(&sch, 1);

        sch.terminate_workers();
        thread::sleep(Duration::from_millis(250));

        assert_true!(workers[0].is_finished())
    }
}
