mod consts;
mod context;
mod defined;
mod scope;
mod system;

use std::{cell::RefCell, mem::Discriminant, rc::Rc};

use pest::iterators::Pair;
use std::mem;

use crate::{parser, util::OptionalStatic};

use self::{
    context::ContextFunction,
    defined::DefinedFunction,
    scope::{FunctionScope, FunctionSignature, VariableScope, ReturnType},
    system::SystemFunction,
};

/// The different sources for a function
#[derive(Debug, Clone)]
pub enum FunctionSource {
    System(SystemFunction),
    Context(ContextFunction),
    Defined(DefinedFunction),
}

/// The invocation of a function (contains the name and raw [Argument]s)
#[derive(Debug, Default, Clone)]
pub struct Invocation {
    name: String,
    args: Vec<Argument>,
}

/// Any data that can be stored
#[derive(Debug, Clone, PartialEq)]
pub enum Data {
    String(String),
    Number(f64),
    Boolean(bool),
    ControlFlow(ControlFlow),
    List(Vec<Rc<RefCell<Data>>>),
    Unit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ControlFlow {
    Break,
    Continue,
    Return(Rc<RefCell<Data>>),
}

/// The raw argument
#[derive(Debug, Clone)]
pub enum Argument {
    Function(Invocation),
    Data(Data),
    Ident(String),
}

pub fn execute(program: Vec<Invocation>) {
    let mut function_scope = FunctionScope::default();
    let variable_scope = Rc::new(RefCell::new(scope::default_variable_scope()));

    for invocation in program.iter() {
        invocation.evaluate(&mut function_scope, variable_scope.clone(), variable_scope.clone());
    }
}

#[macro_export]
macro_rules! signature {
    ($name:expr, $return_type:expr, $repeating:expr, $($arg:expr),+) => {
        FunctionSignature {
            name: $name,
            return_type: $return_type,
            repeating: $repeating,
            args: vec![$($arg),+],
        }
    };
    ($name:expr, $return_type:expr, $repeating:expr) => {
        FunctionSignature {
            name: $name,
            return_type: $return_type,
            repeating: $repeating,
            args: Vec::new(),
        }
    };
}

impl Invocation {
    pub fn evaluate(
        &self,
        function_scope: &mut FunctionScope,
        variable_scope: Rc<RefCell<VariableScope>>,
        global_scope: Rc<RefCell<VariableScope>>,
    ) -> Rc<RefCell<Data>> {
        let got = function_scope
            .get(
                &self.name,
                &self.args,
                function_scope,
                variable_scope.clone(),
            )
            .cloned();

        if let Some(function) = got {
            function.execute(&self.args, function_scope, variable_scope, global_scope)
        } else {
            panic!(
                "Function not found: {} with signature: {:?}",
                self.name, self.args
            );
        }
    }
}

impl From<Pair<'_, parser::Rule>> for Invocation {
    fn from(value: Pair<'_, parser::Rule>) -> Self {
        let mut inner = value.into_inner();

        let name = inner.next().unwrap().as_str().to_string();
        let args = inner.map(Argument::from).collect();

        Self { name, args }
    }
}

impl Argument {
    pub fn eval(
        &self,
        function_scope: &mut FunctionScope,
        variable_scope: Rc<RefCell<VariableScope>>,
        global_scope: Rc<RefCell<VariableScope>>,
    ) -> Rc<RefCell<Data>> {
        match self {
            Argument::Data(data) => Rc::new(RefCell::new(data.clone())),
            Argument::Function(invocation) => invocation.evaluate(
                function_scope,
                variable_scope,
                global_scope,
            ),
            Argument::Ident(ident) => variable_scope
                .borrow()
                .get(ident)
                .expect("Variable not found")
                .clone(),
        }
    }

    pub fn return_type(
        &self,
        function_scope: &FunctionScope,
        variable_scope: Rc<RefCell<VariableScope>>,
    ) -> ReturnType {
        match self {
            Argument::Function(func) => {
                function_scope
                    .get(&func.name, &func.args, function_scope, variable_scope)
                    .unwrap_or_else(|| panic!("Function not found: {}", func.name))
                    .signature()
                    .get_ref()
                    .return_type
                    .clone()
            }
            Argument::Data(data) => ReturnType::Data(mem::discriminant(data)),
            Argument::Ident(ident) => ReturnType::Data(mem::discriminant(
                &*variable_scope
                    .borrow()
                    .get(ident)
                    .expect("Variable not found")
                    .borrow(),
            )),
        }
    }

    pub fn ident(&self) -> String {
        match self {
            Argument::Ident(i) => i.clone(),
            _ => panic!("Argument is not an ident"),
        }
    }

    pub fn invocation(&self) -> Invocation {
        match self {
            Argument::Function(f) => f.clone(),
            _ => panic!("Argument is not a function"),
        }
    }
}

impl From<Pair<'_, parser::Rule>> for Argument {
    fn from(value: Pair<'_, parser::Rule>) -> Self {
        let value = value.into_inner().next().unwrap();
        match value.as_rule() {
            parser::Rule::string => Argument::Data(Data::String(
                value.into_inner().next().unwrap().as_str().to_string(),
            )),
            parser::Rule::number => Argument::Data(Data::Number(value.as_str().parse().unwrap())),
            parser::Rule::invocation => Argument::Function(value.into()),
            parser::Rule::ident => Argument::Ident(value.as_str().to_string()),
            _ => unreachable!(),
        }
    }
}

impl Data {
    fn number(&self) -> f64 {
        match self {
            Data::Number(n) => *n,
            _ => panic!("Data is not a number"),
        }
    }

    fn boolean(&self) -> bool {
        match self {
            Data::Boolean(b) => *b,
            _ => panic!("Data is not a boolean"),
        }
    }

    fn list(&self) -> Vec<Rc<RefCell<Data>>> {
        match self{
            Data::List(l) => l.clone(),
            _ => panic!("Data is not a list"),
        }
    }

    fn list_mut(&mut self) -> &mut Vec<Rc<RefCell<Data>>> {
        match self{
            Data::List(l) => l,
            _ => panic!("Data is not a list"),
        }
    }
}

impl ToString for Data {
    fn to_string(&self) -> String {
        match self {
            Data::String(s) => s.clone(),
            Data::Number(n) => n.to_string(),
            Data::Boolean(b) => b.to_string(),
            Data::ControlFlow(c) => c.to_string(),
            Data::List(l) => format!("[{}]", l.iter().map(|d| d.borrow().to_string()).collect::<Vec<_>>().join(", ")),
            Data::Unit => "()".to_string(),
        }
    }
}

impl ToString for ControlFlow {
    fn to_string(&self) -> String {
        match self {
            ControlFlow::Break => "break".into(),
            ControlFlow::Continue => "continue".into(),
            ControlFlow::Return(_) => "return".into(),
        }
    }
}

impl FunctionSource {
    fn signature(&self) -> OptionalStatic<FunctionSignature> {
        match self {
            FunctionSource::System(func) => func.signature(),
            FunctionSource::Context(func) => func.signature(),
            FunctionSource::Defined(func) => OptionalStatic::Owned(func.signature()),
        }
    }

    pub fn execute(
        &self,
        args: &[Argument],
        function_scope: &mut FunctionScope,
        variable_scope: Rc<RefCell<VariableScope>>,
        global_scope: Rc<RefCell<VariableScope>>,
    ) -> Rc<RefCell<Data>> {
        match self {
            FunctionSource::System(func) => func.execute(
                &args
                    .iter()
                    .map(|arg| {
                        arg.eval(function_scope, variable_scope.clone(), global_scope.clone())
                    })
                    .collect::<Vec<_>>(),
                function_scope,
                variable_scope,
            ),
            FunctionSource::Context(func) => func.execute(
                &context::to_context_args(
                    args,
                    func.signature().get_ref(),
                    function_scope,
                    variable_scope.clone(),
                    global_scope.clone(),
                ),
                function_scope,
                variable_scope,
                global_scope,
            ),
            FunctionSource::Defined(func) => func.execute(
                &args
                    .iter()
                    .map(|arg| {
                        arg.eval(function_scope, variable_scope.clone(), global_scope.clone())
                    })
                    .collect::<Vec<_>>(),
                function_scope,
                global_scope,
            ),
        }
    }
}
