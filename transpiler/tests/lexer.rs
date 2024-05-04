use gdscript_transpiler::*;

fn run_test(path: &str) {
    let src = std::fs::read_to_string(path).unwrap();
    let _tokens = lexer::handwritten::Lexer::new(&src).lex();
}

macros::make_tests!();
