use std::f32::consts::TAU;
use std::sync::LazyLock;

use super::fontti;
use super::kieli::kaanna;
use super::osoitin::{self, Tila};
use super::paivays::pvm;
use super::paletti::ramppi;
use super::rasteri::{Rasteri, kohina, liukuvari, taplassa, valisavy};
use super::tulokset::{PARHAITA, Tulos};
use super::vektori::{self, Kangas, kehapiste, sekoita_liuku};

/// Maisema, mitat konepikseleinä: `horisontti` on vedenrajan rivi ja `aurinko_x` sarake,
/// jonka kohdalla aurinko laskee ja johon meren ruudut katoavat. Kuva lasketaan ruudun
/// tarkkuudella (`tiheys` pikseliä konepikselin sivulla), joten vuorten, auringon ja meren
/// ruutujen reunat ovat sileät. Pilvet, tähdet ja heijastuksen raidat seuraavat yhä
/// konepikselin rivejä, mikä pitää vaakaraidoittaisen tunnelman.
pub fn maisemakuva(
    leveys: usize,
    korkeus: usize,
    horisontti: i32,
    aurinko_x: i32,
    tiheys: usize,
) -> Rasteri {
    const TAIVAS: [u32; 6] = [0x0A0630, 0x1C1058, 0x4A1C80, 0xA82C88, 0xF0587C, 0xFFA868];
    const MERI: [u32; 4] = [0x503C9C, 0x1C2A80, 0x0C1450, 0x060A28];
    let h = horisontti as f32;
    let auringon_sade = (horisontti * 3 / 8) as f32;
    let auringon_y = h - (horisontti * 3 / 8 / 3) as f32;
    let aurinko_x = aurinko_x as f32 + 0.5;
    let kaukainen =
        |x: f32| h * (0.17 + 0.05 * (x * 0.031).sin() + 0.035 * (x * 0.083 + 1.0).sin());
    let lahin =
        |x: f32| h * (0.06 + 0.05 * (x * 0.047 + 2.0).sin() + 0.025 * (x * 0.13).sin()).max(0.01);
    let alin = korkeus as f32;
    Rasteri::tarkka(leveys, korkeus, tiheys, |x, y| {
        let (rivi, sarake) = (y.floor() as i32, x.floor() as i32);
        if y >= h {
            let t = (y - h) / (alin - h);
            let etaisyys = 1.0 / (t + 0.06);
            let u = ((x - aurinko_x) * etaisyys * 0.09).floor() as i32;
            let v = (etaisyys * 2.4).floor() as i32;
            let mut vari = liukuvari(&MERI, t);
            if (u + v).rem_euclid(2) == 0 {
                vari = sekoita_liuku(vari, 0x5078E0, 0.16 * (1.0 - t));
            }
            // Auringon heijastus väpättää rivi riviltä.
            let rivin_t = (rivi - horisontti) as f32 / (alin - h);
            let leveys = auringon_sade * (0.75 - 0.45 * rivin_t);
            let vapina = (rivi as f32 * 1.7).sin() * leveys * 0.4;
            if ((x - aurinko_x) + vapina).abs() < leveys && (rivi % 2 == 0 || rivin_t < 0.2) {
                vari = sekoita_liuku(vari, 0xFFB070, 0.75 - 0.5 * rivin_t);
            }
            return vari;
        }
        let t = y / h;
        let mut vari = liukuvari(&TAIVAS, t);
        if t < 0.55 && kohina(sarake, rivi) > 0.992 && taplassa(x, y, 0.45) {
            vari = sekoita_liuku(vari, 0xFFFFFF, 0.5 + kohina(rivi, sarake) * 0.5);
        }
        // Ohuet pilvet, jotka valaistuvat horisonttia kohti.
        let pilvi = (x * 0.04 + rivi as f32 * 0.9).sin() + (x * 0.013 - rivi as f32).sin();
        if t > 0.35 && pilvi > 1.35 {
            vari = sekoita_liuku(vari, 0xFFC8A0, 0.35);
        }
        let (dx, dy) = (x - aurinko_x, y - auringon_y);
        if dx * dx + dy * dy < auringon_sade * auringon_sade {
            let osuus = (dy + auringon_sade) / (2.0 * auringon_sade);
            vari = liukuvari(&[0xFFF8B0, 0xFFD048, 0xFF6878], osuus);
            // Auringon alaosan raidat levenevät alaspäin.
            let raita = dy - auringon_sade / 4.0;
            if raita > 0.0 && raita.rem_euclid(5.0) < 1.0 + raita * 2.0 / auringon_sade {
                vari = liukuvari(&TAIVAS, t);
            }
        }
        let korkeus_nyt = h - y;
        if korkeus_nyt < kaukainen(x) {
            let harja = kaukainen(x) - korkeus_nyt;
            vari = if harja < 1.5 {
                0xB060B8
            } else {
                valisavy(0x5A2C90, 0x40207C, (harja / 12.0).min(1.0))
            };
        }
        if korkeus_nyt < lahin(x) {
            let harja = lahin(x) - korkeus_nyt;
            vari = if harja < 1.5 {
                0x5848A0
            } else {
                valisavy(0x241860, 0x180F48, (harja / 8.0).min(1.0))
            };
        }
        vari
    })
}

/// Pelinäkymien tausta: tummansininen timanttikuvio, joka vaalenee alaspäin.
static PELITAUSTA: LazyLock<Rasteri> = LazyLock::new(|| {
    const RUUTU: f32 = 16.0;
    Rasteri::tarkka(LEVEYS / 3, KORKEUS / 3, 3, |x, y| {
        let t = y / (KORKEUS / 3) as f32;
        let vari = liukuvari(&[TAUSTA_TUMMA, TAUSTA], t);
        // Timantin reuna: vaalea viiva ja sen ulkopuolella tumma varjo, konepikselin levyiset.
        let (dx, dy) = (
            (x.rem_euclid(RUUTU) - RUUTU / 2.0).abs(),
            (y.rem_euclid(RUUTU) - RUUTU / 2.0).abs(),
        );
        match (dx + dy - 0.5).floor() as i32 {
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
        3,
    )
});

/// Korttipöytien vihreä verka, joka tummenee reunoja kohti. Ruudun tarkkuudella, jotta
/// veran nukka on hienoa eikä konepikselin kokoisia läikkiä.
static VERKA: LazyLock<Rasteri> = LazyLock::new(|| {
    let (l, k) = (RUUDUN_LEVEYS / 4, RUUDUN_KORKEUS / 4);
    let (keski_x, keski_y) = (l as f32 * 0.36, k as f32 * 0.5);
    Rasteri::tarkka(l, k, 4, |x, y| {
        let (dx, dy) = ((x - keski_x) / l as f32, (y - keski_y) / k as f32);
        let t = (dx * dx * 2.2 + dy * dy * 2.4).sqrt() * 1.25;
        let vari = liukuvari(&[0x1C8050, 0x106440, 0x07402C, 0x042418], t);
        if kohina((x * 4.0) as i32, (y * 4.0) as i32) > 0.88 {
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
/// skaalataan keskelle ruutua; ympärille jää tumma palkki kuten television reunoille.
pub const RUUDUN_LEVEYS: usize = 1920;
pub const RUUDUN_KORKEUS: usize = 1080;
// Suhde 4 / 3: kolmen pikselin konepikselistä tulee tasan neljä ruudun pikseliä, joten
// jokainen konepikseli on yhtä leveä. Suhde 45 / 32 täyttäisi korkeuden, mutta silloin
// konepikselit olisivat vuorotellen neljän ja viiden pikselin levyisiä.
const SKAALA_OSOITTAJA: usize = 4;
const SKAALA_NIMITTAJA: usize = 3;
const SKAALATTU_LEVEYS: usize = LEVEYS * SKAALA_OSOITTAJA / SKAALA_NIMITTAJA;
const SKAALATTU_KORKEUS: usize = KORKEUS * SKAALA_OSOITTAJA / SKAALA_NIMITTAJA;
const SKAALATTU_X: usize = (RUUDUN_LEVEYS - SKAALATTU_LEVEYS) / 2;
const SKAALATTU_Y: usize = (RUUDUN_KORKEUS - SKAALATTU_KORKEUS) / 2;

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
/// Kehyksen sisempi, tummempi sävy.
const IKKUNA_UURRE: u32 = 0x7080C8;
const IKKUNA_VIISTE: u32 = 0x4870E0;
const IKKUNA_VARJO: u32 = 0x08104C;
const UPOTUS_VALO: u32 = 0x7C88D8;
const UPOTUS_KULMA: u32 = 0x3C3C7C;
const ULKOVIIVA: u32 = 0x0A0420;

pub const MERKKI: i32 = 3 * fontti::LEVEYS;
/// Tekstirivien väli. Merkki on alaosineen 24 pikseliä korkea, joten väliin jää rako:
/// p:n ja y:n alaosa ei osu seuraavan rivin isoihin kirjaimiin. Enempää ei mahdu, koska
/// paneelin kuusi riviä on sovitettava pelikenttien (alareuna 588) ja kuvan alareunan väliin.
pub const RIVIN_KORKEUS: i32 = 26;
/// Tekstirivien väli korttipöydillä, jotka piirtävät suoraan täydelle ruudulle: siellä
/// fontin pikseli on pienempi kuin klassisessa kuvassa, joten rako tarvitsee enemmän tilaa.
pub const RUUDUN_RIVI: i32 = 28;
pub const PALKIN_PALA: i32 = 18;
pub const PANEELI_Y: i32 = 591;
pub const PANEELI_K: i32 = 174;
pub const PANEELI_RIVI_Y: i32 = PANEELI_Y + 9;
pub const PANEELI_RIVEJA: usize = 6;
pub const TEKSTI_X: i32 = 24;
pub const RIVILLE_MERKKEJA: usize = 38;
pub const NAPPI_Y: i32 = PANEELI_RIVI_Y + 5 * RIVIN_KORKEUS - 6;
pub const NAPPI_K: i32 = 30;
/// Allekkaisten nappien yläreunojen väli.
pub const NAPPI_ASKEL: i32 = NAPPI_K + 12;
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
// Maisema on matalampi kuin ennen 2026-11-14, jotta ohjeiden väljemmät rivit mahtuvat.
const MAISEMA_ALA_Y: i32 = 318;
pub const OHJE_Y: i32 = MAISEMA_ALA_Y + 12;
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

pub const PARHAAT_RIVI: i32 = 26;
/// Listan otsikon ja ensimmäisen rivin väli.
pub const PARHAAT_OTSIKKO: i32 = 30;
/// Päivämäärä nimen alla `TULOS`-kokoisessa listassa.
pub const PVM: (i32, i32, i32) = (12, 14, 15);
const PVM_YLA: i32 = 6;
const PVM_ALA: i32 = 9;
// Alkuruudulla lista on maiseman vasemmassa laidassa, pyörivän mallin vieressä. Laatikon
// alareuna pysyy maiseman sisällä, joten päivämäärien vaatima korkeus kasvaa ylöspäin.
const ALKU_PARHAAT_L: i32 = 318;
const ALKU_PARHAAT_K: i32 =
    2 * 9 + PARHAAT_OTSIKKO + PARHAITA as i32 * parhaiden_askel(TULOS, Some(PVM)) - PVM_ALA;
const ALKU_PARHAAT: (i32, i32, i32, i32) = (
    36,
    MAISEMA_ALA_Y - 18 - ALKU_PARHAAT_K,
    ALKU_PARHAAT_L,
    ALKU_PARHAAT_K,
);

/// Listan rivien väli: päivämäärä vie nimen alta oman rivinsä.
pub const fn parhaiden_askel(koko: (i32, i32, i32), pvm: Option<(i32, i32, i32)>) -> i32 {
    match pvm {
        Some(pvm) => isojen_korkeus(koko) + PVM_YLA + isojen_korkeus(pvm) + PVM_ALA,
        None => PARHAAT_RIVI,
    }
}

/// Isojen kirjainten todellinen korkeus: `suurteksti` pyöristää koon fontin pikseleiksi.
const fn isojen_korkeus(koko: (i32, i32, i32)) -> i32 {
    let pikseli = (2 * koko.1 + 7) / 14;
    7 * if pikseli > 1 { pikseli } else { 1 }
}
const NIMI_OTSIKKO: &str = "yhteinen.nimi_otsikko";
pub const NIMI_OHJE: &str = "yhteinen.nimi_ohje";
pub const TAUKO: &str = "yhteinen.tauko";
pub const TAUON_OHJE: &str = "yhteinen.tauon_ohje";
pub const TAUON_PALUU: &str = "yhteinen.takaisin";
const NIMI_IKKUNA_L: i32 = 912;
const NIMI_IKKUNA_K: i32 = 120;

pub fn aanikytkimella(x: i32, y: i32) -> bool {
    (AANI_X..AANI_X + AANI_L).contains(&x) && (AANI_Y..AANI_Y + AANI_K).contains(&y)
}

pub fn napin_leveys(avain: &str) -> i32 {
    kaanna(avain).chars().count() as i32 * MERKKI + 24
}

/// Vaakarivin nappi pisteessä `(x, y)`. Napit ovat muotoa (nappi, tekstiavain, vasen reuna),
/// ja rivin yläreuna on `rivin_y`.
pub fn rivin_nappi<T: Copy>(napit: &[(T, &str, i32)], rivin_y: i32, x: i32, y: i32) -> Option<T> {
    if !(rivin_y..rivin_y + NAPPI_K).contains(&y) {
        return None;
    }
    napit
        .iter()
        .find(|&&(_, teksti, vasen)| (vasen..vasen + napin_leveys(teksti)).contains(&x))
        .map(|&(nappi, ..)| nappi)
}

/// Allekkaisten nappien sarake: ensimmäisen napin vasen yläkulma on `kulma`, ja napit
/// ovat `NAPPI_ASKEL`in välein. Napit ovat muotoa (nappi, tekstiavain).
pub fn sarakkeen_nappi<T: Copy>(
    napit: &[(T, &str)],
    kulma: (i32, i32),
    x: i32,
    y: i32,
) -> Option<T> {
    let (vasen, yla) = kulma;
    let i = (y - yla).div_euclid(NAPPI_ASKEL);
    let sisalla = (y - yla).rem_euclid(NAPPI_ASKEL) < NAPPI_K;
    if !sisalla || !(0..napit.len() as i32).contains(&i) {
        return None;
    }
    let (nappi, teksti) = napit[i as usize];
    (vasen..vasen + napin_leveys(teksti))
        .contains(&x)
        .then_some(nappi)
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
        let kuvan_alue = SKAALATTU_Y..SKAALATTU_Y + SKAALATTU_KORKEUS;
        for y in 0..RUUDUN_KORKEUS {
            let rivi = y * RUUDUN_LEVEYS;
            if !kuvan_alue.contains(&y) {
                // Ylä- ja alapalkki muuttuvat vain, kun ruudulle on piirretty suoraan.
                self.muuttuneet[y] = kaikki;
                if kaikki {
                    self.ruutu[rivi..rivi + RUUDUN_LEVEYS].fill(PALKKI);
                }
                continue;
            }
            let lahde_y = (y - SKAALATTU_Y) * SKAALA_NIMITTAJA / SKAALA_OSOITTAJA;
            let muuttui = kaikki || kuvan_rivit[lahde_y];
            self.muuttuneet[y] = muuttui;
            if !muuttui {
                continue;
            }
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

/// Ruudun pikseli klassisen kuvan koordinaateiksi. Sivupalkeilla x jää kuvan ulkopuolelle,
/// mutta y rajataan kuvaan, koska näkymät olettavat sen aina osuvan kuvan riville.
pub fn ruudulta_kuvaan(x: i32, y: i32) -> (i32, i32) {
    let (osoittaja, nimittaja) = (SKAALA_OSOITTAJA as i32, SKAALA_NIMITTAJA as i32);
    (
        ((x - SKAALATTU_X as i32) * nimittaja).div_euclid(osoittaja),
        ((y - SKAALATTU_Y as i32) * nimittaja)
            .div_euclid(osoittaja)
            .clamp(0, KORKEUS as i32 - 1),
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

    /// Kuten `ruudulle`, mutta ikkunan ulkopuolinen piste siirtyy kuvan lähimpään reunaan.
    pub fn ruudulle_rajattu(&self, x: i32, y: i32) -> (i32, i32) {
        let rajaa = |arvo: i32, alku: usize, pituus: usize| {
            (arvo.max(0) as usize).clamp(alku, alku + pituus - 1) as i32
        };
        let piste = (rajaa(x, self.x, self.l), rajaa(y, self.y, self.k));
        self.ruudulle(piste.0, piste.1)
            .expect("rajattu piste on kuvan sisällä")
    }
}

/// Painojen tarkkuus: kunkin kohdepikselin painot ovat yhteensä tämän verran.
const PAINO: u32 = 256;

/// Yhden akselin skaalaus: kunkin kohdepikselin lähdepikselit ja niiden painot. Kohde-
/// pikseli peittää lähteestä alueen, joka on pienennyksessä suhteen levyinen ja
/// suurennuksessa yhden pikselin levyinen, ja lähdepikseli painaa sen mukaan, kuinka
/// paljon aluetta se peittää. Näin jokainen pikseli saa saman kohtelun, eikä
/// pienennys hävitä ohutta viivaa eikä kahdenna joka kahdettatoista saraketta.
#[derive(Default)]
struct Suodin {
    /// Kohdepikselin ensimmäinen lähdepikseli ja painojen paikka `painot`-taulussa.
    alut: Vec<(usize, usize, usize)>,
    painot: Vec<u32>,
}

impl Suodin {
    fn new(kohde: usize, lahde: usize) -> Self {
        let suhde = lahde as f64 / kohde as f64;
        let puoli = suhde.max(1.0) / 2.0;
        let mut suodin = Self::default();
        for i in 0..kohde {
            let keski = (i as f64 + 0.5) * suhde;
            let (a, b) = ((keski - puoli).max(0.0), (keski + puoli).min(lahde as f64));
            let ensimmainen = (a.floor() as usize).min(lahde - 1);
            let viimeinen = ((b.ceil() as usize).max(ensimmainen + 1)).min(lahde);
            let osuudet: Vec<f64> = (ensimmainen..viimeinen)
                .map(|j| (b.min(j as f64 + 1.0) - a.max(j as f64)).max(0.0))
                .collect();
            let summa: f64 = osuudet.iter().sum::<f64>().max(f64::EPSILON);
            let paikka = suodin.painot.len();
            let mut jaettu = 0;
            for osuus in &osuudet {
                let paino = (osuus / summa * PAINO as f64).round() as u32;
                suodin.painot.push(paino);
                jaettu += paino;
            }
            // Pyöristys korjataan suurimpaan painoon, jotta tasainen väri pysyy samana.
            let painot = &mut suodin.painot[paikka..];
            let suurin = (0..painot.len()).max_by_key(|&k| painot[k]).unwrap_or(0);
            painot[suurin] = (painot[suurin] + PAINO).saturating_sub(jaettu);
            suodin.alut.push((ensimmainen, paikka, osuudet.len()));
        }
        suodin
    }

    /// Kohdepikselin `i` lähdepikselit `alku..loppu`.
    fn vali(&self, i: usize) -> (usize, usize) {
        let (alku, _, maara) = self.alut[i];
        (alku, alku + maara)
    }

    fn painot(&self, i: usize) -> (usize, &[u32]) {
        let (alku, paikka, maara) = self.alut[i];
        (alku, &self.painot[paikka..paikka + maara])
    }
}

/// Ikkunan kokoinen kuva. Muistaa edellisen ruudun, koska suurin osa riveistä säilyy
/// ruudusta toiseen ja skaalaus on piirron kallein vaihe.
#[derive(Default)]
pub struct Ikkunakuva {
    pub pikselit: Vec<u32>,
    edellinen: Vec<u32>,
    /// Kunkin ruudun rivin muuttunut pikseliväli.
    muuttuneet: Vec<Option<(usize, usize)>>,
    sarakkeet: Suodin,
    rivit: Suodin,
    /// Pystyyn suodatettu välirivi ennen vaakasuodatusta.
    valirivit: Vec<u32>,
    koko: Option<(usize, usize, Sovitus)>,
}

/// Kohdepikselit `s0..` riviltä `lahde` sarakkeiden painoin. Punainen ja sininen
/// lasketaan samassa luvussa: painot ovat yhteensä `PAINO` = 256, joten kanavat eivät
/// valu toisiinsa.
fn vaakasuodata(lahde: &[u32], sarakkeet: &Suodin, s0: usize, kohde: &mut [u32]) {
    for (x, pikseli) in (s0..).zip(kohde.iter_mut()) {
        let (vasen, painot) = sarakkeet.painot(x);
        *pikseli = match *painot {
            [_] => lahde[vasen],
            // Tavallisin tapaus, pieni pienennys: kaksi lähdepikseliä.
            [a, _] => sekoita_painoin(lahde[vasen], lahde[vasen + 1], a),
            _ => {
                let (mut rb, mut g) = (0, 0);
                for (&vari, &paino) in lahde[vasen..].iter().zip(painot) {
                    rb += (vari & 0xFF00FF) * paino;
                    g += (vari & 0x00FF00) * paino;
                }
                pakattu(rb, g)
            }
        };
    }
}

/// `a` painolla `paino` / 256 ja `b` lopulla painolla.
fn sekoita_painoin(a: u32, b: u32, paino: u32) -> u32 {
    let muu = PAINO - paino;
    pakattu(
        (a & 0xFF00FF) * paino + (b & 0xFF00FF) * muu,
        (a & 0x00FF00) * paino + (b & 0x00FF00) * muu,
    )
}

/// Painotetut summat takaisin väriksi.
fn pakattu(rb: u32, g: u32) -> u32 {
    ((rb >> 8) & 0xFF00FF) | ((g >> 8) & 0x00FF00)
}

impl Ikkunakuva {
    /// Skaalaa ruudun ikkunan kokoiseksi painotetulla keskiarvolla (`Suodin`); Windowsin
    /// oma venytys hävittäisi ohuet viivat. Vain `ehdokkaat`-riveiltä etsitään muutoksia,
    /// ja vain muuttuneet kohdat lasketaan. Palauttaa tiedon, muuttuiko kuva.
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
            self.sarakkeet = Suodin::new(sovitus.l, RUUDUN_LEVEYS);
            self.rivit = Suodin::new(sovitus.k, RUUDUN_KORKEUS);
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
        let sarakkeet = &self.sarakkeet;
        for y in 0..sovitus.k {
            let rivit = self.rivit.vali(y);
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
                let kohdat = &sarakkeet.alut;
                (
                    kohdat.partition_point(|&(a, _, m)| a + m <= alku),
                    kohdat.partition_point(|&(a, _, _)| a < loppu),
                )
            };
            let (ylin, rivipainot) = self.rivit.painot(y);
            let alku = (sovitus.y + y) * ikkuna_l + sovitus.x;
            let kohde = &mut ikkuna[alku + s0..alku + s1];
            if rivipainot.len() == 1 {
                vaakasuodata(&ruutu[ylin * RUUDUN_LEVEYS..], sarakkeet, s0, kohde);
                continue;
            }
            // Erotettava suodatus: ensin lähderivit pystyyn yhdeksi väliriviksi, sitten se
            // vaakaan. Pystyvaihe käsittelee vain tarvittavat lähdesarakkeet kerran.
            let c0 = sarakkeet.vali(s0).0;
            let c1 = sarakkeet.vali(s1 - 1).1;
            self.valirivit.resize(RUUDUN_LEVEYS, 0);
            let valirivi = &mut self.valirivit[c0..c1];
            let rivi =
                |i: usize| &ruutu[(ylin + i) * RUUDUN_LEVEYS + c0..(ylin + i) * RUUDUN_LEVEYS + c1];
            if let [paino, _] = *rivipainot {
                // Kahden rivin sekoitus suorana silmukkana, jonka kääntäjä voi vektoroida.
                for ((kohta, &a), &b) in valirivi.iter_mut().zip(rivi(0)).zip(rivi(1)) {
                    *kohta = sekoita_painoin(a, b, paino);
                }
            } else {
                for (c, kohta) in valirivi.iter_mut().enumerate() {
                    let (mut rb, mut g) = (0, 0);
                    for (i, &paino) in rivipainot.iter().enumerate() {
                        let vari = rivi(i)[c];
                        rb += (vari & 0xFF00FF) * paino;
                        g += (vari & 0x00FF00) * paino;
                    }
                    *kohta = pakattu(rb, g);
                }
            }
            vaakasuodata(&self.valirivit, sarakkeet, s0, kohde);
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

#[derive(Clone, Copy)]
struct Tekstityyli {
    aariviiva: bool,
    /// Otsikko saa läpikuultavan pudotusvarjon.
    otsikko: bool,
}

/// Otsikon pudotusvarjon tummuus, 0..=256.
const OTSIKON_VARJO: u32 = 150;

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
        let tyyli = Tekstityyli {
            aariviiva: false,
            otsikko: false,
        };
        let koko = (kerroin, kerroin, kerroin * fontti::LEVEYS);
        self.merkit(x, y, teksti, koko, vari, tyyli);
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
        let (pl, pk) = (pikseli(koko.0), pikseli(koko.1));
        let tyyli = Tekstityyli {
            aariviiva: paksu,
            otsikko: true,
        };
        self.merkit(x, y, teksti, (pl, pk, koko.2), vari, tyyli);
    }

    /// Vaaleat merkit saavat varjon ja sävyportaan liukuvärin ylhäältä alas; tummat ovat
    /// vaalealla pohjalla, joten ne piirtyvät tasaisina.
    fn merkit(
        &mut self,
        x: i32,
        y: i32,
        teksti: &str,
        koko: (i32, i32, i32),
        vari: u32,
        tyyli: Tekstityyli,
    ) {
        let (pl, pk, askel) = koko;
        let vaalea = on_vaalea(vari);
        let porras = ramppi(vari);
        let rivivarit: [u32; 8] = std::array::from_fn(|rivi| {
            if !vaalea {
                return vari;
            }
            match rivi {
                0 | 1 => porras[4],
                2 => porras[3],
                3..=5 => vari,
                _ => porras[1],
            }
        });
        // Merkit ovat pehmennettyjä peittokuvia: porrasreunat oikenevat ja reunat
        // sekoittuvat taustaan, joten teksti ei näytä rosoiselta missään koossa.
        let peitot: Vec<_> = teksti
            .chars()
            .map(|merkki| fontti::peitto(merkki, pl, pk))
            .collect();
        let kohdat = |dx: i32, dy: i32| {
            peitot
                .iter()
                .enumerate()
                .map(move |(i, peitto)| ((x + i as i32 * askel + dx, y + dy), peitto))
        };
        if vaalea && tyyli.otsikko {
            // Läpikuultava pudotusvarjo erottaa otsikon taustasta sitä peittämättä.
            let siirto = if tyyli.aariviiva { 2 } else { 1 };
            for (kohta, peitto) in kohdat(siirto * pl, siirto * pk) {
                self.k.peiton_varjo(kohta, peitto, OTSIKON_VARJO);
            }
        }
        let varjot: &'static [(i32, i32)] = match (vaalea, tyyli.aariviiva) {
            (false, _) => &[],
            // Rivit ovat tiheässä, joten leipätekstin varjo menee vain sivulle.
            (true, false) => &[(1, 0)],
            (true, true) => &[(-1, 0), (1, 0), (0, -1), (0, 1), (1, 1)],
        };
        // Ääriviiva ja merkki on koostettu valmiiksi, joten kukin merkki piirtyy kerran.
        for (i, merkki) in teksti.chars().enumerate() {
            let kooste = fontti::koostettu(merkki, (pl, pk), rivivarit, ULKOVIIVA, varjot);
            let kohta = (x + i as i32 * askel - pl, y - pk);
            self.k.peitto(kohta, &kooste);
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
        let t = kuva.tiheys();
        for rivi in reuna * t..kuva.korkeus() * t {
            let konerivi = rivi / t;
            // Siirto on konepikseleinä: hienompi askel muuttaisi useampia rivejä joka
            // askeleella, ja jokainen muuttunut rivi maksaa venytyksessä ja sovituksessa.
            let siirto = if konerivi < horisontti {
                0
            } else {
                let syvyys = (konerivi - horisontti) as f32 / (kuva.korkeus() - horisontti) as f32;
                let aalto = (vaihe + konerivi as f32 * AALLON_TIHEYS).sin();
                (aalto * AALLON_LAAJUUS * (0.3 + syvyys)).round() as i32 * t as i32
            };
            let alku = (reuna as i32 * t as i32 + siirto) as usize;
            let lahde = &kuva.rivi(rivi)[alku..alku + leveys * t];
            self.k.tarkka_rivi(MAISEMA_REUNA, rivi as i32, lahde);
        }
    }

    /// SNES-roolipelien tapainen ikkuna: pyöristetyt kulmat, vaalea kehys ja sininen liuku.
    pub fn ikkuna(&mut self, x: i32, y: i32, l: i32, k: i32) {
        let p = self.k.pikseli;
        // Kaksisävyinen kehys: vaalea ulkoreuna ja sen sisällä tummempi uurre.
        self.pyorea(x, y, l, k, ULKOVIIVA);
        self.pyorea(x + p, y + p, l - 2 * p, k - 2 * p, IKKUNA_REUNA);
        self.pyorea(x + 2 * p, y + 2 * p, l - 4 * p, k - 4 * p, IKKUNA_UURRE);
        let (sx, sy, sl, sk) = (x + 3 * p, y + 3 * p, l - 6 * p, k - 6 * p);
        self.k.liukulaatikko(sx, sy, sl, sk, IKKUNA_YLA, IKKUNA_ALA);
        // Sisäviiste: valo osuu yläreunaan, ja alareuna jää varjoon.
        self.k.laatikko(sx, sy, sl, p, IKKUNA_VIISTE);
        self.k.laatikko(sx, sy + sk - p, sl, p, IKKUNA_VARJO);
        // Kimallus kehyksen vasemmassa yläkulmassa, josta valo tulee.
        self.k.laatikko(x + 2 * p, y + p, 2 * p, p, VALKOINEN);
        self.k.laatikko(x + p, y + 2 * p, p, p, VALKOINEN);
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
        self.k.laatikko(ulko_x, y + k, ulko_l, p, UPOTUS_VALO);
        self.k.laatikko(x + l, ulko_y, p, ulko_k, UPOTUS_VALO);
        // Varjon ja valon kohtaamiskulmat saavat välisävyn, jolloin reuna kiertää pehmeästi.
        self.k.laatikko(x + l, ulko_y, p, p, UPOTUS_KULMA);
        self.k.laatikko(ulko_x, y + k, p, p, UPOTUS_KULMA);
    }

    /// Valintakehys kohteen `(x, y, l, k)` ympärillä: tumma ääriviiva, värillinen rengas
    /// ja kulmahakaset. Vilkkumisen tahdissa rengas kimaltaa ja hakaset sykkivät
    /// ulospäin, kuten SNES-valikoiden kursori, joten kehys ei koskaan katoa näkyvistä.
    pub fn valintakehys(&mut self, x: i32, y: i32, l: i32, k: i32, vari: u32) {
        let p = self.k.pikseli;
        let porras = ramppi(vari);
        let vaihe = self.vilkku();
        self.rengas(x - 2 * p, y - 2 * p, l + 4 * p, k + 4 * p, ULKOVIIVA);
        let renkaan_vari = if vaihe { vari } else { porras[3] };
        self.rengas(x - p, y - p, l + 2 * p, k + 2 * p, renkaan_vari);
        let ulos = if vaihe { 2 * p } else { 3 * p };
        let (vasen, yla) = (x - ulos - p, y - ulos - p);
        let (oikea, ala) = (x + l + ulos, y + k + ulos);
        let pituus = 4 * p;
        for (kx, ky, sx, sy) in [
            (vasen, yla, 1, 1),
            (oikea, yla, -1, 1),
            (vasen, ala, 1, -1),
            (oikea, ala, -1, -1),
        ] {
            // Vaakavarsi ja pystyvarsi kulmasta kohti kohteen keskustaa, varjo alla.
            let vaaka_x = if sx > 0 { kx } else { kx - pituus + p };
            let pysty_y = if sy > 0 { ky } else { ky - pituus + p };
            for (siirto, vari) in [(p, ULKOVIIVA), (0, porras[4])] {
                self.k
                    .laatikko(vaaka_x + siirto, ky + siirto, pituus, p, vari);
                self.k
                    .laatikko(kx + siirto, pysty_y + siirto, p, pituus, vari);
            }
        }
    }

    /// Konepikselin paksuinen kehys, jonka kulmat on pyöristetty pois.
    fn rengas(&mut self, x: i32, y: i32, l: i32, k: i32, vari: u32) {
        let p = self.k.pikseli;
        self.k.laatikko(x + p, y, l - 2 * p, p, vari);
        self.k.laatikko(x + p, y + k - p, l - 2 * p, p, vari);
        self.k.laatikko(x, y + p, p, k - 2 * p, vari);
        self.k.laatikko(x + l - p, y + p, p, k - 2 * p, vari);
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
        self.kupera(6, 6, l, 30, 0x384890);
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
    /// `pvm` on päivämäärän koko nimen alla; ahtaissa sivupalkeissa se jätetään pois.
    #[allow(clippy::too_many_arguments)]
    pub fn parhaat_lista(
        &mut self,
        x: i32,
        y: i32,
        leveys: i32,
        koko: (i32, i32, i32),
        pvm: Option<(i32, i32, i32)>,
        otsikko: &str,
        parhaat: &[Tulos],
    ) {
        self.teksti(x, y, otsikko, KELTAINEN);
        self.parhaat_rivit(x, y + PARHAAT_OTSIKKO, leveys, koko, pvm, parhaat);
    }

    /// Listan rivit ilman otsikkoa, ensimmäinen kohdassa `y`.
    pub fn parhaat_rivit(
        &mut self,
        x: i32,
        y: i32,
        leveys: i32,
        koko: (i32, i32, i32),
        pvm_koko: Option<(i32, i32, i32)>,
        parhaat: &[Tulos],
    ) {
        let askel = parhaiden_askel(koko, pvm_koko);
        for i in 0..PARHAITA {
            let rivi_y = y + i as i32 * askel;
            let tulos = parhaat.get(i);
            let nimi = tulos.map_or("-", |t| t.nimi.as_str());
            let vasen = format!("{} {nimi}", i + 1);
            self.suurteksti(x, rivi_y, &vasen, koko, VALKOINEN, false);
            let Some(tulos) = tulos else {
                continue;
            };
            let pisteet = tulos.pisteet.to_string();
            let pituus = pisteet.chars().count() as i32 * koko.2;
            let oikea = x + leveys - pituus;
            self.suurteksti(oikea, rivi_y, &pisteet, koko, KELTAINEN, false);
            // Vanhoissa tuloksissa ei ole aikaa, joten niiden kohta jää tyhjäksi.
            if let (Some(pvm_koko), Some(aika)) = (pvm_koko, tulos.aika) {
                let pvm_y = rivi_y + isojen_korkeus(koko) + PVM_YLA;
                self.suurteksti(
                    x + 2 * koko.2,
                    pvm_y,
                    &pvm(aika),
                    pvm_koko,
                    LAVENTELI,
                    false,
                );
            }
        }
    }

    pub fn parhaat_alkuruudulla(&mut self, otsikko: &str, parhaat: &[Tulos]) {
        let (x, y, l, k) = ALKU_PARHAAT;
        self.ikkuna(x, y, l, k);
        self.parhaat_lista(x + 12, y + 9, l - 24, TULOS, Some(PVM), otsikko, parhaat);
    }

    /// Paneelin sisältö tauolla: otsikko, jatkamisen ohje ja paluu valikkoon. Esc pysäyttää
    /// pelin ensin, joten vasta toinen painallus palaa valikkoon.
    pub fn tauko(&mut self) {
        let alarivi = PANEELI_RIVI_Y + (PANEELI_RIVEJA as i32 - 1) * RIVIN_KORKEUS;
        let otsikko_y = alarivi - 2 * RIVIN_KORKEUS - 12;
        self.suurteksti(TEKSTI_X, otsikko_y, kaanna(TAUKO), TULOS, KELTAINEN, true);
        self.teksti(TEKSTI_X, alarivi - RIVIN_KORKEUS, kaanna(TAUON_OHJE), HUTI);
        self.teksti(TEKSTI_X, alarivi, kaanna(TAUON_PALUU), HUTI);
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
        let porras = ramppi(vari);
        self.k.laatikko(x, y, l, k, porras[0]);
        self.k.laatikko(x, y, l - 3, k - 3, vari);
        self.k.laatikko(x, y, l - 3, 3, porras[4]);
    }

    /// Painike, joka korostuu hiiren alla ja painuu painettaessa.
    pub fn nappitausta(&mut self, x: i32, y: i32, l: i32, k: i32, tausta: u32) {
        let tila = osoitin::tila(x, y, l, k);
        self.pinta(x, y, l, k, tausta, tila);
    }

    /// Kupera pinta ilman hiiren korostusta, kuten laatta tai otsikkopalkki.
    pub fn kupera(&mut self, x: i32, y: i32, l: i32, k: i32, tausta: u32) {
        self.pinta(x, y, l, k, tausta, Tila::Lepo);
    }

    /// Kupera pinta: valo osuu yläreunaan, ja kyljet ja pohja ovat tummaa ääriviivaa.
    /// Yläreunassa ei ole ääriviivaa, jottei tekstin ylin rivi, kuten ääkkösten pisteet,
    /// sulaudu siihen. Painettuna yläreuna jää varjoon ja pinta tummuu.
    fn pinta(&mut self, x: i32, y: i32, l: i32, k: i32, tausta: u32, tila: Tila) {
        let p = self.k.pikseli;
        let porras = ramppi(tausta);
        let (reuna, yla, ala) = match tila {
            Tila::Lepo => (porras[4], porras[3], porras[1]),
            Tila::Alla => (VALKOINEN, porras[4], tausta),
            Tila::Painettu => (porras[0], porras[1], porras[0]),
        };
        self.k.laatikko(x, y + p, p, k - 2 * p, ULKOVIIVA);
        self.k.laatikko(x + l - p, y + p, p, k - 2 * p, ULKOVIIVA);
        self.k.laatikko(x + p, y + k - p, l - 2 * p, p, ULKOVIIVA);
        self.k.laatikko(x + p, y, l - 2 * p, p, reuna);
        self.k
            .liukulaatikko(x + p, y + p, l - 2 * p, k - 2 * p, yla, ala);
    }
}

fn on_vaalea(vari: u32) -> bool {
    let (r, g, b) = (vari >> 16 & 0xFF, vari >> 8 & 0xFF, vari & 0xFF);
    r * 3 + g * 6 + b > 1100
}

#[cfg(test)]
mod tests;
#[cfg(test)]
pub use tests::{aanikytkimen_keskipiste, alkuohje_mahtuu};
