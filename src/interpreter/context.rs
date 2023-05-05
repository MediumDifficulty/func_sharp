use std::cell::RefCell;
use std::mem;
use std::rc::Rc;

use strum_macros::EnumIter;

use crate::signature;

use super::consts::{any, number, raw};
use super::scope::{FunctionScope, FunctionSignature, SignatureArgument, VariableScope};
use super::{Argument, Data};

#[derive(EnumIter, Debug, Clone)]
pub enum ContextFunction {
    Println,
    Let,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
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
            ContextFunction::Println => {
                println!(
                    "{}",
                    args.iter()
                        .map(|arg| arg.data().borrow().to_string())
                        .collect::<Vec<_>>()
                        .join(" ")
                );
                Data::Unit
            }
            ContextFunction::Let => {
                let evaluated = args[1].data();
                variable_scope.insert(args[0].raw().to_string(), evaluated);
                Data::Unit
            }
            ContextFunction::Add => {
                operator_impl(|acc, arg| acc + arg.data().borrow().number(), args)
            }
            ContextFunction::Sub => {
                operator_impl(|acc, arg| acc - arg.data().borrow().number(), args)
            }
            ContextFunction::Mul => {
                operator_impl(|acc, arg| acc * arg.data().borrow().number(), args)
            }
            ContextFunction::Div => {
                operator_impl(|acc, arg| acc / arg.data().borrow().number(), args)
            }
            ContextFunction::Mod => {
                operator_impl(|acc, arg| acc % arg.data().borrow().number(), args)
            }
        }
    }

    pub fn signature(&self) -> FunctionSignature {
        match self {
            ContextFunction::Let => signature!("let".into(), Data::Unit, false, raw(), any()),
            ContextFunction::Println => signature!("println".into(), Data::Unit, true, any()),
            ContextFunction::Add => signature!("+".into(), Data::Number(0.), true, number()),
            ContextFunction::Sub => signature!("-".into(), Data::Number(0.), true, number()),
            ContextFunction::Mul => signature!("*".into(), Data::Number(0.), true, number()),
            ContextFunction::Div => signature!("/".into(), Data::Number(0.), true, number()),
            ContextFunction::Mod => signature!("%".into(), Data::Number(0.), true, number()),
        }
    }
}

fn operator_impl(operation: impl FnMut(f64, &ContextArgument) -> f64, args: &[ContextArgument]) -> Data {
    let mut iter = args.iter();
    iter.next()
        .map(|start| Data::Number(iter.fold(start.data().borrow().number(), operation)))
        .expect("Expected at least one argument")
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
