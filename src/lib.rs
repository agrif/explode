//! A decompression implementation for the *implode* algorithm from
//! the PKWARE Data Compression Library.
//!
//! This implementation is based on `blast.c` by Mark Adler,
//! [distributed with zlib][blast].
//!
//!  [blast]: https://github.com/madler/zlib/tree/master/contrib/blast
//!
//! # Examples
//!
//! To decompress a block of bytes in memory, use
//! [`explode`](fn.explode.html).
//!
//! ```
//! # fn main() -> explode::Result<()> {
//! let bytes = vec![0x00, 0x04, 0x82, 0x24, 0x25, 0x8f, 0x80, 0x7f];
//! let result = explode::explode(&bytes)?;
//! assert_eq!(result, "AIAIAIAIAIAIA".as_bytes());
//! # Ok(()) }
//! ```
//!
//! To decompress a [`File`][File] or other type that implements
//! [`Read`][Read], use [`ExplodeReader`](struct.ExplodeReader.html).
//!
//!  [Read]: https://doc.rust-lang.org/std/io/trait.Read.html
//!  [File]: https://doc.rust-lang.org/std/io/struct.File.html
//!
//! ```
//! # fn main() -> explode::Result<()> {
//! # let bytes = vec![0x00, 0x04, 0x82, 0x24, 0x25, 0x8f, 0x80, 0x7f];
//! # let some_file = std::io::Cursor::new(&bytes);
//! use std::io::Read;
//! let mut reader = explode::ExplodeReader::new(some_file);
//! let mut decompressed = vec![];
//! reader.read_to_end(&mut decompressed)?;
//! // or other functions from Read
//! # assert_eq!(decompressed, "AIAIAIAIAIAIA".as_bytes());
//! # Ok(()) }
//! ```
//!
//! For more complicated uses that do not fit into these categories,
//! use [`Explode`](struct.Explode.html).

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
