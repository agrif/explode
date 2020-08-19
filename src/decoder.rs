use super::codes::{DecodeResult, Decoder as HuffmanDecoder};
use super::{tables, Error, Result};

use arraydeque::ArrayDeque;

#[derive(Debug)]
pub struct Decoder {
    state: DecoderState<HuffmanDecoder<'static, &'static [u8]>>,

    // header info
    lit: Option<u8>,
    dict: Option<u8>,

    // input management
    input: DecoderInput,

    // store our window (which cannot exceed 4096 bytes)
    window: ArrayDeque<[u8; 4096], arraydeque::behavior::Wrapping>,
}

// hold a byte until it's ready to use
#[derive(Debug)]
enum DecoderInputState {
    Available(u8),
    Taken,
    Waiting,
}

// help manage the bitstream input
#[derive(Debug)]
struct DecoderInput {
    next: DecoderInputState,

    // store unused bits read in
    bitbuf: u32,
    bitcount: u8,
}

// decoder state. D is the Huffman decoder type
#[derive(Debug)]
enum DecoderState<D> {
    Start,
    Length { decoder: D },
    LengthExtra { symbol: usize },
    Distance { len: usize, decoder: D },
    DistanceExtra { len: usize, symbol: usize },
    Copy { idx: usize, len: usize },
    Literal,
    LiteralCoded { decoder: D },
    End,
}

// a buffer you can feed input into
#[derive(Debug)]
pub struct DecoderBuffer<'a> {
    parent: &'a mut Decoder,
    buf: &'a mut [u8],
    pos: usize,
}

impl DecoderInputState {
    fn feed(&mut self, value: u8) {
        if let DecoderInputState::Waiting = self {
            *self = DecoderInputState::Available(value);
        }
    }

    fn take(&mut self) -> Result<u8> {
        match self {
            DecoderInputState::Available(value) => {
                let v = *value;
                *self = DecoderInputState::Taken;
                Ok(v)
            }
            DecoderInputState::Taken => {
                *self = DecoderInputState::Waiting;
                println!("need more input");
                Err(Error::IncompleteInput)
            }
            DecoderInputState::Waiting => {
                panic!("double take");
            }
        }
    }
}

impl DecoderInput {
    // read n bits
    fn bits(&mut self, n: u8) -> Result<u32> {
        while self.bitcount < n {
            self.bitbuf |= (self.next.take()? as u32) << self.bitcount;
            self.bitcount += 8;
        }

        let val = self.bitbuf;
        self.bitbuf >>= n;
        self.bitcount -= n;

        println!("read bits {:?}", val & ((1 << n) - 1));
        Ok(val & ((1 << n) - 1))
    }

    // decode using a table
    fn decode(
        &mut self,
        d: &mut HuffmanDecoder<&'static [u8]>,
    ) -> Result<u8> {
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
}

impl<'a> DecoderBuffer<'a> {
    pub fn feed(&mut self, input: u8) -> Result<()> {
        // lengths are funny -- base val + extra bits
        static LEN_BASE: &[usize] =
            &[3, 2, 4, 5, 6, 7, 8, 9, 10, 12, 16, 24, 40, 72, 136, 264];
        static LEN_EXTRA: &[u8] =
            &[0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8];

        self.parent.input.next.feed(input);

        // first byte is 0 if literals are uncoded, or 1 if coded
        let lit = if let Some(lit) = self.parent.lit {
            lit
        } else {
            println!("read lit");
            let lit = self.parent.input.bits(8)? as u8;
            if lit > 1 {
                return Err(Error::BadLiteralFlag);
            }
            self.parent.lit = Some(lit);
            lit
        };

        // second byte is 4, 5, or 6 for # extra bits in distance code
        // (distance code is 6 + this bits total)
        let dict = if let Some(dict) = self.parent.dict {
            dict
        } else {
            println!("read dict");
            let dict = self.parent.input.bits(8)? as u8;
            if dict < 4 || dict > 6 {
                return Err(Error::BadDictionary);
            }
            self.parent.dict = Some(dict);
            dict
        };

        // decode literals and length/distance pairs
        // state machine rules:
        // each state may only call bits() once
        // and decode() must store the HuffmanDecoder in the state
        loop {
            use DecoderState::*;
            println!("{:?}", self.parent.state);
            match self.parent.state {
                Start => {
                    if self.parent.input.bits(1)? > 0 {
                        // this is a length/distance pair. length first.
                        self.parent.state = Length {
                            decoder: tables::LENGTH.decoder(),
                        };
                    } else {
                        // this is a literal
                        if lit > 0 {
                            self.parent.state = LiteralCoded {
                                decoder: tables::LITERAL.decoder(),
                            };
                        } else {
                            self.parent.state = Literal;
                        }
                    }
                }

                Length { ref mut decoder } => {
                    let symbol = self.parent.input.decode(decoder)? as usize;
                    self.parent.state = LengthExtra { symbol };
                }

                LengthExtra { symbol } => {
                    let len = LEN_BASE[symbol]
                        + self.parent.input.bits(LEN_EXTRA[symbol])? as usize;
                    if len == 519 {
                        // end code
                        self.parent.state = End;
                    } else {
                        // distance next
                        self.parent.state = Distance {
                            len,
                            decoder: tables::DISTANCE.decoder(),
                        };
                    }
                }

                Distance {
                    len,
                    ref mut decoder,
                } => {
                    let symbol = self.parent.input.decode(decoder)? as usize;
                    self.parent.state = DistanceExtra { len, symbol };
                }

                DistanceExtra { len, symbol } => {
                    let extra_bits = if len == 2 { 2 } else { dict };
                    let mut dist =
                        self.parent.input.bits(extra_bits)? as usize + 1;
                    dist += symbol << extra_bits;

                    if dist > self.parent.window.len() {
                        // too far back
                        return Err(Error::BadDistance);
                    }

                    self.parent.state = Copy {
                        idx: self.parent.window.len() - dist,
                        len,
                    };
                }

                Copy {
                    ref mut idx,
                    ref mut len,
                } => {
                    while *len > 0 {
                        if self.pos >= self.buf.len() {
                            // not enough room
                            return Ok(());
                        }

                        let value = self.parent.window[*idx];
                        println!("value {:?}", value);
                        self.parent.window.push_back(value);
                        self.buf[self.pos] = value;
                        self.pos += 1;

                        *len -= 1;
                        *idx += 1;
                        if *idx > self.parent.window.len() {
                            *idx -= self.parent.window.len();
                        }
                    }
                    self.parent.state = Start;
                }

                Literal => {
                    if self.pos >= self.buf.len() {
                        // not enough room
                        return Ok(());
                    }
                    let value = self.parent.input.bits(8)? as u8;
                    println!("value {:?}", value);
                    self.parent.window.push_back(value);
                    self.buf[self.pos] = value;
                    self.pos += 1;
                    self.parent.state = Start;
                }

                LiteralCoded { ref mut decoder } => {
                    if self.pos >= self.buf.len() {
                        // not enough room
                        return Ok(());
                    }
                    let value = self.parent.input.decode(decoder)?;
                    println!("value {:?}", value);
                    self.parent.window.push_back(value);
                    self.buf[self.pos] = value;
                    self.pos += 1;
                    self.parent.state = Start;
                }

                End => {
                    return Ok(());
                }
            }
        }
    }

    pub fn get(self) -> &'a mut [u8] {
        &mut self.buf[..self.pos]
    }
}

impl Decoder {
    pub fn new() -> Self {
        Decoder {
            state: DecoderState::Start,
            lit: None,
            dict: None,
            input: DecoderInput {
                next: DecoderInputState::Waiting,
                bitbuf: 0,
                bitcount: 0,
            },
            window: ArrayDeque::new(),
        }
    }

    pub fn decode_into<'a>(
        &'a mut self,
        buf: &'a mut [u8],
    ) -> DecoderBuffer<'a> {
        DecoderBuffer {
            parent: self,
            buf,
            pos: 0,
        }
    }
}

pub fn decode_with_buffer(data: &[u8], buf: &mut [u8]) -> Result<Vec<u8>> {
    let mut dec = Decoder::new();
    let mut i = 0;
    let mut out = Vec::with_capacity(buf.len());
    loop {
        let mut decbuf = dec.decode_into(buf);
        while i < data.len() {
            match decbuf.feed(data[i]) {
                Ok(()) => {
                    let decompressed = decbuf.get();
                    if decompressed.len() == 0 {
                        // done
                        return Ok(out);
                    }
                    out.extend_from_slice(decompressed);
                    decbuf = dec.decode_into(buf);
                }

                Err(Error::IncompleteInput) => {
                    i += 1;
                    continue;
                }

                Err(e) => return Err(e),
            }
        }

        // out of input
        return Err(Error::IncompleteInput);
    }
}

pub fn decode(data: &[u8]) -> Result<Vec<u8>> {
    let mut buf = [0; 4096];
    decode_with_buffer(data, &mut buf)
}

#[cfg(test)]
mod tests {
    use super::{decode, decode_with_buffer};
    use crate::examples::EXAMPLES;

    #[test]
    fn decode_simple() {
        for (encoded, decoded) in EXAMPLES {
            let ours = decode(encoded).unwrap();
            assert_eq!(*decoded, &ours[..]);
        }
    }

    #[test]
    fn decode_small() {
        let mut buf = [0; 1];
        for (encoded, decoded) in EXAMPLES {
            let ours = decode_with_buffer(encoded, &mut buf).unwrap();
            assert_eq!(*decoded, &ours[..]);
        }
    }
}
