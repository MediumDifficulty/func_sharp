use criterion::{Criterion, criterion_group, criterion_main};
use func_sharp::{interpreter::Invocation, parser::{FuncParser, self}};
use pest::Parser;

static PRIME_SRC: &str = include_str!("prime.funcs");

fn prime_benchmark(c: &mut Criterion) {
    c.bench_function("prime 10000", |b| b.iter(|| func_sharp::interpreter::execute(parse(PRIME_SRC))));
}

fn parse(src: &str) -> Vec<Invocation> {
    FuncParser::parse(parser::Rule::program, src).unwrap().next().unwrap().into_inner().filter_map(|pair| match pair.as_rule() {
        parser::Rule::invocation => Some(Invocation::from(pair)),
        parser::Rule::EOI => None,
        _ => unreachable!()
    }).collect::<Vec<_>>()
}

criterion_group!(benches, prime_benchmark);
criterion_main!(benches);