//! Supported methods of barcode generation.
//!
//! Each generation option is an optionally-compiled feature, which you must opt-into when
//! compiling.
//!
//! For example:
//!
//! ```toml
//! [dependencies]
//! barcoders = {version = "*", features = ["image"]}
//! ```
//!
//! Features:
//! - `ascii`: Generate ASCII-art barcodes.
//! - `json`: Generate JSON barcodes.
//! - `image`: Generate image-based barcodes.
//! - `svg`: Generate SVG barcodes.

use crate::error::{Error, Result};

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
