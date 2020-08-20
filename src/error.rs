/// Error type produced by decompression.
///
/// Of these, `IncompleteInput` is special as in some circumstances it
/// is possible to recover by providing further input. This is
/// documented wherever it is possible.
#[derive(Debug)]
pub enum Error {
    /// A normal IO error.
    IO(std::io::Error),
    /// The input is incomplete. Decompression may still succeed if
    /// you provide more input.
    IncompleteInput,
    /// The literal flag in the header is invalid.
    BadLiteralFlag,
    /// The dictionary size in the header is invalid.
    BadDictionary,
    /// A repeat command tried to read past the beginning of the buffer.
    BadDistance,
}

/// Result type for decompression functions.
pub type Result<T> = std::result::Result<T, Error>;

impl std::convert::From<std::io::Error> for Error {
    fn from(v: std::io::Error) -> Self {
        Error::IO(v)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::IO(err) => write!(f, "{}", err),
            Error::IncompleteInput => write!(f, "unexpected end of input"),
            Error::BadLiteralFlag => {
                write!(f, "literal flag not zero or one")
            }
            Error::BadDictionary => write!(f, "dictionary size not in 4..=6"),
            Error::BadDistance => write!(f, "distance is too far back"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::IO(err) => Some(err),
            _ => None,
        }
    }
}
