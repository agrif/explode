use crate::{Error, Explode};

use std::io::{Error as IOError, ErrorKind, Read, Result};

pub struct ExplodeReader<R> {
    inner: R,
    dec: Explode,
    leftover: Option<u8>,
}

impl<R> ExplodeReader<R>
where
    R: Read,
{
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
    use std::io::{Cursor, Read};

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
}
