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

pub struct Tokenizer {
    pos: usize,
    chars: Vec<char>,

    src: String,
}

impl Tokenizer {
    pub fn new(input: String) -> Tokenizer {
        Tokenizer {
            pos: 0,
            chars: input.chars().collect(),
            src: input,
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
            if !self.src.is_char_boundary(self.pos) {
                self.pos += 1;
                continue;
            }

            let start_pos = self.pos;
            let mut skip = false;

            let token = match char {
                '0'..='9' => self.number(),
                '#' => self.comment(),
                '\'' | '"' => self.string(),
                '\n' => {
                    indent = self.indent(indent, &mut tokens);
                    skip = true;
                    Token::NewLine
                }
                '$' => self.get_node(),
                '@' => self.anotation(),
                // Identifier or raw string
                c if is_xid_start(c) || c == '_' => self.ident(),

                c if CTRL_OR_OP_POS1.contains(&c) => self.ctrl_or_op(c),

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
            tokens.push(Token::DeIdent)
        }

        tokens
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

        let str = &self.src[start..self.pos];

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

        let str = &self.src[start..self.pos];

        Token::Comment(str.to_string())
    }

    fn string(&mut self) -> Token {
        let start = self.pos;

        let start_char = self.peek().unwrap();
        let is_tripple = start_char == self.next().unwrap();
        if is_tripple {
            if self.next().unwrap() != start_char {
                // it wasn't a triple it was an empty string
                return Token::String(String::new());
            }
        }

        while let Some(char) = self.next() {
            if !self.src.is_char_boundary(self.pos) {
                continue;
            }

            if char == start_char {
                if is_tripple {
                    if self.next().unwrap() == start_char {
                        if self.next().unwrap() == start_char {
                            break;
                        }
                    }
                } else {
                    break;
                }
            } else if char == '\\' {
                self.next();
            }
        }

        self.next();
        /*while let Some(_) = self.next() {
            if self.src.is_char_boundary(self.pos) {
                break;
            }
        }*/

        let offset = if is_tripple { 3 } else { 1 };
        //println!("TEST {}", self.chars[start + 3]);
        let str = &self.src[start + offset..self.pos - offset];

        Token::String(str.to_string())
    }

    fn indent(&mut self, current_indent: usize, tokens: &mut Vec<Token>) -> usize {
        let mut new_indent = 0;

        // skip empty lines
        while let Some('\n') = self.next() {}
        self.pos -= 1;

        while let Some(char) = self.next() {
            // this should be 2 spaces instead of 1 but I am lazy
            if char == '\t' || char == ' ' {
                new_indent += 1;
            } else if char == '#' {
                tokens.push(self.comment());
                tokens.push(Token::NewLine);
                new_indent = 0;
            } else if char == '\\' {
                while let Some(' ' | '\t') = self.next() {}
                return current_indent;
            } else {
                break;
            }
        }

        tokens.push(Token::NewLine);

        if new_indent > current_indent {
            for _ in current_indent..new_indent {
                tokens.push(Token::Ident);
            }
        } else if new_indent < current_indent {
            for _ in new_indent..current_indent {
                tokens.push(Token::DeIdent);
            }
        }

        new_indent
    }

    fn get_node(&mut self) -> Token {
        if let Some('"') = self.next() {
            let string = self.string();
            if let Token::String(str) = string {
                return Token::GetNode(str);
            }
        }
        self.pos -= 1;

        let start = self.pos + 1;

        while let Some(c) = self.next() {
            if !self.src.is_char_boundary(self.pos) {
                continue;
            }
            if !is_xid_continue(c) && c != '/' {
                break;
            }
        }

        let str = &self.src[start..self.pos];
        Token::GetNode(str.to_string())
    }

    fn anotation(&mut self) -> Token {
        let start = self.pos + 1;

        while let Some(c) = self.next() {
            if !self.src.is_char_boundary(self.pos) {
                continue;
            }
            if !is_xid_continue(c) {
                break;
            }
        }

        let str = &self.src[start..self.pos];

        Token::Anotation(str.to_string())
    }

    fn ident(&mut self) -> Token {
        if self.peek().unwrap() == 'r' {
            if let Some(char) = self.next() {
                if char == '\'' || char == '"' {
                    let string = self.string();
                    if let Token::String(str) = string {
                        return Token::RawString(str);
                    }
                }
            }
            self.pos -= 1;
        }

        let start = self.pos;

        while let Some(char) = self.next() {
            /*if !self.src.is_char_boundary(self.pos) {
                continue;
            }*/
            if !is_xid_continue(char) {
                break;
            }
        }
        /*while let Some(_) = self.next() {
            if !self.src.is_char_boundary(self.pos) {
                break;
            }
        }*/

        let str = &self.src[start..self.pos];
        Token::Identifier(str.to_string())
    }

    fn ctrl_or_op(&mut self, c: char) -> Token {
        // Sometimes floats star with .
        if self.peek().unwrap() == '.' {
            if let Some(c) = self.next() {
                if c.is_digit(10) {
                    self.pos -= 1;
                    return self.number();
                }
            }
        }

        // TODO OP
        if let Some(c) = self.next() {
            if c == '>' {
                self.next();
                return Token::Ctrl("->".to_string());
            }
        }

        Token::Ctrl(c.to_string())
    }
}
