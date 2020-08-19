// canonical Huffman codes
// T can be either &[u8] or Vec<u8>
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalHuffman<T> {
    max_len: usize,
    counts: T,
    symbols: T,
}

// decode state
#[derive(Clone, Debug)]
pub struct Decoder<'a, T> {
    codebook: &'a CanonicalHuffman<T>,
    code: u32,    // code so far
    bits: usize,  // how many bits in the code
    index: usize, // index of first code of this length in symbol table
    first: u32,   // first code of this length
}

// decode result
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DecodeResult {
    Incomplete,
    Invalid,
    Ok(u8),
}

impl CanonicalHuffman<Vec<u8>> {
    // create from a list of packed bits 0xHL
    // where H + 1 is a repeat count, and L is a symbol length
    // returns None if oversubscribed
    // (this is weird -- we use this to compare against zlib's tables)
    pub fn new_from_packed_lengths(packed: &[u8]) -> Option<Self> {
        // should not ever go above 256 symbols
        let mut lengths = [0; 256];
        let mut symbol = 0;
        for b in packed.iter() {
            let len = *b & 0b1111;
            let count = (*b >> 4) + 1;
            for _ in 0..count {
                lengths[symbol] = len;
                symbol += 1;
            }
        }
        Self::new_from_lengths(&lengths[..symbol])
    }

    // create from a list of symbol lengths
    // returns None if oversubscribed
    pub fn new_from_lengths(lengths: &[u8]) -> Option<Self> {
        let max_len = (*lengths.iter().max().unwrap_or(&0) + 1) as usize;
        let mut counts = vec![0; max_len];
        for len in lengths.iter() {
            counts[*len as usize] += 1;
        }

        if counts[0] as usize == lengths.len() {
            // empty table
            return Some(CanonicalHuffman {
                max_len,
                counts,
                symbols: vec![],
            });
        }

        // check for oversubscription
        // one code of length zero
        let mut symbols_left = 1;
        for len in 1..max_len {
            // one more bit doubles number of symbols left
            symbols_left <<= 1;
            // do we have enough left for this size?
            if symbols_left < counts[len] {
                // over-subscribed
                return None;
            }
            // remove the symbols used here
            symbols_left -= counts[len];
        }

        // helper to build symbol table
        let mut offsets = vec![0; max_len];
        offsets[1] = 0;
        for len in 1..(max_len - 1) {
            offsets[len + 1] = offsets[len] + counts[len] as usize;
        }

        // okay, finallly build symbol table
        let mut symbols = vec![0; lengths.len()];
        for symbol in 0..lengths.len() {
            if lengths[symbol] > 0 {
                symbols[offsets[lengths[symbol] as usize]] = symbol as u8;
                offsets[lengths[symbol] as usize] += 1;
            }
        }

        Some(CanonicalHuffman {
            max_len,
            counts,
            symbols,
        })
    }

    // turn a Vec-based table into a slice-based one
    // used mostly for comparison
    pub fn as_ref(&self) -> CanonicalHuffman<&[u8]> {
        CanonicalHuffman {
            max_len: self.max_len,
            counts: &self.counts,
            symbols: &self.symbols,
        }
    }
}

impl<'a> CanonicalHuffman<&'a [u8]> {
    // create a code from an array of code counts per length, and symbols
    // unsafe -- does not check that counts.iter().sum() == symbols.len()
    pub const unsafe fn new(counts: &'a [u8], symbols: &'a [u8]) -> Self {
        CanonicalHuffman {
            max_len: counts.len(),
            counts,
            symbols,
        }
    }
}

impl<T> CanonicalHuffman<T>
where
    T: std::ops::Index<usize, Output = u8>,
{
    pub fn decoder(&self) -> Decoder<T> {
        Decoder {
            codebook: self,
            code: 0,
            bits: 0,
            index: 0,
            first: 0,
        }
    }

    pub fn decode<'a, I>(&self, bits: I) -> Option<u8>
    where
        I: IntoIterator<Item = &'a bool>,
    {
        let mut d = self.decoder();
        for b in bits {
            match d.feed(*b) {
                DecodeResult::Incomplete => continue,
                DecodeResult::Invalid => return None,
                DecodeResult::Ok(c) => return Some(c),
            }
        }
        None
    }
}

impl<'a, T> Decoder<'a, T>
where
    T: std::ops::Index<usize, Output = u8>,
{
    pub fn feed(&mut self, bit: bool) -> DecodeResult {
        self.code |= bit as u32;
        self.bits += 1;

        if self.bits >= self.codebook.max_len {
            // this can happen with an empty table
            return DecodeResult::Invalid;
        }

        let count = self.codebook.counts[self.bits] as u32;
        if self.code < self.first + count {
            // this is a valid symbol
            let i = self.index + (self.code - self.first) as usize;
            DecodeResult::Ok(self.codebook.symbols[i])
        } else if self.code > self.first + count {
            // this is an invalid symbol
            DecodeResult::Invalid
        } else if self.bits + 1 >= self.codebook.max_len {
            // this is also invalid
            DecodeResult::Invalid
        } else {
            // this is an incomplete symbol
            self.index += count as usize;
            self.first += count;
            self.first <<= 1;
            self.code <<= 1;
            DecodeResult::Incomplete
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CanonicalHuffman;
    use super::DecodeResult;

    #[test]
    fn constructors() {
        // A = 10
        // B = 0
        // C = 110
        // D = 111
        let a =
            CanonicalHuffman::new_from_packed_lengths(&[2, 1, 19]).unwrap();
        let b = CanonicalHuffman::new_from_lengths(&[2, 1, 3, 3]).unwrap();
        let c =
            unsafe { CanonicalHuffman::new(&[0, 1, 1, 2], &[1, 0, 2, 3]) };
        assert_eq!(a, b);
        assert_eq!(a.as_ref(), c);
        assert_eq!(b.as_ref(), c);
    }

    #[test]
    fn oversubscribed() {
        // A = 0
        // B = 10
        // C = 11
        // D = ???
        let a = CanonicalHuffman::new_from_lengths(&[1, 2, 2, 3]);
        assert_eq!(a, None);
    }

    #[test]
    fn decode() {
        // A = 10
        // B = 0
        // C = 110
        // D = 111
        let a = CanonicalHuffman::new_from_lengths(&[2, 1, 3, 3]).unwrap();
        assert_eq!(a.decode(&[true, false]), Some(0));
        assert_eq!(a.decode(&[false]), Some(1));
        assert_eq!(a.decode(&[true, true, false]), Some(2));
        assert_eq!(a.decode(&[true, true, true]), Some(3));
    }

    #[test]
    fn undersubscribed() {
        // A = 0
        // B = 100
        let a = CanonicalHuffman::new_from_lengths(&[1, 3]).unwrap();
        assert_eq!(a.decode(&[false]), Some(0));
        assert_eq!(a.decode(&[true, false, false]), Some(1));
        assert_eq!(a.decode(&[true, true, true]), None);
        assert_eq!(a.decode(&[true, true]), None);

        let mut d = a.decoder();
        assert_eq!(d.feed(true), DecodeResult::Incomplete);
        assert_eq!(d.feed(true), DecodeResult::Invalid);

        let mut d = a.decoder();
        assert_eq!(d.feed(true), DecodeResult::Incomplete);
        assert_eq!(d.feed(false), DecodeResult::Incomplete);
        assert_eq!(d.feed(true), DecodeResult::Invalid);
    }

    #[test]
    fn empty() {
        // all codes are invalid in an empty table
        let a = CanonicalHuffman::new_from_lengths(&[]).unwrap();

        assert_eq!(a.decoder().feed(false), DecodeResult::Invalid);
        assert_eq!(a.decoder().feed(true), DecodeResult::Invalid);
    }
}
