use std::cell::RefCell;
use std::mem;
use std::rc::Rc;

use strum_macros::EnumIter;

use crate::signature;

use super::consts::{any, boolean, raw};
use super::defined::DefinedFunction;
use super::scope::{FunctionScope, FunctionSignature, SignatureArgument, VariableScope};
use super::{Argument, Data, FunctionSource};

/// A function definition that has access to the raw [`Argument`]s
/// Should be used *only* for functions that require access to the raw [`Argument`]s
#[derive(EnumIter, Debug, Clone)]
pub enum ContextFunction {
    Let,
    If,
    Cmp,
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
    ) -> Data {
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
                            .clone()
                        {
                            return Data::ControlFlow(control);
                        }
                    }
                }

                Data::Unit
            }
            ContextFunction::Cmp => Data::Boolean(args[0].data() == args[1].data()),
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
                                super::ControlFlow::Break => return Data::Unit,
                                super::ControlFlow::Continue => {
                                    continued = true;
                                    break;
                                }
                                super::ControlFlow::Return(data) => return data.borrow().clone(), // TODO: In future have functions return references
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
    }

    pub fn signature(&self) -> FunctionSignature {
        match self {
            ContextFunction::Let => signature!("let".into(), Data::Unit, false, raw(), any()),
            ContextFunction::If => signature!("if".into(), Data::Unit, true, boolean(), raw()),
            ContextFunction::Cmp => {
                signature!("==".into(), Data::Boolean(false), false, any(), any())
            }
            ContextFunction::Assign => signature!("=".into(), Data::Unit, false, raw(), any()),
            ContextFunction::While => signature!("while".into(), Data::Unit, true, raw(), raw()),
            ContextFunction::Fn => signature!("fn".into(), Data::Unit, true, raw(), raw()),
        }
    }
}

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
