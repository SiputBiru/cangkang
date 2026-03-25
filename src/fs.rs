use crate::error::CangkangError;
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

pub fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<(), CangkangError> {
    let src = src.as_ref();
    let dst = dst.as_ref();

    // If the public folder doesn't exist yet, just silently return.
    if !src.exists() {
        return Ok(());
    }

    // Make sure the destination folder exists
    fs::create_dir_all(dst)?;

    // Loop through everything in the source folder
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            // If it's a folder, call this exact same function again (Recursion!)
            copy_dir_all(entry.path(), dst_path)?;
        } else {
            // If it's a file, just copy it over
            fs::copy(entry.path(), dst_path)?;
        }
    }

    Ok(())
}
