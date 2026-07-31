//! Barkod kodlama ve çıktı üretme hata türleri.

use core::fmt;
#[cfg(feature = "std")]
use std::error::Error as StdError;

/// Barkod kodlama ve çıktı üretme sırasında oluşabilecek hatalar.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Error {
    /// Girdi, seçilen barkod türünün desteklemediği bir karakter içeriyor.
    Character,
    /// Girdi uzunluğu seçilen barkod türü için geçersiz.
    Length,
    /// Barkodun hedef biçime dönüştürülmesi başarısız oldu.
    Generate,
    /// Girdideki sağlama basamağı hesaplanan değerle eşleşmiyor.
    Checksum,
    /// Üretece verilen barkod gösterimi `0` ve `1` dışında bir değer içeriyor.
    InvalidEncoding,
    /// İstenen çıktı boyutları desteklenen sayısal aralığı aşıyor.
    Dimension,
}

/// `Result<T, barcoders::error::Error>` için kısa tür adı.
pub type Result<T> = ::core::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Character => write!(f, "Barkod verisi desteklenmeyen bir karakter içeriyor"),
            Error::Length => write!(f, "Barkod verisinin uzunluğu geçersiz"),
            Error::Generate => write!(f, "Barkod çıktısı hedef biçimde oluşturulamadı"),
            Error::Checksum => write!(f, "Barkod sağlama basamağı geçersiz"),
            Error::InvalidEncoding => {
                write!(f, "Barkod gösterimi yalnızca 0 ve 1 değerlerini içerebilir")
            }
            Error::Dimension => write!(f, "Barkod boyutları desteklenen aralığı aşıyor"),
        }
    }
}

#[cfg(feature = "std")]
impl StdError for Error {}
