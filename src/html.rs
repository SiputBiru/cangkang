use crate::parser::{Block, Document, Inline};
use std::collections::HashMap;

pub fn generate_html(document: &Document) -> String {
    let mut html = String::new();

    // PASS 1: Collect all Footnote Definitions
    let mut footnotes = HashMap::new();
    for block in &document.blocks {
        if let Block::FootnoteDef { id, content } = block {
            footnotes.insert(id.clone(), render_inlines(content, &HashMap::new()));
        }
    }

    // PASS 2: Render everything else, passing the footnote map down
    for block in &document.blocks {
        // Skip rendering the definitions at the bottom
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
        Block::Code { language, code } => {
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
        Block::List(items) => {
            let mut html = String::from("<ul>\n");
            for (indent, item) in items {
                let margin = (*indent / 4) as f32 * 1.5;
                html.push_str(&format!(
                    "  <li style=\"margin-left: {}rem;\">{}</li>\n",
                    margin,
                    render_inlines(item, footnotes)
                ));
            }
            html.push_str("</ul>");
            html
        }
        Block::OrderedList(items) => {
            let mut html = String::from("<ol>\n");
            for (indent, item) in items {
                let margin = (*indent / 4) as f32 * 1.5;
                html.push_str(&format!(
                    "  <li style=\"margin-left: {}rem;\">{}</li>\n",
                    margin,
                    render_inlines(item, footnotes)
                ));
            }
            html.push_str("</ol>");
            html
        }

        Block::FootnoteDef { .. } => String::new(), // Handled in Pass 1

        Block::Callout { kind, content } => {
            if kind == "quote" {
                format!(
                    "<blockquote>{}</blockquote>",
                    render_inlines(content, footnotes)
                )
            } else {
                let title = kind.to_uppercase();
                let icon = if kind == "warn" { "⚠️" } else { "💡" };
                format!(
                    "<div class=\"callout callout-{}\">\n  <div class=\"callout-title\">{} {}</div>\n  <p>{}</p>\n</div>",
                    kind,
                    icon,
                    title,
                    render_inlines(content, footnotes)
                )
            }
        }

        Block::Table {
            headers,
            alignments,
            rows,
        } => {
            let mut html = String::from("<div class=\"table-wrapper\">\n<table>\n");

            // --- THEAD ---
            html.push_str("  <thead>\n    <tr>\n");
            for (i, header) in headers.iter().enumerate() {
                let align = alignments
                    .get(i)
                    .unwrap_or(&crate::parser::Alignment::Default);
                html.push_str(&format!(
                    "      <th{}>{}</th>\n",
                    get_align_style(align),
                    render_inlines(header, footnotes)
                ));
            }
            html.push_str("    </tr>\n  </thead>\n");

            // --- TBODY ---
            html.push_str("  <tbody>\n");
            for row in rows {
                html.push_str("    <tr>\n");
                for (i, cell) in row.iter().enumerate() {
                    let align = alignments
                        .get(i)
                        .unwrap_or(&crate::parser::Alignment::Default);
                    html.push_str(&format!(
                        "      <td{}>{}</td>\n",
                        get_align_style(align),
                        render_inlines(cell, footnotes)
                    ));
                }
                html.push_str("    </tr>\n");
            }
            html.push_str("  </tbody>\n</table>\n</div>\n");

            html
        }
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
        Inline::LineBreak => String::from("<br />"),
        Inline::Image { alt, url } => {
            format!("<img src=\"{}\" alt=\"{}\" />", url, escape_html(alt))
        }

        // INLINE INJECTION
        Inline::FootnoteRef(id) => {
            let content = footnotes
                .get(id)
                .cloned()
                .unwrap_or_else(|| "Missing footnote".to_string());

            // inject split label
            format!(
                r#"<label for="sn-{id}" class="sidenote-number-ref">[{id}]</label><input type="checkbox" id="sn-{id}" class="margin-toggle"/><span class="sidenote"><span class="sidenote-number-def" style="user-select: none;">[{id}]</span> {content}</span>"#,
                id = id,
                content = content.trim_start()
            )
        }
    }
}

// Helper to clean up table alignment generation
fn get_align_style(align: &crate::parser::Alignment) -> &'static str {
    match align {
        crate::parser::Alignment::Left => " style=\"text-align: left;\"",
        crate::parser::Alignment::Center => " style=\"text-align: center;\"",
        crate::parser::Alignment::Right => " style=\"text-align: right;\"",
        crate::parser::Alignment::Default => "",
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
                // The footnote definition
                Block::FootnoteDef {
                    id: "1".to_string(),
                    content: vec![Inline::Text("Written in Rust.".to_string())],
                },
            ],
        };

        let generated = generate_html(&doc);

        let expected_html = "<p>Cangkang is fast<label for=\"sn-1\" class=\"sidenote-number-ref\">[1]</label><input type=\"checkbox\" id=\"sn-1\" class=\"margin-toggle\"/><span class=\"sidenote\"><span class=\"sidenote-number-def\" style=\"user-select: none;\">[1]</span> Written in Rust.</span>.</p>\n";

        assert_eq!(generated, expected_html);
    }
}
