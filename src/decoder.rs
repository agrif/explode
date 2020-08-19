use super::{Error, Result, tables};
use super::codes::{CanonicalHuffman, DecodeResult};

use arraydeque::ArrayDeque;

pub struct Decoder<R> {
    input: R,

    // store unused bits read in
    bitbuf: u8,
    bitcount: u8,

    // store our window (which cannot exceed 4096 bytes)
    window: ArrayDeque<[u8; 4096], arraydeque::behavior::Wrapping>,
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
            window: ArrayDeque::new(),
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

    // decode using a table
    fn decode(
        &mut self,
        table: &CanonicalHuffman<&'static [u8]>,
    ) -> Result<u8> {
        let mut d = table.decoder();
        loop {
            // codes in this format are inverted from canonical
            let bit = self.bits(1)? != 1;
            match d.feed(bit) {
                DecodeResult::Incomplete => continue,
                DecodeResult::Invalid => panic!(
                    "Codebooks are under-subscribed but should not be!"
                ),
                DecodeResult::Ok(v) => return Ok(v),
            }
        }
    }

    pub fn decomp(&mut self) -> Result<Vec<u8>> {
        // lengths are funny -- base val + extra bits
        static LEN_BASE: &[usize] =
            &[3, 2, 4, 5, 6, 7, 8, 9, 10, 12, 16, 24, 40, 72, 136, 264];
        static LEN_EXTRA: &[u8] =
            &[0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8];

        // first byte is 0 if literals are uncoded, or 1 if coded
        let lit = self.bits(8)?;
        if lit > 1 {
            return Err(Error::BadLiteralFlag);
        }

        // second byte is 4, 5, or 6 for # extra bits in distance code
        // (distance code is 6 + this bits total)
        let dict = self.bits(8)? as u8;
        if dict < 4 || dict > 6 {
            return Err(Error::BadDictionary);
        }

        // decode literals and length/distance pairs
        let mut out = vec![];
        loop {
            if self.bits(1)? > 0 {
                // this is a length/distance pair
                // length first
                let symbol = self.decode(&tables::LENGTH)? as usize;
                let len =
                    LEN_BASE[symbol] + self.bits(LEN_EXTRA[symbol])? as usize;
                if len == 519 {
                    // end code
                    return Ok(out);
                }

                // now distance
                let extra_bits = if len == 2 { 2 } else { dict };
                let mut dist =
                    (self.decode(&tables::DISTANCE)? as usize) << extra_bits;
                dist += self.bits(extra_bits)? as usize + 1;

                if dist > self.window.len() {
                    // too far back
                    return Err(Error::BadDistance);
                }

                // perform a copy
                let mut idx = self.window.len() - dist;
                for _ in 0..len {
                    let value = self.window[idx];
                    idx += 1;
                    if idx > self.window.len() {
                        idx -= self.window.len();
                    }

                    self.window.push_back(value);
                    out.push(value);
                }
            } else {
                // this is a literal
                let value = if lit > 0 {
                    self.decode(&tables::LITERAL)?
                } else {
                    self.bits(8)? as u8
                };

                self.window.push_back(value);
                out.push(value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn aiaiaiaiaiaia() {
        let c = std::io::Cursor::new([
            0x00, 0x04, 0x82, 0x24, 0x25, 0x8f, 0x80, 0x7f,
        ]);
        let mut d = super::Decoder::new(c);
        let result = d.decomp().unwrap();
        assert_eq!(
            result,
            vec![
                0x41, 0x49, 0x41, 0x49, 0x41, 0x49, 0x41, 0x49, 0x41, 0x49,
                0x41, 0x49, 0x41
            ]
        );
    }
}
