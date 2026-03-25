#[derive(Debug, PartialEq)]
pub enum Token {
    HeadingMarker(u8),
    Text(String),
    Newline,
    Asterisk,     // '*'
    BracketLeft,  // '['
    BracketRight, // ']'
    ParenLeft,    // '('
    ParenRight,   // ')'
    Bang,         // '!' (for images: ![alt](url) )
    BackTick(u8), // '`' (for codeblock: ``` )
    Caret,        // '^'
    Colon,        // ':'
    Eof,
}

pub struct Lexer {
    input: Vec<char>,
    pub position: usize, // Current position in input (points to current char)
    pub read_position: usize, // Current reading position in input (after current char)
    pub ch: char,        // Current char under examination
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let mut l = Lexer {
            input: input.chars().collect(),
            position: 0,
            read_position: 0,
            ch: '\0',
        };
        l.read_char();
        l
    }

    pub fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = '\0';
        } else {
            self.ch = self.input[self.read_position];
        }
        self.position = self.read_position;
        self.read_position += 1;
    }

    // can be used for other things
    // pub fn peek_char(&self) -> char {
    //     if self.read_position >= self.input.len() {
    //         '\0'
    //     } else {
    //         self.input[self.read_position]
    //     }
    // }

    pub fn next_token(&mut self) -> Token {
        match self.ch {
            '\n' => {
                self.read_char(); // Advance past the matched character for single-char tokens
                Token::Newline
            }
            '*' => {
                self.read_char();
                Token::Asterisk
            }
            '\0' => Token::Eof,
            '#' => {
                let mut level = 0;
                // Count how many '#' we have
                while self.ch == '#' {
                    level += 1;
                    self.read_char();
                }
                // Skip the trailing space if there is one
                if self.ch == ' ' {
                    self.read_char();
                }
                Token::HeadingMarker(level) // Early return because we already advanced
            }
            '[' => {
                self.read_char();
                Token::BracketLeft
            }
            ']' => {
                self.read_char();
                Token::BracketRight
            }
            '(' => {
                self.read_char();
                Token::ParenLeft
            }
            ')' => {
                self.read_char();
                Token::ParenRight
            }
            '!' => {
                self.read_char();
                Token::Bang
            }
            '`' => {
                let mut count = 0;
                while self.ch == '`' {
                    count += 1;
                    self.read_char();
                }
                Token::BackTick(count)
            }
            '^' => {
                self.read_char();
                Token::Caret
            }
            ':' => {
                self.read_char();
                Token::Colon
            }
            _ => {
                let start_position = self.position;
                while self.ch != '\n'
                    && self.ch != '*'
                    && self.ch != '#'
                    && self.ch != '\0'
                    && self.ch != '['
                    && self.ch != ']'
                    && self.ch != '('
                    && self.ch != ')'
                    && self.ch != '!'
                    && self.ch != '`'
                    && self.ch != '^'
                    && self.ch != ':'
                {
                    self.read_char();
                }
                let text: String = self.input[start_position..self.position].iter().collect();
                Token::Text(text)
            }
        }
    }
}
