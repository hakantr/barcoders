//! Encoder for EAN-8 barcodes.
//!
//! EAN-8 barcodes are EAN style barcodes for smaller packages on products like
//! cigaretts, chewing gum, etc where package space is limited.

use crate::error::{Error, Result};
use crate::sym::ean13::{ENCODINGS, LEFT_GUARD, MIDDLE_GUARD, RIGHT_GUARD};
use crate::sym::{Parse, helpers};
use core::ops::Range;
use helpers::{Vec, vec};

/// The EAN-8 barcode type.
#[derive(Debug)]
pub struct EAN8(Vec<u8>);

impl EAN8 {
    /// Creates a new barcode.
    /// Returns Result<EAN8, Error> indicating parse success.
    pub fn new<T: AsRef<str>>(data: T) -> Result<EAN8> {
        let d = EAN8::parse(data.as_ref())?;
        let digits = helpers::parse_digits(d)?;

        let ean8 = EAN8(digits.iter().copied().take(7).collect());

        // If checksum digit is provided, check the checksum.
        let checksum = ean8.checksum_digit();
        if digits.get(7).is_some_and(|provided| checksum != *provided) {
            return Err(Error::Checksum);
        }

        Ok(ean8)
    }

    /// Calculates the checksum digit using a weighting algorithm.
    fn checksum_digit(&self) -> u8 {
        helpers::modulo_10_checksum(self.0.as_slice(), false)
    }

    fn number_system_digits(&self) -> &[u8] {
        helpers::invariant_or(
            self.0.get(0..2),
            &[],
            "EAN-8 sayı sistemi basamakları oluşturucuda doğrulanmış olmalıdır",
        )
    }

    fn number_system_encoding(&self) -> Vec<u8> {
        let mut ns = vec![];

        for d in self.number_system_digits() {
            ns.extend(self.char_encoding(0, *d).iter().cloned());
        }

        ns
    }

    fn checksum_encoding(&self) -> [u8; 7] {
        self.char_encoding(2, self.checksum_digit())
    }

    fn char_encoding(&self, side: usize, d: u8) -> [u8; 7] {
        let encoding = ENCODINGS
            .get(side)
            .and_then(|encodings| encodings.get(usize::from(d)))
            .copied();
        helpers::invariant_or(
            encoding,
            [0; 7],
            "EAN-8 kodlama dizini oluşturucuda doğrulanmış olmalıdır",
        )
    }

    fn left_digits(&self) -> &[u8] {
        helpers::invariant_or(
            self.0.get(2..4),
            &[],
            "EAN-8 sol basamakları oluşturucuda doğrulanmış olmalıdır",
        )
    }

    fn right_digits(&self) -> &[u8] {
        helpers::invariant_or(
            self.0.get(4..),
            &[],
            "EAN-8 sağ basamakları oluşturucuda doğrulanmış olmalıdır",
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
            .map(|d| self.char_encoding(2, *d))
            .collect();

        helpers::join_iters(slices.iter())
    }

    /// Encodes the barcode.
    /// Returns a Vec<u8> of binary digits.
    pub fn encode(&self) -> Vec<u8> {
        let number_system = self.number_system_encoding();
        let left_payload = self.left_payload();
        let right_payload = self.right_payload();
        let checksum = self.checksum_encoding();

        helpers::join_slices(&[
            &LEFT_GUARD,
            number_system.as_slice(),
            left_payload.as_slice(),
            &MIDDLE_GUARD,
            right_payload.as_slice(),
            &checksum,
            &RIGHT_GUARD,
        ])
    }
}

impl Parse for EAN8 {
    /// Returns the valid length of data acceptable in this type of barcode.
    fn valid_len() -> Range<u32> {
        7..8
    }

    /// Returns the set of valid characters allowed in this type of barcode.
    fn valid_chars() -> Vec<char> {
        helpers::decimal_chars()
    }
}

#[cfg(test)]
mod tests {
    use crate::error::{Error, Result};
    use crate::sym::ean8::*;
    #[cfg(not(feature = "std"))]
    use alloc::string::String;
    use core::char;

    fn collapse_vec(v: Vec<u8>) -> String {
        v.iter()
            .filter_map(|digit| char::from_digit(u32::from(*digit), 10))
            .collect()
    }

    #[test]
    fn new_ean8() -> Result<()> {
        let ean8 = EAN8::new("1234567");

        assert!(ean8.is_ok());
        Ok(())
    }

    #[test]
    fn invalid_data_ean8() -> Result<()> {
        let ean8 = EAN8::new("1234er1");

        assert!(matches!(ean8, Err(Error::Character)));
        Ok(())
    }

    #[test]
    fn invalid_len_ean8() -> Result<()> {
        let ean8 = EAN8::new("1111112222222333333");

        assert!(matches!(ean8, Err(Error::Length)));
        Ok(())
    }

    #[test]
    fn invalid_checksum_ean8() -> Result<()> {
        let ean8 = EAN8::new("88023020");

        assert!(matches!(ean8, Err(Error::Checksum)));
        Ok(())
    }

    #[test]
    fn ean8_encode() -> Result<()> {
        let ean81 = EAN8::new("5512345")?; // Check digit: 7
        let ean82 = EAN8::new("9834651")?; // Check digit: 3

        assert_eq!(
            collapse_vec(ean81.encode()),
            "1010110001011000100110010010011010101000010101110010011101000100101"
        );
        assert_eq!(
            collapse_vec(ean82.encode()),
            "1010001011011011101111010100011010101010000100111011001101010000101"
        );
        Ok(())
    }
}
