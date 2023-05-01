use std::{fs, time::Instant};

use pest::Parser;

use crate::parser::FuncParser;

extern crate pest;
#[macro_use]
extern crate pest_derive;

pub mod interpreter;
pub mod parser;

fn main() {
    let input = "test.funcs";

    let start_read_time = Instant::now();
    let source = fs::read_to_string(input).unwrap();
    println!("Read time: {}ms", start_read_time.elapsed().as_millis());

    let start_parse_time = Instant::now();
    let program = match FuncParser::parse(parser::Rule::program, source.as_str()) {
        Ok(mut parsed) => parsed.next().unwrap(),
        Err(e) => {
            println!("{}", e.with_path(input));
            return;
        }
    };
    println!("Parse time: {}ms", start_parse_time.elapsed().as_millis());

    let start_execution_time = Instant::now();
    interpreter::execute(program);
    println!("Execution time: {}ms", start_execution_time.elapsed().as_millis());
}
