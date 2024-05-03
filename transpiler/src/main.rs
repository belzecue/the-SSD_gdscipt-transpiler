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

    let tokens = lexer::Lexer::new(&src).lex();
    dbg!(&tokens);

    let ast = parser::Parser::new(&tokens).parse();
    dbg!(&ast);

    let rust_src = transformer::rust_ast::get_rust_src(ast);

    /*let path = "gdext-lib/src/example.rs";
    fs::write(path, rust_src).unwrap();

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
