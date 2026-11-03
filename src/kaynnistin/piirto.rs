//! Käynnistimen näkymä: pelikortit ja ohjeet.

use std::sync::LazyLock;

use klassikot8000::laivanupotus::Vaikeus;

use super::netti::VERSIO;
use super::{Kaynnistin, PELIT};
use crate::pelit::laivanupotus::peli::Tilasto;
use crate::yhteinen::kieli::{kaanna, muotoile};
use crate::yhteinen::piirto::*;
use crate::yhteinen::rasteri::Rasteri;
use crate::yhteinen::tulokset::{PARHAITA, Tulos};
use crate::yhteinen::vektori::{Kangas, Tahko};

// Käynnistin piirtää suoraan ruudulle, joten mitat ovat ruudun pikseleitä.
const RUUTU_L: i32 = RUUDUN_LEVEYS as i32;

const KORTTI_X: i32 = 48;
const KORTTI_L: i32 = 704;
const KORTTI_Y: i32 = 172;
const KORTTI_K: i32 = 72;
const KORTTI_ASKEL: i32 = KORTTI_K + 8;
const KORTIN_NUMERO_X: i32 = 60;
const KORTIN_NIMI: (i32, i32, i32) = (24, 36, 32);
const KORTIN_VARJO: i32 = 6;
/// Valikkoon mahtuvat kortit. Pidempi lista vierii, ja nuolet kertovat piiloon jääneistä.
pub const NAKYVIA: usize = 10;
const NAKYVIA_KORTTEJA: i32 = if PELIT.len() < NAKYVIA {
    PELIT.len()
} else {
    NAKYVIA
} as i32;
const NUOLI_X: i32 = (KORTTI_X + KORTTI_L + KORTIN_VARJO + PANEELI_X) / 2;
const NUOLI_L: i32 = 24;
const NUOLI_K: i32 = 16;

const OTSIKKO_KOKO: (i32, i32, i32) = (60, 88, 78);
const OTSIKKO_Y: i32 = 48;

// Tausta lasketaan kerran, ja se pysyy paikallaan katsojan seuratessa mallia.
const HORISONTTI_RIVI: i32 = 130;
static MAISEMA: LazyLock<Rasteri> = LazyLock::new(|| {
    maisemakuva(
        RUUDUN_LEVEYS / 4,
        ALAPALKKI_Y as usize / 4,
        HORISONTTI_RIVI,
        MALLI_X / 4,
    )
});
const MALLI_X: i32 = 1336;
const MALLI_Y: i32 = 460;
const MALLIN_KOKO: f32 = 76.0;
const MALLIN_KEINUNTA: f32 = 6.0;
pub const MALLIN_KASVU_MS: u32 = 200;
const MALLIN_ALKUKOKO: f32 = 0.6;

const PANEELI_X: i32 = 800;
const PANEELI_L: i32 = 1072;
const PANEELI_YLA: i32 =
    KORTTI_Y + (NAKYVIA_KORTTEJA - 1) * KORTTI_ASKEL + KORTTI_K - PANEELI_KORKEUS;
// Korkeus mahduttaa lajin nimen ja oman ennätyksen listan ympärille sekä laivanupotuksen
// tilaston kuvauksen alle. Malli mahtuu silti paneelin yläpuolelle.
const PANEELI_KORKEUS: i32 = 240;
const PANEELI_SISENNYS: i32 = 24;

const TEKSTIN_KERROIN: i32 = 4;
const SUURI_MERKKI: i32 = TEKSTIN_KERROIN * 8;
const KUVAUS_RIVI: i32 = 40;
const KUVAUS_MERKKEJA: usize = ((PANEELI_L - 2 * PANEELI_SISENNYS) / SUURI_MERKKI) as usize;
const KAPEA_KUVAUS_MERKKEJA: usize =
    ((PANEELI_L - 3 * PANEELI_SISENNYS - LISTA_L) / SUURI_MERKKI) as usize;

const LISTA_L: i32 = 350;
const LISTA_KOKO: (i32, i32, i32) = (14, 21, 20);
/// Lajin nimi on otsikon alla omalla rivillään.
const LISTA_RIVI: i32 = PARHAAT_OTSIKKO;
/// Oma ennätys erottuu listasta pienellä välillä.
const OMA_VALI: i32 = 6;
const OMA_PARAS: &str = "valikko.oma_paras";

// Laivanupotuksen tilasto kuvauksen alla: tasot riveinä, sarakkeiden oikeat reunat.
const TAULUKON_VALI: i32 = 16;
const TAULUKON_SARAKKEET: [(&str, i32); 3] = [
    ("valikko.voitot", 360),
    ("valikko.tappiot", 530),
    ("yhteinen.paras", 650),
];
const TASOJEN_NIMET: [&str; 3] = ["yhteinen.helppo", "yhteinen.normaali", "yhteinen.vaikea"];
/// Otsikkorivi ja kolme tasoa.
const TAULUKON_KORKEUS: i32 = 3 * PARHAAT_RIVI + LISTA_KOKO.1;

const ALAPALKKI_Y: i32 = 976;
const OHJE_X: i32 = 48;
const OHJE_RIVI_Y: i32 = 1012;

const KIELIKYTKIN: &str = "valikko.kielikytkin";
const KIELI_Y: i32 = 6;
const KIELI_VALI: i32 = 12;
// Yhteinen äänikytkin piirtyy kankaan oikeaan laitaan, jossa on nyt koko ruudun leveys.
const AANI_SIIRTO: i32 = (RUUDUN_LEVEYS - LEVEYS) as i32;

const VERSIO_Y: i32 = KIELI_Y + (NAPPI_K - MERKKI) / 2;
// Otsikon ja korttien välissä on vapaata tilaa yhden rivin verran.
const ILMOITUS_Y: i32 = 141;
const ILMOITUS_K: i32 = 28;
const ILMOITUS_SISENNYS: i32 = 12;

/// Uuden version ilmoituksen kehys: (x, leveys). Leveys seuraa tekstiä.
fn ilmoituksen_alue(versio: &str) -> (i32, i32) {
    let leveys = muotoile("valikko.uusi_versio", &[&versio]).chars().count() as i32 * MERKKI;
    let x = (RUUTU_L - leveys) / 2;
    (x - ILMOITUS_SISENNYS, leveys + 2 * ILMOITUS_SISENNYS)
}

/// Ilmoitusta klikkaamalla pääsee julkaisusivulle.
pub fn ilmoituksella(x: i32, y: i32, versio: &str) -> bool {
    let (vasen, leveys) = ilmoituksen_alue(versio);
    (vasen..vasen + leveys).contains(&x) && (ILMOITUS_Y..ILMOITUS_Y + ILMOITUS_K).contains(&y)
}

#[cfg(test)]
pub fn ilmoituksen_keskipiste(versio: &str) -> (i32, i32) {
    let (vasen, leveys) = ilmoituksen_alue(versio);
    (vasen + leveys / 2, ILMOITUS_Y + ILMOITUS_K / 2)
}

/// Listan korkeus otsikosta viimeisen rivin alareunaan.
fn listan_korkeus(laji: bool, oma: bool) -> i32 {
    let rivit = if oma {
        PARHAITA as i32 * PARHAAT_RIVI + OMA_VALI
    } else {
        (PARHAITA as i32 - 1) * PARHAAT_RIVI
    };
    (1 + laji as i32) * LISTA_RIVI + rivit + LISTA_KOKO.1
}

/// Kuvauksen ja mahdollisen tilaston yhteinen korkeus.
fn kuvauksen_korkeus(riveja: usize, tilasto: bool) -> i32 {
    let kuvaus = riveja as i32 * KUVAUS_RIVI - (KUVAUS_RIVI - SUURI_MERKKI);
    if tilasto {
        kuvaus + TAULUKON_VALI + TAULUKON_KORKEUS
    } else {
        kuvaus
    }
}

/// Kytkin on äänikytkimen vasemmalla puolella, ja sen leveys seuraa tekstiä.
fn kielikytkimen_x() -> i32 {
    AANI_X + AANI_SIIRTO - KIELI_VALI - napin_leveys(KIELIKYTKIN)
}

pub fn aanikytkimella(x: i32, y: i32) -> bool {
    crate::yhteinen::piirto::aanikytkimella(x - AANI_SIIRTO, y)
}

#[cfg(test)]
pub fn aanikytkimen_keskipiste() -> (i32, i32) {
    let (x, y) = crate::yhteinen::piirto::aanikytkimen_keskipiste();
    (x + AANI_SIIRTO, y)
}

pub fn kielikytkimella(x: i32, y: i32) -> bool {
    let vasen = kielikytkimen_x();
    (vasen..vasen + napin_leveys(KIELIKYTKIN)).contains(&x)
        && (KIELI_Y..KIELI_Y + NAPPI_K).contains(&y)
}

#[cfg(test)]
pub fn kielikytkimen_keskipiste() -> (i32, i32) {
    (
        kielikytkimen_x() + napin_leveys(KIELIKYTKIN) / 2,
        KIELI_Y + NAPPI_K / 2,
    )
}

/// Näkyvä paikka, jonka kortin kohdalla piste on. Käynnistin tietää, mikä peli paikalla on.
pub fn kortti_pisteesta(x: i32, y: i32) -> Option<usize> {
    if !(KORTTI_X..KORTTI_X + KORTTI_L).contains(&x) {
        return None;
    }
    let i = (y - KORTTI_Y).div_euclid(KORTTI_ASKEL);
    let sisalla = (y - KORTTI_Y).rem_euclid(KORTTI_ASKEL) < KORTTI_K;
    (sisalla && (0..NAKYVIA_KORTTEJA).contains(&i)).then_some(i as usize)
}

#[cfg(test)]
pub fn kortin_keskipiste(i: usize) -> (i32, i32) {
    (
        KORTTI_X + KORTTI_L / 2,
        KORTTI_Y + i as i32 * KORTTI_ASKEL + KORTTI_K / 2,
    )
}

/// Piirtää esillä olevan näkymän ruudulle. Pelien klassiset näkymät kulkevat välikuvan kautta.
pub fn piirra(kaynnistin: &Kaynnistin, kehys: &mut Kehys) {
    if let Some(peli) = &kaynnistin.peli {
        peli.nakyma().piirra(kehys);
        return;
    }
    let mut p = Piirtaja {
        k: Kangas::kokoinen(&mut kehys.ruutu, RUUDUN_LEVEYS, RUUDUN_KORKEUS),
        kello: kaynnistin.kello_ms,
    };
    p.valikko(kaynnistin);
    p.aanikytkin(kaynnistin.aanet_paalla);
    p.nappi(kielikytkimen_x(), KIELI_Y, KIELIKYTKIN, VALKOINEN, MUSTA);
    kehys.piirretty_suoraan();
}

impl Piirtaja<'_> {
    fn valikko(&mut self, kaynnistin: &Kaynnistin) {
        MAISEMA.piirra(&mut self.k, 0, 0);
        let ika = kaynnistin.kello_ms.saturating_sub(kaynnistin.valittu_ms);
        self.valikon_malli(&kaynnistin.malli, ika);
        self.valikon_otsikko(kaanna("valikko.banneri"));
        let versio = muotoile("valikko.versio", &[&VERSIO]);
        self.teksti(OHJE_X, VERSIO_Y, &versio, VALKOINEN);
        if let Some(uusi) = kaynnistin.uusi_versio() {
            self.versioilmoitus(uusi, kaynnistin.ilmoitus_alla());
        }
        for paikka in 0..NAKYVIA_KORTTEJA as usize {
            let i = kaynnistin.alku + paikka;
            self.valikkokortti(paikka, i, i == kaynnistin.valittu);
        }
        let alempana = kaynnistin.alku + NAKYVIA < PELIT.len();
        self.vieritysnuolet(kaynnistin.alku > 0, alempana);
        self.kuvauspaneeli(kaynnistin);
        self.ohjepalkki(kaanna("valikko.ohje"));
    }

    /// `ika` on aika valinnan vaihtumisesta: uusi malli kasvaa täyteen kokoonsa.
    fn valikon_malli(&mut self, malli: &[Tahko], ika: u32) {
        let keinunta = (self.kello as f32 * 0.002).sin() * MALLIN_KEINUNTA;
        let kasvu = (ika as f32 / MALLIN_KASVU_MS as f32).min(1.0);
        self.k.malli(
            malli,
            self.kello as f32 * LAIVAN_KIERTO,
            (MALLI_X, MALLI_Y + keinunta as i32),
            MALLIN_KOKO * (MALLIN_ALKUKOKO + (1.0 - MALLIN_ALKUKOKO) * kasvu),
        );
    }

    fn valikon_otsikko(&mut self, teksti: &str) {
        let x = (RUUTU_L - teksti.chars().count() as i32 * OTSIKKO_KOKO.2) / 2;
        self.suurteksti(x, OTSIKKO_Y, teksti, OTSIKKO_KOKO, KELTAINEN, true);
    }

    /// Hiiren alla ilmoitus vaalenee kuin painike, koska klikkaus avaa julkaisusivun.
    fn versioilmoitus(&mut self, versio: &str, hiiren_alla: bool) {
        let teksti = muotoile("valikko.uusi_versio", &[&versio]);
        let (reuna_x, reuna_l) = ilmoituksen_alue(versio);
        let tausta = if hiiren_alla { VALKOINEN } else { KELTAINEN };
        self.nappitausta(reuna_x, ILMOITUS_Y, reuna_l, ILMOITUS_K, tausta);
        let y = ILMOITUS_Y + (ILMOITUS_K - MERKKI) / 2;
        self.teksti(reuna_x + ILMOITUS_SISENNYS, y, &teksti, MUSTA);
    }

    /// Kortti näkyvällä `paikka`lla. `i` on pelin järjestysnumero `PELIT`-listassa.
    fn valikkokortti(&mut self, paikka: usize, i: usize, valittu: bool) {
        let y = KORTTI_Y + paikka as i32 * KORTTI_ASKEL;
        let keski_y = y + KORTTI_K / 2;
        let (numero_vari, nimi_vari) = if valittu {
            self.k.laatikko(
                KORTTI_X + KORTIN_VARJO,
                y + KORTIN_VARJO,
                KORTTI_L,
                KORTTI_K,
                MUSTA,
            );
            self.nappitausta(KORTTI_X, y, KORTTI_L, KORTTI_K, WB_ORANSSI);
            self.k.monikulmio(
                &[
                    (KORTTI_X + 20, keski_y - 18),
                    (KORTTI_X + 44, keski_y),
                    (KORTTI_X + 20, keski_y + 18),
                ],
                MUSTA,
            );
            (MUSTA, MUSTA)
        } else {
            self.ikkuna(KORTTI_X, y, KORTTI_L, KORTTI_K);
            (KELTAINEN, VALKOINEN)
        };
        let koko = KORTIN_NIMI;
        let teksti_y = keski_y - koko.1 / 2;
        let x = KORTTI_X + KORTIN_NUMERO_X;
        // Kymmenes kortti on nolla, kuten numerorivillä.
        let numero = ((paikka + 1) % 10).to_string();
        self.suurteksti(x, teksti_y, &numero, koko, numero_vari, true);
        let nimi = kaanna(PELIT[i].nimi).to_uppercase();
        self.suurteksti(x + 2 * koko.2, teksti_y, &nimi, koko, nimi_vari, true);
    }

    /// Nuolet listan vieressä, kun sen ylä- tai alapuolelle jää pelejä piiloon.
    fn vieritysnuolet(&mut self, ylempana: bool, alempana: bool) {
        let (vasen, oikea) = (NUOLI_X - NUOLI_L / 2, NUOLI_X + NUOLI_L / 2);
        if ylempana {
            let y = KORTTI_Y;
            let karki = (NUOLI_X, y);
            self.k.monikulmio(
                &[karki, (oikea, y + NUOLI_K), (vasen, y + NUOLI_K)],
                KELTAINEN,
            );
        }
        if alempana {
            let y = KORTTI_Y + (NAKYVIA_KORTTEJA - 1) * KORTTI_ASKEL + KORTTI_K;
            let karki = (NUOLI_X, y);
            self.k.monikulmio(
                &[(vasen, y - NUOLI_K), (oikea, y - NUOLI_K), karki],
                KELTAINEN,
            );
        }
    }

    /// Kuvaus vasemmalla, laivanupotuksella tilasto sen alla. Jos pelillä on tuloksia, ne vievät
    /// paneelin oikean laidan: nettilista, kun se on saapunut, muuten omat tulokset.
    fn kuvauspaneeli(&mut self, kaynnistin: &Kaynnistin) {
        self.ikkuna(PANEELI_X, PANEELI_YLA, PANEELI_L, PANEELI_KORKEUS);
        let merkkeja = if let Some((netista, parhaat)) = kaynnistin.valitun_parhaat() {
            self.valikon_lista(
                netista,
                kaynnistin.valikon_listan_laji(),
                parhaat,
                kaynnistin.valitun_oma_paras(),
            );
            KAPEA_KUVAUS_MERKKEJA
        } else {
            KUVAUS_MERKKEJA
        };
        let rivit = rivita(kaanna(PELIT[kaynnistin.valittu].kuvaus), merkkeja);
        let tilastot = kaynnistin.valitun_tilastot();
        let korkeus = kuvauksen_korkeus(rivit.len(), tilastot.is_some());
        let x = PANEELI_X + PANEELI_SISENNYS;
        let mut y = PANEELI_YLA + (PANEELI_KORKEUS - korkeus) / 2;
        for rivi in rivit {
            self.teksti_koossa(x, y, rivi, TEKSTIN_KERROIN, VALKOINEN);
            y += KUVAUS_RIVI;
        }
        if let Some(tilastot) = tilastot {
            let y = y - (KUVAUS_RIVI - SUURI_MERKKI) + TAULUKON_VALI;
            self.laivatilasto(x, y, tilastot);
        }
    }

    /// Lista paneelin oikeassa laidassa pystysuunnassa keskellä. Otsikon alla on lajin nimi,
    /// kun pelin lajeilla on omat listansa, ja listan alla oma ennätys, kun sellainen on.
    fn valikon_lista(
        &mut self,
        netista: bool,
        laji: Option<&str>,
        parhaat: &[Tulos],
        oma: Option<u32>,
    ) {
        let x = PANEELI_X + PANEELI_L - PANEELI_SISENNYS - LISTA_L;
        let korkeus = listan_korkeus(laji.is_some(), oma.is_some());
        let mut y = PANEELI_YLA + (PANEELI_KORKEUS - korkeus) / 2;
        self.teksti(x, y, listan_otsikko(netista), KELTAINEN);
        y += LISTA_RIVI;
        if let Some(laji) = laji {
            self.teksti(x, y, laji, LAVENTELI);
            y += LISTA_RIVI;
        }
        self.parhaat_rivit(x, y, LISTA_L, LISTA_KOKO, parhaat);
        if let Some(oma) = oma {
            let y = y + PARHAITA as i32 * PARHAAT_RIVI + OMA_VALI;
            self.suurteksti(x, y, kaanna(OMA_PARAS), LISTA_KOKO, LAVENTELI, false);
            self.oikealle(x + LISTA_L, y, &oma.to_string(), KELTAINEN);
        }
    }

    /// Voitot, tappiot ja paras voitto pisteinä kultakin tasolta.
    fn laivatilasto(&mut self, x: i32, y: i32, tilastot: &[Tilasto; 3]) {
        for (avain, oikea) in TAULUKON_SARAKKEET {
            self.oikealle(x + oikea, y, kaanna(avain), KELTAINEN);
        }
        for (i, (tilasto, taso)) in tilastot.iter().zip(Vaikeus::KAIKKI).enumerate() {
            let y = y + (i as i32 + 1) * PARHAAT_RIVI;
            self.suurteksti(x, y, kaanna(TASOJEN_NIMET[i]), LISTA_KOKO, VALKOINEN, false);
            let paras = match tilasto.parhaat_pisteet(taso) {
                0 => "-".to_string(),
                pisteet => pisteet.to_string(),
            };
            let arvot = [
                tilasto.voitot.to_string(),
                tilasto.tappiot.to_string(),
                paras,
            ];
            for (arvo, (_, oikea)) in arvot.iter().zip(TAULUKON_SARAKKEET) {
                self.oikealle(x + oikea, y, arvo, VALKOINEN);
            }
        }
    }

    /// Listan kokoinen teksti, jonka oikea reuna on kohdassa `oikea`.
    fn oikealle(&mut self, oikea: i32, y: i32, teksti: &str, vari: u32) {
        let x = oikea - teksti.chars().count() as i32 * LISTA_KOKO.2;
        self.suurteksti(x, y, teksti, LISTA_KOKO, vari, false);
    }

    fn ohjepalkki(&mut self, ohje: &str) {
        let korkeus = RUUDUN_KORKEUS as i32 - ALAPALKKI_Y;
        self.ikkuna(0, ALAPALKKI_Y, RUUTU_L, korkeus);
        self.teksti_koossa(OHJE_X, OHJE_RIVI_Y, ohje, TEKSTIN_KERROIN, VALKOINEN);
        if self.vilkku() {
            let x = OHJE_X + ohje.chars().count() as i32 * SUURI_MERKKI;
            self.k
                .laatikko(x, OHJE_RIVI_Y, SUURI_MERKKI, SUURI_MERKKI, KELTAINEN);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yhteinen::kieli::kaikilla_kielilla;
    use crate::yhteinen::tulokset::NIMEN_PITUUS;

    #[test]
    fn kortit_ja_paneelit_mahtuvat_kuvaan() {
        const { assert!(KORTTI_Y + NAKYVIA as i32 * KORTTI_ASKEL <= ALAPALKKI_Y) };
        const { assert!(KORTTI_X + KORTTI_L + KORTIN_VARJO < NUOLI_X - NUOLI_L / 2) };
        const { assert!(NUOLI_X + NUOLI_L / 2 < PANEELI_X) };
        const { assert!(PANEELI_X + PANEELI_L < RUUTU_L) };
        const { assert!(OHJE_RIVI_Y + SUURI_MERKKI < RUUDUN_KORKEUS as i32) };
        const { assert!(OTSIKKO_Y + OTSIKKO_KOKO.1 <= ILMOITUS_Y) };
        const { assert!(ILMOITUS_Y + ILMOITUS_K <= KORTTI_Y) };
    }

    #[test]
    fn kielikytkin_loytyy_eika_mene_muiden_paalle() {
        kaikilla_kielilla(|| {
            let (x, y) = kielikytkimen_keskipiste();
            assert!(kielikytkimella(x, y));
            let (aani_x, aani_y) = aanikytkimen_keskipiste();
            assert!(aanikytkimella(aani_x, aani_y));
            assert!(!kielikytkimella(aani_x, aani_y));
            assert!(!aanikytkimella(x, y));
            assert_eq!(kortti_pisteesta(x, y), None);
            assert!(kielikytkimen_x() > 0);
        });
    }

    #[test]
    fn tekstit_mahtuvat_paikoilleen_kummallakin_kielella() {
        kaikilla_kielilla(|| {
            for kortti in &PELIT {
                let nimi = kaanna(kortti.nimi).chars().count() as i32;
                let oikea = KORTIN_NUMERO_X + (2 + nimi) * KORTIN_NIMI.2;
                assert!(oikea <= KORTTI_L, "{}", kortti.nimi);
                for merkkeja in [KUVAUS_MERKKEJA, KAPEA_KUVAUS_MERKKEJA] {
                    let rivit = rivita(kaanna(kortti.kuvaus), merkkeja).len() as i32;
                    let korkeus = 2 * PANEELI_SISENNYS + rivit * KUVAUS_RIVI;
                    assert!(korkeus <= PANEELI_KORKEUS, "{}", kortti.kuvaus);
                }
            }
            // pisin nimi ja seitsennumeroiset pisteet samalle riville
            let rivi = (2 + NIMEN_PITUUS as i32 + 7) * LISTA_KOKO.2;
            assert!(rivi <= LISTA_L);
            for netista in [false, true] {
                let otsikko = listan_otsikko(netista).chars().count() as i32 * MERKKI;
                assert!(otsikko <= LISTA_L);
            }
            let ohje = kaanna("valikko.ohje").chars().count() as i32 + 1;
            assert!(OHJE_X + ohje * SUURI_MERKKI <= RUUTU_L);
            let otsikko = kaanna("valikko.banneri").chars().count() as i32;
            assert!(otsikko * OTSIKKO_KOKO.2 <= RUUTU_L);
            let versio = muotoile("valikko.versio", &[&"100.100.100"]);
            let oikea = OHJE_X + versio.chars().count() as i32 * MERKKI;
            assert!(oikea < kielikytkimen_x());
            let ilmoitus = muotoile("valikko.uusi_versio", &[&"100.100.100"]);
            let leveys = ilmoitus.chars().count() as i32 * MERKKI + 2 * ILMOITUS_SISENNYS;
            assert!(leveys <= RUUTU_L);
        });
    }

    #[test]
    fn paneelin_lista_ja_laivanupotuksen_tilasto_mahtuvat_kummallakin_kielella() {
        let sisa = PANEELI_KORKEUS - 2 * PANEELI_SISENNYS;
        assert!(listan_korkeus(true, true) <= sisa);
        let vasen_l = PANEELI_L - 3 * PANEELI_SISENNYS - LISTA_L;
        let leveys = |teksti: &str| teksti.chars().count() as i32 * LISTA_KOKO.2;
        kaikilla_kielilla(|| {
            let kuvaus = rivita(kaanna(PELIT[0].kuvaus), KAPEA_KUVAUS_MERKKEJA).len();
            assert!(kuvauksen_korkeus(kuvaus, true) <= sisa);
            // Sarakkeiden väliin jää vähintään merkin verran tilaa, myös nelinumeroisille luvuille.
            let tasot = TASOJEN_NIMET.map(|avain| leveys(kaanna(avain)));
            let mut edellinen = *tasot.iter().max().unwrap();
            for (avain, oikea) in TAULUKON_SARAKKEET {
                let sarake = leveys(kaanna(avain)).max(leveys("9999"));
                assert!(edellinen + LISTA_KOKO.2 <= oikea - sarake, "{avain}");
                edellinen = oikea;
            }
            assert!(edellinen <= vasen_l);
            let oma = leveys(kaanna(OMA_PARAS)) + LISTA_KOKO.2 + leveys("1000000");
            assert!(oma <= LISTA_L);
            for variantti in klassikot8000::pasianssi::Variantti::KAIKKI {
                let laji = crate::pelit::pasianssi::piirto::lajin_nimi(variantti);
                assert!(laji.chars().count() as i32 * MERKKI <= LISTA_L, "{laji}");
            }
        });
    }

    #[test]
    fn ilmoitus_loytyy_eika_osu_kortteihin() {
        kaikilla_kielilla(|| {
            let (x, y) = ilmoituksen_keskipiste("100.100.100");
            assert!(ilmoituksella(x, y, "100.100.100"));
            assert_eq!(kortti_pisteesta(x, y), None);
            assert!(!ilmoituksella(x, KORTTI_Y, "100.100.100"));
            assert!(!ilmoituksella(x, ILMOITUS_Y - 1, "100.100.100"));
        });
    }

    #[test]
    fn jokainen_kortti_loytyy_ja_valit_eivat_ole_kohteita() {
        for i in 0..NAKYVIA_KORTTEJA as usize {
            let (x, y) = kortin_keskipiste(i);
            assert_eq!(kortti_pisteesta(x, y), Some(i));
        }
        assert_eq!(kortti_pisteesta(KORTTI_X, KORTTI_Y + KORTTI_K), None);
        assert_eq!(kortti_pisteesta(KORTTI_X, KORTTI_Y - 1), None);
        assert_eq!(kortti_pisteesta(KORTTI_X - 1, KORTTI_Y), None);
        let alin = KORTTI_Y + NAKYVIA_KORTTEJA * KORTTI_ASKEL;
        assert_eq!(kortti_pisteesta(KORTTI_X, alin), None);
    }
}
