use crate::runtime::bytecode::{FunctionCall, OperationInput, VertexBytecode};
use crate::runtime::data::{Data, VertexFunction};
use crate::runtime::multithreading::{AsyncJobScheduler, JobHandle, Scheduler};
use std::sync::{Arc, Mutex};


/// A virtual machine is an execution runtime for Vertex bytecode.
pub struct VirtualMachine {
    scheduler: AsyncJobScheduler,
    bytecode:  Arc<VertexBytecode>,
}

impl VirtualMachine {
    /// Creates a new virtual machine instance for the given Vertex bytecode.
    pub fn new(scheduler: &AsyncJobScheduler, bytecode: Arc<VertexBytecode>) -> Self {
        Self {
            scheduler: scheduler.clone(),
            bytecode,
        }
    }


    /// Executes the internal function within the bytecode at the given function
    /// index.
    ///
    /// The function is executed using the provided vector of data inputs on the
    /// async job scheduler attached to this virtual machine. This function
    /// blocks until the program has finished execution, and the output
    /// value is returned.
    pub fn execute(&self, func_id: usize, inputs: Vec<Data>) -> Arc<Data> {
        let inputs: Vec<AsyncVNode> =
            inputs.into_iter().map(Arc::new).map(VirtualNode::constant).collect();

        let node = VirtualNode::internal(&self.scheduler, inputs, self.bytecode.clone(), func_id);
        let handle = node.lock().unwrap().handle().clone();
        let job = handle.unwrap();
        self.scheduler.wait_for_job(&job);

        let data = &node.lock().unwrap().data;
        data.clone().unwrap()
    }
}


type AsyncVNode = Arc<Mutex<VirtualNode>>;


struct VirtualNode {
    handle: Option<JobHandle>,
    data:   Option<Arc<Data>>,
}

impl VirtualNode {
    /// Creates a new virtual node from an external function.
    fn external(
        scheduler: &AsyncJobScheduler, inputs: Vec<AsyncVNode>, function: VertexFunction,
    ) -> Arc<Mutex<Self>> {
        let virtual_node = Arc::new(Mutex::new(Self {
            handle: None,
            data:   None,
        }));

        let deps = inputs.iter().filter_map(|node| node.lock().unwrap().handle().clone()).collect();

        let job_node = virtual_node.clone();
        let job = move || {
            let mut function_in: Vec<Arc<Data>> = vec![];

            for input in &inputs {
                let data = &input.lock().unwrap().data;
                let data = data.as_ref().unwrap();
                function_in.push(data.clone());
            }

            let output = function(&function_in);
            job_node.lock().unwrap().data = Some(Arc::new(output));

            vec![]
        };

        let handle = scheduler.clone().new_job(deps, job);
        virtual_node.lock().unwrap().handle = Some(handle);

        virtual_node
    }


    /// Creates a new virtual node from an internal function.
    fn internal(
        scheduler: &AsyncJobScheduler, inputs: Vec<AsyncVNode>, bytecode: Arc<VertexBytecode>,
        function: usize,
    ) -> Arc<Mutex<Self>> {
        let virtual_node = Arc::new(Mutex::new(Self {
            handle: None,
            data:   None,
        }));

        let deps = inputs.iter().filter_map(|node| node.lock().unwrap().handle().clone()).collect();

        let internal_scheduler = scheduler.clone();
        let job_node = virtual_node.clone();
        let job = move || {
            let mut operations = vec![];

            let function = &bytecode.get_internal_functions()[function];
            for operation in function.get_operations() {
                let mut op_inputs = vec![];

                for input in operation.get_inputs() {
                    let node = match input {
                        OperationInput::Param(i) => &inputs[*i],
                        OperationInput::Hidden(i) => &operations[*i],
                    };

                    op_inputs.push(node.clone());
                }

                let op_node = match operation.get_function() {
                    FunctionCall::Internal(i) => {
                        VirtualNode::internal(&internal_scheduler, op_inputs, bytecode.clone(), *i)
                    },
                    FunctionCall::External(i) => {
                        let function = *bytecode.get_external_functions()[*i].get_function_exec();
                        VirtualNode::external(&internal_scheduler, op_inputs, function)
                    },
                    FunctionCall::Constant(i) => {
                        let data = bytecode.get_constants()[*i].clone();
                        VirtualNode::constant(data)
                    },
                };

                operations.push(op_node);
            }

            let last_operation = operations[operations.len() - 1].clone();
            let last_job = last_operation.lock().unwrap().handle().clone().unwrap();

            let job_node = job_node.clone();
            let copy_data_job = move || {
                let output = last_operation.lock().unwrap().data.clone();
                job_node.lock().unwrap().data = output;
                vec![]
            };

            let new_job = internal_scheduler.clone().new_job(vec![last_job], copy_data_job);
            vec![new_job]
        };

        let handle = scheduler.clone().new_job(deps, job);
        virtual_node.lock().unwrap().handle = Some(handle);

        virtual_node
    }


    /// Creates a new virtual node from a data value constant.
    fn constant(data: Arc<Data>) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            handle: None,
            data:   Some(data),
        }))
    }


    /// Gets a reference to this node's job handle.
    fn handle(&self) -> &Option<JobHandle> {
        &self.handle
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::IRDataType;
    use crate::runtime::multithreading;
    use crate::runtime::multithreading::JobScheduler;
    use crate::runtime::registry::{FuncMeta, FunctionRegistry};
    use crate::unwrap_data;
    use indoc::indoc;
    use pretty_assertions::assert_eq;


    #[test]
    fn vm_hello_world() {
        fn add(inputs: &[Arc<Data>]) -> Data {
            let a = unwrap_data!(inputs[0], Int);
            let b = unwrap_data!(inputs[1], Int);
            Data::Int(a + b)
        }

        let mut registry = FunctionRegistry::new();
        registry
            .register(
                FuncMeta::new(
                    "Add".to_owned(),
                    add,
                    vec![IRDataType::Int, IRDataType::Int],
                    IRDataType::Int,
                )
                .unwrap(),
            )
            .unwrap();

        let bytecode = VertexBytecode::from_source(
            indoc! { r#"
                MainModule = mod {
                    Func1 = function {
                        params = (in: Int)
                        return = (out: Int)

                        x = Func2(in, extern Add(1, 2))
                        out = extern Add(x, 1)
                    }

                    Func2 = function {
                        params = (a: Int, b: Int)
                        return = (c: Int)

                        x1 = extern Add(a, b)
                        x2 = extern Add(a, x1)
                        c = extern Add(b, x2)
                    }
                }
            "#},
            &registry,
        )
        .unwrap();

        let scheduler = JobScheduler::new().into_async();
        multithreading::build_workers(&scheduler, 1);

        let vm = VirtualMachine::new(&scheduler, Arc::new(bytecode));
        let res = vm.execute(0, vec![Data::Int(5)]);
        assert_eq!(*res, Data::Int(5));
    }
}
