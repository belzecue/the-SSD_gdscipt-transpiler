use proc_macro::TokenStream;

use quote::{format_ident, quote, TokenStreamExt};
use std::fs;

#[proc_macro]
pub fn make_tests(_itput: TokenStream) -> TokenStream {
    let paths = fs::read_dir("transpiler/tests/godot-tests/tests").unwrap();
    let paths = paths.filter(|p| p.as_ref().unwrap().path().extension().unwrap() == "gd");
    let paths = paths.map(|p| p.unwrap());

    let mut output = quote! {
        use gdscript_transpiler::*;
        //use chumsky::prelude::*;

        fn run_test(path: &str) {
            let src = std::fs::read_to_string(path).unwrap();
            let tokens = lexer::handwritten::Tokenizer::new(src.clone()).lex();

            /*let (tokens, errs) = lexer::lexer().parse(src.as_str()).into_output_errors();

            let tokens = lexer::insert_deindents(tokens).unwrap();

            assert!(errs.len() == 0);*/

            /*let (ast, parse_errs) = parser::parser()
                .parse(tokens.as_slice().spanned((src.len()..src.len()).into()))
                .into_output_errors();

            assert!(parse_errs.len() == 0);*/

            //let ast = ast.unwrap().into_iter().map(|(node, _)| node).collect();

            /*let context = inkwell::context::Context::create();
            let mut compiler = Generator::new(&context, inkwell::OptimizationLevel::None);

            compiler.compile(&ast);*/
        }
    };

    let keywords = ["while", "super", "enum", "in", "match"];

    for path in paths {
        let path = path.path();
        let mut name = path.file_stem().unwrap().to_str().unwrap().to_owned();
        if keywords.contains(&name.as_str()) {
            name.push('_');
        }

        let func_name = format_ident!("{}", name);
        let path = path
            .strip_prefix("transpiler/")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let func = quote! {
            #[test]
            fn #func_name() {
                println!("{}", #path);
                run_test(#path);
            }
        };

        output.append_all(func);
    }

    output.into()
}
