use std::{collections::HashMap, cell::RefCell, rc::Rc};
use std::mem::Discriminant;

use strum::IntoEnumIterator;

use super::Argument;
use super::context::ContextFunction;
use super::{FunctionDefinition, Data, system::SystemFunction};

pub struct FunctionSignature {
    pub name: String,
    pub args: Vec<SignatureArgument>,
    pub repeating: bool,
    pub return_type: Discriminant<Data>,
}

pub enum SignatureArgument {
    Raw,
    Any,
    Data(Discriminant<Data>),
}

pub struct FunctionScope {
    scope: Vec<FunctionDefinition>,
}

#[derive(Default)]
pub struct VariableScope {
    scope: HashMap<String, Rc<RefCell<Data>>>,
}

impl FunctionScope {
    pub fn get(&self, name: &str, args: &[Argument],  function_scope: &FunctionScope, variable_scope: &VariableScope) -> Option<&FunctionDefinition> {
        for function in self.scope.iter() {
            if function.signature().name != name {
                continue;
            }

            let signature = function.signature();

            let mut found = true;
            for (i, arg) in signature.args.iter().enumerate() {
                if !match arg {
                    SignatureArgument::Raw => args.get(i).is_some(),
                    SignatureArgument::Data(data) => args.get(i)?.evaluated_discriminant(function_scope, variable_scope) == *data,
                    SignatureArgument::Any => args.get(i).is_some()
                } {
                    found = false;
                    break;
                }
            }

            if found {
                return Some(function);
            }
        }

        None
    }
}

impl Default for FunctionScope {
    fn default() -> Self {
        let mut scope = Vec::new();
    
        for function in SystemFunction::iter() {
            scope.push(FunctionDefinition::System(function));
        }
        
        for function in ContextFunction::iter() {
            scope.push(FunctionDefinition::Context(function));
        }

        Self { scope }
    }
}

impl VariableScope {
    pub fn insert(&mut self, name: String, data: Rc<RefCell<Data>>) {
        self.scope.insert(name, data);
    }

    pub fn get(&self, name: &str) -> Option<Rc<RefCell<Data>>> {
        self.scope.get(name).cloned()
    }
}