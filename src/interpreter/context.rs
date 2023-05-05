use std::cell::RefCell;
use std::mem;
use std::rc::Rc;

use strum_macros::EnumIter;

use crate::signature;

use super::consts::{any, number, raw, boolean};
use super::scope::{FunctionScope, FunctionSignature, SignatureArgument, VariableScope};
use super::{Argument, Data};

/// A function definition that has access to the raw [`Argument`]s
/// Should be used *only* for functions that require access to the raw [`Argument`]s
#[derive(EnumIter, Debug, Clone)]
pub enum ContextFunction {
    Let,
    If,
    Cmp,
    Assign,
}


/// The argument type for [`ContextFunction`]
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
            }
            ContextFunction::If => {
                let mut iter = args.iter();
                if iter.next().unwrap().data().borrow().boolean() {
                    let mut cloned_scope = variable_scope.clone();
                    for invocation in iter {
                        invocation.raw().eval(function_scope, &mut cloned_scope);
                    }
                }

                Data::Unit
            }
            ContextFunction::Cmp => {
                Data::Boolean(args[0].data() == args[1].data())
            }
            ContextFunction::Assign => {
                *variable_scope.get(args[0].raw().to_string().as_str()).expect("Variable not found").borrow_mut() = args[1].data().borrow().clone();
                Data::Unit
            }
        }
    }

    pub fn signature(&self) -> FunctionSignature {
        match self {
            ContextFunction::Let => signature!("let".into(), Data::Unit, false, raw(), any()),
            ContextFunction::If => signature!("if".into(), Data::Unit, true, boolean(), raw()),
            ContextFunction::Cmp => signature!("==".into(), Data::Boolean(false), false, any(), any()),
            ContextFunction::Assign => signature!("=".into(), Data::Unit, false, raw(), any()),
        }
    }
}

pub fn to_context_args(
    args: &[Argument],
    signature: &FunctionSignature,
    function_scope: &mut FunctionScope,
    variable_scope: &mut VariableScope,
) -> Vec<ContextArgument> {
    args.iter()
        .enumerate()
        .map(
            |(i, arg)| match signature.args[i.min(signature.args.len() - 1)] {
                SignatureArgument::Any => {
                    ContextArgument::Data(arg.eval(function_scope, variable_scope))
                }
                SignatureArgument::Raw => ContextArgument::Raw(arg.clone()),
                SignatureArgument::Data(_) => {
                    ContextArgument::Data(arg.eval(function_scope, variable_scope))
                }
            },
        )
        .collect()
}
