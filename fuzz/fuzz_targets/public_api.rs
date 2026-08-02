#![no_main]

use barcoders::encoding::{Barcode, EncodedBarcode};
use barcoders::error::Result;
use barcoders::generators::{
    ascii::ASCII,
    image::{Color, Image, Rotation},
    json::JSON,
    svg::SVG,
};
use barcoders::sym::{
    codabar::Codabar, code11::Code11, code39::Code39, code93::Code93, code128::Code128,
    ean_supp::EANSUPP, ean8::EAN8, ean13::EAN13, tf::TF, upca::UPCA,
};
use libfuzzer_sys::fuzz_target;

#[derive(Clone, Copy)]
struct RenderOptions {
    height: u32,
    xdim: u32,
    rotation: Rotation,
    foreground: [u8; 4],
    background: [u8; 4],
}

impl RenderOptions {
    fn from_bytes(data: &[u8]) -> Self {
        let byte = |index| data.get(index).copied().unwrap_or_default();
        let color = |offset| {
            [
                byte(offset),
                byte(offset + 1),
                byte(offset + 2),
                byte(offset + 3),
            ]
        };
        let rotation = match byte(2) % 4 {
            0 => Rotation::Zero,
            1 => Rotation::Ninety,
            2 => Rotation::OneEighty,
            _ => Rotation::TwoSeventy,
        };

        Self {
            // Sıfır değerleri hata yollarını, diğer değerler üretim yollarını çalıştırır.
            height: u32::from(byte(0) % 33),
            xdim: u32::from(byte(1) % 9),
            rotation,
            foreground: color(3),
            background: color(7),
        }
    }
}

fn exercise_modules(modules: &[u8], namespace: &str, options: RenderOptions) {
    let ascii = ASCII {
        height: options.height as usize,
        xdim: options.xdim as usize,
    };
    let json = JSON {
        height: options.height as usize,
        xdim: options.xdim as usize,
    };
    let svg = SVG::new(options.height)
        .xdim(options.xdim)
        .foreground(barcoders::generators::svg::Color::new(options.foreground))
        .background(barcoders::generators::svg::Color::new(options.background))
        .xmlns(namespace.to_owned());
    let image = Image::ImageBuffer {
        height: options.height,
        xdim: options.xdim,
        rotation: options.rotation,
        foreground: Color::new(options.foreground),
        background: Color::new(options.background),
    };

    let _ = EncodedBarcode::new(modules).map(|encoded| encoded.runs().count());
    let _ = ascii.generate(modules);
    let _ = json.generate(modules);
    let _ = svg.generate(modules);
    let _ = image.generate_buffer(modules);
}

fn exercise_barcode<T: Barcode>(barcode: Result<T>, namespace: &str, options: RenderOptions) {
    if let Ok(barcode) = barcode {
        exercise_modules(barcode.encoded().modules(), namespace, options);
    }
}

fuzz_target!(|data: &[u8]| {
    let text = String::from_utf8_lossy(data);
    let options = RenderOptions::from_bytes(data);
    let raw_modules: Vec<u8> = data.iter().copied().take(512).collect();
    let binary_modules: Vec<u8> = raw_modules.iter().map(|module| module & 1).collect();

    exercise_modules(&raw_modules, &text, options);
    exercise_modules(&binary_modules, &text, options);

    exercise_barcode(Codabar::new(&text), &text, options);
    exercise_barcode(Code11::new(&text), &text, options);
    exercise_barcode(Code39::new(&text), &text, options);
    exercise_barcode(Code93::new(&text), &text, options);
    exercise_barcode(Code128::new(&text), &text, options);
    exercise_barcode(EANSUPP::new(&text), &text, options);
    exercise_barcode(EAN8::new(&text), &text, options);
    exercise_barcode(EAN13::new(&text), &text, options);
    exercise_barcode(TF::interleaved(&text), &text, options);
    exercise_barcode(TF::standard(&text), &text, options);
    exercise_barcode(UPCA::new(&text), &text, options);
});
