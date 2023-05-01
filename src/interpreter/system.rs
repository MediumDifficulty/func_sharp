use std::{cell::RefCell, rc::Rc};
use std::mem;

use strum_macros::EnumIter;

use crate::signature;

use super::consts::{string, number};
use super::scope::{FunctionScope, VariableScope, FunctionSignature};
use super::{Data};

#[derive(EnumIter, Debug, Clone, Copy)]
pub enum SystemFunction {
    Println,
    PrintlnNum,
    Add,
    Str,
}

#[allow(unused_variables)] // For now
impl SystemFunction {
    pub fn execute(
        &self,
        args: &[Rc<RefCell<Data>>],
        function_scope: &mut FunctionScope,
        variable_scope: &mut VariableScope,
    ) -> Data {
        match self {
            SystemFunction::Println => {
                println!(
                    "{}",
                    args.iter()
                        .map(|arg| arg.borrow().to_string())
                        .collect::<Vec<String>>()
                        .join(" ")
                );
                Data::Unit
            }
            SystemFunction::PrintlnNum => {
                println!("{}", args[0].borrow().number());
                Data::Unit
            }
            SystemFunction::Add => {
                Data::Number(args[0].borrow().number() + args[1].borrow().number())
            }
            SystemFunction::Str => Data::String(args[0].borrow().to_string()),
        }
    }

    pub fn signature(&self) -> FunctionSignature {
        match self {
            SystemFunction::Println => signature!("println".into(), Data::Unit, false, string()),
            SystemFunction::PrintlnNum => signature!("println".into(), Data::Unit, false, number()),
            SystemFunction::Add => signature!("+".into(), Data::Number(0.), false, number(), number()),
            SystemFunction::Str => signature!("str".into(), Data::String("".into()), false, number()),
        }
    }
}