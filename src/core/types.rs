type DataTypeIndex = usize;
type VariableIndex = usize;

#[derive(Debug)]
pub struct Module {
    pub name: String,
    pub functions: Vec<FunctionImpl>,
    pub data_types: Vec<DataType>,
}

impl Module {
    #[inline]
    pub fn new(name: &str) -> Module {
        return Module {
            name: String::from(name),
            functions: vec![],
            data_types: vec![],
        };
    }

    pub fn find_function(&self, name: &str) -> Option<&FunctionImpl> {
        for function in &self.functions {
            if function.header.name.eq(name) {
                return Some(&function);
            }
        }

        return None;
    }

    pub fn find_data_type(&self, name: &String) -> Option<&DataType> {
        for data_type in &self.data_types {
            if data_type.name.eq(name) {
                return Some(&data_type);
            }
        }

        return None;
    }

    pub fn get_data_type(&self, index: DataTypeIndex) -> Option<&DataType> {
        if index >= self.data_types.len() {
            return None;
        }
        return Some(&self.data_types[index]);
    }

    pub fn add_function(&mut self, func: FunctionImpl) {
        self.functions.push(func);
    }

    pub fn add_data_type(&mut self, mut data_type: DataType) {
        data_type.index = self.data_types.len();
        self.data_types.push(data_type);
    }
}

#[derive(Debug)]
pub struct FunctionImpl {
    pub header: FunctionHeader,
    pub variables: Vec<Variable>,
    pub parameters: Vec<VariableIndex>,
    pub returned: Vec<VariableIndex>,
    pub expressions: Vec<Expression>,
}

#[derive(Debug)]
pub struct FunctionHeader {
    pub name: String,
    pub serial: bool,
    pub parameters: Vec<DataTypeIndex>,
    pub return_types: Vec<DataTypeIndex>,
}

#[derive(Debug)]
pub struct Variable {
    pub name: String,
    pub data_type: DataTypeIndex,
}

#[derive(Debug, PartialEq)]
pub struct DataType {
    pub name: String,
    pub index: DataTypeIndex,
}

#[derive(Debug)]
pub struct Expression {
    pub func: FunctionHeader,
    pub inputs: Vec<VariableIndex>,
    pub outputs: Vec<VariableIndex>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_module() -> Module {
        let mut my_mod = Module {
            name: String::from("my_mod"),
            functions: vec![],
            data_types: vec![],
        };

        let number_dt = DataType {
            name: String::from("number"),
            index: 0,
        };
        my_mod.add_data_type(number_dt);

        let bool_dt = DataType {
            name: String::from("bool"),
            index: 0,
        };
        my_mod.add_data_type(bool_dt);

        let string_dt = DataType {
            name: String::from("string"),
            index: 0,
        };
        my_mod.add_data_type(string_dt);

        let func1 = FunctionImpl {
            header: FunctionHeader {
                name: String::from("func1"),
                serial: false,
                parameters: vec![
                    my_mod
                        .find_data_type(&String::from("number"))
                        .unwrap()
                        .index,
                    my_mod
                        .find_data_type(&String::from("number"))
                        .unwrap()
                        .index,
                ],
                return_types: vec![my_mod.find_data_type(&String::from("bool")).unwrap().index],
            },
            variables: vec![],
            parameters: vec![],
            expressions: vec![],
            returned: vec![],
        };
        my_mod.add_function(func1);

        let func2 = FunctionImpl {
            header: FunctionHeader {
                name: String::from("func2"),
                serial: false,
                parameters: vec![
                    my_mod
                        .find_data_type(&String::from("string"))
                        .unwrap()
                        .index,
                    my_mod
                        .find_data_type(&String::from("string"))
                        .unwrap()
                        .index,
                    my_mod
                        .find_data_type(&String::from("string"))
                        .unwrap()
                        .index,
                ],
                return_types: vec![
                    my_mod
                        .find_data_type(&String::from("number"))
                        .unwrap()
                        .index,
                ],
            },
            variables: vec![],
            parameters: vec![],
            expressions: vec![],
            returned: vec![],
        };
        my_mod.add_function(func2);

        return my_mod;
    }

    #[test]
    fn module_find_function() {
        let module = build_module();
        let function = module.find_function(&String::from("func2"));

        assert!(!function.is_none());
        assert_eq!(function.unwrap().header.name, String::from("func2"))
    }

    #[test]
    fn module_find_data_type() {
        let module = build_module();
        let data_type = module.find_data_type(&String::from("string"));

        assert!(!data_type.is_none());
        assert_eq!(data_type.unwrap().name, String::from("string"))
    }
}
