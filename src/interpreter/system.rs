use std::io::stdin;
use std::mem;
use std::{cell::RefCell, rc::Rc};

use strum_macros::EnumIter;

use crate::signature;

use super::consts::{number, string};
use super::scope::{FunctionScope, FunctionSignature, VariableScope};
use super::Data;

#[derive(EnumIter, Debug, Clone, Copy)]
pub enum SystemFunction {
    Stdin,
    Number,
    Trim,
}

impl SystemFunction {
    pub fn execute(
        &self,
        args: &[Rc<RefCell<Data>>],
        function_scope: &mut FunctionScope,
        variable_scope: &mut VariableScope,
    ) -> Data {
        match self {
            Self::Stdin => Data::String({
                let mut string = String::new();
                stdin().read_line(&mut string).unwrap();
                string
            }),
            SystemFunction::Number => Data::Number(args[0].borrow().to_string().parse::<f64>().unwrap()),
            SystemFunction::Trim => Data::String(args[0].borrow().to_string().trim().into()),
        }
    }

    pub fn signature(&self) -> FunctionSignature {
        match self {
            SystemFunction::Stdin => signature!("stdin".into(), Data::String("".into()), false),
            SystemFunction::Number => signature!("number".into(), Data::Number(0.), false, string()),
            Self::Trim => signature!("trim".into(), Data::String("".into()), false, string()),
        }
    }
}
