//! Asteroidien näkymät.

use std::sync::LazyLock;

use klassikot8000::asteroidit::{self, Alus, Kivi, Lautanen, Piste};

use super::peli::{Asteroidit, Nappi, RAJAHDYS_MS, Vaihe};
use crate::yhteinen::kieli::kaanna;
use crate::yhteinen::piirto::*;
use crate::yhteinen::rasteri::{Rasteri, kohina, liukuvari};
use crate::yhteinen::vektori::{Kangas, Tahko, sekoita_liuku, tahko4, tummenna, vaalenna};

pub const KENTTA_X: i32 = 30;
pub const KENTTA_Y: i32 = 48;
const KENTTA_L: i32 = asteroidit::LEVEYS as i32;
const KENTTA_K: i32 = asteroidit::KORKEUS as i32;
const SARAKE_X: i32 = KENTTA_X + KENTTA_L + 30;
const TILASTON_ASKEL: i32 = 87;
const PARHAAT_Y: i32 = KENTTA_Y + 4 * TILASTON_ASKEL + 12;
const PARHAAT_L: i32 = LEVEYS as i32 - 12 - SARAKE_X;
// Kapeaan sarakkeeseen mahtuu sijoitus, kahdeksan merkin nimi ja kuusinumeroiset pisteet.
const PARHAAT_KOKO: (i32, i32, i32) = (10, 15, 13);
const KIVEN_VARI: u32 = 0x8C7868;
const ALUKSEN_VARI: u32 = 0xE8ECF8;

/// Avaruus: tumma liuku, violetti kaasusumu ja kolmen kirkkauden tähdet.
static AVARUUS_KUVA: LazyLock<Rasteri> = LazyLock::new(|| {
    let (l, k) = (KENTTA_L / 3, KENTTA_K / 3);
    Rasteri::luo(l as usize, k as usize, move |x, y| {
        let (fx, fy) = (x as f32, y as f32);
        let sumu = (fx * 0.021 + fy * 0.013).sin()
            + (fx * 0.009 - fy * 0.027 + 1.3).sin()
            + (fx * 0.047 + fy * 0.031 + 0.7).sin() * 0.5;
        let vari = liukuvari(
            &[0x02030E, 0x0A0830, 0x2C1458, 0x5A1C70],
            (sumu + 0.6) / 3.0,
            x,
            y,
        );
        match kohina(x, y) {
            n if n > 0.9985 => 0xFFFFFF,
            n if n > 0.994 => 0xA0B8F0,
            n if n > 0.98 => sekoita_liuku(vari, 0x8090D0, 0.5),
            _ => vari,
        }
    })
});
const SUOJAN_VILKKU_MS: u32 = 120;
const LIEKIN_VILKKU_MS: u32 = 80;

const ALUKSEN_MUOTO: [Piste; 4] = [(14.0, 0.0), (-10.0, 8.0), (-6.0, 0.0), (-10.0, -8.0)];
const LIEKKI: [Piste; 3] = [(-7.0, 4.0), (-17.0, 0.0), (-7.0, -4.0)];
// Kivet ovat kymmenkulmioita, joiden kärjet ovat eri etäisyyksillä keskipisteestä.
const KIVEN_MUODOT: [[f32; 10]; 4] = [
    [1.0, 0.8, 0.95, 0.7, 1.0, 0.85, 0.65, 0.95, 0.8, 0.9],
    [0.85, 1.0, 0.7, 0.9, 0.75, 1.0, 0.9, 0.65, 1.0, 0.75],
    [0.95, 0.7, 1.0, 1.0, 0.65, 0.85, 1.0, 0.75, 0.7, 1.0],
    [0.7, 1.0, 0.85, 0.65, 0.95, 1.0, 0.7, 1.0, 0.9, 0.8],
];
const LAUTASEN_MUOTO: [Piste; 8] = [
    (-1.0, 0.0),
    (-0.4, 0.4),
    (0.4, 0.4),
    (1.0, 0.0),
    (0.4, -0.25),
    (0.25, -0.6),
    (-0.25, -0.6),
    (-0.4, -0.25),
];

const OHJE: &str = "asteroidit.ohje";
const TAUKO: &str = "yhteinen.tauko";
const TAUON_OHJE: &str = "yhteinen.tauon_ohje";
const TAPPIO: &str = "yhteinen.peli_paattyi";

const NAPIT: [(Nappi, &str, i32); 2] = [
    (Nappi::Uudestaan, "yhteinen.uudestaan", NAPPI_X),
    (
        Nappi::Valikko,
        "yhteinen.valikkoon",
        NAPPI_X + 13 * MERKKI + 48,
    ),
];

const ALKU_OSAT: [&str; 4] = [
    "asteroidit.tervetuloa",
    "yhteinen.ohjeet",
    "asteroidit.ohje_1",
    "asteroidit.ohje_2",
];
const ALKU_RIVIT: [&str; 5] = [
    "asteroidit.nappain_kaanto",
    "asteroidit.nappain_tyonto",
    "asteroidit.nappain_ammu",
    "asteroidit.nappain_muut",
    "yhteinen.takaisin",
];

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

/// Kenttä jatkuu reunojen yli: laidan yli ulottuva kappale piirretään myös toiselle puolelle.
fn kopiot((x, y): Piste, sade: f32) -> impl Iterator<Item = (i32, i32)> {
    let (l, k) = (asteroidit::LEVEYS, asteroidit::KORKEUS);
    [-l, 0.0, l]
        .into_iter()
        .flat_map(move |dx| [-k, 0.0, k].map(|dy| (x + dx, y + dy)))
        .filter(move |p| p.0 + sade >= 0.0 && p.0 - sade < l && p.1 + sade >= 0.0 && p.1 - sade < k)
        .map(|p| (KENTTA_X + p.0 as i32, KENTTA_Y + p.1 as i32))
}

/// Paikallisen pisteen paikka ruudulla, kun kappale on käännetty kulmaan.
fn ruudulle(keski: (i32, i32), kulma: f32, (x, y): Piste) -> (i32, i32) {
    let (sin, cos) = kulma.sin_cos();
    (
        keski.0 + (x * cos - y * sin) as i32,
        keski.1 + (x * sin + y * cos) as i32,
    )
}

/// Alkuruudun malli: kulmikas kivi, kaksi kuusikulmaista kehää ja kärjet.
pub fn kivimalli() -> Vec<Tahko> {
    const KULMIA: usize = 6;
    const YLA_SADE: [f32; KULMIA] = [1.9, 2.3, 1.7, 2.2, 1.8, 2.4];
    const ALA_SADE: [f32; KULMIA] = [2.2, 1.7, 2.3, 1.8, 2.4, 1.9];
    let kehapiste = |i: usize, sade: f32, y: f32| {
        let (sin, cos) = (i as f32 / KULMIA as f32 * std::f32::consts::TAU).sin_cos();
        [cos * sade, y, sin * sade]
    };
    let (yla, ala) = ([0.2, 3.4, 0.3], [-0.3, -1.0, -0.2]);
    let mut tahkot = Vec::new();
    for i in 0..KULMIA {
        let j = (i + 1) % KULMIA;
        let (a, b) = (
            kehapiste(i, YLA_SADE[i], 2.1),
            kehapiste(j, YLA_SADE[j], 2.1),
        );
        let (c, d) = (
            kehapiste(j, ALA_SADE[j], 0.2),
            kehapiste(i, ALA_SADE[i], 0.2),
        );
        tahkot.push(tahko4(a, b, yla, yla, RUNKO));
        tahkot.push(tahko4(a, b, c, d, KANSI));
        tahkot.push(tahko4(d, c, ala, ala, RUNKO));
    }
    tahkot
}

pub fn piirra(peli: &Asteroidit, kuva: &mut [u32]) {
    let mut p = Piirtaja {
        k: Kangas::new(kuva),
        kello: peli.kello_ms,
    };
    if peli.vaihe == Vaihe::Alku {
        p.asteroidi_alkuruutu(peli);
    } else {
        p.asteroidi_nakyma(peli);
    }
    p.aanikytkin(peli.aanet_paalla);
}

impl Piirtaja<'_> {
    fn asteroidi_alkuruutu(&mut self, peli: &Asteroidit) {
        self.maisema(kaanna("asteroidit.banneri"), &kivimalli());
        let otsikko = listan_otsikko(peli.netti_parhaat.is_some());
        self.parhaat_alkuruudulla(otsikko, peli.nakyva_lista());
        self.alkuohje(&ALKU_OSAT, &ALKU_RIVIT);
        self.aloituskehote(ALOITUS);
    }

    fn asteroidi_nakyma(&mut self, peli: &Asteroidit) {
        AVARUUS_KUVA.piirra(&mut self.k, KENTTA_X, KENTTA_Y);
        self.kappaleet(peli);
        self.kentan_kehys();
        self.otsikkopalkki(&kaanna("peli.asteroidit").to_uppercase(), "");
        self.asteroidi_tilasto(peli);
        self.asteroidi_paneeli(peli);
    }

    fn kappaleet(&mut self, peli: &Asteroidit) {
        let lauta = &peli.lauta;
        for kivi in &lauta.kivet {
            self.kivi(kivi);
        }
        if let Some(lautanen) = &lauta.lautanen {
            self.lautanen(lautanen);
        }
        for ammus in &lauta.ammukset {
            let (hehku, ydin) = if ammus.oma {
                (HUTI, VALKOINEN)
            } else {
                (OSUMA, KELTAINEN)
            };
            let keski = (
                KENTTA_X + ammus.paikka.0 as i32,
                KENTTA_Y + ammus.paikka.1 as i32,
            );
            self.k.kiekko(keski, 5, hehku);
            self.k.kiekko(keski, 2, ydin);
        }
        if peli.alus_nakyy() {
            let suojassa = lauta.suoja_ms > 0;
            if !suojassa || (self.kello / SUOJAN_VILKKU_MS).is_multiple_of(2) {
                self.alus(&lauta.alus, lauta.tyontaa);
            }
        }
        for rajahdys in &peli.rajahdykset {
            let ika = self.kello.saturating_sub(rajahdys.alku) as f32 / RAJAHDYS_MS as f32;
            let keski = (
                KENTTA_X + rajahdys.paikka.0 as i32,
                KENTTA_Y + rajahdys.paikka.1 as i32,
            );
            self.rajahdys(keski, ika, 10, rajahdys.koko);
        }
    }

    fn kivi(&mut self, kivi: &Kivi) {
        let muoto = &KIVEN_MUODOT[kivi.muoto % KIVEN_MUODOT.len()];
        let sade = kivi.koko.sade();
        for keski in kopiot(kivi.paikka, sade) {
            let kulma = |i: usize| i as f32 / muoto.len() as f32 * std::f32::consts::TAU;
            let piste = |kohta: usize, osuus: f32, siirto: (i32, i32)| {
                let (sin, cos) = kulma(kohta).sin_cos();
                let (x, y) = ruudulle(keski, kivi.kulma, (cos * sade * osuus, sin * sade * osuus));
                (x + siirto.0, y + siirto.1)
            };
            // Kerrokset hahmottavat pallomaisen kiven: valo tulee aina samasta suunnasta,
            // vaikka kivi pyörii, joten valoisa puoli ei kierrä sen mukana.
            let s = (sade * 0.14) as i32;
            for (kerroin, siirto, vari) in [
                (1.0, (0, 0), tummenna(KIVEN_VARI, 0.75)),
                (0.86, (-s / 2, -s / 2), tummenna(KIVEN_VARI, 0.25)),
                (0.62, (-s, -s), KIVEN_VARI),
                (0.3, (-2 * s, -2 * s), vaalenna(KIVEN_VARI, 0.4)),
            ] {
                let mut kulmat = [(0, 0); 10];
                for (i, kohta) in kulmat.iter_mut().enumerate() {
                    *kohta = piste(i, muoto[i] * kerroin, siirto);
                }
                self.k.monikulmio(&kulmat, vari);
            }
            // Kraatterit pyörivät kiven mukana.
            for (kohta, osuus) in [(1, 0.45), (6, 0.5)] {
                let kraatteri = piste(kohta, osuus, (0, 0));
                let koko = (sade * 0.14) as i32;
                self.k.kiekko(kraatteri, koko, tummenna(KIVEN_VARI, 0.55));
            }
        }
    }

    fn lautanen(&mut self, lautanen: &Lautanen) {
        let sade = lautanen.sade();
        let keski = (
            KENTTA_X + lautanen.paikka.0 as i32,
            KENTTA_Y + lautanen.paikka.1 as i32,
        );
        let piste = |(x, y): Piste| (keski.0 + (x * sade) as i32, keski.1 + (y * sade) as i32);
        let kulmat = LAUTASEN_MUOTO.map(piste);
        self.k.monikulmio(&kulmat, tummenna(HUTI, 0.3));
        self.k
            .viiva(piste((-1.0, 0.0)), piste((1.0, 0.0)), vaalenna(HUTI, 0.5));
        // Kupu ja vilkkuvat valot alakyljessä.
        self.k
            .kiekko(piste((0.0, -0.32)), (sade * 0.28) as i32, 0xF0F8FF);
        for (i, x) in [-0.55, 0.0, 0.55].into_iter().enumerate() {
            let paalla = (self.kello / 200 + i as u32).is_multiple_of(2);
            let vari = if paalla { KELTAINEN } else { OSUMA };
            self.k.piste(piste((x, 0.2)).0, piste((x, 0.2)).1, vari);
        }
    }

    fn alus(&mut self, alus: &Alus, tyontaa: bool) {
        for keski in kopiot(alus.paikka, 20.0) {
            if tyontaa && (self.kello / LIEKIN_VILKKU_MS).is_multiple_of(2) {
                let liekki = LIEKKI.map(|piste| ruudulle(keski, alus.kulma, piste));
                self.k.monikulmio(&liekki, WB_ORANSSI);
                let ydin = LIEKKI.map(|(x, y)| ruudulle(keski, alus.kulma, (x + 3.0, y / 2.0)));
                self.k.monikulmio(&ydin, KELTAINEN);
            }
            self.aluksen_kuva(keski, alus.kulma, ALUKSEN_VARI);
        }
    }

    fn aluksen_kuva(&mut self, keski: (i32, i32), kulma: f32, vari: u32) {
        let piste = |p: Piste| ruudulle(keski, kulma, p);
        let ulko = ALUKSEN_MUOTO.map(|(x, y)| piste((x * 1.3, y * 1.3)));
        self.k.monikulmio(&ulko, tummenna(vari, 0.8));
        self.k.monikulmio(&ALUKSEN_MUOTO.map(piste), vari);
        // Punaiset siivenkärjet ja sininen ohjaamo.
        for siipi in [(-8.0, 6.0), (-8.0, -6.0)] {
            let (x, y) = piste(siipi);
            self.k.piste(x, y, OSUMA);
        }
        let (x, y) = piste((3.0, 0.0));
        self.k.piste(x, y, 0x2858D8);
    }

    /// Kappaleet saavat piirtyä kentän yli, joten sen ympäristö maalataan vasta niiden jälkeen.
    fn kentan_kehys(&mut self) {
        self.tausta_kentan_ympari(KENTTA_X, KENTTA_Y, KENTTA_L, KENTTA_K);
        self.upotus(KENTTA_X, KENTTA_Y, KENTTA_L, KENTTA_K);
    }

    fn asteroidi_tilasto(&mut self, peli: &Asteroidit) {
        let lauta = &peli.lauta;
        let aalto = (lauta.aalto + 1).to_string();
        for (i, (otsikko, arvo)) in [
            (kaanna("yhteinen.pisteet"), lauta.pisteet.to_string()),
            (kaanna("yhteinen.paras"), peli.paras.to_string()),
            (kaanna("asteroidit.aalto"), aalto),
        ]
        .into_iter()
        .enumerate()
        {
            let y = KENTTA_Y + i as i32 * TILASTON_ASKEL;
            self.suurteksti(SARAKE_X, y, otsikko, OTSIKKO, KELTAINEN, false);
            self.suurteksti(SARAKE_X, y + 30, &arvo, OTSIKKO, VALKOINEN, true);
        }

        let y = KENTTA_Y + 3 * TILASTON_ASKEL;
        self.suurteksti(
            SARAKE_X,
            y,
            kaanna("asteroidit.alukset"),
            OTSIKKO,
            KELTAINEN,
            false,
        );
        // Kymmenen alusta täyttää sarakkeen, eikä suurempi luku enää kerro paljoa.
        for i in 0..lauta.elamat.min(10) as i32 {
            let keski = (SARAKE_X + 10 + i * 22, y + 42);
            self.aluksen_kuva(keski, -std::f32::consts::FRAC_PI_2, VALKOINEN);
        }

        self.parhaat_lista(
            SARAKE_X,
            PARHAAT_Y,
            PARHAAT_L,
            PARHAAT_KOKO,
            listan_otsikko(peli.netti_parhaat.is_some()),
            peli.nakyva_lista(),
        );
    }

    fn asteroidi_paneeli(&mut self, peli: &Asteroidit) {
        self.paneelin_tausta();
        self.teksti(TEKSTI_X, PANEELI_RIVI_Y, &peli.viesti, VALKOINEN);

        let alarivi = PANEELI_RIVI_Y + (PANEELI_RIVEJA as i32 - 1) * RIVIN_KORKEUS;
        let otsikko_y = alarivi - RIVIN_KORKEUS - 12;
        match peli.vaihe {
            Vaihe::Pelissa => self.teksti(TEKSTI_X, alarivi, kaanna(OHJE), HUTI),
            Vaihe::Tauko => {
                self.suurteksti(TEKSTI_X, otsikko_y, kaanna(TAUKO), TULOS, KELTAINEN, true);
                self.teksti(TEKSTI_X, alarivi, kaanna(TAUON_OHJE), HUTI);
            }
            Vaihe::Tappio => {
                self.suurteksti(TEKSTI_X, otsikko_y, kaanna(TAPPIO), TULOS, KELTAINEN, true);
                for (_, teksti, x) in NAPIT {
                    self.nappi(x, NAPPI_Y, teksti, VALKOINEN, WB_SININEN);
                }
            }
            Vaihe::Nimi => self.nimikysely(peli.lauta.pisteet, &peli.nimi),
            Vaihe::Alku => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yhteinen::kieli::{kaikilla_kielilla, muotoile};
    use crate::yhteinen::syote::Syote;

    #[test]
    fn napit_loytyvat_eivatka_mene_paallekkain() {
        for (nappi, ..) in NAPIT {
            let (x, y) = napin_keskipiste(nappi);
            assert_eq!(nappi_pisteesta(x, y), Some(nappi));
        }
        assert_eq!(nappi_pisteesta(0, 0), None);
        kaikilla_kielilla(|| assert!(NAPIT[0].2 + napin_leveys(NAPIT[0].1) < NAPIT[1].2));
    }

    #[test]
    fn tekstit_ja_kentta_mahtuvat_paikoilleen() {
        kaikilla_kielilla(|| {
            for teksti in [
                kaanna(OHJE).to_string(),
                kaanna(TAUON_OHJE).to_string(),
                muotoile("asteroidit.alus_tuhoutui", &[&99]),
                muotoile("yhteinen.tulos", &[&999999]),
            ] {
                assert!(teksti.chars().count() <= RIVILLE_MERKKEJA, "{teksti}");
            }
            for avain in ["asteroidit.aalto", "asteroidit.alukset"] {
                let leveys = kaanna(avain).chars().count() as i32 * OTSIKKO.2;
                assert!(SARAKE_X + leveys < LEVEYS as i32, "{avain}");
            }
        });
        alkuohje_mahtuu(&ALKU_OSAT, &ALKU_RIVIT, ALOITUS_Y);
        const { assert!((2 + 8 + 1 + 6) * PARHAAT_KOKO.2 <= PARHAAT_L) };
        const { assert!(PARHAAT_Y + 30 + 3 * 26 < PANEELI_Y) };
        const { assert!(KENTTA_Y + KENTTA_K < PANEELI_Y) };
        assert!(SARAKE_X + "PISTEET".len() as i32 * OTSIKKO.2 < LEVEYS as i32);
        assert!(SARAKE_X + 6 * OTSIKKO.2 < LEVEYS as i32);
    }

    #[test]
    fn laidan_yli_ulottuva_kappale_piirtyy_molemmille_puolille() {
        assert_eq!(kopiot((300.0, 300.0), 10.0).count(), 1);
        assert_eq!(kopiot((5.0, 300.0), 10.0).count(), 2);
        assert_eq!(kopiot((5.0, 5.0), 10.0).count(), 4);
    }

    #[test]
    fn koko_peli_piirtyy_kaatumatta() {
        let mut kuva = vec![0; LEVEYS * KORKEUS];
        let mut peli = Asteroidit::new(3);
        piirra(&peli, &mut kuva);
        peli.kasittele(Syote::Ammu);
        // Aluksen paikallaan pysyessä kivet osuvat lopulta, ja peli päättyy.
        for i in 0..6000 {
            peli.edista(16);
            piirra(&peli, &mut kuva);
            if i % 10 == 0 {
                peli.kasittele(Syote::Ammu);
            }
        }
        peli.vaihe = Vaihe::Tauko;
        piirra(&peli, &mut kuva);
        peli.vaihe = Vaihe::Tappio;
        piirra(&peli, &mut kuva);
        peli.vaihe = Vaihe::Nimi;
        piirra(&peli, &mut kuva);
    }
}
