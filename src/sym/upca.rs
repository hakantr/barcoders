//! Encoder for UPCA barcodes.
//!
//! UPCA barcodes are common in retail in the US.
//!
//! This module defines types for:
//!   * UPC-A

use crate::error::{Error, Result};
use crate::sym::{Parse, helpers};
use core::ops::Range;
use helpers::Vec;

/// Encoding mappings for UPC barcodes.
/// 1 = bar, 0 = no bar.
///
/// The two indices are:
/// * Left side encodings.
/// * Right side encodings.
pub const ENCODINGS: [[[u8; 7]; 10]; 2] = [
    [
        [0, 0, 0, 1, 1, 0, 1],
        [0, 0, 1, 1, 0, 0, 1],
        [0, 0, 1, 0, 0, 1, 1],
        [0, 1, 1, 1, 1, 0, 1],
        [0, 1, 0, 0, 0, 1, 1],
        [0, 1, 1, 0, 0, 0, 1],
        [0, 1, 0, 1, 1, 1, 1],
        [0, 1, 1, 1, 0, 1, 1],
        [0, 1, 1, 0, 1, 1, 1],
        [0, 0, 0, 1, 0, 1, 1],
    ],
    [
        [1, 1, 1, 0, 0, 1, 0],
        [1, 1, 0, 0, 1, 1, 0],
        [1, 1, 0, 1, 1, 0, 0],
        [1, 0, 0, 0, 0, 1, 0],
        [1, 0, 1, 1, 1, 0, 0],
        [1, 0, 0, 1, 1, 1, 0],
        [1, 0, 1, 0, 0, 0, 0],
        [1, 0, 0, 0, 1, 0, 0],
        [1, 0, 0, 1, 0, 0, 0],
        [1, 1, 1, 0, 1, 0, 0],
    ],
];

/// The left-hand guard pattern.
pub const LEFT_GUARD: [u8; 3] = [1, 0, 1];
/// The middle guard pattern.
pub const MIDDLE_GUARD: [u8; 5] = [0, 1, 0, 1, 0];
/// The right-hand guard pattern.
pub const RIGHT_GUARD: [u8; 3] = [1, 0, 1];

/// The UPCA barcode type.
#[derive(Debug)]
pub struct UPCA(Vec<u8>);

impl UPCA {
    /// Creates a new barcode.
    /// Returns Result<UPCA, Error> indicating parse success.
    pub fn new<T: AsRef<str>>(data: T) -> Result<UPCA> {
        let d = UPCA::parse(data.as_ref())?;
        let digits = helpers::parse_digits(d)?;

        let upca = UPCA(digits.iter().copied().take(11).collect());

        // If checksum digit is provided, check the checksum.
        let checksum = upca.checksum_digit();
        if digits.get(11).is_some_and(|provided| checksum != *provided) {
            return Err(Error::Checksum);
        }

        Ok(upca)
    }

    /// Calculates the checksum digit using a modulo-10 weighting algorithm.
    fn checksum_digit(&self) -> u8 {
        helpers::modulo_10_checksum(self.0.as_slice(), false)
    }

    fn checksum_encoding(&self) -> [u8; 7] {
        self.char_encoding(1, self.checksum_digit())
    }

    fn char_encoding(&self, side: usize, d: u8) -> [u8; 7] {
        let encoding = ENCODINGS
            .get(side)
            .and_then(|encodings| encodings.get(usize::from(d)))
            .copied();
        helpers::invariant_or(
            encoding,
            [0; 7],
            "UPC-A kodlama dizini oluşturucuda doğrulanmış olmalıdır",
        )
    }

    fn left_digits(&self) -> &[u8] {
        helpers::invariant_or(
            self.0.get(0..6),
            &[],
            "UPC-A sol basamakları oluşturucuda doğrulanmış olmalıdır",
        )
    }

    fn right_digits(&self) -> &[u8] {
        helpers::invariant_or(
            self.0.get(6..),
            &[],
            "UPC-A sağ basamakları oluşturucuda doğrulanmış olmalıdır",
        )
    }

    fn left_payload(&self) -> Vec<u8> {
        let slices: Vec<[u8; 7]> = self
            .left_digits()
            .iter()
            .map(|d| self.char_encoding(0, *d))
            .collect();

        helpers::join_iters(slices.iter())
    }

    fn right_payload(&self) -> Vec<u8> {
        let slices: Vec<[u8; 7]> = self
            .right_digits()
            .iter()
            .map(|d| self.char_encoding(1, *d))
            .collect();

        helpers::join_iters(slices.iter())
    }

    /// Encodes the barcode.
    /// Returns a Vec<u8> of binary digits.
    pub fn encode(&self) -> Vec<u8> {
        let left_payload = self.left_payload();
        let right_payload = self.right_payload();
        let checksum = self.checksum_encoding();

        helpers::join_slices(&[
            &LEFT_GUARD,
            left_payload.as_slice(),
            &MIDDLE_GUARD,
            right_payload.as_slice(),
            &checksum,
            &RIGHT_GUARD,
        ])
    }
}

impl Parse for UPCA {
    /// Returns the valid length of data acceptable in this type of barcode.
    fn valid_len() -> Range<u32> {
        11..12
    }

    /// Returns the set of valid characters allowed in this type of barcode.
    fn valid_chars() -> Vec<char> {
        helpers::decimal_chars()
    }
}

#[cfg(test)]
mod tests {
    use crate::error::{Error, Result};
    use crate::sym::upca::*;
    #[cfg(not(feature = "std"))]
    use alloc::string::String;
    use core::char;

    fn collapse_vec(v: Vec<u8>) -> String {
        v.iter()
            .filter_map(|digit| char::from_digit(u32::from(*digit), 10))
            .collect()
    }

    #[test]
    fn new_upca() -> Result<()> {
        let upca = UPCA::new("72527273070");

        assert!(upca.is_ok());
        Ok(())
    }

    #[test]
    fn invalid_data_upca() -> Result<()> {
        let upca = UPCA::new("012345612a45");

        assert!(matches!(upca, Err(Error::Character)));
        Ok(())
    }

    #[test]
    fn invalid_len_upca() -> Result<()> {
        let upca = UPCA::new("1234561234589");

        assert!(matches!(upca, Err(Error::Length)));
        Ok(())
    }

    #[test]
    fn invalid_checksum_upca() -> Result<()> {
        let upca = UPCA::new("725272730705");

        assert!(matches!(upca, Err(Error::Checksum)));
        Ok(())
    }

    #[test]
    fn valid_checksum_upca() -> Result<()> {
        let upca = UPCA::new("725272730706");

        assert!(upca.is_ok());
        Ok(())
    }

    #[test]
    fn upce_encode() -> Result<()> {
        let upca1 = UPCA::new("72527273070")?;
        let upca2 = UPCA::new("738312014094")?;
        let upca3 = UPCA::new("095421076611")?;

        assert_eq!(
            collapse_vec(upca1.encode()),
            "10101110110010011011000100100110111011001001101010100010010000101110010100010011100101010000101"
        );
        assert_eq!(
            collapse_vec(upca2.encode()),
            "10101110110111101011011101111010011001001001101010111001011001101011100111001011101001011100101"
        );
        assert_eq!(
            collapse_vec(upca3.encode()),
            "10100011010001011011000101000110010011001100101010111001010001001010000101000011001101100110101"
        );
        Ok(())
    }
}
