//use proc_macro::TokenStream;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, TokenStreamExt};
use std::fs;

#[proc_macro]
pub fn make_tests(_itput: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let paths = fs::read_dir("transpiler/tests/godot-tests/tests").unwrap();
    let paths = paths.filter(|p| p.as_ref().unwrap().path().extension().unwrap() == "gd");
    let paths = paths.map(|p| p.unwrap());

    let mut output = TokenStream::new();

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
