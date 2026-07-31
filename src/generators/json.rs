//! Barkodların JSON gösterimlerini üretme işlevleri.
//!
//! Kodlanmış veriyi standart bir biçimde üçüncü taraf sistemlere aktarmakta kullanışlıdır.
//!
//! Çıktı şu biçimdedir:
//! ```javascript
//! {
//!   "height": 10,
//!   "xdim": 1,
//!   "encoding": [1, 0, 0, 1, 1, 0, ...],
//! }
//! ```

use crate::error::Result;
use crate::generators::validate_barcode;
#[cfg(not(feature = "std"))]
use alloc::{format, string::String};

/// JSON barkod üreteci türü.
#[derive(Copy, Clone, Debug)]
pub struct JSON {
    /// Barkodun yüksekliği.
    pub height: usize,
    /// X boyutu; "dar" çubukların genişliğini belirler.
    pub xdim: usize,
}

impl Default for JSON {
    fn default() -> Self {
        Self::new()
    }
}

impl JSON {
    /// Varsayılan değerlerle yeni bir JSON üreteci döndürür.
    pub fn new() -> JSON {
        JSON {
            height: 10,
            xdim: 1,
        }
    }

    /// Verilen barkodu üretir; başarı durumunda JSON metnini döndürür.
    pub fn generate<T: AsRef<[u8]>>(&self, barcode: T) -> Result<String> {
        let barcode = barcode.as_ref();
        validate_barcode(barcode)?;
        let mut bits = barcode.iter().fold(String::new(), |acc, &b| {
            let n = match b {
                0 => "0",
                _ => "1",
            };

            acc + n + ","
        });

        // Sondaki virgülü kaldır.
        bits.pop();

        let output = format!(
            "{{\"height\":{},\"xdim\":{},\"encoding\":[{}]}}",
            self.height, self.xdim, bits
        );

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use crate::error::{Error, Result};
    use crate::generators::json::*;
    use crate::sym::codabar::*;
    use crate::sym::code11::*;
    use crate::sym::code39::*;
    use crate::sym::code93::*;
    use crate::sym::code128::*;
    use crate::sym::ean_supp::*;
    use crate::sym::ean8::*;
    use crate::sym::ean13::*;
    use crate::sym::tf::*;

    #[test]
    fn ean_13_as_json() -> Result<()> {
        let ean13 = EAN13::new("750103131130")?;
        let json = JSON::new();
        let generated = json.generate(ean13.encode())?;

        assert_eq!(generated, "{\"height\":10,\"xdim\":1,\"encoding\":[1,0,1,0,1,1,0,0,0,1,0,1,0,0,1,1,1,0,0,1,1,0,0,1,0,1,0,0,1,1,1,0,1,1,1,1,0,1,0,1,1,0,0,1,1,0,1,0,1,0,1,0,0,0,0,1,0,1,1,0,0,1,1,0,1,1,0,0,1,1,0,1,0,0,0,0,1,0,1,1,1,0,0,1,0,1,1,1,0,1,0,0,1,0,1]}".trim());
        Ok(())
    }

    #[test]
    fn ean_13_as_json_small_height_double_width() -> Result<()> {
        let ean13 = EAN13::new("750103131130")?;
        let json = JSON { height: 6, xdim: 2 };
        let generated = json.generate(ean13.encode())?;

        assert_eq!(generated, "{\"height\":6,\"xdim\":2,\"encoding\":[1,0,1,0,1,1,0,0,0,1,0,1,0,0,1,1,1,0,0,1,1,0,0,1,0,1,0,0,1,1,1,0,1,1,1,1,0,1,0,1,1,0,0,1,1,0,1,0,1,0,1,0,0,0,0,1,0,1,1,0,0,1,1,0,1,1,0,0,1,1,0,1,0,0,0,0,1,0,1,1,1,0,0,1,0,1,1,1,0,1,0,0,1,0,1]}".trim());
        Ok(())
    }

    #[test]
    fn ean_8_as_json() -> Result<()> {
        let ean8 = EAN8::new("1234567")?;
        let json = JSON::new();
        let generated = json.generate(ean8.encode())?;

        assert_eq!(generated, "{\"height\":10,\"xdim\":1,\"encoding\":[1,0,1,0,0,1,1,0,0,1,0,0,1,0,0,1,1,0,1,1,1,1,0,1,0,1,0,0,0,1,1,0,1,0,1,0,1,0,0,1,1,1,0,1,0,1,0,0,0,0,1,0,0,0,1,0,0,1,1,1,0,0,1,0,1,0,1]}".trim());
        Ok(())
    }

    #[test]
    fn ean_8_as_json_small_height_double_width() -> Result<()> {
        let ean8 = EAN8::new("1234567")?;
        let json = JSON { height: 5, xdim: 2 };
        let generated = json.generate(ean8.encode())?;

        assert_eq!(generated, "{\"height\":5,\"xdim\":2,\"encoding\":[1,0,1,0,0,1,1,0,0,1,0,0,1,0,0,1,1,0,1,1,1,1,0,1,0,1,0,0,0,1,1,0,1,0,1,0,1,0,0,1,1,1,0,1,0,1,0,0,0,0,1,0,0,0,1,0,0,1,1,1,0,0,1,0,1,0,1]}".trim());
        Ok(())
    }

    #[test]
    fn code_93_as_json() -> Result<()> {
        let code93 = Code93::new("MONKEYMAGIC")?;
        let json = JSON::new();
        let generated = json.generate(code93.encode())?;

        assert_eq!(generated, "{\"height\":10,\"xdim\":1,\"encoding\":[1,0,1,0,1,1,1,1,0,1,0,1,0,0,1,1,0,0,1,0,0,1,0,1,1,0,0,1,0,1,0,0,0,1,1,0,1,0,0,0,1,1,0,1,0,1,1,0,0,1,0,0,1,0,1,0,0,1,1,0,1,1,0,1,0,1,0,0,1,1,0,0,1,1,0,1,0,1,0,0,0,1,0,1,1,0,1,0,0,0,1,0,1,1,0,0,0,1,0,1,1,0,1,0,0,0,1,0,1,0,0,1,1,0,1,1,0,1,0,1,0,0,1,0,0,0,1,0,1,0,1,1,1,1,0,1]}".trim());
        Ok(())
    }

    #[test]
    fn code_93_as_json_small_height_double_weight() -> Result<()> {
        let code93 = Code93::new("1234")?;
        let json = JSON { height: 7, xdim: 2 };
        let generated = json.generate(code93.encode())?;

        assert_eq!(generated, "{\"height\":7,\"xdim\":2,\"encoding\":[1,0,1,0,1,1,1,1,0,1,0,1,0,0,1,0,0,0,1,0,1,0,0,0,1,0,0,1,0,1,0,0,0,0,1,0,1,0,0,1,0,1,0,0,0,1,0,0,0,1,1,0,1,0,1,0,1,0,0,0,0,1,0,1,0,1,0,1,1,1,1,0,1]}".trim());
        Ok(())
    }

    #[test]
    fn code_39_as_json() -> Result<()> {
        let code39 = Code39::new("TEST8052")?;
        let json = JSON::new();
        let generated = json.generate(code39.encode())?;

        assert_eq!(generated, "{\"height\":10,\"xdim\":1,\"encoding\":[1,0,0,1,0,1,1,0,1,1,0,1,0,1,0,1,0,1,1,0,1,1,0,0,1,0,1,1,0,1,0,1,1,0,0,1,0,1,0,1,0,1,1,0,1,0,1,1,0,0,1,0,1,0,1,0,1,1,0,1,1,0,0,1,0,1,1,0,1,0,0,1,0,1,1,0,1,0,1,0,1,0,0,1,1,0,1,1,0,1,0,1,1,0,1,0,0,1,1,0,1,0,1,0,1,0,1,1,0,0,1,0,1,0,1,1,0,1,0,0,1,0,1,1,0,1,1,0,1]}".trim());
        Ok(())
    }

    #[test]
    fn code_39_as_json_small_height_double_weight() -> Result<()> {
        let code39 = Code39::new("1234")?;
        let json = JSON { height: 7, xdim: 2 };
        let generated = json.generate(code39.encode())?;

        assert_eq!(generated, "{\"height\":7,\"xdim\":2,\"encoding\":[1,0,0,1,0,1,1,0,1,1,0,1,0,1,1,0,1,0,0,1,0,1,0,1,1,0,1,0,1,1,0,0,1,0,1,0,1,1,0,1,1,0,1,1,0,0,1,0,1,0,1,0,1,0,1,0,0,1,1,0,1,0,1,1,0,1,0,0,1,0,1,1,0,1,1,0,1]}".trim());
        Ok(())
    }

    #[test]
    fn codabar_as_json() -> Result<()> {
        let codabar = Codabar::new("A98B")?;
        let json = JSON::new();
        let generated = json.generate(codabar.encode())?;

        assert_eq!(generated, "{\"height\":10,\"xdim\":1,\"encoding\":[1,0,1,1,0,0,1,0,0,1,0,1,1,0,1,0,0,1,0,1,0,1,0,0,1,1,0,1,0,1,0,1,0,1,0,0,1,0,0,1,1]}".trim());
        Ok(())
    }

    #[test]
    fn codabar_as_json_small_height_double_weight() -> Result<()> {
        let codabar = Codabar::new("A40156B")?;
        let json = JSON { height: 7, xdim: 2 };
        let generated = json.generate(codabar.encode())?;

        assert_eq!(generated, "{\"height\":7,\"xdim\":2,\"encoding\":[1,0,1,1,0,0,1,0,0,1,0,1,0,1,1,0,1,0,0,1,0,1,0,1,0,1,0,0,1,1,0,1,0,1,0,1,1,0,0,1,0,1,1,0,1,0,1,0,0,1,0,1,0,0,1,0,1,0,1,1,0,1,0,1,0,0,1,0,0,1,1]}".trim());
        Ok(())
    }

    #[test]
    fn code_128_as_json() -> Result<()> {
        let code128 = Code128::new("ÀHELLO")?;
        let json = JSON::new();
        let generated = json.generate(code128.encode())?;

        assert_eq!(generated, "{\"height\":10,\"xdim\":1,\"encoding\":[1,1,0,1,0,0,0,0,1,0,0,1,1,0,0,0,1,0,1,0,0,0,1,0,0,0,1,1,0,1,0,0,0,1,0,0,0,1,1,0,1,1,1,0,1,0,0,0,1,1,0,1,1,1,0,1,0,0,0,1,1,1,0,1,1,0,1,1,0,1,0,0,0,1,0,0,0,1,1,0,0,0,1,1,1,0,1,0,1,1]}".trim());
        Ok(())
    }

    #[test]
    fn code_128_as_json_small_height_double_weight() -> Result<()> {
        let code128 = Code128::new("ÀHELLO")?;
        let json = JSON { height: 7, xdim: 2 };
        let generated = json.generate(code128.encode())?;

        assert_eq!(generated, "{\"height\":7,\"xdim\":2,\"encoding\":[1,1,0,1,0,0,0,0,1,0,0,1,1,0,0,0,1,0,1,0,0,0,1,0,0,0,1,1,0,1,0,0,0,1,0,0,0,1,1,0,1,1,1,0,1,0,0,0,1,1,0,1,1,1,0,1,0,0,0,1,1,1,0,1,1,0,1,1,0,1,0,0,0,1,0,0,0,1,1,0,0,0,1,1,1,0,1,0,1,1]}".trim());
        Ok(())
    }

    #[test]
    fn ean2_as_json() -> Result<()> {
        let ean2 = EANSUPP::new("34")?;
        let json = JSON::new();
        let generated = json.generate(ean2.encode())?;

        assert_eq!(
            generated,
            "{\"height\":10,\"xdim\":1,\"encoding\":[1,0,1,1,0,1,0,0,0,0,1,0,1,0,1,0,0,0,1,1]}"
                .trim()
        );
        Ok(())
    }

    #[test]
    fn ean5_as_json() -> Result<()> {
        let ean5 = EANSUPP::new("50799")?;
        let json = JSON::new();
        let generated = json.generate(ean5.encode())?;

        assert_eq!(generated, "{\"height\":10,\"xdim\":1,\"encoding\":[1,0,1,1,0,1,1,0,0,0,1,0,1,0,1,0,0,1,1,1,0,1,0,0,1,0,0,0,1,0,1,0,0,0,1,0,1,1,0,1,0,0,0,1,0,1,1]}".trim());
        Ok(())
    }

    #[test]
    fn itf_as_json() -> Result<()> {
        let itf = TF::interleaved("12345")?;
        let json = JSON::new();
        let generated = json.generate(itf.encode())?;

        assert_eq!(generated, "{\"height\":10,\"xdim\":1,\"encoding\":[1,0,1,0,1,1,1,0,1,0,0,0,1,0,1,0,1,1,1,0,0,0,1,1,1,0,1,1,1,0,1,0,0,0,1,0,1,0,0,0,1,1,1,0,1,0,1,1,1,0,1,0,0,0,1,0,0,0,1,1,0,1]}".trim());
        Ok(())
    }

    #[test]
    fn code11_as_json() -> Result<()> {
        let code11 = Code11::new("111-999-8")?;
        let json = JSON::new();
        let generated = json.generate(code11.encode())?;

        assert_eq!(generated, "{\"height\":10,\"xdim\":1,\"encoding\":[1,0,1,1,0,0,1,0,1,1,0,1,0,1,1,0,1,1,0,1,0,1,1,0,1,1,0,1,0,1,1,0,1,0,1,1,0,1,0,1,1,0,1,0,1,0,1,1,0,1,0,1,0,1,1,0,1,0,1,0,1,0,1,1,0,1,0,1,1,0,1,0,0,1,0,1,0,1,0,1,1,0,1,0,1,1,0,0,1]}".trim());
        Ok(())
    }

    #[test]
    fn rejects_non_binary_encoding() -> Result<()> {
        let generated = JSON::new().generate([0, 2, 1]);

        assert!(matches!(generated, Err(Error::InvalidEncoding)));
        Ok(())
    }
}
