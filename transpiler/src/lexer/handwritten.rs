use super::Token;
use unicode_ident::*;

/*
const OPERATORS: [&str; 33] = [
    "+", "-", "*", "**", "/", "%", "~", "&", "|", "^", "<<", ">>",
    "=", "+=", "-=", "*=", "**=", "/=", "%=", "&=", "|=", "^=", "<<=", ">>=",

    "<", ">", "!", "&&", "||",
    "<=", ">=", "!=", "=="
];*/

const CTRL_OR_OP_POS1: [char; 23] = [
    '(', ')', ':', ',', '{', '}', ';', '.', '[', ']', '+', '-', '*', '/', '%', '~', '&', '|', '^',
    '<', '>', '=', '!',
];

const OPERATORS: [&str; 34] = [
    "+", "-", "*", "**", "/", "%", "~", "&", "|", "^", "<<", ">>", "=", "+=", "-=", "*=", "**=",
    "/=", "%=", "~=", "&=", "|=", "^=", "<<=", ">>=", "<", ">", "!", "&&", "||", "<=", ">=", "!=",
    "==",
];

const CTRL: [&str; 11] = ["(", ")", ":", ",", "{", "}", ";", ".", "[", "]", "->"];

pub struct Lexer {
    pos: usize,
    chars: Vec<char>,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            pos: 0,
            chars: input.chars().collect(),
        }
    }

    #[inline]
    fn peek(&mut self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    #[inline]
    fn next(&mut self) -> Option<char> {
        self.pos += 1;
        self.peek()
    }

    pub fn lex(&mut self) -> Vec<Token> {
        let mut tokens = vec![];
        let mut indent = 0;

        while let Some(char) = self.peek() {
            let start_pos = self.pos;
            let mut skip = false;

            let token = match char {
                '0'..='9' => self.number(),
                '#' => self.comment(),
                '\'' | '"' => self.string(false),
                '\n' => {
                    indent = self.indent(indent, &mut tokens);
                    skip = true;
                    Token::NewLine
                }
                '$' => self.get_node(),
                '@' => self.anotation(),

                c if CTRL_OR_OP_POS1.contains(&c) => self.ctrl_or_op(c),

                // Identifier or raw string
                c if is_xid_start(c) || c == '_' => self.ident(),

                ' ' | '\t' => {
                    skip = true;
                    Token::NewLine
                }

                '\\' => {
                    if let Some(char) = self.next() {
                        if char != '\n' {
                            panic!(
                                "panic at: {}, char: {:?}, tok: {:#?}",
                                self.pos.clone(),
                                self.peek(),
                                tokens
                            )
                        }
                    }
                    skip = true;
                    Token::NewLine
                }
                _ => panic!("panic at: {}, char: {:?}", self.pos.clone(), self.peek()),
            };

            if !skip {
                tokens.push(token);
            }

            if self.pos == start_pos {
                self.pos += 1;
            }
        }

        while indent > 0 {
            indent -= 1;
            tokens.push(Token::DeIndent)
        }

        tokens
    }

    fn slice(&self, start: usize, end: usize) -> String {
        self.chars[start..end].iter().cloned().collect::<String>()
    }

    fn number(&mut self) -> Token {
        let start = self.pos;
        let mut is_float = self.peek().unwrap() == '.';

        while let Some(char) = self.next() {
            if char != '.' && !char.is_numeric() {
                break;
            } else if char == '.' {
                if is_float {
                    // has more then 1 dot
                    panic!();
                }

                is_float = true
            }
        }

        let str = self.slice(start, self.pos);

        if is_float {
            Token::FPNumber(str.parse().unwrap())
        } else {
            Token::Number(str.parse().unwrap())
        }
    }

    fn comment(&mut self) -> Token {
        let start = self.pos + 1;

        while let Some(c) = self.next() {
            if c == '\n' {
                break;
            }
        }

        Token::Comment(self.slice(start, self.pos))
    }

    fn string(&mut self, raw_string: bool) -> Token {
        let start = self.pos;

        let start_char = self.peek().unwrap();
        let is_tripple = start_char == self.next().unwrap();
        if is_tripple {
            if self.next().unwrap() != start_char {
                // it wasn't a triple it was an empty string
                return Token::String(String::new());
            }
        } else {
            self.pos -= 1;
        }

        //self.next();
        while let Some(char) = self.next() {
            if char == start_char {
                self.pos += 1;
                if is_tripple {
                    if self.next().unwrap() == start_char {
                        if self.next().unwrap() != start_char {
                            break;
                        }
                        self.pos -= 1;
                    }
                    self.pos -= 1;
                } else {
                    break;
                }
            } else if char == '\\' {
                let next = self.next();
                if raw_string && next != Some(start_char) && next != Some('\\') {
                    self.pos -= 1;
                }
            }
        }

        let offset = if is_tripple { 3 } else { 1 };
        let str = self.slice(start + offset, self.pos - offset);

        Token::String(str)
    }

    fn indent(&mut self, current_indent: usize, tokens: &mut Vec<Token>) -> usize {
        let mut new_indent = 0;

        tokens.push(Token::NewLine);
        // skip empty lines
        while let Some('\n') = self.next() {
            tokens.push(Token::NewLine);
        }
        self.pos -= 1;

        let mut comments = vec![];

        while let Some(char) = self.next() {
            // this should be 2 spaces instead of 1 but I am lazy
            if char == '\t' || char == ' ' {
                new_indent += 1;
            } else if char == '#' {
                comments.push(self.comment());
                new_indent = 0;
            } else if char == '\\' {
                while let Some(' ' | '\t') = self.next() {
                    /*if let Some('\n') = self.peek() {
                        tokens.push(Token::NewLine);
                    }*/
                }
                return current_indent;
            } else {
                break;
            }
        }


        if new_indent > current_indent {
            for _ in current_indent..new_indent {
                tokens.push(Token::Indent);
            }
        } else if new_indent < current_indent {
            for _ in new_indent..current_indent {
                tokens.push(Token::DeIndent);
            }
        }

        tokens.append(&mut comments);

        new_indent
    }

    fn get_node(&mut self) -> Token {
        if let Some('"') = self.next() {
            let string = self.string(true);
            if let Token::String(str) = string {
                return Token::GetNode(str);
            }
        }
        self.pos -= 1;

        let start = self.pos + 1;

        while let Some(c) = self.next() {
            if !is_xid_continue(c) && c != '/' {
                break;
            }
        }

        let str = self.slice(start, self.pos);
        Token::GetNode(str)
    }

    fn anotation(&mut self) -> Token {
        let start = self.pos + 1;

        while let Some(c) = self.next() {
            if !is_xid_continue(c) {
                break;
            }
        }

        let str = self.slice(start, self.pos);

        Token::Anotation(str)
    }

    fn ident(&mut self) -> Token {
        if self.peek().unwrap() == 'r' {
            if let Some(char) = self.next() {
                if char == '\'' || char == '"' {
                    let string = self.string(true);
                    if let Token::String(str) = string {
                        return Token::RawString(str);
                    }
                }
            }
            self.pos -= 1;
        }

        let start = self.pos;

        while let Some(char) = self.next() {
            if !is_xid_continue(char) {
                break;
            }
        }

        let str = self.slice(start, self.pos);

        Token::Identifier(str)
    }

    fn ctrl_or_op(&mut self, c: char) -> Token {
        // Sometimes floats star with '.'
        if self.peek().unwrap() == '.' {
            if let Some(c) = self.next() {
                if c.is_digit(10) {
                    self.pos -= 1;
                    return self.number();
                }
            }
        }

        if CTRL.contains(&c.to_string().as_str()) {
            return Token::Ctrl(c.to_string());
        }

        let Some(p2) = self.next() else {
            panic!();
        };

        // handel ->
        if p2 == '>' {
            self.next();
            return Token::Ctrl("->".to_string());
        }

        let Some(p3) = self.next() else {
            panic!();
        };

        let mut op_string = String::new();
        op_string.push(c);
        op_string.push(p2);
        op_string.push(p3);

        if !OPERATORS.contains(&op_string.as_str()) {
            op_string.remove(op_string.len() - p3.len_utf8());
            self.pos -= 1;

            if !OPERATORS.contains(&op_string.as_str()) {
                op_string.remove(op_string.len() - p2.len_utf8());
                self.pos -= 1;
            }

            if !OPERATORS.contains(&op_string.as_str()) {
                panic!()
            }
        }
        self.pos += 1;
        
        return Token::Op(op_string);
    }
}
