use crate::{Error, Explode};

use std::io::{Error as IOError, ErrorKind, Read, Result};

/// A [`Read`][Read] wrapper that decompresses.
///
///  [Read]: https://doc.rust-lang.org/std/io/trait.Read.html
///
/// If you have a [`File`][File] or any other type that implements
/// [`Read`][Read], you can use this wrapper to decompress it as you
/// read it.
///
///  [File]: https://doc.rust-lang.org/std/io/struct.File.html
///
/// ```
/// # fn main() -> explode::Result<()> {
/// # let bytes = vec![0x00, 0x04, 0x82, 0x24, 0x25, 0x8f, 0x80, 0x7f];
/// # let some_file = std::io::Cursor::new(&bytes);
/// use std::io::Read;
/// let mut reader = explode::ExplodeReader::new(some_file);
/// let mut decompressed = vec![];
/// reader.read_to_end(&mut decompressed)?;
/// // or other functions from Read
/// # assert_eq!(decompressed, "AIAIAIAIAIAIA".as_bytes());
/// # Ok(()) }
/// ```
pub struct ExplodeReader<R> {
    inner: R,
    dec: Explode,
    leftover: Option<u8>,
}

impl<R> ExplodeReader<R>
where
    R: Read,
{
    /// Create a new decompression wrapper around `inner`.
    pub fn new(inner: R) -> Self {
        ExplodeReader {
            inner,
            dec: Explode::new(),
            leftover: None,
        }
    }
}

impl<R> Read for ExplodeReader<R>
where
    R: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if self.dec.done() {
            return Ok(0);
        }

        let mut decbuf = self.dec.with_buffer(buf);
        let mut byte = 0;
        loop {
            if let Some(v) = self.leftover {
                byte = v;
                self.leftover = None;
            } else {
                if self.inner.read(std::slice::from_mut(&mut byte))? == 0 {
                    break;
                }
            }

            match decbuf.feed(byte) {
                Ok(()) => {
                    self.leftover = Some(byte);
                    return Ok(decbuf.len());
                }
                Err(Error::IncompleteInput) => continue,
                Err(e) => {
                    return Err(IOError::new(ErrorKind::InvalidData, e))
                }
            }
        }
        Err(IOError::new(
            ErrorKind::UnexpectedEof,
            Error::IncompleteInput,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::ExplodeReader;
    use crate::examples::EXAMPLES;
    use std::io::{Cursor, ErrorKind, Read};

    #[test]
    fn reader() {
        for (encoded, decoded) in EXAMPLES {
            let mut r = ExplodeReader::new(Cursor::new(encoded));
            let mut ours = Vec::with_capacity(decoded.len());
            r.read_to_end(&mut ours).unwrap();
            assert_eq!(*decoded, &ours[..]);
        }
    }

    #[test]
    fn reader_small() {
        for (encoded, decoded) in EXAMPLES {
            let mut r = ExplodeReader::new(Cursor::new(encoded));
            let mut byte = 0;
            let mut ours = Vec::with_capacity(decoded.len());
            while r.read(std::slice::from_mut(&mut byte)).unwrap() > 0 {
                ours.push(byte);
            }
            assert_eq!(*decoded, &ours[..]);
        }
    }

    #[test]
    fn reader_incomplete() {
        for (encoded, decoded) in EXAMPLES {
            let mut r = ExplodeReader::new(Cursor::new(
                &encoded[..encoded.len() - 1],
            ));
            let mut ours = Vec::with_capacity(decoded.len());
            match r.read_to_end(&mut ours) {
                Err(e) => assert_eq!(e.kind(), ErrorKind::UnexpectedEof),
                _ => panic!("incorrectly parsed incomplete input"),
            }
        }
    }

    #[test]
    fn reader_extra() {
        for (encoded, decoded) in EXAMPLES {
            let mut encodedplus: Vec<u8> = encoded.iter().cloned().collect();
            encodedplus.push(42);
            let mut inner = Cursor::new(&encodedplus);
            let mut r = ExplodeReader::new(&mut inner);
            let mut ours = Vec::with_capacity(decoded.len());
            r.read_to_end(&mut ours).unwrap();
            assert_eq!(*decoded, &ours[..]);

            ours.clear();
            inner.read_to_end(&mut ours).unwrap();
            assert_eq!(vec![42], ours);
        }
    }
}
