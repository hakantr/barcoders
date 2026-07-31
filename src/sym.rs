//! Desteklenen barkod sembolojileri.
//!
//! Sembolojiler mantıksal modüllere ayrılmıştır; bu nedenle gerekli modülleri `use` ile kapsamınıza
//! almalısınız.
//!
//! Örneğin:
//!
//! ```rust
//! use barcoders::sym::ean13::*;
//!
//! # fn main() -> barcoders::error::Result<()> {
//! let barcode = EAN13::new("750103131130")?;
//! let encoded = barcode.encode();
//! # Ok(())
//! # }
//! ```
//! Her kodlayıcı, kodlanacak metni kabul eder. Geçerli veri barkod türüne göre değiştiğinden
//! kurucular `Result<T, Error>` döndürür.

pub mod codabar;
pub mod code11;
pub mod code128;
pub mod code39;
pub mod code93;
pub mod ean13;
pub mod ean8;
pub mod ean_supp;
mod helpers;
pub mod tf;
pub mod upca;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::error::Error;
use core::iter::Iterator;
use core::ops::Range;

trait Parse {
    fn valid_chars() -> Vec<char>;
    fn valid_len() -> Range<u32>;

    fn parse(data: &str) -> Result<&str, Error> {
        let valid_chars = Self::valid_chars();
        let valid_len = Self::valid_len();
        let data_len = u32::try_from(data.len()).map_err(|_| Error::Length)?;

        if data_len < valid_len.start || data_len > valid_len.end {
            return Err(Error::Length);
        }

        let bad_char = data.chars().find(|c| !valid_chars.contains(c));

        match bad_char {
            Some(_) => Err(Error::Character),
            None => Ok(data),
        }
    }
}
