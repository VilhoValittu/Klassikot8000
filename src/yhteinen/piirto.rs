use std::f32::consts::TAU;
use std::sync::LazyLock;

use super::fontti;
use super::kieli::kaanna;
use super::rasteri::{Rasteri, ditteroi, kohina, liukuvari};
use super::tulokset::{PARHAITA, Tulos};
use super::vektori::{self, Kangas, kehapiste, sekoita_liuku, tummenna, vaalenna};

/// Maisema konepikseleinä: `horisontti` on vedenrajan rivi ja `aurinko_x` sarake, jonka
/// kohdalla aurinko laskee ja johon meren ruudut katoavat.
pub fn maisemakuva(leveys: usize, korkeus: usize, horisontti: i32, aurinko_x: i32) -> Rasteri {
    const TAIVAS: [u32; 6] = [0x0A0630, 0x1C1058, 0x4A1C80, 0xA82C88, 0xF0587C, 0xFFA868];
    const MERI: [u32; 4] = [0x503C9C, 0x1C2A80, 0x0C1450, 0x060A28];
    let auringon_sade = horisontti * 3 / 8;
    let auringon_y = horisontti - auringon_sade / 3;
    let kaukainen = |x: i32| {
        let x = x as f32;
        horisontti as f32 * (0.17 + 0.05 * (x * 0.031).sin() + 0.035 * (x * 0.083 + 1.0).sin())
    };
    let lahin = |x: i32| {
        let x = x as f32;
        horisontti as f32
            * (0.06 + 0.05 * (x * 0.047 + 2.0).sin() + 0.025 * (x * 0.13).sin()).max(0.01)
    };
    Rasteri::luo(leveys, korkeus, |x, y| {
        if y >= horisontti {
            let t = (y - horisontti) as f32 / (korkeus as i32 - horisontti) as f32;
            let etaisyys = 1.0 / (t + 0.06);
            let u = ((x - aurinko_x) as f32 * etaisyys * 0.09).floor() as i32;
            let v = (etaisyys * 2.4).floor() as i32;
            let mut vari = liukuvari(&MERI, t, x, y);
            if (u + v).rem_euclid(2) == 0 {
                vari = sekoita_liuku(vari, 0x5078E0, 0.16 * (1.0 - t));
            }
            // Auringon heijastus väpättää rivi riviltä.
            let leveys = auringon_sade as f32 * (0.75 - 0.45 * t);
            let vapina = (y as f32 * 1.7).sin() * leveys * 0.4;
            if ((x - aurinko_x) as f32 + vapina).abs() < leveys && (y % 2 == 0 || t < 0.2) {
                vari = sekoita_liuku(vari, 0xFFB070, 0.75 - 0.5 * t);
            }
            return vari;
        }
        let t = y as f32 / horisontti as f32;
        let mut vari = liukuvari(&TAIVAS, t, x, y);
        if t < 0.55 && kohina(x, y) > 0.992 {
            vari = sekoita_liuku(vari, 0xFFFFFF, 0.5 + kohina(y, x) * 0.5);
        }
        // Ohuet pilvet, jotka valaistuvat horisonttia kohti.
        let pilvi = (x as f32 * 0.04 + y as f32 * 0.9).sin() + (x as f32 * 0.013 - y as f32).sin();
        if t > 0.35 && pilvi > 1.35 {
            vari = sekoita_liuku(vari, 0xFFC8A0, 0.35);
        }
        let (dx, dy) = (x - aurinko_x, y - auringon_y);
        if dx * dx + dy * dy < auringon_sade * auringon_sade {
            let osuus = (dy + auringon_sade) as f32 / (2 * auringon_sade) as f32;
            vari = liukuvari(&[0xFFF8B0, 0xFFD048, 0xFF6878], osuus, x, y);
            // Auringon alaosan raidat levenevät alaspäin.
            let raita = dy - auringon_sade / 4;
            if raita > 0 && raita % 5 < 1 + raita * 2 / auringon_sade {
                vari = liukuvari(&TAIVAS, t, x, y);
            }
        }
        let korkeus_nyt = (horisontti - y) as f32;
        if korkeus_nyt < kaukainen(x) {
            let harja = kaukainen(x) - korkeus_nyt;
            vari = if harja < 1.5 {
                0xB060B8
            } else {
                ditteroi(x, y, 0x5A2C90, 0x40207C, (harja / 12.0).min(1.0))
            };
        }
        if korkeus_nyt < lahin(x) {
            let harja = lahin(x) - korkeus_nyt;
            vari = if harja < 1.5 {
                0x5848A0
            } else {
                ditteroi(x, y, 0x241860, 0x180F48, (harja / 8.0).min(1.0))
            };
        }
        vari
    })
}

/// Pelinäkymien tausta: tummansininen timanttikuvio, joka vaalenee alaspäin.
static PELITAUSTA: LazyLock<Rasteri> = LazyLock::new(|| {
    const RUUTU: i32 = 16;
    Rasteri::luo(LEVEYS / 3, KORKEUS / 3, |x, y| {
        let t = y as f32 / (KORKEUS / 3) as f32;
        let vari = liukuvari(&[TAUSTA_TUMMA, TAUSTA], t, x, y);
        let (dx, dy) = (
            (x.rem_euclid(RUUTU) - RUUTU / 2).abs(),
            (y.rem_euclid(RUUTU) - RUUTU / 2).abs(),
        );
        match dx + dy {
            6 => sekoita_liuku(vari, 0x5068D8, 0.28),
            7 => sekoita_liuku(vari, 0x000000, 0.25),
            _ => vari,
        }
    })
});

/// Alalaidan viesti-ikkuna valmiiksi laskettuna: lähes joka pelinäkymässä, joka ruudussa.
static PANEELIKUVA: LazyLock<Rasteri> = LazyLock::new(|| {
    let (l, k) = (LEVEYS - 12, PANEELI_K as usize);
    let mut kuva = vec![0; l * k];
    Piirtaja {
        k: Kangas::kokoinen(&mut kuva, l, k),
        kello: 0,
    }
    .ikkuna(0, 0, l as i32, k as i32);
    Rasteri::kuvasta(&kuva, l, 3)
});

static ALKUMAISEMA: LazyLock<Rasteri> = LazyLock::new(|| {
    maisemakuva(
        LEVEYS / 3,
        MAISEMA_ALA_Y as usize / 3,
        HORISONTTI_Y / 3,
        LEVEYS as i32 / 6,
    )
});

/// Korttipöytien vihreä verka, joka tummenee reunoja kohti.
static VERKA: LazyLock<Rasteri> = LazyLock::new(|| {
    let (l, k) = (RUUDUN_LEVEYS / 4, RUUDUN_KORKEUS / 4);
    let (keski_x, keski_y) = (l as f32 * 0.36, k as f32 * 0.5);
    Rasteri::luo(l, k, |x, y| {
        let (dx, dy) = (
            (x as f32 - keski_x) / l as f32,
            (y as f32 - keski_y) / k as f32,
        );
        let t = (dx * dx * 2.2 + dy * dy * 2.4).sqrt() * 1.25;
        let vari = liukuvari(&[0x1C8050, 0x106440, 0x07402C, 0x042418], t, x, y);
        if kohina(x, y) > 0.88 {
            sekoita_liuku(vari, 0x000000, 0.12)
        } else {
            vari
        }
    })
});

/// Sivupalkkiin mahtuu yhdeksän merkkiä.
pub fn listan_otsikko(netti: bool) -> &'static str {
    kaanna(if netti {
        "yhteinen.netti_top"
    } else {
        "yhteinen.top3"
    })
}

pub const LEVEYS: usize = 960;
pub const KORKEUS: usize = 768;

/// Ikkunan todellinen koko. `LEVEYS` ja `KORKEUS` ovat klassisten näkymien kuva, joka
/// skaalataan ruudun korkuiseksi keskelle; laidoille jää tumma palkki kuten television reunoille.
pub const RUUDUN_LEVEYS: usize = 1920;
pub const RUUDUN_KORKEUS: usize = 1080;
// Suhde 1080 / 768 = 45 / 32.
const SKAALA_OSOITTAJA: usize = 45;
const SKAALA_NIMITTAJA: usize = 32;
const SKAALATTU_LEVEYS: usize = LEVEYS * SKAALA_OSOITTAJA / SKAALA_NIMITTAJA;
const SKAALATTU_X: usize = (RUUDUN_LEVEYS - SKAALATTU_LEVEYS) / 2;

// Paletti: SNES- ja Mega Drive -pelien tyydyttyneet siniset, kulta ja violettiin taittuvat varjot.
pub const PALKKI: u32 = 0x07041A;
pub const REUNA: u32 = 0x8C84F0;
pub const TAUSTA: u32 = 0x2C2070;
pub const TAUSTA_TUMMA: u32 = 0x120A38;
pub const VESI: u32 = 0x0B1650;
pub const VALKOINEN: u32 = 0xFFFFFF;
pub const MUSTA: u32 = 0x000000;
pub const WB_SININEN: u32 = 0x0848B8;
pub const WB_ORANSSI: u32 = 0xFF8800;
pub const RUNKO: u32 = 0x5C6B78;
pub const KANSI: u32 = 0x8A98A4;
pub const TORNI: u32 = 0xC8D2DA;
pub const LAIVA_REUNA: u32 = 0xE8F4FF;
pub const OSUMA: u32 = 0xC8382A;
pub const OSUMA_TUMMA: u32 = 0x4A0E0A;
pub const HUTI: u32 = 0x78D8E0;
pub const KELTAINEN: u32 = 0xFFE860;
pub const TAIVAS_ALA: u32 = 0xD8508C;
pub const LAVENTELI: u32 = 0xC8D0FF;
const IKKUNA_YLA: u32 = 0x2848C8;
const IKKUNA_ALA: u32 = 0x0C1468;
const IKKUNA_REUNA: u32 = 0xD8E4FF;
const IKKUNA_VARJO: u32 = 0x8090C8;
const ULKOVIIVA: u32 = 0x0A0420;

pub const MERKKI: i32 = 3 * fontti::LEVEYS;
pub const RIVIN_KORKEUS: i32 = 24;
pub const PALKIN_PALA: i32 = 18;
pub const PANEELI_Y: i32 = 603;
pub const PANEELI_K: i32 = 162;
pub const PANEELI_RIVI_Y: i32 = PANEELI_Y + 9;
pub const PANEELI_RIVEJA: usize = 6;
pub const TEKSTI_X: i32 = 24;
pub const RIVILLE_MERKKEJA: usize = 38;
pub const NAPPI_Y: i32 = PANEELI_RIVI_Y + 5 * RIVIN_KORKEUS - 6;
pub const NAPPI_K: i32 = 30;
pub const NAPPI_X: i32 = 30;
const AANI_PAALLA: &str = "yhteinen.aani_paalla";
const AANI_POIS: &str = "yhteinen.aani_pois";
// Mahtuu pidemmän tekstin mukaan, jottei kytkin muuta kokoaan.
const AANI_L: i32 = 12 * MERKKI + 24;
pub const AANI_X: i32 = LEVEYS as i32 - 12 - AANI_L;
const AANI_Y: i32 = 6;
const AANI_K: i32 = 30;

pub const VILKKU_MS: u32 = 400;

// Alkuruutujen viimeinen rivi: "Paina välilyöntiä..."
pub const ALOITUS_Y: i32 = 708;
pub const ALOITUS: &str = "yhteinen.aloitus";
const ALKU_MERKKEJA: usize = 36;

// Alkuruudun maisema
const MAISEMA_REUNA: i32 = 24;
const HORISONTTI_Y: i32 = 195;
const MAISEMA_ALA_Y: i32 = 348;
pub const OHJE_Y: i32 = 360;
pub const LAIVAN_KIERTO: f32 = 0.0006;
const LAIVAN_KOKO: f32 = 33.0;
// Meri aaltoilee rivikohtaisella siirrolla, kuten SNES:n HDMA-tehosteissa.
const AALLON_NOPEUS: f32 = 0.002;
const AALLON_ASKEL_MS: u32 = 120;
const AALLON_TIHEYS: f32 = 0.45;
const AALLON_LAAJUUS: f32 = 2.5;

// (merkin leveys, merkin korkeus, askel) suurtekstille
pub const SUURI: (i32, i32, i32) = (30, 45, 39);
pub const OTSIKKO: (i32, i32, i32) = (15, 21, 21);
pub const TULOS: (i32, i32, i32) = (12, 18, 18);

const PARHAAT_RIVI: i32 = 26;
// Alkuruudulla lista on maiseman vasemmassa laidassa, pyörivän mallin vieressä.
const ALKU_PARHAAT: (i32, i32, i32, i32) = (36, 204, 318, 126);
const NIMI_OTSIKKO: &str = "yhteinen.nimi_otsikko";
pub const NIMI_OHJE: &str = "yhteinen.nimi_ohje";
const NIMI_IKKUNA_L: i32 = 912;
const NIMI_IKKUNA_K: i32 = 120;

pub fn aanikytkimella(x: i32, y: i32) -> bool {
    (AANI_X..AANI_X + AANI_L).contains(&x) && (AANI_Y..AANI_Y + AANI_K).contains(&y)
}

#[cfg(test)]
pub fn aanikytkimen_keskipiste() -> (i32, i32) {
    (AANI_X + AANI_L / 2, AANI_Y + AANI_K / 2)
}

pub fn napin_leveys(avain: &str) -> i32 {
    kaanna(avain).chars().count() as i32 * MERKKI + 24
}

/// Ruudun piirtopuskurit. Klassinen kuva verrataan edelliseen ennen venytystä, koska pieni
/// kuva on nopea verrata ja useimmat sen riveistä säilyvät ruudusta toiseen.
pub struct Kehys {
    pub ruutu: Vec<u32>,
    pub kuva: Vec<u32>,
    edellinen_kuva: Vec<u32>,
    /// Ruudun rivit, jotka ovat voineet muuttua edellisestä piirrosta.
    pub muuttuneet: Vec<bool>,
    /// Ruudulla on edellisen klassisen kuvan venytys.
    venytetty: bool,
}

impl Default for Kehys {
    fn default() -> Self {
        Self {
            ruutu: vec![0; RUUDUN_LEVEYS * RUUDUN_KORKEUS],
            kuva: vec![0; LEVEYS * KORKEUS],
            edellinen_kuva: vec![0; LEVEYS * KORKEUS],
            muuttuneet: vec![true; RUUDUN_KORKEUS],
            venytetty: false,
        }
    }
}

impl Kehys {
    /// Venyttää klassisen kuvan muuttuneet rivit ruudulle. Lähimmän pikselin menetelmä
    /// säilyttää kovat reunat, ja sama rivi kopioidaan laskematta uudelleen.
    pub fn venyta(&mut self) {
        let mut kuvan_rivit = [false; KORKEUS];
        for ((muuttui, uusi), vanha) in kuvan_rivit
            .iter_mut()
            .zip(self.kuva.chunks_exact(LEVEYS))
            .zip(self.edellinen_kuva.chunks_exact_mut(LEVEYS))
        {
            *muuttui = uusi != &*vanha;
            if *muuttui {
                vanha.copy_from_slice(uusi);
            }
        }
        let kaikki = !self.venytetty;
        self.venytetty = true;
        let lahde_x: [u16; SKAALATTU_LEVEYS] =
            std::array::from_fn(|x| (x * SKAALA_NIMITTAJA / SKAALA_OSOITTAJA) as u16);
        let mut edellinen = None;
        for y in 0..RUUDUN_KORKEUS {
            let lahde_y = y * SKAALA_NIMITTAJA / SKAALA_OSOITTAJA;
            let muuttui = kaikki || kuvan_rivit[lahde_y];
            self.muuttuneet[y] = muuttui;
            if !muuttui {
                continue;
            }
            let rivi = y * RUUDUN_LEVEYS;
            if edellinen == Some(lahde_y) {
                self.ruutu.copy_within(rivi - RUUDUN_LEVEYS..rivi, rivi);
                continue;
            }
            edellinen = Some(lahde_y);
            let kohde = &mut self.ruutu[rivi..rivi + RUUDUN_LEVEYS];
            kohde.fill(PALKKI);
            let lahde = &self.kuva[lahde_y * LEVEYS..(lahde_y + 1) * LEVEYS];
            for (pikseli, &x) in kohde[SKAALATTU_X..].iter_mut().zip(&lahde_x) {
                *pikseli = lahde[x as usize];
            }
        }
    }

    /// Näkymä piirsi suoraan ruudulle, joten mikä tahansa rivi on voinut muuttua.
    pub fn piirretty_suoraan(&mut self) {
        self.venytetty = false;
        self.muuttuneet.fill(true);
    }
}

/// Ruudun pikseli klassisen kuvan koordinaateiksi. Laidoilla x jää kuvan ulkopuolelle.
pub fn ruudulta_kuvaan(x: i32, y: i32) -> (i32, i32) {
    let (osoittaja, nimittaja) = (SKAALA_OSOITTAJA as i32, SKAALA_NIMITTAJA as i32);
    (
        ((x - SKAALATTU_X as i32) * nimittaja).div_euclid(osoittaja),
        (y * nimittaja).div_euclid(osoittaja),
    )
}

/// Kohta, johon valmis ruutu mahtuu ikkunassa kuvasuhde säilyen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sovitus {
    pub x: usize,
    pub y: usize,
    pub l: usize,
    pub k: usize,
}

impl Sovitus {
    /// Ikkunan koon on oltava vähintään 1x1.
    pub fn new(ikkuna_l: usize, ikkuna_k: usize) -> Self {
        let (l, k) = if ikkuna_l * RUUDUN_KORKEUS > ikkuna_k * RUUDUN_LEVEYS {
            ((ikkuna_k * RUUDUN_LEVEYS / RUUDUN_KORKEUS).max(1), ikkuna_k)
        } else {
            (ikkuna_l, (ikkuna_l * RUUDUN_KORKEUS / RUUDUN_LEVEYS).max(1))
        };
        Self {
            x: (ikkuna_l - l) / 2,
            y: (ikkuna_k - k) / 2,
            l,
            k,
        }
    }

    /// Ikkunan pikseli ruudun koordinaateiksi. Sivupalkeilla ei osu mihinkään.
    pub fn ruudulle(&self, x: i32, y: i32) -> Option<(i32, i32)> {
        let x = usize::try_from(x).ok()?.checked_sub(self.x)?;
        let y = usize::try_from(y).ok()?.checked_sub(self.y)?;
        (x < self.l && y < self.k).then_some((
            (x * RUUDUN_LEVEYS / self.l) as i32,
            (y * RUUDUN_KORKEUS / self.k) as i32,
        ))
    }
}

/// Lähdepikselit, jotka kuuluvat kohteen pikseliin `i`. Aina vähintään yksi.
fn lahdevali(i: usize, kohde: usize, lahde: usize) -> (usize, usize) {
    let alku = i * lahde / kohde;
    (alku, ((i + 1) * lahde / kohde).max(alku + 1))
}

/// Ikkunan kokoinen kuva. Muistaa edellisen ruudun, koska suurin osa riveistä säilyy
/// ruudusta toiseen ja keskiarvojen laskeminen on piirron kallein vaihe.
#[derive(Default)]
pub struct Ikkunakuva {
    pub pikselit: Vec<u32>,
    edellinen: Vec<u32>,
    /// Kunkin ruudun rivin muuttunut pikseliväli.
    muuttuneet: Vec<Option<(usize, usize)>>,
    sarakkeet: Vec<(usize, usize)>,
    koko: Option<(usize, usize, Sovitus)>,
}

impl Ikkunakuva {
    /// Skaalaa ruudun ikkunan kokoiseksi. Pienennys laskee keskiarvon, jotta ohuet viivat
    /// säilyvät; Windowsin oma venytys hävittäisi ne. Vain `ehdokkaat`-riveiltä etsitään
    /// muutoksia, ja vain muuttuneet kohdat lasketaan. Sama rivi kopioidaan laskematta
    /// uudelleen. Palauttaa tiedon, muuttuiko kuva.
    pub fn sovita(
        &mut self,
        ruutu: &[u32],
        ehdokkaat: &[bool],
        ikkuna_l: usize,
        ikkuna_k: usize,
        sovitus: Sovitus,
    ) -> bool {
        let koko = Some((ikkuna_l, ikkuna_k, sovitus));
        let kaikki = self.koko != koko;
        if kaikki {
            self.koko = koko;
            self.pikselit.clear();
            self.pikselit.resize(ikkuna_l * ikkuna_k, PALKKI);
            self.sarakkeet = (0..sovitus.l)
                .map(|x| lahdevali(x, sovitus.l, RUUDUN_LEVEYS))
                .collect();
            self.edellinen.clear();
            self.edellinen.extend_from_slice(ruutu);
        }
        self.muuttuneet.clear();
        for ((uusi, vanha), &ehdokas) in ruutu
            .chunks_exact(RUUDUN_LEVEYS)
            .zip(self.edellinen.chunks_exact_mut(RUUDUN_LEVEYS))
            .zip(ehdokkaat)
        {
            let muutos = (ehdokas && uusi != &*vanha).then(|| {
                let eroaa = |(a, b): (&u32, &u32)| a != b;
                let alku = uusi.iter().zip(&*vanha).position(eroaa).unwrap_or(0);
                let loppu = uusi.iter().zip(&*vanha).rposition(eroaa).unwrap_or(0) + 1;
                vanha.copy_from_slice(uusi);
                (alku, loppu)
            });
            self.muuttuneet.push(muutos);
        }
        if !kaikki && self.muuttuneet.iter().all(Option::is_none) {
            return false;
        }

        let ikkuna = &mut self.pikselit;
        let mut edellinen = None;
        for y in 0..sovitus.k {
            let rivit = lahdevali(y, sovitus.k, RUUDUN_KORKEUS);
            // Pallo ja maila muuttavat eri kohtia, joten sarakkeet rajataan riveittäin.
            let (s0, s1) = if kaikki {
                (0, sovitus.l)
            } else {
                let Some((alku, loppu)) = self.muuttuneet[rivit.0..rivit.1]
                    .iter()
                    .flatten()
                    .copied()
                    .reduce(|a, b| (a.0.min(b.0), a.1.max(b.1)))
                else {
                    continue;
                };
                (
                    self.sarakkeet.partition_point(|s| s.1 <= alku),
                    self.sarakkeet.partition_point(|s| s.0 < loppu),
                )
            };
            let alku = (sovitus.y + y) * ikkuna_l + sovitus.x;
            if edellinen == Some(rivit) {
                let yla = alku - ikkuna_l;
                ikkuna.copy_within(yla + s0..yla + s1, alku + s0);
                continue;
            }
            edellinen = Some(rivit);
            let kohde = &mut ikkuna[alku + s0..alku + s1];
            for (pikseli, &(x0, x1)) in kohde.iter_mut().zip(&self.sarakkeet[s0..s1]) {
                let maara = ((x1 - x0) * (rivit.1 - rivit.0)) as u32;
                if maara == 1 {
                    *pikseli = ruutu[rivit.0 * RUUDUN_LEVEYS + x0];
                    continue;
                }
                let (mut r, mut g, mut b) = (0, 0, 0);
                for rivi in rivit.0..rivit.1 {
                    for &vari in &ruutu[rivi * RUUDUN_LEVEYS + x0..rivi * RUUDUN_LEVEYS + x1] {
                        r += (vari >> 16) & 0xFF;
                        g += (vari >> 8) & 0xFF;
                        b += vari & 0xFF;
                    }
                }
                *pikseli = (r / maara) << 16 | (g / maara) << 8 | (b / maara);
            }
        }
        true
    }
}

/// Rivittää tekstin sanojen välistä. Ei kopioi tekstiä, palauttaa vain viipaleet.
pub fn rivita(teksti: &str, leveys: usize) -> Vec<&str> {
    let mut rivit = Vec::new();
    let mut alku = 0;
    let mut viime_vali = None;
    let mut merkkeja = 0;
    for (i, merkki) in teksti.char_indices() {
        if merkkeja == leveys {
            let katko = viime_vali.unwrap_or(i);
            rivit.push(&teksti[alku..katko]);
            alku = if viime_vali.is_some() {
                katko + 1
            } else {
                katko
            };
            merkkeja = teksti[alku..i].chars().count();
            viime_vali = None;
        }
        if merkki == ' ' {
            viime_vali = Some(i);
        }
        merkkeja += 1;
    }
    if alku < teksti.len() {
        rivit.push(&teksti[alku..]);
    }
    rivit
}

/// Sekoitetaan kaksi väriä, `osuus` 0 antaa ensimmäisen ja 1 toisen.
pub fn sekoita(a: u32, b: u32, osuus: f32) -> u32 {
    let (a, b) = (a as i32, b as i32);
    let kanava = |siirto: i32| {
        let (x, y) = (a >> siirto & 0xFF, b >> siirto & 0xFF);
        ((x + ((y - x) as f32 * osuus) as i32) as u32) << siirto
    };
    kanava(16) | kanava(8) | kanava(0)
}

pub struct Piirtaja<'a> {
    pub k: Kangas<'a>,
    pub kello: u32,
}

impl Piirtaja<'_> {
    pub fn vilkku(&self) -> bool {
        (self.kello / VILKKU_MS).is_multiple_of(2)
    }

    /// Bittikarttateksti kolminkertaisena: 40 merkkiä riville.
    pub fn teksti(&mut self, x: i32, y: i32, teksti: &str, vari: u32) {
        self.teksti_koossa(x, y, teksti, 3, vari);
    }

    /// Bittikarttateksti `kerroin`-kertaisena.
    pub fn teksti_koossa(&mut self, x: i32, y: i32, teksti: &str, kerroin: i32, vari: u32) {
        self.merkit(
            x,
            y,
            teksti,
            (kerroin, kerroin, kerroin * fontti::LEVEYS),
            vari,
            false,
        );
    }

    /// Suurempi kirjasin otsikoille ja lukemille. `koko` on (merkin leveys, korkeus, askel)
    /// pikseleinä, `paksu` lisää tumman ääriviivan.
    pub fn suurteksti(
        &mut self,
        x: i32,
        y: i32,
        teksti: &str,
        koko: (i32, i32, i32),
        vari: u32,
        paksu: bool,
    ) {
        let pikseli = |mitta: i32| ((mitta as f32 / 7.0).round() as i32).max(1);
        self.merkit(
            x,
            y,
            teksti,
            (pikseli(koko.0), pikseli(koko.1), koko.2),
            vari,
            paksu,
        );
    }

    /// Vaaleat merkit saavat varjon ja liukuvärin ylhäältä alas; tummat ovat vaalealla
    /// pohjalla, joten ne piirtyvät tasaisina.
    fn merkit(
        &mut self,
        x: i32,
        y: i32,
        teksti: &str,
        (pl, pk, askel): (i32, i32, i32),
        vari: u32,
        aariviiva: bool,
    ) {
        let vaalea = on_vaalea(vari);
        let rivivarit: [u32; 8] = std::array::from_fn(|rivi| {
            if !vaalea {
                return vari;
            }
            match rivi {
                0 | 1 => vaalenna(vari, 0.55),
                2 => vaalenna(vari, 0.25),
                3 | 4 => vari,
                5 => tummenna(vari, 0.1),
                _ => tummenna(vari, 0.22),
            }
        });
        let varjot: &[(i32, i32)] = match (vaalea, aariviiva) {
            (false, _) => &[],
            // Rivit ovat tiheässä, joten leipätekstin varjo menee vain sivulle.
            (true, false) => &[(1, 0)],
            (true, true) => &[(-1, 0), (1, 0), (0, -1), (0, 1), (1, 1)],
        };
        for &(dx, dy) in varjot {
            self.kuviot(x + dx * pl, y + dy * pk, teksti, (pl, pk, askel), |_| {
                ULKOVIIVA
            });
        }
        self.kuviot(x, y, teksti, (pl, pk, askel), |rivi| rivivarit[rivi]);
    }

    fn kuviot(
        &mut self,
        x: i32,
        y: i32,
        teksti: &str,
        (pl, pk, askel): (i32, i32, i32),
        vari: impl Fn(usize) -> u32,
    ) {
        for (i, merkki) in teksti.chars().enumerate() {
            let vasen = x + i as i32 * askel;
            for (rivi, bitit) in fontti::kuvio(merkki).iter().enumerate() {
                let py = y + rivi as i32 * pk;
                for sarake in 0..8 {
                    if bitit >> sarake & 1 != 0 {
                        self.k.laatikko(vasen + sarake * pl, py, pl, pk, vari(rivi));
                    }
                }
            }
        }
    }

    /// Pelinäkymien yhteinen tausta. Korttipöydillä on täysi ruutu ja vihreä verka.
    pub fn pelitausta(&mut self) {
        if self.k.leveys > LEVEYS {
            VERKA.piirra(&mut self.k, 0, 0);
        } else {
            PELITAUSTA.piirra(&mut self.k, 0, 0);
        }
    }

    /// Tausta muualle kuin kentälle, jonka yli kappaleet saavat ulottua. Mitat pikseleinä.
    /// Alalaidan viesti-ikkuna peittää oman alueensa, joten sen alle tulevat vain reunat.
    pub fn tausta_kentan_ympari(&mut self, x: i32, y: i32, l: i32, k: i32) {
        let p = self.k.pikseli;
        let cu = p as usize;
        let alue = (
            x as usize / cu,
            y as usize / cu,
            l as usize / cu,
            k as usize / cu,
        );
        let paneeli = (PANEELI_Y / p) as usize;
        let ala = ((PANEELI_Y + PANEELI_K) / p) as usize;
        let (leveys, korkeus) = (LEVEYS / cu, KORKEUS / cu);
        let reuna = (6 / p) as usize;
        let tausta = &*PELITAUSTA;
        tausta.piirra_paitsi(&mut self.k, alue, paneeli);
        tausta.piirra_alue(
            &mut self.k,
            0,
            PANEELI_Y,
            (0, paneeli, reuna, ala - paneeli),
        );
        let oikea = leveys - reuna;
        let oikea_x = oikea as i32 * p;
        tausta.piirra_alue(
            &mut self.k,
            oikea_x,
            PANEELI_Y,
            (oikea, paneeli, reuna, ala - paneeli),
        );
        tausta.piirra_alue(
            &mut self.k,
            0,
            ala as i32 * p,
            (0, ala, leveys, korkeus - ala),
        );
    }

    /// Alkuruudun tausta: maisema, otsikko, ikkuna tekstille ja malli edessä.
    pub fn maisema(&mut self, otsikko: &str, malli: &[vektori::Tahko]) {
        self.pelitausta();
        self.alkumaisema();
        let keinunta = (self.kello as f32 * 0.002).sin() * 3.0;
        self.k.malli(
            malli,
            self.kello as f32 * LAIVAN_KIERTO,
            (LEVEYS as i32 / 2, HORISONTTI_Y + 33 + keinunta as i32),
            LAIVAN_KOKO,
        );
        self.maiseman_otsikko(otsikko);

        let (l, k) = (LEVEYS as i32, KORKEUS as i32);
        let leveys = l - 2 * MAISEMA_REUNA;
        self.upotus(
            MAISEMA_REUNA,
            MAISEMA_REUNA,
            leveys,
            MAISEMA_ALA_Y - MAISEMA_REUNA,
        );
        let ikkuna_y = MAISEMA_ALA_Y + 3;
        self.ikkuna(
            MAISEMA_REUNA,
            ikkuna_y,
            leveys,
            k - ikkuna_y - MAISEMA_REUNA,
        );
    }

    /// Taivas pysyy paikallaan ja vedenpinnan rivit aaltoilevat sivusuunnassa.
    fn alkumaisema(&mut self) {
        let p = self.k.pikseli;
        let reuna = (MAISEMA_REUNA / p) as usize;
        let horisontti = (HORISONTTI_Y / p) as usize;
        let kuva = &*ALKUMAISEMA;
        let leveys = LEVEYS / 3 - 2 * reuna;
        // Aalto etenee askelin: harvemmin vaihtuvat rivit keventävät venytystä.
        let vaihe = (self.kello / AALLON_ASKEL_MS * AALLON_ASKEL_MS) as f32 * AALLON_NOPEUS;
        for rivi in reuna..kuva.korkeus() {
            let siirto = if rivi < horisontti {
                0
            } else {
                let syvyys = (rivi - horisontti) as f32 / (kuva.korkeus() - horisontti) as f32;
                let aalto = (vaihe + rivi as f32 * AALLON_TIHEYS).sin();
                (aalto * AALLON_LAAJUUS * (0.3 + syvyys)).round() as i32
            };
            let alku = (reuna as i32 + siirto) as usize;
            let lahde = &kuva.rivi(rivi)[alku..alku + leveys];
            self.k.karkea_rivi(MAISEMA_REUNA, rivi as i32 * p, lahde);
        }
    }

    /// SNES-roolipelien tapainen ikkuna: pyöristetyt kulmat, vaalea kehys ja sininen liuku.
    pub fn ikkuna(&mut self, x: i32, y: i32, l: i32, k: i32) {
        let p = self.k.pikseli;
        self.pyorea(x, y, l, k, ULKOVIIVA);
        self.pyorea(x + p, y + p, l - 2 * p, k - 2 * p, IKKUNA_REUNA);
        self.k
            .laatikko(x + 2 * p, y + k - 2 * p - p, l - 4 * p, p, IKKUNA_VARJO);
        self.k.liukulaatikko(
            x + 2 * p,
            y + 2 * p,
            l - 4 * p,
            k - 4 * p - p,
            IKKUNA_YLA,
            IKKUNA_ALA,
        );
    }

    /// Pelinäkymien alalaidan viesti-ikkuna.
    pub fn paneelin_tausta(&mut self) {
        PANEELIKUVA.piirra(&mut self.k, 6, PANEELI_Y);
    }

    pub fn tausta_alue(&mut self, x: i32, y: i32, l: i32, k: i32) {
        let p = self.k.pikseli;
        let (ax, ay) = ((x / p) as usize, (y / p) as usize);
        let alue = (ax, ay, (l / p) as usize, (k / p) as usize);
        PELITAUSTA.piirra_alue(&mut self.k, x, y, alue);
    }

    fn pyorea(&mut self, x: i32, y: i32, l: i32, k: i32, vari: u32) {
        let p = self.k.pikseli;
        self.k.laatikko(x + p, y, l - 2 * p, k, vari);
        self.k.laatikko(x, y + p, l, k - 2 * p, vari);
    }

    /// Upotettu reuna kentän tai kuvan ympärille: valo tulee vasemmalta ylhäältä.
    pub fn upotus(&mut self, x: i32, y: i32, l: i32, k: i32) {
        let p = self.k.pikseli;
        let (ulko_x, ulko_y) = (x - p, y - p);
        let (ulko_l, ulko_k) = (l + 2 * p, k + 2 * p);
        self.k.laatikko(ulko_x, ulko_y, ulko_l, p, ULKOVIIVA);
        self.k.laatikko(ulko_x, ulko_y, p, ulko_k, ULKOVIIVA);
        self.k.laatikko(ulko_x, y + k, ulko_l, p, 0x7C88D8);
        self.k.laatikko(x + l, ulko_y, p, ulko_k, 0x7C88D8);
    }

    /// Alkuruudun ohjeet: `osat` rivitetään, `rivit` ovat näppäinohjeita. Palauttaa seuraavan vapaan rivin.
    pub fn alkuohje(&mut self, osat: &[&str], rivit: &[&str]) -> i32 {
        let mut y = OHJE_Y;
        for osa in osat {
            for rivi in rivita(kaanna(osa), ALKU_MERKKEJA) {
                self.teksti(48, y, rivi, LAVENTELI);
                y += RIVIN_KORKEUS;
            }
        }
        y += RIVIN_KORKEUS / 2;
        for rivi in rivit {
            self.teksti(48, y, kaanna(rivi), VALKOINEN);
            y += RIVIN_KORKEUS;
        }
        y
    }

    pub fn aloituskehote(&mut self, avain: &str) {
        let teksti = kaanna(avain);
        self.teksti(48, ALOITUS_Y, teksti, KELTAINEN);
        if self.vilkku() {
            let x = 48 + teksti.chars().count() as i32 * MERKKI;
            self.k.laatikko(x, ALOITUS_Y, MERKKI, MERKKI, KELTAINEN);
        }
    }

    pub fn maiseman_otsikko(&mut self, teksti: &str) {
        let x = (LEVEYS as i32 - teksti.chars().count() as i32 * SUURI.2) / 2;
        self.suurteksti(x, 45, teksti, SUURI, KELTAINEN, true);
    }

    /// Ylälaidan metallipalkki: pelin nimi, lisätieto ja niiden välissä niitit.
    pub fn otsikkopalkki(&mut self, otsikko: &str, nimi: &str) {
        let l = self.k.leveys as i32 - 12;
        self.nappitausta(6, 6, l, 30, 0x384890);
        self.teksti(18, 9, otsikko, KELTAINEN);
        let aani_x = self.k.leveys as i32 - 12 - AANI_L;
        let nimi_x = aani_x - 12 - nimi.chars().count() as i32 * MERKKI;
        self.teksti(nimi_x, 9, nimi, VALKOINEN);
        let alku = 36 + otsikko.chars().count() as i32 * MERKKI;
        let mut x = alku;
        while x + 6 < nimi_x - 12 {
            self.k.laatikko(x, 18, 6, 6, 0x182464);
            self.k.laatikko(x, 18, 3, 3, 0x90A0E8);
            x += 24;
        }
    }

    /// Sijoitus ja nimi vasemmalla, pisteet oikeassa laidassa. Tyhjä paikka näkyy viivana.
    pub fn parhaat_lista(
        &mut self,
        x: i32,
        y: i32,
        leveys: i32,
        koko: (i32, i32, i32),
        otsikko: &str,
        parhaat: &[Tulos],
    ) {
        self.teksti(x, y, otsikko, KELTAINEN);
        for i in 0..PARHAITA {
            let rivi_y = y + 30 + i as i32 * PARHAAT_RIVI;
            let tulos = parhaat.get(i);
            let nimi = tulos.map_or("-", |t| t.nimi.as_str());
            let vasen = format!("{} {nimi}", i + 1);
            self.suurteksti(x, rivi_y, &vasen, koko, VALKOINEN, false);
            if let Some(tulos) = tulos {
                let pisteet = tulos.pisteet.to_string();
                let pituus = pisteet.chars().count() as i32 * koko.2;
                let oikea = x + leveys - pituus;
                self.suurteksti(oikea, rivi_y, &pisteet, koko, KELTAINEN, false);
            }
        }
    }

    pub fn parhaat_alkuruudulla(&mut self, otsikko: &str, parhaat: &[Tulos]) {
        let (x, y, l, k) = ALKU_PARHAAT;
        self.ikkuna(x, y, l, k);
        self.parhaat_lista(x + 12, y + 9, l - 24, TULOS, otsikko, parhaat);
    }

    /// Paneelin sisältö, kun tulos pääsi listalle ja odottaa nimimerkkiä.
    pub fn nimikysely(&mut self, pisteet: u32, nimi: &str) {
        let otsikko = format!("{} {pisteet}", kaanna(NIMI_OTSIKKO));
        let ohje_y = PANEELI_RIVI_Y + 2 * RIVIN_KORKEUS;
        self.suurteksti(TEKSTI_X, ohje_y - 21, &otsikko, TULOS, KELTAINEN, true);
        self.teksti(TEKSTI_X, ohje_y + 6, kaanna(NIMI_OHJE), HUTI);
        let alarivi = PANEELI_RIVI_Y + (PANEELI_RIVEJA as i32 - 1) * RIVIN_KORKEUS;
        self.nimikentta(TEKSTI_X, alarivi, nimi);
    }

    /// Sama kysely täysikokoisille korttipöydille, joilla ei ole alapaneelia.
    pub fn nimi_ikkuna(&mut self, pisteet: u32, nimi: &str) {
        let x = (RUUDUN_LEVEYS as i32 - NIMI_IKKUNA_L) / 2;
        let y = (RUUDUN_KORKEUS as i32 - NIMI_IKKUNA_K) / 2;
        self.ikkuna(x, y, NIMI_IKKUNA_L, NIMI_IKKUNA_K);
        let otsikko = format!("{} {pisteet}", kaanna(NIMI_OTSIKKO));
        let teksti_x = x + 18;
        self.suurteksti(teksti_x, y + 15, &otsikko, OTSIKKO, KELTAINEN, true);
        self.teksti(teksti_x, y + 48, kaanna(NIMI_OHJE), HUTI);
        self.nimikentta(teksti_x, y + 84, nimi);
    }

    fn nimikentta(&mut self, x: i32, y: i32, nimi: &str) {
        let kentta = format!("> {nimi}");
        self.teksti(x, y, &kentta, VALKOINEN);
        if self.vilkku() {
            let loppu = x + kentta.chars().count() as i32 * MERKKI;
            self.k.laatikko(loppu, y, MERKKI, MERKKI, REUNA);
        }
    }

    /// Tulipallo, joka haalenee savuksi, ja sen ympärillä sinkoilevat sirpaleet.
    pub fn rajahdys(&mut self, keski: (i32, i32), ika: f32, sateita: i32, koko: f32) {
        let ika = ika.clamp(0.0, 1.0);
        let p = self.k.pikseli;
        let kasvu = (ika * 4.0).min(1.0) * (1.0 - ika * ika);
        let sade = (koko * 0.42 * kasvu) as i32;
        if ika < 0.75 {
            self.k.kiekko(keski, sade + p, OSUMA);
            self.k.kiekko(keski, sade, WB_ORANSSI);
            self.k.kiekko(keski, sade * 2 / 3, KELTAINEN);
            self.k.kiekko(keski, sade / 3, VALKOINEN);
        } else {
            self.k.kiekko(keski, sade, 0x484058);
            self.k
                .kiekko((keski.0 - p, keski.1 - p), sade / 2, 0x6C6478);
        }
        for i in 0..sateita {
            // Sirpaleet lentävät eri pituisina, jottei räjähdys ole ympyrä.
            let kerroin = 0.6 + ((i * 37) % 11) as f32 / 25.0;
            let kulma = i as f32 / sateita as f32 * TAU;
            let (x, y) = kehapiste(keski, ika * koko * kerroin + 6.0, kulma);
            let vari = if ika < 0.3 {
                VALKOINEN
            } else if ika < 0.6 {
                KELTAINEN
            } else {
                WB_ORANSSI
            };
            let sivu = if ika < 0.5 { 2 * p } else { p };
            self.k.laatikko(x / p * p, y / p * p, sivu, sivu, vari);
        }
    }

    pub fn aanikytkin(&mut self, paalla: bool) {
        let (tausta, teksti) = if paalla {
            (VALKOINEN, AANI_PAALLA)
        } else {
            (WB_ORANSSI, AANI_POIS)
        };
        let x = self.k.leveys as i32 - 12 - AANI_L;
        self.nappitausta(x, AANI_Y, AANI_L, AANI_K, tausta);
        self.teksti(x + 12, AANI_Y + 3, kaanna(teksti), MUSTA);
    }

    pub fn nappi(&mut self, x: i32, y: i32, avain: &str, tausta: u32, vari: u32) {
        self.nappitausta(x, y, napin_leveys(avain), NAPPI_K, tausta);
        self.teksti(x + 12, y + 3, kaanna(avain), vari);
    }

    /// Pieni viisteinen palkin pala tilastoissa.
    pub fn pala(&mut self, x: i32, y: i32, l: i32, k: i32, vari: u32) {
        self.k.laatikko(x, y, l, k, tummenna(vari, 0.35));
        self.k.laatikko(x, y, l - 3, k - 3, vari);
        self.k.laatikko(x, y, l - 3, 3, vaalenna(vari, 0.5));
    }

    /// Kupera painike: pyöristetyt kulmat, kiilto ylhäällä ja varjo alhaalla.
    pub fn nappitausta(&mut self, x: i32, y: i32, l: i32, k: i32, tausta: u32) {
        let p = self.k.pikseli;
        self.pyorea(x, y, l, k, ULKOVIIVA);
        let (sx, sy, sl, sk) = (x + p, y + p, l - 2 * p, k - 2 * p);
        let (yla, ala) = (vaalenna(tausta, 0.35), tummenna(tausta, 0.3));
        self.k.liukulaatikko(sx, sy, sl, sk, yla, ala);
        self.k.laatikko(sx, sy, sl, p, vaalenna(tausta, 0.7));
    }
}

fn on_vaalea(vari: u32) -> bool {
    let (r, g, b) = (vari >> 16 & 0xFF, vari >> 8 & 0xFF, vari & 0xFF);
    r * 3 + g * 6 + b > 1100
}

/// Ohjeet ja kehote eivät saa mennä päällekkäin eivätkä yli kuvan laidan millään kielellä.
#[cfg(test)]
pub fn alkuohje_mahtuu(osat: &[&str], rivit: &[&str], raja_y: i32) {
    super::kieli::kaikilla_kielilla(|| {
        let osarivit: usize = osat
            .iter()
            .map(|osa| rivita(kaanna(osa), ALKU_MERKKEJA).len())
            .sum();
        let korkeus = (osarivit + rivit.len()) as i32 * RIVIN_KORKEUS + RIVIN_KORKEUS / 2;
        assert!(OHJE_Y + korkeus <= raja_y, "{osat:?}");
        for rivi in rivit.iter().chain(&[ALOITUS]) {
            let teksti = kaanna(rivi);
            assert!(teksti.chars().count() <= ALKU_MERKKEJA, "{teksti}");
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    const KAIKKI_RIVIT: [bool; RUUDUN_KORKEUS] = [true; RUUDUN_KORKEUS];

    #[test]
    fn nimi_ikkunan_tekstit_mahtuvat_kummallakin_kielella() {
        crate::yhteinen::kieli::kaikilla_kielilla(|| {
            let tila = NIMI_IKKUNA_L - 36;
            assert!(kaanna(NIMI_OHJE).chars().count() as i32 * MERKKI <= tila);
            let otsikko = format!("{} 999999", kaanna(NIMI_OTSIKKO));
            assert!(otsikko.chars().count() as i32 * OTSIKKO.2 <= tila);
        });
    }

    #[test]
    pub fn rivitys_katkaisee_sanavalista_ja_kasittelee_aakkoset() {
        let teksti = "Vastustaja ampui B10: OSUMA! Lentotukialus upposi!";
        let rivit = rivita(teksti, RIVILLE_MERKKEJA);
        assert_eq!(
            rivit,
            ["Vastustaja ampui B10: OSUMA!", "Lentotukialus upposi!"]
        );
        assert_eq!(rivita("ääääää", 4), ["ääää", "ää"]);
        assert!(rivita("", 4).is_empty());
    }

    #[test]
    fn skaalaus_tayttaa_ruudun_ja_hiiri_osuu_samaan_pikseliin() {
        let mut kehys = Kehys::default();
        for (i, pikseli) in kehys.kuva.iter_mut().enumerate() {
            *pikseli = i as u32 + 1;
        }
        kehys.venyta();
        let (kuva, ruutu) = (&kehys.kuva, &kehys.ruutu);
        assert_eq!(ruutu[0], PALKKI);
        assert_eq!(ruutu[RUUDUN_LEVEYS * RUUDUN_KORKEUS - 1], PALKKI);
        for (x, y) in [(285, 0), (700, 300), (1634, 1079), (1000, 1079)] {
            let (kx, ky) = ruudulta_kuvaan(x, y);
            let pikseli = ruutu[y as usize * RUUDUN_LEVEYS + x as usize];
            assert_eq!(pikseli, kuva[ky as usize * LEVEYS + kx as usize], "{x},{y}");
        }
        assert!(ruudulta_kuvaan(0, 0).0 < 0);
        assert!(ruudulta_kuvaan(1700, 0).0 >= LEVEYS as i32);
    }

    #[test]
    fn muuttuneiden_rivien_venytys_vastaa_kokonaan_laskettua() {
        let mut kehys = Kehys::default();
        kehys.kuva.fill(0x123456);
        kehys.venyta();
        assert!(kehys.muuttuneet.iter().all(|&m| m));

        kehys.venyta();
        assert!(!kehys.muuttuneet.contains(&true));

        kehys.kuva[300 * LEVEYS + 500] = 0xABCDEF;
        kehys.venyta();
        let mut uusi = Kehys::default();
        uusi.kuva.copy_from_slice(&kehys.kuva);
        uusi.venyta();
        assert!(kehys.ruutu == uusi.ruutu);
        // Kuvan rivi 300 venyy ruudun riveille 422 ja 423 eikä muualle.
        assert!(kehys.muuttuneet[422] && kehys.muuttuneet[423]);
        assert_eq!(kehys.muuttuneet.iter().filter(|&&m| m).count(), 2);

        // Suora piirto sotkee ruudun, joten seuraava venytys laskee kaiken.
        kehys.ruutu.fill(0);
        kehys.piirretty_suoraan();
        assert!(kehys.muuttuneet.iter().all(|&m| m));
        kehys.venyta();
        assert!(kehys.muuttuneet.iter().all(|&m| m));
        assert!(kehys.ruutu == uusi.ruutu);
    }

    #[test]
    fn sovitus_mahtuu_yleisiin_resoluutioihin_ja_hiiri_osuu_kuvaan() {
        for (l, k) in [
            (800, 450),
            (1024, 768),
            (1120, 630),
            (1280, 1024),
            (1760, 990),
            (1920, 1009),
            (1920, 1080),
            (3680, 2070),
            (1, 1),
        ] {
            let sovitus = Sovitus::new(l, k);
            assert!(
                sovitus.x + sovitus.l <= l && sovitus.y + sovitus.k <= k,
                "{l}x{k}"
            );
            assert!(sovitus.l == l || sovitus.k == k, "{l}x{k}");
            let (x, y) = (
                (sovitus.x + sovitus.l - 1) as i32,
                (sovitus.y + sovitus.k - 1) as i32,
            );
            let (rx, ry) = sovitus.ruudulle(x, y).unwrap();
            assert!((rx as usize) < RUUDUN_LEVEYS && (ry as usize) < RUUDUN_KORKEUS);
            assert_eq!(
                sovitus.ruudulle(sovitus.x as i32, sovitus.y as i32),
                Some((0, 0))
            );
        }
        // 1280x1024: palkit ylhäällä ja alhaalla eivät osu kuvaan
        let sovitus = Sovitus::new(1280, 1024);
        assert_eq!((sovitus.y, sovitus.k), (152, 720));
        assert_eq!(sovitus.ruudulle(640, 100), None);
        assert_eq!(sovitus.ruudulle(640, 900), None);
        assert_eq!(sovitus.ruudulle(-1, 300), None);
        assert_eq!(sovitus.ruudulle(640, 512), Some((960, 540)));
    }

    #[test]
    fn sovitus_tayttaa_palkit_ja_sailyttaa_ohuen_viivan() {
        let mut ruutu = vec![0; RUUDUN_LEVEYS * RUUDUN_KORKEUS];
        // pystyviiva x = 960, leveys yksi pikseli
        for rivi in ruutu.chunks_mut(RUUDUN_LEVEYS) {
            rivi[960] = 0xFFFFFF;
        }
        let (l, k) = (960, 700);
        let sovitus = Sovitus::new(l, k);
        assert_eq!(sovitus.k, 540);
        let mut kuva = Ikkunakuva::default();
        kuva.sovita(&ruutu, &KAIKKI_RIVIT, l, k, sovitus);
        let ikkuna = &kuva.pikselit;
        assert_eq!(ikkuna[0], PALKKI);
        assert_eq!(ikkuna[l * k - 1], PALKKI);
        let rivi = &ikkuna[sovitus.y * l..(sovitus.y + 1) * l];
        assert_eq!(rivi[480], 0x7F7F7F);
        assert_eq!(rivi[479], 0);

        // sama koko on suora kopio
        let sama = Sovitus::new(RUUDUN_LEVEYS, RUUDUN_KORKEUS);
        kuva.sovita(&ruutu, &KAIKKI_RIVIT, RUUDUN_LEVEYS, RUUDUN_KORKEUS, sama);
        assert_eq!(kuva.pikselit, ruutu);
    }

    #[test]
    fn sovitus_kertoo_kuvan_muuttumisesta() {
        let mut ruutu = vec![0; RUUDUN_LEVEYS * RUUDUN_KORKEUS];
        let mut kuva = Ikkunakuva::default();
        let sovitus = Sovitus::new(960, 700);
        let kaikki = &KAIKKI_RIVIT;
        assert!(kuva.sovita(&ruutu, kaikki, 960, 700, sovitus));
        assert!(!kuva.sovita(&ruutu, kaikki, 960, 700, sovitus));
        ruutu[5] = 1;
        assert!(kuva.sovita(&ruutu, kaikki, 960, 700, sovitus));
        assert!(!kuva.sovita(&ruutu, kaikki, 960, 700, sovitus));
        // uusi ikkunan koko piirtää kaiken, vaikka kuva olisi sama
        let suurempi = Sovitus::new(1000, 700);
        assert!(kuva.sovita(&ruutu, kaikki, 1000, 700, suurempi));
    }

    #[test]
    fn sovitus_etsii_muutoksia_vain_ehdokasriveilta() {
        let mut ruutu = vec![0; RUUDUN_LEVEYS * RUUDUN_KORKEUS];
        let mut kuva = Ikkunakuva::default();
        let sovitus = Sovitus::new(960, 700);
        kuva.sovita(&ruutu, &KAIKKI_RIVIT, 960, 700, sovitus);
        ruutu[5] = 1;
        assert!(!kuva.sovita(&ruutu, &[false; RUUDUN_KORKEUS], 960, 700, sovitus));
        assert!(kuva.sovita(&ruutu, &KAIKKI_RIVIT, 960, 700, sovitus));
        // Uusi koko laskee kaiken ehdokkaista välittämättä.
        let suurempi = Sovitus::new(1000, 700);
        assert!(kuva.sovita(&ruutu, &[false; RUUDUN_KORKEUS], 1000, 700, suurempi));
        let mut uusi = Ikkunakuva::default();
        uusi.sovita(&ruutu, &KAIKKI_RIVIT, 1000, 700, suurempi);
        assert!(kuva.pikselit == uusi.pikselit);
    }

    #[test]
    fn muuttuneiden_rivien_sovitus_vastaa_kokonaan_laskettua() {
        let mut ruutu: Vec<u32> = (0..RUUDUN_LEVEYS * RUUDUN_KORKEUS)
            .map(|i| (i as u32).wrapping_mul(0x9E37_79B1) & 0xFFFFFF)
            .collect();
        let mut kuva = Ikkunakuva::default();
        // pienennys, palkit, suurennus ja takaisin ensimmäiseen kokoon
        for (l, k) in [(1760, 990), (960, 700), (2400, 1200), (1760, 990)] {
            let sovitus = Sovitus::new(l, k);
            kuva.sovita(&ruutu, &KAIKKI_RIVIT, l, k, sovitus);
            for (x, y) in [(0, 0), (1919, 1079), (700, 500), (701, 501)] {
                ruutu[y * RUUDUN_LEVEYS + x] ^= 0x00FF00;
                kuva.sovita(&ruutu, &KAIKKI_RIVIT, l, k, sovitus);
                let mut uusi = Ikkunakuva::default();
                uusi.sovita(&ruutu, &KAIKKI_RIVIT, l, k, sovitus);
                assert!(kuva.pikselit == uusi.pikselit, "{l}x{k} ({x}, {y})");
            }
            // Eri kohdat muuttuvat samassa ruudussa: pallo ylhäällä, maila alhaalla.
            ruutu[100 * RUUDUN_LEVEYS + 50] ^= 0x00FF00;
            ruutu[900 * RUUDUN_LEVEYS + 1800] ^= 0x00FF00;
            kuva.sovita(&ruutu, &KAIKKI_RIVIT, l, k, sovitus);
            let mut uusi = Ikkunakuva::default();
            uusi.sovita(&ruutu, &KAIKKI_RIVIT, l, k, sovitus);
            assert!(kuva.pikselit == uusi.pikselit, "{l}x{k}");
        }
    }
}
