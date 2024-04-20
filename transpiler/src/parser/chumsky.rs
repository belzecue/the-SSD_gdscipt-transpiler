use chumsky::prelude::*;
use chumsky::pratt::*;

use crate::ast::Iterator;
use crate::ast::{Expr, Node, TLNode, Variable};
use crate::lexer::Span;
use crate::lexer::Token;

pub type Spanned<T> = (T, Span);
type ParserInput<'a> =
    chumsky::input::SpannedInput<Token, Span, &'a [(Token, Span)]>;



// I am going to use macros everywere I can
macro_rules! token {
    ($t: ident, $str: expr) => {
        just(Token::$t($str.into()))
    };
    ($t: ident) => {
        just(Token::$t)
    };
}

macro_rules! pratt_infix_left {
    ($power: expr, $str: literal) => {
        infix(left($power), token!(Op, $str), |x, y| Expr::Op { lhs: Box::new(x), op: $str.to_string(), rhs: Box::new(y) })
    };
}


/// TODO: Annotation
pub fn parser<'a>()-> impl Parser<
    'a,
    ParserInput<'a>,
    Vec<Spanned<TLNode>>,
    extra::Err<Rich<'a, Token, Span>>,
    > + Clone 
{
    let ident = select! { Token::Identifier(ident) => ident };
    let op = select! { Token::Op(str) => str };

    /*
    let str = select! { Token::String(str) => str };
    */

    let expr = recursive( | exrp | {

        let num = select! { Token::Number(num) => num }.map(Expr::Number);
        let fp_num = select! { Token::FPNumber(num) => num }.map(Expr::FPNumber);

        let var = ident.map(Expr::Variable);

        let call = ident
            .then_ignore(token!(Ctrl, "("))
            .then(exrp.clone()
                .separated_by(token!(Ctrl, ",")).collect()
                //.delimited_by(token!(Ctrl, "("), token!(Ctrl, ")")))
            ).then_ignore(token!(Ctrl, ")"))
        .map(| x | Expr::Call(x.0, x.1));

        let brackets = 
        token!(Ctrl, "(")
        .ignore_then(exrp.clone())
        .then_ignore(token!(Ctrl, ")"));

        
        let expr = num
        .or(fp_num)
        .or(call)
        .or(var) // has to be after call
        .or(brackets);

        expr.pratt((
            prefix(3, token!(Op, "-"), |e| Expr::Neg(Box::new(e))),
            pratt_infix_left!(2, "*"),
            pratt_infix_left!(2, "/"),
            pratt_infix_left!(1, "+"),
            pratt_infix_left!(1, "-"),
            pratt_infix_left!(0, "=="),
            pratt_infix_left!(0, "!="),
            pratt_infix_left!(0, "<"),
            pratt_infix_left!(0, ">"),
        ))
    });


/*

let product = unary.clone()
    .then(op('*').to(Expr::Mul as fn(_, _) -> _)
        .or(op('/').to(Expr::Div as fn(_, _) -> _))
        .then(unary)
        .repeated())
    .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)));
*/
    


    let node = recursive( | node | {

        let body = token!(NewLine)
        .ignore_then(token!(Ident))
        .ignore_then(node.clone().then_ignore(token!(NewLine).repeated()).repeated().collect::<Vec<_>>())
        .then_ignore(token!(DeIdent));

        let if_ = token!(Identifier, "if")
            .ignore_then(expr.clone())
            .then_ignore(token!(Ctrl, ":"))
            .then(body.clone())
            .then(
                token!(Identifier, "elif")
                .ignore_then(expr.clone())
                .then_ignore(token!(Ctrl, ":"))
                .then(body.clone())
                .repeated().collect::<Vec<_>>()
            )
            .then(
                token!(Identifier, "else")
                .ignore_then(token!(Ctrl, ":"))
                .ignore_then(body.clone())
                .or_not()
            )
        .map( |(((condition, body), elif), or_else)| {
            Node::If { 
                condition, 
                body,
                elif,
                or_else: or_else.unwrap_or(vec![])
            }
        });

        let return_ = token!(Identifier, "return")
        .ignore_then(expr.clone().or_not())
        .map(|body| Node::Return(body));

        let init_var = 
            token!(Identifier, "var")
            .ignore_then(ident.then(token!(Ctrl, ":").ignore_then(ident.or_not()).or_not()))
            .then(token!(Op, "=").ignore_then(expr.clone().or_not()))
            .map(|((name, type_), default_value)| Node::InitVar { var: Variable { name, type_: type_.into(), default_value } });

        let set_var = 
            ident.then(op).then(expr.clone())
            .map(|((name, op), value)| Node::SetVar { name, op, value });

        let while_ = token!(Identifier, "while")
        .ignore_then(expr.clone())
        .then_ignore(token!(Ctrl, ":"))
        .then(body.clone())
        .map(|(condition, body)| Node::While { condition, body });

        let num = select! { Token::Number(num) => num };

        let iter = num.map(|x| (0, 1, x)).or(
                token!(Identifier, "range")
                .ignore_then(token!(Ctrl, "("))
                .ignore_then(num)
                .then(token!(Ctrl, ",").ignore_then(num).or_not())
                .then(token!(Ctrl, ",").ignore_then(num).or_not())
                .then_ignore(token!(Ctrl, ")"))
                .map(|thing| match thing {
                    ((a, None), None) => (0, 1, a),
                    ((a, Some(b)), None) => (a, 1, b),
                    ((a, Some(b)), Some(c)) => (a, c, b),
                    _ => panic!()
                })
            )
            .map(|(start, step, end)| Iterator::Range { start, step, end });

        let for_ = token!(Identifier, "for")
        .ignore_then(ident)
        .then_ignore(token!(Identifier, "in"))
        .then(iter)
        .then_ignore(token!(Ctrl, ":"))
        .then(body.clone())

        .map(|((var_name, iterator), body)| Node::For { var_name, iterator, body });

        if_
        .or(for_)
        .or(while_)
        .or(return_)
        .or(init_var)
        .or(set_var)
        .or(token!(Identifier, "break").to(Node::Break))
        .or(token!(Identifier, "continue").to(Node::Continue))
        .or(expr.clone().then_ignore(token!(NewLine)).map(Node::Expr))
    });

    let func = 
        token!(Identifier, "func")
        .ignore_then(
            ident
            //.map_with(|name, e| (name, e.span()))
        )
        .then_ignore(token!(Ctrl, "("))
        .then(
            ident
            .then(token!(Ctrl, ":").ignore_then(ident.or_not()).or_not())
            .then(token!(Op, "=").ignore_then(expr.clone()).or_not())
            .separated_by(token!(Ctrl, ",")).collect::<Vec<_>>()
        )
        .then_ignore(token!(Ctrl, ")"))
        .then(token!(Ctrl, "->").ignore_then(ident).or_not())
        .then_ignore(token!(Ctrl, ":"))
        .then_ignore(token!(NewLine))
        .then_ignore(token!(Ident))
        .then(node
            .then_ignore(
                token!(NewLine).repeated()
            ).repeated().collect())
        .then_ignore(token!(DeIdent))

        .map(| (((name, args_), return_type), body) | {

            let mut args = vec![];
            for ((name, type_), default_value) in args_ {
                let type_ = type_.into();

                let var = Variable {
                    name,
                    type_,
                    default_value,
                };
                args.push(var);
            }

            let return_type = return_type.into();

            TLNode::Function { 
                name,
                args, 
                return_type,
                body
            }
        });

    

    let block = token!(NewLine).repeated().ignore_then(
        func
        .or(token!(Identifier, "class_name").ignore_then(ident).map(TLNode::ClassName))
        .or(token!(Identifier, "extends").ignore_then(ident).map(TLNode::Extends))
    ).then_ignore(token!(NewLine).repeated());

    // func test():
    //   var a = 0
    //       print(a)
    //   a = 1
    /*let block = block.clone().or(
        token!(Ident)
        .ignore_then(block)
        .then_ignore(token!(DeIdent)));*/

    block.map_with(|x, e| (x, e.span()))
        .repeated()
        .collect::<Vec<Spanned<TLNode>>>()
        .then_ignore(end())
}


/*fn parser<'a>() -> impl Parser<'a, &'a [Token], TLNode, Error = Simple<'a, char>> {
    just(Token::Ident).to(TLNode::ClassName("TEST".into()))
}*/
