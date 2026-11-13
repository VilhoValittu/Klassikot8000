//! Haastelista: kunkin pasianssin haasteet ruudukkona ja haasteiden yhteinen top-lista.

use klassikot8000::haasteet::{HAASTEITA, taso};
use klassikot8000::pasianssi::Variantti;

use super::peli::{HAASTERIVILLA, Pasianssi};
use super::piirto::{
    NAPIT_Y, SIVU_L, SIVU_MERKKEJA, SIVU_TEKSTI_X, SIVU_X, VARIANTIT, vaikeuden_nimi,
};
use crate::yhteinen::kieli::kaanna;
use crate::yhteinen::piirto::*;
use crate::yhteinen::tulokset::PARHAITA;

const LAJI_X: i32 = 48;
const LAJI_Y: i32 = 54;
const SOLU_X: i32 = 48;
const SOLU_Y: i32 = 102;
const SOLU_L: i32 = 234;
const SOLU_K: i32 = 222;
const SOLU_ASKEL_X: i32 = 252;
const SOLU_ASKEL_Y: i32 = 240;
const TAKAISIN: &str = "pasianssi.nappi_takaisin";
const LAPAISTY: u32 = 0x1C7A48;
const LISTA_Y: i32 = 54;
const SUMMA_Y: i32 =
    LISTA_Y + PARHAAT_OTSIKKO + PARHAITA as i32 * parhaiden_askel(OTSIKKO, Some(TULOS)) + 6;
const OHJE_Y: i32 = SUMMA_Y + 90;

fn lajin_x(x: i32) -> i32 {
    // Rivi on sama kuin alkuruudulla, mutta se alkaa pelilaudan vasemmasta laidasta.
    x - VARIANTIT[0].2 + LAJI_X
}

pub fn laji_pisteesta(x: i32, y: i32) -> Option<Variantti> {
    if !(LAJI_Y..LAJI_Y + NAPPI_K).contains(&y) {
        return None;
    }
    VARIANTIT
        .iter()
        .find(|&&(_, teksti, nx)| (lajin_x(nx)..lajin_x(nx) + napin_leveys(teksti)).contains(&x))
        .map(|&(laji, ..)| laji)
}

fn solun_paikka(haaste: usize) -> (i32, i32) {
    (
        SOLU_X + (haaste % HAASTERIVILLA) as i32 * SOLU_ASKEL_X,
        SOLU_Y + (haaste / HAASTERIVILLA) as i32 * SOLU_ASKEL_Y,
    )
}

pub fn haaste_pisteesta(x: i32, y: i32) -> Option<usize> {
    (0..HAASTEITA).find(|&haaste| {
        let (sx, sy) = solun_paikka(haaste);
        (sx..sx + SOLU_L).contains(&x) && (sy..sy + SOLU_K).contains(&y)
    })
}

pub fn takaisin_pisteessa(x: i32, y: i32) -> bool {
    (SIVU_TEKSTI_X..SIVU_TEKSTI_X + napin_leveys(TAKAISIN)).contains(&x)
        && (NAPIT_Y..NAPIT_Y + NAPPI_K).contains(&y)
}

impl Piirtaja<'_> {
    pub fn haastelista(&mut self, peli: &Pasianssi) {
        let otsikko = format!(
            "{} - {}",
            kaanna("peli.pasianssi"),
            kaanna("pasianssi.haasteet_otsikko")
        )
        .to_uppercase();
        let lapi = format!(
            "{}/{}",
            peli.lapaistyja_haasteita(),
            HAASTEITA * VARIANTIT.len()
        );
        self.otsikkopalkki(&otsikko, &lapi);
        for (laji, teksti, x) in VARIANTIT {
            if laji == peli.variantti {
                self.nappi(lajin_x(x), LAJI_Y, teksti, WB_ORANSSI, MUSTA);
            } else {
                self.nappi(lajin_x(x), LAJI_Y, teksti, VALKOINEN, WB_SININEN);
            }
        }
        for haaste in 0..HAASTEITA {
            let pisteet = peli.haastepisteet[peli.variantti as usize][haaste];
            self.haastesolu(haaste, pisteet, peli.haaste_kursori == haaste);
        }
        self.haasteiden_sivu(peli);
    }

    fn haastesolu(&mut self, haaste: usize, pisteet: u32, osoitettu: bool) {
        let (x, y) = solun_paikka(haaste);
        let tausta = if pisteet > 0 { LAPAISTY } else { WB_SININEN };
        self.nappitausta(x, y, SOLU_L, SOLU_K, tausta);
        let numero = (haaste + 1).to_string();
        self.suurteksti(x + 18, y + 18, &numero, SUURI, KELTAINEN, true);
        self.teksti(x + 18, y + 96, vaikeuden_nimi(taso(haaste)), VALKOINEN);
        let tulos = if pisteet > 0 {
            pisteet.to_string()
        } else {
            "-".to_string()
        };
        self.suurteksti(x + 18, y + 150, &tulos, OTSIKKO, VALKOINEN, true);
        if osoitettu {
            self.valintakehys(x, y, SOLU_L, SOLU_K, KELTAINEN);
        }
    }

    fn haasteiden_sivu(&mut self, peli: &Pasianssi) {
        self.ikkuna(SIVU_X - 12, 42, SIVU_L + 24, RUUDUN_KORKEUS as i32 - 66);
        let otsikko = format!(
            "{} - {}",
            kaanna("pasianssi.haasteet_otsikko"),
            listan_otsikko(peli.haastetaulu.netti.is_some())
        );
        let leveys = SIVU_L - 36;
        self.parhaat_lista(
            SIVU_TEKSTI_X,
            LISTA_Y,
            leveys,
            OTSIKKO,
            Some(TULOS),
            &otsikko,
            peli.haastetaulu.nakyva(),
        );

        let summa = peli.haasteiden_summa().to_string();
        let yhteensa = kaanna("pasianssi.haaste_yhteensa");
        self.suurteksti(SIVU_TEKSTI_X, SUMMA_Y, yhteensa, OTSIKKO, KELTAINEN, false);
        self.suurteksti(
            SIVU_TEKSTI_X,
            SUMMA_Y + 30,
            &summa,
            OTSIKKO,
            VALKOINEN,
            true,
        );
        let lapi_x = SIVU_TEKSTI_X + leveys / 2;
        let lapi = peli.lapaistyja_haasteita().to_string();
        let otsake = kaanna("pasianssi.haaste_lapi");
        self.suurteksti(lapi_x, SUMMA_Y, otsake, OTSIKKO, KELTAINEN, false);
        self.suurteksti(lapi_x, SUMMA_Y + 30, &lapi, OTSIKKO, VALKOINEN, true);

        let mut y = OHJE_Y;
        for rivi in rivita(kaanna("pasianssi.haaste_ohje"), SIVU_MERKKEJA) {
            self.teksti(SIVU_TEKSTI_X, y, rivi, VALKOINEN);
            y += RIVIN_KORKEUS;
        }
        y += RIVIN_KORKEUS;
        for avain in [
            "pasianssi.haaste_nappaimet_1",
            "pasianssi.haaste_nappaimet_2",
        ] {
            self.teksti(SIVU_TEKSTI_X, y, kaanna(avain), HUTI);
            y += RIVIN_KORKEUS;
        }
        self.nappi(SIVU_TEKSTI_X, NAPIT_Y, TAKAISIN, VALKOINEN, WB_SININEN);
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
pub use tests::{haasteen_keskipiste, lajin_keskipiste, takaisin_keskipiste};
