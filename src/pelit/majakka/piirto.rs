//! Majakka8000:n näkymät. Meri lasketaan kolmena valmiina kuvana (yö, keilan valo ja päivä),
//! ja joka ruudussa niistä vain valitaan pikselit: keila ja aamunkoitto ovat ditteröity raja
//! kuvien välillä, joten valaistus ei maksa juuri mitään.

use std::f32::consts::{PI, TAU};

use klassikot8000::majakka::{
    self, KIINNIOTTO_MS, Kari, Laiva, Laji, MAJAKKA, Meri, Piste, SATAMA, SUUN_PUOLIKAS, Vaikeus,
    ranta_y, suulla,
};

use super::peli::{Majakka, TEHOSTE_MS, Tehoste, Tehostelaji, VIESTEJA, Vaihe};
use crate::yhteinen::kieli::kaanna;
use crate::yhteinen::piirto::*;
use crate::yhteinen::rasteri::{Rasteri, ditteroi, kohina, liukuvari};
use crate::yhteinen::vektori::{Kangas, Tahko, kuutio, sekoita_liuku, tahko4, tummenna, vaalenna};

pub const KENTTA_X: i32 = 30;
pub const KENTTA_Y: i32 = 48;
const KENTTA_L: i32 = majakka::LEVEYS as i32;
const KENTTA_K: i32 = majakka::KORKEUS as i32;
/// Konepikseli klassisessa kuvassa: meri on 220 x 180 konepikseliä.
const P: i32 = 3;
const RUUTUJA_L: usize = (KENTTA_L / P) as usize;
const RUUTUJA_K: usize = (KENTTA_K / P) as usize;
const SARAKE_X: i32 = KENTTA_X + KENTTA_L + 30;
const TILASTON_ASKEL: i32 = 72;
const ARVON_Y: i32 = 27;
const PARHAAT_Y: i32 = KENTTA_Y + 6 * TILASTON_ASKEL;
const PARHAAT_L: i32 = LEVEYS as i32 - 12 - SARAKE_X;
// Kapeaan sarakkeeseen mahtuu sijoitus, kahdeksan merkin nimi ja kuusinumeroiset pisteet.
const PARHAAT_KOKO: (i32, i32, i32) = (10, 15, 13);
const MITTARI_K: i32 = 15;
const RENKAAN_SADE: i32 = 12;

const OHJE: &str = "majakka.ohje";
const TAPPIO: &str = "yhteinen.peli_paattyi";

// Alkuruudun tasovalinta samoilla mitoilla kuin miinaharavassa.
const TASO_Y: i32 = ALOITUS_Y - NAPPI_K - 6;
const TASO_X: i32 = 48 + 6 * MERKKI;
const TASO_VALI: i32 = 12;
const TASOT: [(Vaikeus, &str, i32); 3] = [
    (Vaikeus::Helppo, "yhteinen.taso_1", TASO_X),
    (
        Vaikeus::Normaali,
        "yhteinen.taso_2",
        TASO_X + 8 * MERKKI + 24 + TASO_VALI,
    ),
    (
        Vaikeus::Vaikea,
        "yhteinen.taso_3",
        TASO_X + 18 * MERKKI + 48 + 2 * TASO_VALI,
    ),
];

const ALKU_OSAT: [&str; 4] = [
    "majakka.tervetuloa",
    "yhteinen.ohjeet",
    "majakka.ohje_1",
    "majakka.ohje_2",
];
const ALKU_RIVIT: [&str; 4] = [
    "majakka.nappain_keila",
    "majakka.nappain_torvi",
    "yhteinen.tauko_aanet",
    "yhteinen.takaisin",
];

// Rannan talot: vasen yläkulma kentän pikseleinä. Ikkunat palavat yöllä.
const TALOT: [(f32, f32); 7] = [
    (60.0, 522.0),
    (126.0, 528.0),
    (192.0, 521.0),
    (414.0, 524.0),
    (474.0, 529.0),
    (552.0, 522.0),
    (612.0, 527.0),
];
const TALON_L: f32 = 15.0;
const TALON_K: f32 = 9.0;

const VAAHDON_ASKEL_MS: u32 = 300;
const LAMPUN_VILKKU_MS: u32 = 250;
const SATAMAVALON_VILKKU_MS: u32 = 700;
const VANAN_VILKKU_MS: u32 = 160;

pub fn vaikeus_pisteesta(x: i32, y: i32) -> Option<Vaikeus> {
    rivin_nappi(&TASOT, TASO_Y, x, y)
}

fn tason_kuvaus(vaikeus: Vaikeus) -> &'static str {
    match vaikeus {
        Vaikeus::Helppo => "majakka.kuvaus_1",
        Vaikeus::Normaali => "majakka.kuvaus_2",
        Vaikeus::Vaikea => "majakka.kuvaus_3",
    }
}

fn tason_nimi(vaikeus: Vaikeus) -> &'static str {
    match vaikeus {
        Vaikeus::Helppo => "yhteinen.helppo",
        Vaikeus::Normaali => "yhteinen.normaali",
        Vaikeus::Vaikea => "yhteinen.vaikea",
    }
}

/// Kahdeksankulmainen kartio tai lieriö: kyljet alhaalta `y0` ylös `y1`.
fn prisma(tahkot: &mut Vec<Tahko>, (y0, y1): (f32, f32), (r0, r1): (f32, f32), vari: u32) {
    const SIVUJA: usize = 8;
    let kehalla = |i: usize, r: f32, y: f32| {
        let (sin, cos) = (i as f32 / SIVUJA as f32 * TAU).sin_cos();
        [cos * r, y, sin * r]
    };
    for i in 0..SIVUJA {
        let a = kehalla(i, r1, y1);
        let b = kehalla(i + 1, r1, y1);
        let c = kehalla(i + 1, r0, y0);
        let d = kehalla(i, r0, y0);
        if r1 <= 0.0 {
            tahkot.push(tahko4(c, d, a, a, vari));
        } else {
            tahkot.push(tahko4(a, b, c, d, vari));
        }
    }
}

/// Alkuruudun ja valikon malli: kallio, punavalkoinen torni, lamppu ja vartijan mökki.
pub fn majakkamalli() -> Vec<Tahko> {
    let mut tahkot = Vec::new();
    prisma(&mut tahkot, (-0.5, 0.2), (2.7, 2.1), 0x5A4C58);
    const RAITOJA: usize = 5;
    let (ala, yla) = (0.2, 2.9);
    for i in 0..RAITOJA {
        let osuus = |j: usize| j as f32 / RAITOJA as f32;
        let y = |j: usize| ala + (yla - ala) * osuus(j);
        let r = |j: usize| 0.95 - 0.35 * osuus(j);
        let vari = if i % 2 == 0 { 0xE8E8F0 } else { 0xD03838 };
        prisma(&mut tahkot, (y(i), y(i + 1)), (r(i), r(i + 1)), vari);
    }
    prisma(&mut tahkot, (2.9, 3.05), (0.85, 0.85), 0x302838);
    prisma(&mut tahkot, (3.05, 3.4), (0.5, 0.5), KELTAINEN);
    prisma(&mut tahkot, (3.4, 3.75), (0.62, 0.0), 0x902830);
    kuutio([1.1, 2.5], [0.2, 1.1], [-0.7, 0.7], 0xC8B8A0, &mut tahkot);
    let katto = 0x902830;
    let (harja_a, harja_b) = ([1.0, 1.6, 0.0], [2.6, 1.6, 0.0]);
    tahkot.push(tahko4(
        harja_a,
        harja_b,
        [2.6, 1.1, 0.8],
        [1.0, 1.1, 0.8],
        katto,
    ));
    tahkot.push(tahko4(
        harja_a,
        harja_b,
        [2.6, 1.1, -0.8],
        [1.0, 1.1, -0.8],
        katto,
    ));
    tahkot
}

#[derive(Clone, Copy, PartialEq)]
enum Maasto {
    Vesi,
    Satama,
    Laituri,
    Hiekka,
    Maa,
    Katto,
    Seina,
    Ikkuna,
    /// Karikon pinta: valo tulee vasemmalta ylhäältä, -1 varjo ja 1 valoisin.
    Kari(f32),
    /// Vaahto karikon ympärillä tai rannassa, tiheys 0..1.
    Vaahto(f32),
}

fn maasto(x: f32, y: f32, karikot: &[Kari]) -> Maasto {
    if !suulla(x) {
        let ranta = ranta_y(x);
        if y > ranta + 5.0 {
            for &(tx, ty) in &TALOT {
                if (tx..tx + TALON_L).contains(&x) && (ty..ty + TALON_K).contains(&y) {
                    return if y < ty + 3.5 {
                        Maasto::Katto
                    } else if (x - tx - TALON_L / 2.0).abs() < 2.0 && y < ty + 7.0 {
                        Maasto::Ikkuna
                    } else {
                        Maasto::Seina
                    };
                }
            }
            return Maasto::Maa;
        }
        if y > ranta {
            return Maasto::Hiekka;
        }
        if y > ranta - 4.0 {
            return Maasto::Vaahto(0.5);
        }
    } else {
        let reunasta = SUUN_PUOLIKAS - (x - SATAMA.0).abs();
        let reuna_x = SATAMA.0 + (x - SATAMA.0).signum() * SUUN_PUOLIKAS;
        if reunasta < 5.0 && y > ranta_y(reuna_x) - 9.0 {
            return Maasto::Laituri;
        }
        if y > SATAMA.1 {
            return Maasto::Satama;
        }
    }
    for kari in karikot {
        let (dx, dy) = (x - kari.paikka.0, y - kari.paikka.1);
        let d = dx.hypot(dy);
        if d < kari.sade {
            // Kumpare: keskeltä ja vasemmalta ylhäältä valoisin.
            let reuna = d / kari.sade;
            let suunta = -(dx + dy) / (kari.sade * 1.41);
            return Maasto::Kari((suunta * 0.8 + (1.0 - reuna) * 0.6).clamp(-1.0, 1.0));
        }
        if d < kari.sade + 4.0 {
            return Maasto::Vaahto(0.9);
        }
        if d < kari.sade + 9.0 {
            return Maasto::Vaahto(0.35);
        }
    }
    Maasto::Vesi
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Valo {
    Yo,
    Keila,
    Paiva,
}

/// Aallot vinoina raitoina ja satunnaisina kimalluksina.
fn vesi(x: i32, y: i32, valo: Valo) -> u32 {
    let (fx, fy) = (x as f32 * P as f32, y as f32 * P as f32);
    let t = fy / majakka::KORKEUS;
    let pohja = match valo {
        Valo::Yo => liukuvari(&[0x050A26, 0x08103A, 0x0A1644], t, x, y),
        Valo::Keila => liukuvari(&[0x2A5AA8, 0x3068B8, 0x3A74C4], t, x, y),
        // Aamun taivas heijastuu kaukaiseen mereen.
        Valo::Paiva => liukuvari(&[0xB06898, 0x6A70C0, 0x4A70C0, 0x3A68B8], t, x, y),
    };
    let aalto = (fx * 0.045 + fy * 0.12).sin() + 0.6 * (fx * 0.11 - fy * 0.05 + 1.0).sin();
    if aalto > 1.25 {
        return match valo {
            Valo::Yo => 0x0E1848,
            Valo::Keila => 0x6CA0E0,
            Valo::Paiva => sekoita_liuku(pohja, 0xD0D8FF, 0.45),
        };
    }
    if valo != Valo::Yo && kohina(x, y) > 0.985 {
        return 0xD8ECFF;
    }
    pohja
}

fn maaston_vari(maasto: Maasto, x: i32, y: i32, valo: Valo, vaikeus: Vaikeus) -> u32 {
    let kolme = |yo: u32, keila: u32, paiva: u32| match valo {
        Valo::Yo => yo,
        Valo::Keila => keila,
        Valo::Paiva => paiva,
    };
    match maasto {
        Maasto::Vesi => vesi(x, y, valo),
        Maasto::Satama => kolme(0x0C1A4C, 0x3A78C8, 0x6C88D0),
        Maasto::Laituri => kolme(0x1A1830, 0x8C8CA0, 0xA8A8B8),
        Maasto::Hiekka => kolme(0x15122C, 0xC8B078, 0xE0C890),
        Maasto::Maa => {
            let (a, b) = match valo {
                Valo::Yo => (0x090B1C, 0x0D1124),
                Valo::Keila => (0x3C6A34, 0x4A7A3C),
                Valo::Paiva => (0x5A8A48, 0x6C9C54),
            };
            ditteroi(x, y, a, b, kohina(y, x))
        }
        Maasto::Katto => kolme(0x140C1C, 0x8C3C30, 0xA84838),
        Maasto::Seina => kolme(0x161428, 0xC8B8A0, 0xE0D0B8),
        Maasto::Ikkuna => kolme(0xFFD860, 0x405070, 0x6878A0),
        Maasto::Kari(savy) => {
            let osuus = (savy + 1.0) / 2.0;
            match (valo, vaikeus) {
                (Valo::Yo, Vaikeus::Helppo) => liukuvari(&[0x10122C, 0x24243E], osuus, x, y),
                // Pimeässä karikko on meren värinen: vain vaahto paljastaa sen.
                (Valo::Yo, _) => vesi(x, y, valo),
                (Valo::Keila, _) => liukuvari(&[0x3A2E40, 0x7A6A5A, 0xB4A48E], osuus, x, y),
                (Valo::Paiva, _) => liukuvari(&[0x4C3C50, 0x8C7868, 0xC8B8A0], osuus, x, y),
            }
        }
        Maasto::Vaahto(tiheys) => {
            let (vaahto, kerroin) = match (valo, vaikeus) {
                (Valo::Yo, Vaikeus::Helppo) => (0x34488A, 0.8),
                (Valo::Yo, Vaikeus::Normaali) => (0x26346C, 0.55),
                (Valo::Yo, Vaikeus::Vaikea) => (0x1C2858, 0.4),
                _ => (0xE8F4FF, 1.0),
            };
            ditteroi(x, y, vesi(x, y, valo), vaahto, tiheys * kerroin)
        }
    }
}

/// Yön meri kolmena kuvana: pimeänä, keilan valossa ja aamun valossa.
pub struct Merikuva {
    yo: Rasteri,
    keila: Rasteri,
    paiva: Rasteri,
}

impl Merikuva {
    pub fn new(meri: &Meri) -> Self {
        let keskikohta = |i: i32| (i * P) as f32 + P as f32 / 2.0;
        let maastot: Vec<Maasto> = (0..RUUTUJA_K as i32)
            .flat_map(|y| (0..RUUTUJA_L as i32).map(move |x| (x, y)))
            .map(|(x, y)| maasto(keskikohta(x), keskikohta(y), &meri.karikot))
            .collect();
        let kuva = |valo| {
            Rasteri::luo(RUUTUJA_L, RUUTUJA_K, |x, y| {
                let maasto = maastot[y as usize * RUUTUJA_L + x as usize];
                maaston_vari(maasto, x, y, valo, meri.vaikeus)
            })
        };
        Self {
            yo: kuva(Valo::Yo),
            keila: kuva(Valo::Keila),
            paiva: kuva(Valo::Paiva),
        }
    }
}

/// Keilan muoto konepikselin tarkkuudella: reunat ja kantaman pää ditteröityvät pimeään.
struct Keilanalue {
    kulma: (f32, f32),
    kapea: f32,
    levea: f32,
    kantama: f32,
}

impl Keilanalue {
    fn new(meri: &Meri) -> Self {
        let keila = meri.keila;
        Self {
            kulma: (keila.kulma.cos(), keila.kulma.sin()),
            kapea: keila.puolikas.tan(),
            levea: (keila.puolikas * 1.3).tan(),
            kantama: keila.kantama,
        }
    }

    /// Valon osuus konepikselissä 0..1.
    fn osuus(&self, x: f32, y: f32) -> f32 {
        let (dx, dy) = (x - MAJAKKA.0, y - MAJAKKA.1);
        let pitkin = dx * self.kulma.0 + dy * self.kulma.1;
        let poikki = (dx * self.kulma.1 - dy * self.kulma.0).abs();
        let etaisyys2 = dx * dx + dy * dy;
        if etaisyys2 < 100.0 {
            return 1.0;
        }
        if pitkin <= 0.0 || etaisyys2 > self.kantama * self.kantama {
            return 0.0;
        }
        let (kapea, levea) = (pitkin * self.kapea, pitkin * self.levea);
        let sivu = if poikki <= kapea {
            1.0
        } else if poikki < levea {
            (levea - poikki) / (levea - kapea)
        } else {
            return 0.0;
        };
        let lahella = self.kantama * 0.7;
        if etaisyys2 <= lahella * lahella {
            return sivu;
        }
        sivu * (self.kantama - etaisyys2.sqrt()) / (self.kantama - lahella)
    }
}

/// Paikallisen pisteen paikka ruudulla, kun kappale on käännetty kulmaan.
fn ruudulle(keski: (i32, i32), kulma: f32, (x, y): Piste) -> (i32, i32) {
    let (sin, cos) = kulma.sin_cos();
    (
        keski.0 + (x * cos - y * sin).round() as i32,
        keski.1 + (x * sin + y * cos).round() as i32,
    )
}

fn kentalle(p: Piste) -> (i32, i32) {
    (KENTTA_X + p.0 as i32, KENTTA_Y + p.1 as i32)
}

/// Rungon muoto keula +x-suuntaan, vasen kylki -y. Mitat kentän pikseleitä.
fn rungon_muoto(laji: Laji) -> &'static [Piste] {
    match laji {
        Laji::Vene => &[
            (8.0, 0.0),
            (3.0, 3.5),
            (-6.0, 3.5),
            (-6.0, -3.5),
            (3.0, -3.5),
        ],
        Laji::Kalastaja => &[
            (11.0, 0.0),
            (5.0, 4.5),
            (-9.0, 4.5),
            (-9.0, -4.5),
            (5.0, -4.5),
        ],
        Laji::Rahtilaiva => &[
            (16.0, 0.0),
            (10.0, 6.5),
            (-14.0, 6.5),
            (-14.0, -6.5),
            (10.0, -6.5),
        ],
        Laji::Salakuljettaja => &[
            (11.0, 0.0),
            (4.0, 3.5),
            (-8.0, 3.5),
            (-8.0, -3.5),
            (4.0, -3.5),
        ],
    }
}

fn rungon_vari(laji: Laji) -> u32 {
    match laji {
        Laji::Vene => 0x9C6438,
        Laji::Kalastaja => 0xC83830,
        Laji::Rahtilaiva => 0x3C4C78,
        Laji::Salakuljettaja => 0x404050,
    }
}

pub fn piirra(peli: &Majakka, kuva: &mut [u32]) {
    let mut p = Piirtaja {
        k: Kangas::new(kuva),
        kello: peli.kello_ms,
    };
    match (&peli.kuva, peli.vaihe) {
        (Some(merikuva), vaihe) if vaihe != Vaihe::Alku => p.majakan_nakyma(peli, merikuva),
        _ => p.majakan_alkuruutu(peli),
    }
    p.aanikytkin(peli.aanet.paalla);
}

impl Piirtaja<'_> {
    fn majakan_alkuruutu(&mut self, peli: &Majakka) {
        self.maisema(kaanna("majakka.banneri"), &majakkamalli());
        let otsikko = listan_otsikko(peli.taulu.netti.is_some());
        self.parhaat_alkuruudulla(otsikko, peli.taulu.nakyva());
        let y = self.alkuohje(&ALKU_OSAT, &ALKU_RIVIT) + RIVIN_KORKEUS / 2;
        self.teksti(48, y, kaanna(tason_kuvaus(peli.vaikeus)), KELTAINEN);
        self.teksti(48, TASO_Y + 3, kaanna("yhteinen.taso"), KELTAINEN);
        for (taso, teksti, x) in TASOT {
            if taso == peli.vaikeus {
                self.nappi(x, TASO_Y, teksti, WB_ORANSSI, MUSTA);
            } else {
                self.nappi(x, TASO_Y, teksti, VALKOINEN, WB_SININEN);
            }
        }
        self.aloituskehote(ALOITUS);
    }

    fn majakan_nakyma(&mut self, peli: &Majakka, merikuva: &Merikuva) {
        let meri = &peli.meri;
        self.majakan_meri(meri, merikuva);
        self.karikon_vaahto(meri);
        self.majakan_satamavalot();
        for laiva in &meri.laivat {
            self.majakan_laiva(laiva);
        }
        self.majakan_torni(meri.valoisuus());
        for tehoste in &peli.tehosteet {
            self.majakan_tehoste(tehoste);
        }
        self.tausta_kentan_ympari(KENTTA_X, KENTTA_Y, KENTTA_L, KENTTA_K);
        self.upotus(KENTTA_X, KENTTA_Y, KENTTA_L, KENTTA_K);
        let otsikko = kaanna("peli.majakka").to_uppercase();
        self.otsikkopalkki(&otsikko, kaanna(tason_nimi(meri.vaikeus)));
        self.majakan_tilasto(peli);
        self.majakan_paneeli(peli);
    }

    /// Joka konepikseli valitaan yön, keilan ja päivän kuvista.
    fn majakan_meri(&mut self, meri: &Meri, kuva: &Merikuva) {
        let valoisuus = meri.valoisuus();
        let keila = Keilanalue::new(meri);
        let mut rivi = [0; RUUTUJA_L];
        for y in 0..RUUTUJA_K {
            let (yo, valo, paiva) = (kuva.yo.rivi(y), kuva.keila.rivi(y), kuva.paiva.rivi(y));
            let keski_y = (y as i32 * P) as f32 + P as f32 / 2.0;
            for (x, kohde) in rivi.iter_mut().enumerate() {
                let (xi, yi) = (x as i32, y as i32);
                let mut vari = if valoisuus <= 0.0 {
                    yo[x]
                } else if valoisuus >= 1.0 {
                    paiva[x]
                } else {
                    ditteroi(xi, yi, yo[x], paiva[x], valoisuus)
                };
                if valoisuus < 1.0 {
                    let keski_x = (xi * P) as f32 + P as f32 / 2.0;
                    let osuus = keila.osuus(keski_x, keski_y);
                    if osuus > 0.0 {
                        vari = ditteroi(xi, yi, vari, valo[x], osuus);
                    }
                }
                *kohde = vari;
            }
            self.k.karkea_rivi(KENTTA_X, KENTTA_Y + y as i32 * P, &rivi);
        }
    }

    /// Pimeässä karikon paljastaa vain välähtelevä vaahto. Kuvio vaihtuu harvakseltaan,
    /// jottei koko meri muutu joka ruudussa.
    fn karikon_vaahto(&mut self, meri: &Meri) {
        if meri.valoisuus() >= 1.0 {
            return;
        }
        let (maara, vari) = match meri.vaikeus {
            Vaikeus::Helppo => (5, 0x7088D0),
            Vaikeus::Normaali => (4, 0x5870B8),
            Vaikeus::Vaikea => (2, 0x3C5090),
        };
        let askel = (self.kello / VAAHDON_ASKEL_MS) as i32;
        for (i, kari) in meri.karikot.iter().enumerate() {
            for j in 0..maara {
                let kulma = kohina(askel * 31 + i as i32 * 7, j * 13) * TAU;
                let sade = kari.sade + 2.0 + kohina(askel, i as i32 * 5 + j) * 4.0;
                let paikka = (
                    kari.paikka.0 + kulma.cos() * sade,
                    kari.paikka.1 + kulma.sin() * sade,
                );
                let (x, y) = kentalle(paikka);
                self.k.piste(x, y, vari);
            }
        }
    }

    /// Satamansuun punainen ja vihreä valo vilkkuvat vuorotellen.
    fn majakan_satamavalot(&mut self) {
        let vuoro = (self.kello / SATAMAVALON_VILKKU_MS).is_multiple_of(2);
        for (puoli, vari) in [(-1.0, 0xFF4040), (1.0, 0x40FF70)] {
            let x = SATAMA.0 + puoli * (SUUN_PUOLIKAS - 2.0);
            let (px, py) = kentalle((x, ranta_y(x) - 8.0));
            let vari = if vuoro == (puoli < 0.0) {
                vari
            } else {
                tummenna(vari, 0.5)
            };
            self.k.laatikko(px - P, py - P, 2 * P, 2 * P, vari);
        }
    }

    fn majakan_laiva(&mut self, laiva: &Laiva) {
        let laji = laiva.laji;
        let keski = kentalle(laiva.paikka);
        let kulma = laiva.suunta;
        let piste = |p: Piste| ruudulle(keski, kulma, p);
        let muoto = rungon_muoto(laji);
        let peralla = muoto.iter().map(|p| p.0).fold(0.0, f32::min);
        let valossa = laiva.valaistu;
        // Vana näkyy pimeässäkin heikosti: salakuljettajan ainoa merkki.
        let vanan_vari = if valossa { 0xC8DCF8 } else { 0x1C2C64 };
        let vaihe = (self.kello / VANAN_VILKKU_MS) % 2;
        for i in 1..=3 {
            let taakse = peralla - 3.0 - i as f32 * 4.0;
            let leveys = 1.5 + i as f32 * 1.5 + vaihe as f32;
            for sivu in [-1.0, 1.0] {
                let (x, y) = piste((taakse, sivu * leveys));
                self.k.piste(x, y, vanan_vari);
            }
        }
        if laji == Laji::Salakuljettaja && !valossa {
            return;
        }
        let runko = if valossa { rungon_vari(laji) } else { 0x111A3E };
        let ulko: Vec<(i32, i32)> = muoto
            .iter()
            .map(|&(x, y)| piste((x * 1.15, y * 1.25)))
            .collect();
        self.k.monikulmio(
            &ulko,
            if valossa {
                tummenna(runko, 0.6)
            } else {
                0x0A0F28
            },
        );
        let kulmat: Vec<(i32, i32)> = muoto.iter().map(|&p| piste(p)).collect();
        self.k.monikulmio(&kulmat, runko);
        if valossa {
            self.majakan_kansi(laji, &piste);
        }
        if laji != Laji::Salakuljettaja {
            // Kulkuvalot: punainen vasemmalla, vihreä oikealla ja valkoinen masto keskellä.
            let keula = muoto[1];
            let (x, y) = piste((keula.0, -keula.1));
            self.k.piste(x, y, 0xFF3838);
            let (x, y) = piste(keula);
            self.k.piste(x, y, 0x38FF60);
            let (x, y) = piste((0.0, 0.0));
            self.k.piste(x, y, 0xFFF4B0);
        }
        if laiva.tietaa() && laji != Laji::Salakuljettaja {
            // Kapteeni tietää reitin: kurssi näkyy keulan edessä pisteinä.
            let keula = muoto[0].0;
            for i in 1..=3 {
                let (x, y) = piste((keula + 3.0 + i as f32 * 6.0, 0.0));
                self.k.piste(x, y, 0xFFF0A0);
            }
        }
        if laji == Laji::Salakuljettaja && laiva.keilassa_ms > 0 {
            const KAAREN_PISTEITA: u32 = 16;
            let osuus = laiva.keilassa_ms.min(KIINNIOTTO_MS) * KAAREN_PISTEITA / KIINNIOTTO_MS;
            for i in 0..osuus {
                let kulma = -PI / 2.0 + i as f32 / KAAREN_PISTEITA as f32 * TAU;
                let (x, y) = (
                    keski.0 + (kulma.cos() * 18.0) as i32,
                    keski.1 + (kulma.sin() * 18.0) as i32,
                );
                self.k.piste(x, y, KELTAINEN);
            }
        }
    }

    /// Valaistun laivan kansi: kajuutta, kontit ja tuulilasi.
    fn majakan_kansi(&mut self, laji: Laji, piste: &impl Fn(Piste) -> (i32, i32)) {
        let mut laatikko = |x0: f32, x1: f32, puoli: f32, vari: u32| {
            let kulmat = [(x0, -puoli), (x1, -puoli), (x1, puoli), (x0, puoli)].map(piste);
            self.k.monikulmio(&kulmat, vari);
        };
        match laji {
            Laji::Vene => laatikko(-1.0, 1.0, 3.0, 0x6A4028),
            Laji::Kalastaja => laatikko(-6.0, 1.0, 3.0, 0xF0F0F0),
            Laji::Rahtilaiva => {
                for (i, vari) in [WB_ORANSSI, 0x3878E8, 0x58C850].into_iter().enumerate() {
                    let x = 7.0 - i as f32 * 5.0;
                    laatikko(x - 4.0, x, 5.0, vari);
                }
                laatikko(-13.0, -9.0, 5.0, 0xF0F0F0);
            }
            Laji::Salakuljettaja => laatikko(1.0, 3.0, 3.0, 0x88A0B8),
        }
    }

    fn majakan_torni(&mut self, valoisuus: f32) {
        #[rustfmt::skip]
        const TORNI: [&str; 11] = [
            "..KKK..", ".KYYYK.", "KKKKKKK", "..WWW..", "..RRR..", "..WWW..",
            ".RRRRR.", ".WWWWW.", ".RRRRR.", "DDDDDDD", "DDDDDDD",
        ];
        let lamppu = if (self.kello / LAMPUN_VILKKU_MS).is_multiple_of(2) {
            0xFFFAD0
        } else {
            0xFFD848
        };
        let vari = |yo: u32, paiva: u32| sekoita_liuku(yo, paiva, valoisuus);
        let (x, y) = kentalle((MAJAKKA.0 - 10.5, MAJAKKA.1 - 4.5));
        self.k.sprite((x, y), &TORNI, P, |merkki| {
            Some(match merkki {
                b'K' => vari(0x201828, 0x302838),
                b'Y' => lamppu,
                b'W' => vari(0x6C6C90, 0xF0F0F8),
                b'R' => vari(0x6A2030, 0xD03838),
                b'D' => vari(0x1C1830, 0x6A5A50),
                _ => return None,
            })
        });
        let keski = kentalle(MAJAKKA);
        self.k.ympyra(keski, 9.0, vari(0xC0A040, 0xFFF0B0));
    }

    fn majakan_tehoste(&mut self, tehoste: &Tehoste) {
        let ika = self.kello.saturating_sub(tehoste.alku) as f32 / TEHOSTE_MS as f32;
        let keski = kentalle(tehoste.paikka);
        match tehoste.laji {
            Tehostelaji::Pisteet(pisteet) => {
                let teksti = format!("+{pisteet}");
                let leveys = teksti.chars().count() as i32 * 16;
                let y = keski.1 - 24 - (ika * 36.0) as i32;
                self.teksti_koossa(keski.0 - leveys / 2, y, &teksti, 2, KELTAINEN);
            }
            Tehostelaji::Haaksirikko => {
                let sade = 4.0 + ika * 22.0;
                self.k
                    .ympyra(keski, sade, sekoita_liuku(0xFFFFFF, 0x26346C, ika));
                self.k
                    .ympyra(keski, sade * 0.6, sekoita_liuku(0xC8DCF8, 0x26346C, ika));
                for i in 0..7 {
                    let kulma = i as f32 / 7.0 * TAU + 0.4;
                    let matka = 4.0 + ika * (14.0 + (i % 3) as f32 * 5.0);
                    let (x, y) = (
                        keski.0 + (kulma.cos() * matka) as i32,
                        keski.1 + (kulma.sin() * matka) as i32,
                    );
                    self.k.piste(x, y, 0x8C5A30);
                }
            }
            Tehostelaji::Kiinni => {
                self.k.ympyra(keski, 10.0 + ika * 30.0, KELTAINEN);
            }
        }
    }

    fn majakan_tilasto(&mut self, peli: &Majakka) {
        let meri = &peli.meri;
        let lohko = |i: i32| KENTTA_Y + i * TILASTON_ASKEL;
        for (i, (otsikko, arvo)) in [
            (kaanna("yhteinen.pisteet"), meri.pisteet.to_string()),
            (kaanna("yhteinen.paras"), peli.taulu.paras.to_string()),
            (kaanna("majakka.yo_otsikko"), (meri.yo + 1).to_string()),
            (kaanna("majakka.ketju"), format!("x{}", meri.ketjukerroin())),
        ]
        .into_iter()
        .enumerate()
        {
            let y = lohko(i as i32);
            self.suurteksti(SARAKE_X, y, otsikko, OTSIKKO, KELTAINEN, false);
            self.suurteksti(SARAKE_X, y + ARVON_Y, &arvo, OTSIKKO, VALKOINEN, true);
        }

        // Yön kulku: kuu vasemmalla, aamu oikealla.
        let y = lohko(2) + ARVON_Y + 3;
        let x = SARAKE_X + 4 * OTSIKKO.2;
        let leveys = SARAKE_X + PARHAAT_L - x;
        self.majakan_mittari(x, y, leveys, meri.yon_osuus(), (0x5848A0, 0xFFB070));

        let y = lohko(4);
        self.suurteksti(
            SARAKE_X,
            y,
            kaanna("majakka.renkaat"),
            OTSIKKO,
            KELTAINEN,
            false,
        );
        for i in 0..majakka::RENKAITA {
            let keski = (
                SARAKE_X + RENKAAN_SADE + i as i32 * (2 * RENKAAN_SADE + 12),
                y + ARVON_Y + RENKAAN_SADE,
            );
            self.majakan_rengas(keski, i < meri.renkaat);
        }

        let y = lohko(5);
        self.suurteksti(
            SARAKE_X,
            y,
            kaanna("majakka.torvi"),
            OTSIKKO,
            KELTAINEN,
            false,
        );
        let vari = if meri.torvi_ms > 0 {
            WB_ORANSSI
        } else if meri.lataus_ms == 0 {
            KELTAINEN
        } else {
            HUTI
        };
        let valmius = if meri.torvi_ms > 0 {
            meri.torvi_ms as f32 / majakka::TORVEN_KESTO_MS as f32
        } else {
            meri.torven_valmius()
        };
        self.majakan_mittari(SARAKE_X, y + ARVON_Y + 3, PARHAAT_L, valmius, (vari, vari));

        self.parhaat_lista(
            SARAKE_X,
            PARHAAT_Y,
            PARHAAT_L,
            PARHAAT_KOKO,
            None,
            listan_otsikko(peli.taulu.netti.is_some()),
            peli.taulu.nakyva(),
        );
    }

    /// Upotettu palkki, jonka täyttö liukuu värista toiseen.
    fn majakan_mittari(
        &mut self,
        x: i32,
        y: i32,
        leveys: i32,
        osuus: f32,
        (alku, loppu): (u32, u32),
    ) {
        self.k.laatikko(x, y, leveys, MITTARI_K, 0x0E0830);
        self.upotus(x, y, leveys, MITTARI_K);
        let taysi = ((leveys as f32 * osuus.clamp(0.0, 1.0)) as i32) / P * P;
        if taysi > 0 {
            let vari = sekoita_liuku(alku, loppu, osuus.clamp(0.0, 1.0));
            self.k.laatikko(x, y, taysi, MITTARI_K, vari);
            self.k.laatikko(x, y, taysi, P, vaalenna(vari, 0.5));
        }
    }

    /// Pelastusrengas: valkoinen ja punainen kehä. Menetetty näkyy tummana.
    fn majakan_rengas(&mut self, keski: (i32, i32), jaljella: bool) {
        let (valkoinen, punainen) = if jaljella {
            (0xF0F0F8, 0xE04030)
        } else {
            (0x3C3460, 0x2C2450)
        };
        self.k.kiekko(keski, RENKAAN_SADE, valkoinen);
        for kulma in [0.0, PI / 2.0, PI, 3.0 * PI / 2.0] {
            let (x, y) = (
                keski.0 + (kulma.cos() * 8.0) as i32,
                keski.1 + (kulma.sin() * 8.0) as i32,
            );
            self.k.laatikko(x - P, y - P, 2 * P, 2 * P, punainen);
        }
        self.k.kiekko(keski, 5, TAUSTA);
    }

    fn majakan_paneeli(&mut self, peli: &Majakka) {
        self.paneelin_tausta();
        // Nimikysely tarvitsee paneelin kolme alinta riviä ja otsikon, joten viestejä mahtuu yksi.
        let viesteja = if peli.vaihe == Vaihe::Nimi {
            1
        } else {
            VIESTEJA
        };
        for (i, viesti) in peli.viestit.iter().rev().take(viesteja).enumerate() {
            let vari = if i == 0 { VALKOINEN } else { LAVENTELI };
            let y = PANEELI_RIVI_Y + i as i32 * RIVIN_KORKEUS;
            self.teksti(TEKSTI_X, y, viesti, vari);
        }

        let alarivi = PANEELI_RIVI_Y + (PANEELI_RIVEJA as i32 - 1) * RIVIN_KORKEUS;
        let otsikko_y = alarivi - RIVIN_KORKEUS - 12;
        match peli.vaihe {
            Vaihe::Pelissa => self.teksti(TEKSTI_X, alarivi, kaanna(OHJE), HUTI),
            Vaihe::Tauko => self.tauko(),
            Vaihe::Tappio => {
                self.suurteksti(TEKSTI_X, otsikko_y, kaanna(TAPPIO), TULOS, KELTAINEN, true);
                self.loppunapit();
            }
            Vaihe::Nimi => self.nimikysely(peli.meri.pisteet, &peli.nimi),
            Vaihe::Alku => {}
        }
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
pub use tests::vaikeuden_keskipiste;
