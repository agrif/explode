use super::codes::{DecodeResult, Decoder};
use super::{tables, Error, Result};

use arraydeque::ArrayDeque;

/// Low-level decompression interface.
///
/// This provides low-level access to the decompression algorithm. If
/// possible, prefer using [`explode`](fn.explode.html) or
/// [`ExplodeReader`](struct.ExplodeReader.html) as they are simpler
/// to use.
///
/// The usual control flow with this interface is to provide a buffer
/// to decompress into with [`with_buffer`](#method.with_buffer), and
/// then to feed the resulting
/// [`ExplodeBuffer`](struct.ExplodeBuffer.html) handle with bytes
/// until it returns `Ok`. Then you can retrieve the filled portion of
/// the buffer containing your decompressed data.
///
/// ```
/// # fn main() -> explode::Result<()> {
/// use explode::{Error, Explode};
///
/// // some test data to decompress
/// let input = vec![0x00, 0x04, 0x82, 0x24, 0x25, 0x8f, 0x80, 0x7f];
/// // which byte we are currently feeding in
/// let mut i = 0;
/// // our output buffer
/// let mut outbuf: [u8; 256] = [0; 256];
///
/// // decompress
/// let mut ex = explode::Explode::new();
/// let mut exbuf = ex.with_buffer(&mut outbuf);
/// // loop while we have more input, and decompression is not done
/// while i < input.len() && !exbuf.done() {
///     // note we feed exbuf the *same byte* every loop, until it requests
///     // more input with Error::IncompleteInput.
///     match exbuf.feed(input[i]) {
///         Ok(()) => {
///             // buffer is full. use exbuf.get() to get the filled portion
///             println!("{:?}", exbuf.get());
///             // compression may not be finished, so reset and loop again
///             exbuf.reset();
///         }
///
///         Err(Error::IncompleteInput) => {
///             // advance our input cursor
///             i += 1;
///         }
///
///         Err(e) => {
///             // any other error is a sign the input is invalid
///             panic!("{:?}", e);
///         }
///     }
/// }
///
/// if !exbuf.done() {
///     // we ran out of input, but decompression isn't done!
///     panic!("unexpected end of input");
/// }
/// # Ok(()) }
/// ```
///
/// Be careful that the input byte you provide to
/// [`ExplodeBuffer::feed`](struct.ExplodeBuffer.html#method.feed)
/// only changes when requested by
/// [`Error::IncompleteInput`](enum.Error.html#variant.IncompleteInput). If
/// the input changes at any other time, decompression will fail or
/// produce incorrect output.
#[derive(Debug)]
pub struct Explode {
    state: ExplodeState<Decoder<'static, &'static [u8]>>,

    // header info
    lit: Option<u8>,
    dict: Option<u8>,

    // input management
    input: ExplodeInput,

    // store our window (which cannot exceed 4096 bytes)
    window: ArrayDeque<[u8; 4096], arraydeque::behavior::Wrapping>,
}

// hold a byte until it's ready to use
#[derive(Debug)]
enum ExplodeInputState {
    Available(u8),
    Taken,
    Waiting,
}

// help manage the bitstream input
#[derive(Debug)]
struct ExplodeInput {
    next: ExplodeInputState,

    // store unused bits read in
    bitbuf: u32,
    bitcount: u8,
}

// explode state. D is the Huffman decoder type
#[derive(Debug)]
enum ExplodeState<D> {
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

/// A handle to feed input to the decompressor.
///
/// This is the primary interface for low-level decompression. You can
/// get an instance of this by providing an output buffer to
/// [`Explode::with_buffer`](struct.Explode.html#method.with_buffer).
///
/// For a high-level example of how to use this interface, see
/// [`Explode`](struct.Explode.html).
#[derive(Debug)]
pub struct ExplodeBuffer<'a> {
    parent: &'a mut Explode,
    buf: &'a mut [u8],
    pos: usize,
}

impl ExplodeInputState {
    fn feed(&mut self, value: u8) {
        if let ExplodeInputState::Waiting = self {
            *self = ExplodeInputState::Available(value);
        }
    }

    fn take(&mut self) -> Result<u8> {
        match self {
            ExplodeInputState::Available(value) => {
                let v = *value;
                *self = ExplodeInputState::Taken;
                Ok(v)
            }
            ExplodeInputState::Taken => {
                *self = ExplodeInputState::Waiting;
                Err(Error::IncompleteInput)
            }
            ExplodeInputState::Waiting => {
                panic!("double take");
            }
        }
    }
}

impl ExplodeInput {
    // read n bits
    fn bits(&mut self, n: u8) -> Result<u32> {
        while self.bitcount < n {
            self.bitbuf |= (self.next.take()? as u32) << self.bitcount;
            self.bitcount += 8;
        }

        let val = self.bitbuf;
        self.bitbuf >>= n;
        self.bitcount -= n;

        Ok(val & ((1 << n) - 1))
    }

    // decode using a table
    fn decode(&mut self, d: &mut Decoder<&'static [u8]>) -> Result<u8> {
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

impl<'a> ExplodeBuffer<'a> {
    /// Feed in a byte `input` to decompress.
    ///
    /// Signals a full output buffer by returning `Ok(())`. You can
    /// then get a reference to the full buffer with
    /// [`get`](#method.get), and reset the output buffer to empty
    /// with [`reset`](#method.reset).
    ///
    /// Note that you should feed in the same byte *repeatedly* to
    /// this function, until it signals it is ready for more input by
    /// returning
    /// [`Error::IncompleteInput`](enum.Error.html#variant.IncompleteInput).
    /// Doing anything else will result in a decompression failure or
    /// bad output.
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
        // and decode() must store the HuffmanExplode in the state
        loop {
            use ExplodeState::*;
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
                        self.parent.window.push_back(value);
                        self.buf[self.pos] = value;
                        self.pos += 1;

                        *len -= 1;
                        *idx += 1;
                        if *idx >= self.parent.window.len() {
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

    /// Get a reference to the filled portion of the output buffer.
    ///
    /// This is usually called after [`feed`](#method.feed) returns `Ok(())`.
    pub fn get(&self) -> &[u8] {
        &self.buf[..self.pos]
    }

    /// Return the amount of output produced so far.
    pub fn len(&self) -> usize {
        self.pos
    }

    /// Reset the output buffer to empty.
    ///
    /// Note that this does *not* reset the entire decompressor state.
    pub fn reset(&mut self) {
        self.pos = 0;
    }

    /// Returns true if decompression is finished.
    ///
    /// This does the same thing as
    /// [`Explode::done`](struct.Explode.html#method.done) but is
    /// usable while a `ExplodeBuffer` is still in scope.
    pub fn done(&self) -> bool {
        self.parent.done()
    }
}

impl Explode {
    /// Create a new Explode decompression state.
    pub fn new() -> Self {
        Explode {
            state: ExplodeState::Start,
            lit: None,
            dict: None,
            input: ExplodeInput {
                next: ExplodeInputState::Waiting,
                bitbuf: 0,
                bitcount: 0,
            },
            window: ArrayDeque::new(),
        }
    }

    /// Provide a buffer to decompress into.
    ///
    /// This returns a [`ExplodeBuffer`](struct.ExplodeBuffer.html)
    /// handle that is used for feeding input to decompress and other
    /// operations.
    pub fn with_buffer<'a>(
        &'a mut self,
        buf: &'a mut [u8],
    ) -> ExplodeBuffer<'a> {
        ExplodeBuffer {
            parent: self,
            buf,
            pos: 0,
        }
    }

    /// Returns true if decompression is finished.
    ///
    /// If this function can't be used because a
    /// [`ExplodeBuffer`](struct.ExplodeBuffer.html) is currently
    /// borrowing this object mutably, you can use
    /// [`ExplodeBuffer::done`](struct.ExplodeBuffer.html#method.done)
    /// instead.
    pub fn done(&self) -> bool {
        if let ExplodeState::End = self.state {
            true
        } else {
            false
        }
    }
}

/// Decompress a block of `data` in memory, using the given auxiliary
/// buffer `buf`.
///
/// This gives you control over the size of the internal buffer
/// used. If you do not need that control, use
/// [`explode`](fn.explode.html) instead.
///
/// ```
/// # fn main() -> explode::Result<()> {
/// let mut buf: [u8; 1] = [0; 1];
/// let bytes = vec![0x00, 0x04, 0x82, 0x24, 0x25, 0x8f, 0x80, 0x7f];
/// let result = explode::explode_with_buffer(&bytes, &mut buf)?;
/// assert_eq!(result, "AIAIAIAIAIAIA".as_bytes());
/// # Ok(()) }
/// ```
pub fn explode_with_buffer(data: &[u8], buf: &mut [u8]) -> Result<Vec<u8>> {
    let mut dec = Explode::new();
    let mut i = 0;
    let mut out = Vec::with_capacity(buf.len());
    loop {
        let mut decbuf = dec.with_buffer(buf);
        while i < data.len() {
            match decbuf.feed(data[i]) {
                Ok(()) => {
                    let decompressed = decbuf.get();
                    out.extend_from_slice(decompressed);
                    if decbuf.done() {
                        // we're done
                        return Ok(out);
                    }
                    decbuf.reset();
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

/// Decompress a block of `data` in memory.
///
/// ```
/// # fn main() -> explode::Result<()> {
/// let bytes = vec![0x00, 0x04, 0x82, 0x24, 0x25, 0x8f, 0x80, 0x7f];
/// let result = explode::explode(&bytes)?;
/// assert_eq!(result, "AIAIAIAIAIAIA".as_bytes());
/// # Ok(()) }
/// ```
///
/// This function will internally decompress the given memory in
/// blocks of 4096 bytes. If you wish to use a different block size,
/// see [`explode_with_buffer`](fn.explode_with_buffer.html).
pub fn explode(data: &[u8]) -> Result<Vec<u8>> {
    let mut buf = [0; 4096];
    explode_with_buffer(data, &mut buf)
}

#[cfg(test)]
mod tests {
    use super::{explode, explode_with_buffer, Error};
    use crate::examples::EXAMPLES;

    #[test]
    fn explode_simple() {
        for (encoded, decoded) in EXAMPLES {
            let ours = explode(encoded).unwrap();
            assert_eq!(*decoded, &ours[..]);
        }
    }

    #[test]
    fn explode_small() {
        let mut buf = [0; 1];
        for (encoded, decoded) in EXAMPLES {
            let ours = explode_with_buffer(encoded, &mut buf).unwrap();
            assert_eq!(*decoded, &ours[..]);
        }
    }

    #[test]
    fn explode_incomplete() {
        for (encoded, _) in EXAMPLES {
            let ours = explode(&encoded[..encoded.len() - 1]);
            match ours {
                Err(Error::IncompleteInput) => (),
                _ => panic!("incorrectly parsed incomplete input"),
            }
        }
    }

    #[test]
    fn explode_extra() {
        for (encoded, decoded) in EXAMPLES {
            let mut encodedplus: Vec<u8> = encoded.iter().cloned().collect();
            encodedplus.push(42);
            let ours = explode(&encodedplus).unwrap();
            assert_eq!(*decoded, &ours[..]);
        }
    }
}
