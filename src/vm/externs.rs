use super::{Node, NodeFunction};
use crate::data::{Data, VertexFunction};
use crate::multithreading::jobs::Scheduler;
use std::sync::Arc;


/// Converts a basic external function into a node function.
pub fn extern_func(func: VertexFunction) -> Arc<dyn NodeFunction> {
    Arc::new(move |node: &Arc<Node>| {
        let node = node.clone();
        let mut scheduler = node.get_scheduler();

        let node_fut = node.clone();
        let job = move || {
            let inputs = node_fut.get_inputs();
            let inputs = inputs.iter().filter_map(|input| input.get_data()).collect();
            let data = func(inputs);
            node_fut.set_data(Arc::new(data));
            vec![]
        };

        let depends = node.inputs_as_dependencies();
        scheduler.new_job(depends, job)
    })
}


/// Creates a simple node function that takes no inputs and always returns a
/// literal data value.
pub fn literal(data: Arc<Data>) -> Arc<dyn NodeFunction> {
    Arc::new(move |node: &Arc<Node>| {
        let data = data.clone();
        let node = node.clone();
        let mut scheduler = node.get_scheduler();

        let node_fut = node.clone();
        let job = move || {
            node_fut.set_data(data.clone());
            vec![]
        };

        let depends = node.inputs_as_dependencies();
        scheduler.new_job(depends, job)
    })
}
