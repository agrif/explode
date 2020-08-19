#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    IncompleteInput,
    BadLiteralFlag,
    BadDictionary,
    BadDistance,
}

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
