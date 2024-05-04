use crate::{
    ast::{Expr, Node, TLNode, Type},
    lexer::Token,
};

macro_rules! ctrl {
    ($self: tt, $str: literal) => {
        if let Some(Token::Ctrl(a)) = $self.next() {
            assert_eq!(&a, $str)
        } else {
            panic!()
        };
    };
}

macro_rules! then {
    ($self: tt, $tok: tt) => {
        let Some(Token::$tok) = $self.next() else {
            panic!()
        };
    };
}

/*
macro_rules! peek_is {
    ($self: tt, $tok: tt) => {
        let Some(Token::$tok) = $self.peek() else {
            panic!()
        };
    };
}*/

pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<Token> {
        self.tokens.get(self.pos).cloned()
    }

    fn next(&mut self) -> Option<Token> {
        self.pos += 1;
        self.peek()
    }

    pub fn parse(&mut self) -> Vec<TLNode> {
        let mut ast = vec![];
        while let Some(tok) = self.peek() {
            let start = self.pos;

            let tlnode = self.parse_tlnode(tok);
            ast.push(tlnode);

            if self.pos == start {
                self.pos += 1;
            }
        }
        ast
    }

    fn parse_tlnode(&mut self, tok: Token) -> TLNode {
        match tok {
            Token::Identifier(ident) => match ident.as_str() {
                "func" => self.parse_func(),
                "class_name" => self.parse_class_name(),
                "extends" => self.parse_extends(),
                _ => panic!(),
            },
            Token::Anotation(_) => todo!(),

            Token::Comment(c) => {
                if self.next() == Some(Token::NewLine) {
                    self.next();
                }
                TLNode::Comment(c)
            }
            Token::NewLine => TLNode::NewLine,
            t => panic!("{t:?}"),
        }
    }

    fn parse_func(&mut self) -> TLNode {
        let Some(Token::Identifier(func_name)) = self.next() else {
            panic!()
        };

        ctrl!(self, "(");
        // TODO
        ctrl!(self, ")");

        let type_ = if let Some(Token::Ctrl(ctrl)) = self.next() {
            if &ctrl == "->" {
                self.parse_type()
            } else {
                Type::None
            }
        } else {
            self.pos -= 1;
            Type::None
        };

        ctrl!(self, ":");

        then!(self, NewLine);
        then!(self, Indent);

        /*let mut body = vec![];
        while let Some(tok) = self.peek() {
            let start = self.pos;

            if Token::DeIdent == tok {
                break;
            }

            if let Some(node) = self.parse_node(&tok) {
                body.push(node);
            }

            if self.pos == start {
                self.pos += 1;
            }
        }*/

        let body = self.parse_block();

        TLNode::Function {
            name: func_name,
            args: vec![],
            return_type: type_,
            body,
        }
    }

    fn parse_type(&mut self) -> Type {
        // TODO
        let next = self.next();
        if let Some(Token::Identifier(ident)) = next {
            Type::Some(ident)
        } else {
            Type::Auto
        }
    }

    fn parse_class_name(&mut self) -> TLNode {
        let Some(Token::Identifier(class_name)) = self.next() else {
            panic!();
        };
        self.pos += 1;

        if self.peek() == Some(Token::NewLine) {
            self.next();
        }

        TLNode::ClassName(class_name)
    }

    fn parse_extends(&mut self) -> TLNode {
        let Some(Token::Identifier(extends)) = self.next() else {
            panic!();
        };
        self.pos += 1;

        if self.peek() == Some(Token::NewLine) {
            self.next();
        }

        TLNode::Extends(extends)
    }

    fn parse_node(&mut self, tok: &Token) -> Option<Node> {
        let ret = match tok {
            Token::Identifier(ident) => match ident.as_str() {
                "var" => self.parse_init_var(),
                "if" => self.parse_if(),
                "while" => self.parse_while(),
                "for" => self.parse_for(),
                "break" => Some(Node::Break),
                "continue" => Some(Node::Continue),
                "return" => self.parse_return(),
                _ => self.parser_set_var_or_expr(),
            },
            Token::Indent => Some(Node::Block(self.parse_block())),
            Token::DeIdent => None,
            n => panic!("{n}"),
        };

        println!("{:?}: {:?}: {}: {:?}", self.peek(), ret, self.pos, tok);

        if let Some(Token::NewLine) = self.peek() {
            self.pos += 1;
        }
        ret
    }

    fn parse_expr(&mut self) -> Expr {
        self.parse_expr_bp(0)
    }

    // Prat parser
    fn parse_expr_bp(&mut self, min_bp: u8) -> Expr {
        let mut lhs = self.parse_primary();

        while let Some(tok) = self.peek() {
            let op = match tok {
                Token::Op(op) => op,

                _ => break,
            };

            if let Some((l_bp, r_bp)) = self.infix_binding_power(&op) {
                if l_bp < min_bp {
                    break;
                }

                self.next();

                let rhs = self.parse_expr_bp(r_bp);

                lhs = Expr::Op {
                    lhs: Box::new(lhs),
                    op,
                    rhs: Box::new(rhs),
                };
                continue;
            }

            break;
        }

        lhs
    }
    fn parse_primary(&mut self) -> Expr {
        let node = match self.peek().unwrap() {
            Token::FPNumber(num) => Expr::FPNumber(num),
            Token::Number(num) => Expr::Number(num),

            Token::Identifier(identifier) => {
                let next = self.next().unwrap();
                if Token::Ctrl("(".to_string()) == next {
                    let mut args = vec![];
                    while let Some(next) = self.next() {
                        if next == Token::Ctrl(")".into()) {
                            break;
                        }

                        let expr = self.parse_expr();
                        args.push(expr);

                        if Token::Ctrl(",".to_string()) == self.peek().unwrap() {
                            self.next();
                        } else {
                            break;
                        }
                    }

                    return Expr::Call {
                        name: identifier,
                        args,
                    };
                } else {
                    self.pos -= 1; //roll back next
                    Expr::Variable(identifier)
                }
            }

            Token::Op(sp) => match sp.as_str() {
                "-" => {
                    let ((), r_bp) = self.prefix_binding_power(&sp);

                    self.next();

                    let rhs = self.parse_expr_bp(r_bp);
                    return Expr::Neg(Box::new(rhs));
                }
                "(" => {
                    self.next();
                    let lhs = self.parse_expr_bp(0);
                    self.next();
                    return lhs;
                }
                _ => panic!(),
            },

            _ => panic!(),
        };

        self.pos += 1;
        node
    }

    fn prefix_binding_power(&self, op: &str) -> ((), u8) {
        match op {
            "+" | "-" | "!" => ((), 10),

            _ => panic!("bad op: {:?}", op),
        }
    }

    fn infix_binding_power(&self, op: &str) -> Option<(u8, u8)> {
        let res = match op {
            "&&" | "||" => (1, 2),

            ">" | "<" => (3, 4),
            "==" | "!=" => (3, 4),
            "<=" | ">=" => (3, 4),

            "^" => (5, 6),

            "+" | "-" => (5, 6),
            "*" | "/" => (7, 8),

            _ => return None,
        };

        Some(res)
    }

    fn parse_block(&mut self) -> Vec<Node> {
        //peek_is!(self, Indent);
        self.next();

        let mut body = vec![];
        while let Some(tok) = self.peek() {
            let start = self.pos;

            if Token::DeIdent == tok {
                break;
            }

            if let Some(node) = self.parse_node(&tok) {
                if let Node::Block(body2) = node {
                    body.extend(body2.into_iter())
                } else {
                    body.push(node);
                }
            }

            if self.pos == start {
                self.pos += 1;
            }
        }
        self.next();

        body
    }

    fn parse_init_var(&mut self) -> Option<Node> {
        todo!()
    }

    fn parse_if(&mut self) -> Option<Node> {
        todo!()
    }

    fn parse_while(&mut self) -> Option<Node> {
        todo!()
    }

    fn parse_for(&mut self) -> Option<Node> {
        todo!()
    }

    fn parse_return(&mut self) -> Option<Node> {
        // TODO return with viod
        let expr = Some(self.parse_expr());
        Some(Node::Return(expr))
    }

    fn parser_set_var_or_expr(&mut self) -> Option<Node> {
        // TODO set var
        Some(Node::Expr(self.parse_expr()))
    }
}
