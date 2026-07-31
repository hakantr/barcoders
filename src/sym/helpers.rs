#[cfg(not(feature = "std"))]
pub(crate) use alloc::vec;
#[cfg(not(feature = "std"))]
pub(crate) use alloc::vec::Vec;
#[cfg(feature = "std")]
pub(crate) use std::vec;
#[cfg(feature = "std")]
pub(crate) use std::vec::Vec;

use crate::error::{Error, Result};

/// Joins and flattens the given slice of &[u8] slices into a Vec<u8>.
/// TODO: Work out how to use join_iters with slices and then remove this function.
pub fn join_slices(slices: &[&[u8]]) -> Vec<u8> {
    slices.iter().flat_map(|b| b.iter()).cloned().collect()
}

/// Joins and flattens the given iterator of iterables into a Vec<u8>.
pub fn join_iters<'a, T: Iterator>(iters: T) -> Vec<u8>
where
    T::Item: IntoIterator<Item = &'a u8>,
{
    iters.flat_map(|b| b.into_iter()).cloned().collect()
}

/// Ondalık bir metni doğrulayarak barkod basamaklarına dönüştürür.
pub fn parse_digits(data: &str) -> Result<Vec<u8>> {
    data.chars()
        .map(|character| {
            character
                .to_digit(10)
                .and_then(|digit| u8::try_from(digit).ok())
                .ok_or(Error::Character)
        })
        .collect()
}

/// Ondalık barkod türlerinin kabul ettiği karakterleri döndürür.
pub fn decimal_chars() -> Vec<char> {
    ('0'..='9').collect()
}

/// Doğrulanmış iç verinin taşıması gereken bir değeri güvenli biçimde çıkarır.
pub fn invariant_or<T>(value: Option<T>, fallback: T, message: &'static str) -> T {
    assert!(value.is_some(), "{message}");
    value.unwrap_or(fallback)
}

/// Calculates the checksum digit using a modulo-10 weighting algorithm.
pub fn modulo_10_checksum(data: &[u8], even_start: bool) -> u8 {
    let mut odds = 0u16;
    let mut evens = 0u16;

    for (i, d) in data.iter().enumerate() {
        match i % 2 {
            1 => odds = (odds + u16::from(*d)) % 10,
            _ => evens = (evens + u16::from(*d)) % 10,
        }
    }

    // EAN-13 (and some others?) barcodes use EVEN-first weighting to maintain
    // backwards compatibility.
    if even_start {
        odds = (odds * 3) % 10;
    } else {
        evens = (evens * 3) % 10;
    }

    let checksum = match 10 - ((odds + evens) % 10) {
        10 => 0,
        n => n,
    };

    let checksum = u8::try_from(checksum);
    assert!(
        checksum.is_ok(),
        "Modulo-10 sağlama basamağı u8 aralığında olmalıdır"
    );
    checksum.unwrap_or(0)
}
