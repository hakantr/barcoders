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
//! // Kurucu, bağımsız bir SVG dosyası için standart SVG ad alanını ekler.
//! let svg = SVG::new(100)
//!               .xdim(2)
//!               .background(Color::white())
//!               .foreground(Color::black());
//! ```

use crate::error::{Error, Result};
use crate::generators::{validate_barcode, validate_output_bytes};
#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
};

/// Bağımsız SVG belgelerinde kullanılan standart XML ad alanı.
pub const SVG_NAMESPACE: &str = "http://www.w3.org/2000/svg";

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
        format!("{}", self.rgba[3] as f64 / 255.0)
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
    /// XML ad alanı. `SVG::new`, bağımsız belgeler için standart SVG ad alanını kullanır;
    /// `None`, öğe başka bir XML belgesine gömülecekse niteliği tamamen kaldırır.
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
            xmlns: Some(SVG_NAMESPACE.to_string()),
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

        let opacity = match fill.rgba[3] {
            255 => "".to_string(),
            _ => format!(" fill-opacity=\"{}\" ", fill.to_opacity()),
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
        let xmlns_len = match &self.xmlns {
            Some(xmlns) => escaped_xml_attribute_len(xmlns)?
                .checked_add(9)
                .ok_or_else(|| Error::dimension("SVG ad alanı uzunluğu usize aralığını aşıyor"))?,
            None => 0,
        };
        let rect_count = barcode
            .iter()
            .filter(|digit| **digit == 1)
            .count()
            .checked_add(1)
            .ok_or_else(|| Error::dimension("SVG dikdörtgen sayısı usize aralığını aşıyor"))?;
        // Her dikdörtgenin sayısal alanları ve saydamlık değeri dahil 256 bayttan kısa olması
        // güvence altındadır. Bu üst sınır, büyük çıktıyı oluşturmadan önce kaynak denetimi sağlar.
        let estimated_size = rect_count
            .checked_mul(256)
            .and_then(|size| size.checked_add(128))
            .and_then(|size| size.checked_add(xmlns_len))
            .ok_or_else(|| Error::dimension("SVG çıktı tahmini usize aralığını aşıyor"))?;
        let requested = u64::try_from(estimated_size)
            .map_err(|_| Error::dimension("SVG çıktı tahmini u64 aralığını aşıyor"))?;
        validate_output_bytes(requested)?;
        let mut generated = String::with_capacity(estimated_size);
        generated.push_str("<svg version=\"1.1\" ");
        if let Some(xmlns) = &self.xmlns {
            generated.push_str("xmlns=\"");
            push_escaped_xml_attribute(&mut generated, xmlns);
            generated.push_str("\" ");
        }
        generated.push_str(format!("viewBox=\"0 0 {} {}\">", width, self.height).as_str());
        generated.push_str(self.rect(0, 0, width).as_str());

        for (index, &digit) in barcode.iter().enumerate() {
            if digit == 1 {
                let index = u32::try_from(index)
                    .map_err(|_| Error::dimension("SVG modül konumu u32 aralığını aşıyor"))?;
                let offset = index.checked_mul(self.xdim).ok_or_else(|| {
                    Error::dimension("SVG dikdörtgen konumu u32 aralığını aşıyor")
                })?;
                generated.push_str(self.rect(digit, offset, self.xdim).as_str());
            }
        }
        generated.push_str("</svg>");

        Ok(generated)
    }
}

fn escaped_xml_attribute_len(value: &str) -> Result<usize> {
    if value.is_empty() {
        return Err(Error::generate("SVG", "XML ad alanı boş olamaz"));
    }

    value.chars().try_fold(0usize, |length, character| {
        if !is_xml_character(character) {
            return Err(Error::generate(
                "SVG",
                "XML ad alanı geçersiz bir XML karakteri içeriyor",
            ));
        }

        let escaped_len = match character {
            '&' => 5,
            '<' | '>' => 4,
            '"' | '\'' => 6,
            _ => character.len_utf8(),
        };
        length
            .checked_add(escaped_len)
            .ok_or_else(|| Error::dimension("SVG ad alanı uzunluğu usize aralığını aşıyor"))
    })
}

fn is_xml_character(character: char) -> bool {
    matches!(
        character,
        '\u{0009}' | '\u{000A}' | '\u{000D}' | '\u{0020}'..='\u{D7FF}' | '\u{E000}'..='\u{FFFD}' | '\u{10000}'..='\u{10FFFF}'
    )
}

fn push_escaped_xml_attribute(output: &mut String, value: &str) {
    for character in value.chars() {
        output.push_str(match character {
            '&' => "&amp;",
            '<' => "&lt;",
            '>' => "&gt;",
            '"' => "&quot;",
            '\'' => "&apos;",
            _ => {
                output.push(character);
                continue;
            }
        });
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

    #[cfg(feature = "std")]
    const TEST_DATA_BASE: &str = "./target/debug";
    const WRITE_TO_FILE: bool = true;

    fn assert_svg_structure(svg: &SVG, barcode: &[u8], generated: &str) -> Result<()> {
        let width = u32::try_from(barcode.len())
            .map_err(|_| Error::dimension("test barkodu u32 aralığını aşıyor"))?
            .checked_mul(svg.xdim)
            .ok_or_else(|| Error::dimension("test SVG genişliği u32 aralığını aşıyor"))?;

        assert!(generated.starts_with("<svg version=\"1.1\" "));
        assert!(generated.ends_with("</svg>"));
        assert!(generated.contains(format!("viewBox=\"0 0 {width} {}\"", svg.height).as_str()));
        assert_eq!(
            generated.matches("<rect ").count(),
            barcode.iter().filter(|digit| **digit == 1).count() + 1
        );

        match &svg.xmlns {
            Some(xmlns) => {
                let mut escaped = String::new();
                push_escaped_xml_attribute(&mut escaped, xmlns);
                assert!(generated.contains(format!("xmlns=\"{escaped}\" ").as_str()));
            }
            None => assert!(!generated.contains("xmlns=")),
        }

        Ok(())
    }

    fn next_opacity<'a, I>(values: &mut I) -> Result<f64>
    where
        I: Iterator<Item = &'a str>,
    {
        values
            .next()
            .and_then(|value| value.split('"').next())
            .ok_or_else(|| Error::generate("SVG testi", "saydamlık niteliği bulunamadı"))?
            .parse::<f64>()
            .map_err(|_| Error::generate("SVG testi", "saydamlık değeri çözümlenemedi"))
    }

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
        let barcode = ean13.encode();
        let generated = svg.generate(&barcode)?;

        if WRITE_TO_FILE {
            write_file(generated.as_str(), "ean13.svg")?;
        }

        assert_svg_structure(&svg, &barcode, &generated)?;
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
        let barcode = ean13.encode();
        let generated = svg.generate(&barcode)?;

        if WRITE_TO_FILE {
            write_file(generated.as_str(), "ean13_colored.svg")?;
        }

        assert_svg_structure(&svg, &barcode, &generated)?;
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
        let barcode = ean13.encode();
        let generated = svg.generate(&barcode)?;

        if WRITE_TO_FILE {
            write_file(generated.as_str(), "ean13_colored_semi_transparent.svg")?;
        }

        assert_svg_structure(&svg, &barcode, &generated)?;
        Ok(())
    }

    #[test]
    fn ean_8_as_svg() -> Result<()> {
        let ean8 = EAN8::new("9998823")?;
        let svg = SVG::new(80).xmlns("http://www.w3.org/2000/svg".to_string());
        let barcode = ean8.encode();
        let generated = svg.generate(&barcode)?;

        if WRITE_TO_FILE {
            write_file(generated.as_str(), "ean8.svg")?;
        }

        assert_svg_structure(&svg, &barcode, &generated)?;
        Ok(())
    }

    #[test]
    fn code39_as_svg() -> Result<()> {
        let code39 = Code39::new("IGOT99PROBLEMS")?;
        let svg = SVG::new(80).xmlns("http://www.w3.org/2000/svg".to_string());
        let barcode = code39.encode();
        let generated = svg.generate(&barcode)?;

        if WRITE_TO_FILE {
            write_file(generated.as_str(), "code39.svg")?;
        }

        assert_svg_structure(&svg, &barcode, &generated)?;
        Ok(())
    }

    #[test]
    fn code93_as_svg() -> Result<()> {
        let code93 = Code93::new("IGOT99PROBLEMS")?;
        let svg = SVG::new(80).xmlns("http://www.w3.org/2000/svg".to_string());
        let barcode = code93.encode();
        let generated = svg.generate(&barcode)?;

        if WRITE_TO_FILE {
            write_file(generated.as_str(), "code93.svg")?;
        }

        assert_svg_structure(&svg, &barcode, &generated)?;
        Ok(())
    }

    #[test]
    fn codabar_as_svg() -> Result<()> {
        let codabar = Codabar::new("A12----34A")?;
        let svg = SVG::new(80).xmlns("http://www.w3.org/2000/svg".to_string());
        let barcode = codabar.encode();
        let generated = svg.generate(&barcode)?;

        if WRITE_TO_FILE {
            write_file(generated.as_str(), "codabar.svg")?;
        }

        assert_svg_structure(&svg, &barcode, &generated)?;
        Ok(())
    }

    #[test]
    fn code128_as_svg() -> Result<()> {
        let code128 = Code128::new("ÀHIĆ345678")?;
        let svg = SVG::new(80).xmlns("http://www.w3.org/2000/svg".to_string());
        let barcode = code128.encode();
        let generated = svg.generate(&barcode)?;

        if WRITE_TO_FILE {
            write_file(generated.as_str(), "code128.svg")?;
        }

        assert_svg_structure(&svg, &barcode, &generated)?;
        Ok(())
    }

    #[test]
    fn ean_2_as_svg() -> Result<()> {
        let ean2 = EANSUPP::new("78")?;
        let svg = SVG::new(80).xmlns("http://www.w3.org/2000/svg".to_string());
        let barcode = ean2.encode();
        let generated = svg.generate(&barcode)?;

        if WRITE_TO_FILE {
            write_file(generated.as_str(), "ean2.svg")?;
        }

        assert_svg_structure(&svg, &barcode, &generated)?;
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
        let barcode = itf.encode();
        let generated = svg.generate(&barcode)?;

        if WRITE_TO_FILE {
            write_file(generated.as_str(), "itf.svg")?;
        }

        assert_svg_structure(&svg, &barcode, &generated)?;
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
        let barcode = code11.encode();
        let generated = svg.generate(&barcode)?;

        if WRITE_TO_FILE {
            write_file(generated.as_str(), "code11.svg")?;
        }

        assert_svg_structure(&svg, &barcode, &generated)?;
        Ok(())
    }

    #[test]
    fn default_svg_uses_the_standard_namespace() -> Result<()> {
        let svg = SVG::new(1);
        let generated = svg.generate([1])?;

        assert_eq!(svg.xmlns.as_deref(), Some(SVG_NAMESPACE));
        assert!(generated.contains("xmlns=\"http://www.w3.org/2000/svg\""));
        Ok(())
    }

    #[test]
    fn escapes_custom_xml_namespace_attributes() -> Result<()> {
        let svg = SVG::new(1).xmlns("https://example.test/\" onload=\"alert(1)&<>'".to_string());
        let generated = svg.generate([1])?;

        assert!(generated.contains(
            "xmlns=\"https://example.test/&quot; onload=&quot;alert(1)&amp;&lt;&gt;&apos;\""
        ));
        assert!(!generated.contains("\" onload=\""));
        Ok(())
    }

    #[test]
    fn rejects_invalid_or_empty_xml_namespaces() {
        assert!(matches!(
            SVG::new(1).xmlns(String::new()).generate([1]),
            Err(Error::Generate { .. })
        ));
        assert!(matches!(
            SVG::new(1)
                .xmlns("http://www.w3.org/2000/svg\u{0001}".to_string())
                .generate([1]),
            Err(Error::Generate { .. })
        ));
    }

    #[test]
    fn escaped_namespace_is_included_in_the_resource_limit() -> Result<()> {
        let apostrophe_count = usize::try_from(crate::generators::MAX_OUTPUT_BYTES / 6 + 1)
            .map_err(|_| Error::dimension("test ad alanı uzunluğu usize aralığını aşıyor"))?;
        let generated = SVG::new(1)
            .xmlns("'".repeat(apostrophe_count))
            .generate([1]);

        assert!(matches!(generated, Err(Error::ResourceLimit { .. })));
        Ok(())
    }

    #[test]
    fn preserves_nonzero_and_nonopaque_alpha_values() -> Result<()> {
        let svg = SVG {
            height: 1,
            xdim: 1,
            background: Color::new([255, 255, 255, 254]),
            foreground: Color::new([0, 0, 0, 1]),
            xmlns: None,
        };
        let generated = svg.generate([1])?;
        let mut opacity_values = generated.split("fill-opacity=\"").skip(1);

        assert_eq!(next_opacity(&mut opacity_values)?, 254.0 / 255.0);
        assert_eq!(next_opacity(&mut opacity_values)?, 1.0 / 255.0);
        assert!(opacity_values.next().is_none());
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
