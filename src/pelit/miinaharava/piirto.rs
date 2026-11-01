//! Miinaharavan näkymät.

use std::f32::consts::TAU;

use klassikot8000::miinaharava::{Paikka, Tila, Vaikeus};

use super::peli::{Miinaharava, Nappi, RAJAHDYS_MS, Vaihe};
use crate::yhteinen::kieli::{kaanna, muotoile};
use crate::yhteinen::piirto::*;
use crate::yhteinen::vektori::{Kangas, Tahko, kuutio, tahko4, tummenna, vaalenna};

// Lauta keskitetään otsikkopalkin ja paneelin väliin.
const ALUE_Y: i32 = 48;
const ALUE_K: i32 = PANEELI_Y - 6 - ALUE_Y;

const SULJETTU: u32 = 0x6878A0;
const LIPUN_KUVA: [&str; 11] = [
    "...........",
    "....#......",
    "...r#......",
    "..rr#......",
    ".rRr#......",
    "..rr#......",
    "...r#......",
    "....#......",
    "....#......",
    "..#####....",
    ".#######...",
];
const MIINAN_KUVA: [&str; 11] = [
    ".....r.....",
    ".r...x...r.",
    "..xx###xx..",
    "..#oo###x..",
    "..#o####x..",
    "rx######xxr",
    "..#####xx..",
    "..#####xx..",
    "..xxxxxxx..",
    ".r...x...r.",
    ".....r.....",
];
const AVATTU: u32 = VESI;
const AVATTU_REUNA: u32 = 0x1A2A6E;
const KOROSTUS: u32 = 0x7A8A98;
const RAJAHTANYT: u32 = OSUMA;
const LIPPU: u32 = 0xE03C31;
const NUMEROT: [u32; 8] = [
    0x6FB8F0, 0x70C050, 0xF06060, 0xB888F0, 0xF0A050, 0x4FD8D0, 0xF0F0F0, 0xB0B0B0,
];

const OHJE: &str = "miinaharava.ohje";
const TILASTON_Y: i32 = PANEELI_RIVI_Y + 30;
const TILASTON_ASKEL: i32 = 300;

const NAPIT: [(Nappi, &str, i32); 2] = [
    (Nappi::Uudestaan, "yhteinen.uudestaan", NAPPI_X),
    (
        Nappi::Valikko,
        "yhteinen.valikkoon",
        NAPPI_X + 13 * MERKKI + 48,
    ),
];

// Alkuruudun tasovalinta samoilla mitoilla kuin laivanupotuksessa.
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
    "miinaharava.tervetuloa",
    "yhteinen.ohjeet",
    "miinaharava.ohje_1",
    "miinaharava.ohje_2",
];
const ALKU_RIVIT: [&str; 4] = [
    "miinaharava.nappain_hiiri",
    "miinaharava.nappain_nuolet",
    "yhteinen.aanet_kytkin",
    "yhteinen.takaisin",
];

/// Ruudun sivu pikseleinä: helppo kenttä saa isot ruudut, ja vaikean 30 saraketta mahtuu leveyteen.
fn ruudun_koko(vaikeus: Vaikeus) -> i32 {
    match vaikeus {
        Vaikeus::Helppo => 60,
        Vaikeus::Normaali => 33,
        Vaikeus::Vaikea => 30,
    }
}

/// Laudan vasen yläkulma.
fn origo(vaikeus: Vaikeus) -> (i32, i32) {
    let koko = ruudun_koko(vaikeus);
    (
        (LEVEYS as i32 - koko * vaikeus.leveys() as i32) / 2,
        ALUE_Y + (ALUE_K - koko * vaikeus.korkeus() as i32) / 2,
    )
}

pub fn ruutu_pisteesta(vaikeus: Vaikeus, x: i32, y: i32) -> Option<Paikka> {
    let (ox, oy) = origo(vaikeus);
    let koko = ruudun_koko(vaikeus);
    if x < ox || y < oy {
        return None;
    }
    let (rivi, sarake) = (((y - oy) / koko) as usize, ((x - ox) / koko) as usize);
    (rivi < vaikeus.korkeus() && sarake < vaikeus.leveys()).then_some((rivi, sarake))
}

#[cfg(test)]
pub fn ruudun_keskipiste(vaikeus: Vaikeus, (rivi, sarake): Paikka) -> (i32, i32) {
    let (ox, oy) = origo(vaikeus);
    let koko = ruudun_koko(vaikeus);
    (
        ox + sarake as i32 * koko + koko / 2,
        oy + rivi as i32 * koko + koko / 2,
    )
}

pub fn nappi_pisteesta(x: i32, y: i32) -> Option<Nappi> {
    NAPIT
        .iter()
        .find(|&&(_, teksti, nx)| {
            (nx..nx + napin_leveys(teksti)).contains(&x)
                && (NAPPI_Y..NAPPI_Y + NAPPI_K).contains(&y)
        })
        .map(|&(nappi, ..)| nappi)
}

#[cfg(test)]
pub fn napin_keskipiste(nappi: Nappi) -> (i32, i32) {
    let (_, teksti, x) = NAPIT.iter().find(|n| n.0 == nappi).copied().unwrap();
    (x + napin_leveys(teksti) / 2, NAPPI_Y + NAPPI_K / 2)
}

pub fn vaikeus_pisteesta(x: i32, y: i32) -> Option<Vaikeus> {
    TASOT
        .iter()
        .find(|&&(_, teksti, nx)| {
            (nx..nx + napin_leveys(teksti)).contains(&x) && (TASO_Y..TASO_Y + NAPPI_K).contains(&y)
        })
        .map(|&(taso, ..)| taso)
}

#[cfg(test)]
pub fn vaikeuden_keskipiste(vaikeus: Vaikeus) -> (i32, i32) {
    let (_, teksti, x) = TASOT.iter().find(|t| t.0 == vaikeus).copied().unwrap();
    (x + napin_leveys(teksti) / 2, TASO_Y + NAPPI_K / 2)
}

/// Alkuruudun malli: piikikäs merimiina.
pub fn miinamalli() -> Vec<Tahko> {
    const RENKAITA: usize = 5;
    const KULMIA: usize = 10;
    const SADE: f32 = 2.2;
    const KESKI_Y: f32 = 1.2;
    let piste = |rengas: usize, kulma: usize| {
        let leveys = rengas as f32 / RENKAITA as f32 * TAU / 2.0;
        let (sin_l, cos_l) = leveys.sin_cos();
        let (sin_k, cos_k) = (kulma as f32 / KULMIA as f32 * TAU).sin_cos();
        [
            SADE * sin_l * cos_k,
            KESKI_Y + SADE * cos_l,
            SADE * sin_l * sin_k,
        ]
    };
    let mut tahkot = Vec::new();
    for rengas in 0..RENKAITA {
        for kulma in 0..KULMIA {
            let seuraava = (kulma + 1) % KULMIA;
            let vari = if (rengas + kulma) % 2 == 0 {
                0x46505A
            } else {
                0x2E363E
            };
            tahkot.push(tahko4(
                piste(rengas, kulma),
                piste(rengas, seuraava),
                piste(rengas + 1, seuraava),
                piste(rengas + 1, kulma),
                vari,
            ));
        }
    }
    // Piikit kuudessa pääsuunnassa, punaiset kärjet erottuvat pyöriessä.
    // Piikin juuri on pallon sisällä, jottei piikki jää leijumaan irti.
    let (p, s, k) = (0.28, 1.9, 3.0);
    let y = KESKI_Y;
    for (x, yy, z) in [
        ([s, k], [y - p, y + p], [-p, p]),
        ([-k, -s], [y - p, y + p], [-p, p]),
        ([-p, p], [y + s, y + k], [-p, p]),
        ([-p, p], [y - k, y - s], [-p, p]),
        ([-p, p], [y - p, y + p], [s, k]),
        ([-p, p], [y - p, y + p], [-k, -s]),
    ] {
        kuutio(x, yy, z, LIPPU, &mut tahkot);
    }
    tahkot
}

pub fn piirra(peli: &Miinaharava, kuva: &mut [u32]) {
    let mut p = Piirtaja {
        k: Kangas::new(kuva),
        kello: peli.kello_ms,
    };
    if peli.vaihe == Vaihe::Alku {
        p.miinan_alkuruutu(peli);
    } else {
        p.miinan_nakyma(peli);
    }
    p.aanikytkin(peli.aanet_paalla);
}

impl Piirtaja<'_> {
    fn miinan_alkuruutu(&mut self, peli: &Miinaharava) {
        self.maisema(kaanna("miinaharava.banneri"), &miinamalli());
        let y = self.alkuohje(&ALKU_OSAT, &ALKU_RIVIT) + RIVIN_KORKEUS / 2;
        let taso = peli.vaikeus;
        let kuvaus = muotoile(
            "miinaharava.kuvaus",
            &[&taso.leveys(), &taso.korkeus(), &taso.miinoja()],
        );
        self.teksti(48, y, &kuvaus, KELTAINEN);

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

    fn miinan_nakyma(&mut self, peli: &Miinaharava) {
        self.pelitausta();
        self.miinakentta(peli);
        let otsikko = kaanna("peli.miinaharava").to_uppercase();
        self.otsikkopalkki(&otsikko, "");
        self.miinan_paneeli(peli);
    }

    fn miinakentta(&mut self, peli: &Miinaharava) {
        let taso = peli.vaikeus;
        let koko = ruudun_koko(taso);
        let (ox, oy) = origo(taso);
        let (leveys, korkeus) = (taso.leveys(), taso.korkeus());
        self.upotus(ox, oy, koko * leveys as i32, koko * korkeus as i32);
        let hiiri = if peli.vaihe == Vaihe::Pelissa && !peli.kursori_nakyy {
            peli.hiiren_ruutu
        } else {
            None
        };
        for rivi in 0..korkeus {
            for sarake in 0..leveys {
                let (x, y) = (ox + sarake as i32 * koko, oy + rivi as i32 * koko);
                let kohta = (rivi, sarake);
                self.miinaruutu(peli, kohta, (x, y), koko, hiiri == Some(kohta));
            }
        }
        if peli.kursori_nakyy && peli.vaihe == Vaihe::Pelissa && self.vilkku() {
            let (rivi, sarake) = peli.kursori;
            let (x, y) = (ox + sarake as i32 * koko, oy + rivi as i32 * koko);
            self.k.kehys(x, y, koko, koko, KELTAINEN);
        }
        if let (Some(alku), Some((rivi, sarake))) = (peli.rajahdys, peli.lauta.rajahtanyt) {
            let ika = self.kello.saturating_sub(alku) as f32 / RAJAHDYS_MS as f32;
            let keski = (
                ox + sarake as i32 * koko + koko / 2,
                oy + rivi as i32 * koko + koko / 2,
            );
            self.rajahdys(keski, ika, 16, 150.0);
        }
    }

    fn miinaruutu(
        &mut self,
        peli: &Miinaharava,
        kohta: Paikka,
        (x, y): (i32, i32),
        koko: i32,
        korostettu: bool,
    ) {
        let ruutu = peli.lauta.ruutu(kohta);
        let keski = (x + koko / 2, y + koko / 2);
        let lopussa = peli.lauta.tila == Tila::Tappio;
        if ruutu.auki {
            self.avattu(x, y, koko, AVATTU);
            if ruutu.naapureita > 0 {
                self.miinan_numero(keski, koko, ruutu.naapureita);
            }
            return;
        }
        if lopussa && ruutu.miina && !ruutu.lippu {
            let tausta = if peli.lauta.rajahtanyt == Some(kohta) {
                RAJAHTANYT
            } else {
                AVATTU
            };
            self.avattu(x, y, koko, tausta);
            self.miina(keski, koko);
            return;
        }
        self.suljettu(x, y, koko, korostettu);
        if ruutu.lippu {
            self.lippu(keski, koko);
            if lopussa && !ruutu.miina {
                self.vaara_lippu(x, y, koko);
            }
        }
    }

    /// Avattu ruutu on kuoppa: varjo ylä- ja vasemmassa reunassa.
    fn avattu(&mut self, x: i32, y: i32, koko: i32, vari: u32) {
        self.k.laatikko(x, y, koko, koko, vari);
        self.k.laatikko(x, y, koko, 3, AVATTU_REUNA);
        self.k.laatikko(x, y, 3, koko, AVATTU_REUNA);
    }

    /// Kohoava kivilaatta: viisteet, kiilto ja liuku.
    fn suljettu(&mut self, x: i32, y: i32, koko: i32, korostettu: bool) {
        let vari = if korostettu { KOROSTUS } else { SULJETTU };
        let p = 3;
        self.k.laatikko(x, y, koko, koko, tummenna(vari, 0.7));
        self.k.liukulaatikko(
            x,
            y,
            koko - p,
            koko - p,
            vaalenna(vari, 0.18),
            tummenna(vari, 0.2),
        );
        self.k.laatikko(x, y, koko - p, p, vaalenna(vari, 0.6));
        self.k.laatikko(x, y, p, koko - p, vaalenna(vari, 0.4));
    }

    fn miinan_numero(&mut self, (x, y): (i32, i32), koko: i32, numero: u8) {
        let kerroin = if koko >= 60 { 5 } else { 3 };
        let sivu = 8 * kerroin;
        let vari = NUMEROT[numero as usize - 1];
        let teksti = numero.to_string();
        self.teksti_koossa(x - sivu / 2, y - sivu / 2, &teksti, kerroin, vari);
    }

    /// Kerroin, jolla 11x11-kuva mahtuu ruutuun, ja kuvan kulma ruudun keskeltä.
    fn kuvan_paikka(keski: (i32, i32), koko: i32) -> ((i32, i32), i32) {
        let kerroin = (koko / 11).max(1);
        let puoli = 11 * kerroin / 2;
        ((keski.0 - puoli, keski.1 - puoli), kerroin)
    }

    fn miina(&mut self, keski: (i32, i32), koko: i32) {
        let (kulma, kerroin) = Self::kuvan_paikka(keski, koko);
        self.k
            .sprite(kulma, &MIINAN_KUVA, kerroin, |merkki| match merkki {
                b'#' => Some(0x586078),
                b'x' => Some(0x282C40),
                b'o' => Some(VALKOINEN),
                b'r' => Some(LIPPU),
                _ => None,
            });
    }

    fn lippu(&mut self, keski: (i32, i32), koko: i32) {
        let (kulma, kerroin) = Self::kuvan_paikka(keski, koko);
        self.k
            .sprite(kulma, &LIPUN_KUVA, kerroin, |merkki| match merkki {
                b'#' => Some(0x181828),
                b'r' => Some(LIPPU),
                b'R' => Some(0xFF8870),
                _ => None,
            });
    }

    /// Pelin päätyttyä väärä lippu saa ristin.
    fn vaara_lippu(&mut self, x: i32, y: i32, koko: i32) {
        let (a, b) = (x + 4, x + koko - 5);
        let (c, d) = (y + 4, y + koko - 5);
        self.k.viiva((a, c), (b, d), VALKOINEN);
        self.k.viiva((a, d), (b, c), VALKOINEN);
    }

    fn miinan_paneeli(&mut self, peli: &Miinaharava) {
        self.paneelin_tausta();
        self.teksti(TEKSTI_X, PANEELI_RIVI_Y, &peli.viesti, VALKOINEN);

        let paras = peli
            .paras_aika()
            .map_or("-".to_string(), |aika| aika.to_string());
        let tilasto = [
            format!(
                "{} {}",
                kaanna("miinaharava.miinat"),
                peli.lauta.miinoja_jaljella()
            ),
            format!("{} {}", kaanna("miinaharava.aika"), peli.aika()),
            format!("{} {paras}", kaanna("yhteinen.paras")),
        ];
        for (i, teksti) in tilasto.iter().enumerate() {
            let x = TEKSTI_X + i as i32 * TILASTON_ASKEL;
            self.suurteksti(x, TILASTON_Y, teksti, OTSIKKO, KELTAINEN, false);
        }

        let alarivi = PANEELI_RIVI_Y + (PANEELI_RIVEJA as i32 - 1) * RIVIN_KORKEUS;
        let otsikko_y = alarivi - RIVIN_KORKEUS - 12;
        let otsikko = match peli.vaihe {
            Vaihe::Voitto => Some("yhteinen.voitit"),
            Vaihe::Tappio => Some("miinaharava.tappio"),
            Vaihe::Pelissa | Vaihe::Alku => None,
        };
        if let Some(otsikko) = otsikko {
            self.suurteksti(TEKSTI_X, otsikko_y, kaanna(otsikko), TULOS, KELTAINEN, true);
            for (_, teksti, x) in NAPIT {
                self.nappi(x, NAPPI_Y, teksti, VALKOINEN, WB_SININEN);
            }
        } else {
            self.teksti(TEKSTI_X, alarivi, kaanna(OHJE), HUTI);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yhteinen::kieli::kaikilla_kielilla;
    use crate::yhteinen::syote::Syote;
    use klassikot8000::miinaharava::Miinakentta;

    #[test]
    fn kaikki_laudat_mahtuvat_kuvaan() {
        for taso in Vaikeus::KAIKKI {
            let (x, y) = origo(taso);
            let koko = ruudun_koko(taso);
            assert!(x > 0);
            assert!(y >= ALUE_Y);
            assert!(x + koko * (taso.leveys() as i32) < LEVEYS as i32);
            assert!(y + koko * taso.korkeus() as i32 <= ALUE_Y + ALUE_K);
        }
    }

    #[test]
    fn ruudut_loytyvat_ja_ulkopuoli_ei() {
        for taso in Vaikeus::KAIKKI {
            let (ox, oy) = origo(taso);
            for rivi in 0..taso.korkeus() {
                for sarake in 0..taso.leveys() {
                    let (x, y) = ruudun_keskipiste(taso, (rivi, sarake));
                    assert_eq!(ruutu_pisteesta(taso, x, y), Some((rivi, sarake)));
                }
            }
            assert_eq!(ruutu_pisteesta(taso, ox - 1, oy), None);
            assert_eq!(ruutu_pisteesta(taso, ox, oy - 1), None);
            let koko = ruudun_koko(taso);
            assert_eq!(
                ruutu_pisteesta(taso, ox, oy + koko * taso.korkeus() as i32),
                None
            );
            assert_eq!(
                ruutu_pisteesta(taso, ox + koko * taso.leveys() as i32, oy),
                None
            );
        }
    }

    #[test]
    fn tekstit_mahtuvat_paikoilleen() {
        kaikilla_kielilla(|| {
            for (taso, teksti, x) in TASOT {
                assert!(x + napin_leveys(teksti) < LEVEYS as i32, "{taso:?}");
                assert_eq!(vaikeus_pisteesta(x, TASO_Y), Some(taso));
            }
            for pari in NAPIT.windows(2) {
                assert!(pari[0].2 + napin_leveys(pari[0].1) < pari[1].2);
            }
            for viesti in [
                kaanna(OHJE).to_string(),
                kaanna("miinaharava.aloita").to_string(),
                kaanna("miinaharava.pamaus").to_string(),
                muotoile("miinaharava.ennatys", &[&999]),
                muotoile("miinaharava.aika_viesti", &[&999]),
                muotoile("miinaharava.kuvaus", &[&30, &16, &99]),
            ] {
                assert!(viesti.chars().count() <= RIVILLE_MERKKEJA, "{viesti}");
            }
            let otsikot = [kaanna("yhteinen.voitit"), kaanna("miinaharava.tappio")];
            for otsikko in otsikot {
                assert!(otsikko.chars().count() as i32 * TULOS.2 < LEVEYS as i32 - TEKSTI_X);
            }
            // Pisin tilasto ei saa ulottua seuraavan päälle.
            for avain in ["miinaharava.miinat", "miinaharava.aika", "yhteinen.paras"] {
                let merkkeja = kaanna(avain).chars().count() as i32 + 1 + 4;
                assert!(merkkeja * OTSIKKO.2 < TILASTON_ASKEL, "{avain}");
            }
        });
        // Kuvausrivi tulee näppäinohjeiden jälkeen.
        let rivit: Vec<&str> = ALKU_RIVIT.iter().copied().chain([OHJE]).collect();
        alkuohje_mahtuu(&ALKU_OSAT, &rivit, TASO_Y);
    }

    #[test]
    fn jokainen_ruudun_tila_piirtyy_kaatumatta_kaikilla_tasoilla() {
        let mut kuva = vec![0; LEVEYS * KORKEUS];
        for taso in Vaikeus::KAIKKI {
            let mut peli = Miinaharava::new(1);
            peli.vaikeus = taso;
            piirra(&peli, &mut kuva);
            peli.kasittele(Syote::Ammu);
            piirra(&peli, &mut kuva);
            peli.avaa((0, 0));
            peli.liputa((taso.korkeus() - 1, taso.leveys() - 1));
            let (x, y) = ruudun_keskipiste(taso, (1, 1));
            peli.kasittele(Syote::Hiiri(x, y));
            piirra(&peli, &mut kuva);
            peli.kasittele(Syote::Nuoli(0, 1));
            peli.kasittele(Syote::Nuoli(0, 1));
            piirra(&peli, &mut kuva);
            // Kaikki numerot, väärä lippu ja räjähdys samalla laudalla.
            peli.lauta = Miinakentta::miinoista(
                taso,
                &[
                    (0, 0),
                    (0, 1),
                    (0, 2),
                    (1, 0),
                    (1, 2),
                    (2, 0),
                    (2, 1),
                    (2, 2),
                    (5, 5),
                ],
            );
            peli.avaa((1, 1));
            peli.liputa((4, 4));
            peli.avaa((5, 5));
            assert_eq!(peli.vaihe, Vaihe::Tappio);
            peli.edista(100);
            piirra(&peli, &mut kuva);
            peli.edista(10_000);
            piirra(&peli, &mut kuva);
            peli.vaihe = Vaihe::Voitto;
            piirra(&peli, &mut kuva);
        }
    }
}
