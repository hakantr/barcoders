//! Barkodların ASCII gösterimlerini üretme işlevleri.
//!
//! Testlerde ve barkodun doğruluğunu basitçe denetlemekte kullanışlıdır.
//!
//! Yeni bir işlev eklemediğiniz veya test takımını çalıştırmadığınız sürece bu özelliği etkinleştirmeniz
//! genellikle gerekmez.

use crate::error::{Error, Result};
use crate::generators::{validate_barcode, validate_output_bytes};
#[cfg(not(feature = "std"))]
use alloc::string::String;
use core::iter::repeat_n;

/// ASCII barkod üreteci türü.
#[derive(Copy, Clone, Debug)]
pub struct ASCII {
    /// Barkodun yüksekliği (ASCII çıktısında `self.height` karakter yüksekliğindedir).
    pub height: usize,
    /// X boyutu; "dar" çubukların genişliğini belirler.
    /// ASCII çıktısında her çubuk `self.xdim` karakter genişliğindedir.
    pub xdim: usize,
}

/// İkili basamakları ASCII gösterimine eşler (`0` = `' '`, `1` = `'#'`).
const CHARS: [char; 2] = [' ', '#'];

impl Default for ASCII {
    fn default() -> Self {
        Self::new()
    }
}

impl ASCII {
    /// Varsayılan değerlerle yeni bir ASCII üreteci döndürür.
    pub fn new() -> ASCII {
        ASCII {
            height: 10,
            xdim: 1,
        }
    }

    fn generate_row(&self, barcode: &[u8], capacity: usize) -> Result<String> {
        let mut row = String::with_capacity(capacity);

        for (index, &digit) in barcode.iter().enumerate() {
            let character =
                CHARS
                    .get(usize::from(digit))
                    .copied()
                    .ok_or(Error::InvalidEncoding {
                        index,
                        value: digit,
                    })?;
            row.extend(repeat_n(character, self.xdim));
        }

        Ok(row)
    }

    /// Verilen barkodu üretir; başarı durumunda metin çıktısını döndürür.
    pub fn generate<T: AsRef<[u8]>>(&self, barcode: T) -> Result<String> {
        let barcode = barcode.as_ref();
        validate_barcode(barcode)?;

        if self.height == 0 {
            return Err(Error::dimension("ASCII yüksekliği sıfır olamaz"));
        }
        if self.xdim == 0 {
            return Err(Error::dimension("ASCII modül genişliği sıfır olamaz"));
        }

        let row_width = barcode
            .len()
            .checked_mul(self.xdim)
            .ok_or_else(|| Error::dimension("ASCII satır genişliği usize aralığını aşıyor"))?;
        let rows_size = row_width
            .checked_mul(self.height)
            .ok_or_else(|| Error::dimension("ASCII çıktı uzunluğu usize aralığını aşıyor"))?;
        let output_size = rows_size
            .checked_add(self.height.saturating_sub(1))
            .ok_or_else(|| Error::dimension("ASCII satır sonları usize aralığını aşıyor"))?;
        let requested = u64::try_from(output_size)
            .map_err(|_| Error::dimension("ASCII çıktı uzunluğu u64 aralığını aşıyor"))?;
        validate_output_bytes(requested)?;

        let row = self.generate_row(barcode, row_width)?;
        let mut output = String::with_capacity(output_size);

        for (i, _l) in (0..self.height).enumerate() {
            output.push_str(row.as_str());

            if i < self.height.saturating_sub(1) {
                output.push('\n');
            }
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use crate::error::{Error, Result};
    use crate::generators::ascii::*;
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
    fn ean_13_as_ascii() -> Result<()> {
        let ean13 = EAN13::new("750103131130")?;
        let ascii = ASCII::new();
        let generated = ascii.generate(ean13.encode())?;

        assert_eq!(
            generated,
            "
# # ##   # #  ###  ##  # #  ### #### # ##  ## # # #    # ##  ## ##  ## #    # ###  # ### #  # #
# # ##   # #  ###  ##  # #  ### #### # ##  ## # # #    # ##  ## ##  ## #    # ###  # ### #  # #
# # ##   # #  ###  ##  # #  ### #### # ##  ## # # #    # ##  ## ##  ## #    # ###  # ### #  # #
# # ##   # #  ###  ##  # #  ### #### # ##  ## # # #    # ##  ## ##  ## #    # ###  # ### #  # #
# # ##   # #  ###  ##  # #  ### #### # ##  ## # # #    # ##  ## ##  ## #    # ###  # ### #  # #
# # ##   # #  ###  ##  # #  ### #### # ##  ## # # #    # ##  ## ##  ## #    # ###  # ### #  # #
# # ##   # #  ###  ##  # #  ### #### # ##  ## # # #    # ##  ## ##  ## #    # ###  # ### #  # #
# # ##   # #  ###  ##  # #  ### #### # ##  ## # # #    # ##  ## ##  ## #    # ###  # ### #  # #
# # ##   # #  ###  ##  # #  ### #### # ##  ## # # #    # ##  ## ##  ## #    # ###  # ### #  # #
# # ##   # #  ###  ##  # #  ### #### # ##  ## # # #    # ##  ## ##  ## #    # ###  # ### #  # #
"
            .trim()
        );
        Ok(())
    }

    #[test]
    fn ean_13_as_ascii_small_height_double_width() -> Result<()> {
        let ean13 = EAN13::new("750103131130")?;
        let ascii = ASCII { height: 6, xdim: 2 };
        let generated = ascii.generate(ean13.encode())?;

        assert_eq!(generated,
"
##  ##  ####      ##  ##    ######    ####    ##  ##    ######  ########  ##  ####    ####  ##  ##  ##        ##  ####    ####  ####    ####  ##        ##  ######    ##  ######  ##    ##  ##
##  ##  ####      ##  ##    ######    ####    ##  ##    ######  ########  ##  ####    ####  ##  ##  ##        ##  ####    ####  ####    ####  ##        ##  ######    ##  ######  ##    ##  ##
##  ##  ####      ##  ##    ######    ####    ##  ##    ######  ########  ##  ####    ####  ##  ##  ##        ##  ####    ####  ####    ####  ##        ##  ######    ##  ######  ##    ##  ##
##  ##  ####      ##  ##    ######    ####    ##  ##    ######  ########  ##  ####    ####  ##  ##  ##        ##  ####    ####  ####    ####  ##        ##  ######    ##  ######  ##    ##  ##
##  ##  ####      ##  ##    ######    ####    ##  ##    ######  ########  ##  ####    ####  ##  ##  ##        ##  ####    ####  ####    ####  ##        ##  ######    ##  ######  ##    ##  ##
##  ##  ####      ##  ##    ######    ####    ##  ##    ######  ########  ##  ####    ####  ##  ##  ##        ##  ####    ####  ####    ####  ##        ##  ######    ##  ######  ##    ##  ##
".trim());
        Ok(())
    }

    #[test]
    fn ean_8_as_ascii() -> Result<()> {
        let ean8 = EAN8::new("1234567")?;
        let ascii = ASCII::new();
        let generated = ascii.generate(ean8.encode())?;

        assert_eq!(
            generated,
            "
# #  ##  #  #  ## #### # #   ## # # #  ### # #    #   #  ###  # # #
# #  ##  #  #  ## #### # #   ## # # #  ### # #    #   #  ###  # # #
# #  ##  #  #  ## #### # #   ## # # #  ### # #    #   #  ###  # # #
# #  ##  #  #  ## #### # #   ## # # #  ### # #    #   #  ###  # # #
# #  ##  #  #  ## #### # #   ## # # #  ### # #    #   #  ###  # # #
# #  ##  #  #  ## #### # #   ## # # #  ### # #    #   #  ###  # # #
# #  ##  #  #  ## #### # #   ## # # #  ### # #    #   #  ###  # # #
# #  ##  #  #  ## #### # #   ## # # #  ### # #    #   #  ###  # # #
# #  ##  #  #  ## #### # #   ## # # #  ### # #    #   #  ###  # # #
# #  ##  #  #  ## #### # #   ## # # #  ### # #    #   #  ###  # # #
"
            .trim()
        );
        Ok(())
    }

    #[test]
    fn ean_8_as_ascii_small_height_double_width() -> Result<()> {
        let ean8 = EAN8::new("1234567")?;
        let ascii = ASCII { height: 5, xdim: 2 };
        let generated = ascii.generate(ean8.encode())?;

        assert_eq!(generated,
"
##  ##    ####    ##    ##    ####  ########  ##  ##      ####  ##  ##  ##    ######  ##  ##        ##      ##    ######    ##  ##  ##
##  ##    ####    ##    ##    ####  ########  ##  ##      ####  ##  ##  ##    ######  ##  ##        ##      ##    ######    ##  ##  ##
##  ##    ####    ##    ##    ####  ########  ##  ##      ####  ##  ##  ##    ######  ##  ##        ##      ##    ######    ##  ##  ##
##  ##    ####    ##    ##    ####  ########  ##  ##      ####  ##  ##  ##    ######  ##  ##        ##      ##    ######    ##  ##  ##
##  ##    ####    ##    ##    ####  ########  ##  ##      ####  ##  ##  ##    ######  ##  ##        ##      ##    ######    ##  ##  ##
".trim());
        Ok(())
    }

    #[test]
    fn code_39_as_ascii() -> Result<()> {
        let code39 = Code39::new("TEST8052")?;
        let ascii = ASCII::new();
        let generated = ascii.generate(code39.encode())?;

        assert_eq!(generated,
"
#  # ## ## # # # ## ##  # ## # ##  # # # ## # ##  # # # ## ##  # ## #  # ## # # #  ## ## # ## #  ## # # # ##  # # ## #  # ## ## #
#  # ## ## # # # ## ##  # ## # ##  # # # ## # ##  # # # ## ##  # ## #  # ## # # #  ## ## # ## #  ## # # # ##  # # ## #  # ## ## #
#  # ## ## # # # ## ##  # ## # ##  # # # ## # ##  # # # ## ##  # ## #  # ## # # #  ## ## # ## #  ## # # # ##  # # ## #  # ## ## #
#  # ## ## # # # ## ##  # ## # ##  # # # ## # ##  # # # ## ##  # ## #  # ## # # #  ## ## # ## #  ## # # # ##  # # ## #  # ## ## #
#  # ## ## # # # ## ##  # ## # ##  # # # ## # ##  # # # ## ##  # ## #  # ## # # #  ## ## # ## #  ## # # # ##  # # ## #  # ## ## #
#  # ## ## # # # ## ##  # ## # ##  # # # ## # ##  # # # ## ##  # ## #  # ## # # #  ## ## # ## #  ## # # # ##  # # ## #  # ## ## #
#  # ## ## # # # ## ##  # ## # ##  # # # ## # ##  # # # ## ##  # ## #  # ## # # #  ## ## # ## #  ## # # # ##  # # ## #  # ## ## #
#  # ## ## # # # ## ##  # ## # ##  # # # ## # ##  # # # ## ##  # ## #  # ## # # #  ## ## # ## #  ## # # # ##  # # ## #  # ## ## #
#  # ## ## # # # ## ##  # ## # ##  # # # ## # ##  # # # ## ##  # ## #  # ## # # #  ## ## # ## #  ## # # # ##  # # ## #  # ## ## #
#  # ## ## # # # ## ##  # ## # ##  # # # ## # ##  # # # ## ##  # ## #  # ## # # #  ## ## # ## #  ## # # # ##  # # ## #  # ## ## #
".trim());
        Ok(())
    }

    #[test]
    fn code_39_as_ascii_small_height_double_weight() -> Result<()> {
        let code39 = Code39::new("1234")?;
        let ascii = ASCII { height: 7, xdim: 2 };
        let generated = ascii.generate(code39.encode())?;

        assert_eq!(generated,
"
##    ##  ####  ####  ##  ####  ##    ##  ##  ####  ##  ####    ##  ##  ####  ####  ####    ##  ##  ##  ##  ##    ####  ##  ####  ##    ##  ####  ####  ##
##    ##  ####  ####  ##  ####  ##    ##  ##  ####  ##  ####    ##  ##  ####  ####  ####    ##  ##  ##  ##  ##    ####  ##  ####  ##    ##  ####  ####  ##
##    ##  ####  ####  ##  ####  ##    ##  ##  ####  ##  ####    ##  ##  ####  ####  ####    ##  ##  ##  ##  ##    ####  ##  ####  ##    ##  ####  ####  ##
##    ##  ####  ####  ##  ####  ##    ##  ##  ####  ##  ####    ##  ##  ####  ####  ####    ##  ##  ##  ##  ##    ####  ##  ####  ##    ##  ####  ####  ##
##    ##  ####  ####  ##  ####  ##    ##  ##  ####  ##  ####    ##  ##  ####  ####  ####    ##  ##  ##  ##  ##    ####  ##  ####  ##    ##  ####  ####  ##
##    ##  ####  ####  ##  ####  ##    ##  ##  ####  ##  ####    ##  ##  ####  ####  ####    ##  ##  ##  ##  ##    ####  ##  ####  ##    ##  ####  ####  ##
##    ##  ####  ####  ##  ####  ##    ##  ##  ####  ##  ####    ##  ##  ####  ####  ####    ##  ##  ##  ##  ##    ####  ##  ####  ##    ##  ####  ####  ##
".trim());
        Ok(())
    }

    #[test]
    fn codabar_as_ascii() -> Result<()> {
        let codabar = Codabar::new("A98B")?;
        let ascii = ASCII::new();
        let generated = ascii.generate(codabar.encode())?;

        assert_eq!(
            generated,
            "
# ##  #  # ## #  # # #  ## # # # #  #  ##
# ##  #  # ## #  # # #  ## # # # #  #  ##
# ##  #  # ## #  # # #  ## # # # #  #  ##
# ##  #  # ## #  # # #  ## # # # #  #  ##
# ##  #  # ## #  # # #  ## # # # #  #  ##
# ##  #  # ## #  # # #  ## # # # #  #  ##
# ##  #  # ## #  # # #  ## # # # #  #  ##
# ##  #  # ## #  # # #  ## # # # #  #  ##
# ##  #  # ## #  # # #  ## # # # #  #  ##
# ##  #  # ## #  # # #  ## # # # #  #  ##
"
            .trim()
        );
        Ok(())
    }

    #[test]
    fn codabar_as_ascii_small_height_double_weight() -> Result<()> {
        let codabar = Codabar::new("A40156B")?;
        let ascii = ASCII { height: 7, xdim: 2 };
        let generated = ascii.generate(codabar.encode())?;

        assert_eq!(generated,
"
##  ####    ##    ##  ##  ####  ##    ##  ##  ##  ##    ####  ##  ##  ####    ##  ####  ##  ##    ##  ##    ##  ##  ####  ##  ##    ##    ####
##  ####    ##    ##  ##  ####  ##    ##  ##  ##  ##    ####  ##  ##  ####    ##  ####  ##  ##    ##  ##    ##  ##  ####  ##  ##    ##    ####
##  ####    ##    ##  ##  ####  ##    ##  ##  ##  ##    ####  ##  ##  ####    ##  ####  ##  ##    ##  ##    ##  ##  ####  ##  ##    ##    ####
##  ####    ##    ##  ##  ####  ##    ##  ##  ##  ##    ####  ##  ##  ####    ##  ####  ##  ##    ##  ##    ##  ##  ####  ##  ##    ##    ####
##  ####    ##    ##  ##  ####  ##    ##  ##  ##  ##    ####  ##  ##  ####    ##  ####  ##  ##    ##  ##    ##  ##  ####  ##  ##    ##    ####
##  ####    ##    ##  ##  ####  ##    ##  ##  ##  ##    ####  ##  ##  ####    ##  ####  ##  ##    ##  ##    ##  ##  ####  ##  ##    ##    ####
##  ####    ##    ##  ##  ####  ##    ##  ##  ##  ##    ####  ##  ##  ####    ##  ####  ##  ##    ##  ##    ##  ##  ####  ##  ##    ##    ####
".trim());
        Ok(())
    }

    #[test]
    fn code_128_as_ascii() -> Result<()> {
        let code128 = Code128::new("ÀHELLO")?;
        let ascii = ASCII::new();
        let generated = ascii.generate(code128.encode())?;

        assert_eq!(
            generated,
            "
## #    #  ##   # #   #   ## #   #   ## ### #   ## ### #   ### ## ## #   #   ##   ### # ##
## #    #  ##   # #   #   ## #   #   ## ### #   ## ### #   ### ## ## #   #   ##   ### # ##
## #    #  ##   # #   #   ## #   #   ## ### #   ## ### #   ### ## ## #   #   ##   ### # ##
## #    #  ##   # #   #   ## #   #   ## ### #   ## ### #   ### ## ## #   #   ##   ### # ##
## #    #  ##   # #   #   ## #   #   ## ### #   ## ### #   ### ## ## #   #   ##   ### # ##
## #    #  ##   # #   #   ## #   #   ## ### #   ## ### #   ### ## ## #   #   ##   ### # ##
## #    #  ##   # #   #   ## #   #   ## ### #   ## ### #   ### ## ## #   #   ##   ### # ##
## #    #  ##   # #   #   ## #   #   ## ### #   ## ### #   ### ## ## #   #   ##   ### # ##
## #    #  ##   # #   #   ## #   #   ## ### #   ## ### #   ### ## ## #   #   ##   ### # ##
## #    #  ##   # #   #   ## #   #   ## ### #   ## ### #   ### ## ## #   #   ##   ### # ##
"
            .trim()
        );
        Ok(())
    }

    #[test]
    fn code_128_as_ascii_small_height_double_weight() -> Result<()> {
        let code128 = Code128::new("ÀHELLO")?;
        let ascii = ASCII { height: 7, xdim: 2 };
        let generated = ascii.generate(code128.encode())?;

        assert_eq!(generated,
"
####  ##        ##    ####      ##  ##      ##      ####  ##      ##      ####  ######  ##      ####  ######  ##      ######  ####  ####  ##      ##      ####      ######  ##  ####
####  ##        ##    ####      ##  ##      ##      ####  ##      ##      ####  ######  ##      ####  ######  ##      ######  ####  ####  ##      ##      ####      ######  ##  ####
####  ##        ##    ####      ##  ##      ##      ####  ##      ##      ####  ######  ##      ####  ######  ##      ######  ####  ####  ##      ##      ####      ######  ##  ####
####  ##        ##    ####      ##  ##      ##      ####  ##      ##      ####  ######  ##      ####  ######  ##      ######  ####  ####  ##      ##      ####      ######  ##  ####
####  ##        ##    ####      ##  ##      ##      ####  ##      ##      ####  ######  ##      ####  ######  ##      ######  ####  ####  ##      ##      ####      ######  ##  ####
####  ##        ##    ####      ##  ##      ##      ####  ##      ##      ####  ######  ##      ####  ######  ##      ######  ####  ####  ##      ##      ####      ######  ##  ####
####  ##        ##    ####      ##  ##      ##      ####  ##      ##      ####  ######  ##      ####  ######  ##      ######  ####  ####  ##      ##      ####      ######  ##  ####
".trim());
        Ok(())
    }

    #[test]
    fn ean2_as_ascii() -> Result<()> {
        let ean2 = EANSUPP::new("34")?;
        let ascii = ASCII::new();
        let generated = ascii.generate(ean2.encode())?;

        assert_eq!(
            generated,
            "
# ## #    # # #   ##
# ## #    # # #   ##
# ## #    # # #   ##
# ## #    # # #   ##
# ## #    # # #   ##
# ## #    # # #   ##
# ## #    # # #   ##
# ## #    # # #   ##
# ## #    # # #   ##
# ## #    # # #   ##
"
            .trim()
        );
        Ok(())
    }

    #[test]
    fn ean5_as_ascii() -> Result<()> {
        let ean5 = EANSUPP::new("50799")?;
        let ascii = ASCII::new();
        let generated = ascii.generate(ean5.encode())?;

        assert_eq!(
            generated,
            "
# ## ##   # # #  ### #  #   # #   # ## #   # ##
# ## ##   # # #  ### #  #   # #   # ## #   # ##
# ## ##   # # #  ### #  #   # #   # ## #   # ##
# ## ##   # # #  ### #  #   # #   # ## #   # ##
# ## ##   # # #  ### #  #   # #   # ## #   # ##
# ## ##   # # #  ### #  #   # #   # ## #   # ##
# ## ##   # # #  ### #  #   # #   # ## #   # ##
# ## ##   # # #  ### #  #   # #   # ## #   # ##
# ## ##   # # #  ### #  #   # #   # ## #   # ##
# ## ##   # # #  ### #  #   # #   # ## #   # ##
"
            .trim()
        );
        Ok(())
    }

    #[test]
    fn itf_as_ascii() -> Result<()> {
        let itf = TF::interleaved("12345")?;
        let ascii = ASCII::new();
        let generated = ascii.generate(itf.encode())?;

        assert_eq!(
            generated,
            "
# # ### #   # # ###   ### ### #   # #   ### # ### #   #   ## #
# # ### #   # # ###   ### ### #   # #   ### # ### #   #   ## #
# # ### #   # # ###   ### ### #   # #   ### # ### #   #   ## #
# # ### #   # # ###   ### ### #   # #   ### # ### #   #   ## #
# # ### #   # # ###   ### ### #   # #   ### # ### #   #   ## #
# # ### #   # # ###   ### ### #   # #   ### # ### #   #   ## #
# # ### #   # # ###   ### ### #   # #   ### # ### #   #   ## #
# # ### #   # # ###   ### ### #   # #   ### # ### #   #   ## #
# # ### #   # # ###   ### ### #   # #   ### # ### #   #   ## #
# # ### #   # # ###   ### ### #   # #   ### # ### #   #   ## #
"
            .trim()
        );
        Ok(())
    }

    #[test]
    fn code_93_as_ascii() -> Result<()> {
        let code93 = Code93::new("TEST93")?;
        let ascii = ASCII::new();
        let generated = ascii.generate(code93.encode())?;

        assert_eq!(
            generated,
            "
# # #### ## #  ## ##  #  # ## # ##  ## #  ## #    # # # #    # # ### ## #  #   # # # #### #
# # #### ## #  ## ##  #  # ## # ##  ## #  ## #    # # # #    # # ### ## #  #   # # # #### #
# # #### ## #  ## ##  #  # ## # ##  ## #  ## #    # # # #    # # ### ## #  #   # # # #### #
# # #### ## #  ## ##  #  # ## # ##  ## #  ## #    # # # #    # # ### ## #  #   # # # #### #
# # #### ## #  ## ##  #  # ## # ##  ## #  ## #    # # # #    # # ### ## #  #   # # # #### #
# # #### ## #  ## ##  #  # ## # ##  ## #  ## #    # # # #    # # ### ## #  #   # # # #### #
# # #### ## #  ## ##  #  # ## # ##  ## #  ## #    # # # #    # # ### ## #  #   # # # #### #
# # #### ## #  ## ##  #  # ## # ##  ## #  ## #    # # # #    # # ### ## #  #   # # # #### #
# # #### ## #  ## ##  #  # ## # ##  ## #  ## #    # # # #    # # ### ## #  #   # # # #### #
# # #### ## #  ## ##  #  # ## # ##  ## #  ## #    # # # #    # # ### ## #  #   # # # #### #
"
            .trim()
        );
        Ok(())
    }

    #[test]
    fn code_93_as_ascii_small_height_double_weight() -> Result<()> {
        let code93 = Code93::new("TEST93")?;
        let ascii = ASCII { height: 7, xdim: 2 };
        let generated = ascii.generate(code93.encode())?;

        assert_eq!(generated,
"
##  ##  ########  ####  ##    ####  ####    ##    ##  ####  ##  ####    ####  ##    ####  ##        ##  ##  ##  ##        ##  ##  ######  ####  ##    ##      ##  ##  ##  ########  ##
##  ##  ########  ####  ##    ####  ####    ##    ##  ####  ##  ####    ####  ##    ####  ##        ##  ##  ##  ##        ##  ##  ######  ####  ##    ##      ##  ##  ##  ########  ##
##  ##  ########  ####  ##    ####  ####    ##    ##  ####  ##  ####    ####  ##    ####  ##        ##  ##  ##  ##        ##  ##  ######  ####  ##    ##      ##  ##  ##  ########  ##
##  ##  ########  ####  ##    ####  ####    ##    ##  ####  ##  ####    ####  ##    ####  ##        ##  ##  ##  ##        ##  ##  ######  ####  ##    ##      ##  ##  ##  ########  ##
##  ##  ########  ####  ##    ####  ####    ##    ##  ####  ##  ####    ####  ##    ####  ##        ##  ##  ##  ##        ##  ##  ######  ####  ##    ##      ##  ##  ##  ########  ##
##  ##  ########  ####  ##    ####  ####    ##    ##  ####  ##  ####    ####  ##    ####  ##        ##  ##  ##  ##        ##  ##  ######  ####  ##    ##      ##  ##  ##  ########  ##
##  ##  ########  ####  ##    ####  ####    ##    ##  ####  ##  ####    ####  ##    ####  ##        ##  ##  ##  ##        ##  ##  ######  ####  ##    ##      ##  ##  ##  ########  ##
".trim());
        Ok(())
    }

    #[test]
    fn code_11_as_ascii() -> Result<()> {
        let code11 = Code11::new("12-9")?;
        let ascii = ASCII::new();
        let generated = ascii.generate(code11.encode())?;

        assert_eq!(
            generated,
            "
# ##  # ## # ## #  # ## # ## # ## # # #  ## # # ##  #
# ##  # ## # ## #  # ## # ## # ## # # #  ## # # ##  #
# ##  # ## # ## #  # ## # ## # ## # # #  ## # # ##  #
# ##  # ## # ## #  # ## # ## # ## # # #  ## # # ##  #
# ##  # ## # ## #  # ## # ## # ## # # #  ## # # ##  #
# ##  # ## # ## #  # ## # ## # ## # # #  ## # # ##  #
# ##  # ## # ## #  # ## # ## # ## # # #  ## # # ##  #
# ##  # ## # ## #  # ## # ## # ## # # #  ## # # ##  #
# ##  # ## # ## #  # ## # ## # ## # # #  ## # # ##  #
# ##  # ## # ## #  # ## # ## # ## # # #  ## # # ##  #
"
            .trim()
        );
        Ok(())
    }

    #[test]
    fn rejects_non_binary_encoding() -> Result<()> {
        let generated = ASCII::new().generate([0, 2, 1]);

        assert!(matches!(
            generated,
            Err(Error::InvalidEncoding { index: 1, value: 2 })
        ));
        Ok(())
    }

    #[test]
    fn rejects_empty_encoding() -> Result<()> {
        let generated = ASCII::new().generate([]);

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

    #[test]
    fn rejects_excessive_output_before_allocating() -> Result<()> {
        let maximum = usize::try_from(crate::generators::MAX_OUTPUT_BYTES)
            .map_err(|_| Error::dimension("test sınırı usize aralığına sığmıyor"))?;
        let generator = ASCII {
            height: maximum,
            xdim: 1,
        };

        assert!(matches!(
            generator.generate([0, 1]),
            Err(Error::ResourceLimit {
                resource: "çıktı baytı",
                ..
            })
        ));
        Ok(())
    }
}
