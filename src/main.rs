use std::{fs, process::Command};

use chumsky::prelude::*;

mod ast;
mod lexer;
mod parser;
mod transformer;

fn main() {
    let mut rust_generator = transformer::Generator::new();

    let src = "
#[autocompile]
# adding line above should make addon recompile this file when it is updated

class_name Test
extends Node

func test(test_arg: int) -> int:
    var test_var := 0
    if test_arg == 5:
        test_var = 1
    return test_var

";
    
    let (tokens, errs) = lexer::lexer().parse(src).into_output_errors();

    let tokens = lexer::insert_deindents(tokens).unwrap();

    let (ast, parse_errs) = parser::parser()
        .map_with(|ast, e| (ast, e.span()))
        .parse(tokens.as_slice().spanned((src.len()..src.len()).into()))
        .into_output_errors();

    dbg!(/*&tokens,*/ &errs, /*&ast,*/ &parse_errs);
    
    let ast: Vec<_> = ast.unwrap().0.into_iter().map(|(node, _)| node).collect();

    let code = rust_generator.generate(ast);
    let path = "gdext-lib/src/example.rs";
    fs::write(path, code).unwrap();
    Command::new("cargo").args(["clippy", "--allow-dirty", "--manifest-path", "gdext-lib/Cargo.toml", "--fix"]).spawn().unwrap().wait().unwrap();
    Command::new("rustfmt").args([path]).spawn().unwrap().wait().unwrap();

}
