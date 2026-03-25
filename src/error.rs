use std::fmt;
use std::io;

#[derive(Debug)]
pub enum CangkangError {
    Io(io::Error),
    Parse { message: String, line: usize },
    Template(String),
}

impl fmt::Display for CangkangError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CangkangError::Io(err) => write!(f, "File system Error: {}", err),
            CangkangError::Parse { message, line } => {
                write!(f, "Parsing Error at line {}: {}", line, message)
            }
            CangkangError::Template(msg) => write!(f, "Template Error: {}", msg),
        }
    }
}

impl From<io::Error> for CangkangError {
    fn from(err: io::Error) -> Self {
        CangkangError::Io(err)
    }
}

impl std::error::Error for CangkangError {}
