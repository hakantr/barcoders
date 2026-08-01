//! Üreteçlerden bağımsız, doğrulanmış barkod gösterimi.

use crate::error::{Error, Result};
#[cfg(not(feature = "std"))]
use alloc::{sync::Arc, vec::Vec};
#[cfg(feature = "std")]
use std::{sync::Arc, vec::Vec};

/// Yalnızca ikili modüllerden oluşan, ucuz klonlanabilen barkod gösterimi.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EncodedBarcode {
    modules: Arc<[u8]>,
}

impl EncodedBarcode {
    /// Dışarıdan sağlanan modülleri doğrulayarak yeni bir barkod gösterimi oluşturur.
    pub fn new<T: AsRef<[u8]>>(modules: T) -> Result<Self> {
        let modules = modules.as_ref();

        validate_modules(modules)?;

        Ok(Self {
            modules: Arc::from(modules),
        })
    }

    pub(crate) fn from_validated(modules: Vec<u8>) -> Self {
        Self {
            modules: Arc::from(modules),
        }
    }

    /// Barkodun ikili modüllerini döndürür.
    pub fn modules(&self) -> &[u8] {
        self.modules.as_ref()
    }

    /// Barkodun modül sayısını döndürür.
    pub fn len(&self) -> usize {
        self.modules.len()
    }

    /// Barkod gösteriminin modül içermediğini bildirir.
    pub fn is_empty(&self) -> bool {
        self.modules.is_empty()
    }

    /// Aynı değere sahip ardışık modülleri yineleyen bir iteratör döndürür.
    pub fn runs(&self) -> ModuleRuns<'_> {
        ModuleRuns {
            modules: self.modules(),
            offset: 0,
        }
    }
}

pub(crate) const MAX_MODULES: usize = 100_000;

pub(crate) fn validate_modules(modules: &[u8]) -> Result<()> {
    if modules.is_empty() {
        return Err(Error::length(1, None, 0));
    }

    let requested = u64::try_from(modules.len()).unwrap_or(u64::MAX);
    let maximum = u64::try_from(MAX_MODULES).unwrap_or(u64::MAX);

    if modules.len() > MAX_MODULES {
        return Err(Error::ResourceLimit {
            resource: "modül sayısı",
            requested,
            maximum,
        });
    }

    match modules
        .iter()
        .copied()
        .enumerate()
        .find(|(_, module)| !matches!(module, 0 | 1))
    {
        Some((index, value)) => Err(Error::InvalidEncoding { index, value }),
        None => Ok(()),
    }
}

impl AsRef<[u8]> for EncodedBarcode {
    fn as_ref(&self) -> &[u8] {
        self.modules()
    }
}

impl TryFrom<Vec<u8>> for EncodedBarcode {
    type Error = Error;

    fn try_from(modules: Vec<u8>) -> Result<Self> {
        Self::new(modules)
    }
}

impl TryFrom<&[u8]> for EncodedBarcode {
    type Error = Error;

    fn try_from(modules: &[u8]) -> Result<Self> {
        Self::new(modules)
    }
}

/// Aynı değere sahip ardışık barkod modülleri.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ModuleRun {
    module: u8,
    offset: usize,
    len: usize,
}

impl ModuleRun {
    /// Ardışık bölümün ikili modül değerini döndürür.
    pub fn module(self) -> u8 {
        self.module
    }

    /// Ardışık bölümün barkod içindeki başlangıç konumunu döndürür.
    pub fn offset(self) -> usize {
        self.offset
    }

    /// Ardışık bölümdeki modül sayısını döndürür.
    pub fn len(self) -> usize {
        self.len
    }

    /// Ardışık bölümün modül içermediğini bildirir.
    pub fn is_empty(self) -> bool {
        self.len == 0
    }

    /// Ardışık bölümün çizilecek bir çubuğu temsil ettiğini bildirir.
    pub fn is_bar(self) -> bool {
        self.module == 1
    }
}

/// Barkod modüllerini aynı değere sahip ardışık bölümler halinde yineleyen iteratör.
#[derive(Clone, Debug)]
pub struct ModuleRuns<'a> {
    modules: &'a [u8],
    offset: usize,
}

impl Iterator for ModuleRuns<'_> {
    type Item = ModuleRun;

    fn next(&mut self) -> Option<Self::Item> {
        let remaining = self.modules.get(self.offset..)?;
        let module = remaining.first().copied()?;
        let len = remaining
            .iter()
            .take_while(|candidate| **candidate == module)
            .count();
        let run = ModuleRun {
            module,
            offset: self.offset,
            len,
        };
        self.offset = self.offset.saturating_add(len);
        Some(run)
    }
}

/// Doğrulanmış bir sembolojiyi üreteçlerden bağımsız gösterime dönüştürür.
pub trait Barcode {
    /// Barkodun doğrulanmış ikili gösterimini döndürür.
    fn encoded(&self) -> EncodedBarcode;
}

macro_rules! impl_barcode {
    ($barcode:ty) => {
        impl Barcode for $barcode {
            fn encoded(&self) -> EncodedBarcode {
                EncodedBarcode::from_validated(self.encode())
            }
        }
    };
}

impl_barcode!(crate::sym::codabar::Codabar);
impl_barcode!(crate::sym::code11::Code11);
impl_barcode!(crate::sym::code128::Code128);
impl_barcode!(crate::sym::code39::Code39);
impl_barcode!(crate::sym::code93::Code93);
impl_barcode!(crate::sym::ean_supp::EANSUPP);
impl_barcode!(crate::sym::ean13::EAN13);
impl_barcode!(crate::sym::ean8::EAN8);
impl_barcode!(crate::sym::tf::TF);
impl_barcode!(crate::sym::upca::UPCA);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sym::ean13::EAN13;

    #[test]
    fn invalid_modules_are_rejected() {
        assert!(matches!(
            EncodedBarcode::new([1, 0, 2, 1]),
            Err(Error::InvalidEncoding { index: 2, value: 2 })
        ));
    }

    #[test]
    fn empty_encoding_is_rejected() {
        assert!(matches!(
            EncodedBarcode::new([]),
            Err(Error::Length {
                min: 1,
                max: None,
                found: 0
            })
        ));
    }

    #[test]
    fn runs_are_grouped_without_losing_offsets() -> Result<()> {
        let encoded = EncodedBarcode::new([1, 1, 0, 0, 0, 1])?;
        let runs: Vec<ModuleRun> = encoded.runs().collect();

        assert_eq!(
            runs.as_slice(),
            &[
                ModuleRun {
                    module: 1,
                    offset: 0,
                    len: 2,
                },
                ModuleRun {
                    module: 0,
                    offset: 2,
                    len: 3,
                },
                ModuleRun {
                    module: 1,
                    offset: 5,
                    len: 1,
                },
            ]
        );
        Ok(())
    }

    #[test]
    fn validated_symbology_produces_shared_encoding() -> Result<()> {
        let barcode = EAN13::new("750103131130")?;
        let first = barcode.encoded();
        let second = first.clone();

        assert_eq!(first.modules(), barcode.encode());
        assert_eq!(first, second);
        assert!(Arc::ptr_eq(&first.modules, &second.modules));
        Ok(())
    }

    #[test]
    fn excessive_module_count_is_rejected_before_rendering() {
        let modules: Vec<u8> = core::iter::repeat_n(0, MAX_MODULES.saturating_add(1)).collect();

        assert!(matches!(
            EncodedBarcode::new(modules),
            Err(Error::ResourceLimit {
                resource: "modül sayısı",
                ..
            })
        ));
    }

    #[test]
    fn shared_encoding_is_safe_for_background_work() {
        fn assert_send_sync_static<T: Send + Sync + 'static>() {}

        assert_send_sync_static::<EncodedBarcode>();
    }
}
