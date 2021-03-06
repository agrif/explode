use super::codes::CanonicalHuffman;

// these tables are created unsafely, staticly
// they are compared against computed known-good tables from zlib in tests

pub static LITERAL: CanonicalHuffman<&'static [u8]> = unsafe {
    CanonicalHuffman::new(
        &[0, 0, 0, 0, 1, 11, 20, 21, 16, 7, 5, 10, 91, 74],
        &[
            0x20, 0x45, 0x61, 0x65, 0x69, 0x6c, 0x6e, 0x6f, 0x72, 0x73, 0x74,
            0x75, 0x2d, 0x31, 0x41, 0x43, 0x44, 0x49, 0x4c, 0x4e, 0x4f, 0x52,
            0x53, 0x54, 0x62, 0x63, 0x64, 0x66, 0x67, 0x68, 0x6d, 0x70, 0x0a,
            0x0d, 0x28, 0x29, 0x2c, 0x2e, 0x30, 0x32, 0x33, 0x34, 0x35, 0x37,
            0x38, 0x3d, 0x42, 0x46, 0x4d, 0x50, 0x55, 0x6b, 0x77, 0x09, 0x22,
            0x27, 0x2a, 0x2f, 0x36, 0x39, 0x3a, 0x47, 0x48, 0x57, 0x5b, 0x5f,
            0x76, 0x78, 0x79, 0x2b, 0x3e, 0x4b, 0x56, 0x58, 0x59, 0x5d, 0x21,
            0x24, 0x26, 0x71, 0x7a, 0x00, 0x3c, 0x3f, 0x4a, 0x51, 0x5a, 0x5c,
            0x6a, 0x7b, 0x7c, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
            0x0b, 0x0c, 0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16,
            0x17, 0x18, 0x19, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x23, 0x25, 0x3b,
            0x40, 0x5e, 0x60, 0x7d, 0x7e, 0x7f, 0xb0, 0xb1, 0xb2, 0xb3, 0xb4,
            0xb5, 0xb6, 0xb7, 0xb8, 0xb9, 0xba, 0xbb, 0xbc, 0xbd, 0xbe, 0xbf,
            0xc0, 0xc1, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7, 0xc8, 0xc9, 0xca,
            0xcb, 0xcc, 0xcd, 0xce, 0xcf, 0xd0, 0xd1, 0xd2, 0xd3, 0xd4, 0xd5,
            0xd6, 0xd7, 0xd8, 0xd9, 0xda, 0xdb, 0xdc, 0xdd, 0xde, 0xdf, 0xe1,
            0xe5, 0xe9, 0xee, 0xf2, 0xf3, 0xf4, 0x1a, 0x80, 0x81, 0x82, 0x83,
            0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8a, 0x8b, 0x8c, 0x8d, 0x8e,
            0x8f, 0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99,
            0x9a, 0x9b, 0x9c, 0x9d, 0x9e, 0x9f, 0xa0, 0xa1, 0xa2, 0xa3, 0xa4,
            0xa5, 0xa6, 0xa7, 0xa8, 0xa9, 0xaa, 0xab, 0xac, 0xad, 0xae, 0xaf,
            0xe0, 0xe2, 0xe3, 0xe4, 0xe6, 0xe7, 0xe8, 0xea, 0xeb, 0xec, 0xed,
            0xef, 0xf0, 0xf1, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc,
            0xfd, 0xfe, 0xff,
        ],
    )
};

pub static LENGTH: CanonicalHuffman<&'static [u8]> = unsafe {
    CanonicalHuffman::new(
        &[0, 0, 1, 3, 3, 4, 3, 2],
        &[
            0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0xa, 0xb, 0xc,
            0xd, 0xe, 0xf,
        ],
    )
};

pub static DISTANCE: CanonicalHuffman<&'static [u8]> = unsafe {
    CanonicalHuffman::new(
        &[0, 0, 1, 0, 2, 4, 15, 26, 16],
        &[
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a,
            0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15,
            0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
            0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x2b,
            0x2c, 0x2d, 0x2e, 0x2f, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36,
            0x37, 0x38, 0x39, 0x3a, 0x3b, 0x3c, 0x3d, 0x3e, 0x3f,
        ],
    )
};

#[cfg(test)]
mod tests {
    use super::CanonicalHuffman;

    #[test]
    fn literal() {
        let zlib_literal = CanonicalHuffman::new_from_packed_lengths(&[
            11, 124, 8, 7, 28, 7, 188, 13, 76, 4, 10, 8, 12, 10, 12, 10, 8,
            23, 8, 9, 7, 6, 7, 8, 7, 6, 55, 8, 23, 24, 12, 11, 7, 9, 11, 12,
            6, 7, 22, 5, 7, 24, 6, 11, 9, 6, 7, 22, 7, 11, 38, 7, 9, 8, 25,
            11, 8, 11, 9, 12, 8, 12, 5, 38, 5, 38, 5, 11, 7, 5, 6, 21, 6, 10,
            53, 8, 7, 24, 10, 27, 44, 253, 253, 253, 252, 252, 252, 13, 12,
            45, 12, 45, 12, 61, 12, 45, 44, 173,
        ])
        .unwrap();
        assert_eq!(zlib_literal.as_ref(), super::LITERAL);
    }

    #[test]
    fn length() {
        let zlib_length = CanonicalHuffman::new_from_packed_lengths(&[
            2, 35, 36, 53, 38, 23,
        ])
        .unwrap();
        assert_eq!(zlib_length.as_ref(), super::LENGTH);
    }

    #[test]
    fn distance() {
        let zlib_distance = CanonicalHuffman::new_from_packed_lengths(&[
            2, 20, 53, 230, 247, 151, 248,
        ])
        .unwrap();
        assert_eq!(zlib_distance.as_ref(), super::DISTANCE);
    }
}
