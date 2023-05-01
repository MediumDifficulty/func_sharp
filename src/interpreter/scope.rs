use std::{collections::HashMap, cell::RefCell, rc::Rc};

use strum::IntoEnumIterator;

use super::{FunctionSignature, FunctionDefinition, Data, system::SystemFunction};

pub struct FunctionScope {
    scope: HashMap<FunctionSignature, FunctionDefinition>,
}

#[derive(Default)]
pub struct VariableScope {
    scope: HashMap<String, Rc<RefCell<Data>>>,
}

impl FunctionScope {
    pub fn get(&self, signature: &FunctionSignature) -> Option<&FunctionDefinition> {
        self.scope.get(signature)
    }
}

impl Default for FunctionScope {
    fn default() -> Self {
        let mut scope = HashMap::new();
    
        for function in SystemFunction::iter() {
            scope.insert(function.signature(), FunctionDefinition::System(function));
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