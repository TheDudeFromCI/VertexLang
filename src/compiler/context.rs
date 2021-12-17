pub struct Context {
    modules: Vec<Module>,
}

#[readonly::make]
pub struct Module {
    pub path: ModulePath,
    pub name: String,
    data_types: Vec<DataType>,
    native_functions: Vec<NativeFunction>,
    functions: Vec<Function>,
}

#[readonly::make]
pub struct DataType {
    pub path: DataTypePath,
    pub name: String,
}

#[readonly::make]
pub struct NativeFunction {
    pub path: NativeFunctionPath,
    pub name: String,
    inputs: Vec<usize>,
    outputs: Vec<usize>,
}

#[readonly::make]
pub struct Function {
    pub path: FunctionPath,
    pub name: String,
    pub serial: bool,
    pub exportable: bool,
    pub acceleratable: bool,
    inputs: Vec<usize>,
    outputs: Vec<usize>,
    variables: Vec<Variable>,
    expressions: Vec<Expression>,
    parameters: Vec<usize>,
}

#[readonly::make]
pub struct Expression {
    pub path: ExpressionPath,
    inputs: Vec<usize>,
    outputs: Vec<usize>,
}

#[readonly::make]
pub struct Variable {
    pub path: VariablePath,
    pub name: Option<String>,
    pub data_type: usize,
}

#[derive(Debug, Copy, Clone)]
pub struct ModulePath {
    module_index: usize,
}

#[derive(Debug, Copy, Clone)]
pub struct DataTypePath {
    module_index: usize,
    data_type_index: usize,
}

#[derive(Debug, Copy, Clone)]
pub struct NativeFunctionPath {
    module_index: usize,
    function_index: usize,
}

#[derive(Debug, Copy, Clone)]
pub struct FunctionPath {
    module_index: usize,
    function_index: usize,
}

#[derive(Debug, Copy, Clone)]
pub struct ExpressionPath {
    module_index: usize,
    function_index: usize,
    expression_index: usize,
}

#[derive(Debug, Copy, Clone)]
pub struct VariablePath {
    module_index: usize,
    function_index: usize,
    variable_index: usize,
}

impl Context {
    pub fn new() -> Context {
        Context { modules: vec![] }
    }

    pub fn add_module(&mut self, name: String) -> &Module {
        let index = self.modules.len();
        let module = Module {
            path: ModulePath {
                module_index: index,
            },
            name: name,
            data_types: vec![],
            native_functions: vec![],
            functions: vec![],
        };

        self.modules.push(module);
        return &self.modules[index];
    }

    pub fn get_module(&self, path: ModulePath) -> &Module {
        return &self.modules[path.module_index];
    }

    pub fn find_module(&self, name: &String) -> Option<&Module> {
        for module in &self.modules {
            if module.name.eq(name) {
                return Some(&module);
            }
        }

        return None;
    }

    pub fn add_data_type(&mut self, name: String, module_path: ModulePath) -> &DataType {
        let module = &mut self.modules[module_path.module_index];

        let index = module.data_types.len();
        let data_type = DataType {
            path: DataTypePath {
                module_index: module.path.module_index,
                data_type_index: index,
            },
            name: name,
        };

        module.data_types.push(data_type);
        return &module.data_types[index];
    }

    pub fn get_data_type(&self, path: DataTypePath) -> &DataType {
        let module = &self.modules[path.module_index];
        return &module.data_types[path.data_type_index];
    }

    pub fn find_data_type(&self, module_path: ModulePath, name: &String) -> Option<&DataType> {
        let module = self.get_module(module_path);
        for data_type in &module.data_types {
            if data_type.name.eq(name) {
                return Some(&data_type);
            }
        }

        return None;
    }

    pub fn add_native_function(
        &mut self,
        name: String,
        module_path: ModulePath,
    ) -> &NativeFunction {
        let module = &mut self.modules[module_path.module_index];

        let index = module.native_functions.len();
        let native_function = NativeFunction {
            path: NativeFunctionPath {
                module_index: module.path.module_index,
                function_index: index,
            },
            name: name,
            inputs: vec![],
            outputs: vec![],
        };

        module.native_functions.push(native_function);
        return &module.native_functions[index];
    }

    pub fn get_native_function(&self, path: NativeFunctionPath) -> &NativeFunction {
        let module = &self.modules[path.module_index];
        return &module.native_functions[path.function_index];
    }

    pub fn find_native_function(
        &self,
        module_path: ModulePath,
        name: &String,
    ) -> Option<&NativeFunction> {
        let module = &self.modules[module_path.module_index];
        for native_function in &module.native_functions {
            if native_function.name.eq(name) {
                return Some(&native_function);
            }
        }

        return None;
    }

    pub fn add_function(&mut self, name: String, module_path: ModulePath, serial: bool, exportable: bool, acceleratable: bool) -> &Function {
        let module = &mut self.modules[module_path.module_index];

        let index = module.functions.len();
        let function = Function {
            path: FunctionPath {
                module_index: module.path.module_index,
                function_index: index,
            },
            name: name,
            serial: serial,
            exportable: exportable,
            acceleratable: acceleratable,
            inputs: vec![],
            outputs: vec![],
            expressions: vec![],
            parameters: vec![],
            variables: vec![],
        };

        module.functions.push(function);
        return &module.functions[index];
    }

    pub fn get_function(&self, path: FunctionPath) -> &Function {
        let module = &self.modules[path.module_index];
        return &module.functions[path.function_index];
    }

    pub fn find_function(&self, module_path: ModulePath, name: &String) -> Option<&Function> {
        let module = &self.modules[module_path.module_index];
        for function in &module.functions {
            if function.name.eq(name) {
                return Some(&function);
            }
            }

        return None;
    }
}
