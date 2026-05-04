use crate::error::CangkangError;
use crate::lexer::{Lexer, Token};

#[derive(Debug, PartialEq)]
pub struct Document {
    pub blocks: Vec<Block>,
}

// Table stuff
#[derive(Debug, PartialEq, Clone)]
pub enum Alignment {
    Left,
    Center,
    Right,
    Default,
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
    LineBreak,
}

#[derive(Debug, PartialEq)]
pub enum Block {
    Heading {
        level: u8,
        content: Vec<Inline>,
    },
    Paragraph(Vec<Inline>),
    Code {
        language: String,
        code: String,
    },
    DropdownCode {
        title: String,
        language: String,
        code: String,
    },
    FootnoteDef {
        id: String,
        content: Vec<Inline>,
    },
    List(Vec<(usize, Vec<Inline>)>),
    OrderedList(Vec<(usize, Vec<Inline>)>),
    // HorizontalRule,
    Callout {
        kind: CalloutKind, // "note", "warn", etc.
        content: Vec<Inline>,
    },
    Table {
        headers: Vec<Vec<Inline>>,
        alignments: Vec<Alignment>,
        rows: Vec<Vec<Vec<Inline>>>, // row is a vec of cells, and a cells is a Vec of Inlines
    },
}

// Callout Map and enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalloutKind {
    Note,
    Warn,
    Tip,
    Important,
    Caution,
    Quote,
}

impl CalloutKind {
    // Helper to get the CSS class name
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Note => "note",
            Self::Warn => "warn",
            Self::Tip => "tip",
            Self::Important => "important",
            Self::Caution => "caution",
            Self::Quote => "quote",
        }
    }

    // Helper for icon logic
    pub fn icon(&self) -> &'static str {
        // match self {
        //     // (i) - Information Circle
        //     Self::Note => "🛈 ",
        //     // (!) - Check/Alert Circle or Lightbulb
        //     Self::Tip => "𖡊 ",
        //     // [!] - Message square alert or Exclamation Circle
        //     Self::Important => "❕ ",
        //     // /!\ - Triangle Warning
        //     Self::Warn => "⚠ ",
        //     // (x) - Octagon or Stop sign
        //     Self::Caution => "✖ ",
        //     // Default for Quote
        //     Self::Quote => "",
        // }
        ""
    }
}

static CALLOUT_MAP: &[(&str, CalloutKind)] = &[
    ("[!NOTE]", CalloutKind::Note),
    ("[!WARNING]", CalloutKind::Warn),
    ("[!WARN]", CalloutKind::Warn),
    ("[!TIP]", CalloutKind::Tip),
    ("[!IMPORTANT]", CalloutKind::Important),
    ("[!CAUTION]", CalloutKind::Caution),
];

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
                    let text = match &self.current_token {
                        Token::Colon => ":".to_string(),
                        Token::Caret => "^".to_string(),
                        Token::ParenLeft => "(".to_string(),
                        Token::ParenRight => ")".to_string(),
                        Token::BracketRight => "]".to_string(),
                        Token::Bang => "!".to_string(),
                        _ => format!("{:?}", self.current_token),
                    };
                    inlines.push(Inline::Text(text));
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
            match &self.current_token {
                Token::Text(t) => text.push_str(t),
                Token::Colon => text.push(':'),
                Token::Caret => text.push('^'),
                Token::Asterisk => text.push('*'),
                Token::Bang => text.push('!'),
                _ => {}
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
            match &self.current_token {
                Token::Text(t) => url.push_str(t),
                Token::Colon => url.push(':'),
                Token::Caret => url.push('^'),
                Token::Asterisk => url.push('*'),
                Token::Bang => text.push('!'),
                _ => {}
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
                Token::Colon => code.push(':'),
                Token::Caret => code.push('^'),
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

        Ok(Block::Code {
            language: language.trim().to_string(),
            code,
        })
    }

    fn parse_dropdown_code_block(&mut self, fence_length: u8) -> Result<Block, CangkangError> {
        self.new_token(); // Consume the opening +++

        // Grab the optional language and title (e.g., "rust [My Title]")
        let mut info = String::new();
        while self.current_token != Token::Newline && self.current_token != Token::Eof {
            if let Token::Text(ref text) = self.current_token {
                info.push_str(text);
            } else if self.current_token == Token::BracketLeft {
                info.push('[');
            } else if self.current_token == Token::BracketRight {
                info.push(']');
            }
            self.new_token();
        }

        if self.current_token == Token::Newline {
            self.new_token();
        }

        // Parse language and title from info
        let (language, mut title) = if let Some(bracket_start) = info.find('[') {
            let lang = info[..bracket_start].trim().to_string();
            let t = if let Some(bracket_end) = info.find(']') {
                info[bracket_start + 1..bracket_end].trim().to_string()
            } else {
                info[bracket_start + 1..].trim().to_string()
            };
            (lang, t)
        } else {
            (info.trim().to_string(), String::new())
        };

        if title.is_empty() {
            title = "Click to expand code".to_string();
        }

        // Grab all the raw code inside the block
        let mut code = String::new();
        loop {
            if self.current_token == Token::Eof {
                break;
            }

            // Check if we hit the closing fence
            if let Token::Plus(n) = self.current_token
                && n >= fence_length
            {
                self.new_token(); // Consume closing +++
                break;
            }

            match &self.current_token {
                Token::Text(t) => code.push_str(t),
                Token::Newline => code.push('\n'),
                Token::Asterisk => code.push('*'),
                Token::BracketLeft => code.push('['),
                Token::BracketRight => code.push(']'),
                Token::ParenLeft => code.push('('),
                Token::ParenRight => code.push(')'),
                Token::Bang => code.push('!'),
                Token::Colon => code.push(':'),
                Token::Caret => code.push('^'),
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
                Token::Plus(n) => {
                    for _ in 0..*n {
                        code.push('+');
                    }
                }
                _ => {}
            }
            self.new_token();
        }

        Ok(Block::DropdownCode {
            title,
            language,
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
                Token::Plus(n) if n >= 3 => self.parse_dropdown_code_block(n)?,
                Token::BracketLeft if self.peek_token == Token::Caret => {
                    self.parse_footnote_def()?
                }
                Token::Asterisk => {
                    let is_list = matches!(
                        (&self.current_token, &self.peek_token),
                        (Token::Asterisk, Token::Text(t)) if t.starts_with([' ', '\t'])
                    );

                    if is_list {
                        self.parse_list()?
                    } else {
                        self.parse_paragraph()?
                    }
                }
                Token::Text(ref t) if t.trim().is_empty() && self.peek_token == Token::Asterisk => {
                    self.parse_list()?
                }
                Token::Text(ref t) if t.starts_with('>') => self.parse_callout()?,
                Token::Text(ref t)
                    if {
                        let trimmed = t.trim_start();
                        trimmed.starts_with(|c: char| c.is_ascii_digit()) && trimmed.contains(". ")
                    } =>
                {
                    self.parse_ordered_list()?
                }
                Token::Text(ref t) if t.trim_start().starts_with('|') => self.parse_table()?,
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

    fn parse_list(&mut self) -> Result<Block, CangkangError> {
        let mut items = Vec::new();

        loop {
            let mut indent = 0;
            let mut is_list_item = false;

            // Case 1: The line starts with spaces, then an Asterisk
            if let Token::Text(t) = &self.current_token {
                if t.trim().is_empty() && self.peek_token == Token::Asterisk {
                    indent = t.len(); // Count the spaces!
                    self.new_token(); // Consume the spaces
                    is_list_item = true;
                }
            }
            // Case 2: The line starts directly with an Asterisk
            else if self.current_token == Token::Asterisk {
                is_list_item = true;
            }

            if !is_list_item {
                break;
            }

            self.new_token(); // Consume the '*'

            // Strip leading spaces from the actual text
            if let Token::Text(text) = &mut self.current_token {
                let trimmed = text.trim_start().to_string();
                if trimmed.is_empty() {
                    self.new_token();
                } else {
                    *text = trimmed;
                }
            }

            let content = self.parse_inline()?;

            // Store the indent count with the content!
            items.push((indent, content));

            if self.current_token == Token::Newline {
                self.new_token();
            }

            // Consume any extra newlines between list items
            while self.current_token == Token::Newline {
                self.new_token();
            }
        }

        Ok(Block::List(items))
    }

    fn parse_paragraph(&mut self) -> Result<Block, CangkangError> {
        let content = self.parse_inline()?;

        if self.current_token == Token::Newline {
            self.new_token();
        }

        Ok(Block::Paragraph(content))
    }

    fn parse_ordered_list(&mut self) -> Result<Block, CangkangError> {
        let mut items = Vec::new();

        loop {
            let mut is_numbered = false;
            let mut indent = 0;

            if let Token::Text(t) = &self.current_token {
                // Count the spaces at the start of the line
                let trimmed = t.trim_start();
                indent = t.len() - trimmed.len();

                // Check if the trimmed part starts with a number and a dot
                if trimmed.starts_with(|c: char| c.is_ascii_digit()) && trimmed.contains(". ") {
                    is_numbered = true;
                }
            }

            if !is_numbered {
                break;
            }

            // Strip the spaces AND the "1. " from the start of the token
            if let Token::Text(text) = &mut self.current_token {
                let trimmed = text.trim_start();
                let dot_idx = trimmed.find(". ").unwrap();
                let final_text = trimmed[dot_idx + 2..].to_string();
                *text = final_text;
            }

            let content = self.parse_inline()?;

            // Store the indent count alongside the content
            items.push((indent, content));

            if self.current_token == Token::Newline {
                self.new_token();
            }

            // Consume any extra newlines between list items
            while self.current_token == Token::Newline {
                self.new_token();
            }
        }
        Ok(Block::OrderedList(items))
    }

    fn parse_callout(&mut self) -> Result<Block, CangkangError> {
        let mut raw_text = String::new();

        // Token Collection
        while self.current_token != Token::Eof {
            if self.current_token == Token::Newline && self.peek_token == Token::Newline {
                break;
            }
            match &self.current_token {
                Token::Text(t) => raw_text.push_str(t),
                Token::BracketLeft => raw_text.push('['),
                Token::BracketRight => raw_text.push(']'),
                Token::Bang => raw_text.push('!'),
                Token::Newline => raw_text.push('\n'),
                Token::Asterisk => raw_text.push('*'),
                _ => {}
            }
            self.new_token();
        }

        // Extract Kind and Content
        let cleaned = raw_text.replace("> ", "").replace('>', "");
        let trimmed = cleaned.trim();

        let (kind, content_str) = CALLOUT_MAP
            .iter()
            .find(|(tag, _)| trimmed.starts_with(tag))
            .map(|(tag, k)| (*k, trimmed[tag.len()..].trim())) // *k copies the enum
            .unwrap_or((CalloutKind::Quote, trimmed));

        // Sub-parsing
        let mut sub_parser = Parser::new(Lexer::new(content_str));
        let mut content = Vec::new();

        while sub_parser.current_token != Token::Eof {
            if sub_parser.current_token == Token::Newline {
                content.push(Inline::LineBreak);
            } else if let Ok(mut inlines) = sub_parser.parse_inline() {
                content.append(&mut inlines);
            }
            sub_parser.new_token();
        }

        Ok(Block::Callout { kind, content })
    }

    fn read_line_raw(&mut self) -> String {
        let mut raw = String::new();
        while self.current_token != Token::Eof && self.current_token != Token::Newline {
            match &self.current_token {
                Token::Text(t) => raw.push_str(t),
                Token::Asterisk => raw.push('*'),
                Token::BracketLeft => raw.push('['),
                Token::BracketRight => raw.push(']'),
                Token::ParenLeft => raw.push('('),
                Token::ParenRight => raw.push(')'),
                Token::Bang => raw.push('!'),
                Token::Colon => raw.push(':'),
                Token::Caret => raw.push('^'),
                Token::HeadingMarker(n) => raw.push_str(&"#".repeat(*n as usize)),
                Token::BackTick(n) => raw.push_str(&"`".repeat(*n as usize)),
                _ => {}
            }
            self.new_token();
        }
        if self.current_token == Token::Newline {
            self.new_token();
        }
        raw
    }

    // Table Stuff
    fn parse_table_row(&self, line: &str) -> Vec<Vec<Inline>> {
        let trimmed = line.trim();
        // Strip the outer pipes (e.g., "| cell |" becomes " cell ")
        let content = if trimmed.starts_with('|') && trimmed.ends_with('|') {
            &trimmed[1..trimmed.len() - 1]
        } else {
            trimmed
        };

        let mut parsed_cells = Vec::new();
        for cell in content.split('|') {
            // Spin up a mini-parser for every single cell!
            let mut sub_parser = Parser::new(Lexer::new(cell.trim()));
            let inlines = sub_parser.parse_inline().unwrap_or_default();
            parsed_cells.push(inlines);
        }
        parsed_cells
    }

    fn parse_table(&mut self) -> Result<Block, CangkangError> {
        // Grab the Header row
        let header_line = self.read_line_raw();
        let headers = self.parse_table_row(&header_line);

        // Grab the Alignment/Divider row (e.g., |:---|---:|)
        let divider_line = self.read_line_raw();
        let trimmed_div = divider_line.trim();
        let div_content = if trimmed_div.starts_with('|') && trimmed_div.ends_with('|') {
            &trimmed_div[1..trimmed_div.len() - 1]
        } else {
            trimmed_div
        };

        let alignments: Vec<Alignment> = div_content
            .split('|')
            .map(|cell| {
                let c = cell.trim();
                let left = c.starts_with(':');
                let right = c.ends_with(':');
                if left && right {
                    Alignment::Center
                } else if left {
                    Alignment::Left
                } else if right {
                    Alignment::Right
                } else {
                    Alignment::Default
                }
            })
            .collect();

        // Grab all the Body rows
        let mut rows = Vec::new();
        while matches!(&self.current_token, Token::Text(t) if t.trim_start().starts_with('|')) {
            let row_line = self.read_line_raw();
            rows.push(self.parse_table_row(&row_line));
        }

        Ok(Block::Table {
            headers,
            alignments,
            rows,
        })
    }

    // parse footnote def
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
            Block::Code {
                language: "rust".to_string(),
                code: "let x = 5;\n".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_dropdown_code() {
        let input = "+++rust [My Code]\nprintln!(\"hi\");\n+++\n";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let doc = parser.parse_document().expect("Failed to parse document");

        assert_eq!(doc.blocks.len(), 1);

        assert_eq!(
            doc.blocks[0],
            Block::DropdownCode {
                title: "My Code".to_string(),
                language: "rust".to_string(),
                code: "println!(\"hi\");\n".to_string(),
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
                content: vec![Inline::Text(" The actual note text".to_string())],
            }
        );
    }

    #[test]
    fn test_parse_lists_with_blank_lines() {
        let input = "1. First item\n\n2. Second item\n\n* Unordered 1\n\n* Unordered 2\n";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let doc = parser.parse_document().expect("Failed to parse document");

        assert_eq!(doc.blocks.len(), 2);

        // Check Ordered List
        if let Block::OrderedList(items) = &doc.blocks[0] {
            assert_eq!(items.len(), 2);
            assert_eq!(items[0].1, vec![Inline::Text("First item".to_string())]);
            assert_eq!(items[1].1, vec![Inline::Text("Second item".to_string())]);
        } else {
            panic!("Expected OrderedList, got {:?}", doc.blocks[0]);
        }

        // Check Unordered List
        if let Block::List(items) = &doc.blocks[1] {
            assert_eq!(items.len(), 2);
            assert_eq!(items[0].1, vec![Inline::Text("Unordered 1".to_string())]);
            assert_eq!(items[1].1, vec![Inline::Text("Unordered 2".to_string())]);
        } else {
            panic!("Expected List, got {:?}", doc.blocks[1]);
        }
    }
}
