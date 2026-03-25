use crate::parser::{Block, Document, Inline};

pub fn generate_html(document: &Document) -> String {
    let mut html = String::new();

    for block in &document.blocks {
        html.push_str(&render_block(block));
        html.push('\n');
    }

    html
}

fn render_block(block: &Block) -> String {
    match block {
        Block::Heading { level, content } => {
            let inner_html = render_inlines(content);
            format!("<h{}>{}</h{}>", level, inner_html, level)
        }
        Block::Paragraph(content) => {
            let inner_html = render_inlines(content);
            format!("<p>{}</p>", inner_html)
        }
        Block::CodeBlock { language, code } => {
            let escaped_code = escape_html(code);
            if language.is_empty() {
                format!("<pre><code>{}</code></pre>", escaped_code)
            } else {
                format!(
                    "<pre><code class=\"language-{}\">{}</code></pre>",
                    language, escaped_code
                )
            }
        }
    }
}

fn render_inlines(inlines: &[Inline]) -> String {
    let mut html = String::new();
    for inline in inlines {
        html.push_str(&render_inline(inline));
    }
    html
}

fn render_inline(inline: &Inline) -> String {
    match inline {
        Inline::Text(text) => escape_html(text),
        Inline::Bold(text) => format!("<strong>{}</strong>", escape_html(text)),
        Inline::Italic(text) => format!("<em>{}</em>", escape_html(text)),
        Inline::Code(code) => format!("<code>{}</code>", escape_html(code)),
        Inline::Link { text, url } => {
            // URLs shouldn't be aggressively escaped here, but the text should
            format!("<a href=\"{}\">{}</a>", url, escape_html(text))
        }
        Inline::Image { alt, url } => {
            format!("<img src=\"{}\" alt=\"{}\" />", url, escape_html(alt))
        }
    }
}

// simple HTML escaper
fn escape_html(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '&' => escaped.push_str("&amp;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(c),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_simple_html() {
        // Construct a manual AST to test the generator
        let doc = Document {
            blocks: vec![
                Block::Heading {
                    level: 2,
                    content: vec![Inline::Text("Cangkang".to_string())],
                },
                Block::Paragraph(vec![
                    Inline::Text("Building an SSG in ".to_string()),
                    Inline::Bold("Rust".to_string()),
                    Inline::Text(" is fun. Here is some code: ".to_string()),
                    Inline::Code("x < y".to_string()),
                ]),
            ],
        };

        let expected_html = "\
<h2>Cangkang</h2>
<p>Building an SSG in <strong>Rust</strong> is fun. Here is some code: <code>x &lt; y</code></p>\n";

        let generated = generate_html(&doc);
        assert_eq!(generated, expected_html);
    }
}
