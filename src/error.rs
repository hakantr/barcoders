//! Barkod kodlama ve çıktı üretme hata türleri.

use core::fmt;
#[cfg(feature = "std")]
use std::error::Error as StdError;

/// Barkod kodlama ve çıktı üretme sırasında oluşabilecek hatalar.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// Girdi, seçilen barkod türünün desteklemediği bir karakter içeriyor.
    Character {
        /// Desteklenmeyen karakter biliniyorsa karakterin kendisi.
        character: Option<char>,
        /// Karakter konumu biliniyorsa sıfır tabanlı konum.
        index: Option<usize>,
    },
    /// Girdi uzunluğu seçilen barkod türü için geçersiz.
    Length {
        /// Kabul edilen en kısa uzunluk.
        min: usize,
        /// Üst sınır varsa kabul edilen en uzun uzunluk.
        max: Option<usize>,
        /// Girdide bulunan uzunluk.
        found: usize,
    },
    /// Girdi uzunluğu, kabul edilen ayrık uzunlukların hiçbiriyle eşleşmiyor.
    LengthSet {
        /// Kabul edilen uzunluklar.
        expected: &'static [usize],
        /// Girdide bulunan uzunluk.
        found: usize,
    },
    /// Barkodun hedef biçime dönüştürülmesi başarısız oldu.
    Generate {
        /// Üretilmeye çalışılan hedef.
        target: &'static str,
        /// Üretimin neden tamamlanamadığı.
        reason: &'static str,
    },
    /// Girdideki sağlama basamağı hesaplanan değerle eşleşmiyor.
    Checksum {
        /// Hesaplanan sağlama basamağı.
        expected: u8,
        /// Girdide bulunan sağlama basamağı.
        found: u8,
    },
    /// Üretece verilen barkod gösterimi `0` ve `1` dışında bir değer içeriyor.
    InvalidEncoding {
        /// Geçersiz modülün sıfır tabanlı konumu.
        index: usize,
        /// Konumda bulunan geçersiz değer.
        value: u8,
    },
    /// İstenen çıktı boyutları geçersiz veya desteklenen sayısal aralığın dışında.
    Dimension {
        /// Boyutun neden kabul edilmediği.
        reason: &'static str,
    },
    /// İstenen üretim, güvenli kaynak sınırını aşıyor.
    ResourceLimit {
        /// Sınır uygulanan kaynak türü.
        resource: &'static str,
        /// İstenen kaynak miktarı.
        requested: u64,
        /// İzin verilen en yüksek kaynak miktarı.
        maximum: u64,
    },
}

impl Error {
    pub(crate) const fn character(character: Option<char>, index: Option<usize>) -> Self {
        Self::Character { character, index }
    }

    pub(crate) const fn length(min: usize, max: Option<usize>, found: usize) -> Self {
        Self::Length { min, max, found }
    }

    pub(crate) const fn length_set(expected: &'static [usize], found: usize) -> Self {
        Self::LengthSet { expected, found }
    }

    #[cfg(any(feature = "image", all(test, feature = "svg")))]
    pub(crate) const fn generate(target: &'static str, reason: &'static str) -> Self {
        Self::Generate { target, reason }
    }

    // Boyut hataları yalnız çizim yapan üreteçlerden doğar; çekirdek kodlama derlemesinde
    // kullanılmaz.
    #[cfg(any(
        feature = "ascii",
        feature = "gpui",
        feature = "svg",
        all(feature = "image", feature = "std")
    ))]
    pub(crate) const fn dimension(reason: &'static str) -> Self {
        Self::Dimension { reason }
    }
}

/// `Result<T, barcoders::error::Error>` için kısa tür adı.
pub type Result<T> = ::core::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Character {
                character: Some(character),
                index: Some(index),
            } => write!(
                f,
                "Barkod verisindeki '{character}' karakteri {index}. konumda desteklenmiyor"
            ),
            Error::Character {
                character: Some(character),
                index: None,
            } => write!(
                f,
                "Barkod verisindeki '{character}' karakteri desteklenmiyor"
            ),
            Error::Character { .. } => {
                write!(f, "Barkod verisi desteklenmeyen bir karakter içeriyor")
            }
            Error::Length {
                min,
                max: Some(max),
                found,
            } if min == max => write!(
                f,
                "Barkod verisi {min} karakter olmalı; {found} karakter bulundu"
            ),
            Error::Length {
                min,
                max: Some(max),
                found,
            } => write!(
                f,
                "Barkod verisi {min} ile {max} karakter arasında olmalı; {found} karakter bulundu"
            ),
            Error::Length {
                min,
                max: None,
                found,
            } => write!(
                f,
                "Barkod verisi en az {min} karakter olmalı; {found} karakter bulundu"
            ),
            Error::LengthSet { expected, found } => {
                write!(f, "Barkod verisi ")?;
                for (index, length) in expected.iter().enumerate() {
                    if index > 0 {
                        let separator = if index + 1 == expected.len() {
                            " veya "
                        } else {
                            ", "
                        };
                        f.write_str(separator)?;
                    }
                    write!(f, "{length}")?;
                }
                write!(f, " karakter olmalı; {found} karakter bulundu")
            }
            Error::Generate { target, reason } => {
                write!(
                    f,
                    "Barkod çıktısı {target} hedefinde oluşturulamadı: {reason}"
                )
            }
            Error::Checksum { expected, found } => write!(
                f,
                "Barkod sağlama basamağı geçersiz: {expected} beklenirken {found} bulundu"
            ),
            Error::InvalidEncoding { index, value } => write!(
                f,
                "Barkod gösteriminin {index}. konumunda 0 veya 1 yerine {value} bulundu"
            ),
            Error::Dimension { reason } => write!(f, "Barkod boyutları geçersiz: {reason}"),
            Error::ResourceLimit {
                resource,
                requested,
                maximum,
            } => write!(
                f,
                "Barkod üretimi {resource} sınırını aşıyor: {requested} istendi, en fazla {maximum} kullanılabilir"
            ),
        }
    }
}

#[cfg(feature = "std")]
impl StdError for Error {}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(not(feature = "std"))]
    use alloc::string::ToString;

    #[test]
    fn character_error_contains_value_and_position() {
        let error = Error::character(Some('ğ'), Some(3));

        assert_eq!(
            error.to_string(),
            "Barkod verisindeki 'ğ' karakteri 3. konumda desteklenmiyor"
        );
    }

    #[test]
    fn length_error_contains_expected_range() {
        let error = Error::length(7, Some(8), 4);

        assert_eq!(
            error.to_string(),
            "Barkod verisi 7 ile 8 karakter arasında olmalı; 4 karakter bulundu"
        );
    }

    #[test]
    fn length_set_error_lists_allowed_lengths() {
        let error = Error::length_set(&[2, 5], 3);

        assert_eq!(
            error.to_string(),
            "Barkod verisi 2 veya 5 karakter olmalı; 3 karakter bulundu"
        );
    }
}
