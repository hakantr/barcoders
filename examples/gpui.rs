//! Yerel GPUI kaynak ağacıyla erişilebilir barkod öğesi oluşturma örneği.

use barcoders::encoding::Barcode;
use barcoders::generators::gpui::GPUI;
use barcoders::sym::code128::Code128;
use gpui::{InteractiveElement, IntoElement, ParentElement, Role, StatefulInteractiveElement, div};

fn barcode_element(data: &str) -> barcoders::error::Result<impl IntoElement> {
    let barcode = Code128::new(data)?;
    let canvas = GPUI::new(96)?.generate_encoded(barcode.encoded())?;
    let label = format!("Code 128 barkodu: {data}");

    Ok(div()
        .id("product-barcode")
        .role(Role::Image)
        .aria_label(label)
        .child(canvas))
}

fn main() -> barcoders::error::Result<()> {
    // Gerçek bileşende bu öğeyi `Render::render` sonucuna ekleyin ve `EncodedBarcode` değerini
    // yalnız veri değiştiğinde yenileyen bir `Entity` alanında saklayın.
    let _element = barcode_element("Ć12345678")?;
    Ok(())
}
