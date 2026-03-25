use crate::parser::{Block, Document, Inline};
use std::collections::HashMap;

pub fn generate_html(document: &Document) -> String {
    let mut html = String::new();

    // PASS 1: Collect all Footnote Definitions
    let mut footnotes = HashMap::new();
    for block in &document.blocks {
        if let Block::FootnoteDef { id, content } = block {
            // Render the inner content of the footnote ahead of time
            let rendered_content = render_inlines(content, &HashMap::new());
            footnotes.insert(id.clone(), rendered_content);
        }
    }

    // PASS 2: Render everything else, passing the footnote map down
    for block in &document.blocks {
        // Skip rendering the definitions at the bottom (just like Gingerbill!)
        if let Block::FootnoteDef { .. } = block {
            continue;
        }

        html.push_str(&render_block(block, &footnotes));
        html.push('\n');
    }

    html
}

fn render_block(block: &Block, footnotes: &HashMap<String, String>) -> String {
    match block {
        Block::Heading { level, content } => {
            format!(
                "<h{}>{}</h{}>",
                level,
                render_inlines(content, footnotes),
                level
            )
        }
        Block::Paragraph(content) => {
            format!("<p>{}</p>", render_inlines(content, footnotes))
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
        Block::FootnoteDef { .. } => String::new(), // Handled above
    }
}

fn render_inlines(inlines: &[Inline], footnotes: &HashMap<String, String>) -> String {
    let mut html = String::new();
    for inline in inlines {
        html.push_str(&render_inline(inline, footnotes));
    }
    html
}

fn render_inline(inline: &Inline, footnotes: &HashMap<String, String>) -> String {
    match inline {
        Inline::Text(text) => escape_html(text),
        Inline::Bold(text) => format!("<strong>{}</strong>", escape_html(text)),
        Inline::Italic(text) => format!("<em>{}</em>", escape_html(text)),
        Inline::Code(code) => format!("<code>{}</code>", escape_html(code)),
        Inline::Link { text, url } => format!("<a href=\"{}\">{}</a>", url, escape_html(text)),
        Inline::Image { alt, url } => {
            format!("<img src=\"{}\" alt=\"{}\" />", url, escape_html(alt))
        }

        // INLINE INJECTION
        Inline::FootnoteRef(id) => {
            let content = footnotes
                .get(id)
                .cloned()
                .unwrap_or_else(|| "Missing footnote".to_string());
            format!(
                r#"<label for="sn-{}" class="sidenote-number">[{}]</label><input type="checkbox" id="sn-{}" class="margin-toggle"/><span class="sidenote">{}</span>"#,
                id, id, id, content
            )
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

    #[test]
    fn test_generate_html_with_footnotes() {
        let doc = Document {
            blocks: vec![
                // The paragraph using the footnote
                Block::Paragraph(vec![
                    Inline::Text("Cangkang is fast".to_string()),
                    Inline::FootnoteRef("1".to_string()),
                    Inline::Text(".".to_string()),
                ]),
                // The footnote definition (which should be hidden in the final HTML!)
                Block::FootnoteDef {
                    id: "1".to_string(),
                    content: vec![Inline::Text(" Written in Rust.".to_string())],
                },
            ],
        };

        let generated = generate_html(&doc);

        // We expect the paragraph to contain the label/input/span hack,
        // and the definition block to be completely missing from the end.
        let expected_html = "<p>Cangkang is fast<label for=\"sn-1\" class=\"margin-toggle sidenote-number\">[1]</label><input type=\"checkbox\" id=\"sn-1\" class=\"margin-toggle\"/><span class=\"sidenote\"> Written in Rust.</span>.</p>\n";

        assert_eq!(generated, expected_html);
    }
}
