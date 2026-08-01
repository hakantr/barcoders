//! Code11 (USD-8) barkodlarını kodlayan bileşen.
//!
//! Code11, tüm ondalık basamakları ve tire karakterini kodlayabilir. Başlıca telekomünikasyon
//! sektöründe kullanılır.
//!
//! Code11 ayrık bir sembolojidir. Bu kodlayıcı her zaman C sağlama değerini ekler. On karakterden
//! uzun barkodlara ikinci bir sağlama basamağı (K) da eklenir.

use crate::error::Result;
use crate::sym::{Parse, helpers};
use core::ops::RangeInclusive;
use helpers::{Vec, vec};

// İzin verilen her karakter için karakter -> ikili değer eşlemeleri.
// Özel "tam ASCII" karakterleri (, ), [ ve ] ile temsil edilir.
const CHARS: [(char, &[u8]); 11] = [
    ('0', &[1, 0, 1, 0, 1, 1]),
    ('1', &[1, 1, 0, 1, 0, 1, 1]),
    ('2', &[1, 0, 0, 1, 0, 1, 1]),
    ('3', &[1, 1, 0, 0, 1, 0, 1]),
    ('4', &[1, 0, 1, 1, 0, 1, 1]),
    ('5', &[1, 1, 0, 1, 1, 0, 1]),
    ('6', &[1, 0, 0, 1, 1, 0, 1]),
    ('7', &[1, 0, 1, 0, 0, 1, 1]),
    ('8', &[1, 1, 0, 1, 0, 0, 1]),
    ('9', &[1, 1, 0, 1, 0, 1]),
    ('-', &[1, 0, 1, 1, 0, 1]),
];

// Code11 barkodları özel bir karakterle başlayıp bitmelidir.
const GUARD: [u8; 7] = [1, 0, 1, 1, 0, 0, 1];
const SEPARATOR: [u8; 1] = [0];

/// Code11 barkod türü.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Code11(Vec<char>);

/// USD-8 barkod türü.
pub type USD8 = Code11;

impl Code11 {
    /// Yeni bir barkod oluşturur.
    /// Girdinin çözümlenme sonucunu `Result<Code11, Error>` olarak döndürür.
    pub fn new<T: AsRef<str>>(data: T) -> Result<Code11> {
        Code11::parse(data.as_ref()).map(|d| Code11(d.chars().collect()))
    }

    fn char_encoding(&self, c: char) -> &'static [u8] {
        let encoding = CHARS
            .iter()
            .find(|character| character.0 == c)
            .map(|(_, encoding)| *encoding);
        helpers::invariant_or(
            encoding,
            &[],
            "Code11 iç verisindeki karakter oluşturucuda doğrulanmış olmalıdır",
        )
    }

    /// Ağırlıklı modülo-11 algoritmasıyla bir sağlama karakteri hesaplar.
    fn checksum_char(&self, data: &[char], weight_threshold: usize) -> char {
        assert!(weight_threshold > 0, "Code11 sağlama ağırlığı sıfır olamaz");

        let mut index = 0usize;
        for (offset, character) in data.iter().rev().enumerate() {
            let position = CHARS.iter().position(|candidate| candidate.0 == *character);
            let position = helpers::invariant_or(
                position,
                0,
                "Code11 sağlama hesabındaki karakter doğrulanmış olmalıdır",
            );
            let weight = (offset % weight_threshold) + 1;
            index = (index + (weight * position)) % CHARS.len();
        }

        // Bazı kaynaklar C sağlamasının modülo-11, K sağlamasının ise modülo-9 kullanmasını
        // önerir. Ancak üreteçlerin çoğu her ikisi için de modülo-11 kullanır. Bu algoritma
        // şimdilik iki sağlama için de 11 kullanır; gerekirse daha sonra kolayca değiştirilebilir.
        let character = CHARS
            .get(index % CHARS.len())
            .map(|(character, _)| *character);
        helpers::invariant_or(
            character,
            '0',
            "Code11 sağlama karakteri tablo sınırları içinde olmalıdır",
        )
    }

    /// Ağırlıklı modülo-11 algoritmasıyla C sağlama karakterini hesaplar.
    fn c_checksum_char(&self) -> char {
        self.checksum_char(&self.0, 10)
    }

    /// Ağırlıklı modülo-11 algoritmasıyla K sağlama karakterini hesaplar.
    fn k_checksum_char(&self, c_checksum: char) -> char {
        let mut data: Vec<char> = self.0.clone();
        data.push(c_checksum);

        self.checksum_char(&data, 9)
    }

    fn push_encoding(&self, into: &mut Vec<u8>, from: &[u8]) {
        into.extend(from.iter().cloned());
        into.extend(&SEPARATOR);
    }

    fn payload(&self) -> Vec<u8> {
        let mut enc = vec![];
        let c_checksum = self.c_checksum_char();

        for &c in &self.0 {
            self.push_encoding(&mut enc, self.char_encoding(c));
        }

        self.push_encoding(&mut enc, self.char_encoding(c_checksum));

        // K sağlaması yalnızca on karakterden uzun barkodlara eklenir.
        if self.0.len() > 10 {
            let k_checksum = self.k_checksum_char(c_checksum);

            self.push_encoding(&mut enc, self.char_encoding(k_checksum));
        }

        enc
    }

    /// Barkodu kodlar.
    /// Kodlanmış ikili basamakları bir `Vec<u8>` içinde döndürür.
    pub fn encode(&self) -> Vec<u8> {
        let payload = self.payload();

        helpers::join_slices(&[&GUARD, &SEPARATOR, payload.as_slice(), &GUARD])
    }
}

impl Parse for Code11 {
    /// Bu barkod türünün kabul ettiği geçerli veri uzunluğu aralığını döndürür.
    /// Code11 barkodları değişken uzunluktadır.
    fn valid_len() -> RangeInclusive<usize> {
        1..=256
    }

    /// Bu barkod türünde kullanılabilen geçerli karakter kümesini döndürür.
    fn valid_chars() -> Vec<char> {
        let (chars, _): (Vec<_>, Vec<_>) = CHARS.iter().cloned().unzip();
        chars
    }
}

#[cfg(test)]
mod tests {
    use crate::error::{Error, Result};
    use crate::sym::code11::*;
    #[cfg(not(feature = "std"))]
    use alloc::string::String;
    use core::char;

    fn collapse_vec(v: Vec<u8>) -> String {
        v.iter()
            .filter_map(|digit| char::from_digit(u32::from(*digit), 10))
            .collect()
    }

    #[test]
    fn invalid_length_code11() -> Result<()> {
        let code11 = Code11::new("");

        assert!(matches!(code11, Err(Error::Length { .. })));
        Ok(())
    }

    #[test]
    fn invalid_data_code11() -> Result<()> {
        let code11 = Code11::new("NOTDIGITS");

        assert!(matches!(code11, Err(Error::Character { .. })));
        Ok(())
    }

    #[test]
    fn code11_encode_less_than_10_chars() -> Result<()> {
        let code111 = Code11::new("123-45")?;
        let code112 = Code11::new("666")?;
        let code113 = Code11::new("12-9")?;

        assert_eq!(
            collapse_vec(code111.encode()),
            "1011001011010110100101101100101010110101011011011011010110110101011001"
        );
        assert_eq!(
            collapse_vec(code112.encode()),
            "10110010100110101001101010011010110010101011001"
        );
        assert_eq!(
            collapse_vec(code113.encode()),
            "10110010110101101001011010110101101010100110101011001"
        );
        Ok(())
    }

    #[test]
    fn code11_encode_more_than_10_chars() -> Result<()> {
        let code111 = Code11::new("1234-5678-4321")?;

        assert_eq!(
            collapse_vec(code111.encode()),
            "101100101101011010010110110010101011011010110101101101010011010101001101101001010110101011011011001010100101101101011011011010100110101011001"
        );
        Ok(())
    }
}
