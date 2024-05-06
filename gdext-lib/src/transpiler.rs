use gdscript_transpiler::{lexer::Lexer, parser::Parser, transformer::rust_ast};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(init, base=Object)]
struct GDScriptTranspiler {
    base: Base<Object>,
}

#[godot_api]
impl GDScriptTranspiler {
    #[func]
    fn transpile_to_rust(&mut self, gdscript_code: String) -> String {
        let tokens = Lexer::new(&gdscript_code).lex();
        let ast = Parser::new(&tokens).parse();
        
        rust_ast::get_rust_src(ast)
    }
}
