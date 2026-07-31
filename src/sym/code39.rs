//! Code39 barkodlarını kodlayan bileşen.
//!
//! Code39 ayrık ve değişken uzunluklu bir barkoddur. Sıklıkla "3-of-9" adıyla anılır.
//!
//! Code39, ABD Savunma Bakanlığının kullandığı standart barkoddur ve perakende dışı ortamlarda da
//! yaygındır. ASCII alfabesini kodlamayı destekleyen ilk sembolojilerden biridir.

use crate::error::Result;
use crate::sym::{Parse, helpers};
use core::ops::Range;
use helpers::{Vec, vec};

// İzin verilen 43 karakterin her biri için karakter -> ikili değer eşlemeleri.
const CHARS: [(char, [u8; 12]); 43] = [
    ('0', [1, 0, 1, 0, 0, 1, 1, 0, 1, 1, 0, 1]),
    ('1', [1, 1, 0, 1, 0, 0, 1, 0, 1, 0, 1, 1]),
    ('2', [1, 0, 1, 1, 0, 0, 1, 0, 1, 0, 1, 1]),
    ('3', [1, 1, 0, 1, 1, 0, 0, 1, 0, 1, 0, 1]),
    ('4', [1, 0, 1, 0, 0, 1, 1, 0, 1, 0, 1, 1]),
    ('5', [1, 1, 0, 1, 0, 0, 1, 1, 0, 1, 0, 1]),
    ('6', [1, 0, 1, 1, 0, 0, 1, 1, 0, 1, 0, 1]),
    ('7', [1, 0, 1, 0, 0, 1, 0, 1, 1, 0, 1, 1]),
    ('8', [1, 1, 0, 1, 0, 0, 1, 0, 1, 1, 0, 1]),
    ('9', [1, 0, 1, 1, 0, 0, 1, 0, 1, 1, 0, 1]),
    ('A', [1, 1, 0, 1, 0, 1, 0, 0, 1, 0, 1, 1]),
    ('B', [1, 0, 1, 1, 0, 1, 0, 0, 1, 0, 1, 1]),
    ('C', [1, 1, 0, 1, 1, 0, 1, 0, 0, 1, 0, 1]),
    ('D', [1, 0, 1, 0, 1, 1, 0, 0, 1, 0, 1, 1]),
    ('E', [1, 1, 0, 1, 0, 1, 1, 0, 0, 1, 0, 1]),
    ('F', [1, 0, 1, 1, 0, 1, 1, 0, 0, 1, 0, 1]),
    ('G', [1, 0, 1, 0, 1, 0, 0, 1, 1, 0, 1, 1]),
    ('H', [1, 1, 0, 1, 0, 1, 0, 0, 1, 1, 0, 1]),
    ('I', [1, 0, 1, 1, 0, 1, 0, 0, 1, 1, 0, 1]),
    ('J', [1, 0, 1, 0, 1, 1, 0, 0, 1, 1, 0, 1]),
    ('K', [1, 1, 0, 1, 0, 1, 0, 1, 0, 0, 1, 1]),
    ('L', [1, 0, 1, 1, 0, 1, 0, 1, 0, 0, 1, 1]),
    ('M', [1, 1, 0, 1, 1, 0, 1, 0, 1, 0, 0, 1]),
    ('N', [1, 0, 1, 0, 1, 1, 0, 1, 0, 0, 1, 1]),
    ('O', [1, 1, 0, 1, 0, 1, 1, 0, 1, 0, 0, 1]),
    ('P', [1, 0, 1, 1, 0, 1, 1, 0, 1, 0, 0, 1]),
    ('Q', [1, 0, 1, 0, 1, 0, 1, 1, 0, 0, 1, 1]),
    ('R', [1, 1, 0, 1, 0, 1, 0, 1, 1, 0, 0, 1]),
    ('S', [1, 0, 1, 1, 0, 1, 0, 1, 1, 0, 0, 1]),
    ('T', [1, 0, 1, 0, 1, 1, 0, 1, 1, 0, 0, 1]),
    ('U', [1, 1, 0, 0, 1, 0, 1, 0, 1, 0, 1, 1]),
    ('V', [1, 0, 0, 1, 1, 0, 1, 0, 1, 0, 1, 1]),
    ('W', [1, 1, 0, 0, 1, 1, 0, 1, 0, 1, 0, 1]),
    ('X', [1, 0, 0, 1, 0, 1, 1, 0, 1, 0, 1, 1]),
    ('Y', [1, 1, 0, 0, 1, 0, 1, 1, 0, 1, 0, 1]),
    ('Z', [1, 0, 0, 1, 1, 0, 1, 1, 0, 1, 0, 1]),
    ('-', [1, 0, 0, 1, 0, 1, 0, 1, 1, 0, 1, 1]),
    ('.', [1, 1, 0, 0, 1, 0, 1, 0, 1, 1, 0, 1]),
    (' ', [1, 0, 0, 1, 1, 0, 1, 0, 1, 1, 0, 1]),
    ('$', [1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 1]),
    ('/', [1, 0, 0, 1, 0, 0, 1, 0, 1, 0, 0, 1]),
    ('+', [1, 0, 0, 1, 0, 1, 0, 0, 1, 0, 0, 1]),
    ('%', [1, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1]),
];

// Code39 barkodları özel `*` karakteriyle başlayıp bitmelidir.
const GUARD: [u8; 12] = [1, 0, 0, 1, 0, 1, 1, 0, 1, 1, 0, 1];

/// Code39 barkod türü.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Code39 {
    data: Vec<char>,
    /// Sağlama basamağının kodlanıp kodlanmayacağını belirtir.
    pub checksum: bool,
}

impl Code39 {
    fn init(data: &str, checksum: bool) -> Result<Code39> {
        Code39::parse(data).map(|d| Code39 {
            data: d.chars().collect(),
            checksum,
        })
    }

    /// Yeni bir barkod oluşturur.
    /// Girdinin çözümlenme sonucunu `Result<Code39, Error>` olarak döndürür.
    pub fn new<T: AsRef<str>>(data: T) -> Result<Code39> {
        Code39::init(data.as_ref(), false)
    }

    /// Modülo-43 ile hesaplanan sağlama basamağını ekleyerek yeni bir barkod oluşturur.
    /// Girdinin çözümlenme sonucunu `Result<Code39, Error>` olarak döndürür.
    pub fn with_checksum<T: AsRef<str>>(data: T) -> Result<Code39> {
        Code39::init(data.as_ref(), true)
    }

    /// Modülo-43 algoritmasıyla sağlama karakterini hesaplar.
    fn checksum_char(&self) -> char {
        let mut sum = 0usize;
        for character in &self.data {
            let position = CHARS.iter().position(|candidate| candidate.0 == *character);
            let position = helpers::invariant_or(
                position,
                0,
                "Code39 sağlama hesabındaki karakter doğrulanmış olmalıdır",
            );
            sum = (sum + position) % CHARS.len();
        }

        let character = CHARS
            .get(sum % CHARS.len())
            .map(|(character, _)| *character);
        helpers::invariant_or(
            character,
            '0',
            "Code39 sağlama karakteri tablo sınırları içinde olmalıdır",
        )
    }

    fn checksum_encoding(&self) -> [u8; 12] {
        self.char_encoding(self.checksum_char())
    }

    fn char_encoding(&self, c: char) -> [u8; 12] {
        let encoding = CHARS
            .iter()
            .find(|candidate| candidate.0 == c)
            .map(|(_, encoding)| *encoding);
        helpers::invariant_or(
            encoding,
            [0; 12],
            "Code39 iç verisindeki karakter oluşturucuda doğrulanmış olmalıdır",
        )
    }

    // Code39 barkodlarında kodlanmış karakterler tek bir "dar" çubukla ayrılır.
    fn push_encoding(&self, into: &mut Vec<u8>, from: [u8; 12]) {
        into.extend(from.iter().cloned());
        into.push(0);
    }

    fn payload(&self) -> Vec<u8> {
        let mut enc = vec![0];

        for c in &self.data {
            self.push_encoding(&mut enc, self.char_encoding(*c));
        }

        if self.checksum {
            self.push_encoding(&mut enc, self.checksum_encoding());
        }

        enc
    }

    /// Barkodu kodlar.
    /// İkili basamakları bir `Vec<u8>` içinde döndürür.
    pub fn encode(&self) -> Vec<u8> {
        let payload = self.payload();

        helpers::join_slices(&[&GUARD, payload.as_slice(), &GUARD])
    }
}

impl Parse for Code39 {
    fn valid_len() -> Range<u32> {
        1..256
    }

    fn valid_chars() -> Vec<char> {
        let (chars, _): (Vec<_>, Vec<_>) = CHARS.iter().cloned().unzip();
        chars
    }
}

#[cfg(test)]
mod tests {
    use crate::error::{Error, Result};
    use crate::sym::code39::*;
    #[cfg(not(feature = "std"))]
    use alloc::string::String;
    use core::char;

    fn collapse_vec(v: Vec<u8>) -> String {
        v.iter()
            .filter_map(|digit| char::from_digit(u32::from(*digit), 10))
            .collect()
    }

    #[test]
    fn new_code39() -> Result<()> {
        let code39 = Code39::new("12345");

        assert!(code39.is_ok());
        Ok(())
    }

    #[test]
    fn invalid_data_code39() -> Result<()> {
        let code39 = Code39::new("1212s");

        assert!(matches!(code39, Err(Error::Character { .. })));
        Ok(())
    }

    #[test]
    fn invalid_len_code39() -> Result<()> {
        let code39 = Code39::new("");

        assert!(matches!(code39, Err(Error::Length { .. })));
        Ok(())
    }

    #[test]
    fn code39_encode() -> Result<()> {
        let code391 = Code39::new("1234")?;
        let code392 = Code39::new("983RD512")?;
        let code393 = Code39::new("TEST8052")?;

        assert_eq!(
            collapse_vec(code391.encode()),
            "10010110110101101001010110101100101011011011001010101010011010110100101101101"
        );
        assert_eq!(
            collapse_vec(code392.encode()),
            "100101101101010110010110101101001011010110110010101011010101100101010110010110110100110101011010010101101011001010110100101101101"
        );
        assert_eq!(
            collapse_vec(code393.encode()),
            "100101101101010101101100101101011001010101101011001010101101100101101001011010101001101101011010011010101011001010110100101101101"
        );
        Ok(())
    }

    #[test]
    fn code39_encode_with_checksum() -> Result<()> {
        let code391 = Code39::with_checksum("1234")?;
        let code392 = Code39::with_checksum("983RD512")?;

        assert_eq!(
            collapse_vec(code391.encode()),
            "100101101101011010010101101011001010110110110010101010100110101101101010010110100101101101"
        );
        assert_eq!(
            collapse_vec(code392.encode()),
            "1001011011010101100101101011010010110101101100101010110101011001010101100101101101001101010110100101011010110010101101011011010010100101101101"
        );
        Ok(())
    }
}
