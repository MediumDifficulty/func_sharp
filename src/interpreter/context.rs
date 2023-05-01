use std::cell::RefCell;
use std::rc::Rc;
use std::{mem::Discriminant, collections::HashMap};
use std::mem;

use once_cell::sync::Lazy;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::signature;

use super::consts::{raw, any, unit};
use super::scope::{FunctionScope, VariableScope, FunctionSignature, SignatureArgument};
use super::{Argument, Data, Invocation};

#[derive(EnumIter, Debug, Clone)]
pub enum ContextFunction {
    Let,
    Fn,
}

pub enum ContextArgument {
    Raw(Argument),
    Data(Rc<RefCell<Data>>),
}

impl ContextArgument {
    fn data(&self) -> Rc<RefCell<Data>> {
        match self {
            Self::Data(data) => data.clone(),
            _ => panic!("Expected data argument"),
        }
    }

    fn raw(&self) -> &Argument {
        match self {
            Self::Raw(raw) => raw,
            _ => panic!("Expected raw argument"),
        }
    }
}

impl ContextFunction {
    pub fn execute(
        &self,
        args: &[ContextArgument],
        function_scope: &mut FunctionScope,
        variable_scope: &mut VariableScope,
    ) -> Data {
        match self {
            ContextFunction::Let => {
                let evaluated = args[1].data();
                variable_scope.insert(args[0].raw().to_string(), evaluated);
                Data::Unit
            },
            ContextFunction::Fn => {
                Data::Unit
            }
        }
    }

    pub fn signature(&self) -> FunctionSignature {
        match self {
            ContextFunction::Let => signature!("let".into(), Data::Unit, false, raw(), any()),
            ContextFunction::Fn => signature!("fn".into(), Data::Unit, true, raw()),
        }
    }

    fn repeating(&self) -> bool {
        match self {
            ContextFunction::Let => false,
            ContextFunction::Fn  => true,
        }
    }
}

pub fn to_context_args(args: &[Argument], signature: &FunctionSignature, function_scope: &mut FunctionScope, variable_scope: &mut VariableScope) -> Vec<ContextArgument> {
    args.iter().enumerate().map(|(i, arg)| match signature.args[i] {
        SignatureArgument::Any => ContextArgument::Data(arg.eval(function_scope, variable_scope)),
        SignatureArgument::Raw => ContextArgument::Raw(arg.clone()),
        SignatureArgument::Data(_) => ContextArgument::Data(arg.eval(function_scope, variable_scope))
    }).collect()
}