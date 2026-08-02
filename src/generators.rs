//! Desteklenen barkod üretme yöntemleri.
//!
//! Her üretim seçeneği isteğe bağlı derlenen bir özelliktir; derleme sırasında kullanmak
//! istediklerinizi açıkça etkinleştirmeniz gerekir.
//!
//! Örneğin:
//!
//! ```toml
//! [dependencies]
//! barcoders = { version = "*", default-features = false, features = ["image"] }
//! ```
//!
//! Özellikler:
//! - `ascii`: ASCII çizimi olarak barkod üretir.
//! - `gpui`: Yerel GPUI kaynak ağacı için barkod öğeleri üretir.
//! - `json`: JSON barkod gösterimi üretir.
//! - `image`: Görüntü tabanlı barkod üretir.
//! - `svg`: SVG barkod gösterimi üretir.

#[cfg(any(
    feature = "ascii",
    feature = "json",
    feature = "svg",
    all(feature = "image", feature = "std")
))]
use crate::encoding::validate_modules;
#[cfg(any(
    feature = "ascii",
    feature = "json",
    feature = "svg",
    all(feature = "image", feature = "std")
))]
use crate::error::Result;

#[cfg(any(
    feature = "ascii",
    feature = "json",
    feature = "svg",
    all(feature = "image", feature = "std")
))]
pub(crate) fn validate_barcode(barcode: &[u8]) -> Result<()> {
    validate_modules(barcode)
}

#[cfg(any(feature = "ascii", feature = "image", feature = "svg"))]
pub(crate) const MAX_OUTPUT_BYTES: u64 = 64 * 1024 * 1024;

#[cfg(any(feature = "ascii", feature = "image", feature = "svg"))]
pub(crate) fn validate_output_bytes(requested: u64) -> Result<()> {
    if requested > MAX_OUTPUT_BYTES {
        Err(crate::error::Error::ResourceLimit {
            resource: "çıktı baytı",
            requested,
            maximum: MAX_OUTPUT_BYTES,
        })
    } else {
        Ok(())
    }
}

#[cfg(feature = "ascii")]
pub mod ascii;

#[cfg(feature = "gpui")]
pub mod gpui;

#[cfg(feature = "json")]
pub mod json;

#[cfg(all(feature = "image", feature = "std"))]
pub mod image;

#[cfg(feature = "svg")]
pub mod svg;
