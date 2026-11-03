use std::f32::consts::PI;
use std::sync::LazyLock;

use klassikot8000::laivanupotus::{
    KOKO, Laiva, Laivasto, Laukaustulos, Ruudut, Ruutu, SARAKKEET, Vaikeus,
};

use super::peli::{Peli, TULOS_VIIVE_MS, Tehoste, Vaihe, laivan_nimi, tason_numero};
use crate::yhteinen::kieli::{kaanna, muotoile};
use crate::yhteinen::piirto::*;
use crate::yhteinen::rasteri::{Rasteri, liukuvari};
use crate::yhteinen::vektori::{self, Kangas, sekoita_liuku, tummenna, vaalenna};

const SOLU: i32 = 33;
const LAUDAN_KOKO: i32 = SOLU * KOKO as i32;
const LAUTA_Y: i32 = 99;
const VASTUSTAJA_X: i32 = 90;
const OMA_X: i32 = 570;
const OTSIKKO_Y: i32 = 48;
const TILASTO_Y: i32 = 441;
const TILASTO_SISENNYS: i32 = 48;
const TILASTO_PALKKI_X: i32 = 14 * MERKKI;

const TEHOSTE_KESTO_MS: f32 = super::peli::TEHOSTE_KESTO_MS as f32;
/// Nostetun laivan kehys: vihreä käy, punainen koskettaisi toista laivaa.
const KELPAA: u32 = 0x58E070;
const EI_KELPAA: u32 = OSUMA;
const MERKINTA: (i32, i32, i32) = (12, 18, 18);

// Vesi on paikallaan pysyvä valmiiksi laskettu kuva, jotta laivat ja merkit erottuvat rauhassa.
const LAUDAN_RIVEJA: i32 = LAUDAN_KOKO / 3;
const SOLUN_RIVEJA: i32 = SOLU / 3;

static VESI_KUVA: LazyLock<Rasteri> = LazyLock::new(|| {
    let n = LAUDAN_RIVEJA as usize;
    Rasteri::luo(n, n, |x, y| {
        let t = y as f32 / LAUDAN_RIVEJA as f32;
        let vari = liukuvari(&[0x1450B0, 0x0C2C80], t, x, y);
        // Ruudukko on pisteviiva, jotta se ei syö pieniä soluja.
        if (x % SOLUN_RIVEJA == 0 || y % SOLUN_RIVEJA == 0) && (x + y) % 2 == 0 {
            sekoita_liuku(vari, 0xA0C8FF, 0.45)
        } else {
            vari
        }
    })
});

const OHJE: &str = "laivanupotus.ohje";
const AMMUIT_JO: &str = "laivanupotus.ammuit_jo";
const VOITTO: &str = "yhteinen.voitit";
const TAPPIO: &str = "laivanupotus.tappio";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nappi {
    Uudestaan,
    Lopeta,
    Arvo,
    Taisteluun,
}

const NAPIT: [(Nappi, &str, i32); 2] = [
    (Nappi::Uudestaan, "yhteinen.uudestaan", NAPPI_X),
    (
        Nappi::Lopeta,
        "yhteinen.valikkoon",
        NAPPI_X + 13 * MERKKI + 48,
    ),
];

const SIJOITUSNAPIT: [(Nappi, &str, i32); 2] = [
    (Nappi::Arvo, "laivanupotus.arvo", NAPPI_X),
    (
        Nappi::Taisteluun,
        "laivanupotus.taisteluun",
        NAPPI_X + 13 * MERKKI + 48,
    ),
];
const SIJOITUS_OHJEET: [&str; 3] = [
    "laivanupotus.sijoitus_1",
    "laivanupotus.sijoitus_2",
    "laivanupotus.sijoitus_3",
];
const TILASTO: &str = "laivanupotus.tilasto";
const ENNATYS: &str = "laivanupotus.ennatys";

const TASO_OTSIKKO: &str = "yhteinen.taso";
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
    "laivanupotus.tervetuloa",
    "yhteinen.ohjeet",
    "laivanupotus.ohje_1",
    "laivanupotus.ohje_2",
];
const ALKU_RIVIT: [&str; 5] = [
    "laivanupotus.nappain_hiiri",
    "laivanupotus.nappain_nuolet",
    "laivanupotus.nappain_ammu",
    "yhteinen.aanet_kytkin",
    "yhteinen.takaisin",
];

fn vaikeuden_nimi(vaikeus: Vaikeus) -> &'static str {
    kaanna(match vaikeus {
        Vaikeus::Helppo => "yhteinen.helppo",
        Vaikeus::Normaali => "yhteinen.normaali",
        Vaikeus::Vaikea => "yhteinen.vaikea",
    })
}

pub fn vaikeus_pisteesta(x: i32, y: i32) -> Option<Vaikeus> {
    TASOT
        .iter()
        .find(|&&(_, teksti, nx)| {
            (nx..nx + napin_leveys(teksti)).contains(&x) && (TASO_Y..TASO_Y + NAPPI_K).contains(&y)
        })
        .map(|&(vaikeus, ..)| vaikeus)
}

#[cfg(test)]
pub fn vaikeuden_keskipiste(vaikeus: Vaikeus) -> (i32, i32) {
    let (_, teksti, x) = TASOT.iter().find(|t| t.0 == vaikeus).copied().unwrap();
    (x + napin_leveys(teksti) / 2, TASO_Y + NAPPI_K / 2)
}

fn laudan_ruutu(x0: i32, x: i32, y: i32) -> Option<Ruutu> {
    let (dx, dy) = (x - x0, y - LAUTA_Y);
    if dx < 0 || dy < 0 {
        return None;
    }
    Ruutu::new((dy / SOLU) as usize, (dx / SOLU) as usize)
}

/// Vastustajan laudan ruutu annetussa pikselissä.
pub fn ruutu_pisteesta(x: i32, y: i32) -> Option<Ruutu> {
    laudan_ruutu(VASTUSTAJA_X, x, y)
}

/// Oman laudan ruutu annetussa pikselissä.
pub fn oma_ruutu_pisteesta(x: i32, y: i32) -> Option<Ruutu> {
    laudan_ruutu(OMA_X, x, y)
}

/// Sijoituksen napit ovat samalla paikalla kuin lopun napit.
fn vaiheen_napit(vaihe: Vaihe) -> &'static [(Nappi, &'static str, i32)] {
    if vaihe == Vaihe::Sijoitus {
        &SIJOITUSNAPIT
    } else {
        &NAPIT
    }
}

pub fn nappi_pisteesta(vaihe: Vaihe, x: i32, y: i32) -> Option<Nappi> {
    vaiheen_napit(vaihe)
        .iter()
        .find(|&&(_, teksti, nx)| {
            let leveys = napin_leveys(teksti);
            (nx..nx + leveys).contains(&x) && (NAPPI_Y..NAPPI_Y + NAPPI_K).contains(&y)
        })
        .map(|&(nappi, ..)| nappi)
}

#[cfg(test)]
pub fn ruudun_keskipiste(ruutu: Ruutu) -> (i32, i32) {
    solun_keskipiste(VASTUSTAJA_X, ruutu)
}

#[cfg(test)]
pub fn oman_ruudun_keskipiste(ruutu: Ruutu) -> (i32, i32) {
    solun_keskipiste(OMA_X, ruutu)
}

/// Vaakasuora, kun laivan kaksi ensimmäistä ruutua ovat samalla rivillä.
fn laivan_suunta(ruudut: Ruudut) -> bool {
    Laiva { nimi: "", ruudut }.vaaka()
}

fn solun_keskipiste(x0: i32, ruutu: Ruutu) -> (i32, i32) {
    (
        x0 + ruutu.sarake as i32 * SOLU + SOLU / 2,
        LAUTA_Y + ruutu.rivi as i32 * SOLU + SOLU / 2,
    )
}

#[cfg(test)]
pub fn napin_keskipiste(nappi: Nappi) -> (i32, i32) {
    let (_, teksti, x) = NAPIT
        .iter()
        .chain(&SIJOITUSNAPIT)
        .find(|n| n.0 == nappi)
        .copied()
        .unwrap();
    (x + napin_leveys(teksti) / 2, NAPPI_Y + NAPPI_K / 2)
}

pub fn piirra(peli: &Peli, kuva: &mut [u32]) {
    let mut p = Piirtaja {
        k: Kangas::new(kuva),
        kello: peli.kello_ms,
    };
    if peli.vaihe == Vaihe::Alku {
        p.alkuruutu(peli.vaikeus);
    } else {
        p.pelinakyma(peli);
    }
    p.aanikytkin(peli.aanet_paalla);
}

/// Alkuruudun ja käynnistimen malli.
pub fn alkumalli() -> Vec<vektori::Tahko> {
    vektori::taistelulaiva(RUNKO, KANSI, TORNI)
}

impl Piirtaja<'_> {
    fn alkuruutu(&mut self, vaikeus: Vaikeus) {
        self.maisema(kaanna("laivanupotus.banneri"), &alkumalli());

        self.alkuohje(&ALKU_OSAT, &ALKU_RIVIT);
        self.tasovalinta(vaikeus);
        self.aloituskehote(ALOITUS);
    }

    fn tasovalinta(&mut self, valittu: Vaikeus) {
        self.teksti(48, TASO_Y + 3, kaanna(TASO_OTSIKKO), KELTAINEN);
        for (vaikeus, teksti, x) in TASOT {
            if vaikeus == valittu {
                self.nappi(x, TASO_Y, teksti, WB_ORANSSI, MUSTA);
            } else {
                self.nappi(x, TASO_Y, teksti, VALKOINEN, WB_SININEN);
            }
        }
    }

    fn pelinakyma(&mut self, peli: &Peli) {
        self.pelitausta();
        self.otsikkopalkki(kaanna("peli.laivanupotus"), vaikeuden_nimi(peli.vaikeus));

        self.suurteksti(
            VASTUSTAJA_X,
            OTSIKKO_Y,
            kaanna("laivanupotus.vastustajan_lauta"),
            OTSIKKO,
            VALKOINEN,
            false,
        );
        let oma_lauta = kaanna("laivanupotus.oma_lauta");
        self.suurteksti(OMA_X, OTSIKKO_Y, oma_lauta, OTSIKKO, VALKOINEN, false);
        self.lauta(VASTUSTAJA_X, &peli.vastustaja, false, peli);
        self.lauta(OMA_X, &peli.oma, true, peli);
        match peli.vaihe {
            Vaihe::Ammunta => self.tahtain(VASTUSTAJA_X, peli.kursori),
            Vaihe::Sijoitus if peli.nostettu.is_none() => self.tahtain(OMA_X, peli.kursori),
            _ => {}
        }
        self.tilasto(VASTUSTAJA_X - TILASTO_SISENNYS, &peli.vastustaja);
        self.tilasto(OMA_X - TILASTO_SISENNYS, &peli.oma);
        self.paneeli(peli);
        for tehoste in &peli.tehosteet {
            self.tehoste(tehoste);
        }
    }

    /// Laudan alle: montako laivaa on upotettu ja mitkä. Osumia ei kerrota,
    /// koska vastustajan laivoista pelaajan kuuluu tietää vain uponneet.
    fn tilasto(&mut self, x: i32, laivasto: &Laivasto) {
        let otsikko = muotoile(
            "laivanupotus.upotettu",
            &[&laivasto.upotettuja(), &laivasto.laivat().len()],
        );
        self.teksti(x, TILASTO_Y, &otsikko, KELTAINEN);
        let mut y = TILASTO_Y + RIVIN_KORKEUS + 3;
        for laiva in laivasto.laivat() {
            let upotettu = laivasto.on_upotettu(laiva);
            let (teksti_vari, pala_vari) = if upotettu {
                (WB_ORANSSI, OSUMA)
            } else {
                (VALKOINEN, KANSI)
            };
            let nimi = laivan_nimi(laiva.nimi);
            self.teksti(x, y, nimi, teksti_vari);
            if upotettu {
                // yliviivaus, jottei tieto jää pelkän värin varaan
                let leveys = nimi.chars().count() as i32 * MERKKI;
                self.k.laatikko(x, y + 9, leveys, 3, teksti_vari);
            }
            for i in 0..laiva.ruudut.maara() as i32 {
                let pala_x = x + TILASTO_PALKKI_X + i * PALKIN_PALA;
                self.pala(pala_x, y + 3, PALKIN_PALA - 3, 15, pala_vari);
            }
            y += RIVIN_KORKEUS;
        }
    }

    fn lauta(&mut self, x0: i32, laivasto: &Laivasto, omalle: bool, peli: &Peli) {
        for (i, kirjain) in SARAKKEET.chars().enumerate() {
            let x = x0 + i as i32 * SOLU + 10;
            let mut puskuri = [0; 4];
            let merkki = kirjain.encode_utf8(&mut puskuri);
            self.suurteksti(x, LAUTA_Y - 22, merkki, MERKINTA, REUNA, false);
        }
        for rivi in 0..KOKO {
            let y = LAUTA_Y + rivi as i32 * SOLU + 7;
            let tx = if rivi == 9 { x0 - 39 } else { x0 - 21 };
            let numero = (rivi + 1).to_string();
            self.suurteksti(tx, y, &numero, MERKINTA, REUNA, false);
        }

        VESI_KUVA.piirra(&mut self.k, x0, LAUTA_Y);
        let nostettu = peli.nostettu.map(|n| n.laiva);
        if omalle {
            for (i, laiva) in laivasto.laivat().iter().enumerate() {
                if nostettu != Some(i) {
                    self.laiva(x0, laiva);
                }
            }
        }
        let tyhjat = if omalle {
            Default::default()
        } else {
            peli.varmasti_tyhjat()
        };
        for rivi in 0..KOKO {
            for sarake in 0..KOKO {
                if let Some(ruutu) = Ruutu::new(rivi, sarake) {
                    let ammuttu = laivasto.ammutut().sisaltaa(ruutu);
                    if ammuttu && !peli.odottaa_osumaa(omalle, ruutu) {
                        self.laukaus(x0, ruutu, laivasto.on_laiva(ruutu));
                    } else if tyhjat.sisaltaa(ruutu) {
                        let keski = solun_keskipiste(x0, ruutu);
                        self.k.laatikko(keski.0 - 2, keski.1 - 2, 5, 5, HUTI);
                    }
                }
            }
        }
        if omalle && let Some((i, ruudut, kelpaa)) = peli.haamu() {
            self.nostettu_laiva(x0, laivasto.laivat()[i].nimi, ruudut, kelpaa);
        }
        self.upotus(x0, LAUTA_Y, LAUDAN_KOKO, LAUDAN_KOKO);
    }

    /// Nostettu laiva kursorin kohdalla ja kehys, joka kertoo, käykö paikka.
    fn nostettu_laiva(&mut self, x0: i32, nimi: &'static str, ruudut: Ruudut, kelpaa: bool) {
        self.laiva(x0, &Laiva { nimi, ruudut });
        let Some(alku) = ruudut.ensimmainen() else {
            return;
        };
        let pituus = ruudut.maara() as i32 * SOLU;
        let vaaka = laivan_suunta(ruudut);
        let (leveys, korkeus) = if vaaka {
            (pituus, SOLU)
        } else {
            (SOLU, pituus)
        };
        let x = x0 + alku.sarake as i32 * SOLU;
        let y = LAUTA_Y + alku.rivi as i32 * SOLU;
        let vari = if kelpaa { KELPAA } else { EI_KELPAA };
        self.k.kehys(x, y, leveys + 1, korkeus + 1, vari);
        self.k.kehys(x + 3, y + 3, leveys - 5, korkeus - 5, vari);
    }

    /// Laiva ylhäältä: suippo keula, kansirakenne ja tykkitornit joka ruudussa.
    fn laiva(&mut self, x0: i32, laiva: &Laiva) {
        let mut rajat: Option<(i32, i32, i32)> = None;
        for rivi in 0..KOKO {
            for sarake in 0..KOKO {
                let Some(ruutu) = Ruutu::new(rivi, sarake) else {
                    continue;
                };
                if laiva.ruudut.sisaltaa(ruutu) {
                    let (r, s) = (rivi as i32, sarake as i32);
                    rajat = Some(match rajat {
                        Some((r0, s0, r1)) => (r0.min(r), s0.min(s), r1.max(r)),
                        None => (r, s, r),
                    });
                }
            }
        }
        let Some((r0, s0, r1)) = rajat else {
            return;
        };
        let vaaka = r0 == r1;
        let pituus = laiva.ruudut.maara() as i32 * SOLU;
        let (vasen, yla) = (x0 + s0 * SOLU, LAUTA_Y + r0 * SOLU);
        // u kulkee perästä keulaan, v laidasta laitaan
        let p = |u: i32, v: i32| {
            if vaaka {
                (vasen + u, yla + SOLU / 2 + v)
            } else {
                (vasen + SOLU / 2 + v, yla + u)
            }
        };
        let runko = [
            p(3, -8),
            p(pituus - 18, -11),
            p(pituus - 3, 0),
            p(pituus - 18, 11),
            p(3, 8),
        ];
        // Tumma ääriviiva, valoisa kansi ja varjoinen laita.
        self.k.monikulmio(&runko, tummenna(RUNKO, 0.75));
        let sisa = [
            p(6, -6),
            p(pituus - 20, -8),
            p(pituus - 8, 0),
            p(pituus - 20, 8),
            p(6, 6),
        ];
        self.k.monikulmio(&sisa, RUNKO);
        let valo = [p(6, -6), p(pituus - 20, -8), p(pituus - 8, 0), p(6, 0)];
        self.k.monikulmio(&valo, KANSI);
        let kansi = pituus / 3;
        let rakenne = [
            p(kansi, -4),
            p(pituus - kansi - 3, -4),
            p(pituus - kansi - 3, 4),
            p(kansi, 4),
        ];
        self.k.monikulmio(&rakenne, TORNI);
        // Jokaisessa ruudussa on tykkitorni, jonka putki osoittaa keulaan.
        for osa in 0..laiva.ruudut.maara() as i32 {
            let u = osa * SOLU + SOLU / 2;
            let (tx, ty) = p(u, 0);
            self.k.viiva(p(u, 0), p(u + 12, 0), 0x303848);
            self.k
                .laatikko(tx / 3 * 3 - 3, ty / 3 * 3 - 3, 9, 9, 0x303848);
            self.k.laatikko(tx / 3 * 3, ty / 3 * 3, 3, 3, LAIVA_REUNA);
        }
    }

    fn laukaus(&mut self, x0: i32, ruutu: Ruutu, osui: bool) {
        let keski = solun_keskipiste(x0, ruutu);
        if osui {
            let x = x0 + ruutu.sarake as i32 * SOLU + 1;
            let y = LAUTA_Y + ruutu.rivi as i32 * SOLU + 1;
            self.k.laatikko(x, y, SOLU - 1, SOLU - 1, OSUMA_TUMMA);
            // Liekki väpättää, mutta eri ruudut eri tahtiin.
            let tahti = self.kello / 130 + ruutu.rivi as u32 * 7 + ruutu.sarake as u32 * 3;
            for i in 0..5 {
                let rivi = i as i32 * 6;
                let levein = [15, 12, 9, 6, 3][i as usize];
                let huojunta = ((tahti + i) % 3) as i32 * 3 - 3;
                let kx = keski.0 + huojunta / 2;
                let y = keski.1 + 12 - rivi;
                self.k.laatikko(kx - levein, y, 2 * levein, 6, OSUMA);
                self.k
                    .laatikko(kx - levein * 2 / 3, y, levein * 4 / 3, 6, WB_ORANSSI);
                self.k
                    .laatikko(kx - levein / 3, y, levein * 2 / 3, 6, KELTAINEN);
            }
        } else {
            self.k.ympyra(keski, 9.0, vaalenna(HUTI, 0.3));
            self.k
                .laatikko(keski.0 / 3 * 3, keski.1 / 3 * 3, 3, 3, VALKOINEN);
        }
    }

    /// Tähtäin: kulmat ja keskelle osoittavat viivat sykkivät.
    fn tahtain(&mut self, x0: i32, kursori: Ruutu) {
        let keski = solun_keskipiste(x0, kursori);
        let vari = if self.vilkku() { KELTAINEN } else { WB_ORANSSI };
        let (sade, kulma) = (SOLU / 2 + 1, 8);
        let (cx, cy) = keski;
        for (sx, sy) in [(-1, -1), (1, -1), (1, 1), (-1, 1)] {
            let nurkka = (cx + sx * sade, cy + sy * sade);
            self.k
                .viiva(nurkka, (nurkka.0 - sx * kulma, nurkka.1), vari);
            self.k
                .viiva(nurkka, (nurkka.0, nurkka.1 - sy * kulma), vari);
        }
        let sisa = 5 + (self.kello / 150 % 3) as i32 * 2;
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            self.k.viiva(
                (cx + dx * sisa, cy + dy * sisa),
                (cx + dx * (sisa + 6), cy + dy * (sisa + 6)),
                vari,
            );
        }
    }

    fn tehoste(&mut self, tehoste: &Tehoste) {
        if self.kello < tehoste.laukaus_ms {
            return;
        }
        let (x0, lahto_x) = if tehoste.omalle {
            (OMA_X, VASTUSTAJA_X)
        } else {
            (VASTUSTAJA_X, OMA_X)
        };
        let kohde = solun_keskipiste(x0, tehoste.ruutu);
        let kulunut = self.kello - tehoste.laukaus_ms;
        if kulunut < TULOS_VIIVE_MS {
            let lahto = (lahto_x + LAUDAN_KOKO / 2, LAUTA_Y + LAUDAN_KOKO / 2);
            self.ammus(lahto, kohde, kulunut as f32 / TULOS_VIIVE_MS as f32);
            return;
        }
        let ika = (kulunut - TULOS_VIIVE_MS) as f32 / TEHOSTE_KESTO_MS;
        match tehoste.tulos {
            Laukaustulos::Ohi => self.roiske(kohde, ika),
            Laukaustulos::Osuma => self.rajahdys(kohde, ika, 12, 42.0),
            Laukaustulos::Upposi(_) => self.rajahdys(kohde, ika, 24, 105.0),
        }
    }

    /// Ammus lentää kaarena toiselta laudalta; häntä näyttää suunnan.
    fn ammus(&mut self, lahto: (i32, i32), kohde: (i32, i32), t: f32) {
        let paikka = |t: f32| {
            let x = lahto.0 as f32 + (kohde.0 - lahto.0) as f32 * t;
            let y = lahto.1 as f32 + (kohde.1 - lahto.1) as f32 * t;
            (x as i32, (y - (t * PI).sin() * 90.0) as i32)
        };
        let karki = paikka(t);
        self.k.viiva(paikka((t - 0.18).max(0.0)), karki, WB_ORANSSI);
        self.k.kiiltopallo(karki, 7, 0x708090);
    }

    fn roiske(&mut self, keski: (i32, i32), ika: f32) {
        let vari = sekoita(VALKOINEN, VESI, ika);
        for i in 0..3 {
            let sade = (ika * 39.0 - i as f32 * 9.0).max(0.0);
            if sade > 0.0 {
                self.k.ympyra(keski, sade, vari);
            }
        }
    }

    fn paneeli(&mut self, peli: &Peli) {
        self.paneelin_tausta();

        let vastaus_kesken = peli.vastaus_kesken();
        let mut y = PANEELI_RIVI_Y;
        for (i, viesti) in peli.viestit.iter().enumerate() {
            if i == 1 && vastaus_kesken {
                continue;
            }
            for rivi in rivita(viesti, RIVILLE_MERKKEJA) {
                self.teksti(TEKSTI_X, y, rivi, VALKOINEN);
                y += RIVIN_KORKEUS;
            }
        }

        let alarivi = PANEELI_RIVI_Y + (PANEELI_RIVEJA as i32 - 1) * RIVIN_KORKEUS;
        match peli.vaihe {
            Vaihe::Sijoitus => self.sijoitusohje(peli),
            Vaihe::Ammunta if peli.ammuttu_jo => {
                self.teksti(TEKSTI_X, alarivi, kaanna(AMMUIT_JO), WB_ORANSSI);
            }
            Vaihe::Ammunta => self.teksti(TEKSTI_X, alarivi, kaanna(OHJE), HUTI),
            Vaihe::Voitto | Vaihe::Tappio if !vastaus_kesken => {
                let teksti = if peli.vaihe == Vaihe::Voitto {
                    VOITTO
                } else {
                    TAPPIO
                };
                let y = alarivi - RIVIN_KORKEUS - 2;
                self.suurteksti(TEKSTI_X, y, kaanna(teksti), TULOS, KELTAINEN, true);
                self.napit(&NAPIT);
            }
            Vaihe::Voitto | Vaihe::Tappio | Vaihe::Alku => {}
        }
    }

    /// Ohjeet, tämän tason tilasto ja napit ennen taistelua.
    fn sijoitusohje(&mut self, peli: &Peli) {
        let mut y = PANEELI_RIVI_Y;
        for avain in SIJOITUS_OHJEET {
            self.teksti(TEKSTI_X, y, kaanna(avain), VALKOINEN);
            y += RIVIN_KORKEUS;
        }
        let tilasto = peli.tilastot[tason_numero(peli.vaikeus)];
        let rivi = muotoile(TILASTO, &[&tilasto.voitot, &tilasto.tappiot]);
        self.teksti(TEKSTI_X, y, &rivi, HUTI);
        if tilasto.vahiten_laukauksia > 0 {
            let rivi = muotoile(ENNATYS, &[&tilasto.vahiten_laukauksia]);
            self.teksti(TEKSTI_X, y + RIVIN_KORKEUS, &rivi, HUTI);
        }
        self.napit(&SIJOITUSNAPIT);
    }

    fn napit(&mut self, napit: &[(Nappi, &str, i32)]) {
        for &(_, teksti, x) in napit {
            self.nappi(x, NAPPI_Y, teksti, VALKOINEN, WB_SININEN);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pelit::laivanupotus::peli::tulos_teksti;
    use crate::yhteinen::kieli::kaikilla_kielilla;

    #[test]
    fn jokainen_ruutu_loytyy_keskipisteestaan() {
        for rivi in 0..KOKO {
            for sarake in 0..KOKO {
                let ruutu = Ruutu::new(rivi, sarake).unwrap();
                let (x, y) = ruudun_keskipiste(ruutu);
                assert_eq!(ruutu_pisteesta(x, y), Some(ruutu));
            }
        }
    }

    #[test]
    fn oman_laudan_ruudut_loytyvat_eivatka_osu_vastustajan_laudalle() {
        for rivi in 0..KOKO {
            for sarake in 0..KOKO {
                let ruutu = Ruutu::new(rivi, sarake).unwrap();
                let (x, y) = oman_ruudun_keskipiste(ruutu);
                assert_eq!(oma_ruutu_pisteesta(x, y), Some(ruutu));
                let (x, y) = ruudun_keskipiste(ruutu);
                assert_eq!(oma_ruutu_pisteesta(x, y), None);
            }
        }
    }

    #[test]
    fn oma_lauta_ja_reunat_eivat_ole_kohteita() {
        assert_eq!(ruutu_pisteesta(OMA_X + 5, LAUTA_Y + 5), None);
        assert_eq!(ruutu_pisteesta(VASTUSTAJA_X - 1, LAUTA_Y), None);
        assert_eq!(ruutu_pisteesta(VASTUSTAJA_X, LAUTA_Y + SOLU * 10), None);
    }

    #[test]
    fn pisin_kierros_mahtuu_paneeliin() {
        kaikilla_kielilla(|| {
            let ruutu = Ruutu::new(9, 1).unwrap();
            for (nimi, _) in klassikot8000::laivanupotus::LAIVAT {
                // kaksi viestiä, lopputulos ja napit
                let tulos = Laukaustulos::Upposi(nimi);
                let a = tulos_teksti("laivanupotus.sina_ammuit", ruutu, tulos);
                let b = tulos_teksti("laivanupotus.vastustaja_ampui", ruutu, tulos);
                let rivit = rivita(&a, RIVILLE_MERKKEJA).len() + rivita(&b, RIVILLE_MERKKEJA).len();
                assert!(rivit + 2 <= PANEELI_RIVEJA, "{a} {b}");
                // Nimi ei saa osua tilaston palkkiin.
                let leveys = laivan_nimi(nimi).chars().count() as i32 * MERKKI;
                assert!(leveys < TILASTO_PALKKI_X, "{nimi}");
            }
            assert!(kaanna(OHJE).chars().count() <= RIVILLE_MERKKEJA);
            for rivi in SIJOITUS_OHJEET.map(kaanna).into_iter().chain([
                muotoile(TILASTO, &[&999, &999]).as_str(),
                muotoile(ENNATYS, &[&100]).as_str(),
            ]) {
                assert!(rivi.chars().count() <= RIVILLE_MERKKEJA, "{rivi}");
            }
            for pari in SIJOITUSNAPIT.windows(2) {
                assert!(pari[0].2 + napin_leveys(pari[0].1) < pari[1].2);
            }
            let (_, viimeinen, x) = SIJOITUSNAPIT[1];
            assert!(x + napin_leveys(viimeinen) <= LEVEYS as i32);
            assert!(kaanna(AMMUIT_JO).chars().count() <= RIVILLE_MERKKEJA);
            assert!(kaanna(TASO_OTSIKKO).chars().count() <= 6);
            for pari in NAPIT.windows(2) {
                assert!(pari[0].2 + napin_leveys(pari[0].1) < pari[1].2);
            }
            for pari in TASOT.windows(2) {
                assert!(pari[0].2 + napin_leveys(pari[0].1) < pari[1].2);
            }
        });
        alkuohje_mahtuu(&ALKU_OSAT, &ALKU_RIVIT, TASO_Y);
    }

    #[test]
    fn kaikki_vaiheet_piirtyvat_kaatumatta_tehosteiden_aikana() {
        let mut kuva = vec![0; LEVEYS * KORKEUS];
        let mut peli = Peli::new(3);
        piirra(&peli, &mut kuva);
        peli.vaihe = Vaihe::Sijoitus;
        piirra(&peli, &mut kuva);
        peli.kasittele(crate::yhteinen::syote::Syote::Numero(1));
        peli.tilastot[1].vahiten_laukauksia = 40;
        piirra(&peli, &mut kuva);
        peli.vaihe = Vaihe::Ammunta;
        for i in 0..KOKO * KOKO {
            peli.ammu(Ruutu::new(i / KOKO, i % KOKO).unwrap());
            for _ in 0..4 {
                peli.edista(100);
                piirra(&peli, &mut kuva);
            }
        }
    }
}
