//! Encoder for Code93 barcodes.
//!
//! Code93 is intented to improve upon Code39 barcodes by offering a wider array of encodable
//! ASCII characters. It also produces denser barcodes than Code39.
//!
//! Code93 is a continuous, variable-length symbology.
//!
//! NOTE: This encoder currently only supports the basic Code93 implementation and not full-ASCII
//! mode.

use super::helpers::{Vec, vec};
use crate::error::Result;
use crate::sym::{Parse, helpers};
use core::ops::Range;

// Character -> Binary mappings for each of the 47 allowable character.
// The special "full-ASCII" characters are represented with (, ), [, ].
const CHARS: [(char, [u8; 9]); 47] = [
    ('0', [1, 0, 0, 0, 1, 0, 1, 0, 0]),
    ('1', [1, 0, 1, 0, 0, 1, 0, 0, 0]),
    ('2', [1, 0, 1, 0, 0, 0, 1, 0, 0]),
    ('3', [1, 0, 1, 0, 0, 0, 0, 1, 0]),
    ('4', [1, 0, 0, 1, 0, 1, 0, 0, 0]),
    ('5', [1, 0, 0, 1, 0, 0, 1, 0, 0]),
    ('6', [1, 0, 0, 1, 0, 0, 0, 1, 0]),
    ('7', [1, 0, 1, 0, 1, 0, 0, 0, 0]),
    ('8', [1, 0, 0, 0, 1, 0, 0, 1, 0]),
    ('9', [1, 0, 0, 0, 0, 1, 0, 1, 0]),
    ('A', [1, 1, 0, 1, 0, 1, 0, 0, 0]),
    ('B', [1, 1, 0, 1, 0, 0, 1, 0, 0]),
    ('C', [1, 1, 0, 1, 0, 0, 0, 1, 0]),
    ('D', [1, 1, 0, 0, 1, 0, 1, 0, 0]),
    ('E', [1, 1, 0, 0, 1, 0, 0, 1, 0]),
    ('F', [1, 1, 0, 0, 0, 1, 0, 1, 0]),
    ('G', [1, 0, 1, 1, 0, 1, 0, 0, 0]),
    ('H', [1, 0, 1, 1, 0, 0, 1, 0, 0]),
    ('I', [1, 0, 1, 1, 0, 0, 0, 1, 0]),
    ('J', [1, 0, 0, 1, 1, 0, 1, 0, 0]),
    ('K', [1, 0, 0, 0, 1, 1, 0, 1, 0]),
    ('L', [1, 0, 1, 0, 1, 1, 0, 0, 0]),
    ('M', [1, 0, 1, 0, 0, 1, 1, 0, 0]),
    ('N', [1, 0, 1, 0, 0, 0, 1, 1, 0]),
    ('O', [1, 0, 0, 1, 0, 1, 1, 0, 0]),
    ('P', [1, 0, 0, 0, 1, 0, 1, 1, 0]),
    ('Q', [1, 1, 0, 1, 1, 0, 1, 0, 0]),
    ('R', [1, 1, 0, 1, 1, 0, 0, 1, 0]),
    ('S', [1, 1, 0, 1, 0, 1, 1, 0, 0]),
    ('T', [1, 1, 0, 1, 0, 0, 1, 1, 0]),
    ('U', [1, 1, 0, 0, 1, 0, 1, 1, 0]),
    ('V', [1, 1, 0, 0, 1, 1, 0, 1, 0]),
    ('W', [1, 0, 1, 1, 0, 1, 1, 0, 0]),
    ('X', [1, 0, 1, 1, 0, 0, 1, 1, 0]),
    ('Y', [1, 0, 0, 1, 1, 0, 1, 1, 0]),
    ('Z', [1, 0, 0, 1, 1, 1, 0, 1, 0]),
    ('-', [1, 0, 0, 1, 0, 1, 1, 1, 0]),
    ('.', [1, 1, 1, 0, 1, 0, 1, 0, 0]),
    (' ', [1, 1, 1, 0, 1, 0, 0, 1, 0]),
    ('$', [1, 1, 1, 0, 0, 1, 0, 1, 0]),
    ('/', [1, 0, 1, 1, 0, 1, 1, 1, 0]),
    ('+', [1, 0, 1, 1, 1, 0, 1, 1, 0]),
    ('%', [1, 1, 0, 1, 0, 1, 1, 1, 0]),
    ('(', [1, 0, 0, 1, 0, 0, 1, 1, 0]),
    (')', [1, 1, 1, 0, 1, 1, 0, 1, 0]),
    ('[', [1, 1, 1, 0, 1, 0, 1, 1, 0]),
    (']', [1, 0, 0, 1, 1, 0, 0, 1, 0]),
];

// Code93 barcodes must start and end with the '*' special character.
const GUARD: [u8; 9] = [1, 0, 1, 0, 1, 1, 1, 1, 0];
const TERMINATOR: [u8; 1] = [1];

/// The Code93 barcode type.
#[derive(Debug)]
pub struct Code93(Vec<char>);

impl Code93 {
    /// Creates a new barcode.
    /// Returns Result<Code93, Error> indicating parse success.
    pub fn new<T: AsRef<str>>(data: T) -> Result<Code93> {
        Code93::parse(data.as_ref()).map(|d| Code93(d.chars().collect()))
    }

    fn char_encoding(&self, c: char) -> [u8; 9] {
        let encoding = CHARS
            .iter()
            .find(|candidate| candidate.0 == c)
            .map(|(_, encoding)| *encoding);
        helpers::invariant_or(
            encoding,
            [0; 9],
            "Code93 iç verisindeki karakter oluşturucuda doğrulanmış olmalıdır",
        )
    }

    /// Calculates a checksum character using a weighted modulo-47 algorithm.
    fn checksum_char(&self, data: &[char], weight_threshold: usize) -> char {
        assert!(weight_threshold > 0, "Code93 sağlama ağırlığı sıfır olamaz");

        let mut index = 0usize;
        for (offset, character) in data.iter().rev().enumerate() {
            let position = CHARS.iter().position(|candidate| candidate.0 == *character);
            let position = helpers::invariant_or(
                position,
                0,
                "Code93 sağlama hesabındaki karakter doğrulanmış olmalıdır",
            );
            let weight = (offset % weight_threshold) + 1;
            index = (index + (weight * position)) % CHARS.len();
        }

        let character = CHARS
            .get(index % CHARS.len())
            .map(|(character, _)| *character);
        helpers::invariant_or(
            character,
            '0',
            "Code93 sağlama karakteri tablo sınırları içinde olmalıdır",
        )
    }

    /// Calculates the C checksum character using a weighted modulo-47 algorithm.
    fn c_checksum_char(&self) -> char {
        self.checksum_char(&self.0, 20)
    }

    /// Calculates the K checksum character using a weighted modulo-47 algorithm.
    fn k_checksum_char(&self, c_checksum: char) -> char {
        let mut data: Vec<char> = self.0.clone();
        data.push(c_checksum);

        self.checksum_char(&data, 15)
    }

    fn push_encoding(&self, into: &mut Vec<u8>, from: [u8; 9]) {
        into.extend(from.iter().cloned());
    }

    fn payload(&self) -> Vec<u8> {
        let mut enc = vec![];
        let c_checksum = self.c_checksum_char();
        let k_checksum = self.k_checksum_char(c_checksum);

        for &c in &self.0 {
            self.push_encoding(&mut enc, self.char_encoding(c));
        }

        // Checksums.
        self.push_encoding(&mut enc, self.char_encoding(c_checksum));
        self.push_encoding(&mut enc, self.char_encoding(k_checksum));

        enc
    }

    /// Encodes the barcode.
    /// Returns a Vec<u8> of encoded binary digits.
    pub fn encode(&self) -> Vec<u8> {
        let payload = self.payload();

        helpers::join_slices(&[&GUARD, payload.as_slice(), &GUARD, &TERMINATOR])
    }
}

impl Parse for Code93 {
    /// Returns the valid length of data acceptable in this type of barcode.
    /// Code93 barcodes are variable-length.
    fn valid_len() -> Range<u32> {
        1..256
    }

    /// Returns the set of valid characters allowed in this type of barcode.
    fn valid_chars() -> Vec<char> {
        let (chars, _): (Vec<_>, Vec<_>) = CHARS.iter().cloned().unzip();
        chars
    }
}

#[cfg(test)]
mod tests {
    use crate::error::{Error, Result};
    use crate::sym::code93::*;
    #[cfg(not(feature = "std"))]
    use alloc::string::String;
    use core::char;

    fn collapse_vec(v: Vec<u8>) -> String {
        v.iter()
            .filter_map(|digit| char::from_digit(u32::from(*digit), 10))
            .collect()
    }

    #[test]
    fn invalid_length_code93() -> Result<()> {
        let code93 = Code93::new("");

        assert!(matches!(code93, Err(Error::Length)));
        Ok(())
    }

    #[test]
    fn invalid_data_code93() -> Result<()> {
        let code93 = Code93::new("lowerCASE");

        assert!(matches!(code93, Err(Error::Character)));
        Ok(())
    }

    #[test]
    fn code93_encode() -> Result<()> {
        // Tests for data longer than 15, data longer than 20
        let code931 = Code93::new("TEST93")?;
        let code932 = Code93::new("FLAM")?;
        let code933 = Code93::new("99")?;
        let code934 = Code93::new("1111111111111111111111")?;

        assert_eq!(
            collapse_vec(code931.encode()),
            "1010111101101001101100100101101011001101001101000010101010000101011101101001000101010111101"
        );
        assert_eq!(
            collapse_vec(code932.encode()),
            "1010111101100010101010110001101010001010011001001011001010011001010111101"
        );
        assert_eq!(
            collapse_vec(code933.encode()),
            "1010111101000010101000010101101100101000101101010111101"
        );
        assert_eq!(
            collapse_vec(code934.encode()),
            "1010111101010010001010010001010010001010010001010010001010010001010010001010010001010010001010010001010010001010010001010010001010010001010010001010010001010010001010010001010010001010010001010010001010010001000101101110010101010111101"
        );
        Ok(())
    }
}
