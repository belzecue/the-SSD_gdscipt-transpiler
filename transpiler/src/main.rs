use std::{
    env, fs,
    process::{Command, Stdio},
};

//use chumsky::prelude::*;

mod ast;
mod lexer;
mod parser;
mod transformer;

fn main() {
    let src = fs::read_to_string("gdext-lib/src/example.gd").unwrap();

    let tokens = lexer::handwritten::Tokenizer::new(src.clone()).lex();
    dbg!(tokens);
    /*let (_tokens, errs) = lexer::lexer().parse(&src).into_output_errors();

    let _tokens = lexer::insert_deindents(_tokens).unwrap();

    let (ast, parse_errs) = parser::parser()
        .map_with(|ast, e| (ast, e.span()))
        .parse(tokens.as_slice().spanned((src.len()..src.len()).into()))
        .into_output_errors();

    dbg!(/*&tokens,*/ &errs, /*&ast,*/ &parse_errs);

    let ast: Vec<_> = ast.unwrap().0.into_iter().map(|(node, _)| node).collect();*/

    //let mut rust_generator = transformer::Generator::new();
    //let code = rust_generator.generate(ast);

    /*let code = transformer::rust_ast::get_rust_src(ast);
    let path = "gdext-lib/src/example.rs";
    fs::write(path, code).unwrap();
    env::set_var("__CARGO_FIX_YOLO", "1");
    Command::new("cargo")
        .args([
            "clippy",
            "--allow-dirty",
            "--manifest-path",
            "gdext-lib/Cargo.toml",
            "--fix",
        ])
        .stderr(Stdio::null())
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    Command::new("rustfmt")
        .args([path])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();*/
}
