use crate::error::{CangkangError, IoContext};
use std::fs;
use std::path::Path;

pub fn read_markdown_file<P: AsRef<Path>>(file_path: P) -> Result<String, CangkangError> {
    let path = file_path.as_ref();
    fs::read_to_string(path).with_ctx(path.to_string_lossy())
}

pub fn write_html_file<P: AsRef<Path>>(file_path: P, content: &str) -> Result<(), CangkangError> {
    let path = file_path.as_ref();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_ctx(parent.to_string_lossy())?;
    }

    fs::write(path, content).with_ctx(path.to_string_lossy())
}

pub fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<(), CangkangError> {
    let src = src.as_ref();
    let dst = dst.as_ref();

    if !src.exists() {
        return Ok(());
    }

    fs::create_dir_all(dst).with_ctx(dst.to_string_lossy())?;

    for entry in fs::read_dir(src).with_ctx(src.to_string_lossy())? {
        let entry = entry.with_ctx("directory entry")?;
        let file_type = entry.file_type().with_ctx("file type")?;
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_all(entry.path(), dst_path)?;
        } else {
            fs::copy(entry.path(), &dst_path).with_ctx(format!(
                "{} to {}",
                entry.path().display(),
                dst_path.display()
            ))?;
        }
    }

    Ok(())
}
