use std::fs;
use std::io;
use std::path::Path;

pub fn read_markdown_file<P: AsRef<Path>>(file_path: P) -> io::Result<String> {
    fs::read_to_string(file_path)
}

pub fn write_html_file<P: AsRef<Path>>(file_path: P, content: &str) -> io::Result<()> {
    let path = file_path.as_ref();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, content)
}
