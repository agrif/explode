// based on zlib/contrib/blast/blast.c by Mark Adler
// https://github.com/madler/zlib/tree/master/contrib/blast

mod codes;
mod error;
mod examples;
mod explode;
mod reader;
mod tables;

pub use self::explode::{
    explode, explode_with_buffer, Explode, ExplodeBuffer,
};
pub use error::{Error, Result};
pub use reader::ExplodeReader;
