//! Barkodların SVG gösterimlerini üretme işlevleri.
//!
//! SVG, standart yapı kurma söz dizimiyle ya da varsayılan değerler isteniyorsa bir kurucu metotla
//! oluşturulabilir.
//!
//! Örneğin:
//!
//! ```rust
//! use barcoders::generators::svg::*;
//!
//! // Yapı alanlarını kendiniz belirtin.
//! let svg = SVG{height: 80,
//!               xdim: 1,
//!               background: Color{rgba: [255, 0, 0, 255]},
//!               foreground: Color::black(),
//!               xmlns: Some(String::from("http://www.w3.org/2000/svg"))};
//!
//! // Ya da varsayılanlar için kurucuyu kullanın (yüksekliği belirtmeniz gerekir).
//! let svg = SVG::new(100)
//!               .xdim(2)
//!               .background(Color::white())
//!               .foreground(Color::black())
//!               .xmlns(String::from("http://www.w3.org/2000/svg"));
//! ```

use crate::error::{Error, Result};
use crate::generators::{validate_barcode, validate_output_bytes};
#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
};

trait ToHex {
    fn to_hex(self) -> String;

    fn format_hex(n: u8) -> String {
        format!(
            "{}{}",
            Self::to_hex_digit(n / 16),
            Self::to_hex_digit(n % 16)
        )
    }

    fn to_hex_digit(n: u8) -> char {
        match n {
            d if d < 10 => (d + 48) as char,
            d if d < 16 => (d + 87) as char,
            _ => '0',
        }
    }
}

/// Barkodun ön ve arka planında kullanılan bir RGBA rengini temsil eder.
#[derive(Copy, Clone, Debug)]
pub struct Color {
    /// Kırmızı, yeşil, mavi ve alfa değeri.
    pub rgba: [u8; 4],
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

    fn to_opacity(self) -> String {
        format!("{:.*}", 2, (self.rgba[3] as f64 / 255.0))
    }
}

impl ToHex for Color {
    fn to_hex(self) -> String {
        self.rgba
            .iter()
            .take(3)
            .map(|&c| Self::format_hex(c))
            .collect()
    }
}

/// SVG barkod üreteci türü.
#[derive(Clone, Debug)]
pub struct SVG {
    /// Barkodun yüksekliği (SVG çıktısında `self.height` piksel yüksekliğindedir).
    pub height: u32,
    /// X boyutu; "dar" çubukların genişliğini belirler.
    /// SVG çıktısında her çubuk `self.xdim` piksel genişliğindedir.
    pub xdim: u32,
    /// Ön planın RGBA rengi.
    pub foreground: Color,
    /// Arka planın RGBA rengi.
    pub background: Color,
    /// XML ad alanı.
    pub xmlns: Option<String>,
}

impl SVG {
    /// Varsayılan değerlerle yeni bir SVG üreteci döndürür.
    pub fn new(height: u32) -> SVG {
        SVG {
            height,
            xdim: 1,
            foreground: Color {
                rgba: [0, 0, 0, 255],
            },
            background: Color {
                rgba: [255, 255, 255, 255],
            },
            xmlns: None,
        }
    }

    /// SVG'nin XML ad alanını (`xmlns`) ayarlar.
    pub fn xmlns(mut self, xmlns_uri: String) -> Self {
        self.xmlns = Some(xmlns_uri);
        self
    }

    /// X boyutundaki çubuk genişliğini ayarlar.
    pub fn xdim(mut self, xdim: u32) -> Self {
        self.xdim = xdim;
        self
    }

    /// Ön plan (çubuk) rengini ayarlar.
    pub fn foreground(mut self, color: Color) -> Self {
        self.foreground = color;
        self
    }

    /// Arka plan rengini ayarlar.
    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    fn rect(&self, style: u8, offset: u32, width: u32) -> String {
        let fill = match style {
            1 => self.foreground,
            _ => self.background,
        };

        let opacity_value = fill.to_opacity();
        let opacity = match opacity_value.as_str() {
            "1.00" | "1" => "".to_string(),
            o => format!(" fill-opacity=\"{}\" ", o),
        };

        format!(
            "<rect x=\"{}\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"#{}\"{}/>",
            offset,
            width,
            self.height,
            fill.to_hex(),
            opacity
        )
    }

    /// Verilen barkodu üretir; başarı durumunda SVG verisini döndürür.
    pub fn generate<T: AsRef<[u8]>>(&self, barcode: T) -> Result<String> {
        let barcode = barcode.as_ref();
        validate_barcode(barcode)?;
        if self.height == 0 {
            return Err(Error::dimension("SVG yüksekliği sıfır olamaz"));
        }
        if self.xdim == 0 {
            return Err(Error::dimension("SVG modül genişliği sıfır olamaz"));
        }

        let barcode_len = u32::try_from(barcode.len())
            .map_err(|_| Error::dimension("SVG modül sayısı u32 aralığını aşıyor"))?;
        let width = barcode_len
            .checked_mul(self.xdim)
            .ok_or_else(|| Error::dimension("SVG genişliği u32 aralığını aşıyor"))?;
        let bar_count = barcode.iter().filter(|digit| **digit == 1).count();
        let estimated_size = bar_count
            .checked_mul(128)
            .and_then(|size| size.checked_add(512))
            .ok_or_else(|| Error::dimension("SVG çıktı tahmini usize aralığını aşıyor"))?;
        let requested = u64::try_from(estimated_size)
            .map_err(|_| Error::dimension("SVG çıktı tahmini u64 aralığını aşıyor"))?;
        validate_output_bytes(requested)?;
        let mut rects = String::with_capacity(estimated_size);

        for (index, &digit) in barcode.iter().enumerate() {
            if digit == 1 {
                let index = u32::try_from(index)
                    .map_err(|_| Error::dimension("SVG modül konumu u32 aralığını aşıyor"))?;
                let offset = index.checked_mul(self.xdim).ok_or_else(|| {
                    Error::dimension("SVG dikdörtgen konumu u32 aralığını aşıyor")
                })?;
                rects.push_str(self.rect(digit, offset, self.xdim).as_str());
            }
        }

        let xmlns = match &self.xmlns {
            Some(xmlns) => format!("xmlns=\"{xmlns}\" "),
            None => "".to_string(),
        };

        Ok(format!(
            "<svg version=\"1.1\" {x}viewBox=\"0 0 {w} {h}\">{s}{r}</svg>",
            x = xmlns,
            w = width,
            h = self.height,
            s = self.rect(0, 0, width),
            r = rects
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::error::{Error, Result};
    use crate::generators::svg::*;
    use crate::sym::codabar::*;
    use crate::sym::code11::*;
    use crate::sym::code39::*;
    use crate::sym::code93::*;
    use crate::sym::code128::*;
    use crate::sym::ean_supp::*;
    use crate::sym::ean8::*;
    use crate::sym::ean13::*;
    use crate::sym::tf::*;
    #[cfg(feature = "std")]
    use std::fs::File;
    #[cfg(feature = "std")]
    use std::io::BufWriter;
    #[cfg(feature = "std")]
    use std::io::prelude::*;
    #[cfg(feature = "std")]
    use std::path::Path;

    const TEST_DATA_BASE: &str = "./target/debug";
    const WRITE_TO_FILE: bool = true;

    #[cfg(feature = "std")]
    fn write_file(data: &str, file: &'static str) -> Result<()> {
        let path = open_file(file)?;
        let mut writer = BufWriter::new(path);
        writer
            .write_all(data.as_bytes())
            .map_err(|_| Error::generate("test dosyası", "SVG verisi dosyaya yazılamadı"))
    }

    #[cfg(not(feature = "std"))]
    fn write_file(_data: &str, _file: &'static str) -> Result<()> {
        Ok(())
    }

    #[cfg(feature = "std")]
    fn open_file(name: &'static str) -> Result<File> {
        let path = format!("{TEST_DATA_BASE}/{name}");
        File::create(Path::new(path.as_str()))
            .map_err(|_| Error::generate("test dosyası", "SVG dosyası oluşturulamadı"))
    }

    #[test]
    fn ean_13_as_svg() -> Result<()> {
        let ean13 = EAN13::new("750103131130")?;
        let svg = SVG::new(80);
        let generated = svg.generate(ean13.encode())?;

        if WRITE_TO_FILE {
            write_file(generated.as_str(), "ean13.svg")?;
        }

        assert_eq!(generated.len(), 2890);
        Ok(())
    }

    #[test]
    fn colored_ean_13_as_svg() -> Result<()> {
        let ean13 = EAN13::new("750103131130")?;
        let svg = SVG {
            height: 80,
            xdim: 1,
            background: Color {
                rgba: [255, 0, 0, 255],
            },
            foreground: Color {
                rgba: [0, 0, 255, 255],
            },
            xmlns: None,
        };
        let generated = svg.generate(ean13.encode())?;

        if WRITE_TO_FILE {
            write_file(generated.as_str(), "ean13_colored.svg")?;
        }

        assert_eq!(generated.len(), 2890);
        Ok(())
    }

    #[test]
    fn colored_semi_transparent_ean_13_as_svg() -> Result<()> {
        let ean13 = EAN13::new("750103131130")?;
        let svg = SVG {
            height: 70,
            xdim: 1,
            background: Color {
                rgba: [255, 0, 0, 128],
            },
            foreground: Color {
                rgba: [0, 0, 255, 128],
            },
            xmlns: None,
        };
        let generated = svg.generate(ean13.encode())?;

        if WRITE_TO_FILE {
            write_file(generated.as_str(), "ean13_colored_semi_transparent.svg")?;
        }

        assert_eq!(generated.len(), 3940);
        Ok(())
    }

    #[test]
    fn ean_8_as_svg() -> Result<()> {
        let ean8 = EAN8::new("9998823")?;
        let svg = SVG::new(80).xmlns("http://www.w3.org/2000/svg".to_string());
        let generated = svg.generate(ean8.encode())?;

        if WRITE_TO_FILE {
            write_file(generated.as_str(), "ean8.svg")?;
        }

        assert_eq!(generated.len(), 1956);
        Ok(())
    }

    #[test]
    fn code39_as_svg() -> Result<()> {
        let code39 = Code39::new("IGOT99PROBLEMS")?;
        let svg = SVG::new(80).xmlns("http://www.w3.org/2000/svg".to_string());
        let generated = svg.generate(code39.encode())?;

        if WRITE_TO_FILE {
            write_file(generated.as_str(), "code39.svg")?;
        }

        assert_eq!(generated.len(), 6574);
        Ok(())
    }

    #[test]
    fn code93_as_svg() -> Result<()> {
        let code93 = Code93::new("IGOT99PROBLEMS")?;
        let svg = SVG::new(80).xmlns("http://www.w3.org/2000/svg".to_string());
        let generated = svg.generate(code93.encode())?;

        if WRITE_TO_FILE {
            write_file(generated.as_str(), "code93.svg")?;
        }

        assert_eq!(generated.len(), 4493);
        Ok(())
    }

    #[test]
    fn codabar_as_svg() -> Result<()> {
        let codabar = Codabar::new("A12----34A")?;
        let svg = SVG::new(80).xmlns("http://www.w3.org/2000/svg".to_string());
        let generated = svg.generate(codabar.encode())?;

        if WRITE_TO_FILE {
            write_file(generated.as_str(), "codabar.svg")?;
        }

        assert_eq!(generated.len(), 2985);
        Ok(())
    }

    #[test]
    fn code128_as_svg() -> Result<()> {
        let code128 = Code128::new("ÀHIĆ345678")?;
        let svg = SVG::new(80).xmlns("http://www.w3.org/2000/svg".to_string());
        let generated = svg.generate(code128.encode())?;

        if WRITE_TO_FILE {
            write_file(generated.as_str(), "code128.svg")?;
        }

        assert_eq!(generated.len(), 2758);
        Ok(())
    }

    #[test]
    fn ean_2_as_svg() -> Result<()> {
        let ean2 = EANSUPP::new("78")?;
        let svg = SVG::new(80).xmlns("http://www.w3.org/2000/svg".to_string());
        let generated = svg.generate(ean2.encode())?;

        if WRITE_TO_FILE {
            write_file(generated.as_str(), "ean2.svg")?;
        }

        assert_eq!(generated.len(), 760);
        Ok(())
    }

    #[test]
    fn itf_as_svg() -> Result<()> {
        let itf = TF::interleaved("1234123488993344556677118")?;
        let svg = SVG {
            height: 80,
            xdim: 1,
            background: Color::black(),
            foreground: Color::white(),
            xmlns: None,
        };
        let generated = svg.generate(itf.encode())?;

        if WRITE_TO_FILE {
            write_file(generated.as_str(), "itf.svg")?;
        }

        assert_eq!(generated.len(), 7123);
        Ok(())
    }

    #[test]
    fn code11_as_svg() -> Result<()> {
        let code11 = Code11::new("9988-45643201")?;
        let svg = SVG {
            height: 80,
            xdim: 1,
            background: Color::black(),
            foreground: Color::white(),
            xmlns: None,
        };
        let generated = svg.generate(code11.encode())?;

        if WRITE_TO_FILE {
            write_file(generated.as_str(), "code11.svg")?;
        }

        assert_eq!(generated.len(), 4219);
        Ok(())
    }

    #[test]
    fn rejects_dimension_overflow() -> Result<()> {
        let generated = SVG::new(1).xdim(u32::MAX).generate([0, 1]);

        assert!(matches!(generated, Err(Error::Dimension { .. })));
        Ok(())
    }

    #[test]
    fn rejects_empty_encoding() -> Result<()> {
        let generated = SVG::new(80).generate([]);

        assert!(matches!(
            generated,
            Err(Error::Length {
                min: 1,
                max: None,
                found: 0
            })
        ));
        Ok(())
    }
}
