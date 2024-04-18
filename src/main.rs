use std::{fs, process::Command};

use chumsky::prelude::*;
use generator::Generator;

mod ast;
mod lexer;
mod parser;
mod generator;

fn main() {
    let mut rust_generator = Generator::new();

    let src = "
#[autocompile]
# adding line above should make addon recompile this file when it is updated

class_name Test
extends Node

func test(test_arg: int) -> int:
    var test_var := 0
    test_var = test_arg
    return test_var

";
    
    let (tokens, errs) = lexer::lexer().parse(src).into_output_errors();

    let tokens = lexer::insert_deindents(tokens).unwrap();

    let (ast, parse_errs) = parser::parser()
        .map_with(|ast, e| (ast, e.span()))
        .parse(tokens.as_slice().spanned((src.len()..src.len()).into()))
        .into_output_errors();

    dbg!(/*&tokens,*/ &errs, /*&ast,*/ &parse_errs);
    
    let ast = ast.unwrap().0.into_iter().map(|(node, _)| node).collect();

    let code = rust_generator.generate(ast);
    fs::write("gdext-lib/src/example.rs", code).unwrap();
    Command::new("rustfmt").args(["example.rs"]).spawn().unwrap().wait().unwrap();
}
