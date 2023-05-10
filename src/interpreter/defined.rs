use std::{
    cell::RefCell,
    mem::{self, Discriminant},
    rc::Rc,
};

use super::{
    scope::{FunctionScope, FunctionSignature, SignatureArgument, VariableScope},
    Argument, Data, Invocation, ControlFlow,
};

/// A user defined function.
#[derive(Debug, Clone)]
pub struct DefinedFunction {
    body: Vec<Invocation>,
    signature: FunctionSignature,
    scope: Rc<RefCell<VariableScope>>,
    argument_names: Vec<String>,
}

impl DefinedFunction {
    pub fn signature(&self) -> FunctionSignature {
        self.signature.clone()
    }

    pub fn execute(
        &self,
        args: &[Rc<RefCell<Data>>],
        function_scope: &mut FunctionScope,
        global_scope: Rc<RefCell<VariableScope>>,
    ) -> Rc<RefCell<Data>> {
        // Load arguments into scope
        let scope = Rc::new(RefCell::new(self.scope.borrow().clone()));
        for (i, name) in self.argument_names.iter().enumerate() {
            scope.borrow_mut().insert(name.clone(), args[i].clone());
        }

        // Execute body
        for invocation in self.body.iter() {
            if let Data::ControlFlow(ControlFlow::Return(data)) = invocation.evaluate(function_scope, scope.clone(), global_scope.clone()).borrow().clone() {
                return data;
            }
        }

        Rc::new(RefCell::new(Data::Unit))
    }

    pub fn new(arguments: &[Argument], global_scope: Rc<RefCell<VariableScope>>) -> Self {
        let mut args = arguments.iter();
        let name = args.next().expect("No function name given").ident();
        let return_type =
            str_to_data_discriminant(&args.next().expect("Malformed function").ident());

        let mut argument_names = Vec::new();
        let mut argument_types = Vec::new();
        let mut body = Vec::new();

        let mut in_signature = true;
        while let Some(arg) = args.next() {
            if in_signature {
                if let Some(arg_type) = args.next() {
                    if mem::discriminant(arg_type) == mem::discriminant(&Argument::Ident("".into()))
                    {
                        argument_names.push(arg.ident());
                        argument_types.push(str_to_data_discriminant(&arg_type.ident()));
                    } else {
                        body.push(arg.invocation());
                        body.push(arg_type.invocation());
                        in_signature = false;
                    }
                } else {
                    body.push(arg.invocation());
                }

                continue;
            }

            body.push(arg.invocation());
        }

        Self {
            body,
            signature: FunctionSignature {
                name,
                args: argument_types
                    .iter()
                    .map(|e| SignatureArgument::Data(*e))
                    .collect(),
                repeating: false,
                return_type,
            },
            scope: global_scope,
            argument_names,
        }
    }
}

fn str_to_data_discriminant(string: &str) -> Discriminant<Data> {
    match string {
        "str" | "string" => mem::discriminant(&Data::String("".to_string())),
        "num" | "number" => mem::discriminant(&Data::Number(0.)),
        "bool" | "boolean" => mem::discriminant(&Data::Boolean(false)),
        "void" => mem::discriminant(&Data::Unit),
        _ => panic!("Argument is not a recognised data type"),
    }
}
