//! 5 suoran näkymät: ruutupaperilauta keskellä, pelaajat sivuilla ja viestit alhaalla.

use klassikot8000::suora::{KOKO, Merkki, Paikka, Vaikeus};

use super::peli::{PELAAJA, Suora, Vaihe};
use crate::yhteinen::kieli::kaanna;
use crate::yhteinen::piirto::*;
use crate::yhteinen::vektori::{Kangas, Tahko, kuutio, tahko4, tummenna};

/// Ruudun sivu: viiva vie ylä- ja vasemmasta reunasta konepikselin.
const RUUTU: i32 = 36;
const LAUTA: i32 = RUUTU * KOKO as i32;
const ALUE_Y: i32 = 48;
const ALUE_K: i32 = PANEELI_Y - 6 - ALUE_Y;
const ORIGO: (i32, i32) = (
    (LEVEYS as i32 - LAUTA) / 2,
    ALUE_Y + (ALUE_K - LAUTA) / 6 * 3,
);

// Ruutupaperi: kellertävä pohja ja siniset viivat.
const PAPERI: u32 = 0xF4EED8;
const VIIVA: u32 = 0x9AB0DC;
const KOROSTUS: u32 = 0xFFF8C8;
const VIIMEISIN: u32 = 0xDCE6FF;
const VOITTO: [u32; 2] = [0xFFD860, 0xFFB848];
const MUSTE: u32 = 0x1A1A48;
const VARJO: u32 = 0xC8C0A8;
/// Sivuikkunan sinisellä pohjalla varjo on tummaa sinistä.
const IKKUNAN_VARJO: u32 = 0x0C1450;
const RISTI: u32 = 0xD8302A;
const RISTI_VALO: u32 = 0xFF9080;
const NOLLA: u32 = 0x2860D0;
const NOLLA_VALO: u32 = 0x90C0FF;

#[rustfmt::skip]
const RISTIN_KUVA: [&str; 9] = [
    "wr.....wr",
    "rrr...wrr",
    ".rrr.wrr.",
    "..rrrrr..",
    "...rrr...",
    "..rrrrr..",
    ".rrr.rrr.",
    "rrr...rrr",
    "rr.....rr",
];
#[rustfmt::skip]
const NOLLAN_KUVA: [&str; 9] = [
    "..wwrrr..",
    ".wrrrrrr.",
    "wrr...rrr",
    "rr.....rr",
    "rr.....rr",
    "rr.....rr",
    "rrr...rrr",
    ".rrrrrrr.",
    "..rrrrr..",
];

// Pelaajien ikkunat laudan sivuilla.
const SIVU_X: [i32; 2] = [18, LEVEYS as i32 - 18 - SIVU_L];
const SIVU_Y: i32 = 66;
const SIVU_L: i32 = 174;
const SIVU_K: i32 = 195;
const ISO_KERROIN: i32 = 9;

const OHJE: &str = "suora.ohje";
const TILASTON_Y: i32 = PANEELI_RIVI_Y + 30;
const TILASTON_ASKEL: i32 = 300;

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
    "suora.tervetuloa",
    "yhteinen.ohjeet",
    "suora.ohje_1",
    "suora.ohje_2",
];
const ALKU_RIVIT: [&str; 4] = [
    "suora.nappain_hiiri",
    "suora.nappain_nuolet",
    "yhteinen.aanet_kytkin",
    "yhteinen.takaisin",
];

fn kuvaus(vaikeus: Vaikeus) -> &'static str {
    match vaikeus {
        Vaikeus::Helppo => "suora.kuvaus_1",
        Vaikeus::Normaali => "suora.kuvaus_2",
        Vaikeus::Vaikea => "suora.kuvaus_3",
    }
}

fn tason_nimi(vaikeus: Vaikeus) -> &'static str {
    match vaikeus {
        Vaikeus::Helppo => "yhteinen.helppo",
        Vaikeus::Normaali => "yhteinen.normaali",
        Vaikeus::Vaikea => "yhteinen.vaikea",
    }
}

fn ruudun_kulma((rivi, sarake): Paikka) -> (i32, i32) {
    (
        ORIGO.0 + sarake as i32 * RUUTU,
        ORIGO.1 + rivi as i32 * RUUTU,
    )
}

pub fn ruutu_pisteesta(x: i32, y: i32) -> Option<Paikka> {
    let (dx, dy) = (x - ORIGO.0, y - ORIGO.1);
    if dx < 0 || dy < 0 {
        return None;
    }
    let (rivi, sarake) = ((dy / RUUTU) as usize, (dx / RUUTU) as usize);
    (rivi < KOKO && sarake < KOKO).then_some((rivi, sarake))
}

pub fn vaikeus_pisteesta(x: i32, y: i32) -> Option<Vaikeus> {
    rivin_nappi(&TASOT, TASO_Y, x, y)
}

/// Palkki maanpinnan suuntaisena: janasta `a`-`b` leveyden verran kumpaankin sivuun.
fn palkki(a: [f32; 2], b: [f32; 2], leveys: f32, y: [f32; 2], vari: u32, tahkot: &mut Vec<Tahko>) {
    let (dx, dz) = (b[0] - a[0], b[1] - a[1]);
    let pituus = (dx * dx + dz * dz).sqrt();
    let (nx, nz) = (-dz / pituus * leveys, dx / pituus * leveys);
    let kulmat = [
        [a[0] + nx, a[1] + nz],
        [b[0] + nx, b[1] + nz],
        [b[0] - nx, b[1] - nz],
        [a[0] - nx, a[1] - nz],
    ];
    let p = |i: usize, korkeus: f32| [kulmat[i][0], korkeus, kulmat[i][1]];
    tahkot.push(tahko4(p(0, y[1]), p(1, y[1]), p(2, y[1]), p(3, y[1]), vari));
    for i in 0..4 {
        let j = (i + 1) % 4;
        let sivu = tummenna(vari, 0.25);
        tahkot.push(tahko4(p(i, y[0]), p(j, y[0]), p(j, y[1]), p(i, y[1]), sivu));
    }
}

/// Alkuruudun ja valikon malli: ruutulauta, jolla viisi ristiä vinossa ja nollia vastassa.
pub fn suoramalli() -> Vec<Tahko> {
    const RUUTUJA: i32 = 8;
    const SOLU: f32 = 0.7;
    const PUOLI: f32 = RUUTUJA as f32 * SOLU / 2.0;
    const PAKSU: f32 = 0.4;
    let mut tahkot = Vec::new();
    let reuna = [-PUOLI, PUOLI];
    kuutio(reuna, [0.0, PAKSU], reuna, PAPERI, &mut tahkot);
    let pinta = [PAKSU, PAKSU + 0.03];
    for i in 1..RUUTUJA {
        let v = -PUOLI + i as f32 * SOLU;
        let viiva = [v - 0.03, v + 0.03];
        kuutio(viiva, pinta, reuna, VIIVA, &mut tahkot);
        kuutio(reuna, pinta, viiva, VIIVA, &mut tahkot);
    }
    let merkki_y = [PAKSU, PAKSU + 0.3];
    let keski = |ruutu: i32| -PUOLI + (ruutu as f32 + 0.5) * SOLU;
    let s = 0.24;
    for i in 1..6 {
        let (x, z) = (keski(i), keski(i + 1));
        let ristin = |a: [f32; 2], b: [f32; 2], tahkot: &mut Vec<Tahko>| {
            palkki(a, b, 0.07, merkki_y, RISTI, tahkot);
        };
        ristin([x - s, z - s], [x + s, z + s], &mut tahkot);
        ristin([x - s, z + s], [x + s, z - s], &mut tahkot);
    }
    for (sarake, rivi) in [(2, 1), (3, 2), (5, 3), (6, 6)] {
        let (x, z) = (keski(sarake), keski(rivi));
        const SIVUJA: usize = 8;
        let piste = |i: usize| {
            let kulma = i as f32 / SIVUJA as f32 * std::f32::consts::TAU;
            [x + s * kulma.cos(), z + s * kulma.sin()]
        };
        for i in 0..SIVUJA {
            palkki(piste(i), piste(i + 1), 0.06, merkki_y, NOLLA, &mut tahkot);
        }
    }
    tahkot
}

pub fn piirra(peli: &Suora, kuva: &mut [u32]) {
    let mut p = Piirtaja {
        k: Kangas::new(kuva),
        kello: peli.kello_ms,
    };
    if peli.vaihe == Vaihe::Alku {
        p.suoran_alkuruutu(peli);
    } else {
        p.suoran_nakyma(peli);
    }
    p.aanikytkin(peli.aanet.paalla);
}

impl Piirtaja<'_> {
    fn suoran_alkuruutu(&mut self, peli: &Suora) {
        self.maisema(kaanna("suora.banneri"), &suoramalli());
        let otsikko = listan_otsikko(peli.taulu.netti.is_some());
        self.parhaat_alkuruudulla(otsikko, peli.taulu.nakyva());
        let y = self.alkuohje(&ALKU_OSAT, &ALKU_RIVIT) + RIVIN_KORKEUS / 2;
        self.teksti(48, y, kaanna(kuvaus(peli.vaikeus)), KELTAINEN);

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

    fn suoran_nakyma(&mut self, peli: &Suora) {
        self.pelitausta();
        self.suoran_lauta(peli);
        let otsikko = kaanna("peli.suora").to_uppercase();
        self.otsikkopalkki(&otsikko, "");
        for merkki in [Merkki::Risti, Merkki::Nolla] {
            self.pelaajan_ikkuna(peli, merkki);
        }
        self.suoran_paneeli(peli);
    }

    fn suoran_lauta(&mut self, peli: &Suora) {
        let lopussa = !matches!(peli.vaihe, Vaihe::Pelissa);
        let hiiri = if !lopussa && !peli.kursori_nakyy && !peli.kone_vuorossa() {
            peli.hiiren_ruutu.filter(|&p| peli.lauta.ruutu(p).is_none())
        } else {
            None
        };
        let voittovari = VOITTO[usize::from(self.vilkku())];
        self.upotus(ORIGO.0, ORIGO.1, LAUTA + 3, LAUTA + 3);
        self.k
            .laatikko(ORIGO.0, ORIGO.1, LAUTA + 3, LAUTA + 3, VIIVA);
        for rivi in 0..KOKO {
            for sarake in 0..KOKO {
                let kohta = (rivi, sarake);
                let (x, y) = ruudun_kulma(kohta);
                let pohja = if peli.lauta.voittorivi.contains(&kohta) {
                    voittovari
                } else if peli.lauta.viimeisin() == Some(kohta) && !lopussa {
                    VIIMEISIN
                } else if hiiri == Some(kohta) {
                    KOROSTUS
                } else {
                    PAPERI
                };
                self.k.laatikko(x + 3, y + 3, RUUTU - 3, RUUTU - 3, pohja);
                if let Some(merkki) = peli.lauta.ruutu(kohta) {
                    self.merkki((x + 6, y + 6), merkki, 3, VARJO);
                }
            }
        }
        if let (Some(&alku), Some(&loppu)) =
            (peli.lauta.voittorivi.first(), peli.lauta.voittorivi.last())
        {
            self.yliviivaus(alku, loppu);
        }
        if peli.kursori_nakyy && !lopussa {
            let (x, y) = ruudun_kulma(peli.kursori);
            self.valintakehys(x + 3, y + 3, RUUTU - 3, RUUTU - 3, KELTAINEN);
        }
    }

    /// Risti tai nolla varjoineen: varjo osuu oikealle alas.
    fn merkki(&mut self, kulma: (i32, i32), merkki: Merkki, kerroin: i32, varjon_vari: u32) {
        let (kuva, vari, valo) = match merkki {
            Merkki::Risti => (&RISTIN_KUVA, RISTI, RISTI_VALO),
            Merkki::Nolla => (&NOLLAN_KUVA, NOLLA, NOLLA_VALO),
        };
        let varjo = (kulma.0 + kerroin.max(3), kulma.1 + kerroin.max(3));
        self.k
            .sprite(varjo, kuva, kerroin, |m| (m != b'.').then_some(varjon_vari));
        self.k.sprite(kulma, kuva, kerroin, |m| match m {
            b'r' => Some(vari),
            b'w' => Some(valo),
            _ => None,
        });
    }

    /// Voittorivin yli kaksi konepikseliä paksu musteviiva keskeltä keskelle.
    fn yliviivaus(&mut self, alku: Paikka, loppu: Paikka) {
        let keski = |p: Paikka| {
            let (x, y) = ruudun_kulma(p);
            (x + 3 + RUUTU / 2, y + 3 + RUUTU / 2)
        };
        let (a, b) = (keski(alku), keski(loppu));
        let p = self.k.pikseli;
        for (dx, dy) in [(0, 0), (p, 0), (0, p)] {
            self.k
                .viiva((a.0 + dx, a.1 + dy), (b.0 + dx, b.1 + dy), MUSTE);
        }
    }

    fn pelaajan_ikkuna(&mut self, peli: &Suora, merkki: Merkki) {
        let oma = merkki == PELAAJA;
        let x = SIVU_X[usize::from(!oma)];
        self.ikkuna(x, SIVU_Y, SIVU_L, SIVU_K);
        let vuorossa = peli.vaihe == Vaihe::Pelissa && peli.lauta.vuoro == merkki;
        if vuorossa {
            self.valintakehys(x, SIVU_Y, SIVU_L, SIVU_K, KELTAINEN);
        }
        let nimi = kaanna(if oma { "suora.sina" } else { "suora.kone" });
        let vari = if vuorossa { KELTAINEN } else { LAVENTELI };
        let keski = x + SIVU_L / 2;
        let leveys = |teksti: &str| teksti.chars().count() as i32 * MERKKI;
        self.teksti(keski - leveys(nimi) / 2, SIVU_Y + 15, nimi, vari);
        let koko = 9 * ISO_KERROIN;
        self.merkki(
            (keski - koko / 2 / 3 * 3, SIVU_Y + 45),
            merkki,
            ISO_KERROIN,
            IKKUNAN_VARJO,
        );
        // Istunnon voitot isolla kuin tulostaululla.
        let voitot = if oma { peli.ottelu.0 } else { peli.ottelu.1 }.to_string();
        let x = keski - voitot.chars().count() as i32 * OTSIKKO.2 / 2;
        let y = SIVU_Y + SIVU_K - 48;
        self.suurteksti(x, y, &voitot, OTSIKKO, KELTAINEN, false);
    }

    fn suoran_paneeli(&mut self, peli: &Suora) {
        self.paneelin_tausta();
        if peli.vaihe == Vaihe::Nimi {
            self.nimikysely(peli.pisteet(), &peli.nimi);
            return;
        }
        self.teksti(TEKSTI_X, PANEELI_RIVI_Y, &peli.viesti, VALKOINEN);

        let paras = match peli.taulu.paras {
            0 => "-".to_string(),
            paras => paras.to_string(),
        };
        let tilasto = [
            format!("{} {}", kaanna("yhteinen.siirrot"), peli.omat_siirrot()),
            format!("{} {paras}", kaanna("yhteinen.paras")),
            kaanna(tason_nimi(peli.vaikeus)).to_uppercase(),
        ];
        for (i, teksti) in tilasto.iter().enumerate() {
            let x = TEKSTI_X + i as i32 * TILASTON_ASKEL;
            self.suurteksti(x, TILASTON_Y, teksti, OTSIKKO, KELTAINEN, false);
        }

        let alarivi = PANEELI_RIVI_Y + (PANEELI_RIVEJA as i32 - 1) * RIVIN_KORKEUS;
        let otsikko_y = alarivi - RIVIN_KORKEUS - 12;
        let otsikko = match peli.vaihe {
            Vaihe::Voitto => Some("yhteinen.voitit"),
            Vaihe::Tappio => Some("suora.tappio"),
            Vaihe::Tasapeli => Some("suora.tasapeli"),
            Vaihe::Pelissa | Vaihe::Alku | Vaihe::Nimi => None,
        };
        if let Some(otsikko) = otsikko {
            self.suurteksti(TEKSTI_X, otsikko_y, kaanna(otsikko), TULOS, KELTAINEN, true);
            self.loppunapit();
        } else {
            self.teksti(TEKSTI_X, alarivi, kaanna(OHJE), HUTI);
        }
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
pub use tests::{ruudun_keskipiste, vaikeuden_keskipiste};
