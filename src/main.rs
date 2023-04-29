use std::fs;

use pest::Parser;

use crate::parser::FuncParser;

extern crate pest;
#[macro_use]
extern crate pest_derive;

pub mod interpreter;
pub mod parser;

fn main() {
    let input = "test.funcs";

    let source = fs::read_to_string(input).unwrap();
    let program = match FuncParser::parse(parser::Rule::program, source.as_str()) {
        Ok(mut parsed) => parsed.next().unwrap(),
        Err(e) => {
            println!("{}", e.with_path(input));
            return;
        }
    };

    interpreter::execute(program);
}
