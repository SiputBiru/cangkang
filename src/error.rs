use std::fmt;
use std::io;

#[derive(Debug)]
pub enum CangkangError {
    // IO Error with option context
    Io(String, io::Error),

    // Markdown parsing errors
    Parse { message: String, line: usize },

    // Frontmatter syntax errors
    Frontmatter(String),

    // HTML Template errors
    Template(String),
}

impl fmt::Display for CangkangError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CangkangError::Io(ctx, err) => {
                if ctx.is_empty() {
                    write!(f, "File system Error: {}", err)
                } else {
                    write!(f, "File system Error at '{}': {}", ctx, err)
                }
            }
            CangkangError::Parse { message, line } => {
                write!(f, "Parsing Error at line {}: {}", line, message)
            }
            CangkangError::Frontmatter(msg) => write!(f, "Frontmatter Error: {}", msg),
            CangkangError::Template(msg) => write!(f, "Template Error: {}", msg),
        }
    }
}

impl std::error::Error for CangkangError {}

pub trait IoContext<T> {
    fn with_ctx<S: Into<String>>(self, ctx: S) -> Result<T, CangkangError>;
}

impl<T> IoContext<T> for Result<T, io::Error> {
    fn with_ctx<S: Into<String>>(self, ctx: S) -> Result<T, CangkangError> {
        self.map_err(|e| CangkangError::Io(ctx.into(), e))
    }
}
