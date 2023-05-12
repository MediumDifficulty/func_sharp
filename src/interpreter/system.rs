use std::io::stdin;
use std::{cell::RefCell, rc::Rc};

use once_cell::sync::Lazy;
use strum_macros::EnumIter;

use crate::signature;
use crate::util::OptionalStatic;

use super::consts::{arg_string, arg_any, return_string, return_number, return_boolean, return_unit, return_control, arg_number, arg_boolean, arg_list, return_any, return_list};
use super::scope::{FunctionScope, FunctionSignature, VariableScope};
use super::{Data, ControlFlow};

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
    Break,
    Continue,
    Return,
    Cmp,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Push,
    Pop,
    Index,
    Length,
    List,
}

impl SystemFunction {
    pub fn execute(
        &self,
        args: &[Rc<RefCell<Data>>],
        function_scope: &mut FunctionScope,
        variable_scope: Rc<RefCell<VariableScope>>,
    ) -> Rc<RefCell<Data>> {
        Rc::new(RefCell::new(
            match self {
                SystemFunction::Stdin => Data::String({
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
                },
                SystemFunction::Break => Data::ControlFlow(ControlFlow::Break),
                SystemFunction::Continue => Data::ControlFlow(ControlFlow::Continue),
                SystemFunction::Return => Data::ControlFlow(ControlFlow::Return(args[0].clone())),
                SystemFunction::Cmp => Data::Boolean(args[0].borrow().clone() == args[1].borrow().clone()),
                SystemFunction::GreaterThan => Data::Boolean(args[0].borrow().number() > args[1].borrow().number()),
                SystemFunction::GreaterThanOrEqual => Data::Boolean(args[0].borrow().number() >= args[1].borrow().number()),
                SystemFunction::LessThan => Data::Boolean(args[0].borrow().number() < args[1].borrow().number()),
                SystemFunction::LessThanOrEqual => Data::Boolean(args[0].borrow().number() <= args[1].borrow().number()),
                SystemFunction::Push => {
                    let mut iter = args.iter();
                    iter.next().unwrap().borrow_mut().list_mut().append(&mut iter.cloned().collect::<Vec<_>>());
                    Data::Unit
                },
                SystemFunction::Pop => return args[0].borrow_mut().list_mut().pop().unwrap(),
                SystemFunction::Index => return args[0].borrow_mut().list().get(args[1].borrow().number() as usize).unwrap().clone(),
                SystemFunction::Length => Data::Number(args[0].borrow_mut().list_mut().len() as f64),
                SystemFunction::List => Data::List(args.to_vec()),
            }
        ))
    }

    pub fn signature(&self) -> OptionalStatic<FunctionSignature> { // TODO: Optimise with lazy static
        match self {
            SystemFunction::Stdin => OptionalStatic::Static(&STDIN),
            SystemFunction::Number => OptionalStatic::Static(&NUMBER),
            SystemFunction::Trim => OptionalStatic::Static(&TRIM),
            SystemFunction::Not => OptionalStatic::Static(&NOT),
            SystemFunction::And => OptionalStatic::Static(&AND),
            SystemFunction::Or => OptionalStatic::Static(&OR),
            SystemFunction::Xor => OptionalStatic::Static(&XOR),
            SystemFunction::Add => OptionalStatic::Static(&ADD),
            SystemFunction::Sub => OptionalStatic::Static(&SUB),
            SystemFunction::Mul => OptionalStatic::Static(&MUL),
            SystemFunction::Div => OptionalStatic::Static(&DIV),
            SystemFunction::Mod => OptionalStatic::Static(&MOD),
            SystemFunction::Println => OptionalStatic::Static(&PRINTLN),
            SystemFunction::Break => OptionalStatic::Static(&BREAK),
            SystemFunction::Continue => OptionalStatic::Static(&CONTINUE),
            SystemFunction::Return => OptionalStatic::Static(&RETURN),
            SystemFunction::Cmp => OptionalStatic::Static(&CMP),
            SystemFunction::GreaterThan => OptionalStatic::Static(&GREATER_THAN),
            SystemFunction::GreaterThanOrEqual => OptionalStatic::Static(&GREATER_THAN_OR_EQUAL),
            SystemFunction::LessThan => OptionalStatic::Static(&LESS_THAN),
            SystemFunction::LessThanOrEqual => OptionalStatic::Static(&LESS_THAN_OR_EQUAL),
            SystemFunction::Push => OptionalStatic::Static(&PUSH),
            SystemFunction::Pop => OptionalStatic::Static(&POP),
            SystemFunction::Index => OptionalStatic::Static(&INDEX),
            SystemFunction::Length => OptionalStatic::Static(&LENGTH),
            SystemFunction::List => OptionalStatic::Static(&LIST),
        }
    }
}

static STDIN: Lazy<FunctionSignature> = Lazy::new(|| signature!("stdin".into(), return_string(), false));
static NUMBER: Lazy<FunctionSignature> = Lazy::new(|| signature!("number".into(), return_number(), false, arg_string()));
static TRIM: Lazy<FunctionSignature> = Lazy::new(|| signature!("trim".into(), return_string(), false, arg_string()));
static NOT: Lazy<FunctionSignature> = Lazy::new(|| signature!("!".into(), return_boolean(), false, arg_boolean()));
static AND: Lazy<FunctionSignature> = Lazy::new(|| signature!("&&".into(), return_boolean(), true, arg_boolean()));
static OR: Lazy<FunctionSignature> = Lazy::new(|| signature!("||".into(), return_boolean(), true, arg_boolean()));
static XOR: Lazy<FunctionSignature> = Lazy::new(|| signature!("^".into(), return_boolean(), true, arg_boolean()));
static ADD: Lazy<FunctionSignature> = Lazy::new(|| signature!("+".into(), return_number(), true, arg_number()));
static SUB: Lazy<FunctionSignature> = Lazy::new(|| signature!("-".into(), return_number(), true, arg_number()));
static MUL: Lazy<FunctionSignature> = Lazy::new(|| signature!("*".into(), return_number(), true, arg_number()));
static DIV: Lazy<FunctionSignature> = Lazy::new(|| signature!("/".into(), return_number(), true, arg_number()));
static MOD: Lazy<FunctionSignature> = Lazy::new(|| signature!("%".into(), return_number(), true, arg_number()));
static PRINTLN: Lazy<FunctionSignature> = Lazy::new(|| signature!("println".into(), return_unit(), true, arg_any()));
static BREAK: Lazy<FunctionSignature> = Lazy::new(|| signature!("break".into(), return_control(), false));
static CONTINUE: Lazy<FunctionSignature> = Lazy::new(|| signature!("continue".into(), return_control(), false));
static RETURN: Lazy<FunctionSignature> = Lazy::new(|| signature!("return".into(), return_control(), false, arg_any()));
static CMP: Lazy<FunctionSignature> = Lazy::new(|| signature!("==".into(), return_boolean(), false, arg_any(), arg_any()));
static GREATER_THAN: Lazy<FunctionSignature> = Lazy::new(|| signature!(">".into(), return_boolean(), false, arg_number(), arg_number()));
static GREATER_THAN_OR_EQUAL: Lazy<FunctionSignature> = Lazy::new(|| signature!(">=".into(), return_boolean(), false, arg_number(), arg_number()));
static LESS_THAN: Lazy<FunctionSignature> = Lazy::new(|| signature!("<".into(), return_boolean(), false, arg_number(), arg_number()));
static LESS_THAN_OR_EQUAL: Lazy<FunctionSignature> = Lazy::new(|| signature!("<=".into(), return_boolean(), false, arg_number(), arg_number()));
static PUSH: Lazy<FunctionSignature> = Lazy::new(|| signature!("push".into(), return_unit(), true, arg_list(), arg_any()));
static POP: Lazy<FunctionSignature> = Lazy::new(|| signature!("pop".into(), return_unit(), true, arg_list()));
static INDEX: Lazy<FunctionSignature> = Lazy::new(|| signature!("index".into(), return_any(), true, arg_list(), arg_number()));
static LENGTH: Lazy<FunctionSignature> = Lazy::new(|| signature!("length".into(), return_number(), true, arg_list()));
static LIST: Lazy<FunctionSignature> = Lazy::new(|| signature!("list".into(), return_list(), true, arg_any()));

fn operator_impl(operation: impl FnMut(f64, &Rc<RefCell<Data>>) -> f64, args: &[Rc<RefCell<Data>>]) -> Data {
    let mut iter = args.iter();
    iter.next()
        .map(|start| Data::Number(iter.fold(start.borrow().number(), operation)))
        .expect("Expected at least one argument")
}