# explode

[![build status](https://api.travis-ci.com/agrif/explode.svg?branch=master)](https://travis-ci.com/github/agrif/explode)
[![crates.io](https://img.shields.io/crates/v/explode.svg)](https://crates.io/crates/explode)
[![docs.rs](https://docs.rs/explode/badge.svg)](https://docs.rs/explode)

A decompression implementation for the *implode* algorithm from the
PKWARE Data Compression Library.

This implementation is based on `blast.c` by Mark Adler,
[distributed with zlib][blast].

 [blast]: https://github.com/madler/zlib/tree/master/contrib/blast

## Examples

To decompress a block of bytes in memory, use `explode`.

```rust
let bytes = vec![0x00, 0x04, 0x82, 0x24, 0x25, 0x8f, 0x80, 0x7f];
let result = explode::explode(&bytes)?;
assert_eq!(result, "AIAIAIAIAIAIA".as_bytes());
```

To decompress a `File` or other type that implements `Read`, use
`ExplodeReader`.

```rust
use std::io::Read;
let mut reader = explode::ExplodeReader::new(some_file);
let mut decompressed = vec![];
reader.read_to_end(&mut decompressed)?;
// or other functions from Read
```

For more complicated uses that do not fit into these categories, use
`Explode`.

## License

Licensed under the [MIT license](LICENSE). Unless stated otherwise,
any contributions to this work will also be licensed this way, with no
additional terms or conditions.
