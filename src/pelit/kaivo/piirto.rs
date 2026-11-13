//! Kaivo8000:n näkymät. Kaivos lasketaan kahtena valmiina kuvana (valossa ja pimeässä) aina,
//! kun jokin ruutu muuttuu, ja joka ruudussa niistä vain valitaan pikselit lampun etäisyyden
//! mukaan: valon reuna on ditteröity raja kuvien välillä, joten valaistus ei maksa juuri mitään.

use std::f32::consts::TAU;

use klassikot8000::kaivo::{
    HISSI, HISSIN_MATKA_MS, KYPARIA, Kaivos, MAA_ALKAA, PUTOAMINEN_MS, RIVEJA, Ruutu, SARAKKEITA,
    Solu, Toimi, Vaikeus,
};

use super::peli::{Kaivo, TEHOSTE_MS, Tehoste, Tehostelaji, VIESTEJA, Vaihe};
use crate::yhteinen::kieli::{kaanna, muotoile};
use crate::yhteinen::piirto::*;
use crate::yhteinen::rasteri::{Rasteri, ditteroi, kohina, liukuvari};
use crate::yhteinen::vektori::{Kangas, Tahko, kuutio, sekoita_liuku, tahko4, tummenna, vaalenna};

pub const KENTTA_X: i32 = 30;
pub const KENTTA_Y: i32 = 48;
/// Ruudun koko pikseleinä: kymmenen konepikseliä.
pub const SOLU: i32 = 30;
/// Konepikseli klassisessa kuvassa.
const P: i32 = 3;
const SOLU_P: i32 = SOLU / P;
const KENTTA_L: i32 = SARAKKEITA as i32 * SOLU;
const KENTTA_K: i32 = RIVEJA as i32 * SOLU;
const RUUTUJA_L: usize = SARAKKEITA * SOLU_P as usize;
const RUUTUJA_K: usize = RIVEJA * SOLU_P as usize;
/// Taivas ja ruohon reuna ovat aina valossa.
const VALOISA_RAJA: usize = MAA_ALKAA as usize * SOLU_P as usize + 3;
const SARAKE_X: i32 = KENTTA_X + KENTTA_L + 30;
const TILASTON_ASKEL: i32 = 54;
const ARVON_Y: i32 = 25;
const PARHAAT_Y: i32 = KENTTA_Y + 8 * TILASTON_ASKEL;
const PARHAAT_L: i32 = LEVEYS as i32 - 12 - SARAKE_X;
// Kapeaan sarakkeeseen mahtuu sijoitus, kahdeksan merkin nimi ja kuusinumeroiset pisteet.
const PARHAAT_KOKO: (i32, i32, i32) = (10, 15, 13);
const MITTARI_K: i32 = 15;

const OHJE: &str = "kaivo.ohje";
const TAPPIO: &str = "yhteinen.peli_paattyi";

// Alkuruudun tasovalinta samoilla mitoilla kuin majakassa.
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
    "kaivo.tervetuloa",
    "yhteinen.ohjeet",
    "kaivo.ohje_1",
    "kaivo.ohje_2",
];
const ALKU_RIVIT: [&str; 4] = [
    "kaivo.nappain_kaiva",
    "kaivo.nappain_tuki",
    "yhteinen.tauko_aanet",
    "yhteinen.takaisin",
];

const KAVELYN_TAHTI_MS: u32 = 60;
const KUOKAN_TAHTI_MS: u32 = 90;
const LEKOTUS_MS: u32 = 90;
const KIMALLUS_MS: u32 = 180;
const POLYN_TAHTI_MS: u32 = 30;
/// Näin vähän ennen putoamista ritisevä ruutu vilkkuu varoituksena.
const VAROITUS_MS: u32 = 400;
const VAROITUKSEN_VILKKU_MS: u32 = 100;
const HISSIVALON_VILKKU_MS: u32 = 400;

pub fn vaikeus_pisteesta(x: i32, y: i32) -> Option<Vaikeus> {
    rivin_nappi(&TASOT, TASO_Y, x, y)
}

/// Kentän ruutu kuvan pisteessä.
pub fn ruutu_pisteesta(x: i32, y: i32) -> Option<Ruutu> {
    let (kx, ky) = (x - KENTTA_X, y - KENTTA_Y);
    ((0..KENTTA_L).contains(&kx) && (0..KENTTA_K).contains(&ky)).then_some((kx / SOLU, ky / SOLU))
}

fn tason_kuvaus(vaikeus: Vaikeus) -> &'static str {
    match vaikeus {
        Vaikeus::Helppo => "kaivo.kuvaus_1",
        Vaikeus::Normaali => "kaivo.kuvaus_2",
        Vaikeus::Vaikea => "kaivo.kuvaus_3",
    }
}

fn tason_nimi(vaikeus: Vaikeus) -> &'static str {
    match vaikeus {
        Vaikeus::Helppo => "yhteinen.helppo",
        Vaikeus::Normaali => "yhteinen.normaali",
        Vaikeus::Vaikea => "yhteinen.vaikea",
    }
}

/// Alkuruudun ja valikon malli: kultaa täynnä oleva kaivosvaunu kiskoilla.
pub fn kaivosmalli() -> Vec<Tahko> {
    let mut tahkot = Vec::new();
    for z in [-0.95, 0.95] {
        kuutio(
            [-3.4, 3.4],
            [-0.35, -0.2],
            [z - 0.12, z + 0.12],
            0x9C9CB0,
            &mut tahkot,
        );
    }
    for x in [-2.6, -0.9, 0.9, 2.6] {
        kuutio(
            [x - 0.3, x + 0.3],
            [-0.5, -0.35],
            [-1.4, 1.4],
            0x6A4428,
            &mut tahkot,
        );
    }
    for x in [-1.3, 1.3] {
        for z in [-1.05, 1.05] {
            kuutio(
                [x - 0.45, x + 0.45],
                [-0.2, 0.7],
                [z - 0.12, z + 0.12],
                0x302838,
                &mut tahkot,
            );
        }
    }
    let ala = |x: f32, z: f32| [x, 0.45, z];
    let yla = |x: f32, z: f32| [x, 2.0, z];
    let (ax, az, yx, yz) = (1.6, 0.95, 2.1, 1.3);
    let kori = 0xA84C34;
    tahkot.push(tahko4(
        ala(-ax, -az),
        ala(ax, -az),
        yla(yx, -yz),
        yla(-yx, -yz),
        kori,
    ));
    tahkot.push(tahko4(
        ala(-ax, az),
        ala(ax, az),
        yla(yx, yz),
        yla(-yx, yz),
        kori,
    ));
    tahkot.push(tahko4(
        ala(-ax, -az),
        ala(-ax, az),
        yla(-yx, yz),
        yla(-yx, -yz),
        0x8C3C2C,
    ));
    tahkot.push(tahko4(
        ala(ax, -az),
        ala(ax, az),
        yla(yx, yz),
        yla(yx, -yz),
        0x8C3C2C,
    ));
    for (x, z, koko) in [
        (-1.2, -0.5, 0.55),
        (-0.3, 0.45, 0.6),
        (0.7, -0.35, 0.55),
        (1.35, 0.5, 0.45),
        (0.15, -0.05, 0.7),
        (-1.1, 0.6, 0.4),
    ] {
        let puoli = koko / 2.0;
        kuutio(
            [x - puoli, x + puoli],
            [1.75, 2.05 + koko],
            [z - puoli, z + puoli],
            0xF0C030,
            &mut tahkot,
        );
    }
    tahkot
}

// ---- kaivoksen kuva ----

fn syvyys(rivi: i32) -> f32 {
    (rivi - MAA_ALKAA) as f32 / (RIVEJA as i32 - 1 - MAA_ALKAA) as f32
}

fn maan_vari(rivi: i32, gx: i32, gy: i32) -> u32 {
    let pohja = liukuvari(
        &[0xA46C3C, 0x8A523A, 0x6C3E42, 0x563254],
        syvyys(rivi),
        gx,
        gy,
    );
    let k = kohina(gx, gy);
    if k > 0.93 {
        vaalenna(pohja, 0.22)
    } else if k < 0.06 {
        tummenna(pohja, 0.3)
    } else {
        pohja
    }
}

/// Tunnelin takaseinä: maata tummempi ja kylmempi, jotta onkalo erottuu valossa.
fn seinan_vari(rivi: i32, gx: i32, gy: i32) -> u32 {
    let pohja = liukuvari(&[0x6A4440, 0x5A3A44, 0x4A324E], syvyys(rivi), gx, gy);
    // Vaakasuorat kerrosraidat ja irtokivet.
    let raita = gy + (kohina(gx / 4, rivi + 90) * 2.0) as i32;
    if raita.rem_euclid(5) == 0 {
        tummenna(pohja, 0.25)
    } else if kohina(gx + 400, gy) > 0.92 {
        vaalenna(pohja, 0.12)
    } else {
        pohja
    }
}

fn taivaan_vari(gx: i32, gy: i32) -> u32 {
    let t = gy as f32 / VALOISA_RAJA as f32;
    let (fx, fy) = (gx as f32, gy as f32);
    let lahi = 16.5 + 1.8 * (fx * 0.09 + 1.0).sin() + 1.2 * (fx * 0.23).sin();
    let kauka = 12.5 + 2.5 * (fx * 0.05).sin() + 1.5 * (fx * 0.13 + 2.0).sin();
    if fy > lahi {
        ditteroi(gx, gy, 0x1E1438, 0x261A44, kohina(gx, gy))
    } else if fy > kauka {
        ditteroi(gx, gy, 0x3A2660, 0x44306C, kohina(gx, gy))
    } else if gy < 9 && kohina(gx, gy + 500) > 0.975 {
        0xE8E0FF
    } else {
        liukuvari(&[0x18104A, 0x3C2470, 0x8C3C78, 0xE07A5A], t, gx, gy)
    }
}

/// Pieni pikselikuva ruudun sisällä kohdassa `(x0, y0)`.
fn kuvio(kuva: &[&str], px: i32, py: i32, (x0, y0): (i32, i32)) -> Option<u8> {
    let rivi = kuva.get(usize::try_from(py - y0).ok()?)?;
    let merkki = *rivi.as_bytes().get(usize::try_from(px - x0).ok()?)?;
    (merkki != b'.').then_some(merkki)
}

#[rustfmt::skip]
const TIMANTTI: [&str; 4] = [".WWCC.", "WCCCCB", ".CCCB.", "..CB.."];
#[rustfmt::skip]
const LYHTY: [&str; 7] = [".KK.", "K..K", "KYYK", "YWYY", "YYYY", "KYYK", "KKKK"];
const NUGETIT: [(i32, i32); 4] = [(1, 2), (6, 1), (3, 6), (7, 6)];
const PUUT: [(f32, f32); 3] = [(2.5, 6.5), (6.5, 6.5), (4.5, 3.0)];

/// Materiaalin väri valossa ruudun konepikselissä `(px, py)`. `None` on läpinäkyvä: sen
/// kohdalla näkyy tunnelin seinä.
fn materiaali(solu: Solu, rivi: i32, (px, py): (i32, i32), (gx, gy): (i32, i32)) -> Option<u32> {
    let maa = || maan_vari(rivi, gx, gy);
    Some(match solu {
        Solu::Tyhja => return None,
        Solu::Maa => maa(),
        Solu::Savi => {
            let pohja = liukuvari(&[0xC4683E, 0x9C4838], syvyys(rivi), gx, gy);
            let raita = gy + (kohina(gx / 5, rivi) * 3.0) as i32;
            if raita.rem_euclid(4) == 0 {
                tummenna(pohja, 0.2)
            } else {
                pohja
            }
        }
        Solu::Hiekka => {
            let k = kohina(gx, gy);
            if k > 0.9 {
                0xF8E8B8
            } else if k < 0.06 {
                0xA88850
            } else {
                ditteroi(gx, gy, 0xE6C888, 0xD0AC6C, k)
            }
        }
        Solu::Kivi => {
            let (dx, dy) = (px as f32 - 4.5, py as f32 - 4.5);
            let d = dx.hypot(dy);
            if d > 4.7 {
                return None;
            }
            if d > 3.9 {
                0x3A3048
            } else {
                let valo = (0.5 - (dx + dy) / 11.0).clamp(0.0, 1.0);
                liukuvari(&[0x5C5470, 0x948CA8, 0xD0C8DC], valo, gx, gy)
            }
        }
        Solu::Kallio => {
            if px == 0 || py == 0 || kohina(gx / 2 + 50, gy / 2) > 0.9 {
                0x30304E
            } else {
                ditteroi(gx, gy, 0x54587E, 0x484C70, kohina(gx, gy))
            }
        }
        Solu::Kulta => {
            let peili = kohina(gx / SOLU_P, rivi) > 0.5;
            let px = if peili { SOLU_P - 2 - px } else { px };
            let nuggetti = NUGETIT.iter().find_map(|&(nx, ny)| {
                let (x, y) = (px - nx, py - ny);
                ((0..2).contains(&x) && (0..2).contains(&y)).then_some((x, y))
            });
            match nuggetti {
                Some((0, 0)) => 0xFFF4B0,
                Some((1, 1)) => 0xB87820,
                Some(_) => 0xFFD040,
                None => maa(),
            }
        }
        Solu::Timantti => match kuvio(&TIMANTTI, px, py, (2, 3)) {
            Some(b'W') => 0xFFFFFF,
            Some(b'C') => 0x68E0F8,
            Some(_) => 0x2878C8,
            None => maa(),
        },
        Solu::Oljy => match kuvio(&LYHTY, px, py, (3, 1)) {
            Some(b'K') => 0x3A2A2A,
            Some(b'W') => 0xFFFFE0,
            Some(_) => 0xFFC840,
            None => maa(),
        },
        Solu::Puut => {
            let tukki = PUUT
                .iter()
                .map(|&(x, y)| (px as f32 + 0.5 - x).hypot(py as f32 + 0.5 - y))
                .fold(f32::MAX, f32::min);
            if tukki < 0.9 {
                0xA06838
            } else if tukki < 1.8 {
                0xE0B070
            } else if tukki < 2.3 {
                0x5A3418
            } else {
                maa()
            }
        }
        Solu::Tuki => match (px, py) {
            (_, 0) => 0xD8A060,
            (_, 1) => 0x8A5A2C,
            (1 | 7, _) => 0xC8904C,
            (2 | 8, _) => 0x8A5A2C,
            (3..=6, 5) => 0xA87038,
            (3..=6, 6) => 0x5A3418,
            _ => return None,
        },
        // Vaaleat johteet ja puolat ilman tuen ylintä palkkia, jotta ne erottuvat tuesta.
        // Puolat jatkuvat ruudusta toiseen samassa tahdissa.
        Solu::Tikkaat => match (px, py) {
            (2 | 7, _) => 0xE8C888,
            (3 | 8, _) => 0x8A6A3C,
            (4..=6, 2 | 7) => 0xD0A868,
            (4..=6, 3 | 8) => 0x4A3420,
            _ => return None,
        },
    })
}

/// Kaivaja näkee tämän läpi: tunnelin seinä näkyy.
fn avoin(solu: Solu) -> bool {
    matches!(solu, Solu::Tyhja | Solu::Tuki | Solu::Tikkaat)
}

/// Läpinäkymättömän maan reunat: valo ylhäältä, varjo alas, ruoho pinnalla.
fn reunat(
    vari: u32,
    solu: Solu,
    (px, py): (i32, i32),
    (gx, gy): (i32, i32),
    naapuri: impl Fn(i32, i32) -> Solu,
) -> u32 {
    if matches!(solu, Solu::Kivi | Solu::Tuki | Solu::Tikkaat) {
        return vari;
    }
    let rivi = gy / SOLU_P;
    let pinnalla = rivi == MAA_ALKAA && naapuri(0, -1) == Solu::Tyhja && solu != Solu::Kallio;
    if pinnalla && (py < 2 || (py == 2 && kohina(gx, gy) > 0.5)) {
        return ditteroi(gx, gy, 0x3C8A3C, 0x58B048, kohina(gx + 7, gy));
    }
    if py == 0 && avoin(naapuri(0, -1)) {
        return vaalenna(vari, 0.28);
    }
    if py == SOLU_P - 1 && avoin(naapuri(0, 1)) {
        return tummenna(vari, 0.45);
    }
    if px == SOLU_P - 1 && avoin(naapuri(1, 0)) {
        return tummenna(vari, 0.3);
    }
    if px == 0 && avoin(naapuri(-1, 0)) {
        return vaalenna(vari, 0.1);
    }
    vari
}

fn pikselin_vari(kaivos: &Kaivos, gx: i32, gy: i32) -> u32 {
    let (x, y) = (gx / SOLU_P, gy / SOLU_P);
    let (px, py) = (gx % SOLU_P, gy % SOLU_P);
    if y < MAA_ALKAA {
        return taivaan_vari(gx, gy);
    }
    let solu = kaivos.solu((x, y));
    let naapuri = |dx: i32, dy: i32| kaivos.solu((x + dx, y + dy));
    if let Some(vari) = materiaali(solu, y, (px, py), (gx, gy)) {
        return reunat(vari, solu, (px, py), (gx, gy), naapuri);
    }
    // Kivi istuu maassa, ei mustassa kolossa.
    if solu == Solu::Kivi {
        return tummenna(maan_vari(y, gx, gy), 0.3);
    }
    let seina = seinan_vari(y, gx, gy);
    // Katon ja seinien reunassa on varjo.
    if (py == 0 && !avoin(naapuri(0, -1))) || (px == 0 && !avoin(naapuri(-1, 0))) {
        tummenna(seina, 0.35)
    } else if px == SOLU_P - 1 && !avoin(naapuri(1, 0)) {
        tummenna(seina, 0.2)
    } else {
        seina
    }
}

fn pimenna(vari: u32) -> u32 {
    sekoita_liuku(vari, 0x06030C, 0.8)
}

/// Kaivos valmiiksi laskettuna: valossa ja pimeässä.
pub struct Kaivoskuva {
    pub versio: u32,
    valo: Rasteri,
    pimea: Rasteri,
}

impl Kaivoskuva {
    pub fn new(kaivos: &Kaivos) -> Self {
        let valo = Rasteri::luo(RUUTUJA_L, RUUTUJA_K, |x, y| pikselin_vari(kaivos, x, y));
        let pimea = Rasteri::luo(RUUTUJA_L, RUUTUJA_K, |x, y| {
            let vari = valo.rivi(y as usize)[x as usize];
            if (y as usize) < VALOISA_RAJA {
                vari
            } else {
                pimenna(vari)
            }
        });
        Self {
            versio: kaivos.versio,
            valo,
            pimea,
        }
    }
}

/// Lampun valo konepikseleinä: keskipiste, säde ja täyden valon säde.
#[derive(Clone, Copy)]
struct Lamppu {
    x: f32,
    y: f32,
    sade: f32,
    sisa: f32,
}

impl Lamppu {
    fn new(peli: &Kaivo) -> Self {
        let kaivos = &peli.kaivos;
        let (x, y) = kaivajan_sijainti(kaivos);
        let lepatus = (kohina((peli.kello_ms / LEKOTUS_MS) as i32, 3) - 0.5) * 0.3;
        let sade = (kaivos.valon_sade() + lepatus) * SOLU_P as f32;
        Self {
            x: x * SOLU_P as f32 + SOLU_P as f32 / 2.0 + kaivos.kaivaja.katse as f32 * 2.0,
            y: y * SOLU_P as f32 + 3.0,
            sade,
            sisa: sade * 0.7,
        }
    }

    /// Valon osuus konepikselissä 0..1.
    fn osuus(&self, gx: i32, gy: i32) -> f32 {
        if (gy as usize) < VALOISA_RAJA {
            return 1.0;
        }
        let (dx, dy) = (gx as f32 + 0.5 - self.x, gy as f32 + 0.5 - self.y);
        let d = dx.hypot(dy);
        ((self.sade - d) / (self.sade - self.sisa)).clamp(0.0, 1.0)
    }
}

/// Kaivajan paikka ruutuina. Hissin matkalla kaivaja on häkissä.
fn kaivajan_sijainti(kaivos: &Kaivos) -> (f32, f32) {
    match kaivos.hissi_ms {
        Some(_) => (
            HISSI.0 as f32,
            HISSI.1 as f32 + hissin_lasku(kaivos) as f32 / SOLU as f32,
        ),
        None => kaivos.kaivaja.sijainti(&kaivos.solut),
    }
}

/// Häkin lasku pikseleinä hissin matkalla.
fn hissin_lasku(kaivos: &Kaivos) -> i32 {
    let jaljella = kaivos.hissi_ms.unwrap_or(HISSIN_MATKA_MS);
    let osuus = 1.0 - jaljella as f32 / HISSIN_MATKA_MS as f32;
    ((osuus * osuus * KENTTA_K as f32) as i32) / P * P
}

// ---- kaivaja ----

#[rustfmt::skip]
const SEISOO: [&str; 10] = [
    "...hHHH...", "..hHHHHHL.", "...SSSS...", "...SSES...", "..bBBBBb..",
    ".S.BBBB.S.", "...BBBB...", "...Bb.B...", "...Bb.B...", "..KK..KK..",
];
#[rustfmt::skip]
const KAVELEE: [&str; 10] = [
    "...hHHH...", "..hHHHHHL.", "...SSSS...", "...SSES...", "..bBBBBb..",
    ".S.BBBB.S.", "...BBBB...", "..Bb..B...", "..B....B..", ".KK....KK.",
];
#[rustfmt::skip]
const KAIVAA: [&str; 10] = [
    "...hHHH...", "..hHHHHHL.", "...SSSS...", "...SSES...", "..bBBBBb..",
    "...BBBBS..", "...BBBB...", "...Bb.B...", "..Bb...B..", "..KK...KK.",
];
#[rustfmt::skip]
const KIIPEAA: [&str; 10] = [
    ".S.hHHH...", ".S.hHHHHL.", ".b.SSSS...", ".b.SSES...", "..bBBBBb..",
    "...BBBB.S.", "...BBBB...", "...Bb.B...", "...Bb..B..", "..KK...KK.",
];
#[rustfmt::skip]
const PUTOAA: [&str; 10] = [
    "S..hHHH..S", ".b.hHHHHL.", ".b.SSSS.b.", "...SSES...", "..bBBBBb..",
    "...BBBB...", "...BBBB...", "..Bb..B...", ".Bb....B..", ".K......K.",
];
/// Kuokka ylhäällä ja iskussa: (sarake, rivi, väri).
const KUOKKA_YLHAALLA: [(i32, i32, u8); 7] = [
    (7, 4, b'P'),
    (8, 3, b'P'),
    (8, 2, b'P'),
    (7, 1, b'M'),
    (8, 1, b'M'),
    (9, 1, b'M'),
    (9, 2, b'M'),
];
const KUOKKA_ISKUSSA: [(i32, i32, u8); 7] = [
    (8, 5, b'P'),
    (9, 5, b'P'),
    (9, 3, b'M'),
    (9, 4, b'M'),
    (9, 6, b'M'),
    (9, 7, b'M'),
    (8, 7, b'M'),
];
const SAKKI: [(i32, i32); 5] = [(1, 4), (0, 5), (1, 5), (0, 6), (1, 6)];

fn kaivajan_vari(merkki: u8) -> Option<u32> {
    Some(match merkki {
        b'H' => 0xF8C838,
        b'h' => 0xC08820,
        b'L' => 0xFFFFE0,
        b'S' => 0xF0B088,
        b'E' => 0x201018,
        b'B' => 0x3858C8,
        b'b' => 0x24388A,
        b'K' => 0x3A2420,
        b'M' => 0xC8C8D8,
        b'P' => 0x9A6A38,
        b'Y' => 0xC8A060,
        _ => return None,
    })
}

// ---- piirto ----

pub fn piirra(peli: &Kaivo, kuva: &mut [u32]) {
    let mut p = Piirtaja {
        k: Kangas::new(kuva),
        kello: peli.kello_ms,
    };
    match (&peli.kuva, peli.vaihe) {
        (Some(kaivoskuva), vaihe) if vaihe != Vaihe::Alku => p.kaivon_nakyma(peli, kaivoskuva),
        _ => p.kaivon_alkuruutu(peli),
    }
    p.aanikytkin(peli.aanet.paalla);
}

fn kentalle(gx: i32, gy: i32) -> (i32, i32) {
    (KENTTA_X + gx * P, KENTTA_Y + gy * P)
}

impl Piirtaja<'_> {
    fn kaivon_alkuruutu(&mut self, peli: &Kaivo) {
        self.maisema(kaanna("kaivo.banneri"), &kaivosmalli());
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

    fn kaivon_nakyma(&mut self, peli: &Kaivo, kuva: &Kaivoskuva) {
        let kaivos = &peli.kaivos;
        let lamppu = Lamppu::new(peli);
        self.kaivon_kentta(kuva, lamppu);
        self.kimallukset(kaivos, lamppu);
        self.ritisevat(kaivos);
        self.putoavat(kaivos, lamppu);
        self.nostotorni(kaivos);
        if kaivos.hissi_ms.is_none() {
            self.kaivaja(kaivos);
        }
        for tehoste in &peli.tehosteet {
            self.kaivon_tehoste(tehoste);
        }
        if kaivos.hissi_ms.is_some() {
            self.taso_valmis(kaivos);
        }
        self.tausta_kentan_ympari(KENTTA_X, KENTTA_Y, KENTTA_L, KENTTA_K);
        self.upotus(KENTTA_X, KENTTA_Y, KENTTA_L, KENTTA_K);
        let otsikko = kaanna("peli.kaivo").to_uppercase();
        self.otsikkopalkki(&otsikko, kaanna(tason_nimi(kaivos.vaikeus)));
        self.kaivon_tilasto(peli);
        self.kaivon_paneeli(peli);
    }

    /// Joka konepikseli valitaan valon ja pimeän kuvista lampun etäisyyden mukaan.
    fn kaivon_kentta(&mut self, kuva: &Kaivoskuva, lamppu: Lamppu) {
        let mut rivi = [0; RUUTUJA_L];
        for y in 0..RUUTUJA_K {
            let (valo, pimea) = (kuva.valo.rivi(y), kuva.pimea.rivi(y));
            let dy = y as f32 + 0.5 - lamppu.y;
            let lahde: &[u32] = if y < VALOISA_RAJA {
                valo
            } else if dy.abs() >= lamppu.sade {
                pimea
            } else {
                for (x, kohde) in rivi.iter_mut().enumerate() {
                    let osuus = lamppu.osuus(x as i32, y as i32);
                    *kohde = if osuus >= 1.0 {
                        valo[x]
                    } else if osuus <= 0.0 {
                        pimea[x]
                    } else {
                        ditteroi(x as i32, y as i32, pimea[x], valo[x], osuus)
                    };
                }
                &rivi
            };
            self.k.karkea_rivi(KENTTA_X, KENTTA_Y + y as i32 * P, lahde);
        }
    }

    /// Pimeässä kulta ja timantit välähtelevät, joten ahne kaivaja tietää, minne kaivaa.
    fn kimallukset(&mut self, kaivos: &Kaivos, lamppu: Lamppu) {
        let tahti = (self.kello / KIMALLUS_MS) as i32;
        for y in MAA_ALKAA..RIVEJA as i32 {
            for x in 0..SARAKKEITA as i32 {
                let vari = match kaivos.solu((x, y)) {
                    Solu::Kulta => 0xD8A840,
                    Solu::Timantti => 0x90D8F8,
                    _ => continue,
                };
                let (gx, gy) = (x * SOLU_P + 4, y * SOLU_P + 4);
                if lamppu.osuus(gx, gy) > 0.0 || kohina(x * 31 + y * 17, tahti) < 0.88 {
                    continue;
                }
                let (sx, sy) = kentalle(gx, gy);
                self.k.laatikko(sx, sy, P, P, vari);
                let himmea = tummenna(vari, 0.5);
                for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    self.k.laatikko(sx + dx * P, sy + dy * P, P, P, himmea);
                }
            }
        }
    }

    /// Ritisevässä ruudussa on halkeamia, ja sen alta varisee pölyä. Juuri ennen putoamista
    /// ruutu vilkkuu punaisena.
    fn ritisevat(&mut self, kaivos: &Kaivos) {
        for y in 0..RIVEJA as i32 {
            for x in 0..SARAKKEITA as i32 {
                let ritina = kaivos.ritina[y as usize][x as usize];
                if ritina == 0 {
                    continue;
                }
                let (sx, sy) = kentalle(x * SOLU_P, y * SOLU_P);
                for i in 0..4 {
                    let py = 3 + (kohina(x * 5 + i, y) * 4.0) as i32;
                    self.k
                        .laatikko(sx + (2 + i * 2) * P, sy + py * P, P, P, 0x1A0C14);
                }
                if y + 1 < RIVEJA as i32 && avoin(kaivos.solu((x, y + 1))) {
                    for i in 0..3 {
                        let vaihe = (self.kello / POLYN_TAHTI_MS) as i32 + i * 5 + x * 7;
                        let px = 1 + (kohina(x * 3 + i, y * 5) * 8.0) as i32;
                        let py = SOLU_P + vaihe.rem_euclid(SOLU_P + 2);
                        self.k.laatikko(sx + px * P, sy + py * P, P, P, 0xE8DCC0);
                    }
                }
                if ritina < VAROITUS_MS && (self.kello / VAROITUKSEN_VILKKU_MS).is_multiple_of(2) {
                    self.k.kehys(sx, sy, SOLU, SOLU, 0xFF5030);
                }
            }
        }
    }

    fn putoavat(&mut self, kaivos: &Kaivos, lamppu: Lamppu) {
        for putoava in &kaivos.putoavat {
            let lasku = (putoava.ms * SOLU_P as u32 / PUTOAMINEN_MS) as i32;
            let (x0, y0) = (putoava.paikka.0 * SOLU_P, putoava.paikka.1 * SOLU_P + lasku);
            for py in 0..SOLU_P {
                for px in 0..SOLU_P {
                    let (gx, gy) = (x0 + px, y0 + py);
                    let Some(mut vari) =
                        materiaali(putoava.solu, putoava.paikka.1, (px, py), (gx, gy))
                    else {
                        continue;
                    };
                    if py == 0 {
                        vari = vaalenna(vari, 0.25);
                    } else if py == SOLU_P - 1 {
                        vari = tummenna(vari, 0.4);
                    }
                    let osuus = lamppu.osuus(gx, gy);
                    let vari = ditteroi(gx, gy, pimenna(vari), vari, osuus);
                    let (sx, sy) = kentalle(gx, gy);
                    self.k.laatikko(sx, sy, P, P, vari);
                }
            }
        }
    }

    /// Nostotorni hissin yllä. Häkin valo palaa vihreänä, kun säkissä on kiintiön verran.
    fn nostotorni(&mut self, kaivos: &Kaivos) {
        let (x, y) = (KENTTA_X, KENTTA_Y);
        let palkki = 0x4A3A5A;
        let kirkas = 0x8A7898;
        self.k.viiva((x + 12, y + 57), (x + 39, y + 12), palkki);
        self.k.viiva((x + 78, y + 57), (x + 51, y + 12), palkki);
        self.k.viiva((x + 15, y + 54), (x + 39, y + 15), kirkas);
        for (korkeus, puoli) in [(24, 18), (42, 27)] {
            self.k
                .laatikko(x + 45 - puoli, y + korkeus, 2 * puoli, P, palkki);
        }
        self.k.laatikko(x + 33, y + 12, 24, P * 2, kirkas);
        // Pyörä pyörii, kun häkki liikkuu.
        let keski = (x + 45, y + 9);
        self.k.ympyra(keski, 6.0, 0xC8C8D8);
        let kulma = hissin_lasku(kaivos) as f32 * 0.05;
        let (sin, cos) = kulma.sin_cos();
        let pinna = (keski.0 + (cos * 4.0) as i32, keski.1 + (sin * 4.0) as i32);
        self.k.laatikko(pinna.0 - 1, pinna.1 - 1, P, P, 0xC8C8D8);

        let lasku = hissin_lasku(kaivos);
        let (hx, hy) = (x + HISSI.0 * SOLU + 3, y + HISSI.1 * SOLU + 3 + lasku);
        if lasku > 0 {
            // Kuilu häkin yläpuolella.
            let ylin = y + (HISSI.1 + 1) * SOLU;
            self.k
                .laatikko(hx, ylin, SOLU - 6, (hy - ylin).max(0), 0x0A0610);
        }
        self.k
            .laatikko(x + 45 - 1, y + 15, P, hy - (y + 15), 0x2A2238);
        self.k.laatikko(hx, hy, SOLU - 6, SOLU - 3, 0x16101E);
        if lasku > 0 {
            // Kaivaja seisoo häkissä matkalla alas.
            let (kx, ky) = (x + HISSI.0 * SOLU, y + HISSI.1 * SOLU + lasku);
            self.kaivajan_kuva((kx, ky), &SEISOO, false, &[], false);
        }
        for i in 0..4 {
            self.k.laatikko(hx + i * 7, hy, P, SOLU - 3, 0x8C8CA0);
        }
        self.k.kehys(hx, hy, SOLU - 6, SOLU - 3, 0xB0B0C4);
        let vari = if !kaivos.kiintio_taynna() {
            0xC83030
        } else if (self.kello / HISSIVALON_VILKKU_MS).is_multiple_of(2) {
            0x58FF60
        } else {
            0x2C8030
        };
        self.k
            .laatikko(hx + (SOLU - 6) / 2 - P, hy - P * 2, P * 2, P * 2, vari);
    }

    fn kaivaja(&mut self, kaivos: &Kaivos) {
        let k = kaivos.kaivaja;
        let (x, y) = k.sijainti(&kaivos.solut);
        let kohta = |arvo: f32| ((arvo * SOLU as f32) as i32) / P * P;
        let paikka = (KENTTA_X + kohta(x), KENTTA_Y + kohta(y));
        let (kuva, kuokka): (&[&str; 10], &[(i32, i32, u8)]) = match k.toimi {
            Toimi::Liikkuu(_, 0) if (k.ms / KAVELYN_TAHTI_MS).is_multiple_of(2) => (&KAVELEE, &[]),
            Toimi::Liikkuu(_, 0) => (&SEISOO, &[]),
            Toimi::Liikkuu(..) => (&KIIPEAA, &[]),
            Toimi::Kaivaa(..) if (k.ms / KUOKAN_TAHTI_MS).is_multiple_of(2) => {
                (&KAIVAA, &KUOKKA_YLHAALLA)
            }
            Toimi::Kaivaa(..) => (&KAIVAA, &KUOKKA_ISKUSSA),
            Toimi::Putoaa => (&PUTOAA, &[]),
            Toimi::Seisoo if kaivos.solu(k.paikka).kiivettava() => (&KIIPEAA, &[]),
            Toimi::Seisoo => (&SEISOO, &[]),
        };
        // Kiivetessä kädet vaihtavat puolta.
        let peilaa = match k.toimi {
            Toimi::Liikkuu(0, _) => !(k.ms / KAVELYN_TAHTI_MS).is_multiple_of(2),
            _ => k.katse < 0,
        };
        self.kaivajan_kuva(paikka, kuva, peilaa, kuokka, kaivos.saalis > 0);
    }

    fn kaivajan_kuva(
        &mut self,
        (x, y): (i32, i32),
        kuva: &[&str; 10],
        peilaa: bool,
        kuokka: &[(i32, i32, u8)],
        sakki: bool,
    ) {
        let sarake = |c: i32| if peilaa { SOLU_P - 1 - c } else { c };
        for (r, rivi) in kuva.iter().enumerate() {
            for (c, merkki) in rivi.bytes().enumerate() {
                if let Some(vari) = kaivajan_vari(merkki) {
                    self.k
                        .laatikko(x + sarake(c as i32) * P, y + r as i32 * P, P, P, vari);
                }
            }
        }
        let lisat = kuokka
            .iter()
            .copied()
            .chain(SAKKI.iter().filter(|_| sakki).map(|&(c, r)| (c, r, b'Y')));
        for (c, r, merkki) in lisat {
            if let Some(vari) = kaivajan_vari(merkki) {
                self.k.laatikko(x + sarake(c) * P, y + r * P, P, P, vari);
            }
        }
    }

    fn kaivon_tehoste(&mut self, tehoste: &Tehoste) {
        let ika = self.kello.saturating_sub(tehoste.alku) as f32 / TEHOSTE_MS as f32;
        let (x, y) = tehoste.paikka;
        let keski = (
            KENTTA_X + x * SOLU + SOLU / 2,
            KENTTA_Y + y * SOLU + SOLU / 2,
        );
        match tehoste.laji {
            Tehostelaji::Pisteet(pisteet) => {
                let teksti = format!("+{pisteet}");
                let leveys = teksti.chars().count() as i32 * 16;
                let y = keski.1 - 24 - (ika * 36.0) as i32;
                self.teksti_koossa(keski.0 - leveys / 2, y, &teksti, 2, KELTAINEN);
            }
            Tehostelaji::Poly => {
                let vari = sekoita_liuku(0xD8C8A8, 0x4A3A44, ika);
                for i in 0..8 {
                    let kulma = i as f32 / 8.0 * TAU + 0.3;
                    let matka = 6.0 + ika * (14.0 + (i % 3) as f32 * 4.0);
                    let (px, py) = (
                        keski.0 + (kulma.cos() * matka) as i32,
                        keski.1 + 9 + (kulma.sin() * matka * 0.5) as i32,
                    );
                    self.k.piste(px, py, vari);
                }
            }
            Tehostelaji::Isku => {
                let sade = 4.0 + ika * 24.0;
                self.k
                    .ympyra(keski, sade, sekoita_liuku(0xFFFFFF, 0x6A1820, ika));
                self.k
                    .ympyra(keski, sade * 0.6, sekoita_liuku(0xFF5030, 0x2A0C14, ika));
            }
        }
    }

    /// Hissin matkalla kentän keskellä: taso valmis ja seuraava taso.
    fn taso_valmis(&mut self, kaivos: &Kaivos) {
        let (l, k) = (540, 96);
        let (x, y) = (KENTTA_X + (KENTTA_L - l) / 2, KENTTA_Y + (KENTTA_K - k) / 2);
        self.ikkuna(x, y, l, k);
        let otsikko = kaanna("kaivo.valmis_otsikko");
        let leveys = otsikko.chars().count() as i32 * OTSIKKO.2;
        self.suurteksti(
            x + (l - leveys) / 2,
            y + 18,
            otsikko,
            OTSIKKO,
            KELTAINEN,
            true,
        );
        let seuraava = muotoile("kaivo.seuraava", &[&(kaivos.taso + 2)]);
        let leveys = seuraava.chars().count() as i32 * MERKKI;
        self.teksti(x + (l - leveys) / 2, y + 60, &seuraava, VALKOINEN);
    }

    fn kaivon_tilasto(&mut self, peli: &Kaivo) {
        let kaivos = &peli.kaivos;
        let lohko = |i: i32| KENTTA_Y + i * TILASTON_ASKEL;
        let arvot = [
            (0, "yhteinen.pisteet", kaivos.pisteet.to_string()),
            (1, "yhteinen.paras", peli.taulu.paras.to_string()),
            (2, "kaivo.taso_otsikko", (kaivos.taso + 1).to_string()),
            (4, "kaivo.sakki", kaivos.saalis.to_string()),
        ];
        for (i, otsikko, arvo) in arvot {
            let y = lohko(i);
            self.suurteksti(SARAKE_X, y, kaanna(otsikko), OTSIKKO, KELTAINEN, false);
            self.suurteksti(SARAKE_X, y + ARVON_Y, &arvo, OTSIKKO, VALKOINEN, true);
        }

        // Tuet ja tikkaat jakavat lohkon: nimi vasemmalla ja määrä oikeassa reunassa.
        let varasto = [
            ("kaivo.tuet", kaivos.tuet),
            ("kaivo.tikkaat", kaivos.tikkaat),
        ];
        for (rivi, (otsikko, maara)) in varasto.into_iter().enumerate() {
            let y = lohko(5) + rivi as i32 * ARVON_Y;
            self.suurteksti(SARAKE_X, y, kaanna(otsikko), OTSIKKO, KELTAINEN, false);
            let arvo = maara.to_string();
            let x = SARAKE_X + PARHAAT_L - arvo.chars().count() as i32 * OTSIKKO.2;
            self.suurteksti(x, y, &arvo, OTSIKKO, VALKOINEN, true);
        }

        // Kiintiö: hissiin viety täytenä, säkissä oleva himmeämpänä perässä.
        let y = lohko(3);
        let otsikko = kaanna("kaivo.kiintio");
        self.suurteksti(SARAKE_X, y, otsikko, OTSIKKO, KELTAINEN, false);
        let kiintio = kaivos.kiintio.max(1) as f32;
        let nostettu = kaivos.nostettu as f32 / kiintio;
        let sakissa = (kaivos.nostettu + kaivos.saalis) as f32 / kiintio;
        let my = y + ARVON_Y + 3;
        self.kaivon_mittari(SARAKE_X, my, PARHAAT_L, sakissa, 0x8A6A30);
        self.kaivon_mittari_tayte(SARAKE_X, my, PARHAAT_L, nostettu, 0xF0C030);

        let y = lohko(6);
        let otsikko = kaanna("kaivo.kyparat");
        self.suurteksti(SARAKE_X, y, otsikko, OTSIKKO, KELTAINEN, false);
        for i in 0..KYPARIA {
            let x = SARAKE_X + i as i32 * 33;
            self.kypara((x, y + ARVON_Y + 3), i < kaivos.kyparat);
        }

        let y = lohko(7);
        let otsikko = kaanna("kaivo.lamppu");
        self.suurteksti(SARAKE_X, y, otsikko, OTSIKKO, KELTAINEN, false);
        let oljy = kaivos.oljyn_osuus();
        let vari = sekoita_liuku(0xE03828, 0xFFE070, oljy);
        self.kaivon_mittari(SARAKE_X, y + ARVON_Y + 3, PARHAAT_L, oljy, vari);

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

    /// Upotettu palkki ja täyttö.
    fn kaivon_mittari(&mut self, x: i32, y: i32, leveys: i32, osuus: f32, vari: u32) {
        self.k.laatikko(x, y, leveys, MITTARI_K, 0x0E0830);
        self.upotus(x, y, leveys, MITTARI_K);
        self.kaivon_mittari_tayte(x, y, leveys, osuus, vari);
    }

    fn kaivon_mittari_tayte(&mut self, x: i32, y: i32, leveys: i32, osuus: f32, vari: u32) {
        let taysi = ((leveys as f32 * osuus.clamp(0.0, 1.0)) as i32) / P * P;
        if taysi > 0 {
            self.k.laatikko(x, y, taysi, MITTARI_K, vari);
            self.k.laatikko(x, y, taysi, P, vaalenna(vari, 0.5));
        }
    }

    fn kypara(&mut self, kohta: (i32, i32), jaljella: bool) {
        #[rustfmt::skip]
        const KYPARA: [&str; 5] = ["..HHH..", ".HHHHL.", "HHHHHHH", "hhhhhhh", "......."];
        let (vari, varjo, lamppu) = if jaljella {
            (0xF8C838, 0xC08820, 0xFFFFE0)
        } else {
            (0x3C3460, 0x2C2450, 0x3C3460)
        };
        self.k.sprite(kohta, &KYPARA, P, |merkki| match merkki {
            b'H' => Some(vari),
            b'h' => Some(varjo),
            b'L' => Some(lamppu),
            _ => None,
        });
    }

    fn kaivon_paneeli(&mut self, peli: &Kaivo) {
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
            Vaihe::Nimi => self.nimikysely(peli.kaivos.pisteet, &peli.nimi),
            Vaihe::Alku => {}
        }
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
pub use tests::{ruudun_keskipiste, vaikeuden_keskipiste};
