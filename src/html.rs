use crate::parser::{Block, CalloutKind, Document, Inline};
use std::collections::HashMap;

/// A list block in a nesting chain: the items slice and whether it's ordered.
type ListChainItem<'a> = (&'a [(usize, Vec<Inline>)], bool);

pub fn generate_html(document: &Document) -> String {
    let mut html = String::new();

    let mut footnotes = HashMap::new();
    for block in &document.blocks {
        if let Block::FootnoteDef { id, content } = block {
            footnotes.insert(id.clone(), render_inlines(content, &HashMap::new()));
        }
    }

    let mut i = 0;
    while i < document.blocks.len() {
        let block = &document.blocks[i];
        if let Block::FootnoteDef { .. } = block {
            i += 1;
            continue;
        }

        let did_nest = render_list_chain(i, &document.blocks, &footnotes)
            .map(|(rendered, consumed)| {
                html.push_str(&rendered);
                html.push('\n');
                i += consumed;
            })
            .is_some();

        if !did_nest {
            html.push_str(&render_block(block, &footnotes));
            html.push('\n');
            i += 1;
        }
    }

    html
}

/// Collect a chain of consecutive list blocks of **alternating** types
/// (ordered ↔ unordered) where each successive block has a positive indent,
/// indicating it should nest inside the previous block's last `<li>`.
///
/// Returns `None` if the starting block isn't a list or if no nesting
/// partner is found. Otherwise returns the rendered HTML and the number
/// of blocks consumed.
fn render_list_chain(
    start_idx: usize,
    blocks: &[Block],
    footnotes: &std::collections::HashMap<String, String>,
) -> Option<(String, usize)> {
    let mut chain: Vec<ListChainItem<'_>> = Vec::new();

    for offset in 0.. {
        let idx = start_idx + offset;
        if idx >= blocks.len() {
            break;
        }
        match &blocks[idx] {
            Block::OrderedList(items) if !items.is_empty() => {
                if offset == 0 || !chain.last().unwrap().1 {
                    if offset > 0 && items[0].0 == 0 {
                        break;
                    }
                    chain.push((items, true));
                } else {
                    break;
                }
            }
            Block::List(items) if !items.is_empty() => {
                if offset == 0 || chain.last().unwrap().1 {
                    if offset > 0 && items[0].0 == 0 {
                        break;
                    }
                    chain.push((items, false));
                } else {
                    break;
                }
            }
            _ => break,
        }
    }

    if chain.len() < 2 {
        return None;
    }

    let (innermost, innermost_ordered) = chain.last().copied().unwrap();
    let inner_tag = if innermost_ordered { "ol" } else { "ul" };
    let mut result = render_nested_list_with_tag(innermost, inner_tag, footnotes);

    for i in (0..chain.len() - 1).rev() {
        let (outer, outer_ordered) = chain[i];
        let outer_tag = if outer_ordered { "ol" } else { "ul" };
        let n = outer.len();

        let mut outer_html = String::new();
        outer_html.push_str(&format!("<{}>\n", outer_tag));

        for (idx, (_, content)) in outer.iter().enumerate() {
            let is_last = idx == n - 1;
            outer_html.push_str("<li>");
            outer_html.push_str(&render_inlines(content, footnotes));

            if is_last {
                outer_html.push('\n');
                outer_html.push_str(&result);
                outer_html.push('\n');
                outer_html.push_str("</li>\n");
            } else {
                outer_html.push_str("</li>\n");
            }
        }

        outer_html.push_str(&format!("</{}>", outer_tag));
        result = outer_html;
    }

    Some((result, chain.len()))
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
        Block::DropdownCode {
            title,
            language,
            code,
        } => {
            let escaped_code = escape_html(code);
            let code_html = if language.is_empty() {
                format!("<pre><code>{}</code></pre>", escaped_code)
            } else {
                format!(
                    "<pre><code class=\"language-{}\">{}</code></pre>",
                    language, escaped_code
                )
            };
            format!(
                "<details class=\"dropdown-code\">\n  <summary>{}</summary>\n  {}\n</details>",
                escape_html(title),
                code_html
            )
        }
        Block::List(items) => render_nested_list(items, false, footnotes),
        Block::OrderedList(items) => render_nested_list(items, true, footnotes),

        Block::FootnoteDef { .. } => String::new(), // Handled in Pass 1

        Block::Callout { kind, content } => {
            if let CalloutKind::Quote = kind {
                return format!(
                    "<blockquote>\n  <p>{}</p>\n</blockquote>",
                    render_inlines(content, footnotes)
                );
            }

            let class_name = kind.as_str();
            let icon = kind.icon();
            let title = class_name.to_uppercase();
            let body = render_inlines(content, footnotes);

            format!(
                r#"<div class="callout callout-{class_name}">
  <div class="callout-title"><span style="user-select: none;">{icon}</span> {title}</div>
  <p>{body}</p>
</div>"#,
            )
        }

        Block::Table {
            headers,
            alignments,
            rows,
        } => {
            let mut html = String::from("<div class=\"table-wrapper\">\n<table>\n");

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

/// Render a list (ordered or unordered) with proper nested `<ol>`/`<ul>` tags
/// based on indent levels, instead of a flat list with margin hacks.
fn render_nested_list(
    items: &[(usize, Vec<Inline>)],
    ordered: bool,
    footnotes: &HashMap<String, String>,
) -> String {
    let tag = if ordered { "ol" } else { "ul" };
    render_nested_list_with_tag(items, tag, footnotes)
}

/// Internal helper that renders a list using an explicit tag name.
/// Used both for same-type nesting and for mixed-type nesting
/// (e.g. an `<ul>` nested inside an `<ol>`).
fn render_nested_list_with_tag(
    items: &[(usize, Vec<Inline>)],
    tag: &str,
    footnotes: &HashMap<String, String>,
) -> String {
    let n = items.len();
    if n == 0 {
        return String::new();
    }

    let mut html = String::new();
    let mut indent_stack: Vec<usize> = Vec::new();

    for i in 0..n {
        let (indent, content) = &items[i];

        while let Some(&top) = indent_stack.last() {
            if *indent < top {
                indent_stack.pop();
                html.push_str(&format!("</{}>\n", tag));
                html.push_str("</li>\n");
            } else {
                break;
            }
        }

        if indent_stack.is_empty() || *indent > *indent_stack.last().unwrap() {
            let is_nested = !indent_stack.is_empty();
            indent_stack.push(*indent);
            if is_nested {
                // Place nested <ol>/<ul> on a new line after the parent <li> content
                html.push('\n');
            }
            html.push_str(&format!("<{}>\n", tag));
        }

        html.push_str("<li>");
        html.push_str(&render_inlines(content, footnotes));

        let has_children = i + 1 < n && items[i + 1].0 > *indent;
        if !has_children {
            html.push_str("</li>\n");
        }
    }

    while indent_stack.len() > 1 {
        indent_stack.pop();
        html.push_str(&format!("</{}>\n", tag));
        html.push_str("</li>\n");
    }
    if indent_stack.pop().is_some() {
        html.push_str(&format!("</{}>", tag));
    }

    html
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
        Inline::Link { text, url } => format!(
            "<a href=\"{}\" target=\"_blank\" rel=\"noopener noreferrer\">{}</a>",
            escape_html(url),
            escape_html(text)
        ),
        Inline::LineBreak => String::from("<br />"),
        Inline::Image { alt, url } => {
            format!(
                "<img src=\"{}\" alt=\"{}\" />",
                escape_html(url),
                escape_html(alt)
            )
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

    // ---------------------------------------------------------------------------
    // Nested list rendering tests
    // ---------------------------------------------------------------------------

    #[test]
    fn test_nested_ordered_in_ordered() {
        // Same-type nesting: ordered list with an indented sub-list inside.
        // The sub-list items should restart numbering at 1 (via a nested <ol>).
        let doc = Document {
            blocks: vec![Block::OrderedList(vec![
                (0, vec![Inline::Text("Item A".to_string())]),
                (3, vec![Inline::Text("Nested A".to_string())]),
                (3, vec![Inline::Text("Nested B".to_string())]),
                (0, vec![Inline::Text("Item B".to_string())]),
            ])],
        };

        let html = generate_html(&doc);

        let expected = "\
<ol>
<li>Item A
<ol>
<li>Nested A</li>
<li>Nested B</li>
</ol>
</li>
<li>Item B</li>
</ol>\n";

        assert_eq!(html, expected);
    }

    #[test]
    fn test_nested_unordered_in_unordered() {
        // Same-type nesting: unordered list with a deeper indented sub-list.
        let doc = Document {
            blocks: vec![Block::List(vec![
                (0, vec![Inline::Text("Bullet A".to_string())]),
                (3, vec![Inline::Text("Nested A".to_string())]),
                (3, vec![Inline::Text("Nested B".to_string())]),
                (0, vec![Inline::Text("Bullet B".to_string())]),
            ])],
        };

        let html = generate_html(&doc);

        let expected = "\
<ul>
<li>Bullet A
<ul>
<li>Nested A</li>
<li>Nested B</li>
</ul>
</li>
<li>Bullet B</li>
</ul>\n";

        assert_eq!(html, expected);
    }

    #[test]
    fn test_nested_unordered_in_ordered() {
        // Mixed-type nesting: ordered list followed by an indented unordered list.
        // The unordered list should be nested inside the last <li> of the ordered list.
        let doc = Document {
            blocks: vec![
                Block::OrderedList(vec![(0, vec![Inline::Text("Main item".to_string())])]),
                Block::List(vec![
                    (3, vec![Inline::Text("Nested bullet".to_string())]),
                    (3, vec![Inline::Text("Another nested bullet".to_string())]),
                ]),
            ],
        };

        let html = generate_html(&doc);

        let expected = "\
<ol>
<li>Main item
<ul>
<li>Nested bullet</li>
<li>Another nested bullet</li>
</ul>
</li>
</ol>\n";

        assert_eq!(html, expected);
    }

    #[test]
    fn test_nested_ordered_in_unordered() {
        // Mixed-type nesting: unordered list followed by an indented ordered list.
        let doc = Document {
            blocks: vec![
                Block::List(vec![(0, vec![Inline::Text("Main bullet".to_string())])]),
                Block::OrderedList(vec![
                    (3, vec![Inline::Text("Nested one".to_string())]),
                    (3, vec![Inline::Text("Nested two".to_string())]),
                ]),
            ],
        };

        let html = generate_html(&doc);

        let expected = "\
<ul>
<li>Main bullet
<ol>
<li>Nested one</li>
<li>Nested two</li>
</ol>
</li>
</ul>\n";

        assert_eq!(html, expected);
    }

    #[test]
    fn test_nested_mixed_not_nested_when_indent_zero() {
        // When the second list has indent 0 (not nested), they should remain
        // as sibling blocks, not nested.
        let doc = Document {
            blocks: vec![
                Block::OrderedList(vec![(0, vec![Inline::Text("First".to_string())])]),
                Block::List(vec![(0, vec![Inline::Text("Sibling".to_string())])]),
            ],
        };

        let html = generate_html(&doc);

        let expected = "\
<ol>
<li>First</li>
</ol>
<ul>
<li>Sibling</li>
</ul>\n";

        assert_eq!(html, expected);
    }

    #[test]
    fn test_nested_multi_level_mixed() {
        // Deep nesting: ordered → unordered → ordered, each deeper than the last.
        let doc = Document {
            blocks: vec![
                Block::OrderedList(vec![(0, vec![Inline::Text("Top".to_string())])]),
                Block::List(vec![(3, vec![Inline::Text("Middle".to_string())])]),
                Block::OrderedList(vec![(6, vec![Inline::Text("Deep".to_string())])]),
            ],
        };

        let html = generate_html(&doc);

        let expected = "\
<ol>
<li>Top
<ul>
<li>Middle
<ol>
<li>Deep</li>
</ol>
</li>
</ul>
</li>
</ol>\n";

        assert_eq!(html, expected);
    }

    #[test]
    fn test_nested_adjacent_sibling_blocks_after_nest() {
        // A non-nested block after a nested pair should render normally.
        let doc = Document {
            blocks: vec![
                Block::OrderedList(vec![(0, vec![Inline::Text("Item".to_string())])]),
                Block::List(vec![(3, vec![Inline::Text("Sub".to_string())])]),
                Block::Paragraph(vec![Inline::Text("After list.".to_string())]),
            ],
        };

        let html = generate_html(&doc);

        let expected = "\
<ol>
<li>Item
<ul>
<li>Sub</li>
</ul>
</li>
</ol>
<p>After list.</p>\n";

        assert_eq!(html, expected);
    }
}
