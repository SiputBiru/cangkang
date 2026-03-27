use std::path::Path;
use std::time::Instant;

use crate::error::CangkangError;
use crate::frontmatter;
use crate::fs;
use crate::lexer::Lexer;
use crate::parser::{Block, Document, Inline, Parser};

// simple logging things
macro_rules! log_info { ($($arg:tt)*) => { println!("[INFO] {}", format_args!($($arg)*)); } }
macro_rules! log_success { ($($arg:tt)*) => { println!("[ OK ] {}", format_args!($($arg)*)); } }
macro_rules! log_warn { ($($arg:tt)*) => { eprintln!("[WARN] {}", format_args!($($arg)*)); } }

#[derive(Debug)]
pub struct PageInfo {
    pub title: String,
    pub url: String,
    pub date: String,
    pub pinned: bool,
}

pub fn build_site() -> Result<(), CangkangError> {
    let start_time = Instant::now();
    log_info!("Starting Cangkang compiler...");

    let index_template_path = "templates/index_template.html";
    let index_template = std::fs::read_to_string(index_template_path).map_err(|e| {
        CangkangError::Template(format!(
            "Could not read index/home template at '{}': {}",
            index_template_path, e
        ))
    })?;
    if !index_template.contains("{{ content }}") {
        return Err(CangkangError::Template(
            "The index_template.html template is missing the '{{ content }}' placeholder."
                .to_string(),
        ));
    }

    let post_template_path = "templates/post_template.html";
    let post_template = std::fs::read_to_string(post_template_path).map_err(|e| {
        CangkangError::Template(format!(
            "Could not read post template at '{}': {}",
            post_template_path, e
        ))
    })?;
    if !post_template.contains("{{ content }}") {
        return Err(CangkangError::Template(
            "The post_template.html template is missing the '{{ content }}' placeholder."
                .to_string(),
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

    let public_dir = Path::new("public");
    if public_dir.exists() {
        log_info!("Copying static assets from public/...");
        if let Err(e) = fs::copy_dir_all(public_dir, dist_dir) {
            log_warn!("Failed to copy public directory: {}", e);
        }
    }

    let mut all_pages = process_directory(content_dir, content_dir, dist_dir, &post_template)?;
    all_pages.sort_by(|a, b| (b.pinned, &b.date).cmp(&(a.pinned, &a.date)));

    build_index(&all_pages, dist_dir, &index_template)?;

    let duration = start_time.elapsed();
    log_success!(
        "Build complete in {:.2?}! Check the '{}' directory.",
        duration,
        dist_dir.display()
    );

    Ok(())
}

fn build_index(
    pages: &[PageInfo],
    dist_dir: &Path,
    index_template: &str,
) -> Result<(), CangkangError> {
    let mut index_content = String::new();

    let index_md_path = Path::new("content/index.md");
    let mut page_title = String::from("SiputBiru's Notes"); // Default

    if index_md_path.exists() {
        let raw_content = fs::read_markdown_file(index_md_path)?;
        let (metadata, md_content) = frontmatter::parse(&raw_content)?;

        if metadata.title != "Untitled" {
            page_title = metadata.title;
        }

        let lexer = Lexer::new(md_content);
        let mut parser = Parser::new(lexer);
        let document = parser.parse_document()?;
        index_content.push_str(&crate::html::generate_html(&document));
    }

    // index_content.push_str("\n<hr>\n<h3>All Posts</h3>\n<ul class=\"index-list\">\n");
    index_content.push_str("\n<ul class=\"index-list\">\n");

    for page in pages {
        let date_str = if page.date.is_empty() {
            String::new()
        } else {
            format!(
                " <span style=\"color: var(--muted-text); font-size: 0.85em;\">({})</span>",
                page.date
            )
        };

        let pin_class = if page.pinned {
            " class=\"pinned-post\""
        } else {
            ""
        };

        index_content.push_str(&format!(
            "<li{}><a href=\"./{}\">{}</a>{}</li>\n",
            pin_class, page.url, page.title, date_str
        ));
    }
    index_content.push_str("</ul>\n");

    let final_index_html = index_template
        .replace("{{ content }}", &index_content)
        .replace("{{ title }}", &page_title)
        .replace("{{ root_dir }}", "./");

    let index_path = dist_dir.join("index.html");

    fs::write_html_file(&index_path, &final_index_html)?;
    log_success!("Generated index.html");

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
    log_info!("Compiling: {}", input_path.display());

    let raw_content = fs::read_markdown_file(input_path)?;
    let (metadata, md_content) = frontmatter::parse(&raw_content)?;

    let lexer = Lexer::new(md_content);
    let mut parser = Parser::new(lexer);
    let document = parser.parse_document()?;

    let mut title = metadata.title.clone();
    if title == "Untitled" {
        title = extract_title(&document);
    }

    let relative_path = input_path.strip_prefix(base_content_dir).unwrap();
    let depth = relative_path.components().count() - 1;
    let file_stem = input_path.file_stem().and_then(|s| s.to_str());

    let root_dir = if file_stem == Some("404") {
        String::from("/")
    } else if depth == 0 {
        String::from("./")
    } else {
        "../".repeat(depth)
    };

    let mut output_path = base_dist_dir.join(relative_path);
    output_path.set_extension("html");

    let url_path = output_path.strip_prefix(base_dist_dir).unwrap();
    // let url = url_path.to_string_lossy().replace("\\", "/");
    let url = url_path
        .to_string_lossy()
        .replace("\\", "/")
        .replace(".html", "");

    let html_output = crate::html::generate_html(&document).replace("{{ root_dir }}", &root_dir);

    let final_html = template
        .replace("{{ content }}", &html_output)
        .replace("{{ title }}", &title)
        .replace("{{ date }}", &metadata.date)
        .replace("{{ root_dir }}", &root_dir);

    fs::write_html_file(&output_path, &final_html)?;

    Ok(PageInfo {
        title,
        url,
        date: metadata.date,
        pinned: metadata.pinned,
    })
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
