//! 2-of-5 barkodlarını kodlayan bileşen.
//!
//! 2-of-5 barkodları havayollarında ve bazı endüstriyel ortamlarda sıkça kullanılır.
//!
//! Perakendede de ürün gruplarının dış kolilerinde (kola kolileri gibi) kullanılabilir.
//!
//! Çoğu durumda standart seçenek yerine aralıklı barkodu kullanmak istersiniz.

use crate::error::Result;
use crate::sym::Parse;
use crate::sym::helpers;
use core::ops::RangeInclusive;
use helpers::{Vec, vec};

#[rustfmt::skip]
const WIDTHS: [&str; 10] = [
    "NNWWN", "WNNNW", "NWNNW",
    "WWNNN", "NNWNW", "WNWNN",
    "NWWNN", "NNNWW", "WNNWN",
    "NWNWN",
];

// Koruma desenlerindeki geniş öğeler, veri karakterleriyle aynı 3:1 geniş/dar oranını kullanır;
// spesifikasyon aynı sembol içinde tek bir oran ister.
const ITF_START: [u8; 4] = [1, 0, 1, 0];
const ITF_STOP: [u8; 5] = [1, 1, 1, 0, 1];
const STF_START: [u8; 10] = [1, 1, 1, 0, 1, 1, 1, 0, 1, 0];
const STF_STOP: [u8; 10] = [1, 1, 1, 0, 1, 0, 1, 1, 1, 0];

/// Doğrulanmış standart 2-of-5 verisi.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Standard(Vec<u8>);

impl AsRef<[u8]> for Standard {
    fn as_ref(&self) -> &[u8] {
        self.0.as_slice()
    }
}

/// Doğrulanmış aralıklı 2-of-5 verisi.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Interleaved(Vec<u8>);

impl AsRef<[u8]> for Interleaved {
    fn as_ref(&self) -> &[u8] {
        self.0.as_slice()
    }
}

/// 2-of-5 barkod türü.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TF {
    /// Standart 2-of-5 barkod türü.
    Standard(Standard),
    /// Aralıklı 2-of-5 barkod türü.
    Interleaved(Interleaved),
}

impl TF {
    /// Yeni bir ITF barkodu oluşturur.
    /// Verilen verinin uzunluğu tekse bir sağlama değeri hesaplanarak kodlanacak veriye eklenir.
    ///
    /// Girdinin çözümlenme sonucunu `Result<TF::Interleaved, Error>` olarak döndürür.
    pub fn interleaved<T: AsRef<str>>(data: T) -> Result<TF> {
        let data = TF::parse(data.as_ref())?;
        let mut digits = helpers::parse_digits(data)?;
        let checksum_required = digits.len() % 2 == 1;

        if checksum_required {
            let check_digit = helpers::modulo_10_checksum(digits.as_slice(), false);
            digits.push(check_digit);
        }

        Ok(TF::Interleaved(Interleaved(digits)))
    }

    /// Yeni bir STF barkodu oluşturur.
    ///
    /// Girdinin çözümlenme sonucunu `Result<TF::Standard, Error>` olarak döndürür.
    pub fn standard<T: AsRef<str>>(data: T) -> Result<TF> {
        let data = TF::parse(data.as_ref())?;
        let digits = helpers::parse_digits(data)?;
        Ok(TF::Standard(Standard(digits)))
    }

    fn raw_data(&self) -> &[u8] {
        match *self {
            TF::Standard(ref data) => data.as_ref(),
            TF::Interleaved(ref data) => data.as_ref(),
        }
    }

    fn interleave(&self, bars: u8, spaces: u8) -> Vec<u8> {
        let bwidths = self.char_widths(bars).chars();
        let swidths = self.char_widths(spaces).chars();
        let mut encoding: Vec<u8> = vec![];

        for (b, s) in bwidths.zip(swidths) {
            for &(c, i) in &[(b, 1), (s, 0)] {
                match c {
                    'W' => encoding.extend([i; 3].iter().cloned()),
                    _ => encoding.push(i),
                }
            }
        }

        encoding
    }

    fn char_encoding(&self, d: u8) -> Vec<u8> {
        let bars: Vec<Vec<u8>> = self
            .char_widths(d)
            .chars()
            .map(|c| match c {
                'W' => vec![1, 1, 1, 0],
                _ => vec![1, 0],
            })
            .collect();

        helpers::join_iters(bars.iter())
    }

    fn char_widths(&self, d: u8) -> &'static str {
        helpers::invariant_or(
            WIDTHS.get(usize::from(d)).copied(),
            "",
            "2-of-5 basamağı oluşturucuda doğrulanmış olmalıdır",
        )
    }

    fn stf_payload(&self) -> Vec<u8> {
        let mut encodings = vec![];

        for d in self.raw_data() {
            encodings.extend(self.char_encoding(*d).iter().cloned());
        }

        encodings
    }

    fn itf_payload(&self) -> Vec<u8> {
        assert!(
            self.raw_data().len().is_multiple_of(2),
            "Aralıklı 2-of-5 iç verisi çift sayıda basamak taşımalıdır"
        );
        let mut weaves = Vec::new();

        for chunk in self.raw_data().chunks(2) {
            let mut digits = chunk.iter().copied();
            let bars = helpers::invariant_or(
                digits.next(),
                0,
                "Aralıklı 2-of-5 çubuk basamağı oluşturucuda doğrulanmış olmalıdır",
            );
            let spaces = helpers::invariant_or(
                digits.next(),
                0,
                "Aralıklı 2-of-5 boşluk basamağı oluşturucuda doğrulanmış olmalıdır",
            );
            weaves.push(self.interleave(bars, spaces));
        }

        helpers::join_iters(weaves.iter())
    }

    /// Barkodu kodlar.
    /// İkili basamakları bir `Vec<u8>` içinde döndürür.
    pub fn encode(&self) -> Vec<u8> {
        match *self {
            TF::Standard(_) => {
                let payload = self.stf_payload();
                helpers::join_slices(&[&STF_START, payload.as_slice(), &STF_STOP])
            }
            TF::Interleaved(_) => {
                let payload = self.itf_payload();
                helpers::join_slices(&[&ITF_START, payload.as_slice(), &ITF_STOP])
            }
        }
    }
}

impl Parse for TF {
    /// Bu barkod türünün kabul ettiği geçerli veri uzunluğu aralığını döndürür.
    /// 2-of-5 barkodları değişken uzunluktadır.
    fn valid_len() -> RangeInclusive<usize> {
        1..=256
    }

    /// Bu barkod türünde kullanılabilen geçerli karakter kümesini döndürür.
    fn valid_chars() -> Vec<char> {
        helpers::decimal_chars()
    }
}

#[cfg(test)]
mod tests {
    use crate::error::{Error, Result};
    use crate::sym::tf::*;
    #[cfg(not(feature = "std"))]
    pub(crate) use alloc::string::{String, ToString};
    use core::char;

    fn collapse_vec(v: Vec<u8>) -> String {
        v.iter()
            .filter_map(|digit| char::from_digit(u32::from(*digit), 10))
            .collect()
    }

    #[test]
    fn new_itf() -> Result<()> {
        let itf = TF::interleaved("12345679");

        assert!(itf.is_ok());
        Ok(())
    }

    #[test]
    fn new_stf() -> Result<()> {
        let stf = TF::standard("12345");

        assert!(stf.is_ok());
        Ok(())
    }

    #[test]
    fn invalid_data_itf() -> Result<()> {
        let itf = TF::interleaved("1234er123412");

        assert!(matches!(itf, Err(Error::Character { .. })));
        Ok(())
    }

    #[test]
    fn invalid_data_stf() -> Result<()> {
        let stf = TF::standard("WORDUP");

        assert!(matches!(stf, Err(Error::Character { .. })));
        Ok(())
    }

    #[test]
    fn itf_raw_data() -> Result<()> {
        let itf = TF::interleaved("12345679")?;

        assert_eq!(itf.raw_data(), &[1, 2, 3, 4, 5, 6, 7, 9]);
        Ok(())
    }

    #[test]
    fn itf_encode() -> Result<()> {
        let itf = TF::interleaved("1234567")?; // Sağlama basamağı: 0

        assert_eq!(
            collapse_vec(itf.encode()),
            "101011101000101011100011101110100010100011101000111000101010101000111000111011101"
                .to_string()
        );
        Ok(())
    }

    #[test]
    fn stf_encode() -> Result<()> {
        let stf = TF::standard("1234567")?;

        assert_eq!(
            collapse_vec(stf.encode()),
            "1110111010111010101011101011101010111011101110101010101011101011101110101110101010111011101010101010111011101110101110"
                .to_string()
        );
        Ok(())
    }
}
