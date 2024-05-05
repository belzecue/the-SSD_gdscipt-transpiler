use crate::{
    ast::{Expr, Iterator, Node, TLNode, Type, Variable},
    lexer::Token,
};

macro_rules! then_ctrl {
    ($self: tt, $str: literal) => {
        if let Some(Token::Ctrl(a)) = $self.next() {
            assert_eq!(&a, $str)
        } else {
            panic!()
        };
    };
}

macro_rules! peek_ctrl {
    ($self: tt, $str: literal) => {
        if let Some(Token::Ctrl(a)) = $self.peek() {
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

            Token::Comment(c) => TLNode::Comment(c),
            Token::NewLine => TLNode::NewLine,
            t => panic!("{t:?}"),
        }
    }

    fn parse_func(&mut self) -> TLNode {
        let Some(Token::Identifier(func_name)) = self.next() else {
            panic!()
        };

        then_ctrl!(self, "(");
        let mut args = vec![];
        while Some(Token::Ctrl(")".to_string())) != self.next() {
            let Some(Token::Identifier(name)) = self.peek() else {
                break;
            };
            
            let mut next = self.next();
            let type_ = if next == Some(Token::Ctrl(":".into())) {
                next = self.next();
                if next != Some(Token::Op("=".into())) {
                    self.pos -= 1;
                    let t = self.parse_type();
                    next = self.next();
                    t
                } else {
                    Type::Auto
                }
            } else {
                Type::None
            };
    
            let default_value = if next == Some(Token::Op("=".into())) {
                self.next();
                Some(self.parse_expr())
            } else {
                None
            };

            args.push(Variable { name, type_, default_value });
        }

        if Some(Token::Ctrl("->".into())) == self.peek() {
            self.pos -= 1;
        } else if Some(Token::Ctrl(":".into())) == self.peek() {
            self.pos -= 1;
        }

        let type_ = if Some(Token::Ctrl("->".into())) == self.next() {
            self.parse_type()
        } else {
            self.pos -= 1;
            Type::None
        };


        then_ctrl!(self, ":");

        then!(self, NewLine);

        let body = self.parse_block();

        TLNode::Function {
            name: func_name,
            args,
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
            Token::Indent => {
                self.pos -= 1;
                Some(Node::Block(self.parse_block()))
            },

            Token::Comment(c) => Some(Node::Comment(c.to_string())),
            Token::NewLine => Some(Node::NewLine),

            t => panic!("not expected: {t:?}"),
        };

        //println!("{:?}: {:?}: {}: {:?}", self.peek(), ret, self.pos, tok);

        /*if let Some(Token::NewLine) = self.peek() {
            self.pos += 1;
        }*/
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

                        if Token::Ctrl(",".to_string()) != self.peek().unwrap() {
                            break;
                        }
                        self.next();
                    }
                    self.next();

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
                "+" => {
                    let ((), r_bp) = self.prefix_binding_power(&sp);

                    self.next();

                    let rhs = self.parse_expr_bp(r_bp);
                    return rhs;
                }
                "-" => {
                    let ((), r_bp) = self.prefix_binding_power(&sp);

                    self.next();

                    let rhs = self.parse_expr_bp(r_bp);
                    return Expr::Neg(Box::new(rhs));
                }
                "~" => {
                    let ((), r_bp) = self.prefix_binding_power(&sp);

                    self.next();

                    let rhs = self.parse_expr_bp(r_bp);
                    return Expr::BitNeg(Box::new(rhs));
                }
                op => panic!("{op}"),
            },

            Token::Ctrl(ctrl) => match ctrl.as_str() {
                "(" => {
                    self.next();
                    let lhs = self.parse_expr_bp(0);
                    self.next();
                    return lhs;
                }
                c => panic!("{c}"),
            }

            t => panic!("{t:?}"),
        };

        self.pos += 1;
        node
    }

    fn prefix_binding_power(&self, op: &str) -> ((), u8) {
        match op {
            "+" | "-" | "!" | "~"=> ((), 10),

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
        then!(self, Indent);
        self.next();

        let mut body = vec![];
        while let Some(tok) = self.peek() {
            let start = self.pos;

            if Token::DeIndent == tok {
                break;
            }

            if let Some(node) = self.parse_node(&tok) {
                body.push(node);
            }

            if self.pos == start {
                self.pos += 1;
            }
        }
        self.next();

        // If body is just a single block then unwrap it
        if body.len() == 1 {
            if let Node::Block(body2) = body[0].clone() {
                body = body2;
            }
        }

        body
    }

    fn parse_init_var(&mut self) -> Option<Node> {
        let Some(Token::Identifier(var_name)) = self.next() else {
            panic!();
        };

        let mut next = self.next();

        let type_ = if next == Some(Token::Ctrl(":".into())) {
            next = self.next();
            if next != Some(Token::Op("=".into())) {
                self.pos -= 1;
                let t = self.parse_type();
                next = self.next();
                t
            } else {
                Type::Auto
            }
        } else {
            Type::None
        };

        let default_value = if next == Some(Token::Op("=".into())) {
            self.next();
            Some(self.parse_expr())
        } else {
            None
        };

        self.next();

        let var = Variable {
            name: var_name,
            type_,
            default_value,
        };
        Some(Node::InitVar { var })
    }

    fn parse_if(&mut self) -> Option<Node> {
        self.next();
        let condition = self.parse_expr();

        peek_ctrl!(self, ":");
        then!(self, NewLine);

        let body = self.parse_block();

        let mut elif = vec![];
        let mut or_else = vec![];

        while let Some(Token::Identifier(ident)) = self.peek() {
            let start = self.pos;

            if ident == "elif" {
                self.next();
                let cond = self.parse_expr();

                peek_ctrl!(self, ":");
                then!(self, NewLine);

                let body = self.parse_block();
                elif.push((cond, body));
            } else if ident == "else" {
                then_ctrl!(self, ":");
                then!(self, NewLine);

                or_else = self.parse_block();
            } else {
                break;
            }

            if start == self.pos {
                self.pos += 1;
            }
        }
        

        Some(Node::If {
            condition,
            body,
            elif,
            or_else,
        })
    }

    fn parse_while(&mut self) -> Option<Node> {
        self.next();
        let condition = self.parse_expr();

        peek_ctrl!(self, ":");
        then!(self, NewLine);

        let body = self.parse_block();

        Some(Node::While { condition, body })
    }

    fn parse_for(&mut self) -> Option<Node> {
        let Some(Token::Identifier(var_name)) = self.next() else {
            panic!();
        };

        assert_eq!(Token::Identifier("in".into()), self.next().unwrap());

        let next = self.next();

        let iterator = if Some(Token::Identifier("range".into())) == next {
            then_ctrl!(self, "(");
            self.next();

            let mut start = match self.parse_expr() {
                Expr::Number(num) => num,
                _ => panic!(),
            };
            let step;
            let end;

            if Some(Token::Ctrl(",".into())) == self.peek() {
                self.next();

                end = match self.parse_expr() {
                    Expr::Number(num) => num,
                    _ => panic!(),
                };

                if Some(Token::Ctrl(",".into())) == self.peek() {
                    self.next();

                    step = match self.parse_expr() {
                        Expr::Number(num) => num,
                        _ => panic!(),
                    };
                } else {
                    step = 1;
                };
            } else {
                end = start;
                start = 0;
                step = 1;
            }

            peek_ctrl!(self, ")");

            Iterator::Range { start, step, end }
        } else if let Some(Token::Number(num)) = next {
            Iterator::Range {
                start: 0,
                step: 1,
                end: num,
            }
        } else {
            panic!("{:?}", self.peek())
        };

        then_ctrl!(self, ":");
        then!(self, NewLine);

        Some(Node::For {
            var_name,
            iterator,
            body: self.parse_block(),
        })
    }

    fn parse_return(&mut self) -> Option<Node> {

        let next = self.next();
        let expr = if Some(Token::NewLine) != next && Some(Token::DeIndent) != next {
            Some(self.parse_expr())
        } else {
            self.next();
            None
        };

        Some(Node::Return(expr))
    }

    fn parser_set_var_or_expr(&mut self) -> Option<Node> {

        let Some(Token::Identifier(name)) = self.peek() else {
            return Some(Node::Expr(self.parse_expr()));
        };

        if name == "pass" {
            self.next();
            return None;
        }

        if let Some(Token::Op(op)) = self.next() {
            self.next();
            let value = self.parse_expr();
            Some(Node::SetVar { name, op, value })
        } else {
            self.pos -= 1;
            Some(Node::Expr(self.parse_expr()))
        }
    }
}
