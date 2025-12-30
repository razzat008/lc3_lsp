use std::{iter::Peekable, str::Chars};

use crate::parser::{Token, WhiteSpace};

struct Lexer<'i> {
    chars: Peekable<Chars<'i>>,
    eof_returned: bool,
    line_number: usize,
    pos: usize,
}

impl<'i> Lexer<'i> {
    pub fn init(input: &'i str) -> Self {
        Self {
            chars: input.chars().peekable(),
            eof_returned: false,
            line_number: 1,
            pos: 0,
        }
    }

    pub fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }

    pub fn advance(&mut self) -> Option<char> {
        self.chars.next().inspect(|_c| {
            self.pos += 1;
        })
    }

    fn skip_whitespaces(&mut self) {
        while let Some(char) = self.peek() {
            if char.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn next_token(&mut self) -> Option<Token> {

        // safely stfu
        if self.peek().is_none() {
            if self.eof_returned {
                return None;
            } else {
                self.eof_returned = true;
                return Some(Token::EOF);
            }
        }

        let next_char = self.peek().unwrap();


        todo!()
    }
}

#[cfg(test)]
#[test]
fn test_tokens() {
    let tokens = vec!["ADD", "SUB"];
}
