//! Yerel GPUI kaynak ağacı için cihaz pikseline hizalı barkod üretimi.

use crate::encoding::EncodedBarcode;
use crate::error::{Error, Result};
use ::gpui::{
    Bounds, Hsla, IntoElement, Pixels, Styled, black, canvas, fill, point, px, size, white,
};

/// GPUI `canvas` öğesi üreten barkod yapılandırması.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GPUI {
    height: u32,
    quiet_zone: u32,
    foreground: Hsla,
    background: Hsla,
}

impl GPUI {
    /// Her iki yanda kullanılan varsayılan sessiz alanın modül sayısı.
    pub const DEFAULT_QUIET_ZONE: u32 = 12;

    /// Verilen mantıksal piksel yüksekliğiyle yeni bir GPUI üreteci oluşturur.
    pub fn new(height: u32) -> Result<Self> {
        if height == 0 {
            return Err(Error::dimension("GPUI barkod yüksekliği sıfır olamaz"));
        }

        Ok(Self {
            height,
            quiet_zone: Self::DEFAULT_QUIET_ZONE,
            foreground: black(),
            background: white(),
        })
    }

    /// Barkodun her iki yanındaki sessiz alanı modül cinsinden ayarlar.
    pub fn quiet_zone(mut self, modules: u32) -> Self {
        self.quiet_zone = modules;
        self
    }

    /// Çubuk rengini ayarlar.
    pub fn foreground(mut self, color: Hsla) -> Self {
        self.foreground = color;
        self
    }

    /// Arka plan rengini ayarlar.
    pub fn background(mut self, color: Hsla) -> Self {
        self.background = color;
        self
    }

    /// Yapılandırılmış yüksekliği mantıksal piksel cinsinden döndürür.
    pub fn height(self) -> u32 {
        self.height
    }

    /// Her iki yanda kullanılacak sessiz alanın modül sayısını döndürür.
    pub fn quiet_zone_modules(self) -> u32 {
        self.quiet_zone
    }

    /// Ham ikili gösterimi doğrulayıp GPUI barkod öğesi üretir.
    pub fn generate<T: AsRef<[u8]>>(self, barcode: T) -> Result<impl IntoElement + Styled> {
        self.generate_encoded(EncodedBarcode::new(barcode)?)
    }

    /// Önceden doğrulanmış gösterimden GPUI barkod öğesi üretir.
    pub fn generate_encoded(self, barcode: EncodedBarcode) -> Result<impl IntoElement + Styled> {
        let quiet_zone = usize::try_from(self.quiet_zone)
            .map_err(|_| Error::dimension("GPUI sessiz alanı usize aralığını aşıyor"))?;
        let quiet_modules = quiet_zone
            .checked_mul(2)
            .ok_or_else(|| Error::dimension("GPUI sessiz alan toplamı usize aralığını aşıyor"))?;
        let total_modules = barcode
            .len()
            .checked_add(quiet_modules)
            .ok_or_else(|| Error::dimension("GPUI toplam modül sayısı usize aralığını aşıyor"))?;
        let background = self.background;
        let foreground = self.foreground;

        Ok(canvas(
            move |bounds, window, _cx| {
                calculate_layout(bounds, total_modules, quiet_zone, window.scale_factor())
            },
            move |bounds, layout, window, _cx| {
                window.paint_quad(fill(bounds, background));

                let Some(layout) = layout else {
                    return;
                };

                for run in barcode.runs().filter(|run| run.is_bar()) {
                    let x = layout.first_module_x + layout.module_width * run.offset();
                    let width = layout.module_width * run.len();
                    let bar_bounds =
                        Bounds::new(point(x, bounds.origin.y), size(width, bounds.size.height));
                    window.paint_quad(fill(bar_bounds, foreground));
                }
            },
        )
        .w_full()
        .min_w(px(total_modules as f32))
        .h(px(self.height as f32)))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct PaintLayout {
    first_module_x: Pixels,
    module_width: Pixels,
}

fn calculate_layout(
    bounds: Bounds<Pixels>,
    total_modules: usize,
    quiet_zone: usize,
    scale_factor: f32,
) -> Option<PaintLayout> {
    if total_modules == 0 || !scale_factor.is_finite() || scale_factor <= 0.0 {
        return None;
    }

    let available_device_width = (bounds.size.width.as_f32() * scale_factor).floor();
    let module_device_width = (available_device_width / total_modules as f32).floor();

    if !module_device_width.is_finite() || module_device_width < 1.0 {
        return None;
    }

    let content_device_width = module_device_width * total_modules as f32;
    let leading_device_width = ((available_device_width - content_device_width) / 2.0).floor();
    let module_width = px(module_device_width / scale_factor);
    let first_module_offset =
        (leading_device_width + module_device_width * quiet_zone as f32) / scale_factor;

    Some(PaintLayout {
        first_module_x: px(bounds.origin.x.as_f32() + first_module_offset),
        module_width,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_height_is_rejected() {
        assert!(matches!(GPUI::new(0), Err(Error::Dimension { .. })));
    }

    #[test]
    fn invalid_encoding_is_rejected() -> Result<()> {
        let generator = GPUI::new(80)?;
        assert!(generator.generate([1, 0, 3, 1]).is_err());
        Ok(())
    }

    #[test]
    fn module_width_uses_whole_device_pixels() -> Result<()> {
        let bounds = Bounds::new(point(px(0.0), px(0.0)), size(px(120.0), px(80.0)));
        let Some(layout) = calculate_layout(bounds, 30, 5, 1.25) else {
            return Err(Error::dimension("GPUI test yerleşimi üretilemedi"));
        };
        let device_width = layout.module_width.as_f32() * 1.25;

        assert_eq!(device_width.fract(), 0.0);
        assert_eq!(device_width, 5.0);
        Ok(())
    }

    #[test]
    fn layout_rejects_width_smaller_than_module_count() {
        let bounds = Bounds::new(point(px(0.0), px(0.0)), size(px(10.0), px(80.0)));

        assert_eq!(calculate_layout(bounds, 20, 5, 1.0), None);
    }
}
