//! Silmukan näkymät.

use std::f32::consts::{FRAC_PI_2, TAU};
use std::sync::OnceLock;

use klassikot8000::silmukka::{
    self, Ennuste, Herkku, KULLAN_KESTO_MS, KUORIUTUMINEN_MS, Laji, Niitty, Otus, RIVEJA, Ruutu,
    SARAKKEITA,
};

use super::peli::{
    KEHU_MS, LEIJUNTA_MS, Nappi, PAMAUS_MS, Silmukka, TARINA_MS, VALAHDYS_MS, Vaihe, Valahdys,
};
use crate::yhteinen::kieli::kaanna;
use crate::yhteinen::piirto::*;
use crate::yhteinen::rasteri::{Rasteri, ditteroi, kohina};
use crate::yhteinen::vektori::{Kangas, Tahko, kehapiste, kuutio, tahko4, vaalenna};

const RUUTU: i32 = 24;
const KENTTA_X: i32 = 30;
const KENTTA_Y: i32 = 48;
const KENTTA_L: i32 = SARAKKEITA * RUUTU;
const KENTTA_K: i32 = RIVEJA * RUUTU;
const SARAKE_X: i32 = KENTTA_X + KENTTA_L + 30;
const TILASTON_ASKEL: i32 = 87;
const PARHAAT_Y: i32 = KENTTA_Y + 4 * TILASTON_ASKEL + 12;
const PARHAAT_L: i32 = LEVEYS as i32 - 12 - SARAKE_X;
// Kapeaan sarakkeeseen mahtuu sijoitus, kahdeksan merkin nimi ja kuusinumeroiset pisteet.
const PARHAAT_KOKO: (i32, i32, i32) = (10, 15, 13);

// Kesäyön värit: jokainen taso vie niittyä kohti seuraavaa aamua.
const NIITYT: [(u32, u32); 6] = [
    (0x10243A, 0x0A2A24),
    (0x1E1F48, 0x0E2C2C),
    (0x331C4E, 0x142C30),
    (0x48203F, 0x1C2C2C),
    (0x4C2A2C, 0x202E22),
    (0x2A3A48, 0x18362A),
];
const RUUDUN_VAALENNUS: f32 = 0.07;
// Tärisevä kenttä paljastaa reunansa, joten kuvassa on varaa joka puolella.
const NIITYN_VARA: i32 = 3;

/// Tason niitty valmiiksi laskettuna: ruudutus, ditteröity liuku ja heinätuppaat.
fn niittykuva((yla, ala): (u32, u32)) -> Rasteri {
    let sivu = RUUTU / 3;
    let leveys = KENTTA_L / 3 + 2 * NIITYN_VARA;
    let korkeus = KENTTA_K / 3 + 2 * NIITYN_VARA;
    Rasteri::luo(leveys as usize, korkeus as usize, move |x, y| {
        let (kx, ky) = (x - NIITYN_VARA, y - NIITYN_VARA);
        let osuus = (ky.max(0) as f32 / (KENTTA_K / 3) as f32).min(1.0);
        let pohja = ditteroi(x, y, yla, ala, osuus);
        let ruutu = (kx.div_euclid(sivu) + ky.div_euclid(sivu)).rem_euclid(2) == 0;
        let vari = if ruutu {
            sekoita(pohja, VALKOINEN, RUUDUN_VAALENNUS)
        } else {
            pohja
        };
        match kohina(x / 2, y) {
            n if n > 0.985 => sekoita(vari, MATO_A, 0.35),
            n if n > 0.96 => sekoita(vari, MATO_A, 0.15),
            _ => vari,
        }
    })
}

static NIITTY_KUVAT: [OnceLock<Rasteri>; NIITYT.len()] = [const { OnceLock::new() }; NIITYT.len()];

const MATO_TUMMA: u32 = 0x0B3A1E;
const MATO_A: u32 = 0x35C25A;
const MATO_B: u32 = 0xB6E84A;
const MATO_KIILTO: u32 = 0xE8FFC0;
const MATON_SADE: i32 = 9;
const PAAN_SADE: i32 = 11;
const AALLON_TIHEYS: f32 = 0.55;
const AALLON_NOPEUS: f32 = 0.006;
const HANNAN_KAPENEMINEN: usize = 5;
const KIELEN_JAKSO_MS: u32 = 1100;
const KIELEN_KESTO_MS: u32 = 180;

pub const OMENAN_VARI: u32 = 0xE03C31;
pub const KULLAN_VARI: u32 = 0xFFD84A;
pub const MUNAN_VARI: u32 = 0xF4EFD8;
const LEHTI: u32 = 0x48C050;
const VARSI: u32 = 0x8A5A2B;
const ITIKAN_RUUMIS: u32 = 0xFF8A3C;
const ITIKAN_SIIPI: u32 = 0xDCEEFF;
const PAARMAN_RUUMIS: u32 = 0x2A1A12;
const PAARMAN_RAITA: u32 = 0xF2B632;
const PAARMAN_SILMA: u32 = 0xFF2A2A;
const SIIVEN_VILKKU_MS: u32 = 45;
const LEVON_VAROITUS_MS: u32 = 350;
const KULLAN_VAROITUS_MS: u32 = 2000;

const ENNUSTE_SAALIS: u32 = 0x7A6414;
const ENNUSTE_TYHJA: u32 = 0x24303E;
const ENNUSTEEN_SYKE: f32 = 0.008;
// Silmukan purkautuminen kestää saman verran, oli se lyhyt tai pitkä.
const PURKU_MS: u32 = 450;
const LEIMAHDUKSEN_VIIVE_MS: i32 = 12;

const OHJE: &str = "silmukka.ohje";
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
    "silmukka.tervetuloa",
    "yhteinen.ohjeet",
    "silmukka.ohje_1",
    "silmukka.ohje_2",
];
const ALKU_RIVIT: [&str; 3] = [
    "silmukka.nappain_ohjaus",
    "yhteinen.tauko_aanet",
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

/// Hiiren paikka kentän ruuduksi. Kentän ulkopuolella lähin reunaruutu.
pub fn ruutu_pisteesta(x: i32, y: i32) -> Ruutu {
    (
        ((x - KENTTA_X).div_euclid(RUUTU)).clamp(0, SARAKKEITA - 1),
        ((y - KENTTA_Y).div_euclid(RUUTU)).clamp(0, RIVEJA - 1),
    )
}

#[cfg(test)]
pub fn ruudun_keskipiste(ruutu: Ruutu) -> (i32, i32) {
    keski((KENTTA_X, KENTTA_Y), liukuluvuksi(ruutu))
}

fn liukuluvuksi(r: Ruutu) -> (f32, f32) {
    (r.0 as f32, r.1 as f32)
}

fn valilla(a: (f32, f32), b: (f32, f32), osuus: f32) -> (f32, f32) {
    (a.0 + (b.0 - a.0) * osuus, a.1 + (b.1 - a.1) * osuus)
}

/// Ruudun keskipiste kuvassa. `o` on kentän yläkulma, joka tärisee saaliin osuessa.
fn keski(o: (i32, i32), (x, y): (f32, f32)) -> (i32, i32) {
    (
        o.0 + (x * RUUTU as f32) as i32 + RUUTU / 2,
        o.1 + (y * RUUTU as f32) as i32 + RUUTU / 2,
    )
}

/// Madon nivelet ruuduissa, pää ensin. Nivel liukuu takanaan olevasta ruudusta omaansa.
fn nivelet(lauta: &Niitty) -> Vec<(f32, f32)> {
    let osuus = lauta.vaihe();
    let mato = &lauta.mato;
    mato.iter()
        .enumerate()
        .map(|(i, &ruutu)| {
            let takana = mato.get(i + 1).copied().unwrap_or(lauta.jalki);
            valilla(liukuluvuksi(takana), liukuluvuksi(ruutu), osuus)
        })
        .collect()
}

/// Alkuruudun malli: häntäänsä pureva mato pystyasennossa, kapenee hännästä päähän.
pub fn ouroboros() -> Vec<Tahko> {
    const OSIA: usize = 22;
    const KULMIA: usize = 6;
    const KEHAN_SADE: f32 = 2.5;
    let kehapiste = |osa: usize, kulma: usize| {
        let osuus = osa as f32 / OSIA as f32;
        let paksuus = 0.2 + 0.75 * osuus;
        let (sin_a, cos_a) = (osuus * TAU).sin_cos();
        let (sin_b, cos_b) = (kulma as f32 / KULMIA as f32 * TAU).sin_cos();
        let sade = KEHAN_SADE + paksuus * cos_b;
        [sade * cos_a, 1.0 + sade * sin_a, paksuus * sin_b]
    };
    let mut tahkot = Vec::new();
    for osa in 0..OSIA {
        let vari = if osa % 2 == 0 { MATO_A } else { MATO_B };
        for kulma in 0..KULMIA {
            let seuraava = (kulma + 1) % KULMIA;
            tahkot.push(tahko4(
                kehapiste(osa, kulma),
                kehapiste(osa + 1, kulma),
                kehapiste(osa + 1, seuraava),
                kehapiste(osa, seuraava),
                vari,
            ));
        }
    }
    // pää ja silmät hännän kohdalla
    kuutio([1.7, 3.3], [0.55, 1.75], [-0.8, 0.8], MATO_B, &mut tahkot);
    kuutio(
        [2.75, 3.4],
        [1.45, 1.95],
        [0.25, 0.65],
        VALKOINEN,
        &mut tahkot,
    );
    kuutio(
        [2.75, 3.4],
        [1.45, 1.95],
        [-0.65, -0.25],
        VALKOINEN,
        &mut tahkot,
    );
    tahkot
}

pub fn piirra(peli: &Silmukka, kuva: &mut [u32]) {
    let mut p = Piirtaja {
        k: Kangas::new(kuva),
        kello: peli.kello_ms,
    };
    if peli.vaihe == Vaihe::Alku {
        p.silmukan_alkuruutu(peli);
    } else {
        p.silmukan_nakyma(peli);
    }
    p.aanikytkin(peli.aanet_paalla);
}

fn pallo(k: &mut Kangas, keski: (i32, i32), sade: i32, vari: u32) {
    k.kiekko(keski, sade, vari);
}

impl Piirtaja<'_> {
    fn silmukan_alkuruutu(&mut self, peli: &Silmukka) {
        self.maisema(kaanna("silmukka.banneri"), &ouroboros());
        let otsikko = listan_otsikko(peli.netti_parhaat.is_some());
        self.parhaat_alkuruudulla(otsikko, peli.nakyva_lista());
        self.alkuohje(&ALKU_OSAT, &ALKU_RIVIT);
        self.aloituskehote(ALOITUS);
    }

    fn silmukan_nakyma(&mut self, peli: &Silmukka) {
        let o = self.tarinan_siirto(peli);
        let lauta = &peli.lauta;
        self.niitty(o, lauta.taso);
        if peli.vaihe == Vaihe::Pelissa
            && let Some(ennuste) = lauta.ennuste()
        {
            self.ennuste(o, &ennuste);
        }
        for valahdys in &peli.valahdykset {
            self.sisuksen_leimahdus(o, valahdys, lauta.taso);
        }
        for herkku in &lauta.herkut {
            self.herkku(o, herkku);
        }
        for valahdys in &peli.valahdykset {
            self.reitin_purku(o, valahdys);
        }
        self.mato(o, lauta);
        for otus in &lauta.otukset {
            self.otus(o, otus);
        }
        for pamaus in &peli.pamaukset {
            let ika = self.kello.saturating_sub(pamaus.alku) as f32 / PAMAUS_MS as f32;
            self.rajahdys(keski(o, liukuluvuksi(pamaus.paikka)), ika, 10, pamaus.koko);
        }
        if peli.vaihe == Vaihe::Pelissa
            && let Some(ennuste) = lauta.ennuste()
        {
            self.ennusteen_pisteet(o, &ennuste);
        }
        self.leijuvat(o, peli);
        self.kehu(peli);
        self.silmukan_kehys();
        self.otsikkopalkki(&kaanna("peli.silmukka").to_uppercase(), "");
        self.silmukan_tilasto(peli);
        self.silmukan_paneeli(peli);
    }

    fn tarinan_siirto(&self, peli: &Silmukka) -> (i32, i32) {
        let Some((voima, alku)) = peli.tarina else {
            return (KENTTA_X, KENTTA_Y);
        };
        let ika = self.kello.saturating_sub(alku) as f32;
        let jaljella = (1.0 - ika / TARINA_MS as f32).max(0.0);
        let (x, y) = ((ika * 0.09).sin(), (ika * 0.13).cos());
        (
            KENTTA_X + (x * voima * jaljella) as i32,
            KENTTA_Y + (y * voima * jaljella) as i32,
        )
    }

    fn niitty(&mut self, o: (i32, i32), taso: u32) {
        let i = (taso as usize - 1) % NIITYT.len();
        let kuva = NIITTY_KUVAT[i].get_or_init(|| niittykuva(NIITYT[i]));
        let vara = NIITYN_VARA * 3;
        kuva.piirra(&mut self.k, o.0 - vara, o.1 - vara);
    }

    /// Tuleva silmukka: sisus sykkii ja kulkematon matka näkyy pisteinä, joten pelaaja
    /// näkee saaliin ennen kuin päättää.
    fn ennuste(&mut self, o: (i32, i32), ennuste: &Ennuste) {
        let vari = if ennuste.saalis > 0 {
            ENNUSTE_SAALIS
        } else {
            ENNUSTE_TYHJA
        };
        let syke = 0.75 + 0.25 * (self.kello as f32 * ENNUSTEEN_SYKE).sin();
        let vari = sekoita(MUSTA, vari, syke);
        for &ruutu in &ennuste.sisus {
            let x = o.0 + ruutu.0 * RUUTU;
            let y = o.1 + ruutu.1 * RUUTU;
            self.k.laatikko(x + 1, y + 1, RUUTU - 2, RUUTU - 2, vari);
        }
        let pisteen_vari = if ennuste.saalis > 0 { KELTAINEN } else { HUTI };
        for &ruutu in &ennuste.reitti[..ennuste.suora] {
            pallo(&mut self.k, keski(o, liukuluvuksi(ruutu)), 2, pisteen_vari);
        }
    }

    fn ennusteen_pisteet(&mut self, o: (i32, i32), ennuste: &Ennuste) {
        if ennuste.saalis == 0 || ennuste.sisus.is_empty() {
            return;
        }
        let n = ennuste.sisus.len() as i32;
        let (sx, sy) = ennuste
            .sisus
            .iter()
            .fold((0, 0), |s, r| (s.0 + r.0, s.1 + r.1));
        let teksti = format!("+{}", ennuste.pisteet);
        let leveys = teksti.chars().count() as i32 * MERKKI;
        let (x, y) = keski(o, (sx as f32 / n as f32, sy as f32 / n as f32));
        let (x, y) = (x - leveys / 2, y - MERKKI / 2);
        self.teksti(x + 2, y + 2, &teksti, MUSTA);
        self.teksti(x, y, &teksti, KELTAINEN);
    }

    fn sisuksen_leimahdus(&mut self, o: (i32, i32), valahdys: &Valahdys, taso: u32) {
        let Some(&risteys) = valahdys.reitti.last() else {
            return;
        };
        let ika = self.kello.saturating_sub(valahdys.alku) as i32;
        let (_, tausta) = NIITYT[(taso as usize - 1) % NIITYT.len()];
        for &ruutu in &valahdys.sisus {
            // Aalto leviää risteyksestä, jossa silmukka sulkeutui.
            let matka = (ruutu.0 - risteys.0).abs() + (ruutu.1 - risteys.1).abs();
            let kulunut = ika - matka * LEIMAHDUKSEN_VIIVE_MS;
            if kulunut < 0 {
                continue;
            }
            let osuus = kulunut as f32 / VALAHDYS_MS as f32 * 1.6;
            if osuus >= 1.0 {
                continue;
            }
            let vari = if osuus < 0.25 {
                sekoita(VALKOINEN, KULLAN_VARI, osuus * 4.0)
            } else {
                sekoita(KULLAN_VARI, tausta, (osuus - 0.25) / 0.75)
            };
            let x = o.0 + ruutu.0 * RUUTU;
            let y = o.1 + ruutu.1 * RUUTU;
            self.k.laatikko(x, y, RUUTU, RUUTU, vari);
        }
    }

    /// Vartalon kulunut osa poksahtelee pois päästä alkaen.
    fn reitin_purku(&mut self, o: (i32, i32), valahdys: &Valahdys) {
        let ika = self.kello.saturating_sub(valahdys.alku) as f32;
        let n = valahdys.reitti.len().max(1) as f32;
        let (vari, vaalea) = if valahdys.saalis {
            (KULLAN_VARI, VALKOINEN)
        } else {
            (MATO_TUMMA, MATO_A)
        };
        for (i, &ruutu) in valahdys.reitti.iter().enumerate() {
            let alkaa = i as f32 / n * PURKU_MS as f32;
            let osuus = (ika - alkaa) / (VALAHDYS_MS - PURKU_MS) as f32;
            let keskipiste = keski(o, liukuluvuksi(ruutu));
            if osuus < 0.0 {
                pallo(&mut self.k, keskipiste, MATON_SADE, vari);
            } else if osuus < 1.0 {
                let sade = (MATON_SADE as f32 * (1.0 - osuus)) as i32;
                pallo(&mut self.k, keskipiste, sade, vari);
                pallo(&mut self.k, keskipiste, sade / 2, vaalea);
            }
        }
    }

    fn mato(&mut self, o: (i32, i32), lauta: &Niitty) {
        let nivelet = nivelet(lauta);
        let n = nivelet.len();
        // Vartalo on kaksi kerrosta, jotta se erottuu itsestään ja taustasta.
        for kerros in 0..2 {
            for i in (0..n).rev() {
                let hannasta = n - 1 - i;
                let sade = MATON_SADE
                    - (HANNAN_KAPENEMINEN.saturating_sub(hannasta) as i32).min(MATON_SADE - 3);
                let (sade, vari) = if kerros == 0 {
                    (sade + 2, MATO_TUMMA)
                } else {
                    let aalto =
                        (i as f32 * AALLON_TIHEYS - self.kello as f32 * AALLON_NOPEUS).sin();
                    (sade, sekoita(MATO_A, MATO_B, 0.5 + 0.5 * aalto))
                };
                let nyt = keski(o, nivelet[i]);
                pallo(&mut self.k, nyt, sade, vari);
                if let Some(&seuraava) = nivelet.get(i + 1) {
                    // Tiheä väli tekee vartalosta sileän, eikä se näytä helminauhalta.
                    for osuus in [0.25, 0.5, 0.75] {
                        let vali = keski(o, valilla(nivelet[i], seuraava, osuus));
                        pallo(&mut self.k, vali, sade, vari);
                    }
                }
            }
        }
        for (i, &nivel) in nivelet.iter().enumerate().skip(1).step_by(2) {
            if n - 1 - i >= HANNAN_KAPENEMINEN {
                let (x, y) = keski(o, nivel);
                self.k.piste(x - 3, y - 5, MATO_KIILTO);
                self.k.piste(x + 3, y - 2, vaalenna(MATO_B, 0.3));
            }
        }
        self.paa(o, lauta, nivelet[0]);
    }

    fn paa(&mut self, o: (i32, i32), lauta: &Niitty, nivel: (f32, f32)) {
        let keskipiste = keski(o, nivel);
        let (dx, dy) = lauta.suunta.siirto();
        let (dx, dy) = (dx as f32, dy as f32);
        pallo(&mut self.k, keskipiste, PAAN_SADE + 3, MATO_TUMMA);
        self.k.kiiltopallo(keskipiste, PAAN_SADE, MATO_B);

        if !lauta.ohi && self.kello % KIELEN_JAKSO_MS < KIELEN_KESTO_MS {
            let kanta = siirra(keskipiste, dx * 11.0, dy * 11.0);
            let karki = siirra(kanta, dx * 7.0, dy * 7.0);
            self.k.viiva(kanta, karki, OMENAN_VARI);
            self.k.viiva(
                karki,
                siirra(karki, dx * 3.0 + dy * 3.0, dy * 3.0 + dx * 3.0),
                OMENAN_VARI,
            );
            self.k.viiva(
                karki,
                siirra(karki, dx * 3.0 - dy * 3.0, dy * 3.0 - dx * 3.0),
                OMENAN_VARI,
            );
        }

        let katse = lauta
            .otukset
            .iter()
            .filter(|otus| !otus.on_muna())
            .min_by_key(|otus| {
                (otus.paikka.0 - lauta.paa().0).abs() + (otus.paikka.1 - lauta.paa().1).abs()
            })
            .map_or((dx, dy), |otus| {
                let x = (otus.paikka.0 - lauta.paa().0) as f32;
                let y = (otus.paikka.1 - lauta.paa().1) as f32;
                let pituus = (x * x + y * y).sqrt().max(1.0);
                (x / pituus, y / pituus)
            });
        for puoli in [-1.0, 1.0] {
            let silma = siirra(
                keskipiste,
                dx * 3.0 - dy * puoli * 5.0,
                dy * 3.0 + dx * puoli * 5.0,
            );
            pallo(&mut self.k, silma, 4, VALKOINEN);
            if lauta.ohi {
                let (x, y) = silma;
                self.k.viiva((x - 3, y - 3), (x + 3, y + 3), MUSTA);
                self.k.viiva((x - 3, y + 3), (x + 3, y - 3), MUSTA);
            } else {
                let pupilli = siirra(silma, katse.0 * 2.0, katse.1 * 2.0);
                pallo(&mut self.k, pupilli, 2, MUSTA);
            }
        }
    }

    fn herkku(&mut self, o: (i32, i32), herkku: &Herkku) {
        let (x, y) = keski(o, liukuluvuksi(herkku.paikka));
        let keinunta =
            ((self.kello as f32 * 0.005 + (herkku.paikka.0 * 7) as f32).sin() * 2.0) as i32;
        let (x, y) = (x, y + keinunta);
        let Some(jaljella) = herkku.kulta_ms else {
            self.omena((x, y), OMENAN_VARI);
            return;
        };
        // Viimeisinä hetkinä kultaomena vilkkuu ja himmenee kohti katoamistaan.
        if jaljella < KULLAN_VAROITUS_MS && (self.kello / 100).is_multiple_of(2) {
            return;
        }
        debug_assert!(jaljella <= KULLAN_KESTO_MS);
        self.omena((x, y), KULLAN_VARI);
        let kulma = self.kello as f32 * 0.004;
        for i in 0..4 {
            let suunta = kulma + i as f32 * FRAC_PI_2;
            self.k.viiva(
                kehapiste((x, y), 11.0, suunta),
                kehapiste((x, y), 15.0, suunta),
                VALKOINEN,
            );
        }
    }

    fn omena(&mut self, (x, y): (i32, i32), vari: u32) {
        pallo(&mut self.k, (x + 2, y + 3), 8, varjo(MUSTA));
        self.k.kiiltopallo((x, y), 9, vari);
        self.k.viiva((x, y - 7), (x + 1, y - 11), VARSI);
        self.k
            .monikulmio(&[(x + 1, y - 9), (x + 7, y - 12), (x + 5, y - 7)], LEHTI);
    }

    fn otus(&mut self, o: (i32, i32), otus: &Otus) {
        let paikka = valilla(
            liukuluvuksi(otus.edellinen),
            liukuluvuksi(otus.paikka),
            otus.vaihe(),
        );
        let (x, y) = keski(o, paikka);
        if otus.on_muna() {
            self.muna((x, y), otus);
            return;
        }
        let vaihe = otus.paikka.0 * 3 + otus.paikka.1;
        let (x, y) = (
            x + ((self.kello as f32 * 0.021 + vaihe as f32).sin() * 2.0) as i32,
            y + ((self.kello as f32 * 0.017 + vaihe as f32).cos() * 2.0) as i32,
        );
        // Levossa siivet ovat suppuissa, joten pelaaja näkee, milloin kiertäminen kannattaa.
        let isku = (self.kello / SIIVEN_VILKKU_MS).is_multiple_of(2);
        let (x, y) = if otus.lepo_ms > 0 {
            keski(o, liukuluvuksi(otus.paikka))
        } else {
            (x, y)
        };
        match otus.laji {
            Laji::Itikka if otus.lepo_ms > 0 => self.levossa((x, y), otus),
            Laji::Itikka => self.itikka((x, y), otus.suunta, isku),
            Laji::Paarma => self.paarma((x, y), isku),
        }
    }

    fn muna(&mut self, (x, y): (i32, i32), otus: &Otus) {
        let osuus = 1.0 - otus.kuoriutuu_ms as f32 / KUORIUTUMINEN_MS as f32;
        let sade = 4 + (osuus * 5.0) as i32;
        // Varoitus nopeutuu kohti kuoriutumista.
        let jakso = (240.0 - osuus * 180.0) as u32;
        if (self.kello / jakso).is_multiple_of(2) {
            self.k.ympyra((x, y), sade as f32 + 4.0, ITIKAN_RUUMIS);
        }
        pallo(&mut self.k, (x, y), sade, MUNAN_VARI);
        self.k.laatikko(x - 1, y - 1, 2, 2, MUSTA);
    }

    fn levossa(&mut self, (x, y): (i32, i32), otus: &Otus) {
        for puoli in [-1, 1] {
            self.k.monikulmio(
                &[(x, y - 2), (x + puoli * 8, y + 9), (x + puoli, y + 4)],
                ITIKAN_SIIPI,
            );
        }
        pallo(&mut self.k, (x, y), 7, MUSTA);
        pallo(&mut self.k, (x, y), 6, ITIKAN_RUUMIS);
        pallo(&mut self.k, (x + otus.suunta.0 * 3, y - 4), 2, MUSTA);
        // Viimeinen hetki ennen lähtöä: siipi värähtää.
        if otus.lepo_ms < LEVON_VAROITUS_MS && (self.kello / 60).is_multiple_of(2) {
            self.k.ympyra((x, y), 11.0, ITIKAN_SIIPI);
        }
    }

    fn itikka(&mut self, (x, y): (i32, i32), suunta: Ruutu, isku: bool) {
        let nokka = (suunta.0 * 12, suunta.1 * 12);
        self.k.viiva((x, y), (x + nokka.0, y + nokka.1), MUSTA);
        for puoli in [-1, 1] {
            let korkeus = if isku { 11 } else { 4 };
            let siipi = (x + puoli * 11, y - korkeus);
            self.k
                .monikulmio(&[(x, y - 1), siipi, (x + puoli * 3, y + 2)], ITIKAN_SIIPI);
        }
        pallo(&mut self.k, (x, y), 7, MUSTA);
        pallo(&mut self.k, (x, y), 6, ITIKAN_RUUMIS);
        pallo(&mut self.k, (x + suunta.0 * 5, y + suunta.1 * 5), 2, MUSTA);
    }

    fn paarma(&mut self, (x, y): (i32, i32), isku: bool) {
        for puoli in [-1, 1] {
            let korkeus = if isku { 12 } else { 4 };
            let siipi = (x + puoli * 14, y - korkeus);
            self.k
                .monikulmio(&[(x, y - 2), siipi, (x + puoli * 5, y + 3)], ITIKAN_SIIPI);
        }
        pallo(&mut self.k, (x, y), 9, PAARMAN_SILMA);
        pallo(&mut self.k, (x, y), 8, PAARMAN_RUUMIS);
        self.k.laatikko(x - 7, y - 3, 15, 2, PAARMAN_RAITA);
        self.k.laatikko(x - 7, y + 2, 15, 2, PAARMAN_RAITA);
        pallo(&mut self.k, (x - 4, y - 5), 2, PAARMAN_SILMA);
        pallo(&mut self.k, (x + 4, y - 5), 2, PAARMAN_SILMA);
    }

    fn leijuvat(&mut self, o: (i32, i32), peli: &Silmukka) {
        for leijuva in &peli.leijuvat {
            let ika = self.kello.saturating_sub(leijuva.alku);
            let nousu = (ika * 48 / LEIJUNTA_MS) as i32;
            let leveys = leijuva.teksti.chars().count() as i32 * MERKKI;
            let (x, y) = keski(o, liukuluvuksi(leijuva.paikka));
            let x = (x - leveys / 2).clamp(KENTTA_X, KENTTA_X + KENTTA_L - leveys);
            let y = (y - MERKKI / 2 - nousu).max(KENTTA_Y);
            self.teksti(x + 2, y + 2, &leijuva.teksti, MUSTA);
            self.teksti(x, y, &leijuva.teksti, leijuva.vari);
        }
    }

    fn kehu(&mut self, peli: &Silmukka) {
        let Some((teksti, alku)) = peli.kehu else {
            return;
        };
        let ika = self.kello.saturating_sub(alku) as f32 / KEHU_MS as f32;
        let leveys = teksti.chars().count() as i32 * SUURI.2;
        let x = KENTTA_X + (KENTTA_L - leveys) / 2;
        let y = KENTTA_Y + KENTTA_K / 2 - SUURI.1 / 2 - (ika * 36.0) as i32;
        let vari = if (self.kello / 80).is_multiple_of(2) {
            KELTAINEN
        } else {
            VALKOINEN
        };
        self.suurteksti(x, y, teksti, SUURI, vari, true);
    }

    /// Tärisevä kenttä saa ylittää rajansa, joten ympäristö maalataan vasta sen jälkeen.
    fn silmukan_kehys(&mut self) {
        self.tausta_kentan_ympari(KENTTA_X, KENTTA_Y, KENTTA_L, KENTTA_K);
        self.upotus(KENTTA_X, KENTTA_Y, KENTTA_L, KENTTA_K);
    }

    fn silmukan_tilasto(&mut self, peli: &Silmukka) {
        let lauta = &peli.lauta;
        for (i, (otsikko, arvo)) in [
            (kaanna("yhteinen.pisteet"), lauta.pisteet.to_string()),
            (kaanna("yhteinen.paras"), peli.paras.to_string()),
            (kaanna("silmukka.taso"), lauta.taso.to_string()),
            (kaanna("silmukka.pituus"), lauta.mato.len().to_string()),
        ]
        .into_iter()
        .enumerate()
        {
            let y = KENTTA_Y + i as i32 * TILASTON_ASKEL;
            self.suurteksti(SARAKE_X, y, otsikko, OTSIKKO, KELTAINEN, false);
            self.suurteksti(SARAKE_X, y + 30, &arvo, OTSIKKO, VALKOINEN, true);
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

    fn silmukan_paneeli(&mut self, peli: &Silmukka) {
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

fn siirra((x, y): (i32, i32), dx: f32, dy: f32) -> (i32, i32) {
    (x + dx as i32, y + dy as i32)
}

fn varjo(vari: u32) -> u32 {
    sekoita(vari, MUSTA, 0.5)
}

const _: () = {
    assert!(silmukka::ALKUPITUUS > HANNAN_KAPENEMINEN);
    assert!(PURKU_MS < VALAHDYS_MS);
    assert!(KEHU_MS > 0);
};

#[cfg(test)]
mod tests {
    use super::*;

    use crate::yhteinen::kieli::{kaikilla_kielilla, muotoile};
    use crate::yhteinen::syote::Syote;
    use klassikot8000::silmukka::Suunta;

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
                kaanna("silmukka.tyhja").to_string(),
                kaanna("silmukka.muna_litistyi").to_string(),
                kaanna("silmukka.kulta_ilmestyi").to_string(),
                muotoile("silmukka.uusi_taso", &[&99]),
                muotoile("silmukka.saalis", &[&19, &999999]),
                muotoile("yhteinen.tulos", &[&999999]),
            ] {
                assert!(teksti.chars().count() <= RIVILLE_MERKKEJA, "{teksti}");
            }
            // Pisin kehu mahtuu kentän leveydelle.
            for avain in [
                "silmukka.kehu_2",
                "silmukka.kehu_3",
                "silmukka.kehu_4",
                "silmukka.kehu_5",
                "silmukka.kehu_7",
                "silmukka.kehu_9",
            ] {
                assert!(
                    kaanna(avain).chars().count() as i32 * SUURI.2 < KENTTA_L,
                    "{avain}"
                );
            }
            for avain in ["silmukka.taso", "silmukka.pituus"] {
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
    fn hiiri_osuu_ruutuun_ja_ulkopuoli_rajataan_kentalle() {
        for ruutu in [(0, 0), (5, 7), (SARAKKEITA - 1, RIVEJA - 1)] {
            let (x, y) = ruudun_keskipiste(ruutu);
            assert_eq!(ruutu_pisteesta(x, y), ruutu);
        }
        assert_eq!(ruutu_pisteesta(-100, -100), (0, 0));
        assert_eq!(ruutu_pisteesta(5000, 5000), (SARAKKEITA - 1, RIVEJA - 1));
    }

    #[test]
    fn suunnat_ovat_kaikki_ruudun_mittaisia() {
        for suunta in [Suunta::Ylos, Suunta::Alas, Suunta::Vasen, Suunta::Oikea] {
            let (x, y) = suunta.siirto();
            assert_eq!(x.abs() + y.abs(), 1);
        }
    }

    #[test]
    fn koko_peli_piirtyy_kaatumatta() {
        let mut kuva = vec![0; LEVEYS * KORKEUS];
        let mut peli = Silmukka::new(3);
        piirra(&peli, &mut kuva);
        peli.kasittele(Syote::Ammu);
        // Kierros kiertää omia jälkiään, joten ennuste, silmukat ja kuolema tulevat kaikki vastaan.
        let nuolet = [(1, 0), (0, -1), (-1, 0), (0, 1)];
        for i in 0..6000 {
            peli.edista(16);
            piirra(&peli, &mut kuva);
            if i % 12 == 0 {
                let (rivi, sarake) = nuolet[i / 12 % 4];
                peli.kasittele(Syote::Nuoli(rivi, sarake));
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
