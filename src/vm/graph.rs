use super::NodeFunction;
use crate::multithreading::jobs::Scheduler;
use crate::vm::Node;
use std::fmt::{Debug, Display};
use std::sync::Arc;


/// An index pointer to a sibling node within a graph that produces an input for
/// the target node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeInputPointer {
    /// Pointer to the given graph input parameter index.
    ParamsNode(usize),

    /// Pointer to another hidden node with the given node index.
    HiddenNode(usize),
}

impl Display for NodeInputPointer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeInputPointer::ParamsNode(index) => write!(f, "Params({})", index),
            NodeInputPointer::HiddenNode(index) => write!(f, "Hidden({})", index),
        }
    }
}


/// A blank template for how a node should be created at runtime.
pub struct NodeInitializer {
    inputs: Vec<NodeInputPointer>,
    func:   Arc<dyn NodeFunction>,
}

impl NodeInitializer {
    /// Creates a new node initializer using the provided list of input pointers
    /// and an executable node function.
    pub fn new(func: Arc<dyn NodeFunction>, inputs: Vec<NodeInputPointer>) -> Self {
        NodeInitializer {
            inputs,
            func,
        }
    }
}


/// Creates a new graph as an executable job function.
pub fn graph(output_node: usize, nodes: Vec<NodeInitializer>) -> Arc<dyn NodeFunction> {
    Arc::new(move |node| {
        let node = node.clone();
        let inputs = node.get_inputs();
        let mut scheduler = node.get_scheduler();
        let mut hidden_nodes: Vec<Arc<Node>> = vec![];

        for node_init in &nodes {
            let mut node_inputs = vec![];
            for node_input in &node_init.inputs {
                node_inputs.push(match *node_input {
                    NodeInputPointer::ParamsNode(index) => inputs[index].clone(),
                    NodeInputPointer::HiddenNode(index) => hidden_nodes[index].clone(),
                });
            }

            let node_func = node_init.func.clone();
            let hidden_node = Node::new(&scheduler, node_inputs, node_func);
            hidden_nodes.push(Arc::new(hidden_node));
        }

        let output_node_fut = hidden_nodes[output_node].clone();
        let job = move || {
            let data = output_node_fut.get_data().unwrap();
            node.set_data(data);
            vec![]
        };

        let mut depends = hidden_nodes[output_node].inputs_as_dependencies();
        depends.push(hidden_nodes[output_node].execute().unwrap());
        dbg!(&depends);
        scheduler.new_job(depends, job)
    })
}
