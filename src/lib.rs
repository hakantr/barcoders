//! # Barcoders
//! Barcoders, seçilen barkod sembolojisine uygun veriyi temel ikili yapıyı temsil eden bir
//! `Vec<u8>` değerine kodlamanızı sağlar. Ardından isteğe bağlı yerleşik üreteçlerden birini
//! (GIF veya PNG dışa aktarımı gibi) kullanabilir ya da kendi üretecinizi oluşturabilirsiniz.
//!
//! ## Güncel destek
//!
//! Barcoders'ın nihai amacı, yaygın sembolojilerin tümünü ve daha az kullanılanların çoğunu
//! kodlayabilmektir.
//!
//! ### Sembolojiler
//!
//! * EAN-13
//!   * UPC-A
//!   * JAN
//!   * Bookland
//! * EAN-8
//! * Ek EAN barkodları
//!   * EAN-2
//!   * EAN-5
//! * Code11
//!   * USD-8
//! * Code39
//! * Code93
//! * Code128 (A, B, C)
//! * 2-of-5
//!   * Aralıklı (ITF)
//!   * Standart (STF)
//! * Codabar
//! * Yeni türler eklenecek
//!
//! ### Üreteçler
//!
//! Her üreteç isteğe bağlı bir özellik olarak tanımlanır. İşlevinin uygulamanıza derlenmesi için
//! ilgili özelliği açıkça etkinleştirmeniz gerekir.
//!
//! * ASCII (özellik: `ascii`)
//! * JSON (özellik: `json`)
//! * SVG (özellik: `svg`)
//! * PNG (özellik: `image`)
//! * GIF (özellik: `image`)
//! * WEBP (özellik: `image`)
//! * Görüntü tamponu (özellik: `image`)
//! * Yerel GPUI `canvas` öğesi (özellik: `gpui`)
//! * Kendi üreteciniz
//!
//! ## Örnekler
//!
//! GitHub deposundaki örneklere bakın.

#![warn(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod encoding;
pub mod error;
pub mod generators;
pub mod sym;
