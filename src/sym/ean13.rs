//! EAN barkodlarını kodlayan bileşen.
//!
//! EAN13 barkodları perakendede çok yaygındır. Bir süpermarketten satın aldığınız ürünlerin yaklaşık
//! %90'ı EAN13 kullanır.
//!
//! Bu modül şu türleri tanımlar:
//!   * EAN-13
//!   * Bookland
//!   * JAN

use crate::error::{Error, Result};
use crate::sym::{Parse, helpers};
use core::ops::RangeInclusive;
use helpers::Vec;

/// EAN barkodları için kodlama eşlemeleri.
/// `1` çubuğu, `0` boşluğu belirtir.
///
/// Üç dizin şunlardır:
/// * Sol taraf A (tek eşlik).
/// * Sol taraf B (çift eşlik).
/// * Sağ taraf kodlamaları.
pub const ENCODINGS: [[[u8; 7]; 10]; 3] = [
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
        [0, 1, 0, 0, 1, 1, 1],
        [0, 1, 1, 0, 0, 1, 1],
        [0, 0, 1, 1, 0, 1, 1],
        [0, 1, 0, 0, 0, 0, 1],
        [0, 0, 1, 1, 1, 0, 1],
        [0, 1, 1, 1, 0, 0, 1],
        [0, 0, 0, 0, 1, 0, 1],
        [0, 0, 1, 0, 0, 0, 1],
        [0, 0, 0, 1, 0, 0, 1],
        [0, 0, 1, 0, 1, 1, 1],
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

/// Sol taraftaki basamakların eşliğini (tek/çift), barkod verisinin sayı sistemi bölümündeki ilk
/// basamağa göre eşler.
const PARITY: [[usize; 5]; 10] = [
    [0, 0, 0, 0, 0],
    [0, 1, 0, 1, 1],
    [0, 1, 1, 0, 1],
    [0, 1, 1, 1, 0],
    [1, 0, 0, 1, 1],
    [1, 1, 0, 0, 1],
    [1, 1, 1, 0, 0],
    [1, 0, 1, 0, 1],
    [1, 0, 1, 1, 0],
    [1, 1, 0, 1, 0],
];

/// Sol koruma deseni.
pub const LEFT_GUARD: [u8; 3] = [1, 0, 1];
/// Orta koruma deseni.
pub const MIDDLE_GUARD: [u8; 5] = [0, 1, 0, 1, 0];
/// Sağ koruma deseni.
pub const RIGHT_GUARD: [u8; 3] = [1, 0, 1];

/// EAN-13 barkod türü.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EAN13(Vec<u8>);

/// Bookland barkod türü.
/// Bookland barkodları, 978 sayı sistemini kullanan EAN-13 barkodlarıdır.
pub type Bookland = EAN13;

/// JAN barkod türü.
/// JAN barkodları, 49 sayı sistemini kullanan EAN-13 barkodlarıdır.
pub type JAN = EAN13;

impl EAN13 {
    /// Yeni bir barkod oluşturur.
    /// Girdinin çözümlenme sonucunu `Result<EAN13, Error>` olarak döndürür.
    pub fn new<T: AsRef<str>>(data: T) -> Result<EAN13> {
        let d = EAN13::parse(data.as_ref())?;
        let digits = helpers::parse_digits(d)?;

        let ean13 = EAN13(digits.iter().copied().take(12).collect());

        // Sağlama basamağı verilmişse doğruluğunu denetle.
        let checksum = ean13.checksum_digit();
        if let Some(provided) = digits.get(12).filter(|provided| checksum != **provided) {
            return Err(Error::Checksum {
                expected: checksum,
                found: *provided,
            });
        }

        Ok(ean13)
    }

    /// Modülo-10 ağırlıklandırma algoritmasıyla sağlama basamağını hesaplar.
    fn checksum_digit(&self) -> u8 {
        helpers::modulo_10_checksum(self.0.as_slice(), true)
    }

    fn number_system_digit(&self) -> u8 {
        helpers::invariant_or(
            self.0.get(1).copied(),
            0,
            "EAN-13 sayı sistemi basamağı oluşturucuda doğrulanmış olmalıdır",
        )
    }

    fn number_system_encoding(&self) -> [u8; 7] {
        self.char_encoding(0, self.number_system_digit())
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
            "EAN-13 kodlama dizini oluşturucuda doğrulanmış olmalıdır",
        )
    }

    fn left_digits(&self) -> &[u8] {
        helpers::invariant_or(
            self.0.get(2..7),
            &[],
            "EAN-13 sol basamakları oluşturucuda doğrulanmış olmalıdır",
        )
    }

    fn right_digits(&self) -> &[u8] {
        helpers::invariant_or(
            self.0.get(7..),
            &[],
            "EAN-13 sağ basamakları oluşturucuda doğrulanmış olmalıdır",
        )
    }

    fn parity_mapping(&self) -> [usize; 5] {
        let first_digit = helpers::invariant_or(
            self.0.first().copied(),
            0,
            "EAN-13 ilk basamağı oluşturucuda doğrulanmış olmalıdır",
        );
        let parity = PARITY.get(usize::from(first_digit)).copied();
        helpers::invariant_or(
            parity,
            [0; 5],
            "EAN-13 eşlik düzeni doğrulanmış basamakla eşleşmelidir",
        )
    }

    fn left_payload(&self) -> Vec<u8> {
        let parity = self.parity_mapping();
        let slices: Vec<[u8; 7]> = self
            .left_digits()
            .iter()
            .zip(parity.iter())
            .map(|(d, s)| self.char_encoding(*s, *d))
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

    /// Barkodu kodlar.
    /// İkili basamakları bir `Vec<u8>` içinde döndürür.
    pub fn encode(&self) -> Vec<u8> {
        let number_system = self.number_system_encoding();
        let left_payload = self.left_payload();
        let right_payload = self.right_payload();
        let checksum = self.checksum_encoding();

        helpers::join_slices(&[
            &LEFT_GUARD,
            &number_system,
            left_payload.as_slice(),
            &MIDDLE_GUARD,
            right_payload.as_slice(),
            &checksum,
            &RIGHT_GUARD,
        ])
    }
}

impl Parse for EAN13 {
    /// Bu barkod türünün kabul ettiği geçerli veri uzunluğu aralığını döndürür.
    fn valid_len() -> RangeInclusive<usize> {
        12..=13
    }

    /// Bu barkod türünde kullanılabilen geçerli karakter kümesini döndürür.
    fn valid_chars() -> Vec<char> {
        helpers::decimal_chars()
    }
}

#[cfg(test)]
mod tests {
    use crate::error::{Error, Result};
    use crate::sym::ean13::*;
    #[cfg(not(feature = "std"))]
    use alloc::string::String;
    use core::char;

    fn collapse_vec(v: Vec<u8>) -> String {
        v.iter()
            .filter_map(|digit| char::from_digit(u32::from(*digit), 10))
            .collect()
    }

    #[test]
    fn new_ean13() -> Result<()> {
        let ean13 = EAN13::new("123456123456");

        assert!(ean13.is_ok());
        Ok(())
    }

    #[test]
    fn new_bookland() -> Result<()> {
        let bookland = Bookland::new("978456123456");

        assert!(bookland.is_ok());
        Ok(())
    }

    #[test]
    fn invalid_data_ean13() -> Result<()> {
        let ean13 = EAN13::new("1234er123412");

        assert!(matches!(ean13, Err(Error::Character { .. })));
        Ok(())
    }

    #[test]
    fn invalid_len_ean13() -> Result<()> {
        let ean13 = EAN13::new("1111112222222333333");

        assert!(matches!(ean13, Err(Error::Length { .. })));
        Ok(())
    }

    #[test]
    fn invalid_checksum_ean13() -> Result<()> {
        let ean13 = EAN13::new("8801051294881");

        assert!(matches!(ean13, Err(Error::Checksum { .. })));
        Ok(())
    }

    #[test]
    fn ean13_encode_as_bookland() -> Result<()> {
        let bookland1 = Bookland::new("978345612345")?; // Sağlama basamağı: 5
        let bookland2 = Bookland::new("978118999561")?; // Sağlama basamağı: 5

        assert_eq!(
            collapse_vec(bookland1.encode()),
            "10101110110001001010000101000110111001010111101010110011011011001000010101110010011101001110101"
        );
        assert_eq!(
            collapse_vec(bookland2.encode()),
            "10101110110001001011001100110010001001000101101010111010011101001001110101000011001101001110101"
        );
        Ok(())
    }

    #[test]
    fn ean13_encode() -> Result<()> {
        let ean131 = EAN13::new("750103131130")?; // Sağlama basamağı: 5
        let ean132 = EAN13::new("983465123499")?; // Sağlama basamağı: 5

        assert_eq!(
            collapse_vec(ean131.encode()),
            "10101100010100111001100101001110111101011001101010100001011001101100110100001011100101110100101"
        );
        assert_eq!(
            collapse_vec(ean132.encode()),
            "10101101110100001001110101011110111001001100101010110110010000101011100111010011101001000010101"
        );
        Ok(())
    }
}
