use std::{mem::Discriminant, collections::HashMap};
use std::mem;

use once_cell::sync::Lazy;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use super::{Argument, FunctionScope, VariableScope, Data};

pub static CONTEXT_FUNCTIONS: Lazy<HashMap<ContextSignature, ContextFunction>> = Lazy::new(|| {
    let mut map = HashMap::new();

    for function in ContextFunction::iter() {
        map.insert(function.signature(), function);
    }

    map
});

#[derive(PartialEq, Eq, Hash)]
pub struct ContextSignature {
    pub name: String,
    pub args: Vec<Discriminant<Argument>>,
}

#[derive(EnumIter)]
pub enum ContextFunction {
    Let,
}

macro_rules! signature {
    ($name:expr, $($arg:expr),+) => {
        ContextSignature {
            name: $name,
            args: vec![$(mem::discriminant(&$arg)),+],
        }
    };
}

impl ContextFunction {
    pub fn execute(
        &self,
        args: &[Argument],
        function_scope: &mut FunctionScope,
        variable_scope: &mut VariableScope,
    ) -> Data {
        match self {
            ContextFunction::Let => {
                let evaluated = args[1].eval(function_scope, variable_scope);
                variable_scope.insert(args[0].to_string(), evaluated);
                return Data::Unit;
            },
        }
    }

    fn signature(&self) -> ContextSignature {
        match self {
            ContextFunction::Let => signature!("let".into(), Argument::Ident("".into()), Argument::Data(super::Data::Unit)),
        }
    }
}