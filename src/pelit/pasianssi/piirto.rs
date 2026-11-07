//! Pasianssin näkymät. Alkuruutu käyttää muiden pelien kanssa samaa klassista kuvaa,
//! pelilauta piirretään suoraan täysikokoiselle ruudulle.

use klassikot8000::haasteet::HAASTEITA;
use klassikot8000::pasianssi::{
    KUNINGAS, Kohde, Kortti, Lahde, Lauta, MAITA, Maa, SARAKKEITA, Sarake, Vaikeus, Variantti,
};

use super::peli::{Nappi, Pasianssi, Vaihe};
use crate::yhteinen::kieli::{kaanna, muotoile};
use crate::yhteinen::kortit::{KORTTI_K, KORTTI_L};
use crate::yhteinen::piirto::*;
use crate::yhteinen::vektori::{Kangas, Tahko, kuutio};

const SARAKE_X: i32 = 48;
const SARAKE_ASKEL: i32 = 180;
const YLARIVI_Y: i32 = 54;
const TAULU_Y: i32 = YLARIVI_Y + KORTTI_K + 24;
const PIILO_ASKEL: i32 = 15;
const NAKYVA_ASKEL: i32 = 36;
const VIUHKA_ASKEL: i32 = 36;
const PERUSTA_ALKAA: usize = 3;
const JATE_SARAKE: i32 = 1;

pub(super) const SIVU_X: i32 = SARAKE_X + SARAKKEITA as i32 * SARAKE_ASKEL + 6;
pub(super) const SIVU_L: i32 = RUUDUN_LEVEYS as i32 - 24 - SIVU_X;
pub(super) const SIVU_TEKSTI_X: i32 = SIVU_X + 18;
pub(super) const SIVU_MERKKEJA: usize = 22;
const TILASTO_ASKEL: i32 = 66;
const VIESTI_Y: i32 = YLARIVI_Y + 5 * TILASTO_ASKEL + 6;
const VIESTI_K: i32 = 4 * RIVIN_KORKEUS + 18;
const SIVU_OHJE_Y: i32 = VIESTI_Y + VIESTI_K + 18;
pub(super) const NAPIT_Y: i32 = SIVU_OHJE_Y + 7 * RIVIN_KORKEUS + 18;
const NAPPI_ASKEL: i32 = NAPPI_K + 12;

const SIVU_OHJEET: [&str; 6] = [
    "pasianssi.sivu_hiiri",
    "pasianssi.sivu_tupla",
    "pasianssi.sivu_nuolet",
    "pasianssi.sivu_nosta",
    "pasianssi.sivu_aanet",
    "pasianssi.sivu_valikko",
];

const NAPIT: [(Nappi, &str); 5] = [
    (Nappi::Uusi, "pasianssi.nappi_uusi"),
    (Nappi::Alusta, "pasianssi.nappi_alusta"),
    (Nappi::Peruuta, "pasianssi.nappi_peruuta"),
    (Nappi::Keraa, "pasianssi.nappi_keraa"),
    (Nappi::Valikko, "yhteinen.valikkoon"),
];

// Alkuruudun tasovalinta samoilla mitoilla kuin laivanupotuksessa, lajivalinta sen yläpuolella.
const TASO_Y: i32 = ALOITUS_Y - NAPPI_K - 6;
const VARIANTTI_Y: i32 = TASO_Y - NAPPI_K - 6;
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

pub(super) const VARIANTIT: [(Variantti, &str, i32); 3] = [
    (Variantti::Klondike, "pasianssi.laji_klondike", TASO_X),
    (
        Variantti::Spider,
        "pasianssi.laji_spider",
        TASO_X + 8 * MERKKI + 24 + TASO_VALI,
    ),
    (
        Variantti::Pyramidi,
        "pasianssi.laji_pyramidi",
        TASO_X + 14 * MERKKI + 48 + 2 * TASO_VALI,
    ),
];

// Alkuruudun kehote on muita lyhyempi, jotta haastenappi mahtuu samalle riville.
const PASIANSSI_ALOITUS: &str = "pasianssi.aloitus";
const HAASTENAPPI: &str = "pasianssi.nappi_haasteet_alku";
const NAPPI_HAASTEET: &str = "pasianssi.nappi_haasteet";

const VOITTOTEKSTI: &str = "yhteinen.voitit";
const TAPPIOTEKSTI: &str = "pasianssi.tappio";
const VOITTO_KORTTI_VALI_MS: u32 = 110;
const VOITTO_JALKIA: u32 = 6;
const VOITTO_JALKI_MS: u32 = 45;

const ALKU_OSAT: [&str; 2] = ["pasianssi.ohje_1", "pasianssi.ohje_2"];
const ALKU_RIVIT: [&str; 5] = [
    "pasianssi.nappain_valinta",
    "pasianssi.nappain_hiiri",
    "pasianssi.nappain_nuolet",
    "pasianssi.nappain_kirjaimet",
    "pasianssi.nappain_muut",
];
const SPIDER_OSAT: [&str; 2] = ["spider.ohje_1", "spider.ohje_2"];
const SPIDER_RIVIT: [&str; 5] = [
    "pasianssi.nappain_valinta",
    "spider.nappain_hiiri",
    "pasianssi.nappain_nuolet",
    "spider.nappain_kirjaimet",
    "pasianssi.nappain_muut",
];
const PYRAMIDI_OSAT: [&str; 2] = ["pyramidi.ohje_1", "pyramidi.ohje_2"];
const PYRAMIDI_RIVIT: [&str; 5] = [
    "pasianssi.nappain_valinta",
    "pyramidi.nappain_hiiri",
    "pasianssi.nappain_nuolet",
    "pyramidi.nappain_kirjaimet",
    "pasianssi.nappain_muut",
];

/// Kuvaus kertoo, mitä valittu taso tarkoittaa juuri tässä lajissa.
fn tason_kuvaus(variantti: Variantti, vaikeus: Vaikeus) -> &'static str {
    match (variantti, vaikeus) {
        (Variantti::Klondike, Vaikeus::Helppo) => "pasianssi.kuvaus_helppo",
        (Variantti::Klondike, Vaikeus::Normaali) => "pasianssi.kuvaus_normaali",
        (Variantti::Klondike, Vaikeus::Vaikea) => "pasianssi.kuvaus_vaikea",
        (Variantti::Spider, Vaikeus::Helppo) => "spider.kuvaus_helppo",
        (Variantti::Spider, Vaikeus::Normaali) => "spider.kuvaus_normaali",
        (Variantti::Spider, Vaikeus::Vaikea) => "spider.kuvaus_vaikea",
        (Variantti::Pyramidi, Vaikeus::Helppo) => "pyramidi.kuvaus_helppo",
        (Variantti::Pyramidi, Vaikeus::Normaali) => "pyramidi.kuvaus_normaali",
        (Variantti::Pyramidi, Vaikeus::Vaikea) => "pyramidi.kuvaus_vaikea",
    }
}

pub fn lajin_nimi(variantti: Variantti) -> &'static str {
    let (_, avain, _) = VARIANTIT[variantti as usize];
    kaanna(avain)
}

pub(super) fn vaikeuden_nimi(vaikeus: Vaikeus) -> &'static str {
    kaanna(match vaikeus {
        Vaikeus::Helppo => "yhteinen.helppo",
        Vaikeus::Normaali => "yhteinen.normaali",
        Vaikeus::Vaikea => "yhteinen.vaikea",
    })
}

/// Laudan paikka hiiren tai osoittimen alla. `kortti` on `None` tyhjässä sarakkeessa.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kohta {
    Pakka,
    Jate,
    Perusta(usize),
    Sarake {
        sarake: usize,
        kortti: Option<usize>,
    },
}

fn sarakkeen_x(sarake: usize) -> i32 {
    SARAKE_X + sarake as i32 * SARAKE_ASKEL
}

/// Kuinka monta jätepinon korttia näkyy viuhkana: niin monta kuin kerralla nostetaan.
fn viuhka(lauta: &Lauta) -> usize {
    lauta.vaikeus.nosto().min(lauta.tila.jate.len())
}

fn jatteen_x(lauta: &Lauta) -> i32 {
    let viuhka = viuhka(lauta).max(1) as i32;
    sarakkeen_x(JATE_SARAKE as usize) + (viuhka - 1) * VIUHKA_ASKEL
}

fn kortin_y(sarake: &Sarake, i: usize) -> i32 {
    let piilossa = i.min(sarake.piilossa) as i32;
    TAULU_Y + piilossa * PIILO_ASKEL + (i as i32 - piilossa) * NAKYVA_ASKEL
}

/// Jonon alimman kortin vasen yläkulma.
pub fn kortin_paikka(lauta: &Lauta, lahde: Lahde) -> (i32, i32) {
    match lahde {
        Lahde::Jate => (jatteen_x(lauta), YLARIVI_Y),
        Lahde::Perusta(perusta) => (sarakkeen_x(perusta + PERUSTA_ALKAA), YLARIVI_Y),
        Lahde::Sarake { sarake, kortti } => (
            sarakkeen_x(sarake),
            kortin_y(&lauta.tila.sarakkeet[sarake], kortti),
        ),
    }
}

fn sarake_pisteesta(x: i32) -> Option<usize> {
    let sarake = (x - SARAKE_X).div_euclid(SARAKE_ASKEL);
    let sisalla = (x - SARAKE_X).rem_euclid(SARAKE_ASKEL) < KORTTI_L;
    (sisalla && (0..SARAKKEITA as i32).contains(&sarake)).then_some(sarake as usize)
}

pub fn kohta_pisteesta(lauta: &Lauta, x: i32, y: i32) -> Option<Kohta> {
    if (YLARIVI_Y..YLARIVI_Y + KORTTI_K).contains(&y) {
        let jate_x = jatteen_x(lauta);
        if !lauta.tila.jate.is_empty() && (jate_x..jate_x + KORTTI_L).contains(&x) {
            return Some(Kohta::Jate);
        }
        return match sarake_pisteesta(x)? {
            0 => Some(Kohta::Pakka),
            sarake if sarake >= PERUSTA_ALKAA => Some(Kohta::Perusta(sarake - PERUSTA_ALKAA)),
            _ => None,
        };
    }
    let sarake = sarake_pisteesta(x)?;
    let tiedot = &lauta.tila.sarakkeet[sarake];
    let kortti = (0..tiedot.kortit.len()).rev().find(|&i| {
        let kortti_y = kortin_y(tiedot, i);
        (kortti_y..kortti_y + KORTTI_K).contains(&y)
    });
    let tyhja_kohta = tiedot.kortit.is_empty() && (TAULU_Y..TAULU_Y + KORTTI_K).contains(&y);
    (kortti.is_some() || tyhja_kohta).then_some(Kohta::Sarake { sarake, kortti })
}

/// Pudotuskohde on anteliaampi kuin poiminta: koko sarakkeen kaista kelpaa.
pub fn kohde_pisteesta(x: i32, y: i32) -> Option<Kohde> {
    let sarake = sarake_pisteesta(x)?;
    if (YLARIVI_Y..YLARIVI_Y + KORTTI_K).contains(&y) {
        return sarake
            .checked_sub(PERUSTA_ALKAA)
            .filter(|&perusta| perusta < MAITA)
            .map(Kohde::Perusta);
    }
    (y >= TAULU_Y).then_some(Kohde::Sarake(sarake))
}

pub fn vedetyn_keskipiste(x: i32, y: i32) -> (i32, i32) {
    (x + KORTTI_L / 2, y + NAKYVA_ASKEL / 2)
}

pub fn nappi_pisteesta(x: i32, y: i32) -> Option<Nappi> {
    nappi_listasta(&NAPIT, x, y)
}

/// Sivupaneelin nappi. Kullakin lajilla on oma listansa samoilla paikoilla.
pub fn nappi_listasta(napit: &[(Nappi, &str)], x: i32, y: i32) -> Option<Nappi> {
    let i = (y - NAPIT_Y).div_euclid(NAPPI_ASKEL);
    let sisalla = (y - NAPIT_Y).rem_euclid(NAPPI_ASKEL) < NAPPI_K;
    if !sisalla || !(0..napit.len() as i32).contains(&i) {
        return None;
    }
    let (nappi, teksti) = napit[i as usize];
    (SIVU_TEKSTI_X..SIVU_TEKSTI_X + napin_leveys(teksti))
        .contains(&x)
        .then_some(nappi)
}

pub fn vaikeus_pisteesta(x: i32, y: i32) -> Option<Vaikeus> {
    TASOT
        .iter()
        .find(|&&(_, teksti, nx)| {
            (nx..nx + napin_leveys(teksti)).contains(&x) && (TASO_Y..TASO_Y + NAPPI_K).contains(&y)
        })
        .map(|&(vaikeus, ..)| vaikeus)
}

pub fn variantti_pisteesta(x: i32, y: i32) -> Option<Variantti> {
    VARIANTIT
        .iter()
        .find(|&&(_, teksti, nx)| {
            (nx..nx + napin_leveys(teksti)).contains(&x)
                && (VARIANTTI_Y..VARIANTTI_Y + NAPPI_K).contains(&y)
        })
        .map(|&(variantti, ..)| variantti)
}

fn haastenapin_x() -> i32 {
    // Jää alapaneelin sisään.
    LEVEYS as i32 - 36 - napin_leveys(HAASTENAPPI)
}

pub fn haastenappi_pisteessa(x: i32, y: i32) -> bool {
    let vasen = haastenapin_x();
    (vasen..vasen + napin_leveys(HAASTENAPPI)).contains(&x)
        && (ALOITUS_Y..ALOITUS_Y + NAPPI_K).contains(&y)
}

/// Haasteessa Uusi-nappi vie haastelistaan. Teksti ei ole pidempi kuin tavallinen, joten osuma-alue kelpaa.
pub fn uuden_napin_teksti<'a>(peli: &Pasianssi, nappi: Nappi, teksti: &'a str) -> &'a str {
    if nappi == Nappi::Uusi && peli.haaste.is_some() {
        NAPPI_HAASTEET
    } else {
        teksti
    }
}

/// Pelilaudan näkymässä äänikytkin on täyden ruudun oikeassa laidassa.
pub fn aanikytkimella(vaihe: Vaihe, x: i32, y: i32) -> bool {
    let siirto = if vaihe == Vaihe::Alku {
        0
    } else {
        (RUUDUN_LEVEYS - LEVEYS) as i32
    };
    crate::yhteinen::piirto::aanikytkimella(x - siirto, y)
}

/// Alkuruudun malli: kolme pystyssä olevaa ohutta korttia. Mitat kuin laivalla.
pub fn korttipinot() -> Vec<Tahko> {
    let mut tahkot = Vec::new();
    for (i, vari) in [WB_SININEN, VALKOINEN, OSUMA].into_iter().enumerate() {
        let x = -3.6 + i as f32 * 2.4;
        let z = i as f32 * 0.5 - 0.5;
        kuutio([x, x + 2.4], [0.0, 3.4], [z, z + 0.12], vari, &mut tahkot);
    }
    tahkot
}

pub fn kesto(ms: u32) -> String {
    let sekunnit = ms / 1000;
    format!("{}:{:02}", sekunnit / 60, sekunnit % 60)
}

/// Alkuruutu piirretään klassiseen kuvaan, jonka käynnistin skaalaa ruudulle.
pub fn piirra_alku(peli: &Pasianssi, kuva: &mut [u32]) {
    let mut p = Piirtaja {
        k: Kangas::new(kuva),
        kello: peli.kello_ms,
    };
    p.pasianssi_alkuruutu(peli);
    p.aanikytkin(peli.aanet.paalla);
}

pub fn piirra(peli: &Pasianssi, ruutu: &mut [u32]) {
    let mut p = Piirtaja {
        k: Kangas::kokoinen(ruutu, RUUDUN_LEVEYS, RUUDUN_KORKEUS),
        kello: peli.kello_ms,
    };
    p.pasianssi_nakyma(peli);
    p.aanikytkin(peli.aanet.paalla);
}

impl Piirtaja<'_> {
    fn pasianssi_alkuruutu(&mut self, peli: &Pasianssi) {
        let vaikeus = peli.vaikeus;
        self.maisema(kaanna("pasianssi.banneri"), &korttipinot());
        let otsikko = listan_otsikko(peli.netti_vastasi());
        self.parhaat_alkuruudulla(otsikko, peli.nakyva_lista());
        let (osat, rivit): (&[&str], &[&str]) = match peli.variantti {
            Variantti::Klondike => (&ALKU_OSAT, &ALKU_RIVIT),
            Variantti::Spider => (&SPIDER_OSAT, &SPIDER_RIVIT),
            Variantti::Pyramidi => (&PYRAMIDI_OSAT, &PYRAMIDI_RIVIT),
        };
        let y = self.alkuohje(osat, rivit) + RIVIN_KORKEUS / 2;
        self.teksti(
            48,
            y,
            kaanna(tason_kuvaus(peli.variantti, vaikeus)),
            KELTAINEN,
        );

        self.teksti(48, VARIANTTI_Y + 3, kaanna("pasianssi.laji"), KELTAINEN);
        for (laji, teksti, x) in VARIANTIT {
            if laji == peli.variantti {
                self.nappi(x, VARIANTTI_Y, teksti, WB_ORANSSI, MUSTA);
            } else {
                self.nappi(x, VARIANTTI_Y, teksti, VALKOINEN, WB_SININEN);
            }
        }
        self.teksti(48, TASO_Y + 3, kaanna("yhteinen.taso"), KELTAINEN);
        for (taso, teksti, x) in TASOT {
            if taso == vaikeus {
                self.nappi(x, TASO_Y, teksti, WB_ORANSSI, MUSTA);
            } else {
                self.nappi(x, TASO_Y, teksti, VALKOINEN, WB_SININEN);
            }
        }
        self.aloituskehote(PASIANSSI_ALOITUS);
        self.nappi(
            haastenapin_x(),
            ALOITUS_Y,
            HAASTENAPPI,
            VALKOINEN,
            WB_SININEN,
        );
    }

    fn pasianssi_nakyma(&mut self, peli: &Pasianssi) {
        self.pelitausta();
        if peli.vaihe == Vaihe::Haasteet {
            self.haastelista(peli);
            return;
        }
        let mut nimi = format!(
            "{} - {}",
            kaanna("peli.pasianssi"),
            lajin_nimi(peli.variantti)
        );
        if let Some(haaste) = peli.haaste {
            nimi += " - ";
            nimi += &muotoile("pasianssi.haaste", &[&(haaste + 1), &HAASTEITA]);
        }
        self.otsikkopalkki(&nimi.to_uppercase(), vaikeuden_nimi(peli.laudan_vaikeus()));

        match peli.variantti {
            Variantti::Klondike => self.klondike_lauta(peli),
            Variantti::Spider => self.spider_lauta(peli),
            Variantti::Pyramidi => self.pyramidi_lauta(peli),
        }
        match peli.vaihe {
            Vaihe::Voitto => {
                self.pasianssi_juhla(peli);
                self.pasianssi_loppupalkki(VOITTOTEKSTI);
            }
            Vaihe::Nimi => {
                self.pasianssi_juhla(peli);
                self.nimi_ikkuna(peli.nimen_pisteet(), &peli.nimi);
            }
            Vaihe::Tappio => self.pasianssi_loppupalkki(TAPPIOTEKSTI),
            Vaihe::Alku | Vaihe::Haasteet | Vaihe::Pelissa => {}
        }
    }

    fn klondike_lauta(&mut self, peli: &Pasianssi) {
        self.pasianssi_ylarivi(peli);
        for sarake in 0..SARAKKEITA {
            self.pasianssi_sarake(peli, sarake);
        }
        self.pasianssi_sivu(peli);
        self.pasianssi_kursori(peli);

        if let Some(pito) = peli.pito.filter(|pito| pito.liikkui)
            && let Some(kortit) = peli.lauta.siirrettavat(pito.lahde)
        {
            let x = peli.hiiri.0 - pito.tartunta.0;
            let y = peli.hiiri.1 - pito.tartunta.1;
            for (i, &kortti) in kortit.iter().enumerate() {
                self.pelikortti(x, y + i as i32 * NAKYVA_ASKEL, kortti);
            }
        }
    }

    /// Korttia vedetään juuri nyt, joten se piirretään osoittimeen eikä paikalleen.
    fn vedossa(peli: &Pasianssi, lahde: Lahde) -> bool {
        peli.pito
            .is_some_and(|pito| pito.liikkui && pito.lahde == lahde)
    }

    fn pasianssi_ylarivi(&mut self, peli: &Pasianssi) {
        let tila = &peli.lauta.tila;

        let pakka_x = sarakkeen_x(0);
        if tila.pakka.is_empty() {
            self.korttipaikka(pakka_x, YLARIVI_Y);
            if peli.lauta.voi_kierrattaa() {
                let (kx, ky) = (pakka_x + KORTTI_L / 2, YLARIVI_Y + KORTTI_K / 2);
                self.k.ympyra((kx, ky), 30.0, TORNI);
            }
        } else {
            self.kortin_tausta(pakka_x, YLARIVI_Y);
        }

        let jate_x = sarakkeen_x(JATE_SARAKE as usize);
        let nakyvia = viuhka(&peli.lauta);
        if nakyvia == 0 {
            self.korttipaikka(jate_x, YLARIVI_Y);
        }
        let viuhkan_kortit = &tila.jate[tila.jate.len() - nakyvia..];
        for (i, &kortti) in viuhkan_kortit.iter().enumerate() {
            let paallimmainen = i + 1 == nakyvia;
            if paallimmainen && Self::vedossa(peli, Lahde::Jate) {
                continue;
            }
            let x = jate_x + i as i32 * VIUHKA_ASKEL;
            self.pelikortti(x, YLARIVI_Y, kortti);
            if paallimmainen && peli.valittu == Some(Lahde::Jate) {
                self.korostus(x, YLARIVI_Y, KORTTI_K, WB_ORANSSI);
            }
        }

        for perusta in 0..MAITA {
            let x = sarakkeen_x(perusta + PERUSTA_ALKAA);
            let lahde = Lahde::Perusta(perusta);
            let pino = &tila.perustat[perusta];
            let nakyva = if Self::vedossa(peli, lahde) {
                pino.len().saturating_sub(1)
            } else {
                pino.len()
            };
            match nakyva.checked_sub(1).map(|i| pino[i]) {
                Some(kortti) => self.pelikortti(x, YLARIVI_Y, kortti),
                None => {
                    self.korttipaikka(x, YLARIVI_Y);
                    let keski = (x + KORTTI_L / 2, YLARIVI_Y + KORTTI_K / 2);
                    self.himmea_maa(Maa::KAIKKI[perusta], keski);
                }
            }
            if peli.valittu == Some(lahde) {
                self.korostus(x, YLARIVI_Y, KORTTI_K, WB_ORANSSI);
            }
        }
    }

    fn pasianssi_sarake(&mut self, peli: &Pasianssi, sarake: usize) {
        let x = sarakkeen_x(sarake);
        let tiedot = &peli.lauta.tila.sarakkeet[sarake];
        if tiedot.kortit.is_empty() {
            self.korttipaikka(x, TAULU_Y);
            return;
        }
        // Vedettävä jono jää piirtämättä: se kulkee osoittimen mukana.
        let piirrettavia = match peli.pito {
            Some(pito) if pito.liikkui => match pito.lahde {
                Lahde::Sarake { sarake: s, kortti } if s == sarake => kortti,
                _ => tiedot.kortit.len(),
            },
            _ => tiedot.kortit.len(),
        };
        for (i, &kortti) in tiedot.kortit[..piirrettavia].iter().enumerate() {
            let y = kortin_y(tiedot, i);
            if i < tiedot.piilossa {
                self.kortin_tausta(x, y);
            } else {
                self.pelikortti(x, y, kortti);
            }
        }
        if let Some(Lahde::Sarake { sarake: s, kortti }) = peli.valittu
            && s == sarake
            && kortti < tiedot.kortit.len()
        {
            let y = kortin_y(tiedot, kortti);
            let korkeus = kortin_y(tiedot, tiedot.kortit.len() - 1) + KORTTI_K - y;
            self.korostus(x, y, korkeus, WB_ORANSSI);
        }
    }

    fn pasianssi_kursori(&mut self, peli: &Pasianssi) {
        if !peli.kursori_nakyy || peli.vaihe != Vaihe::Pelissa || !self.vilkku() {
            return;
        }
        let (x, y, korkeus) = match peli.kursorin_kohta() {
            Some(Kohta::Pakka) => (sarakkeen_x(0), YLARIVI_Y, KORTTI_K),
            Some(Kohta::Jate) => (jatteen_x(&peli.lauta), YLARIVI_Y, KORTTI_K),
            Some(Kohta::Perusta(perusta)) => {
                (sarakkeen_x(perusta + PERUSTA_ALKAA), YLARIVI_Y, KORTTI_K)
            }
            Some(Kohta::Sarake { sarake, kortti }) => {
                let tiedot = &peli.lauta.tila.sarakkeet[sarake];
                let i = kortti.unwrap_or(0);
                let y = kortin_y(tiedot, i);
                let viimeinen = tiedot.kortit.len().saturating_sub(1);
                let korkeus = kortin_y(tiedot, viimeinen) + KORTTI_K - y;
                (sarakkeen_x(sarake), y, korkeus)
            }
            None => return,
        };
        self.korostus(x, y, korkeus, KELTAINEN);
    }

    pub fn korostus(&mut self, x: i32, y: i32, korkeus: i32, vari: u32) {
        for i in 0..3 {
            self.k
                .kehys(x - i, y - i, KORTTI_L + 2 * i, korkeus + 2 * i, vari);
        }
    }

    fn pasianssi_sivu(&mut self, peli: &Pasianssi) {
        let tila = &peli.lauta.tila;
        let kierros = match peli.lauta.vaikeus.kierroksia() {
            Some(kierroksia) => format!("{}/{kierroksia}", peli.lauta.kierros()),
            None => peli.lauta.kierros().to_string(),
        };
        let tilastot = [
            (kaanna("yhteinen.pisteet"), peli.pisteet().to_string()),
            (kaanna("yhteinen.siirrot"), tila.siirrot.to_string()),
            (kaanna("pasianssi.aika"), kesto(peli.peliaika_ms)),
            (kaanna("pasianssi.kierros"), kierros),
            (kaanna("pasianssi.pakassa"), tila.pakka.len().to_string()),
        ];
        let peruutus = peli.lauta.voi_perua();
        self.sivupaneeli(peli, tilastot, &SIVU_OHJEET, &NAPIT, peruutus);
    }

    /// Klondiken ja Pyramidin yhteinen sivupaneeli.
    pub fn sivupaneeli(
        &mut self,
        peli: &Pasianssi,
        tilastot: [(&str, String); 5],
        ohjeet: &[&str],
        napit: &[(Nappi, &str)],
        peruutus: bool,
    ) {
        self.ikkuna(SIVU_X - 12, 42, SIVU_L + 24, RUUDUN_KORKEUS as i32 - 66);
        for (i, (otsikko, arvo)) in tilastot.into_iter().enumerate() {
            let y = YLARIVI_Y + i as i32 * TILASTO_ASKEL;
            self.suurteksti(SIVU_TEKSTI_X, y, otsikko, OTSIKKO, KELTAINEN, false);
            self.suurteksti(SIVU_TEKSTI_X, y + 30, &arvo, OTSIKKO, VALKOINEN, true);
        }

        self.ikkuna(SIVU_X, VIESTI_Y, SIVU_L, VIESTI_K);
        let mut y = VIESTI_Y + 9;
        for rivi in rivita(&peli.viesti, SIVU_MERKKEJA) {
            self.teksti(SIVU_TEKSTI_X, y, rivi, VALKOINEN);
            y += RIVIN_KORKEUS;
        }

        let mut y = SIVU_OHJE_Y;
        for rivi in ohjeet {
            self.teksti(SIVU_TEKSTI_X, y, kaanna(rivi), HUTI);
            y += RIVIN_KORKEUS;
        }

        for (i, &(nappi, teksti)) in napit.iter().enumerate() {
            let y = NAPIT_Y + i as i32 * NAPPI_ASKEL;
            let kaytossa = peli.vaihe == Vaihe::Pelissa
                || (peli.vaihe != Vaihe::Nimi
                    && matches!(nappi, Nappi::Uusi | Nappi::Alusta | Nappi::Valikko));
            let kaytossa = kaytossa && (nappi != Nappi::Peruuta || peruutus);
            let vari = if kaytossa { WB_SININEN } else { KANSI };
            let teksti = uuden_napin_teksti(peli, nappi, teksti);
            self.nappi(SIVU_TEKSTI_X, y, teksti, VALKOINEN, vari);
        }
    }

    /// Klassinen loppu: kortit pomppivat perustoilta ruudun yli ja jättävät jäljen.
    fn pasianssi_juhla(&mut self, peli: &Pasianssi) {
        let kulunut = self.kello.saturating_sub(peli.voitto_alku);
        // Spiderissa kortteja on tuplasti, mutta neljä sarjaa riittää juhlaan ja pitää kuorman samana.
        let maat: [Maa; MAITA] = std::array::from_fn(|i| match peli.variantti {
            Variantti::Spider => peli
                .spider
                .tila
                .valmiit
                .get(i)
                .copied()
                .unwrap_or(Maa::KAIKKI[i]),
            Variantti::Klondike | Variantti::Pyramidi => Maa::KAIKKI[i],
        });
        for (i, &kortin_maa) in maat.iter().enumerate() {
            for kortin_arvo in (1..=KUNINGAS).rev() {
                let kortti = Kortti {
                    maa: kortin_maa,
                    arvo: kortin_arvo,
                };
                let jarjestys = (KUNINGAS - kortin_arvo) as u32 * MAITA as u32 + i as u32;
                let alku = jarjestys * VOITTO_KORTTI_VALI_MS;
                for jalki in (0..VOITTO_JALKIA).rev() {
                    let viive = jalki * VOITTO_JALKI_MS;
                    let Some(aika) = kulunut.checked_sub(alku + viive) else {
                        continue;
                    };
                    let (x, y) = juhlapaikka(i, jarjestys, aika);
                    if x + KORTTI_L > 0 && x < RUUDUN_LEVEYS as i32 {
                        self.pelikortti(x, y, kortti);
                    }
                }
            }
        }
    }

    /// Oma tausta, jotta teksti erottuu korttien päältä.
    fn pasianssi_loppupalkki(&mut self, avain: &str) {
        let teksti = kaanna(avain);
        let y = RUUDUN_KORKEUS as i32 - 84;
        let leveys = teksti.chars().count() as i32 * OTSIKKO.2 + 36;
        self.ikkuna(SARAKE_X, y, leveys, 48);
        self.suurteksti(SARAKE_X + 18, y + 13, teksti, OTSIKKO, KELTAINEN, true);
    }
}

/// Kortin paikka juhlassa `aika` millisekuntia lähdön jälkeen: tasainen vaakaliike
/// ja vaimenevat pomput ruudun alareunaan.
fn juhlapaikka(maa: usize, jarjestys: u32, aika: u32) -> (i32, i32) {
    let suunta = if jarjestys.is_multiple_of(2) {
        1.0
    } else {
        -1.0
    };
    let nopeus = (0.18 + (jarjestys % 5) as f32 * 0.05) * suunta;
    let t = aika as f32;
    let x = sarakkeen_x(maa + PERUSTA_ALKAA) as f32 + nopeus * t;
    let pohja = (RUUDUN_KORKEUS as i32 - KORTTI_K) as f32;
    let korkeus = pohja - YLARIVI_Y as f32;
    let pomppu = (t * 0.0035 + std::f32::consts::FRAC_PI_2).cos().abs();
    let vaimennus = 1.0 / (1.0 + t * 0.0006);
    (x as i32, (pohja - pomppu * korkeus * vaimennus) as i32)
}

#[cfg(test)]
mod tests;
#[cfg(test)]
pub use tests::{
    SIVU_ALKAA, haastenapin_keskipiste, kortin_kulma_pakka, napin_keskipiste,
    napin_keskipiste_listasta, sivu_mahtuu, vaikeuden_keskipiste, variantin_keskipiste,
};
