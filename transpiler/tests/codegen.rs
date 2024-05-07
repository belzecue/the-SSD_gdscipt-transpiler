use gdscript_transpiler::*;

#[allow(dead_code)]
fn run_test(path: &str) {
    let src = std::fs::read_to_string(path).unwrap();
    let tokens = lexer::handwritten::Lexer::new(&src).lex();

    let ast = parser::Parser::new(&tokens).parse();

    let _rust_src = transformer::rust_ast::get_rust_src(ast);
}

macros::make_tests!();
