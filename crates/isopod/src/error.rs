use std::fmt;
use std::io;

/// Error type for ISO operations
#[derive(Debug)]
pub enum Error {
    /// I/O error from underlying operations
    Io(io::Error),
    
    /// Invalid ISO format
    InvalidFormat(String),
    
    /// Validation error during ISO creation
    ValidationError(String),
    
    /// Path error (too long, invalid characters, etc.)
    PathError(String),
    
    /// Size limit exceeded
    SizeLimit(String),
    
    /// UDF specific error
    UdfError(String),
    
    /// Joliet extension error
    JolietError(String),
    
    /// Rock Ridge extension error
    RockRidgeError(String),
    
    /// El Torito boot extension error
    ElToritoError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(err) => write!(f, "I/O error: {}", err),
            Error::InvalidFormat(msg) => write!(f, "Invalid ISO format: {}", msg),
            Error::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            Error::PathError(msg) => write!(f, "Path error: {}", msg),
            Error::SizeLimit(msg) => write!(f, "Size limit exceeded: {}", msg),
            Error::UdfError(msg) => write!(f, "UDF error: {}", msg),
            Error::JolietError(msg) => write!(f, "Joliet error: {}", msg),
            Error::RockRidgeError(msg) => write!(f, "Rock Ridge error: {}", msg),
            Error::ElToritoError(msg) => write!(f, "El Torito error: {}", msg),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}