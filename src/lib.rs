// based on zlib/contrib/blast/blast.c by Mark Adler
// https://github.com/madler/zlib/tree/master/contrib/blast

mod error;
pub use error::{Error, Result};

mod codes;
mod tables;

pub struct Decoder<R> {
    input: R,

    // store unused bits
    bitbuf: u8,
    bitcount: u8,
}

impl<R> Decoder<R>
where
    R: std::io::Read,
{
    pub fn new(input: R) -> Self {
        Decoder {
            input,
            bitbuf: 0,
            bitcount: 0,
        }
    }

    // read a single byte
    fn byte(&mut self) -> Result<u8> {
        let mut byte = 0;
        match self.input.read(std::slice::from_mut(&mut byte))? {
            0 => Err(Error::IncompleteInput),
            _ => Ok(byte),
        }
    }

    // read n bits
    fn bits(&mut self, n: u8) -> Result<u32> {
        let mut val = self.bitbuf as u32;
        while self.bitcount < n {
            val |= (self.byte()? as u32) << self.bitcount;
            self.bitcount += 8;
        }

        self.bitbuf = (val >> n) as u8;
        self.bitcount -= n;

        Ok(val & ((1 << n) - 1))
    }

    pub fn decomp(&mut self) -> Result<()> {
        // first byte is 0 if literals are uncoded, or 1 if coded
        let lit = self.bits(8)?;
        if lit > 1 {
            return Err(Error::BadLiteralFlag);
        }

        // second byte is 4, 5, or 6 for # extra bits in distance code
        // (distance code is 6 + this bits total)
        let dict = self.bits(8)?;
        if dict < 4 || dict > 6 {
            return Err(Error::BadDictionary);
        }

        // decode literals and length/distance pairs
        loop {
            if self.bits(1)? > 0 {
                // this is a length/distance pair
                panic!("length/distance pair")
            } else {
                // this is a literal
                panic!("literal")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn basic() {
        let c = std::io::Cursor::new([
            0x00, 0x04, 0x82, 0x24, 0x25, 0x8f, 0x80, 0x7f,
        ]);
        let mut d = super::Decoder::new(c);
        d.decomp().unwrap();
    }
}
