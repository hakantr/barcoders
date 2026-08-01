[![Sürekli entegrasyon](https://github.com/hakantr/barcoders/actions/workflows/ci.yml/badge.svg)](https://github.com/hakantr/barcoders/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/barcoders.svg)](https://crates.io/crates/barcoders)
[![Belgeler](https://docs.rs/barcoders/badge.svg)](https://docs.rs/barcoders)
[![Lisans: MIT veya Apache-2.0](https://img.shields.io/crates/l/barcoders.svg)](#lisans)

![BARCODERS](/media/logo.jpg?raw=true "BARCODERS")

**Barcoders**, Rust programlama dili için bir barkod kodlama kütüphanesidir.

Seçilen barkod sembolojisine uygun veriyi temel ikili yapıyı temsil eden bir `Vec<u8>` değerine
kodlar. Ardından isteğe bağlı yerleşik üreteçlerden biriyle SVG, GIF, PNG, WEBP, JSON veya ASCII
çıktısı alabilir, yerel GPUI ağacı için cihaz pikseline hizalı bir öğe oluşturabilir ya da kendi
üretecinizi yazabilirsiniz.

## Kurulum

Barcoders, Rust 1.95.0 veya daha yeni bir sürüm gerektirir ve Rust 2024 sürümünü kullanır.

Yalnızca kodlama işlevleri için:

```toml
[dependencies]
barcoders = "3.0.0"
```

Belirli çıktı biçimleri üretmek için gerekli özellikleri etkinleştirin:

```toml
[dependencies]
barcoders = { version = "3.0.0", features = ["image", "ascii", "svg", "json"] }
```

Her üreteç isteğe bağlı bir özelliktir; böylece yalnızca kullandığınız işlevleri derlersiniz.

## Belgeler

API belgeleri ve örnekler [docs.rs/barcoders](https://docs.rs/barcoders) adresindedir.

## Güncel destek

Barcoders'ın amacı, yaygın sembolojilerin tümünü ve daha az kullanılanların çoğunu kodlayabilmektir.

### Sembolojiler

- EAN-13
  - UPC-A
  - JAN
  - Bookland
- EAN-8
- Ek EAN barkodları
  - EAN-2
  - EAN-5
- Code11
  - USD-8
- Code39
- Code93
- Code128 (A, B, C)
- 2-of-5
  - Aralıklı (ITF)
  - Standart (STF)
- Codabar

Codabar verisi, sembolojinin gerektirdiği biçimde A, B, C veya D koruma karakterlerinden biriyle
başlayıp bitmelidir; bu karakterler veri bölümünde kullanılamaz.

### Üreteçler

- ASCII (özellik: `ascii`)
- JSON (özellik: `json`)
- SVG (özellik: `svg`)
- PNG (özellik: `image`)
- GIF (özellik: `image`)
- WEBP (özellik: `image`)
- Görüntü tamponu (özellik: `image`)
- Yerel GPUI `canvas` öğesi (özellik: `gpui`)
- Kendi üreteciniz

## Hata ve panik politikası

Kamuya açık kurucular, geçersiz kullanıcı girdisini `Result<T, Error>` ile bildirir. Üreteçler de
geçersiz ikili gösterim, taşan çıktı boyutları ve hedef biçime dönüştürme hataları için yapılandırılmış
bir hata döndürür. Doğrulanmış bir barkodun `encode` işlemi mantıksal olarak hatasız olduğundan
doğrudan `Vec<u8>` döndürür.

Hatalar desteklenmeyen karakterin konumu, beklenen uzunluk aralığı, sağlama basamağı ve geçersiz
boyutun nedeni gibi çağıranın kullanıcı arayüzünde gösterebileceği alanları taşır. Hata enumu yeni
durumların geriye uyumlu eklenebilmesi için `non_exhaustive` olarak tanımlıdır; desen eşlemelerinde
bir genel kol bulundurun.

Geçersiz durumların tip sistemiyle engellenebildiği yerlerde özel alanlar ve doğrulanmış yeni tipler
kullanılır. Geçersiz dış girdi, desteklenmeyen seçenek veya normal çalışma hatası nedeniyle kütüphane
kasıtlı olarak paniklemez. Bellek tükenmesi, yığın taşması, bağımlılık davranışları ya da bozulan bir
iç değişmez gibi süreç düzeyindeki durumlar bu garantinin dışındadır.

Özet yaklaşım: girdiyi sınırda doğrula, geçerli tipe dönüştür, çekirdek işlemleri mümkün olduğunca
hatasız tut.

Üreteçlere verilen ham ikili gösterimler en fazla 100.000 modül, bellek ayıran çıktılar ise en fazla
64 MiB ile sınırlıdır. Bu sınırlar bellek tahsisinden önce denetlenir ve aşıldığında
`Error::ResourceLimit` döndürülür. Boş bir gösterim çizilebilir bir barkod tanımlamadığından
üreteçler tarafından `Error::Length` ile reddedilir.

## Örnekler

### Kodlama

```rust
use barcoders::sym::ean13::EAN13;

fn main() -> barcoders::error::Result<()> {
    // Her kurucu kodlanacak metni doğrular; kurallar barkod türüne göre değişir.
    let barcode = EAN13::new("593456661897")?;

    // `encode`, barkodun ikili gösterimini döndürür ve özel üreteçlerde kullanılabilir.
    let encoded: Vec<u8> = barcode.encode();
    println!("{encoded:?}");
    Ok(())
}
```

### Görüntü üretimi (GIF, WEBP, PNG)

```rust
use barcoders::generators::image::Image;
use barcoders::sym::code39::Code39;
use std::fs::File;
use std::io::{BufWriter, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let barcode = Code39::new("1ISTHELONELIESTNUMBER")?;
    let png = Image::png(80); // Yüksekliği piksel cinsinden belirtin.
    let bytes = png.generate(barcode.encode())?;

    // Kodlanmış görüntüyü diske kaydedin.
    let file = File::create("my_barcode.png")?;
    let mut writer = BufWriter::new(file);
    writer.write_all(&bytes)?;
    Ok(())
}
```

Üretilen dosya:

![Code 39: 1ISTHELONELIESTNUMBER](/media/code39_1istheloneliestnumber.png?raw=true "Code 39: 1ISTHELONELIESTNUMBER")

Görüntüyü kendiniz işlemek için bir `image::RgbaImage` da üretebilirsiniz:

```rust
use barcoders::generators::image::Image;
use barcoders::sym::code39::Code39;

fn main() -> barcoders::error::Result<()> {
    let barcode = Code39::new("BEELZEBUB")?;
    let image = Image::image_buffer(100).generate_buffer(barcode.encode())?;
    println!("{} × {}", image.width(), image.height());
    Ok(())
}
```

X boyutu, döndürme, ön ve arka plan renkleri ile saydamlık enum alanlarından ayarlanabilir:

```rust
use barcoders::generators::image::{Color, Image, Rotation};

let gif = Image::GIF {
    height: 80,
    xdim: 1,
    rotation: Rotation::Zero,
    // Siyah ve beyaz dışındaki renkler çoğu sağlayıcı tarafından önerilmez.
    foreground: Color::new([255, 0, 0, 255]),
    background: Color::new([0, 255, 20, 255]),
};
```

### Yerel GPUI ile çizim

`gpui` özelliği crates.io üzerindeki GPUI paketini kullanmaz. Hem Barcoders hem GPUI depoları
bileşen deposunun kardeşleri olarak bulunmalı ve bağımlılıklar doğrudan yerel kaynaklara
bağlanmalıdır:

```toml
[dependencies]
barcoders = { path = "../barcoders", default-features = false, features = ["gpui"] }
gpui = { path = "../gpui/crates/gpui", default-features = false }
```

`GPUI` üreteci siyah çubukları beyaz zemin üzerinde, her iki yanda varsayılan 12 modüllük sessiz
alanla çizer. Kullanılabilir genişliğe sığan en büyük tam cihaz pikseli modül genişliğini seçer ve
aynı değerdeki ardışık modülleri tek bir dikdörtgen olarak boyar:

```rust
use barcoders::encoding::Barcode;
use barcoders::generators::gpui::GPUI;
use barcoders::sym::code128::Code128;
use gpui::{
    InteractiveElement, IntoElement, ParentElement, Role, StatefulInteractiveElement, div,
};

fn barcode_element(data: &str) -> barcoders::error::Result<impl IntoElement> {
    let barcode = Code128::new(data)?;
    let canvas = GPUI::new(96)?.generate_encoded(barcode.encoded())?;

    Ok(div()
        .id("product-barcode")
        .role(Role::Image)
        .aria_label(format!("Code 128 barkodu: {data}"))
        .child(canvas))
}
```

`EncodedBarcode` iç verisini `Arc` ile paylaştığından klonlanması ucuzdur ve arka plan işlerinde
`Send + Sync + 'static` olarak kullanılabilir. Canlı bir GPUI bileşeninde doğrulama ve kodlama
sonucunu bir `Entity` alanında saklayın; `render` sırasında yalnızca önbellekteki değeri klonlayıp
öğeyi oluşturun. İnsan tarafından okunabilir veriyi GPUI metni olarak ayrıca göstermek ve barkodu
durumlu bir `div` içinde erişilebilir etiketle sunmak bileşenin sorumluluğundadır.

### SVG üretimi

SVG ayrı bir özelliktir ve üçüncü taraf bağımlılık gerektirmez:

```rust
use barcoders::generators::svg::SVG;
use barcoders::sym::code39::Code39;
use std::fs::File;
use std::io::{BufWriter, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let barcode = Code39::new("56DFU4A777H")?;
    let data = SVG::new(200).generate(barcode.encode())?;

    let file = File::create("my_barcode.svg")?;
    let mut writer = BufWriter::new(file);
    writer.write_all(data.as_bytes())?;
    Ok(())
}
```

### ASCII üretimi

ASCII üreteci testler ve hızlı görsel denetimler için kullanışlıdır:

```rust
use barcoders::generators::ascii::ASCII;
use barcoders::sym::ean13::EAN13;

fn main() -> barcoders::error::Result<()> {
    let barcode = EAN13::new("750103131130")?;
    let generated = ASCII::new().generate(barcode.encode())?;
    println!("{generated}");
    Ok(())
}
```

### JSON üretimi

JSON çıktısı, kodlanmış veriyi standart bir biçimde üçüncü taraf sistemlere aktarmakta kullanışlıdır:

```rust
use barcoders::generators::json::JSON;
use barcoders::sym::codabar::Codabar;

fn main() -> barcoders::error::Result<()> {
    let barcode = Codabar::new("A98B")?;
    let generated = JSON::new().generate(barcode.encode())?;
    println!("{generated}");
    Ok(())
}
```

## Testler

Tam özellik kümesi:

```console
cargo test --all-features --all-targets
```

Yalnızca kodlama ve varsayılan özellikler:

```console
cargo test
```

Görsel doğrulama amacıyla gerçek görüntü veya SVG dosyaları yazdırmak için ilgili test modülündeki
`WRITE_TO_FILE` değerini etkinleştirin.

## Lisans

Bu proje tercihinize bağlı olarak aşağıdaki lisanslardan biriyle kullanılabilir:

- [Apache Lisansı, Sürüm 2.0](LICENSE-APACHE)
- [MIT Lisansı](LICENSE-MIT)

### Katkı

Aksini açıkça belirtmediğiniz sürece, Apache-2.0 lisansında tanımlandığı biçimiyle projeye dahil
edilmek üzere bilerek gönderdiğiniz her katkı, ek koşul olmaksızın yukarıdaki iki lisans kapsamında
lisanslanır.
