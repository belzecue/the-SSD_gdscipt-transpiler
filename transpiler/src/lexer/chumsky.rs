use chumsky::prelude::*;

use super::Token;




/*

Operators:
     "+", "-", "*", "**", "/", "%", "~", "&", "|", "^", "<<", ">>"

Assainment:
"=", "+=", "-=", "*=", "**=", "/=", "%=", "&=", "|=", "^=", "<<=", ">>=",
where is ~= ???


True/False Comperison:
"<", ">", "!", "&&", "||", 
"<=", ">=", "!=", "=="

Control:
"(" ")" ":" "," "->"
"{" "}" ";" "."
"[" "]"


*/

const OPERATORS: [&str; 33] = [
    "+", "-", "*", "**", "/", "%", "~", "&", "|", "^", "<<", ">>", 
    "=", "+=", "-=", "*=", "**=", "/=", "%=", "&=", "|=", "^=", "<<=", ">>=",

    "<", ">", "!", "&&", "||", 
    "<=", ">=", "!=", "=="
]; // +*-/%~&|^<>=!


pub type Span = SimpleSpan<usize>;

/// NOTE:
/// 
/// This doesn't produce the final tokens for the parser.
/// 
/// You can mix tabs with 4 spaces (fix it to be 2 spaces)
/// 
/// TODO: fix op when -- is used
pub fn lexer<'src>(
) -> impl Parser<'src, &'src str, Vec<(Token, Span)>, extra::Err<Rich<'src, char, Span>>> {
    
    let num = text::int(10)
        .to_slice()
        .from_str::<u64>()
        .unwrapped()
        .map(Token::Number);

    let fp_num = text::int(10)
        .then(just('.').then(text::digits(10)))
        .to_slice()
        .from_str()
        .unwrapped()
        .map(Token::FPNumber);


    // single double quote
    let sdq_str = just('"')
        .ignore_then(none_of('\"').repeated().collect())
        .then_ignore(just('"'))
        .map(Token::String);

    // single single quote
    let ssq_str = just('\'')
        .ignore_then(none_of('\'').repeated().collect())
        .then_ignore(just('\''))
        .map(Token::String);

    // triple double quote
    let tdq_str = just("\"\"\"")
        .ignore_then(any().and_is(just("\"\"\"").not()).repeated().collect())
        .then_ignore(just("\"\"\""))
        .map(Token::String);

    // triple single quote
    let tsq_str = just("\'\'\'")
        .ignore_then(any().and_is(just("\'\'\'").not()).repeated().collect())
        .then_ignore(just("\'\'\'"))
        .map(Token::String);
        
    let str = sdq_str.or(ssq_str).or(tdq_str).or(tsq_str);

    let raw_str = just('r').ignore_then(str.clone());    

    let op = one_of("+*-/%~&|^<>=!") // FIXME: "--" becomes Op("--")
        .repeated()
        .at_least(1)
        .at_most(3)
        .to_slice()
        .filter(|s| OPERATORS.contains(s))
        .map(|s: &str| Token::Op(s.into()));

        
    let ctrl = one_of("()[]{};:,.")
        .repeated()
        .exactly(1)
        .to_slice().or(just("->"))
        .map(|s: &str| Token::Ctrl(s.into()));

    let identifier = just('_').repeated().then(text::ident().or_not())
        .to_slice()
        .filter(|s: &&str| s.len() != 0)
        .map(|s: &str| Token::Identifier(s.into()));

    let anotation = just('@').ignore_then(identifier)
        .to_slice()
        .map(|s: &str| Token::Anotation(s[1..].into()));

    /*let get_node = just('$').ignore_then(identifier)
        .to_slice()
        .map(|s: &str| Token::Anotation(s[1..].into()));*/


        
    let newline = text::newline()
        .to(Token::NewLine);

    
    let indent = just("\t").or(just("    ")).or(just("  ")) // tab or 4 or 2 spaces
        .to(Token::Ident);


    let token = identifier
        .or(ctrl)
        .or(op)
        .or(fp_num)
        .or(num)
        .or(str)
        .or(raw_str)
        .or(anotation);

    let comment = just("#")
        //.then(any().and_is(just('\n').not()).repeated())
        .then(none_of('\n').repeated())
        .padded_by(text::inline_whitespace());

        
    token
        .then_ignore(just(" ").repeated())
        .or(newline)
        .or(indent
        // (optional) skip at most 3 spases if not followed by space
        .then_ignore(
            just(' ').repeated().at_most(3).then(just(' ').not().rewind()).or_not()
        ))
        
        .padded_by(comment.repeated())
        
        .map_with(|tok, e| (tok, e.span()))

        .recover_with(skip_then_retry_until(any().ignored(), end()))
        .repeated()
        .collect()


}

pub fn insert_deindents(input: Option<Vec<(Token, Span)>>) -> Option<Vec<(Token, SimpleSpan)>> {
    let input = if let Some(thing) = input {
        thing
    } else {
        return None;
    };


    let mut new_input = Vec::new();
    new_input.reserve(input.len());

    let mut indent_count = 0;
    let mut new_indent_count = -1;
    let mut prev_token = (Token::NewLine, SimpleSpan::new(0, 0));

    for (i, (token, span)) in input.clone().into_iter().enumerate() {
        if new_indent_count != -1 && token != Token::Ident {
            /*if new_indent_count != indent_count {
                println!("TEST: {:?} ({:?} -> {:?})", prev_token, indent_count, new_indent_count);
            }*/
            while new_indent_count != indent_count{
                if new_indent_count > indent_count {
                    new_input.push((Token::Ident, prev_token.1));
                    indent_count += 1;
                } else if new_indent_count < indent_count {
                    new_input.push((Token::DeIdent, prev_token.1));
                    indent_count -= 1;
                }
            }

            new_indent_count = -1;
        }

        if token == Token::NewLine {
            if let Some((Token::NewLine, _)) = input.get(i + 1) {
                new_indent_count = -1;
            } else  {
                new_input.push((token.clone(), span));
                new_indent_count = 0;
            }

        }

        else if token == Token::Ident {
            new_indent_count += 1;
        } else {
            new_input.push((token.clone(), span));
        }


        prev_token = (token, span);
    }

    while indent_count > 0 {
        indent_count -= 1;
        new_input.push((Token::DeIdent, prev_token.1));
    }

    Some(new_input)
}
