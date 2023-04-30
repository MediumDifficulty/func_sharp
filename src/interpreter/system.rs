use std::{cell::RefCell, rc::Rc};
use std::mem;

use strum_macros::EnumIter;

use crate::signature;

use super::{Data, FunctionScope, VariableScope, FunctionSignature};

#[derive(EnumIter, Debug, Clone, Copy)]
pub enum SystemFunction {
    Println,
    PrintlnNum,
    Add,
    Str,
}

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
                        .map(|arg| arg.borrow_mut().to_string())
                        .collect::<Vec<String>>()
                        .join(" ")
                );
                Data::Unit
            }
            SystemFunction::PrintlnNum => {
                println!("{}", args[0].borrow_mut().number());
                Data::Unit
            }
            SystemFunction::Add => {
                Data::Number(args[0].borrow_mut().number() + args[1].borrow_mut().number())
            }
            SystemFunction::Str => Data::String(args[0].borrow_mut().to_string()),
        }
    }

    pub fn signature(&self) -> FunctionSignature {
        match self {
            SystemFunction::Println => signature!("println".into(), Data::String("".to_string())),
            SystemFunction::PrintlnNum => signature!("println".into(), Data::Number(0.)),
            SystemFunction::Add => signature!("+".into(), Data::Number(0.), Data::Number(0.)),
            SystemFunction::Str => signature!("str".into(), Data::Number(0.)),
        }
    }
}