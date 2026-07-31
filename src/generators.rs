//! Desteklenen barkod üretme yöntemleri.
//!
//! Her üretim seçeneği isteğe bağlı derlenen bir özelliktir; derleme sırasında kullanmak
//! istediklerinizi açıkça etkinleştirmeniz gerekir.
//!
//! Örneğin:
//!
//! ```toml
//! [dependencies]
//! barcoders = {version = "*", features = ["image"]}
//! ```
//!
//! Özellikler:
//! - `ascii`: ASCII çizimi olarak barkod üretir.
//! - `json`: JSON barkod gösterimi üretir.
//! - `image`: Görüntü tabanlı barkod üretir.
//! - `svg`: SVG barkod gösterimi üretir.

#[cfg(any(
    feature = "ascii",
    feature = "json",
    feature = "svg",
    all(feature = "image", feature = "std")
))]
use crate::error::{Error, Result};

#[cfg(any(
    feature = "ascii",
    feature = "json",
    feature = "svg",
    all(feature = "image", feature = "std")
))]
pub(crate) fn validate_barcode(barcode: &[u8]) -> Result<()> {
    if barcode.iter().all(|bit| matches!(bit, 0 | 1)) {
        Ok(())
    } else {
        Err(Error::InvalidEncoding)
    }
}

#[cfg(feature = "ascii")]
pub mod ascii;

#[cfg(feature = "json")]
pub mod json;

#[cfg(all(feature = "image", feature = "std"))]
pub mod image;

#[cfg(feature = "svg")]
pub mod svg;
