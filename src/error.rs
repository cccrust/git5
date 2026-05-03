use std::fmt;

#[derive(Debug, Clone)]
pub enum Git5Error {
    NotARepository(String),
    ObjectNotFound(String),
    InvalidObject(String),
    InvalidRef(String),
    Conflict(String),
    IoError(String),
    ParseError(String),
}

impl fmt::Display for Git5Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Git5Error::NotARepository(path) => write!(f, "Not a git5 repository: {}", path),
            Git5Error::ObjectNotFound(hash) => write!(f, "Object not found: {}", hash),
            Git5Error::InvalidObject(msg) => write!(f, "Invalid object: {}", msg),
            Git5Error::InvalidRef(name) => write!(f, "Invalid reference: {}", name),
            Git5Error::Conflict(msg) => write!(f, "Conflict: {}", msg),
            Git5Error::IoError(msg) => write!(f, "IO error: {}", msg),
            Git5Error::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for Git5Error {}

impl From<std::io::Error> for Git5Error {
    fn from(err: std::io::Error) -> Self {
        Git5Error::IoError(err.to_string())
    }
}

impl From<hex::FromHexError> for Git5Error {
    fn from(err: hex::FromHexError) -> Self {
        Git5Error::ParseError(format!("Invalid hex: {}", err))
    }
}

pub type Result<T> = std::result::Result<T, Git5Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Git5Error::NotARepository("/some/path".to_string());
        assert_eq!(err.to_string(), "Not a git5 repository: /some/path");

        let err = Git5Error::ObjectNotFound("abc123".to_string());
        assert_eq!(err.to_string(), "Object not found: abc123");
    }

    #[test]
    fn test_error_debug() {
        let err = Git5Error::Conflict("merge conflict".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("Conflict"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let git5_err: Git5Error = io_err.into();
        assert!(matches!(git5_err, Git5Error::IoError(_)));
    }
}