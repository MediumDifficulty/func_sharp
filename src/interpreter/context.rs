use std::cell::RefCell;
use std::mem;
use std::rc::Rc;

use once_cell::sync::Lazy;
use strum_macros::EnumIter;

use crate::signature;
use crate::util::OptionalStatic;

use super::consts::{arg_any, arg_raw, return_unit, arg_boolean};
use super::defined::DefinedFunction;
use super::scope::{FunctionScope, FunctionSignature, SignatureArgument, VariableScope};
use super::{Argument, Data, FunctionSource, ControlFlow};

/// A function definition that has access to the raw [`Argument`]s
/// Should be used *only* for functions that require access to the raw [`Argument`]s
#[derive(EnumIter, Debug, Clone)]
pub enum ContextFunction {
    Let,
    If,
    Assign,
    While,
    Fn,
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
            _ => panic!("Argument is not data"),
        }
    }

    fn raw(&self) -> &Argument {
        match self {
            Self::Raw(raw) => raw,
            _ => panic!("Argument is not raw"),
        }
    }
}

impl ContextFunction {
    pub fn execute(
        &self,
        args: &[ContextArgument],
        function_scope: &mut FunctionScope,
        variable_scope: Rc<RefCell<VariableScope>>,
        global_scope: Rc<RefCell<VariableScope>>,
    ) -> Rc<RefCell<Data>> {
        Rc::new(RefCell::new(
            match self {
                ContextFunction::Let => {
                    let evaluated = args[1].data();
                    variable_scope
                        .borrow_mut()
                        .insert(args[0].raw().ident(), evaluated);
                    Data::Unit
                }
                ContextFunction::If => {
                    let mut iter = args.iter();
                    if iter.next().unwrap().data().borrow().boolean() {
                        let cloned_scope = Rc::new(RefCell::new(variable_scope.borrow().clone()));
                        for invocation in iter {
                            if let Data::ControlFlow(control) = invocation
                                .raw()
                                .eval(function_scope, cloned_scope.clone(), global_scope.clone())
                                .borrow()
                                .clone() {
                                return Rc::new(RefCell::new(Data::ControlFlow(control)));
                            }
                        }
                    }
    
                    Data::Unit
                }
                ContextFunction::Assign => {
                    *variable_scope
                        .borrow()
                        .get(args[0].raw().ident().as_str())
                        .expect("Variable not found")
                        .borrow_mut() = args[1].data().borrow().clone();
                    Data::Unit
                }
                ContextFunction::While => {
                    let mut iter = args.iter();
                    let predicate = iter.next().unwrap().raw().clone();
                    let body = iter.map(|e| e.raw()).collect::<Vec<_>>();
    
                    let mut continued = false;
    
                    while predicate
                        .eval(function_scope, variable_scope.clone(), global_scope.clone())
                        .borrow()
                        .boolean()
                    {
                        if continued {
                            continued = false;
                            continue;
                        }
    
                        for &invocation in body.iter() {
                            if let Data::ControlFlow(control) = invocation
                                .eval(function_scope, variable_scope.clone(), global_scope.clone())
                                .borrow()
                                .clone()
                            {
                                match control {
                                    ControlFlow::Break => return Rc::new(RefCell::new(Data::Unit)),
                                    ControlFlow::Continue => {
                                        continued = true;
                                        break;
                                    }
                                    ControlFlow::Return(_) => return Rc::new(RefCell::new(Data::ControlFlow(control))),
                                }
                            }
                        }
                    }
    
                    Data::Unit
                }
                ContextFunction::Fn => {
                    function_scope.insert(FunctionSource::Defined(DefinedFunction::new(
                        &args
                            .iter()
                            .map(|arg| arg.raw())
                            .cloned()
                            .collect::<Vec<_>>(),
                        global_scope,
                    )));
                    Data::Unit
                }
            }
        ))
    }

    pub fn signature(&self) -> OptionalStatic<FunctionSignature> {
        match self {
            ContextFunction::Let => OptionalStatic::Static(&LET),
            ContextFunction::If => OptionalStatic::Static(&IF),
            ContextFunction::Assign => OptionalStatic::Static(&ASSIGN),
            ContextFunction::While => OptionalStatic::Static(&WHILE),
            ContextFunction::Fn => OptionalStatic::Static(&FN),
        }
    }
}

static LET: Lazy<FunctionSignature> = Lazy::new(|| signature!("let".into(), return_unit(), false, arg_raw(), arg_any()));
static IF: Lazy<FunctionSignature> = Lazy::new(|| signature!("if".into(), return_unit(), true, arg_boolean(), arg_raw()));
static ASSIGN: Lazy<FunctionSignature> = Lazy::new(|| signature!("=".into(), return_unit(), false, arg_raw(), arg_any()));
static WHILE: Lazy<FunctionSignature> = Lazy::new(|| signature!("while".into(), return_unit(), true, arg_raw(), arg_raw()));
static FN: Lazy<FunctionSignature> = Lazy::new(|| signature!("fn".into(), return_unit(), true, arg_raw(), arg_raw()));

pub fn to_context_args(
    args: &[Argument],
    signature: &FunctionSignature,
    function_scope: &mut FunctionScope,
    variable_scope: Rc<RefCell<VariableScope>>,
    global_scope: Rc<RefCell<VariableScope>>,
) -> Vec<ContextArgument> {
    args.iter()
        .enumerate()
        .map(
            |(i, arg)| match signature.args[i.min(signature.args.len() - 1)] {
                SignatureArgument::Any => ContextArgument::Data(arg.eval(
                    function_scope,
                    variable_scope.clone(),
                    global_scope.clone(),
                )),
                SignatureArgument::Raw => ContextArgument::Raw(arg.clone()),
                SignatureArgument::Data(_) => ContextArgument::Data(arg.eval(
                    function_scope,
                    variable_scope.clone(),
                    global_scope.clone(),
                )),
            },
        )
        .collect()
}
