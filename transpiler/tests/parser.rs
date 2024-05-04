/*use gdscript_transpiler::*;

fn run_test(path: &str) {
    let src = std::fs::read_to_string(path).unwrap();
    let tokens = lexer::handwritten::Lexer::new(&src).lex();

    let _ast = parser::Parser::new(&tokens).parse();
}

macros::make_tests!();
*/
