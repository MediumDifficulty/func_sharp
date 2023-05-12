use std::mem::Discriminant;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use strum::IntoEnumIterator;

use super::context::ContextFunction;
use super::{system::SystemFunction, Data, FunctionSource};
use super::{Argument, ControlFlow};

#[derive(Debug, Clone)]
/// The signature of a [`FunctionSource`]
pub struct FunctionSignature {
    pub name: String,
    pub args: Vec<SignatureArgument>,
    pub repeating: bool,
    pub return_type: ReturnType,
}

#[derive(Debug, Clone)]
/// The way of identifying an [`Argument`] without the data used by [`FunctionSignature`]
pub enum SignatureArgument {
    Raw,
    Any,
    Data(Discriminant<Data>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReturnType {
    Data(Discriminant<Data>),
    Any,
}

/// Contains a list of all defined functions
pub struct FunctionScope {
    scope: Vec<FunctionSource>,
}

// Contains all defined Variables
pub type VariableScope = HashMap<String, Rc<RefCell<Data>>>;

impl FunctionScope {
    pub fn get(
        &self,
        name: &str,
        args: &[Argument],
        function_scope: &FunctionScope,
        variable_scope: Rc<RefCell<VariableScope>>,
    ) -> Option<&FunctionSource> {
        self.scope.iter().find(|&function| {
            if function.signature().get_ref().name != name {
                return false;
            }

            let signature = function.signature();

            for (i, arg) in args.iter().enumerate() {
                let corresponding = if signature.get_ref().repeating {
                    Some(&signature.get_ref().args[i.min(signature.get_ref().args.len() - 1)])
                } else {
                    signature.get_ref().args.get(i)
                };

                if let Some(corresponding) = corresponding {
                    if !match corresponding {
                        SignatureArgument::Data(data) => {
                            match arg.return_type(function_scope, variable_scope.clone()) {
                                ReturnType::Data(value) => value == *data,
                                ReturnType::Any => true,
                            }
                        }
                        _ => true,
                    } {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            true
        })
    }

    pub fn insert(&mut self, function: FunctionSource) {
        self.scope.push(function);
    }
}

impl Default for FunctionScope {
    fn default() -> Self {
        let mut scope = Vec::new();

        for function in SystemFunction::iter() {
            scope.push(FunctionSource::System(function));
        }

        for function in ContextFunction::iter() {
            scope.push(FunctionSource::Context(function));
        }

        Self { scope }
    }
}

/// Generate default variable scope
pub fn default_variable_scope() -> VariableScope {
    let mut scope = HashMap::new();

    scope.insert("true".into(), Rc::new(RefCell::new(Data::Boolean(true))));
    scope.insert("false".into(), Rc::new(RefCell::new(Data::Boolean(false))));
    scope.insert(
        "break".into(),
        Rc::new(RefCell::new(Data::ControlFlow(ControlFlow::Break))),
    );
    scope.insert(
        "continue".into(),
        Rc::new(RefCell::new(Data::ControlFlow(ControlFlow::Continue))),
    );

    scope
}
