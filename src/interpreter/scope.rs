use std::mem::Discriminant;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use strum::IntoEnumIterator;

use super::context::ContextFunction;
use super::Argument;
use super::{system::SystemFunction, Data, FunctionSource};

/// The signature of a [`FunctionSource`]
pub struct FunctionSignature {
    pub name: String,
    pub args: Vec<SignatureArgument>,
    pub repeating: bool,
    pub return_type: Discriminant<Data>,
}

/// The way of identifying an [`Argument`] without the data used by [`FunctionSignature`]
pub enum SignatureArgument {
    Raw,
    Any,
    Data(Discriminant<Data>),
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
        variable_scope: &VariableScope,
    ) -> Option<&FunctionSource> {
        self.scope.iter().find(|&function| {
            if function.signature().name != name {
                return false;
            }

            let signature = function.signature();

            for (i, arg) in args.iter().enumerate() {
                let corresponding = if signature.repeating {
                    Some(&signature.args[i.min(signature.args.len() - 1)])
                } else {
                    signature.args.get(i)
                };

                if let Some(corresponding) = corresponding {
                    if !match corresponding {
                        SignatureArgument::Data(data) => {
                            arg.evaluated_discriminant(function_scope, variable_scope) == *data
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

pub fn default_variable_scope() -> VariableScope {
    let mut scope = HashMap::new();

    scope.insert("true".into(), Rc::new(RefCell::new(Data::Boolean(true))));
    scope.insert("false".into(), Rc::new(RefCell::new(Data::Boolean(false))));

    scope
}