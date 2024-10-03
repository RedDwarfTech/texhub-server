use std::fmt;
use std::io;

#[derive(Debug)]
pub enum TexhubError {
    IoError(io::Error),
    SizeError(&'static str),
}

impl fmt::Display for TexhubError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TexhubError::IoError(err) => write!(f, "IO error: {}", err),
            TexhubError::SizeError(msg) => write!(f, "Size error: {}", msg),
        }
    }
}

impl std::error::Error for TexhubError {}

impl From<io::Error> for TexhubError {
    fn from(err: io::Error) -> TexhubError {
        TexhubError::IoError(err)
    }
}