use std::{fs, time::Instant, mem};

use func_sharp::{parser::{FuncParser, self}, interpreter::{Invocation, self}};
use pest::Parser;

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

    let start_execution_time = Instant::now();
    interpreter::execute(program);
    println!(
        "Execution time: {}ms",
        start_execution_time.elapsed().as_millis()
    );
}
