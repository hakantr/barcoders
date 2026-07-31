# Değişiklik günlüğü

Bu proje anlamsal sürümlemeyi izler.

Kayıt türleri:

- `[eklendi]`: Yeni özellikler.
- `[değişti]`: Mevcut işlevlerdeki değişiklikler.
- `[kullanımdan-kalkıyor]`: Gelecek sürümlerde kaldırılacak, daha önce kararlı özellikler.
- `[kaldırıldı]`: Bu sürümde kaldırılan, kullanımdan kalkmış özellikler.
- `[düzeltildi]`: Hata düzeltmeleri.
- `[güvenlik]`: Güvenlik açığı nedeniyle kullanıcıları yükseltmeye çağıran değişiklikler.

### v3.0.0 (Yayımlanmadı)

- [eklendi] Yalnızca `../gpui/crates/gpui` yerel kaynağına bağlanan `gpui` özelliği ve cihaz
  pikseline hizalı `canvas` üreteci eklendi.
- [eklendi] Ucuz klonlanan `EncodedBarcode`, ortak `Barcode` trait'i ve ardışık modül iteratörü
  eklendi.
- [değişti] `Error` varyantları karakter konumu, uzunluk aralığı, sağlama değeri ve boyut nedeni
  gibi yapılandırılmış bağlam alanları taşıyor ve enum `non_exhaustive` olarak işaretlendi.
- [değişti] Kamuya açık barkod tipleri GPUI durum karşılaştırmaları için `Clone`, `PartialEq` ve
  `Eq` uyguluyor.
- [eklendi] Ham kodlamalar 100.000 modül, bellek ayıran üreteçler 64 MiB ile sınırlandı; sınırlar
  tahsisten önce `Error::ResourceLimit` ile bildiriliyor.
- [değişti] `image` ve `gpui` özellikleri ihtiyaç duydukları `std` özelliğini kendileri
  etkinleştiriyor.
- [değişti] CI, `hakantr/gpui` deposunun `981b10eb6da5621c3ba0b456dba82609da1ab550`
  revizyonunu kardeş dizine alıyor ve GPUI'nin registry yerine path kaynağından geldiğini denetliyor.
- [değişti] Yerel GPUI'nin `stacksafe` bağımlılığı 1.0.3'e yükseltildi; böylece
  `proc-macro-error2` gelecek-uyumluluk uyarısı bağımlılık ağacından çıkarıldı.

- [değişti] Crate, Rust 2024 sürümüne geçirildi; araç zinciri ve desteklenen en eski Rust sürümü
  1.95.0 olarak sabitlendi.
- [kaldırıldı] 1.95.0'dan eski Rust araç zincirleri için destek kaldırıldı.
- [değişti] PNG testleri artık değişken sıkıştırılmış bayt uzunlukları yerine çözülmüş boyutları ve
  pikselleri doğruluyor.
- [değişti] Kilit dosyasındaki bağımlılıklar Rust 1.95 ile uyumlu en güncel sürümlere yükseltildi.
- [değişti] Geçersiz dış girdiler ve çıktı boyutu taşmaları yapılandırılmış Türkçe hatalarla
  bildiriliyor; doğrulanmış çekirdek kodlama işlemleri doğrudan değer döndürüyor.
- [değişti] Ek EAN ve 2-of-5 verileri, geçersiz iç durumları engelleyen özel alanlı doğrulanmış
  tiplerle temsil ediliyor.
- [eklendi] Dizinleme, `unwrap`, `expect`, açık panik ve süreçten çıkış gibi panik riski taşıyan
  kullanımları reddeden Clippy kuralları eklendi.
- [değişti] Kaynak açıklamaları, örnekler, hata iletileri ve kullanıcı belgeleri Türkçeleştirildi.
- [değişti] CI kapsamı Rust 1.95, güncel kararlı Rust, Linux, macOS, Windows, özellik birleşimleri,
  rustfmt, Clippy ve rustdoc denetimlerini içerecek biçimde genişletildi.
- [eklendi] RustSec, GitHub bağımlılık incelemesi ve haftalık GitHub Actions Dependabot
  güncellemeleri eklendi. Cargo bağımlılıkları, harici GPUI path kaynağını da içeren RustSec
  kilit dosyası üzerinden denetleniyor.

#### 2.x sürümünden geçiş

- `Error::Character`, `Error::Length`, `Error::Checksum`, `Error::Generate`,
  `Error::InvalidEncoding` ve `Error::Dimension` desenlerinde yeni alanları `..` ile karşılayın.
- `Error` için kapsamlı eşleşmelerde, gelecekte eklenecek hata durumları için genel bir kol ekleyin.
- `EANSUPP` ve `TF` varyantlarını doğrudan ham `Vec<u8>` ile kurmak yerine kamuya açık kurucuları
  kullanın.
- GPUI bileşenlerinde `encode()` sonucunu her render çağrısında üretmek yerine `Barcode::encoded()`
  sonucunu durum içinde saklayın.

### v2.0.0 (2024-04-04)

- [değişti] `image` bağımlılığı 0.22.0 sürümünden 0.25.0 sürümüne yükseltildi.
- [kaldırıldı] Image 0.25 RGBA biçimini desteklemediği için görüntü üretiminden JPEG desteği
  kaldırıldı.
- [değişti] Varsayılan özellikler artık `["ascii", "json", "svg", "std"]`.
- [eklendi] Görüntü üretimine WEBP desteği eklendi.
- [düzeltildi] EAN13 barkodlarında sağlama basamağının verilmesi güvence altına alındı.
- [eklendi] Üretilen SVG'ye XML ad alanı ekleme desteği eklendi.
- [eklendi] `no_std` desteği eklendi.

### v1.0.2 (2020-09-09)

- [düzeltildi] Code128 ikili eşlemelerinde FS, `|` ve 92 karakterlerine ilişkin yazım hatası
  düzeltildi.

### v1.0.1 (2019-12-03)

- [düzeltildi] Çeşitli lint sorunları giderildi.

### v1.0.0 (2019-12-03)

- [değişti] `image` bağımlılığı 0.18.0 sürümünden 0.22.0 sürümüne yükseltildi.

### v0.10.0 (2018-02-28)

- [eklendi] Code128 sembolojisine FNC1, FNC2, FNC3, FNC4 ve SHIFT üst karakter desteği eklendi.
- [değişti] `image` bağımlılığı 0.16.0 sürümünden 0.18.0 sürümüne yükseltildi.

### v0.9.0 (2017-10-30)

- [eklendi] Code11 barkod kodlayıcısı eklendi.
- [kaldırıldı] Tüm kodlayıcıların genel API'lerinden sağlama işlevleri kaldırıldı; bu geriye uyumsuz
  bir değişikliktir.
- [değişti] `image` bağımlılığı 0.15.0 sürümünden 0.16.0 sürümüne yükseltildi.
- [değişti] `clippy` bağımlılığı 0.0.134 sürümünden 0.0.166 sürümüne yükseltildi.
- [değişti] Çeşitli iç yeniden düzenlemeler yapıldı.

### v0.8.2 (2017-09-03)

- [eklendi] Code93 barkod kodlayıcısı eklendi.
- [değişti] Bazı sabitlerdeki gereksiz niteleyiciler kaldırıldı.
- [değişti] `image` bağımlılığı 0.13.0 sürümünden 0.15.0 sürümüne yükseltildi.

### v0.8.1 (2017-06-20)

- [değişti] SVG kurucusu artık `height` bağımsız değişkenini zorunlu tutuyor.

### v0.8.0 (2017-06-12)

- [değişti] `Image` enum varyantlarının kurucuları artık `height` bağımsız değişkenini zorunlu
  tutuyor.
- [değişti] Görüntü üreteci, RGBA niteliklerini belirtmek için `background` ve `foreground`
  alanlarını kabul ediyor.
- [eklendi] `generators::image::*` modülüne `image::ImageBuffer<Rgba<u8>, Vec<u8>>` döndüren
  `generate_buffer` metodu eklendi.
- [değişti] `clippy` bağımlılığı 0.0.83 sürümünden 0.0.134 sürümüne yükseltildi.
- [değişti] `image` bağımlılığı 0.10.3 sürümünden 0.13.0 sürümüne yükseltildi.
- [kaldırıldı] Sabitlerdeki açık statik yaşam süresi belirteçleri kaldırıldı (Rust 1.17'de
  uygulandığı biçimiyle).
- [değişti] SVG üretiminde 88 bayt tasarruf edildi.
- [değişti] Desen eşleme ifadeleri `and_then` birleştiricisini kullanacak biçimde düzenlendi.

### v0.7.0 (2017-02-11)

- [eklendi] Basit ama kodlanmış veriyi üçüncü taraflara aktarmakta kullanışlı JSON üreteci eklendi.
- [düzeltildi] ASCII üretecindeki eski içe aktarımlar kaldırıldı.
- [değişti] README kullanım belgeleri güncellendi.

### v0.6.0 (2016-12-09)

- [değişti] `try!()` makroları, Rust 1.13'te kararlı hâle gelen `?` işleciyle değiştirildi.
- [değişti] README kullanım belgeleri güncellendi.

### v0.5.1 (2016-08-18)

- [değişti] Çözümleme sırasında sahipli `String` kullanımından kaçınıldı.
- [değişti] Bağımlılıklar güncellendi.
- [düzeltildi] README kullanım belgeleri düzeltildi.

### v0.5.0 (2016-02-04)

- [eklendi] Codabar semboloji kodlayıcısı eklendi.
- [kaldırıldı] Tüm kodlayıcılardan `raw_data` metodu kaldırıldı.

### v0.4.0 (2016-01-30)

- [eklendi] Code128 semboloji kodlayıcısı eklendi.

### v0.3.6 (2016-01-04)

- [değişti] Proje çift MIT/Apache lisansına geçirildi.
- [değişti] Bağımlılıklar en güncel kararlı alt sürümlere yükseltildi.

### v0.3.5 (2015-12-03)

- [eklendi] PNG, GIF ve JPEG görüntü üreteçlerine döndürme desteği eklendi.
- [eklendi] Üreteçler için hata türü eklendi.

### v0.3.4 (2015-11-30)

- [eklendi] Tüm kodlayıcılar için hata türleri eklendi.

### v0.3.3 (2015-11-28)

- [eklendi] SVG üreteci eklendi.
