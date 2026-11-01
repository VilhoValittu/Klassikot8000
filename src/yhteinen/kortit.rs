//! Pelikorttien piirto. Kortti piirretään aina täysikokoiselle ruudulle.

use std::f32::consts::TAU;

use klassikot8000::kortit::{KUNINGAS, Kortti, Maa};

use super::piirto::*;
use super::vektori::{Tahko, sekoita_liuku, tahko4, tummenna, vaalenna};

pub const KORTTI_L: i32 = 150;
pub const KORTTI_K: i32 = 210;

const SELKA_TUMMA: u32 = 0x8C1830;
const SELKA_VAALEA: u32 = 0xD84858;
const PAPERI: u32 = 0xF8F4E8;
const PAPERI_VARJO: u32 = 0xC8C4D8;
const KULTA: u32 = 0xF0C040;
const KULTA_TUMMA: u32 = 0xA87020;
const IHO: u32 = 0xF8C8A0;
const HIUS: u32 = 0x6A3C20;
const VIITTA_MUSTA: u32 = 0x2838A0;
const VERKA_VAALEA: u32 = 0x10583C;
const VERKA_TUMMA: u32 = 0x07382A;

// Maat ovat 9x9-kuvia: '#' on maan väri, 'o' sen kiilto.
const HERTTA: [&str; 9] = [
    ".##...##.",
    "#oo#.####",
    "#o#######",
    "#########",
    ".#######.",
    "..#####..",
    "...###...",
    "....#....",
    ".........",
];
const RUUTU: [&str; 9] = [
    "....#....",
    "...o##...",
    "..oo###..",
    ".oo#####.",
    "#########",
    ".#######.",
    "..#####..",
    "...###...",
    "....#....",
];
const PATA: [&str; 9] = [
    "....#....",
    "...o##...",
    "..oo###..",
    ".oo#####.",
    "#o#######",
    "#########",
    ".##.#.##.",
    "....#....",
    "...###...",
];
const RISTI: [&str; 9] = [
    "...###...",
    "..oo###..",
    "..o####..",
    "...###...",
    ".##.#.##.",
    "#oo#####o",
    "#########",
    ".##.#.##.",
    "...###...",
];

// Kuvakorttien hahmon yläpuolikas, joka toistuu alhaalla peilattuna kuten oikeassa kortissa.
// g kulta, r kirkas jalokivi, s iho, h hius, e silmä, m suu, v viitta, w vaalea reunus.
const KUVAN_LEVEYS: i32 = 15;
const KUVAN_KERROIN: i32 = 4;
const KUNINGAS_KUVA: [&str; 13] = [
    "...g..g.g..g...",
    "...ggggggggg...",
    "...grgggggrg...",
    "....sssssss....",
    "...hsesssesh...",
    "...hssssssssh..",
    "...hhsmmmshh...",
    "....hhhhhhh....",
    "...vvvhhhvvv...",
    "..vvvvgggvvvv..",
    ".vvvwvvgvvwvvv.",
    ".vvvwvvgvvwvvv.",
    "gvvvwvvgvvwvvvg",
];
const KUNINGATAR_KUVA: [&str; 13] = [
    "......g.g......",
    ".....ggrgg.....",
    "....hgggggh....",
    "...hhsssssshh..",
    "...hsesssesh...",
    "..hhsssssssshh.",
    "..hhhssmsshhh..",
    "..hh.sssss.hh..",
    "..hvvvwwwvvvh..",
    "..vvvvgggvvvv..",
    ".vvvwvvgvvwvvv.",
    ".vvvwvvgvvwvvv.",
    "gvvvwvvgvvwvvvg",
];
const JATKA_KUVA: [&str; 13] = [
    "............ww.",
    "...vvvvvvvvww..",
    "..vvvvvvvvvvv..",
    "..gggggggggggg.",
    "...hsssssssh...",
    "...hsesssesh...",
    "...hsssssssh...",
    "....ssmmmss....",
    "...vvvsssvvv...",
    "..vvvvgggvvvv..",
    ".vvvwvvgvvwvvv.",
    ".vvvwvvgvvwvvv.",
    "gvvvwvvgvvwvvvg",
];

/// Yksi pelimerkki kolmiulotteiseen malliin. `taso` kertoo sen paikan pinossa alhaalta lukien.
pub fn pelimerkki(x: f32, z: f32, taso: usize, vari: u32, tahkot: &mut Vec<Tahko>) {
    const KULMIA: usize = 8;
    const SADE: f32 = 1.2;
    const KORKEUS: f32 = 0.35;
    let (y0, y1) = (taso as f32 * KORKEUS, (taso + 1) as f32 * KORKEUS);
    let piste = |kulma: usize, y: f32| {
        let (sin, cos) = (kulma as f32 / KULMIA as f32 * TAU).sin_cos();
        [x + SADE * cos, y, z + SADE * sin]
    };
    for kulma in 0..KULMIA {
        let seuraava = (kulma + 1) % KULMIA;
        tahkot.push(tahko4(
            piste(kulma, y0),
            piste(seuraava, y0),
            piste(seuraava, y1),
            piste(kulma, y1),
            vari,
        ));
    }
    // Kansi kolmesta nelikulmiosta, koska tahkossa ei ole kahdeksaa kulmaa.
    for [a, b, c, d] in [[0, 1, 2, 3], [0, 3, 4, 7], [4, 5, 6, 7]] {
        tahkot.push(tahko4(
            piste(a, y1),
            piste(b, y1),
            piste(c, y1),
            piste(d, y1),
            vari,
        ));
    }
}

fn maan_vari(maa: Maa) -> u32 {
    if maa.on_punainen() { OSUMA } else { 0x181828 }
}

fn arvon_nimi(arvo: u8) -> String {
    match arvo {
        1 => "A".to_string(),
        11 => "J".to_string(),
        12 => "Q".to_string(),
        KUNINGAS => "K".to_string(),
        _ => arvo.to_string(),
    }
}

/// Maakuvioiden paikat kortin keskialueella, -1..1 kumpaankin suuntaan.
fn kuviot(arvo: u8) -> &'static [(f32, f32)] {
    const A: f32 = 0.55;
    const B: f32 = 0.33;
    const C: f32 = 0.66;
    match arvo {
        2 => &[(0.0, -1.0), (0.0, 1.0)],
        3 => &[(0.0, -1.0), (0.0, 0.0), (0.0, 1.0)],
        4 => &[(-A, -1.0), (A, -1.0), (-A, 1.0), (A, 1.0)],
        5 => &[(-A, -1.0), (A, -1.0), (0.0, 0.0), (-A, 1.0), (A, 1.0)],
        6 => &[
            (-A, -1.0),
            (A, -1.0),
            (-A, 0.0),
            (A, 0.0),
            (-A, 1.0),
            (A, 1.0),
        ],
        7 => &[
            (-A, -1.0),
            (A, -1.0),
            (0.0, -0.5),
            (-A, 0.0),
            (A, 0.0),
            (-A, 1.0),
            (A, 1.0),
        ],
        8 => &[
            (-A, -1.0),
            (A, -1.0),
            (0.0, -0.5),
            (-A, 0.0),
            (A, 0.0),
            (0.0, 0.5),
            (-A, 1.0),
            (A, 1.0),
        ],
        9 => &[
            (-A, -1.0),
            (A, -1.0),
            (-A, -B),
            (A, -B),
            (0.0, 0.0),
            (-A, B),
            (A, B),
            (-A, 1.0),
            (A, 1.0),
        ],
        10 => &[
            (-A, -1.0),
            (A, -1.0),
            (0.0, -C),
            (-A, -B),
            (A, -B),
            (-A, B),
            (A, B),
            (0.0, C),
            (-A, 1.0),
            (A, 1.0),
        ],
        _ => &[],
    }
}

impl Piirtaja<'_> {
    /// Kortin muoto kulmat pyöristettyinä ja varjo pöydällä.
    fn kortin_pohja(&mut self, x: i32, y: i32, vari: u32) {
        let p = self.k.pikseli;
        self.k
            .laatikko(x + 2 * p, y + KORTTI_K, KORTTI_L - 2 * p, p, 0x042418);
        self.k
            .laatikko(x + KORTTI_L, y + 2 * p, p, KORTTI_K - 2 * p, 0x042418);
        self.k.laatikko(x + p, y, KORTTI_L - 2 * p, KORTTI_K, MUSTA);
        self.k.laatikko(x, y + p, KORTTI_L, KORTTI_K - 2 * p, MUSTA);
        self.k
            .laatikko(x + p, y + p, KORTTI_L - 2 * p, KORTTI_K - 2 * p, vari);
    }

    /// Selkäpuolella on punaisella pohjalla kultareunainen vinoneliöristikko.
    pub fn kortin_tausta(&mut self, x: i32, y: i32) {
        let p = self.k.pikseli;
        self.kortin_pohja(x, y, PAPERI);
        let (sx, sy) = (x + 2 * p, y + 2 * p);
        let (sl, sk) = (KORTTI_L - 4 * p, KORTTI_K - 4 * p);
        self.k.laatikko(sx, sy, sl, sk, SELKA_TUMMA);
        self.k.kehys(sx, sy, sl, sk, KULTA);
        for rivi in 2..sk / p - 2 {
            for sarake in 2..sl / p - 2 {
                let vari = match ((rivi + sarake) % 6, (rivi - sarake).rem_euclid(6)) {
                    (0, _) | (_, 0) => SELKA_VAALEA,
                    (3, 3) => KULTA,
                    _ => continue,
                };
                self.k.laatikko(sx + sarake * p, sy + rivi * p, p, p, vari);
            }
        }
    }

    pub fn pelikortti(&mut self, x: i32, y: i32, kortti: Kortti) {
        let vari = maan_vari(kortti.maa);
        let p = self.k.pikseli;
        self.kortin_pohja(x, y, PAPERI);
        self.k.laatikko(
            x + p,
            y + KORTTI_K - 2 * p,
            KORTTI_L - 2 * p,
            p,
            PAPERI_VARJO,
        );
        self.k.laatikko(
            x + KORTTI_L - 2 * p,
            y + p,
            p,
            KORTTI_K - 2 * p,
            PAPERI_VARJO,
        );

        let nimi = arvon_nimi(kortti.arvo);
        self.suurteksti(x + 9, y + 8, &nimi, OTSIKKO, vari, true);
        self.maakuvio(kortti.maa, (x + KORTTI_L - 24, y + 21), 12.0, vari);

        let keski = (x + KORTTI_L / 2, y + 42 + (KORTTI_K - 42) / 2);
        match kortti.arvo {
            1 => self.maakuvio(kortti.maa, keski, 42.0, vari),
            11..=KUNINGAS => self.kuvakortti(x, y, kortti),
            arvo => {
                for &(px, py) in kuviot(arvo) {
                    let kohta = (keski.0 + (px * 54.0) as i32, keski.1 + (py * 66.0) as i32);
                    self.maakuvio(kortti.maa, kohta, 15.0, vari);
                }
            }
        }
    }

    /// Kullalla kehystetty hahmo, jonka alapuoli on yläpuolen peilikuva.
    fn kuvakortti(&mut self, x: i32, y: i32, kortti: Kortti) {
        let (kx, ky) = (x + 21, y + 42);
        let (kl, kk) = (KORTTI_L - 42, KORTTI_K - 57);
        self.k.liukulaatikko(kx, ky, kl, kk, 0xFFF0B0, 0xF0C860);
        self.k.kehys(kx, ky, kl, kk, KULTA_TUMMA);

        let kuva = match kortti.arvo {
            11 => &JATKA_KUVA,
            12 => &KUNINGATAR_KUVA,
            _ => &KUNINGAS_KUVA,
        };
        let viitta = if kortti.maa.on_punainen() {
            OSUMA
        } else {
            VIITTA_MUSTA
        };
        let vari = move |merkki: u8| match merkki {
            b'g' => Some(KULTA),
            b'r' => Some(0xE82848),
            b's' => Some(IHO),
            b'h' => Some(HIUS),
            b'e' | b'm' => Some(0x401818),
            b'v' => Some(viitta),
            b'w' => Some(vaalenna(viitta, 0.55)),
            _ => None,
        };
        let sx = kx + (kl - KUVAN_LEVEYS * KUVAN_KERROIN) / 2;
        let rivit = kuva.len() as i32;
        let sy = ky + kk / 2 - rivit * KUVAN_KERROIN;
        self.k.sprite((sx, sy), kuva, KUVAN_KERROIN, vari);
        let mut peilattu = *kuva;
        peilattu.reverse();
        let ala_y = sy + rivit * KUVAN_KERROIN;
        self.k.sprite((sx, ala_y), &peilattu, KUVAN_KERROIN, vari);
    }

    /// Tyhjä paikka kortille: upotettu kuoppa verassa.
    pub fn korttipaikka(&mut self, x: i32, y: i32) {
        self.k.laatikko(x, y, KORTTI_L, KORTTI_K, VERKA_TUMMA);
        self.k.kehys(x, y, KORTTI_L, KORTTI_K, VERKA_VAALEA);
    }

    /// Tyhjän perustan himmeä maa.
    pub fn himmea_maa(&mut self, maa: Maa, keski: (i32, i32)) {
        self.maakuvio(maa, keski, 36.0, VERKA_VAALEA);
    }

    /// Maa pikselikuvana. `koko` on kuvan säde pikseleinä kuten ennenkin.
    pub fn maakuvio(&mut self, maa: Maa, keski: (i32, i32), koko: f32, vari: u32) {
        let kuva = match maa {
            Maa::Hertta => &HERTTA,
            Maa::Ruutu => &RUUTU,
            Maa::Pata => &PATA,
            Maa::Risti => &RISTI,
        };
        let kerroin = ((koko / 6.5).round() as i32).max(1);
        let puoli = 9 * kerroin / 2;
        let kiilto = sekoita_liuku(vari, VALKOINEN, 0.45);
        let varjo = tummenna(vari, 0.2);
        self.k.sprite(
            (keski.0 - puoli, keski.1 - puoli),
            kuva,
            kerroin,
            move |merkki| match merkki {
                b'#' => Some(vari),
                b'o' => Some(kiilto),
                b'x' => Some(varjo),
                _ => None,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kuvat_ovat_tasalevyisia() {
        for kuva in [&HERTTA, &RUUTU, &PATA, &RISTI] {
            assert!(kuva.iter().all(|rivi| rivi.len() == 9));
        }
        for kuva in [&KUNINGAS_KUVA, &KUNINGATAR_KUVA, &JATKA_KUVA] {
            for rivi in kuva {
                assert!(rivi.len() as i32 <= KUVAN_LEVEYS, "{rivi}");
            }
        }
    }

    #[test]
    fn hahmo_mahtuu_kehykseen() {
        const { assert!(KUVAN_LEVEYS * KUVAN_KERROIN <= KORTTI_L - 42) };
        const { assert!(2 * 13 * KUVAN_KERROIN <= KORTTI_K - 57) };
    }
}
