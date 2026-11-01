//! Palikka8000:n näkymät.

use klassikot8000::palikka::{Kierros, Paikka, Palikka, Ruudut, SIVU, tason_tavoite};

use super::peli::{Animaatio, HUTI_MS, Kohta, Nappi, Palikkapeli, Vaihe, kohta};
use crate::yhteinen::kieli::kaanna;
use crate::yhteinen::piirto::*;
use crate::yhteinen::vektori::{Kangas, Tahko, tahko4};

const RUUTU: i32 = 68;
const LAUDAN_SIVU: i32 = RUUTU * SIVU as i32;
const RUUDUT_X: i32 = 30;
const RUUDUT_Y: i32 = 48;
const SARAKE_X: i32 = RUUDUT_X + LAUDAN_SIVU + 30;
const TILASTON_ASKEL: i32 = 87;
const EDISTYS_Y: i32 = RUUDUT_Y + 3 * TILASTON_ASKEL;
const EDISTYS_PALOJA: u32 = 16;
const PARHAAT_Y: i32 = EDISTYS_Y + 54;
const PARHAAT_L: i32 = LEVEYS as i32 - 12 - SARAKE_X;
const PALIKAN_SADE: f32 = 29.0;
const RUUTU_TUMMA: u32 = VESI;
const RUUTU_VAALEA: u32 = 0x12205E;
const RAJAHDYKSEN_KOKO: f32 = 150.0;
const LEIJUNTA: f32 = 50.0;
const VARJO: i32 = 3;

const OHJE: &str = "palikka.ohje";
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
    "palikka.tervetuloa",
    "yhteinen.ohjeet",
    "palikka.ohje_1",
    "palikka.ohje_2",
];
const ALKU_RIVIT: [&str; 4] = [
    "palikka.nappain_hiiri",
    "palikka.nappain_nuolet",
    "yhteinen.aanet_kytkin",
    "yhteinen.takaisin",
];

// Jokaisella värillä on oma muotonsa, jotta palikat erottuvat ilman värinäköäkin.
const KAHDEKSAN: [(f32, f32); 8] = [
    (-0.5, -0.9),
    (0.5, -0.9),
    (0.9, -0.5),
    (0.9, 0.5),
    (0.5, 0.9),
    (-0.5, 0.9),
    (-0.9, 0.5),
    (-0.9, -0.5),
];
const VIISI: [(f32, f32); 5] = [
    (0.0, -1.0),
    (0.95, -0.31),
    (0.59, 0.81),
    (-0.59, 0.81),
    (-0.95, -0.31),
];
const VINONELIO: [(f32, f32); 4] = [(0.0, -1.0), (0.9, 0.0), (0.0, 1.0), (-0.9, 0.0)];
const KUUSI: [(f32, f32); 6] = [
    (-0.5, -0.87),
    (0.5, -0.87),
    (1.0, 0.0),
    (0.5, 0.87),
    (-0.5, 0.87),
    (-1.0, 0.0),
];
const KOLMIO: [(f32, f32); 3] = [(0.0, -1.0), (1.0, 0.8), (-1.0, 0.8)];
const KILPI: [(f32, f32); 5] = [
    (-0.85, -0.8),
    (0.85, -0.8),
    (0.85, 0.2),
    (0.0, 1.0),
    (-0.85, 0.2),
];
const TAHTI: [(f32, f32); 8] = [
    (0.0, -1.0),
    (0.3, -0.3),
    (1.0, 0.0),
    (0.3, 0.3),
    (0.0, 1.0),
    (-0.3, 0.3),
    (-1.0, 0.0),
    (-0.3, -0.3),
];
const PYOREA: [(f32, f32); 12] = [
    (0.0, -0.95),
    (0.48, -0.82),
    (0.82, -0.48),
    (0.95, 0.0),
    (0.82, 0.48),
    (0.48, 0.82),
    (0.0, 0.95),
    (-0.48, 0.82),
    (-0.82, 0.48),
    (-0.95, 0.0),
    (-0.82, -0.48),
    (-0.48, -0.82),
];
const MUODOT: [(&[(f32, f32)], u32); 8] = [
    (&KAHDEKSAN, 0xE03C31),
    (&VIISI, 0xFF8C1A),
    (&VINONELIO, 0xF2D03B),
    (&KUUSI, 0x35C25A),
    (&KOLMIO, 0x4FD8E0),
    (&KILPI, 0x3A6CF0),
    (&TAHTI, 0xC060F0),
    (&PYOREA, 0xE8EEF4),
];

pub fn ruutu_pisteesta(x: i32, y: i32) -> Option<Paikka> {
    let sisalla = (RUUDUT_X..RUUDUT_X + LAUDAN_SIVU).contains(&x)
        && (RUUDUT_Y..RUUDUT_Y + LAUDAN_SIVU).contains(&y);
    sisalla.then(|| {
        (
            ((y - RUUDUT_Y) / RUUTU) as usize,
            ((x - RUUDUT_X) / RUUTU) as usize,
        )
    })
}

#[cfg(test)]
pub fn ruudun_keskipiste(ruutu: Paikka) -> (i32, i32) {
    keskipiste(ruutu)
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

fn keskipiste((rivi, sarake): Paikka) -> (i32, i32) {
    (
        RUUDUT_X + sarake as i32 * RUUTU + RUUTU / 2,
        RUUDUT_Y + rivi as i32 * RUUTU + RUUTU / 2,
    )
}

fn valilla(a: (i32, i32), b: (i32, i32), osuus: f32) -> (i32, i32) {
    (
        a.0 + ((b.0 - a.0) as f32 * osuus) as i32,
        a.1 + ((b.1 - a.1) as f32 * osuus) as i32,
    )
}

/// Alkuruudun malli: hiottu jalokivi, joka on kapeimmillaan alhaalla.
pub fn jalokivi() -> Vec<Tahko> {
    const KULMIA: usize = 8;
    let kehapiste = |i: usize, sade: f32, y: f32| {
        let (sin, cos) = (i as f32 / KULMIA as f32 * std::f32::consts::TAU).sin_cos();
        [sade * cos, y, sade * sin]
    };
    let (kirkas, tumma) = (0xF0485C, 0xB02440);
    let kansi = [0.0, 3.4, 0.0];
    let karki = [0.0, -1.4, 0.0];
    let mut tahkot = Vec::new();
    for i in 0..KULMIA {
        let seuraava = (i + 1) % KULMIA;
        let vari = if i % 2 == 0 { kirkas } else { tumma };
        let (kansi_a, kansi_b) = (kehapiste(i, 1.6, 3.4), kehapiste(seuraava, 1.6, 3.4));
        let (vyo_a, vyo_b) = (kehapiste(i, 2.9, 2.2), kehapiste(seuraava, 2.9, 2.2));
        tahkot.push(tahko4(kansi, kansi_a, kansi_b, kansi_b, 0xFF8A9A));
        tahkot.push(tahko4(kansi_a, kansi_b, vyo_b, vyo_a, vari));
        tahkot.push(tahko4(vyo_a, vyo_b, karki, karki, vari));
    }
    tahkot
}

pub fn piirra(peli: &Palikkapeli, kuva: &mut [u32]) {
    let mut p = Piirtaja {
        k: Kangas::new(kuva),
        kello: peli.kello_ms,
    };
    if peli.vaihe == Vaihe::Alku {
        p.palikan_alkuruutu(peli);
    } else {
        p.palikan_nakyma(peli);
    }
    p.aanikytkin(peli.aanet_paalla);
}

impl Piirtaja<'_> {
    fn palikan_alkuruutu(&mut self, peli: &Palikkapeli) {
        self.maisema(kaanna("palikka.banneri"), &jalokivi());
        let otsikko = listan_otsikko(peli.netti_parhaat.is_some());
        self.parhaat_alkuruudulla(otsikko, peli.nakyva_lista());
        self.alkuohje(&ALKU_OSAT, &ALKU_RIVIT);
        self.aloituskehote(ALOITUS);
    }

    fn palikan_nakyma(&mut self, peli: &Palikkapeli) {
        let l = LEVEYS as i32;
        self.pelitausta();
        self.palikkalauta();
        self.palikat(peli);
        if peli.animaatio.is_none() {
            self.merkinnat(peli);
        }
        // Yläpuolelta tulevat palikat piiloutuvat otsikkopalkin taakse.
        self.tausta_alue(0, 0, l, RUUDUT_Y - 3);
        self.upotus(RUUDUT_X, RUUDUT_Y, LAUDAN_SIVU, LAUDAN_SIVU);
        self.otsikkopalkki(&kaanna("peli.palikka").to_uppercase(), "");
        self.palikan_tilasto(peli);
        self.palikan_paneeli(peli);
    }

    fn palikkalauta(&mut self) {
        for rivi in 0..SIVU as i32 {
            for sarake in 0..SIVU as i32 {
                let vari = if (rivi + sarake) % 2 == 0 {
                    RUUTU_TUMMA
                } else {
                    RUUTU_VAALEA
                };
                let (x, y) = (RUUDUT_X + sarake * RUUTU, RUUDUT_Y + rivi * RUUTU);
                self.k.laatikko(x, y, RUUTU, RUUTU, vari);
                self.k.laatikko(x, y, RUUTU, 3, sekoita(vari, MUSTA, 0.35));
                self.k.laatikko(x, y, 3, RUUTU, sekoita(vari, MUSTA, 0.35));
            }
        }
    }

    fn palikat(&mut self, peli: &Palikkapeli) {
        match &peli.animaatio {
            None => self.ruudut(peli.lauta.ruudut(), &[]),
            Some(Animaatio::Huti { a, b, alku }) => {
                let t = self.kello.saturating_sub(*alku) as f32 / HUTI_MS as f32;
                // Liike kulkee puoleen väliin ja takaisin.
                let osuus = (1.0 - (2.0 * t - 1.0).abs()).max(0.0) * 0.5;
                let ruudut = peli.lauta.ruudut();
                self.ruudut(ruudut, &[*a, *b]);
                self.liikkuva(ruudut, *a, keskipiste(*a), keskipiste(*b), osuus);
                self.liikkuva(ruudut, *b, keskipiste(*b), keskipiste(*a), osuus);
            }
            Some(Animaatio::Vaihto { vaihto, alku }) => {
                let kulunut = self.kello.saturating_sub(*alku);
                match kohta(kulunut, vaihto.kierrokset.len()) {
                    Kohta::Vaihdetaan(t) => {
                        // `ennen`-laudalla palikat ovat jo vaihtaneet paikkaa.
                        let ruudut = &vaihto.kierrokset[0].ennen;
                        let (a, b) = (vaihto.a, vaihto.b);
                        self.ruudut(ruudut, &[a, b]);
                        self.liikkuva(ruudut, b, keskipiste(a), keskipiste(b), t);
                        self.liikkuva(ruudut, a, keskipiste(b), keskipiste(a), t);
                    }
                    Kohta::Poksuu(i, t) => self.poksahdus(&vaihto.kierrokset[i], t),
                    Kohta::Putoaa(i, t) => self.putoaminen(&vaihto.kierrokset[i], t),
                }
            }
        }
    }

    fn ruudut(&mut self, ruudut: &Ruudut, ohitettavat: &[Paikka]) {
        for (r, rivi) in ruudut.iter().enumerate() {
            for (s, ruutu) in rivi.iter().enumerate() {
                if let Some(palikka) = ruutu
                    && !ohitettavat.contains(&(r, s))
                {
                    self.palikka(keskipiste((r, s)), PALIKAN_SADE, *palikka);
                }
            }
        }
    }

    /// Piirtää paikassa `p` olevan palikan matkalla pisteestä toiseen.
    fn liikkuva(&mut self, ruudut: &Ruudut, p: Paikka, a: (i32, i32), b: (i32, i32), t: f32) {
        if let Some(palikka) = ruudut[p.0][p.1] {
            self.palikka(valilla(a, b, t), PALIKAN_SADE, palikka);
        }
    }

    fn poksahdus(&mut self, kierros: &Kierros, t: f32) {
        self.ruudut(&kierros.ennen, &kierros.poistuneet);
        for p in &kierros.poistuneet {
            if let Some(palikka) = kierros.ennen[p.0][p.1] {
                // Palikka paisuu hetken ja kutistuu pois.
                let koko = PALIKAN_SADE * (1.0 + 0.25 * (t * 3.0).min(1.0)) * (1.0 - t);
                self.palikka(keskipiste(*p), koko, palikka);
            }
        }
        for p in &kierros.rajahdykset {
            self.rajahdys(keskipiste(*p), t, 14, RAJAHDYKSEN_KOKO);
        }
        self.leijuva(kierros, t * 0.5);
    }

    fn putoaminen(&mut self, kierros: &Kierros, t: f32) {
        let mut paikallaan = kierros.valissa;
        for p in &kierros.pudotukset {
            if p.mista_rivi >= 0 {
                paikallaan[p.mista_rivi as usize][p.mihin.1] = None;
            }
        }
        self.ruudut(&paikallaan, &[]);
        // Kiihtyvä putoaminen tuntuu painavammalta kuin tasainen.
        let matka = t * t;
        for p in &kierros.pudotukset {
            let loppu = keskipiste(p.mihin);
            let alku_y = RUUDUT_Y + p.mista_rivi * RUUTU + RUUTU / 2;
            let y = alku_y + ((loppu.1 - alku_y) as f32 * matka) as i32;
            self.palikka((loppu.0, y), PALIKAN_SADE, p.palikka);
        }
        self.leijuva(kierros, 0.5 + t * 0.5);
    }

    /// Kierroksen pisteet nousevat poistuneiden ruutujen keskeltä.
    fn leijuva(&mut self, kierros: &Kierros, t: f32) {
        let maara = kierros.poistuneet.len().max(1) as i32;
        let (sx, sy) = kierros.poistuneet.iter().fold((0, 0), |(x, y), &p| {
            let (px, py) = keskipiste(p);
            (x + px, y + py)
        });
        let teksti = format!("+{}", kierros.pisteet);
        let leveys = teksti.chars().count() as i32 * OTSIKKO.2;
        let x = sx / maara - leveys / 2;
        let y = sy / maara - (t * LEIJUNTA) as i32;
        self.suurteksti(x, y, &teksti, OTSIKKO, KELTAINEN, true);
    }

    fn palikka(&mut self, keski: (i32, i32), sade: f32, palikka: Palikka) {
        if sade < 1.0 {
            return;
        }
        match palikka {
            Palikka::Tavallinen(vari) => self.jalokivi(keski, sade, vari),
            Palikka::Pommi(vari) => {
                self.jalokivi(keski, sade * 0.85, vari);
                let sade_ulko = sade * if self.vilkku() { 1.08 } else { 0.96 };
                self.k.ympyra(keski, sade_ulko, WB_ORANSSI);
                self.k.ympyra(keski, sade_ulko - 2.0, KELTAINEN);
            }
            Palikka::Kaikki => self.kaikkipalikka(keski, sade),
        }
    }

    fn jalokivi(&mut self, keski: (i32, i32), sade: f32, vari: u8) {
        let (muoto, vari) = MUODOT[vari as usize % MUODOT.len()];
        let mut kulmat = [(0, 0); 12];
        let kulmat = &mut kulmat[..muoto.len()];
        let piste = |(x, y): (f32, f32), kerroin: f32, siirto: (f32, f32)| {
            (
                keski.0 + ((x * kerroin + siirto.0) * sade) as i32,
                keski.1 + ((y * kerroin + siirto.1) * sade) as i32,
            )
        };
        for (kulma, &m) in kulmat.iter_mut().zip(muoto) {
            *kulma = piste(m, 1.0, (0.0, 0.0));
        }
        let mut varjo = [(0, 0); 12];
        for (kohde, &(x, y)) in varjo.iter_mut().zip(kulmat.iter()) {
            *kohde = (x + VARJO, y + VARJO);
        }
        self.k.monikulmio(&varjo[..muoto.len()], MUSTA);
        self.k.monikulmio(kulmat, vari);
        // Vino sisähionta näyttää valon tulevan vasemmalta ylhäältä.
        let mut hionta = [(0, 0); 12];
        let hionta = &mut hionta[..muoto.len()];
        for (kulma, &m) in hionta.iter_mut().zip(muoto) {
            *kulma = piste(m, 0.52, (-0.1, -0.12));
        }
        self.k.monikulmio(hionta, sekoita(vari, VALKOINEN, 0.42));
        // Tumma hionta alhaalla oikealla ja kirkas tähtikimallus.
        let mut pohja = [(0, 0); 12];
        let pohja = &mut pohja[..muoto.len()];
        for (kulma, &m) in pohja.iter_mut().zip(muoto) {
            *kulma = piste(m, 0.42, (0.28, 0.32));
        }
        self.k.monikulmio(pohja, sekoita(vari, MUSTA, 0.28));
        for (i, &a) in kulmat.iter().enumerate() {
            let b = kulmat[(i + 1) % kulmat.len()];
            self.k.viiva(a, b, sekoita(vari, MUSTA, 0.5));
        }
        let (kx, ky) = piste((-0.3, -0.34), 1.0, (0.0, 0.0));
        self.k.piste(kx, ky, VALKOINEN);
        self.k.piste(kx + 3, ky, sekoita(vari, VALKOINEN, 0.8));
        self.k.piste(kx, ky + 3, sekoita(vari, VALKOINEN, 0.8));
    }

    /// Kaikkipalikka on musta pallo, jonka kehällä näkyvät kaikki värit.
    fn kaikkipalikka(&mut self, keski: (i32, i32), sade: f32) {
        let (x, y) = keski;
        let s = sade as i32;
        let kahdeksan = KAHDEKSAN.map(|(px, py)| (x + (px * sade) as i32, y + (py * sade) as i32));
        self.k.monikulmio(&kahdeksan, MUSTA);
        // Kehä kiertää vilkun tahdissa, eli harvoin: tämä ei pakota piirtämään joka ruudussa.
        let kierto = (self.kello / VILKKU_MS) as f32 * 0.4;
        for (i, (_, vari)) in MUODOT.iter().enumerate() {
            let kulma = kierto + i as f32 / MUODOT.len() as f32 * std::f32::consts::TAU;
            let (sin, cos) = kulma.sin_cos();
            let px = x + (cos * sade * 0.68) as i32;
            let py = y + (sin * sade * 0.68) as i32;
            let koko = (s / 4).max(1);
            self.k
                .laatikko(px - koko / 2, py - koko / 2, koko, koko, *vari);
        }
        let ydin = (s / 3).max(1);
        self.k
            .laatikko(x - ydin / 2, y - ydin / 2, ydin, ydin, VALKOINEN);
        self.k.ympyra(keski, sade, VALKOINEN);
    }

    /// Valittu ruutu, kursori, hiiren alla oleva ruutu ja pitkän odotuksen vihje.
    fn merkinnat(&mut self, peli: &Palikkapeli) {
        if peli.vaihe != Vaihe::Pelissa {
            return;
        }
        if let Some((a, b)) = peli.vihje()
            && self.vilkku()
        {
            self.ruudun_kehys(a, WB_ORANSSI, 2);
            self.ruudun_kehys(b, WB_ORANSSI, 2);
        }
        if let Some(ruutu) = peli.hiiren_ruutu
            && !peli.kursori_nakyy
        {
            self.ruudun_kehys(ruutu, REUNA, 1);
        }
        if peli.kursori_nakyy {
            self.ruudun_kehys(peli.kursori, VALKOINEN, 2);
        }
        if let Some(ruutu) = peli.valittu {
            let vari = if self.vilkku() { KELTAINEN } else { WB_ORANSSI };
            self.ruudun_kehys(ruutu, vari, 3);
        }
    }

    fn ruudun_kehys(&mut self, (rivi, sarake): Paikka, vari: u32, paksuus: i32) {
        let (x, y) = (
            RUUDUT_X + sarake as i32 * RUUTU,
            RUUDUT_Y + rivi as i32 * RUUTU,
        );
        for i in 0..paksuus {
            self.k
                .kehys(x + i, y + i, RUUTU - 2 * i, RUUTU - 2 * i, vari);
        }
    }

    fn palikan_tilasto(&mut self, peli: &Palikkapeli) {
        let lauta = &peli.lauta;
        for (i, (otsikko, arvo)) in [
            (kaanna("yhteinen.pisteet"), lauta.pisteet),
            (kaanna("yhteinen.paras"), peli.paras),
            (kaanna("palikka.taso"), lauta.taso),
        ]
        .into_iter()
        .enumerate()
        {
            let y = RUUDUT_Y + i as i32 * TILASTON_ASKEL;
            self.suurteksti(SARAKE_X, y, otsikko, OTSIKKO, KELTAINEN, false);
            let numero = arvo.to_string();
            self.suurteksti(SARAKE_X, y + 30, &numero, OTSIKKO, VALKOINEN, true);
        }
        // Palkki kertoo matkan seuraavalle tasolle.
        let tavoite = tason_tavoite(lauta.taso).max(1);
        let taysia = lauta.tason_edistys.min(tavoite) * EDISTYS_PALOJA / tavoite;
        for i in 0..EDISTYS_PALOJA {
            let x = SARAKE_X + i as i32 * PALKIN_PALA;
            let vari = if i < taysia { KELTAINEN } else { KANSI };
            self.pala(x, EDISTYS_Y, PALKIN_PALA - 3, 15, vari);
        }
        self.parhaat_lista(
            SARAKE_X,
            PARHAAT_Y,
            PARHAAT_L,
            TULOS,
            listan_otsikko(peli.netti_parhaat.is_some()),
            peli.nakyva_lista(),
        );
    }

    fn palikan_paneeli(&mut self, peli: &Palikkapeli) {
        self.paneelin_tausta();
        self.teksti(TEKSTI_X, PANEELI_RIVI_Y, &peli.viesti, VALKOINEN);

        let alarivi = PANEELI_RIVI_Y + (PANEELI_RIVEJA as i32 - 1) * RIVIN_KORKEUS;
        let otsikko_y = alarivi - RIVIN_KORKEUS - 12;
        match peli.vaihe {
            Vaihe::Pelissa => self.teksti(TEKSTI_X, alarivi, kaanna(OHJE), HUTI),
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
    use klassikot8000::palikka::{SUURIN_TULOS, VAREJA};

    #[test]
    fn lauta_ja_sarake_mahtuvat_kuvaan() {
        const { assert!(RUUDUT_Y + LAUDAN_SIVU < PANEELI_Y) };
        const { assert!(SARAKE_X + EDISTYS_PALOJA as i32 * PALKIN_PALA < LEVEYS as i32) };
        const { assert!(PARHAAT_Y + 30 + 3 * 26 < PANEELI_Y) };
        assert_eq!(MUODOT.len(), VAREJA);
        // sijoitus, kahdeksan merkin nimi ja kuusinumeroiset pisteet samalle riville
        assert!((2 + 8 + 1 + 6) * TULOS.2 <= PARHAAT_L);
    }

    #[test]
    fn ruudut_loytyvat_ja_ulkopuoli_ei() {
        for rivi in 0..SIVU {
            for sarake in 0..SIVU {
                let (x, y) = ruudun_keskipiste((rivi, sarake));
                assert_eq!(ruutu_pisteesta(x, y), Some((rivi, sarake)));
            }
        }
        assert_eq!(ruutu_pisteesta(RUUDUT_X - 1, RUUDUT_Y), None);
        assert_eq!(ruutu_pisteesta(RUUDUT_X, RUUDUT_Y + LAUDAN_SIVU), None);
    }

    #[test]
    fn tekstit_mahtuvat_paikoilleen() {
        kaikilla_kielilla(|| {
            for pari in NAPIT.windows(2) {
                assert!(pari[0].2 + napin_leveys(pari[0].1) < pari[1].2);
            }
            let viestit = [
                kaanna(OHJE).to_string(),
                kaanna("palikka.huono").to_string(),
                muotoile("palikka.siirto", &[&SUURIN_TULOS]),
                muotoile("palikka.ketju", &[&99, &SUURIN_TULOS]),
                muotoile("palikka.uusi_taso", &[&999]),
                muotoile("palikka.ei_siirtoja", &[&SUURIN_TULOS]),
            ];
            for viesti in viestit {
                assert!(viesti.chars().count() <= RIVILLE_MERKKEJA, "{viesti}");
            }
        });
        alkuohje_mahtuu(&ALKU_OSAT, &ALKU_RIVIT, ALOITUS_Y);
    }

    #[test]
    fn koko_peli_piirtyy_kaatumatta_toiston_aikana() {
        let mut kuva = vec![0; LEVEYS * KORKEUS];
        let mut peli = Palikkapeli::new(3);
        piirra(&peli, &mut kuva);
        peli.kasittele(Syote::Ammu);
        for _ in 0..60 {
            let Some((a, b)) = peli.lauta.vihje() else {
                break;
            };
            peli.vaihda(b, a);
            while peli.animoi() {
                peli.edista(40);
                piirra(&peli, &mut kuva);
            }
        }
        peli.kursori_nakyy = true;
        peli.valittu = Some((0, 0));
        peli.edista(10_000);
        piirra(&peli, &mut kuva);
        // Erikoispalikat ja hutivaihto
        peli.lauta = klassikot8000::palikka::Palikkalauta::ruuduista({
            let mut ruudut = *peli.lauta.ruudut();
            ruudut[0][0] = Some(Palikka::Pommi(1));
            ruudut[0][1] = Some(Palikka::Kaikki);
            ruudut
        });
        piirra(&peli, &mut kuva);
        peli.vaihe = Vaihe::Tappio;
        piirra(&peli, &mut kuva);
        peli.vaihe = Vaihe::Nimi;
        piirra(&peli, &mut kuva);
    }
}
