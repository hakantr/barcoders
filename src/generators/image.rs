//! Barkodların görüntü gösterimlerini üretme işlevleri.
//!
//! Her enum varyantı standart yapı kurma söz dizimiyle ya da varsayılan değerler isteniyorsa bir
//! kurucu metotla oluşturulabilir.
//!
//! Örneğin:
//!
//! ```rust
//! use barcoders::generators::image::*;
//!
//! // Yapı alanlarını kendiniz belirtin.
//! let png = Image::PNG{height: 80,
//!                      xdim: 1,
//!                      rotation: Rotation::Zero,
//!                      foreground: Color::new([0, 0, 0, 255]),
//!                      background: Color::new([255, 255, 255, 255])};
//!
//! // Ya da varsayılanlar için kurucuyu kullanın (yüksekliği belirtmeniz gerekir).
//! let png = Image::png(100);
//! ```
//!
//! Daha fazla örnek için README belgesine bakın.

use crate::error::{Error, Result};
use crate::generators::{validate_barcode, validate_output_bytes};
use image::{
    DynamicImage::{self, ImageRgba8},
    ImageBuffer, ImageFormat, Rgba,
};
use std::io::Cursor;

macro_rules! image_variants {
    ( $( #[$attr:meta] $v:ident ),* ) => {
        /// Görüntü üreteci türü.
        #[derive(Copy, Clone, Debug)]
        pub enum Image {
        $(
            #[$attr]
            $v {
                /// Barkodun piksel cinsinden yüksekliği.
                height: u32,
                /// X boyutu; her biri `self.xdim` piksel genişliğinde olan "dar"
                /// çubukların genişliğini belirler.
                xdim: u32,
                /// Üretilen barkoda uygulanacak döndürme.
                rotation: Rotation,
                /// Ön planın RGBA rengi.
                foreground: Color,
                /// Arka planın RGBA rengi.
                background: Color,
            },
        )*
        }
    };
}

macro_rules! image_defaults {
    ($v:ident, $h:expr) => {
        Image::$v {
            height: $h,
            xdim: 1,
            rotation: Rotation::Zero,
            foreground: Color {
                rgba: [0, 0, 0, 255],
            },
            background: Color {
                rgba: [255, 255, 255, 255],
            },
        }
    };
}

macro_rules! expand_image_variants {
    ($s:expr, $b:tt => $e:tt, $($v:ident),+) => (
        match $s {
            $(
                Image::$v$b => $e
            ),+
        }
    );
}

/// Barkodun ön ve arka planında kullanılan bir RGBA rengini temsil eder.
#[derive(Copy, Clone, Debug)]
pub struct Color {
    /// Kırmızı, yeşil, mavi ve alfa değeri.
    rgba: [u8; 4],
}

impl Color {
    /// Yeni bir renk oluşturur.
    pub fn new(rgba: [u8; 4]) -> Color {
        Color { rgba }
    }

    /// Siyah (`#000000`) renk oluşturur.
    pub fn black() -> Color {
        Color::new([0, 0, 0, 255])
    }

    /// Beyaz (`#FFFFFF`) renk oluşturur.
    pub fn white() -> Color {
        Color::new([255, 255, 255, 255])
    }

    fn to_rgba(self) -> Rgba<u8> {
        Rgba(self.rgba)
    }
}

/// Görüntüler için kullanılabilen döndürme değerleri.
#[derive(Copy, Clone, Debug)]
pub enum Rotation {
    /// Döndürme uygulamaz; varsayılan değer budur.
    Zero,
    /// 90 derece döndürür.
    Ninety,
    /// 180 derece döndürür.
    OneEighty,
    /// 270 derece döndürür.
    TwoSeventy,
}

image_variants![
    /// GIF görüntü üreteci türü.
    GIF,
    /// PNG görüntü üreteci türü.
    PNG,
    /// WEBP görüntü üreteci türü.
    WEBP,
    /// Görüntü tamponu üreteci türü.
    ImageBuffer
];

impl Image {
    /// Varsayılan değerlerle yeni bir GIF üreteci döndürür.
    pub fn gif(height: u32) -> Image {
        image_defaults!(GIF, height)
    }

    /// Varsayılan değerlerle yeni bir PNG üreteci döndürür.
    pub fn png(height: u32) -> Image {
        image_defaults!(PNG, height)
    }

    /// Varsayılan değerlerle yeni bir WEBP üreteci döndürür.
    pub fn webp(height: u32) -> Image {
        image_defaults!(WEBP, height)
    }

    /// Varsayılan değerlerle yeni bir `ImageBuffer` üreteci döndürür.
    pub fn image_buffer(height: u32) -> Image {
        image_defaults!(ImageBuffer, height)
    }

    /// Verilen barkodu üretir; başarı durumunda kodlanmış baytları döndürür.
    pub fn generate<T: AsRef<[u8]>>(&self, barcode: T) -> Result<Vec<u8>> {
        let (format, target) = match *self {
            Image::GIF { .. } => (ImageFormat::Gif, "GIF"),
            Image::PNG { .. } => (ImageFormat::Png, "PNG"),
            Image::WEBP { .. } => (ImageFormat::WebP, "WEBP"),
            _ => {
                return Err(Error::generate(
                    "görüntü tamponu",
                    "bayt üretimi yerine generate_buffer kullanılmalı",
                ));
            }
        };

        let mut bytes: Vec<u8> = vec![];
        let img = self.place_pixels(&barcode)?;

        img.write_to(&mut Cursor::new(&mut bytes), format)
            .map_err(|_| Error::generate(target, "görüntü kodlayıcısı çıktıyı yazamadı"))?;
        Ok(bytes)
    }

    /// Verilen barkodu bir `image::ImageBuffer` içine üretir; başarı durumunda görüntü tamponunu
    /// döndürür.
    pub fn generate_buffer<T: AsRef<[u8]>>(
        self,
        barcode: T,
    ) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>> {
        let img = self.place_pixels(&barcode)?;

        Ok(img.to_rgba8())
    }

    fn place_pixels<T: AsRef<[u8]>>(&self, barcode: T) -> Result<DynamicImage> {
        let barcode = barcode.as_ref();
        validate_barcode(barcode)?;
        let (xdim, height, rotation, bg, fg) = expand_image_variants!(
            *self,
            {height: h, xdim: x, rotation: r, background: b, foreground: f} => (x, h, r, b.to_rgba(), f.to_rgba()),
            GIF, PNG, WEBP, ImageBuffer
        );
        if height == 0 {
            return Err(Error::dimension("görüntü yüksekliği sıfır olamaz"));
        }
        if xdim == 0 {
            return Err(Error::dimension("görüntü modül genişliği sıfır olamaz"));
        }

        let barcode_len = u32::try_from(barcode.len())
            .map_err(|_| Error::dimension("modül sayısı u32 aralığını aşıyor"))?;
        let width = barcode_len
            .checked_mul(xdim)
            .ok_or_else(|| Error::dimension("görüntü genişliği u32 aralığını aşıyor"))?;
        let row_channels = u64::from(width)
            .checked_mul(4)
            .ok_or_else(|| Error::dimension("satır kanal sayısı u64 aralığını aşıyor"))?;
        let channel_count = row_channels
            .checked_mul(u64::from(height))
            .ok_or_else(|| Error::dimension("RGBA kanal sayısı u64 aralığını aşıyor"))?;
        validate_output_bytes(channel_count)?;
        usize::try_from(channel_count)
            .map_err(|_| Error::dimension("RGBA kanal sayısı usize aralığını aşıyor"))?;
        let row_length = usize::try_from(row_channels)
            .map_err(|_| Error::dimension("satır kanal sayısı usize aralığını aşıyor"))?;

        // Barkod yalnız yatay eksende değiştiğinden tek bir satır kurulur ve tüm yüksekliğe
        // kopyalanır.
        let mut row: Vec<u8> = Vec::with_capacity(row_length);

        for &module in barcode {
            let color = if module == 0 { bg } else { fg };

            for _ in 0..xdim {
                row.extend_from_slice(&color.0);
            }
        }

        let mut buffer: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(width, height);

        for line in buffer.chunks_mut(row_length) {
            line.copy_from_slice(row.as_slice());
        }

        let img = ImageRgba8(buffer);

        Ok(match rotation {
            Rotation::Ninety => img.rotate90(),
            Rotation::OneEighty => img.rotate180(),
            Rotation::TwoSeventy => img.rotate270(),
            _ => img,
        })
    }
}

#[cfg(test)]
mod tests {
    extern crate image;

    use crate::error::{Error, Result};
    use crate::generators::image::*;
    use crate::sym::codabar::*;
    use crate::sym::code11::*;
    use crate::sym::code39::*;
    use crate::sym::code93::*;
    use crate::sym::code128::*;
    use crate::sym::ean_supp::*;
    use crate::sym::ean8::*;
    use crate::sym::ean13::*;
    use crate::sym::tf::*;
    use std::fs::File;
    use std::io::BufWriter;
    use std::io::prelude::*;
    use std::path::Path;

    const TEST_DATA_BASE: &str = "./target/debug";
    const WRITE_TO_FILE: bool = true;

    fn open_file(name: &'static str) -> Result<File> {
        let path = format!("{TEST_DATA_BASE}/{name}");
        File::create(Path::new(path.as_str()))
            .map_err(|_| Error::generate("test dosyası", "dosya oluşturulamadı"))
    }

    fn write_file(bytes: &[u8], file: &'static str) -> Result<()> {
        let path = open_file(file)?;
        let mut writer = BufWriter::new(path);
        writer
            .write_all(bytes)
            .map_err(|_| Error::generate("test dosyası", "baytlar dosyaya yazılamadı"))
    }

    fn assert_png(generator: Image, barcode: &[u8], generated: &[u8]) -> Result<()> {
        let decoded = image::load_from_memory_with_format(generated, ImageFormat::Png)
            .map_err(|_| Error::generate("PNG testi", "üretilen görüntü çözülemedi"))?
            .to_rgba8();
        let expected = generator.generate_buffer(barcode)?;

        assert_eq!(decoded.dimensions(), expected.dimensions());
        assert_eq!(decoded.as_raw(), expected.as_raw());
        Ok(())
    }

    #[test]
    fn ean_13_as_gif() -> Result<()> {
        let ean13 = EAN13::new("750103131130")?;
        let gif = Image::gif(80);
        let generated = gif.generate(ean13.encode())?;

        if WRITE_TO_FILE {
            write_file(generated.as_slice(), "ean13.gif")?;
        }

        assert_eq!(generated.len(), 918);
        Ok(())
    }

    #[test]
    fn ean_13_as_png() -> Result<()> {
        let ean13 = EAN13::new("750103131130")?;
        let png = Image::PNG {
            height: 100,
            xdim: 1,
            rotation: Rotation::Zero,
            foreground: Color {
                rgba: [0, 0, 0, 255],
            },
            background: Color {
                rgba: [255, 255, 255, 255],
            },
        };
        let generated = png.generate(ean13.encode())?;

        if WRITE_TO_FILE {
            write_file(generated.as_slice(), "ean13.png")?;
        }

        assert_png(png, &ean13.encode(), &generated)?;
        Ok(())
    }

    #[test]
    fn rotated_ean_13_as_png() -> Result<()> {
        let ean13 = EAN13::new("750103131130")?;
        let png = Image::PNG {
            height: 100,
            xdim: 1,
            rotation: Rotation::Ninety,
            foreground: Color {
                rgba: [0, 0, 0, 255],
            },
            background: Color {
                rgba: [255, 255, 255, 255],
            },
        };
        let generated = png.generate(ean13.encode())?;

        if WRITE_TO_FILE {
            write_file(generated.as_slice(), "ean13_90.png")?;
        }

        assert_png(png, &ean13.encode(), &generated)?;
        Ok(())
    }

    #[test]
    fn ean_13_as_webp() -> Result<()> {
        let ean13 = EAN13::new("999988881234")?;
        let webp = Image::WEBP {
            height: 100,
            xdim: 3,
            rotation: Rotation::Zero,
            foreground: Color {
                rgba: [0, 0, 0, 255],
            },
            background: Color {
                rgba: [255, 255, 255, 255],
            },
        };
        let generated = webp.generate(ean13.encode())?;

        if WRITE_TO_FILE {
            write_file(generated.as_slice(), "ean13.webp")?;
        }

        assert_eq!(generated.len(), 150);
        Ok(())
    }

    #[test]
    fn ean_13_as_image_buffer() -> Result<()> {
        let ean13 = EAN13::new("750399599113")?;
        let img = Image::ImageBuffer {
            height: 99,
            xdim: 1,
            rotation: Rotation::Zero,
            foreground: Color {
                rgba: [0, 0, 0, 255],
            },
            background: Color {
                rgba: [255, 255, 255, 255],
            },
        };
        let generated = img.generate_buffer(ean13.encode())?;

        assert_eq!(generated.height(), 99);
        assert_eq!(generated.width(), 95);
        Ok(())
    }

    #[test]
    fn colored_ean_13_as_gif() -> Result<()> {
        let ean13 = EAN13::new("750103131130")?;
        let gif = Image::GIF {
            height: 99,
            xdim: 1,
            rotation: Rotation::Zero,
            foreground: Color {
                rgba: [255, 38, 42, 255],
            },
            background: Color {
                rgba: [34, 52, 255, 255],
            },
        };

        let generated = gif.generate(ean13.encode())?;

        if WRITE_TO_FILE {
            write_file(generated.as_slice(), "colored_ean13.gif")?;
        }

        assert_eq!(generated.len(), 1084);
        Ok(())
    }

    #[test]
    fn colored_semi_opaque_ean_13_as_png() -> Result<()> {
        let ean13 = EAN13::new("750153666132")?;
        let png = Image::PNG {
            height: 99,
            xdim: 1,
            rotation: Rotation::Zero,
            foreground: Color {
                rgba: [255, 38, 42, 120],
            },
            background: Color {
                rgba: [34, 52, 255, 120],
            },
        };

        let generated = png.generate(ean13.encode())?;

        if WRITE_TO_FILE {
            write_file(generated.as_slice(), "colored_opaque_ean13.png")?;
        }

        assert_png(png, &ean13.encode(), &generated)?;
        Ok(())
    }

    #[test]
    fn code39_as_png() -> Result<()> {
        let code39 = Code39::new("ILOVEMEL")?;
        let png = Image::PNG {
            height: 60,
            xdim: 1,
            rotation: Rotation::Zero,
            foreground: Color {
                rgba: [0, 0, 0, 255],
            },
            background: Color {
                rgba: [255, 255, 255, 255],
            },
        };
        let generated = png.generate(code39.encode())?;

        if WRITE_TO_FILE {
            write_file(generated.as_slice(), "code39.png")?;
        }

        assert_png(png, &code39.encode(), &generated)?;
        Ok(())
    }

    #[test]
    fn code39_as_gif() -> Result<()> {
        let code39 = Code39::new("WIKIPEDIA")?;
        let gif = Image::GIF {
            height: 60,
            xdim: 1,
            rotation: Rotation::Zero,
            foreground: Color {
                rgba: [0, 0, 0, 255],
            },
            background: Color {
                rgba: [255, 255, 255, 255],
            },
        };
        let generated = gif.generate(code39.encode())?;

        if WRITE_TO_FILE {
            write_file(generated.as_slice(), "code39.gif")?;
        }

        assert_eq!(generated.len(), 911);
        Ok(())
    }

    #[test]
    fn rotated_code39_as_gif() -> Result<()> {
        let code39 = Code39::new("HELLOWORLD")?;
        let gif = Image::GIF {
            height: 60,
            xdim: 1,
            rotation: Rotation::OneEighty,
            foreground: Color {
                rgba: [0, 0, 0, 255],
            },
            background: Color {
                rgba: [255, 255, 255, 255],
            },
        };
        let generated = gif.generate(code39.encode())?;

        if WRITE_TO_FILE {
            write_file(generated.as_slice(), "code39_180.gif")?;
        }

        assert_eq!(generated.len(), 969);
        Ok(())
    }

    #[test]
    fn code93_as_png() -> Result<()> {
        let code93 = Code93::new("ILOVEBAH")?;
        let png = Image::PNG {
            height: 60,
            xdim: 1,
            rotation: Rotation::Zero,
            foreground: Color {
                rgba: [0, 0, 0, 255],
            },
            background: Color {
                rgba: [255, 255, 255, 255],
            },
        };
        let generated = png.generate(code93.encode())?;

        if WRITE_TO_FILE {
            write_file(generated.as_slice(), "code93.png")?;
        }

        assert_png(png, &code93.encode(), &generated)?;
        Ok(())
    }

    #[test]
    fn code93_as_gif() -> Result<()> {
        let code93 = Code93::new("CIVIC VIDEO")?;
        let gif = Image::GIF {
            height: 60,
            xdim: 1,
            rotation: Rotation::Zero,
            foreground: Color {
                rgba: [0, 0, 0, 255],
            },
            background: Color {
                rgba: [255, 255, 255, 255],
            },
        };
        let generated = gif.generate(code93.encode())?;

        if WRITE_TO_FILE {
            write_file(generated.as_slice(), "code93.gif")?;
        }

        assert_eq!(generated.len(), 982);
        Ok(())
    }

    #[test]
    fn rotated_code93_as_gif() -> Result<()> {
        let code93 = Code93::new("TWISTIES 100")?;
        let gif = Image::GIF {
            height: 60,
            xdim: 1,
            rotation: Rotation::OneEighty,
            foreground: Color {
                rgba: [0, 0, 0, 255],
            },
            background: Color {
                rgba: [255, 255, 255, 255],
            },
        };
        let generated = gif.generate(code93.encode())?;

        if WRITE_TO_FILE {
            write_file(generated.as_slice(), "code93_180.gif")?;
        }

        assert_eq!(generated.len(), 1061);
        Ok(())
    }

    #[test]
    fn code11_as_png() -> Result<()> {
        let code11 = Code11::new("9923-1111")?;
        let png = Image::PNG {
            height: 60,
            xdim: 1,
            rotation: Rotation::Zero,
            foreground: Color {
                rgba: [0, 0, 0, 255],
            },
            background: Color {
                rgba: [255, 255, 255, 255],
            },
        };
        let generated = png.generate(code11.encode())?;

        if WRITE_TO_FILE {
            write_file(generated.as_slice(), "code11.png")?;
        }

        assert_png(png, &code11.encode(), &generated)?;
        Ok(())
    }

    #[test]
    fn code11_as_gif() -> Result<()> {
        let code11 = Code11::new("122333444455556666")?;
        let gif = Image::GIF {
            height: 60,
            xdim: 1,
            rotation: Rotation::Zero,
            foreground: Color {
                rgba: [0, 0, 0, 255],
            },
            background: Color {
                rgba: [255, 255, 255, 255],
            },
        };
        let generated = gif.generate(code11.encode())?;

        if WRITE_TO_FILE {
            write_file(generated.as_slice(), "code11.gif")?;
        }

        assert_eq!(generated.len(), 981);
        Ok(())
    }

    #[test]
    fn codabar_as_png() -> Result<()> {
        let codabar = Codabar::new("B12354999A")?;
        let png = Image::PNG {
            height: 60,
            xdim: 1,
            rotation: Rotation::Zero,
            foreground: Color {
                rgba: [0, 0, 0, 255],
            },
            background: Color {
                rgba: [255, 255, 255, 255],
            },
        };
        let generated = png.generate(codabar.encode())?;

        if WRITE_TO_FILE {
            write_file(generated.as_slice(), "codabar.png")?;
        }

        assert_png(png, &codabar.encode(), &generated)?;
        Ok(())
    }

    #[test]
    fn codabar_as_gif() -> Result<()> {
        let codabar = Codabar::new("A5675+++3$$B")?;
        let gif = Image::GIF {
            height: 80,
            xdim: 2,
            rotation: Rotation::Zero,
            foreground: Color {
                rgba: [0, 0, 0, 255],
            },
            background: Color {
                rgba: [255, 255, 255, 255],
            },
        };
        let generated = gif.generate(codabar.encode())?;

        if WRITE_TO_FILE {
            write_file(generated.as_slice(), "codabar.gif")?;
        }

        assert_eq!(generated.len(), 1653);
        Ok(())
    }

    #[test]
    fn rotated_codabar_as_gif() -> Result<()> {
        let codabar = Codabar::new("C1234D")?;
        let gif = Image::GIF {
            height: 60,
            xdim: 1,
            rotation: Rotation::Ninety,
            foreground: Color {
                rgba: [0, 0, 0, 255],
            },
            background: Color {
                rgba: [255, 255, 255, 255],
            },
        };
        let generated = gif.generate(codabar.encode())?;

        if WRITE_TO_FILE {
            write_file(generated.as_slice(), "codabar_90.gif")?;
        }

        assert_eq!(generated.len(), 174);
        Ok(())
    }

    #[test]
    fn code128_as_png() -> Result<()> {
        let code128 = Code128::new("ÀHIĆ345678")?;
        let png = Image::PNG {
            height: 60,
            xdim: 1,
            rotation: Rotation::Zero,
            foreground: Color {
                rgba: [0, 0, 0, 255],
            },
            background: Color {
                rgba: [255, 255, 255, 255],
            },
        };
        let generated = png.generate(code128.encode())?;

        if WRITE_TO_FILE {
            write_file(generated.as_slice(), "code128.png")?;
        }

        assert_png(png, &code128.encode(), &generated)?;
        Ok(())
    }

    #[test]
    fn code128_as_gif() -> Result<()> {
        let code128 = Code128::new("ÀHELLOWORLD")?;
        let gif = Image::GIF {
            height: 90,
            xdim: 3,
            rotation: Rotation::Zero,
            foreground: Color {
                rgba: [0, 0, 0, 255],
            },
            background: Color {
                rgba: [255, 255, 255, 255],
            },
        };
        let generated = gif.generate(code128.encode())?;

        if WRITE_TO_FILE {
            write_file(generated.as_slice(), "code128.gif")?;
        }

        assert_eq!(generated.len(), 2741);
        Ok(())
    }

    #[test]
    fn rotated_code128_as_gif() -> Result<()> {
        let code128 = Code128::new("ÀHELLOWORLD")?;
        let gif = Image::GIF {
            height: 90,
            xdim: 3,
            rotation: Rotation::OneEighty,
            foreground: Color {
                rgba: [0, 0, 0, 255],
            },
            background: Color {
                rgba: [255, 255, 255, 255],
            },
        };
        let generated = gif.generate(code128.encode())?;

        if WRITE_TO_FILE {
            write_file(generated.as_slice(), "code128_180.gif")?;
        }

        assert_eq!(generated.len(), 2752);
        Ok(())
    }

    #[test]
    fn rotated_code128_as_image_buffer() -> Result<()> {
        let code128 = Code128::new("ƁCLOJURE")?;
        let img = Image::ImageBuffer {
            height: 93,
            xdim: 2,
            rotation: Rotation::OneEighty,
            foreground: Color {
                rgba: [0, 0, 0, 255],
            },
            background: Color {
                rgba: [255, 255, 255, 255],
            },
        };
        let generated = img.generate_buffer(code128.encode())?;

        assert_eq!(generated.height(), 93);
        assert_eq!(generated.width(), 224);
        Ok(())
    }

    #[test]
    fn ean8_as_png() -> Result<()> {
        let ean8 = EAN8::new("5512345")?;
        let png = Image::PNG {
            height: 70,
            xdim: 2,
            rotation: Rotation::Zero,
            foreground: Color {
                rgba: [0, 0, 0, 255],
            },
            background: Color {
                rgba: [255, 255, 255, 255],
            },
        };
        let generated = png.generate(ean8.encode())?;

        if WRITE_TO_FILE {
            write_file(generated.as_slice(), "ean8.png")?;
        }

        assert_png(png, &ean8.encode(), &generated)?;
        Ok(())
    }

    #[test]
    fn rotated_ean8_as_png() -> Result<()> {
        let ean8 = EAN8::new("5512345")?;
        let png = Image::PNG {
            height: 70,
            xdim: 2,
            rotation: Rotation::TwoSeventy,
            foreground: Color {
                rgba: [0, 0, 0, 255],
            },
            background: Color {
                rgba: [255, 255, 255, 255],
            },
        };
        let generated = png.generate(ean8.encode())?;

        if WRITE_TO_FILE {
            write_file(generated.as_slice(), "ean8_270.png")?;
        }

        assert_png(png, &ean8.encode(), &generated)?;
        Ok(())
    }

    #[test]
    fn ean8_as_gif() -> Result<()> {
        let ean8 = EAN8::new("9992227")?;
        let gif = Image::GIF {
            height: 70,
            xdim: 2,
            rotation: Rotation::Zero,
            foreground: Color {
                rgba: [0, 0, 0, 255],
            },
            background: Color {
                rgba: [255, 255, 255, 255],
            },
        };
        let generated = gif.generate(ean8.encode())?;

        if WRITE_TO_FILE {
            write_file(generated.as_slice(), "ean8.gif")?;
        }

        assert_eq!(generated.len(), 897);
        Ok(())
    }

    #[test]
    fn ean8_as_webp() -> Result<()> {
        let ean8 = EAN8::new("9992227")?;
        let webp = Image::WEBP {
            height: 70,
            xdim: 2,
            rotation: Rotation::Zero,
            foreground: Color {
                rgba: [0, 0, 0, 255],
            },
            background: Color {
                rgba: [255, 255, 255, 255],
            },
        };
        let generated = webp.generate(ean8.encode())?;

        if WRITE_TO_FILE {
            write_file(generated.as_slice(), "ean8.webp")?;
        }

        assert_eq!(generated.len(), 114);
        Ok(())
    }

    #[test]
    fn ean2_as_png() -> Result<()> {
        let ean2 = EANSUPP::new("94")?;
        let png = Image::PNG {
            height: 70,
            xdim: 2,
            rotation: Rotation::Zero,
            foreground: Color {
                rgba: [0, 0, 0, 255],
            },
            background: Color {
                rgba: [255, 255, 255, 255],
            },
        };
        let generated = png.generate(ean2.encode())?;

        if WRITE_TO_FILE {
            write_file(generated.as_slice(), "ean2.png")?;
        }

        assert_png(png, &ean2.encode(), &generated)?;
        Ok(())
    }

    #[test]
    fn ean5_as_gif() -> Result<()> {
        let ean5 = EANSUPP::new("51234")?;
        let gif = Image::GIF {
            height: 70,
            xdim: 2,
            rotation: Rotation::Zero,
            foreground: Color {
                rgba: [0, 0, 0, 255],
            },
            background: Color {
                rgba: [255, 255, 255, 255],
            },
        };
        let generated = gif.generate(ean5.encode())?;

        if WRITE_TO_FILE {
            write_file(generated.as_slice(), "ean5.gif")?;
        }

        assert_eq!(generated.len(), 655);
        Ok(())
    }

    #[test]
    fn ean5_as_webp() -> Result<()> {
        let ean5 = EANSUPP::new("51574")?;
        let webp = Image::WEBP {
            height: 140,
            xdim: 5,
            rotation: Rotation::Zero,
            foreground: Color {
                rgba: [0, 0, 0, 255],
            },
            background: Color {
                rgba: [255, 255, 255, 255],
            },
        };
        let generated = webp.generate(ean5.encode())?;

        if WRITE_TO_FILE {
            write_file(generated.as_slice(), "ean5.webp")?;
        }

        assert_eq!(generated.len(), 124);
        Ok(())
    }

    #[test]
    fn ean5_as_imagebuffer() -> Result<()> {
        let ean5 = EANSUPP::new("99888")?;
        let img = Image::ImageBuffer {
            height: 140,
            xdim: 1,
            rotation: Rotation::Zero,
            foreground: Color {
                rgba: [0, 0, 0, 255],
            },
            background: Color {
                rgba: [255, 255, 255, 255],
            },
        };
        let generated = img.generate_buffer(ean5.encode())?;

        assert_eq!(generated.height(), 140);
        assert_eq!(generated.width(), 47);
        Ok(())
    }

    #[test]
    fn itf_as_png() -> Result<()> {
        let itf = TF::interleaved("1234567")?;
        let png = Image::PNG {
            height: 100,
            xdim: 2,
            rotation: Rotation::Zero,
            foreground: Color {
                rgba: [0, 0, 0, 255],
            },
            background: Color {
                rgba: [255, 255, 255, 255],
            },
        };
        let generated = png.generate(itf.encode())?;

        if WRITE_TO_FILE {
            write_file(generated.as_slice(), "itf.png")?;
        }

        assert_png(png, &itf.encode(), &generated)?;
        Ok(())
    }

    #[test]
    fn stf_as_png() -> Result<()> {
        let stf = TF::standard("1234567")?;
        let png = Image::PNG {
            height: 100,
            xdim: 2,
            rotation: Rotation::Zero,
            foreground: Color {
                rgba: [0, 0, 0, 255],
            },
            background: Color {
                rgba: [255, 255, 255, 255],
            },
        };
        let generated = png.generate(stf.encode())?;

        if WRITE_TO_FILE {
            write_file(generated.as_slice(), "stf.png")?;
        }

        assert_png(png, &stf.encode(), &generated)?;
        Ok(())
    }

    #[test]
    fn itf_as_gif() -> Result<()> {
        let itf = TF::interleaved("98766543561")?;
        let gif = Image::GIF {
            height: 130,
            xdim: 1,
            rotation: Rotation::Zero,
            foreground: Color {
                rgba: [0, 0, 0, 255],
            },
            background: Color {
                rgba: [255, 255, 255, 255],
            },
        };
        let generated = gif.generate(itf.encode())?;

        if WRITE_TO_FILE {
            write_file(generated.as_slice(), "itf.gif")?;
        }

        assert_eq!(generated.len(), 1410);
        Ok(())
    }

    #[test]
    fn itf_as_webp() -> Result<()> {
        let itf = TF::interleaved("98766543561")?;
        let webp = Image::WEBP {
            height: 130,
            xdim: 1,
            rotation: Rotation::Zero,
            foreground: Color {
                rgba: [0, 0, 0, 255],
            },
            background: Color {
                rgba: [255, 255, 255, 255],
            },
        };
        let generated = webp.generate(itf.encode())?;

        if WRITE_TO_FILE {
            write_file(generated.as_slice(), "itf.webp")?;
        }

        assert_eq!(generated.len(), 116);
        Ok(())
    }

    #[test]
    fn itf_as_imagebuffer() -> Result<()> {
        let itf = TF::interleaved("98766543561")?;
        let img = Image::ImageBuffer {
            height: 130,
            xdim: 1,
            rotation: Rotation::Zero,
            foreground: Color {
                rgba: [0, 0, 0, 255],
            },
            background: Color {
                rgba: [255, 255, 255, 255],
            },
        };
        let generated = img.generate_buffer(itf.encode())?;

        assert_eq!(generated.height(), 130);
        assert_eq!(generated.width(), 116);
        Ok(())
    }

    #[test]
    fn image_buffer_fails_on_generate() -> Result<()> {
        let itf = TF::interleaved("98766543561")?;
        let img = Image::ImageBuffer {
            height: 130,
            xdim: 1,
            rotation: Rotation::Zero,
            foreground: Color {
                rgba: [0, 0, 0, 255],
            },
            background: Color {
                rgba: [255, 255, 255, 255],
            },
        };

        assert!(img.generate(itf.encode()).is_err());
        Ok(())
    }

    #[test]
    fn rejects_dimension_overflow() -> Result<()> {
        let img = Image::PNG {
            height: 1,
            xdim: u32::MAX,
            rotation: Rotation::Zero,
            foreground: Color::black(),
            background: Color::white(),
        };

        assert!(matches!(
            img.generate_buffer([0, 1]),
            Err(Error::Dimension { .. })
        ));
        Ok(())
    }

    #[test]
    fn rejects_excessive_image_before_allocating() -> Result<()> {
        let img = Image::PNG {
            height: 10_000,
            xdim: 1_000,
            rotation: Rotation::Zero,
            foreground: Color::black(),
            background: Color::white(),
        };

        assert!(matches!(
            img.generate_buffer([0, 1]),
            Err(Error::ResourceLimit {
                resource: "çıktı baytı",
                ..
            })
        ));
        Ok(())
    }

    #[test]
    fn rejects_empty_encoding() -> Result<()> {
        assert!(matches!(
            Image::png(80).generate_buffer([]),
            Err(Error::Length {
                min: 1,
                max: None,
                found: 0
            })
        ));
        Ok(())
    }

    #[test]
    fn every_row_repeats_the_module_pattern() -> Result<()> {
        let img = Image::ImageBuffer {
            height: 3,
            xdim: 2,
            rotation: Rotation::Zero,
            foreground: Color::new([1, 2, 3, 4]),
            background: Color::new([5, 6, 7, 8]),
        };
        let generated = img.generate_buffer([1, 0])?;

        assert_eq!(generated.dimensions(), (4, 3));
        assert_eq!(
            generated.as_raw().as_slice(),
            [
                1, 2, 3, 4, 1, 2, 3, 4, 5, 6, 7, 8, 5, 6, 7, 8, //
                1, 2, 3, 4, 1, 2, 3, 4, 5, 6, 7, 8, 5, 6, 7, 8, //
                1, 2, 3, 4, 1, 2, 3, 4, 5, 6, 7, 8, 5, 6, 7, 8,
            ]
        );
        Ok(())
    }
}
