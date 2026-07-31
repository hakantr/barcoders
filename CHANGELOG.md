# Değişiklik günlüğü

Bu proje anlamsal sürümlemeyi izler.

Kayıt türleri:

- `[eklendi]`: Yeni özellikler.
- `[değişti]`: Mevcut işlevlerdeki değişiklikler.
- `[kullanımdan-kalkıyor]`: Gelecek sürümlerde kaldırılacak, daha önce kararlı özellikler.
- `[kaldırıldı]`: Bu sürümde kaldırılan, kullanımdan kalkmış özellikler.
- `[düzeltildi]`: Hata düzeltmeleri.
- `[güvenlik]`: Güvenlik açığı nedeniyle kullanıcıları yükseltmeye çağıran değişiklikler.

### Yayımlanmadı

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
