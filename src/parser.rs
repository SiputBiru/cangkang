use crate::error::CangkangError;
use crate::lexer::{Lexer, Token};

#[derive(Debug, PartialEq)]
pub struct Document {
    pub blocks: Vec<Block>,
}

#[derive(Debug, PartialEq)]
pub enum Inline {
    Text(String),
    Bold(String),
    Italic(String),
    Link { text: String, url: String },
    Image { alt: String, url: String },
    Code(String),
    FootnoteRef(String),
}

#[derive(Debug, PartialEq)]
pub enum Block {
    Heading { level: u8, content: Vec<Inline> },
    Paragraph(Vec<Inline>),
    CodeBlock { language: String, code: String },
    FootnoteDef { id: String, content: Vec<Inline> },
}

pub struct Parser {
    lexer: Lexer,
    current_token: Token,
    peek_token: Token,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Self {
        let current_token = lexer.next_token();
        let peek_token = lexer.next_token();

        Parser {
            lexer,
            current_token,
            peek_token,
        }
    }

    fn new_token(&mut self) {
        self.current_token = std::mem::replace(&mut self.peek_token, self.lexer.next_token());
    }

    fn parse_inline(&mut self) -> Result<Vec<Inline>, CangkangError> {
        let mut inlines = Vec::new();

        while self.current_token != Token::Eof && self.current_token != Token::Newline {
            match &self.current_token {
                Token::Text(text) => {
                    inlines.push(Inline::Text(text.clone()));
                    self.new_token();
                }
                Token::Asterisk => {
                    if self.peek_token == Token::Asterisk {
                        self.new_token();
                        self.new_token();

                        let mut content = String::new();
                        while self.current_token != Token::Eof {
                            if self.current_token == Token::Asterisk
                                && self.peek_token == Token::Asterisk
                            {
                                self.new_token();
                                self.new_token();
                                break;
                            }
                            if let Token::Text(ref t) = self.current_token {
                                content.push_str(t);
                            }
                            self.new_token();
                        }
                        inlines.push(Inline::Bold(content));
                    } else {
                        self.new_token(); // consume *
                        let mut content = String::new();
                        while self.current_token != Token::Eof
                            && self.current_token != Token::Asterisk
                        {
                            if let Token::Text(ref t) = self.current_token {
                                content.push_str(t);
                            }
                            self.new_token();
                        }
                        if self.current_token == Token::Asterisk {
                            self.new_token(); // consume closing *
                        }
                        inlines.push(Inline::Italic(content));
                    }
                }
                Token::BracketLeft => {
                    // It's a Footnote
                    if self.peek_token == Token::Caret {
                        self.new_token();
                        self.new_token();

                        let mut id = String::new();
                        if let Token::Text(ref t) = self.current_token {
                            id.push_str(t);
                            self.new_token();
                        }

                        if self.current_token == Token::BracketRight {
                            self.new_token();
                        }
                        inlines.push(Inline::FootnoteRef(id))
                    } else {
                        // It's a Link: [text](url)
                        let link = self.parse_link_or_image(false)?;
                        inlines.push(link)
                    }
                }
                Token::Bang => {
                    // It might be an image: ![alt](url)
                    if self.peek_token == Token::BracketLeft {
                        self.new_token(); // consume !
                        let img = self.parse_link_or_image(true)?;
                        inlines.push(img);
                    } else {
                        inlines.push(Inline::Text("!".to_string()));
                        self.new_token();
                    }
                }
                Token::BackTick(n) => {
                    // For inline code, expect a specific number of backticks (usually 1 or 2)
                    let start_fence = *n;
                    self.new_token(); // Consume opening backtick(s)

                    let mut code = String::new();
                    while self.current_token != Token::Eof {
                        if self.current_token == Token::BackTick(start_fence) {
                            self.new_token(); // Consume closing backtick(s)
                            break;
                        }

                        // Just append the raw text of whatever is inside
                        if let Token::Text(ref t) = self.current_token {
                            code.push_str(t);
                        } else {
                            // Quick hack to convert other formatting tokens back to text if they are inside code
                            code.push_str(&format!("{:?}", self.current_token)); // can refined this later
                        }
                        self.new_token();
                    }
                    inlines.push(Inline::Code(code));
                }
                _ => {
                    inlines.push(Inline::Text(format!("{:?}", self.current_token)));
                    self.new_token();
                }
            }
        }
        Ok(inlines)
    }

    fn parse_link_or_image(&mut self, is_image: bool) -> Result<Inline, CangkangError> {
        self.new_token();

        let mut text = String::new();
        while self.current_token != Token::Eof && self.current_token != Token::BracketRight {
            if let Token::Text(ref t) = self.current_token {
                text.push_str(t);
            }
            self.new_token();
        }

        if self.current_token == Token::BracketRight {
            self.new_token(); // consume ']'
        }

        if self.current_token != Token::ParenLeft {
            return Err(CangkangError::Parse {
                message: "Expected '(' after link text".to_string(),
                line: 0,
            });
        }
        self.new_token(); // consume '('

        let mut url = String::new();
        while self.current_token != Token::Eof && self.current_token != Token::ParenRight {
            if let Token::Text(ref t) = self.current_token {
                url.push_str(t);
            }
            self.new_token();
        }

        if self.current_token == Token::ParenRight {
            self.new_token(); // consume ')'
        }

        if is_image {
            Ok(Inline::Image { alt: text, url })
        } else {
            Ok(Inline::Link { text, url })
        }
    }

    fn parse_code_block(&mut self, fence_length: u8) -> Result<Block, CangkangError> {
        self.new_token(); // Consume the opening ```

        // Grab the optional language identifier (e.g., "rust" or "c")
        let mut language = String::new();
        while self.current_token != Token::Newline && self.current_token != Token::Eof {
            if let Token::Text(ref text) = self.current_token {
                language.push_str(text);
            }
            self.new_token();
        }

        if self.current_token == Token::Newline {
            self.new_token(); // Consume the newline after the language name
        }

        // Grab all the raw code inside the block
        let mut code = String::new();
        loop {
            if self.current_token == Token::Eof {
                break; // Technically an unclosed code block, but we can forgive it
            }

            // Check if we hit the closing fence
            if let Token::BackTick(n) = self.current_token
                && n >= fence_length
            {
                self.new_token(); // Consume closing ```
                break;
            }

            // Reconstruct the raw text by converting tokens back to strings
            match &self.current_token {
                Token::Text(t) => code.push_str(t),
                Token::Newline => code.push('\n'),
                Token::Asterisk => code.push('*'),
                Token::BracketLeft => code.push('['),
                Token::BracketRight => code.push(']'),
                Token::ParenLeft => code.push('('),
                Token::ParenRight => code.push(')'),
                Token::Bang => code.push('!'),
                Token::HeadingMarker(n) => {
                    for _ in 0..*n {
                        code.push('#');
                    }
                }
                Token::BackTick(n) => {
                    for _ in 0..*n {
                        code.push('`');
                    }
                }
                _ => {}
            }
            self.new_token();
        }

        Ok(Block::CodeBlock {
            language: language.trim().to_string(),
            code,
        })
    }

    pub fn parse_document(&mut self) -> Result<Document, CangkangError> {
        let mut document = Document { blocks: Vec::new() };

        while self.current_token != Token::Eof {
            if self.current_token == Token::Newline {
                self.new_token();
                continue;
            }

            let block = match self.current_token {
                Token::HeadingMarker(_) => self.parse_heading()?,
                Token::BackTick(n) if n >= 3 => self.parse_code_block(n)?,
                Token::BracketLeft if self.peek_token == Token::Caret => {
                    self.parse_footnote_def()?
                }
                _ => self.parse_paragraph()?,
            };

            document.blocks.push(block);
        }

        Ok(document)
    }

    fn parse_heading(&mut self) -> Result<Block, CangkangError> {
        let level = match self.current_token {
            Token::HeadingMarker(l) => l,
            _ => {
                return Err(CangkangError::Parse {
                    message: "Expected HeadingMarker".to_string(),
                    line: 0,
                });
            }
        };

        self.new_token();

        let content = self.parse_inline()?;

        if self.current_token == Token::Newline {
            self.new_token();
        }

        Ok(Block::Heading { level, content })
    }

    fn parse_paragraph(&mut self) -> Result<Block, CangkangError> {
        let content = self.parse_inline()?;

        if self.current_token == Token::Newline {
            self.new_token();
        }

        Ok(Block::Paragraph(content))
    }

    fn parse_footnote_def(&mut self) -> Result<Block, CangkangError> {
        self.new_token(); // Consume '['
        self.new_token(); // Consume '^'

        let mut id = String::new();

        if let Token::Text(ref t) = self.current_token {
            id.push_str(t);
            self.new_token();
        }

        self.new_token(); // Consume ']'

        if self.current_token == Token::Colon {
            self.new_token(); // Consume ':'
        }

        let content = self.parse_inline()?;
        if self.current_token == Token::Newline {
            self.new_token();
        }

        Ok(Block::FootnoteDef { id, content })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    #[test]
    fn test_parse_simple_document() {
        let input = "### Hello Cangkang\n\nThis is a paragraph about building an SSG.\n";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let doc = parser.parse_document().expect("Failed to parse document");

        assert_eq!(doc.blocks.len(), 2);

        assert_eq!(
            doc.blocks[0],
            Block::Heading {
                level: 3,
                content: vec![Inline::Text("Hello Cangkang".to_string())],
            }
        );

        assert_eq!(
            doc.blocks[1],
            Block::Paragraph(vec![Inline::Text(
                "This is a paragraph about building an SSG.".to_string()
            )])
        );
    }

    #[test]
    fn test_parse_inline_formatting() {
        let input = "This has **bold** and *italic* text.\n\nCheck out my [Github](url) and this `inline code`.\n\n```rust\nlet x = 5;\n```";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let doc = parser.parse_document().expect("Failed to parse document");

        assert_eq!(doc.blocks.len(), 3);

        // Check Paragraph (Bold and Italic)
        assert_eq!(
            doc.blocks[0],
            Block::Paragraph(vec![
                Inline::Text("This has ".to_string()),
                Inline::Bold("bold".to_string()),
                Inline::Text(" and ".to_string()),
                Inline::Italic("italic".to_string()),
                Inline::Text(" text.".to_string()),
            ])
        );

        // Check Paragraph (Link and Inline Code)
        assert_eq!(
            doc.blocks[1],
            Block::Paragraph(vec![
                Inline::Text("Check out my ".to_string()),
                Inline::Link {
                    text: "Github".to_string(),
                    url: "url".to_string(),
                },
                Inline::Text(" and this ".to_string()),
                Inline::Code("inline code".to_string()),
                Inline::Text(".".to_string()),
            ])
        );

        // Check Block Fenced Code Block)
        assert_eq!(
            doc.blocks[2],
            Block::CodeBlock {
                language: "rust".to_string(),
                code: "let x = 5;\n".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_footnotes() {
        let input = "Here is a note[^1].\n\n[^1]: The actual note text\n";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let doc = parser.parse_document().expect("Failed to parse document");

        assert_eq!(doc.blocks.len(), 2);

        // Check the Inline Reference
        assert_eq!(
            doc.blocks[0],
            Block::Paragraph(vec![
                Inline::Text("Here is a note".to_string()),
                Inline::FootnoteRef("1".to_string()),
                Inline::Text(".".to_string()),
            ])
        );

        // Check the Definition Block
        assert_eq!(
            doc.blocks[1],
            Block::FootnoteDef {
                id: "1".to_string(),
                content: vec![Inline::Text(" The actual note text".to_string())]
            }
        );
    }
}
