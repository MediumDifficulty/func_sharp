use std::io::stdin;
use std::mem;
use std::{cell::RefCell, rc::Rc};

use once_cell::sync::Lazy;
use strum_macros::EnumIter;

use crate::signature;
use crate::util::OptionalStatic;

use super::consts::{string, boolean, number, any, list};
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
        }
    }
}

static STDIN: Lazy<FunctionSignature> = Lazy::new(|| signature!("stdin".into(), Data::String("".into()), false));
static NUMBER: Lazy<FunctionSignature> = Lazy::new(|| signature!("number".into(), Data::Number(0.), false, string()));
static TRIM: Lazy<FunctionSignature> = Lazy::new(|| signature!("trim".into(), Data::String("".into()), false, string()));
static NOT: Lazy<FunctionSignature> = Lazy::new(|| signature!("!".into(), Data::Boolean(false), false, boolean()));
static AND: Lazy<FunctionSignature> = Lazy::new(|| signature!("&&".into(), Data::Boolean(false), true, boolean()));
static OR: Lazy<FunctionSignature> = Lazy::new(|| signature!("||".into(), Data::Boolean(false), true, boolean()));
static XOR: Lazy<FunctionSignature> = Lazy::new(|| signature!("^".into(), Data::Boolean(false), true, boolean()));
static ADD: Lazy<FunctionSignature> = Lazy::new(|| signature!("+".into(), Data::Number(0.), true, number()));
static SUB: Lazy<FunctionSignature> = Lazy::new(|| signature!("-".into(), Data::Number(0.), true, number()));
static MUL: Lazy<FunctionSignature> = Lazy::new(|| signature!("*".into(), Data::Number(0.), true, number()));
static DIV: Lazy<FunctionSignature> = Lazy::new(|| signature!("/".into(), Data::Number(0.), true, number()));
static MOD: Lazy<FunctionSignature> = Lazy::new(|| signature!("%".into(), Data::Number(0.), true, number()));
static PRINTLN: Lazy<FunctionSignature> = Lazy::new(|| signature!("println".into(), Data::Unit, true, any()));
static BREAK: Lazy<FunctionSignature> = Lazy::new(|| signature!("break".into(), Data::ControlFlow(ControlFlow::Break), false));
static CONTINUE: Lazy<FunctionSignature> = Lazy::new(|| signature!("continue".into(), Data::ControlFlow(ControlFlow::Break), false));
static RETURN: Lazy<FunctionSignature> = Lazy::new(|| signature!("return".into(), Data::ControlFlow(ControlFlow::Break), false, any()));
static CMP: Lazy<FunctionSignature> = Lazy::new(|| signature!("==".into(), Data::Boolean(false), false, any(), any()));
static GREATER_THAN: Lazy<FunctionSignature> = Lazy::new(|| signature!(">".into(), Data::Boolean(false), false, number(), number()));
static GREATER_THAN_OR_EQUAL: Lazy<FunctionSignature> = Lazy::new(|| signature!(">=".into(), Data::Boolean(false), false, number(), number()));
static LESS_THAN: Lazy<FunctionSignature> = Lazy::new(|| signature!("<".into(), Data::Boolean(false), false, number(), number()));
static LESS_THAN_OR_EQUAL: Lazy<FunctionSignature> = Lazy::new(|| signature!("<=".into(), Data::Boolean(false), false, number(), number()));
static PUSH: Lazy<FunctionSignature> = Lazy::new(|| signature!("push".into(), Data::Unit, true, list(), any()));
static POP: Lazy<FunctionSignature> = Lazy::new(|| signature!("pop".into(), Data::Unit, true, list()));
static INDEX: Lazy<FunctionSignature> = Lazy::new(|| signature!("index".into(), Data::Unit, true, list(), number())); // TODO:
static LENGTH: Lazy<FunctionSignature> = Lazy::new(|| signature!("length".into(), Data::Number(0.), true, list()));

fn operator_impl(operation: impl FnMut(f64, &Rc<RefCell<Data>>) -> f64, args: &[Rc<RefCell<Data>>]) -> Data {
    let mut iter = args.iter();
    iter.next()
        .map(|start| Data::Number(iter.fold(start.borrow().number(), operation)))
        .expect("Expected at least one argument")
}