use std::{fs, time::Instant};

use pest::Parser;

use crate::{parser::FuncParser, interpreter::Invocation};

extern crate pest;
#[macro_use]
extern crate pest_derive;

pub mod interpreter;
pub mod parser;
pub mod util;

fn main() {
    let input = "prime.funcs";

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
    }.into_inner().filter_map(|pair| match pair.as_rule() {
        parser::Rule::invocation => Some(Invocation::from(pair)),
        parser::Rule::EOI => None,
        _ => unreachable!()
    }).collect::<Vec<_>>();
    println!("Parse time: {}ms", start_parse_time.elapsed().as_millis());

    // TODO: fs::write("test.funb",  bytemuck::bytes_of(&program));

    let start_execution_time = Instant::now();
    interpreter::execute(program);
    println!(
        "Execution time: {}ms",
        start_execution_time.elapsed().as_millis()
    );
}
