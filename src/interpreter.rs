use std::{cell::RefCell, collections::HashMap, mem::Discriminant, rc::Rc};

use pest::iterators::Pair;
use std::mem;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::parser;

type FunctionScope = HashMap<FunctionSignature, FunctionDefinition>;
type VariableScope = HashMap<String, Rc<RefCell<Data>>>;

#[derive(Debug, PartialEq, Eq, Hash)]
struct FunctionSignature {
    name: String,
    args: Vec<Discriminant<Data>>,
}

#[derive(Debug, Clone, Copy)]
enum FunctionDefinition {
    System(SystemFunction),
}

#[derive(EnumIter, Debug, Clone, Copy)]
enum SystemFunction {
    Println,
    PrintlnNum,
    Add,
    Str,
}

enum ContextFunction {
    Let,
}

#[derive(Debug)]
struct Invocation {
    name: String,
    args: Vec<Argument>,
}

#[derive(Debug, Clone)]
enum Data {
    String(String),
    Number(f64),
    Boolean(bool),
    Ident(String),
    Unit,
}

#[derive(Debug)]
enum Argument {
    Function(Invocation),
    Data(Data),
    Ident(String),
}

pub fn execute(program: Pair<parser::Rule>) {
    let mut function_scope = generate_default_scope();
    let mut variable_scope = HashMap::new();

    for invocation in program.into_inner() {
        match invocation.as_rule() {
            parser::Rule::invocation => {
                execute_function(invocation, &mut function_scope, &mut variable_scope)
            }
            parser::Rule::EOI => (),
            _ => unreachable!(),
        }
    }
}

fn execute_function(
    function: Pair<parser::Rule>,
    function_scope: &mut FunctionScope,
    variable_scope: &mut VariableScope,
) {
    let parsed = Invocation::from(function);

    parsed.evaluate(function_scope, variable_scope);
    // println!("{:?}", parsed);
}

fn generate_default_scope() -> FunctionScope {
    let mut scope = HashMap::new();

    for function in SystemFunction::iter() {
        scope.insert(function.signature(), FunctionDefinition::System(function));
    }

    scope
}

macro_rules! signature {
    ($name:expr, $($arg:expr),+) => {
        FunctionSignature {
            name: $name,
            args: vec![$(mem::discriminant(&$arg)),+],
        }
    };
}

impl SystemFunction {
    fn execute(
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

    fn signature(&self) -> FunctionSignature {
        match self {
            SystemFunction::Println => signature!("println".into(), Data::String("".to_string())),
            SystemFunction::PrintlnNum => signature!("println".into(), Data::Number(0.)),
            SystemFunction::Add => signature!("+".into(), Data::Number(0.), Data::Number(0.)),
            SystemFunction::Str => signature!("str".into(), Data::Number(0.)),
        }
    }
}

impl ContextFunction {
    fn execute(
        &self,
        args: &[Argument],
        function_scope: &mut FunctionScope,
        variable_scope: &mut VariableScope,
    ) {
        match self {
            ContextFunction::Let => {
                
            },
        }
    }
}

impl Invocation {
    pub fn evaluate(&self, function_scope: &mut FunctionScope, variable_scope: &mut VariableScope) -> Data {
        let args = self
            .args
            .iter()
            .map(|arg| arg.eval(function_scope, variable_scope))
            .collect::<Vec<_>>();
        let args_signature = args
            .iter()
            .map(|arg| mem::discriminant(&*arg.borrow_mut()))
            .collect::<Vec<_>>(); // FIXME: this is bad

        let signature = FunctionSignature {
            name: self.name.clone(),
            args: args_signature,
        };

        // println!("{}", self.name);
        match *function_scope.get(&signature).expect("Function not found") {
            FunctionDefinition::System(function) => function.execute(
                &self
                    .args
                    .iter()
                    .map(|e| e.eval(function_scope, variable_scope))
                    .collect::<Vec<Rc<RefCell<Data>>>>(),
                function_scope,
                variable_scope,
            ),
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
        variable_scope: &mut VariableScope,
    ) -> Rc<RefCell<Data>> {
        match self {
            Argument::Data(data) => Rc::new(RefCell::new(data.clone())),
            Argument::Function(invocation) => Rc::new(RefCell::new(
                invocation.evaluate(function_scope, variable_scope),
            )),
            Argument::Ident(ident) => match ident.as_str() {
                "true" => Rc::new(RefCell::new(Data::Boolean(true))),
                "false" => Rc::new(RefCell::new(Data::Boolean(false))),
                _ => variable_scope
                    .get(ident)
                    .expect("Variable not found")
                    .clone(),
            },
        }
    }
}

impl From<Pair<'_, parser::Rule>> for Argument {
    fn from(value: Pair<'_, parser::Rule>) -> Self {
        let value = value.into_inner().next().unwrap();
        // println!("{value:?} {}", value.as_str());
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
}

impl ToString for Data {
    fn to_string(&self) -> String {
        match self {
            Data::String(s) => s.clone(),
            Data::Number(n) => n.to_string(),
            Data::Boolean(b) => b.to_string(),
            Data::Unit => "()".to_string(),
            Data::Ident(s) => s.clone(),
        }
    }
}