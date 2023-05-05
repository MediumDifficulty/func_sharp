use std::io::stdin;
use std::mem;
use std::{cell::RefCell, rc::Rc};

use strum_macros::EnumIter;

use crate::signature;

use super::consts::{string, boolean, number, any};
use super::scope::{FunctionScope, FunctionSignature, VariableScope};
use super::Data;

/// A function that has the same power as a user defined function but is hard-coded.
/// This means it does not have access to the raw [`Argument`](super::Argument) but rather the parsed [`Data`]
#[derive(EnumIter, Debug, Clone, Copy)]
pub enum SystemFunction {
    Stdin,
    Number,
    Trim,
    Not,
    And,
    Or,
    Xor,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Println,
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
            SystemFunction::Not => Data::Boolean(!args[0].borrow().boolean()),
            SystemFunction::And => Data::Boolean(args.iter().all(|arg| arg.borrow().boolean())),
            SystemFunction::Or => Data::Boolean(args.iter().any(|arg| arg.borrow().boolean())),
            SystemFunction::Xor => Data::Boolean(args.iter().fold(0, |acc, arg| acc + arg.borrow().boolean() as usize) == 1),
            SystemFunction::Add => operator_impl(|acc, arg| acc + arg.borrow().number(), args),
            SystemFunction::Sub => operator_impl(|acc, arg| acc - arg.borrow().number(), args),
            SystemFunction::Mul => operator_impl(|acc, arg| acc * arg.borrow().number(), args),
            SystemFunction::Div => operator_impl(|acc, arg| acc / arg.borrow().number(), args),
            SystemFunction::Mod => operator_impl(|acc, arg| acc % arg.borrow().number(), args),
            SystemFunction::Println => {
                println!("{}", args.iter().map(|e| e.borrow().to_string()).collect::<Vec<_>>().join(" "));
                Data::Unit
            }
        }
    }

    pub fn signature(&self) -> FunctionSignature {
        match self {
            SystemFunction::Stdin => signature!("stdin".into(), Data::String("".into()), false),
            SystemFunction::Number => signature!("number".into(), Data::Number(0.), false, string()),
            SystemFunction::Trim => signature!("trim".into(), Data::String("".into()), false, string()),
            SystemFunction::Not => signature!("!".into(), Data::Boolean(false), false, boolean()),
            SystemFunction::And => signature!("&&".into(), Data::Boolean(false), true, boolean()),
            SystemFunction::Or => signature!("||".into(), Data::Boolean(false), true, boolean()),
            SystemFunction::Xor => signature!("^".into(), Data::Boolean(false), true, boolean()),
            SystemFunction::Add => signature!("+".into(), Data::Number(0.), true, number()),
            SystemFunction::Sub => signature!("-".into(), Data::Number(0.), true, number()),
            SystemFunction::Mul => signature!("*".into(), Data::Number(0.), true, number()),
            SystemFunction::Div => signature!("/".into(), Data::Number(0.), true, number()),
            SystemFunction::Mod => signature!("%".into(), Data::Number(0.), true, number()),
            SystemFunction::Println => signature!("println".into(), Data::Unit, true, any()),
        }
    }
}

fn operator_impl(operation: impl FnMut(f64, &Rc<RefCell<Data>>) -> f64, args: &[Rc<RefCell<Data>>]) -> Data {
    let mut iter = args.iter();
    iter.next()
        .map(|start| Data::Number(iter.fold(start.borrow().number(), operation)))
        .expect("Expected at least one argument")
}