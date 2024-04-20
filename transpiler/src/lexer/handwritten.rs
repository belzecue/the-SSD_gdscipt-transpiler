


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

    fn peek(&self) -> char {
        self.chars[self.pos]
    }

    fn next(&mut self) -> char {
        self.pos += 1;
        self.chars[self.pos]
    }
}
