// based on zlib/contrib/blast/blast.c by Mark Adler
// https://github.com/madler/zlib/tree/master/contrib/blast

mod error;
pub use error::{Error, Result};

mod codes;
mod tables;

mod decoder;
pub use decoder::Decoder;
