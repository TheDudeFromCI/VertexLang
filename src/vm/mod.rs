//! This module is in charge of executing a Vertex runtime.


mod externs;
mod graph;
mod node;

pub use externs::*;
pub use graph::*;
pub use node::*;


#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Data;
    use crate::multithreading::jobs::{JobScheduler, Scheduler};
    use crate::{multithreading, unwrap_data};
    use ntest::timeout;
    use std::sync::Arc;


    #[test]
    #[timeout(1000)]
    fn madd_operation() {
        fn mul(inputs: Vec<Arc<Data>>) -> Data {
            let a = unwrap_data!(inputs[0], Int);
            let b = unwrap_data!(inputs[1], Int);
            Data::Int(a * b)
        }

        fn add(inputs: Vec<Arc<Data>>) -> Data {
            let a = unwrap_data!(inputs[0], Int);
            let b = unwrap_data!(inputs[1], Int);
            Data::Int(a + b)
        }

        let const_2 = literal(Arc::new(Data::Int(2)));
        let const_4 = literal(Arc::new(Data::Int(4)));
        let const_6 = literal(Arc::new(Data::Int(6)));
        let ext_mul = extern_func(mul);
        let ext_add = extern_func(add);

        let graph_madd = graph(1, vec![
            NodeInitializer::new(ext_mul, vec![
                NodeInputPointer::ParamsNode(0),
                NodeInputPointer::ParamsNode(1),
            ]),
            NodeInitializer::new(ext_add, vec![
                NodeInputPointer::HiddenNode(0),
                NodeInputPointer::ParamsNode(2),
            ]),
        ]);

        let graph_main = graph(3, vec![
            NodeInitializer::new(const_2, vec![]),
            NodeInitializer::new(const_4, vec![]),
            NodeInitializer::new(const_6, vec![]),
            NodeInitializer::new(graph_madd, vec![
                NodeInputPointer::HiddenNode(0),
                NodeInputPointer::HiddenNode(1),
                NodeInputPointer::HiddenNode(2),
            ]),
        ]);

        let scheduler = JobScheduler::new().into_async();
        multithreading::build_workers(&scheduler, 1);

        let vm = evaluate(&scheduler, graph_main);
        let data = vm.complete();

        assert_eq!(*data, Data::Int(14));
        scheduler.terminate_workers();
    }
}
