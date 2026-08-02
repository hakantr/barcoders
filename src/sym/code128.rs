//! Code128 barkodlarını kodlayan bileşen.
//!
//! Code128; üç ayrı karakter kümesi kullanarak alfasayısal verileri ve birçok özel karakteri
//! kodlayabilen, yaygın ve yüksek yoğunluklu bir sembolojidir.
//!
//! Code128 ayrıca basamakları çift yoğunlukta kodlayabilir.
//!
//! ## Karakter kümeleri
//!
//! Barcoders, barkodda kullanılacak karakter kümelerini belirtmek için özel bir Unicode söz dizimi
//! sunar:
//!
//! <ul><li>\u{00C0} = A karakter kümesine geç (À)</li>
//! <li>\u{0181} = B karakter kümesine geç (Ɓ)</li>
//! <li>\u{0106} = C karakter kümesine geç (Ć)</li></ul>
//!
//! Başlangıç karakter kümesini ve veri içindeki tüm küme değişikliklerini belirtmeniz gerekir. Bu
//! nedenle bütün Code128 barkodları "À", "Ɓ" veya "Ć" ile başlamalıdır. Basit alfasayısal verilerde
//! genellikle yalnızca A karakter kümesi kullanılabilir.
//!
//! Örneğin bu barkod B karakter kümesini kullanır:
//!
//! <ul><li>\u{0181}HE1234A*1</li></ul>
//!
//! Diğer gösterimi:
//!
//! <ul><li>ƁHE1234A*1</li></ul>
//!
//! Bu örnek ise varsayılan A karakter kümesiyle başlar, ardından basamakları daha verimli kodlamak
//! için C kümesine geçer:
//!
//! <ul><li>\u{00C0}HE@$A\u{0106}123456</li></ul>
//!
//! Diğer gösterimi:
//!
//! <ul><li>ÀHE@$AĆ123456</li></ul>
//!
//! ## Unicode karakterleri
//!
//! A karakter kümesindeki görünmez Unicode karakterleri, Unicode dizileriyle gösterilmelidir.
//! Örneğin `ACK` karakterini göstermek için:
//!
//! <ul><li>À\u{0006}</li></ul>
//!
//! ## Özel amaçlı işlev karakterleri (FNC1 - 4)
//!
//! İşlev dizileri aşağıdaki Unicode karakterleriyle gösterilebilir:
//!
//! - FNC1: ```Ź``` (```\u{0179}```)
//! - FNC2: ```ź``` (```\u{017A}```)
//! - FNC3: ```Ż``` (```\u{017B}```)
//! - FNC4: ```ż``` (```\u{017C}```)
//! - SHIFT: ```Ž``` (```\u{017D}```)
//!
//! ## SHIFT
//!
//! ```Ž``` yalnız A ve B kümelerinde geçerlidir ve hemen sonraki tek karakteri diğer kümede
//! (A ⇄ B) kodlar; sonrasında etkin küme değişmeden sürer. Örneğin ```À``` ile başlayan bir
//! barkodda ```Že```, yalnız küçük ```e``` karakterini B kümesinde kodlar. SHIFT'in son karakter
//! olması, art arda gelmesi, C kümesinde kullanılması veya hedef kümede bulunmayan bir karakterle
//! izlenmesi `Error::Character` döndürür.

use crate::error::*;
use crate::sym::helpers;
#[cfg(not(feature = "std"))]
use alloc::{format, string::ToString};
use helpers::{Vec, vec};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Unit {
    kind: UnitKind,
    index: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UnitKind {
    A,
    B,
    C,
}

type Encoding = [u8; 11];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CharacterSet {
    A,
    B,
    C,
    None,
}

// Her karakter kümesinde izin verilen karakterler için karakter -> ikili değer eşlemeleri.
const CHARS: [([&str; 3], Encoding); 106] = [
    ([" ", " ", "00"], [1, 1, 0, 1, 1, 0, 0, 1, 1, 0, 0]),
    (["!", "!", "01"], [1, 1, 0, 0, 1, 1, 0, 1, 1, 0, 0]),
    (["\"", "\"", "02"], [1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0]),
    (["#", "#", "03"], [1, 0, 0, 1, 0, 0, 1, 1, 0, 0, 0]),
    (["$", "$", "04"], [1, 0, 0, 1, 0, 0, 0, 1, 1, 0, 0]),
    (["%", "%", "05"], [1, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0]),
    (["&", "&", "06"], [1, 0, 0, 1, 1, 0, 0, 1, 0, 0, 0]),
    (["'", "'", "07"], [1, 0, 0, 1, 1, 0, 0, 0, 1, 0, 0]),
    (["(", "(", "08"], [1, 0, 0, 0, 1, 1, 0, 0, 1, 0, 0]),
    ([")", ")", "09"], [1, 1, 0, 0, 1, 0, 0, 1, 0, 0, 0]),
    (["*", "*", "10"], [1, 1, 0, 0, 1, 0, 0, 0, 1, 0, 0]),
    (["+", "+", "11"], [1, 1, 0, 0, 0, 1, 0, 0, 1, 0, 0]),
    ([",", ",", "12"], [1, 0, 1, 1, 0, 0, 1, 1, 1, 0, 0]),
    (["-", "-", "13"], [1, 0, 0, 1, 1, 0, 1, 1, 1, 0, 0]),
    ([".", ".", "14"], [1, 0, 0, 1, 1, 0, 0, 1, 1, 1, 0]),
    (["/", "/", "15"], [1, 0, 1, 1, 1, 0, 0, 1, 1, 0, 0]),
    (["0", "0", "16"], [1, 0, 0, 1, 1, 1, 0, 1, 1, 0, 0]),
    (["1", "1", "17"], [1, 0, 0, 1, 1, 1, 0, 0, 1, 1, 0]),
    (["2", "2", "18"], [1, 1, 0, 0, 1, 1, 1, 0, 0, 1, 0]),
    (["3", "3", "19"], [1, 1, 0, 0, 1, 0, 1, 1, 1, 0, 0]),
    (["4", "4", "20"], [1, 1, 0, 0, 1, 0, 0, 1, 1, 1, 0]),
    (["5", "5", "21"], [1, 1, 0, 1, 1, 1, 0, 0, 1, 0, 0]),
    (["6", "6", "22"], [1, 1, 0, 0, 1, 1, 1, 0, 1, 0, 0]),
    (["7", "7", "23"], [1, 1, 1, 0, 1, 1, 0, 1, 1, 1, 0]),
    (["8", "8", "24"], [1, 1, 1, 0, 1, 0, 0, 1, 1, 0, 0]),
    (["9", "9", "25"], [1, 1, 1, 0, 0, 1, 0, 1, 1, 0, 0]),
    ([":", ":", "26"], [1, 1, 1, 0, 0, 1, 0, 0, 1, 1, 0]),
    ([";", ";", "27"], [1, 1, 1, 0, 1, 1, 0, 0, 1, 0, 0]),
    (["<", "<", "28"], [1, 1, 1, 0, 0, 1, 1, 0, 1, 0, 0]),
    (["=", "=", "29"], [1, 1, 1, 0, 0, 1, 1, 0, 0, 1, 0]),
    ([">", ">", "30"], [1, 1, 0, 1, 1, 0, 1, 1, 0, 0, 0]),
    (["?", "?", "31"], [1, 1, 0, 1, 1, 0, 0, 0, 1, 1, 0]),
    (["@", "@", "32"], [1, 1, 0, 0, 0, 1, 1, 0, 1, 1, 0]),
    (["A", "A", "33"], [1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0]),
    (["B", "B", "34"], [1, 0, 0, 0, 1, 0, 1, 1, 0, 0, 0]),
    (["C", "C", "35"], [1, 0, 0, 0, 1, 0, 0, 0, 1, 1, 0]),
    (["D", "D", "36"], [1, 0, 1, 1, 0, 0, 0, 1, 0, 0, 0]),
    (["E", "E", "37"], [1, 0, 0, 0, 1, 1, 0, 1, 0, 0, 0]),
    (["F", "F", "38"], [1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0]),
    (["G", "G", "39"], [1, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0]),
    (["H", "H", "40"], [1, 1, 0, 0, 0, 1, 0, 1, 0, 0, 0]),
    (["I", "I", "41"], [1, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0]),
    (["J", "J", "42"], [1, 0, 1, 1, 0, 1, 1, 1, 0, 0, 0]),
    (["K", "K", "43"], [1, 0, 1, 1, 0, 0, 0, 1, 1, 1, 0]),
    (["L", "L", "44"], [1, 0, 0, 0, 1, 1, 0, 1, 1, 1, 0]),
    (["M", "M", "45"], [1, 0, 1, 1, 1, 0, 1, 1, 0, 0, 0]),
    (["N", "N", "46"], [1, 0, 1, 1, 1, 0, 0, 0, 1, 1, 0]),
    (["O", "O", "47"], [1, 0, 0, 0, 1, 1, 1, 0, 1, 1, 0]),
    (["P", "P", "48"], [1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 0]),
    (["Q", "Q", "49"], [1, 1, 0, 1, 0, 0, 0, 1, 1, 1, 0]),
    (["R", "R", "50"], [1, 1, 0, 0, 0, 1, 0, 1, 1, 1, 0]),
    (["S", "S", "51"], [1, 1, 0, 1, 1, 1, 0, 1, 0, 0, 0]),
    (["T", "T", "52"], [1, 1, 0, 1, 1, 1, 0, 0, 0, 1, 0]),
    (["U", "U", "53"], [1, 1, 0, 1, 1, 1, 0, 1, 1, 1, 0]),
    (["V", "V", "54"], [1, 1, 1, 0, 1, 0, 1, 1, 0, 0, 0]),
    (["W", "W", "55"], [1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0]),
    (["X", "X", "56"], [1, 1, 1, 0, 0, 0, 1, 0, 1, 1, 0]),
    (["Y", "Y", "57"], [1, 1, 1, 0, 1, 1, 0, 1, 0, 0, 0]),
    (["Z", "Z", "58"], [1, 1, 1, 0, 1, 1, 0, 0, 0, 1, 0]),
    (["[", "[", "59"], [1, 1, 1, 0, 0, 0, 1, 1, 0, 1, 0]),
    (["\\", "\\", "60"], [1, 1, 1, 0, 1, 1, 1, 1, 0, 1, 0]),
    (["]", "]", "61"], [1, 1, 0, 0, 1, 0, 0, 0, 0, 1, 0]),
    (["^", "^", "62"], [1, 1, 1, 1, 0, 0, 0, 1, 0, 1, 0]),
    (["_", "_", "63"], [1, 0, 1, 0, 0, 1, 1, 0, 0, 0, 0]),
    (["\u{0000}", "`", "64"], [1, 0, 1, 0, 0, 0, 0, 1, 1, 0, 0]),
    (["\u{0001}", "a", "65"], [1, 0, 0, 1, 0, 1, 1, 0, 0, 0, 0]),
    (["\u{0002}", "b", "66"], [1, 0, 0, 1, 0, 0, 0, 0, 1, 1, 0]),
    (["\u{0003}", "c", "67"], [1, 0, 0, 0, 0, 1, 0, 1, 1, 0, 0]),
    (["\u{0004}", "d", "68"], [1, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0]),
    (["\u{0005}", "e", "69"], [1, 0, 1, 1, 0, 0, 1, 0, 0, 0, 0]),
    (["\u{0006}", "f", "70"], [1, 0, 1, 1, 0, 0, 0, 0, 1, 0, 0]),
    (["\u{0007}", "g", "71"], [1, 0, 0, 1, 1, 0, 1, 0, 0, 0, 0]),
    (["\u{0008}", "h", "72"], [1, 0, 0, 1, 1, 0, 0, 0, 0, 1, 0]),
    (["\u{0009}", "i", "73"], [1, 0, 0, 0, 0, 1, 1, 0, 1, 0, 0]),
    (["\u{000A}", "j", "74"], [1, 0, 0, 0, 0, 1, 1, 0, 0, 1, 0]),
    (["\u{000B}", "k", "75"], [1, 1, 0, 0, 0, 0, 1, 0, 0, 1, 0]),
    (["\u{000C}", "l", "76"], [1, 1, 0, 0, 1, 0, 1, 0, 0, 0, 0]),
    (["\u{000D}", "m", "77"], [1, 1, 1, 1, 0, 1, 1, 1, 0, 1, 0]),
    (["\u{000E}", "n", "78"], [1, 1, 0, 0, 0, 0, 1, 0, 1, 0, 0]),
    (["\u{000F}", "o", "79"], [1, 0, 0, 0, 1, 1, 1, 1, 0, 1, 0]),
    (["\u{0010}", "p", "80"], [1, 0, 1, 0, 0, 1, 1, 1, 1, 0, 0]),
    (["\u{0011}", "q", "81"], [1, 0, 0, 1, 0, 1, 1, 1, 1, 0, 0]),
    (["\u{0012}", "r", "82"], [1, 0, 0, 1, 0, 0, 1, 1, 1, 1, 0]),
    (["\u{0013}", "s", "83"], [1, 0, 1, 1, 1, 1, 0, 0, 1, 0, 0]),
    (["\u{0014}", "t", "84"], [1, 0, 0, 1, 1, 1, 1, 0, 1, 0, 0]),
    (["\u{0015}", "u", "85"], [1, 0, 0, 1, 1, 1, 1, 0, 0, 1, 0]),
    (["\u{0016}", "v", "86"], [1, 1, 1, 1, 0, 1, 0, 0, 1, 0, 0]),
    (["\u{0017}", "w", "87"], [1, 1, 1, 1, 0, 0, 1, 0, 1, 0, 0]),
    (["\u{0018}", "x", "88"], [1, 1, 1, 1, 0, 0, 1, 0, 0, 1, 0]),
    (["\u{0019}", "y", "89"], [1, 1, 0, 1, 1, 0, 1, 1, 1, 1, 0]),
    (["\u{001A}", "z", "90"], [1, 1, 0, 1, 1, 1, 1, 0, 1, 1, 0]),
    (["\u{001B}", "{", "91"], [1, 1, 1, 1, 0, 1, 1, 0, 1, 1, 0]),
    (["\u{001C}", "|", "92"], [1, 0, 1, 0, 1, 1, 1, 1, 0, 0, 0]),
    (["\u{001D}", "}", "93"], [1, 0, 1, 0, 0, 0, 1, 1, 1, 1, 0]),
    (["\u{001E}", "~", "94"], [1, 0, 0, 0, 1, 0, 1, 1, 1, 1, 0]),
    (
        ["\u{001F}", "\u{007F}", "95"],
        [1, 0, 1, 1, 1, 1, 0, 1, 0, 0, 0],
    ),
    (
        ["\u{017B}", "\u{017B}", "96"],
        [1, 0, 1, 1, 1, 1, 0, 0, 0, 1, 0],
    ),
    (
        ["\u{017A}", "\u{017A}", "97"],
        [1, 1, 1, 1, 0, 1, 0, 1, 0, 0, 0],
    ),
    (
        ["\u{017D}", "\u{017D}", "98"],
        [1, 1, 1, 1, 0, 1, 0, 0, 0, 1, 0],
    ),
    (["Ć", "Ć", "99"], [1, 0, 1, 1, 1, 0, 1, 1, 1, 1, 0]),
    (["Ɓ", "\u{017C}", "Ɓ"], [1, 0, 1, 1, 1, 1, 0, 1, 1, 1, 0]),
    (["\u{017C}", "À", "À"], [1, 1, 1, 0, 1, 0, 1, 1, 1, 1, 0]),
    (
        ["\u{0179}", "\u{0179}", "\u{0179}"],
        [1, 1, 1, 1, 0, 1, 0, 1, 1, 1, 0],
    ),
    (
        ["START-À", "START-À", "START-À"],
        [1, 1, 0, 1, 0, 0, 0, 0, 1, 0, 0],
    ),
    (
        ["START-Ɓ", "START-Ɓ", "START-Ɓ"],
        [1, 1, 0, 1, 0, 0, 1, 0, 0, 0, 0],
    ),
    (
        ["START-Ć", "START-Ć", "START-Ć"],
        [1, 1, 0, 1, 0, 0, 1, 1, 1, 0, 0],
    ),
];

// Durdurma dizisi.
const STOP: Encoding = [1, 1, 0, 0, 0, 1, 1, 1, 0, 1, 0];

// Sonlandırma dizisi.
const TERM: [u8; 2] = [1, 1];

/// Code128 barkod türü.
///
/// # Karakter kümeleri
///
/// * 128A (A Kod Kümesi) – 00 ile 95 arasındaki ASCII karakterleri (0–9, A–Z ve denetim
///   kodları), özel karakterler ve FNC 1–4
/// * 128B (B Kod Kümesi) – 32 ile 127 arasındaki ASCII karakterleri (0–9, A–Z, a–z), özel
///   karakterler ve FNC 1–4
/// * 128C (C Kod Kümesi) – 00–99 (iki basamağı tek kod noktasıyla kodlar) ve FNC1
///
/// Ek bilgi için [modül] belgelerine bakın.
///
/// [modül]: crate::sym::code128
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Code128(Vec<Unit>);

impl Unit {
    fn index(&self) -> usize {
        self.index
    }
}

impl CharacterSet {
    fn from_char(c: char) -> Result<CharacterSet> {
        match c {
            'À' => Ok(CharacterSet::A),
            'Ɓ' => Ok(CharacterSet::B),
            'Ć' => Ok(CharacterSet::C),
            _ => Err(Error::character(Some(c), None)),
        }
    }

    fn unit(self, n: usize) -> Result<Unit> {
        let kind = match self {
            CharacterSet::A => UnitKind::A,
            CharacterSet::B => UnitKind::B,
            CharacterSet::C => UnitKind::C,
            CharacterSet::None => return Err(Error::character(None, None)),
        };
        Ok(Unit { kind, index: n })
    }

    fn index(self) -> Result<usize> {
        match self {
            CharacterSet::A => Ok(0),
            CharacterSet::B => Ok(1),
            CharacterSet::C => Ok(2),
            CharacterSet::None => Err(Error::character(None, None)),
        }
    }

    fn lookup(self, s: &str) -> Result<Unit> {
        let p = self.index()?;

        match CHARS
            .iter()
            .position(|character| character.0.get(p).is_some_and(|value| *value == s))
        {
            Some(i) => self.unit(i),
            None => Err(Error::character(s.chars().next(), None)),
        }
    }
}

impl Code128 {
    /// Yeni bir barkod oluşturur.
    /// Girdinin çözümlenme sonucunu `Result<Code128, Error>` olarak döndürür.
    pub fn new<T: AsRef<str>>(data: T) -> Result<Code128> {
        let data = data.as_ref();
        let data_len = data.chars().count();

        // Üst sınır diğer sembolojilerle tutarlıdır ve sembolojiden üretilen kodlamaların
        // 100.000 modüllük gösterim sınırının altında kalmasını güvence altına alır.
        if !(2..=256).contains(&data_len) {
            return Err(Error::length(2, Some(256), data_len));
        }

        Code128::parse(data.chars().collect()).map(Code128)
    }

    // Veriyi belirteçlere ayırır ve uygun karakter kümelerinde toplar.
    fn parse(chars: Vec<char>) -> Result<Vec<Unit>> {
        let mut units: Vec<Unit> = vec![];
        let mut char_set = CharacterSet::None;
        let mut carry: Option<char> = None;
        // SHIFT sonrasında yalnız bir sonraki karakterin çözümleneceği geçici küme.
        let mut shift: Option<CharacterSet> = None;

        for (index, ch) in chars.into_iter().enumerate() {
            match ch {
                'À' | 'Ɓ' | 'Ć' if units.is_empty() => {
                    char_set = CharacterSet::from_char(ch)?;

                    let c = format!("START-{}", ch);
                    let u = char_set.lookup(&c)?;
                    units.push(u);
                }
                'À' | 'Ɓ' | 'Ć' => {
                    if shift.is_some() || (char_set == CharacterSet::C && carry.is_some()) {
                        return Err(Error::character(Some(ch), Some(index)));
                    }

                    let u = char_set.lookup(&ch.to_string())?;
                    units.push(u);

                    char_set = CharacterSet::from_char(ch)?;
                }
                'Ž' => {
                    // SHIFT yalnız A ve B kümelerinde tanımlıdır ve art arda kullanılamaz.
                    let target = match char_set {
                        CharacterSet::A => CharacterSet::B,
                        CharacterSet::B => CharacterSet::A,
                        CharacterSet::C | CharacterSet::None => {
                            return Err(Error::character(Some(ch), Some(index)));
                        }
                    };

                    if shift.is_some() {
                        return Err(Error::character(Some(ch), Some(index)));
                    }

                    let u = char_set.lookup(&ch.to_string())?;
                    units.push(u);
                    shift = Some(target);
                }
                d if d.is_ascii_digit() && char_set == CharacterSet::C => match carry {
                    None => carry = Some(d),
                    Some(n) => {
                        let num = format!("{}{}", n, d);
                        let u = char_set.lookup(&num)?;
                        units.push(u);
                        carry = None;
                    }
                },
                _ => {
                    let set = shift.take().unwrap_or(char_set);
                    let u = set.lookup(&ch.to_string())?;
                    units.push(u);
                }
            }
        }

        if shift.is_some() {
            // Askıda kalan SHIFT, kodlanacak karakteri olmayan bir küme geçişi bırakır.
            return Err(Error::character(Some('Ž'), None));
        }

        match carry {
            Some(character) => Err(Error::character(Some(character), None)),
            None => Ok(units),
        }
    }

    /// Modülo-103 algoritmasıyla sağlama dizinini hesaplar.
    fn checksum_value(&self) -> usize {
        let mut sum = 0usize;

        for (position, unit) in self.0.iter().enumerate() {
            let weight = position.max(1) % 103;
            let contribution = (unit.index() % 103) * weight;
            sum = (sum + contribution) % 103;
        }

        sum
    }

    fn checksum_encoding(&self) -> Encoding {
        let value = self.checksum_value();
        self.unit_encoding(&Unit {
            kind: UnitKind::A,
            index: value,
        })
    }

    fn unit_encoding(&self, c: &Unit) -> Encoding {
        let encoding = CHARS.get(c.index()).map(|(_, encoding)| *encoding);
        helpers::invariant_or(
            encoding,
            [0; 11],
            "Code128 birim dizini oluşturucuda doğrulanmış olmalıdır",
        )
    }

    fn payload(&self) -> Vec<u8> {
        let slices: Vec<Encoding> = self.0.iter().map(|unit| self.unit_encoding(unit)).collect();

        helpers::join_iters(slices.iter())
    }

    /// Barkodu kodlar.
    /// İkili basamakları bir `Vec<u8>` içinde döndürür.
    pub fn encode(&self) -> Vec<u8> {
        let payload = self.payload();
        let checksum = self.checksum_encoding();

        helpers::join_slices(&[payload.as_slice(), &checksum, &STOP, &TERM])
    }
}

#[cfg(test)]
mod tests {
    use crate::error::{Error, Result};
    use crate::sym::code128::*;
    #[cfg(not(feature = "std"))]
    use alloc::string::String;
    use core::char;

    fn collapse_vec(v: Vec<u8>) -> String {
        v.iter()
            .filter_map(|digit| char::from_digit(u32::from(*digit), 10))
            .collect()
    }

    #[test]
    fn new_code128() -> Result<()> {
        let code128_a = Code128::new("À !! Ć0201");
        let code128_b = Code128::new("À!!  \" ");

        assert!(code128_a.is_ok());
        assert!(code128_b.is_ok());
        Ok(())
    }

    #[test]
    fn invalid_length_code128() -> Result<()> {
        let code128_a = Code128::new("");

        assert!(matches!(code128_a, Err(Error::Length { .. })));
        Ok(())
    }

    #[test]
    fn invalid_data_code128() -> Result<()> {
        let code128_a = Code128::new("À☺ "); // Bilinmeyen karakter.
        let code128_b = Code128::new("ÀHELLOĆ12352"); // Sonda eşleşmemiş basamak.
        let code128_c = Code128::new("HELLO"); // Karakter kümesi belirtilmemiş.

        assert!(matches!(code128_a, Err(Error::Character { .. })));
        assert!(matches!(code128_b, Err(Error::Character { .. })));
        assert!(matches!(code128_c, Err(Error::Character { .. })));
        Ok(())
    }

    #[test]
    fn code128_encode() -> Result<()> {
        let code128_a = Code128::new("ÀHELLO")?;
        let code128_b = Code128::new("ÀXYĆ2199")?;
        let code128_c = Code128::new("ƁxyZÀ199!*1")?;

        assert_eq!(
            collapse_vec(code128_a.encode()),
            "110100001001100010100010001101000100011011101000110111010001110110110100010001100011101011"
        );
        assert_eq!(
            collapse_vec(code128_b.encode()),
            "110100001001110001011011101101000101110111101101110010010111011110100111011001100011101011"
        );
        assert_eq!(
            collapse_vec(code128_c.encode()),
            "1101001000011110010010110110111101110110001011101011110100111001101110010110011100101100110011011001100100010010011100110100101111001100011101011"
        );
        Ok(())
    }

    #[test]
    fn code128_encode_special_chars() -> Result<()> {
        let code128_a = Code128::new("ÀB\u{0006}")?;

        assert_eq!(
            collapse_vec(code128_a.encode()),
            "110100001001000101100010110000100100110100001100011101011"
        );
        Ok(())
    }

    #[test]
    fn code128_encode_del_in_set_b() -> Result<()> {
        let code128_a = Code128::new("Ɓa\u{007F}")?;

        assert_eq!(
            collapse_vec(code128_a.encode()),
            "110100100001001011000010111101000110001011101100011101011"
        );
        Ok(())
    }

    #[test]
    fn code128_shift_encodes_next_char_in_other_set() -> Result<()> {
        // A kümesinden tek karakterlik B geçişi: 'e' yalnız B kümesinde bulunur.
        let a_to_b = Code128::new("ÀŽe")?;
        // B kümesinden tek karakterlik A geçişi: ENQ yalnız A kümesinde bulunur.
        let b_to_a = Code128::new("ƁaŽ\u{0005}a")?;

        assert_eq!(
            collapse_vec(a_to_b.encode()),
            "110100001001111010001010110010000110110110001100011101011"
        );
        assert_eq!(
            collapse_vec(b_to_a.encode()),
            "1101001000010010110000111101000101011001000010010110000100011001001100011101011"
        );
        Ok(())
    }

    #[test]
    fn code128_shift_rejects_invalid_usage() -> Result<()> {
        // SHIFT hedef kümede bulunmayan bir karakterle izlenemez.
        assert!(matches!(
            Code128::new("ÀŽ\u{0005}"),
            Err(Error::Character { .. })
        ));
        // SHIFT, C kümesinde tanımlı değildir.
        assert!(matches!(Code128::new("ĆŽ12"), Err(Error::Character { .. })));
        // SHIFT son karakter olamaz.
        assert!(matches!(Code128::new("ÀAŽ"), Err(Error::Character { .. })));
        // SHIFT art arda kullanılamaz.
        assert!(matches!(Code128::new("ÀŽŽA"), Err(Error::Character { .. })));
        // SHIFT'ten hemen sonra küme değişimi gelemez.
        assert!(matches!(Code128::new("ÀŽƁA"), Err(Error::Character { .. })));
        Ok(())
    }

    #[test]
    fn code128_length_limits() -> Result<()> {
        let longest_valid = format!("À{}", "A".repeat(255));
        let too_long = format!("À{}", "A".repeat(256));

        assert!(Code128::new(longest_valid).is_ok());
        assert!(matches!(
            Code128::new(too_long),
            Err(Error::Length {
                min: 2,
                max: Some(256),
                found: 257
            })
        ));
        Ok(())
    }

    #[test]
    fn code128_encode_fnc_chars() -> Result<()> {
        let code128_a = Code128::new("ĆŹ4218402050À0")?;

        assert_eq!(
            collapse_vec(code128_a.encode()),
            "110100111001111010111010110111000110011100101100010100011001001110110001011101110101111010011101100101011110001100011101011"
        );
        Ok(())
    }

    #[test]
    fn code128_encode_longhand() -> Result<()> {
        let code128_a = Code128::new("\u{00C0}HELLO")?;
        let code128_b = Code128::new("\u{00C0}XY\u{0106}2199")?;
        let code128_c = Code128::new("\u{0181}xyZ\u{00C0}199!*1")?;

        assert_eq!(
            collapse_vec(code128_a.encode()),
            "110100001001100010100010001101000100011011101000110111010001110110110100010001100011101011"
        );
        assert_eq!(
            collapse_vec(code128_b.encode()),
            "110100001001110001011011101101000101110111101101110010010111011110100111011001100011101011"
        );
        assert_eq!(
            collapse_vec(code128_c.encode()),
            "1101001000011110010010110110111101110110001011101011110100111001101110010110011100101100110011011001100100010010011100110100101111001100011101011"
        );
        Ok(())
    }
}
