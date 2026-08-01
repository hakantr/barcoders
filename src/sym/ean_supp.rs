//! İki ve beş basamaklı ek EAN barkodlarını kodlayan bileşenler.
//!
//! EAN-2 barkodları dergi ve gazetelerde sayı numarasını belirtmek için kullanılır.
//!
//! EAN-5 barkodları çoğunlukla kitapların önerilen perakende fiyatını belirtir.
//!
//! Bu ek barkodlar yanlarında tam bir EAN-13 barkodu olmadan kullanılmaz.

use crate::error::{Error, Result};
use crate::sym::ean13::ENCODINGS;
use crate::sym::{Parse, helpers};
use core::ops::RangeInclusive;
use helpers::{Vec, vec};

const LEFT_GUARD: [u8; 4] = [1, 0, 1, 1];

/// EAN-5 barkodlarının eşliğini (tek/çift), sağlama basamağına göre eşler.
const EAN5_PARITY: [[usize; 5]; 10] = [
    [1, 1, 0, 0, 0],
    [1, 0, 1, 0, 0],
    [1, 0, 0, 1, 0],
    [1, 0, 0, 0, 1],
    [0, 1, 1, 0, 0],
    [0, 0, 1, 1, 0],
    [0, 0, 0, 1, 1],
    [0, 1, 0, 1, 0],
    [0, 1, 0, 0, 1],
    [0, 0, 1, 0, 1],
];

/// EAN-2 barkodlarının eşliğini (tek/çift), sağlama basamağına göre eşler.
const EAN2_PARITY: [[usize; 5]; 4] = [
    [0, 0, 0, 0, 0],
    [0, 1, 0, 0, 0],
    [1, 0, 0, 0, 0],
    [1, 1, 0, 0, 0],
];

/// Doğrulanmış iki basamaklı ek EAN verisi.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EAN2([u8; 2]);

impl AsRef<[u8]> for EAN2 {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// Doğrulanmış beş basamaklı ek EAN verisi.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EAN5([u8; 5]);

impl AsRef<[u8]> for EAN5 {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// Ek EAN barkod türü.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EANSUPP {
    /// Ek EAN-2 barkod türü.
    EAN2(EAN2),
    /// Ek EAN-5 barkod türü.
    EAN5(EAN5),
}

impl EANSUPP {
    /// Yeni bir barkod oluşturur.
    /// Girdinin çözümlenme sonucunu `Result<EANSUPP, Error>` olarak döndürür.
    /// `data` uzunluğuna göre `EAN2` veya `EAN5` varyantını döndürür.
    pub fn new<T: AsRef<str>>(data: T) -> Result<EANSUPP> {
        EANSUPP::parse(data.as_ref()).and_then(|d| {
            let digits = helpers::parse_digits(d)?;

            match digits.as_slice() {
                [first, second] => Ok(EANSUPP::EAN2(EAN2([*first, *second]))),
                [first, second, third, fourth, fifth] => Ok(EANSUPP::EAN5(EAN5([
                    *first, *second, *third, *fourth, *fifth,
                ]))),
                _ => Err(Error::length(2, Some(5), digits.len())),
            }
        })
    }

    fn raw_data(&self) -> &[u8] {
        match *self {
            EANSUPP::EAN2(ref data) => data.as_ref(),
            EANSUPP::EAN5(ref data) => data.as_ref(),
        }
    }

    fn char_encoding(&self, side: usize, d: u8) -> [u8; 7] {
        let encoding = ENCODINGS
            .get(side)
            .and_then(|encodings| encodings.get(usize::from(d)))
            .copied();
        helpers::invariant_or(
            encoding,
            [0; 7],
            "Ek EAN kodlama dizini oluşturucuda doğrulanmış olmalıdır",
        )
    }

    /// Değiştirilmiş modülo-10 ağırlıklandırma algoritmasıyla sağlama basamağını hesaplar.
    /// Bu işlem yalnızca EAN-5 barkodları için anlamlıdır.
    fn checksum_digit(&self) -> u8 {
        let mut odds = 0u16;
        let mut evens = 0u16;
        let data = self.raw_data();

        for (i, d) in data.iter().enumerate() {
            match i % 2 {
                1 => evens = (evens + u16::from(*d)) % 10,
                _ => odds = (odds + u16::from(*d)) % 10,
            }
        }

        let checksum = ((odds * 3) + (evens * 9)) % 10;

        helpers::invariant_or(
            u8::try_from(checksum).ok(),
            0,
            "Ek EAN sağlama basamağı u8 aralığında olmalıdır",
        )
    }

    fn parity(&self) -> [usize; 5] {
        match *self {
            EANSUPP::EAN2(data) => {
                let [first, second] = data.0;
                let modulo = ((usize::from(first) * 10) + usize::from(second)) % 4;
                helpers::invariant_or(
                    EAN2_PARITY.get(modulo).copied(),
                    [0; 5],
                    "EAN-2 eşlik düzeni tablo sınırları içinde olmalıdır",
                )
            }
            EANSUPP::EAN5(_) => {
                let check = usize::from(self.checksum_digit());
                helpers::invariant_or(
                    EAN5_PARITY.get(check).copied(),
                    [0; 5],
                    "EAN-5 eşlik düzeni tablo sınırları içinde olmalıdır",
                )
            }
        }
    }

    fn payload(&self) -> Vec<u8> {
        let mut p = vec![];
        let parity = self.parity();
        let slices: Vec<[u8; 7]> = self
            .raw_data()
            .iter()
            .zip(parity.iter())
            .map(|(d, s)| self.char_encoding(*s, *d))
            .collect();

        for (i, d) in slices.iter().enumerate() {
            if i > 0 {
                p.push(0);
                p.push(1);
            }

            p.extend(d.iter().cloned());
        }

        p
    }

    /// Barkodu kodlar.
    /// İkili basamakları bir `Vec<u8>` içinde döndürür.
    pub fn encode(&self) -> Vec<u8> {
        let payload = self.payload();
        helpers::join_slices(&[&LEFT_GUARD, payload.as_slice()])
    }
}

impl Parse for EANSUPP {
    /// Bu barkod türünün kabul ettiği geçerli veri uzunluğu aralığını döndürür.
    fn valid_len() -> RangeInclusive<usize> {
        2..=5
    }

    /// Bu barkod türünde kullanılabilen geçerli karakter kümesini döndürür.
    fn valid_chars() -> Vec<char> {
        helpers::decimal_chars()
    }
}

#[cfg(test)]
mod tests {
    use crate::error::{Error, Result};
    use crate::sym::ean_supp::*;
    #[cfg(not(feature = "std"))]
    use alloc::string::String;
    use core::char;

    fn collapse_vec(v: Vec<u8>) -> String {
        v.iter()
            .filter_map(|digit| char::from_digit(u32::from(*digit), 10))
            .collect()
    }

    #[test]
    fn new_ean2() -> Result<()> {
        let ean2 = EANSUPP::new("12");

        assert!(ean2.is_ok());
        Ok(())
    }

    #[test]
    fn new_ean5() -> Result<()> {
        let ean5 = EANSUPP::new("12345");

        assert!(ean5.is_ok());
        Ok(())
    }

    #[test]
    fn invalid_data_ean2() -> Result<()> {
        let ean2 = EANSUPP::new("AT");

        assert!(matches!(ean2, Err(Error::Character { .. })));
        Ok(())
    }

    #[test]
    fn invalid_len_ean2() -> Result<()> {
        let ean2 = EANSUPP::new("123");

        assert!(matches!(ean2, Err(Error::Length { .. })));
        Ok(())
    }

    #[test]
    fn ean2_encode() -> Result<()> {
        let ean21 = EANSUPP::new("34")?;

        assert_eq!(collapse_vec(ean21.encode()), "10110100001010100011");
        Ok(())
    }

    #[test]
    fn ean5_encode() -> Result<()> {
        let ean51 = EANSUPP::new("51234")?;

        assert_eq!(
            collapse_vec(ean51.encode()),
            "10110110001010011001010011011010111101010011101"
        );
        Ok(())
    }
}
