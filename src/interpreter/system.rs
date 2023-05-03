use std::{cell::RefCell, rc::Rc};
use std::mem;

use strum_macros::EnumIter;

use crate::signature;

use super::consts::number;
use super::scope::{FunctionScope, VariableScope, FunctionSignature};
use super::Data;

#[derive(EnumIter, Debug, Clone, Copy)]
pub enum SystemFunction {
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
            SystemFunction::Add => {
                Data::Number(args[0].borrow().number() + args[1].borrow().number())
            }
            SystemFunction::Str => Data::String(args[0].borrow().to_string()),
        }
    }

    pub fn signature(&self) -> FunctionSignature {
        match self {
            SystemFunction::Add => signature!("+".into(), Data::Number(0.), false, number(), number()),
            SystemFunction::Str => signature!("str".into(), Data::String("".into()), false, number()),
        }
    }
}