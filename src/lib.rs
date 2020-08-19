// based on zlib/contrib/blast/blast.c by Mark Adler
// https://github.com/madler/zlib/tree/master/contrib/blast

mod codes;
mod decoder;
mod error;
mod examples;
mod tables;

pub use decoder::{decode, decode_with_buffer, Decoder};
pub use error::{Error, Result};
