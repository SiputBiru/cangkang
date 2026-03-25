use std::path::Path;

use crate::error::CangkangError;
use crate::fs;
use crate::lexer::Lexer;
use crate::parser::{Block, Document, Inline, Parser};

#[derive(Debug)]
pub struct PageInfo {
    pub title: String,
    pub url: String,
}

pub fn build_site() -> Result<(), CangkangError> {
    println!("Starting Cangkang...");

    let template_path = "templates/base.html";
    let template = std::fs::read_to_string(template_path).map_err(|e| {
        CangkangError::Template(format!(
            "Could not read template at '{}': {}",
            template_path, e
        ))
    })?;

    if !template.contains("{{ content }}") {
        return Err(CangkangError::Template(
            "The base.html template is missing the '{{ content }}' placeholder.".to_string(),
        ));
    }

    let content_dir = Path::new("content");
    let dist_dir = Path::new("dist");

    if !content_dir.exists() {
        return Err(CangkangError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "The 'content' directory does not exist.",
        )));
    }

    let mut all_pages = process_directory(content_dir, content_dir, dist_dir, &template)?;
    all_pages.sort_by(|a, b| a.title.cmp(&b.title));

    build_index(&all_pages, dist_dir, &template)?;

    println!(
        "Build complete! Check the '{}' directory.",
        dist_dir.display()
    );
    Ok(())
}

fn build_index(pages: &[PageInfo], dist_dir: &Path, template: &str) -> Result<(), CangkangError> {
    let mut index_content = String::from("<h1>SiputBiru's Notes</h1>\n<ul>\n");
    for page in pages {
        index_content.push_str(&format!(
            "  <li><a href=\"./{}\">{}</a></li>\n",
            page.url, page.title
        ));
    }
    index_content.push_str("</ul>\n");

    let final_index_html = template
        .replace("{{ content }}", &index_content)
        .replace("{{ body_class }}", "is-home");
    let index_path = dist_dir.join("index.html");

    fs::write_html_file(&index_path, &final_index_html)?;
    println!("  -> Generated index.html!");

    Ok(())
}

fn process_directory(
    dir: &Path,
    base_content_dir: &Path,
    base_dist_dir: &Path,
    template: &str,
) -> Result<Vec<PageInfo>, CangkangError> {
    let mut pages = Vec::new();

    let entries = std::fs::read_dir(dir)?;

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();

        if path.is_dir() {
            let mut sub_pages =
                process_directory(&path, base_content_dir, base_dist_dir, template)?;
            pages.append(&mut sub_pages);
        } else if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            let file_stem = path.file_stem().and_then(|s| s.to_str());
            // Skip BOTH index.md and 404.md
            if file_stem == Some("index") || file_stem == Some("404") {
                if file_stem == Some("404") {
                    let _ = compile_file(&path, base_content_dir, base_dist_dir, template)?;
                }
                continue;
            }

            let page_info = compile_file(&path, base_content_dir, base_dist_dir, template)?;
            pages.push(page_info);
        }
    }
    Ok(pages)
}

fn compile_file(
    input_path: &Path,
    base_content_dir: &Path,
    base_dist_dir: &Path,
    template: &str,
) -> Result<PageInfo, CangkangError> {
    println!("Compiling: {}", input_path.display());

    let md_content = fs::read_markdown_file(input_path)?;
    let lexer = Lexer::new(&md_content);
    let mut parser = Parser::new(lexer);

    // parser.parse_document() already returns Result<Document, CangkangError>
    let document = parser.parse_document()?;

    let title = extract_title(&document);
    let html_output = crate::html::generate_html(&document);
    let final_html = template
        .replace("{{ content }}", &html_output)
        .replace("{{ body_class }}", "is-post");

    let relative_path = input_path.strip_prefix(base_content_dir).unwrap();
    let mut output_path = base_dist_dir.join(relative_path);
    output_path.set_extension("html");

    let url_path = output_path.strip_prefix(base_dist_dir).unwrap();
    let url = url_path.to_string_lossy().replace("\\", "./");

    fs::write_html_file(&output_path, &final_html)?;

    Ok(PageInfo { title, url })
}

fn extract_title(doc: &Document) -> String {
    for block in &doc.blocks {
        if let Block::Heading { level: 1, content } = block {
            let mut title = String::new();
            for inline in content {
                match inline {
                    Inline::Text(t) | Inline::Bold(t) | Inline::Italic(t) => title.push_str(t),
                    _ => {}
                }
            }
            return if title.is_empty() {
                "Untitled".to_string()
            } else {
                title
            };
        }
    }
    "Untitled".to_string()
}
