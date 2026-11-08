//! Pyramidipasianssin näkymä: pyramidi ja pakka pöydällä, sivupaneeli kuten Klondikessa.

use klassikot8000::pyramidi::{JATTEEN_NAKYVAT, Lauta, Paikka, RIVEJA, kierroksia};

use crate::pelit::pasianssi::peli::{Nappi, Pasianssi, Vaihe};
use crate::pelit::pasianssi::piirto::{kesto, nappi_listasta};
use crate::yhteinen::kieli::kaanna;
use crate::yhteinen::kortit::{KORTTI_K, KORTTI_L};
use crate::yhteinen::piirto::*;

const YLARIVI_Y: i32 = 54;
const PAKKA_X: i32 = 48;
const PAKKA_ASKEL: i32 = 156;
const KESKI_X: i32 = 700;
const RIVI_ASKEL: i32 = 120;
const VAAKA_ASKEL: i32 = 165;

const OHJEET: [&str; 6] = [
    "pyramidi.sivu_hiiri",
    "pyramidi.sivu_kuningas",
    "pasianssi.sivu_nuolet",
    "pasianssi.sivu_nosta",
    "pasianssi.sivu_aanet",
    "pasianssi.sivu_valikko",
];

const NAPIT: [(Nappi, &str); 5] = [
    (Nappi::Nosta, "pyramidi.nappi_nosta"),
    (Nappi::Peruuta, "pasianssi.nappi_peruuta"),
    (Nappi::Uusi, "pasianssi.nappi_uusi"),
    (Nappi::Alusta, "pasianssi.nappi_alusta"),
    (Nappi::Valikko, "yhteinen.valikkoon"),
];

/// Laudan paikka hiiren alla. `Paikka` on aina kortti, joka on vapaana.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kohta {
    Pakka,
    Paikka(Paikka),
}

fn jatteen_x(syvyys: usize) -> i32 {
    PAKKA_X + PAKKA_ASKEL * (JATTEEN_NAKYVAT - syvyys) as i32
}

fn pyramidin_paikka(rivi: usize, sarake: usize) -> (i32, i32) {
    // Rivit puolittavat vaaka-askeleen, jotta kortti on kahden alemman päällä.
    let x = KESKI_X - KORTTI_L / 2 + (2 * sarake as i32 - rivi as i32) * VAAKA_ASKEL / 2;
    (x, YLARIVI_Y + rivi as i32 * RIVI_ASKEL)
}

/// Kortin vasen yläkulma.
pub fn kortin_paikka(_lauta: &Lauta, paikka: Paikka) -> (i32, i32) {
    match paikka {
        Paikka::Pyramidi { rivi, sarake } => pyramidin_paikka(rivi, sarake),
        Paikka::Jate(syvyys) => (jatteen_x(syvyys), YLARIVI_Y),
    }
}

pub fn kohta_pisteesta(lauta: &Lauta, x: i32, y: i32) -> Option<Kohta> {
    let sisalla =
        |(kx, ky): (i32, i32)| (kx..kx + KORTTI_L).contains(&x) && (ky..ky + KORTTI_K).contains(&y);
    if sisalla((PAKKA_X, YLARIVI_Y)) {
        return Some(Kohta::Pakka);
    }
    for syvyys in 0..JATTEEN_NAKYVAT {
        let paikka = Paikka::Jate(syvyys);
        if sisalla((jatteen_x(syvyys), YLARIVI_Y)) && lauta.pelattava(paikka).is_some() {
            return Some(Kohta::Paikka(paikka));
        }
    }
    // Alimmat rivit ovat päällimmäisinä. Peitetty kortti ei reagoi, eikä sen läpi osu ylemmälle.
    for rivi in (0..RIVEJA).rev() {
        for sarake in 0..=rivi {
            let kohdalla =
                lauta.tila.rivit[rivi][sarake].is_some() && sisalla(pyramidin_paikka(rivi, sarake));
            if kohdalla {
                let paikka = Paikka::Pyramidi { rivi, sarake };
                return lauta.pelattava(paikka).map(|_| Kohta::Paikka(paikka));
            }
        }
    }
    None
}

pub fn nappi_pisteesta(x: i32, y: i32) -> Option<Nappi> {
    nappi_listasta(&NAPIT, x, y)
}

fn kierros_teksti(lauta: &Lauta) -> String {
    match kierroksia(lauta.vaikeus) {
        Some(kierroksia) => format!("{}/{kierroksia}", lauta.kierros()),
        None => lauta.kierros().to_string(),
    }
}

impl Piirtaja<'_> {
    pub fn pyramidi_lauta(&mut self, peli: &Pasianssi) {
        let lauta = &peli.pyramidi;
        self.pyramidi_pakka(peli);
        let pito = peli.pyramidi_pito.filter(|pito| pito.liikkui);
        for rivi in 0..RIVEJA {
            for sarake in 0..=rivi {
                let paikka = Paikka::Pyramidi { rivi, sarake };
                if pito.is_some_and(|pito| pito.lahde == paikka) {
                    continue;
                }
                if let Some(kortti) = lauta.tila.rivit[rivi][sarake] {
                    let (x, y) = pyramidin_paikka(rivi, sarake);
                    self.pelikortti(x, y, kortti);
                }
            }
        }
        // Valinta piirretään vasta nyt: alempi rivi ei saa peittää sen kehystä.
        if let Some(valittu) = peli.pyramidi_valittu {
            let (x, y) = kortin_paikka(lauta, valittu);
            self.korostus(x, y, KORTTI_K, WB_ORANSSI);
        }
        self.pyramidi_kursori(peli);
        self.pyramidi_sivu(peli);
        if let Some(pito) = pito
            && let Some(kortti) = lauta.pelattava(pito.lahde)
        {
            let x = peli.hiiri.0 - pito.tartunta.0;
            let y = peli.hiiri.1 - pito.tartunta.1;
            self.pelikortti(x, y, kortti);
        }
    }

    fn pyramidi_pakka(&mut self, peli: &Pasianssi) {
        let lauta = &peli.pyramidi;
        if lauta.tila.pakka.is_empty() {
            self.korttipaikka(PAKKA_X, YLARIVI_Y);
            if lauta.voi_kierrattaa() {
                let keski = (PAKKA_X + KORTTI_L / 2, YLARIVI_Y + KORTTI_K / 2);
                self.k.ympyra(keski, 30.0, TORNI);
            }
        } else {
            self.kortin_tausta(PAKKA_X, YLARIVI_Y);
        }
        let vedossa = peli
            .pyramidi_pito
            .filter(|pito| pito.liikkui)
            .map(|pito| pito.lahde);
        for syvyys in (0..JATTEEN_NAKYVAT).rev() {
            let paikka = Paikka::Jate(syvyys);
            let x = jatteen_x(syvyys);
            match lauta.pelattava(paikka).filter(|_| vedossa != Some(paikka)) {
                Some(kortti) => self.pelikortti(x, YLARIVI_Y, kortti),
                None => self.korttipaikka(x, YLARIVI_Y),
            }
        }
    }

    fn pyramidi_kursori(&mut self, peli: &Pasianssi) {
        if !peli.kursori_nakyy || peli.vaihe != Vaihe::Pelissa {
            return;
        }
        let (x, y) = match peli.pyramidi_kursorin_kohta() {
            Kohta::Pakka => (PAKKA_X, YLARIVI_Y),
            Kohta::Paikka(paikka) => kortin_paikka(&peli.pyramidi, paikka),
        };
        self.korostus(x, y, KORTTI_K, KELTAINEN);
    }

    fn pyramidi_sivu(&mut self, peli: &Pasianssi) {
        let lauta = &peli.pyramidi;
        let tilastot = [
            (kaanna("yhteinen.pisteet"), peli.pisteet().to_string()),
            (kaanna("yhteinen.siirrot"), lauta.tila.siirrot.to_string()),
            (kaanna("pasianssi.aika"), kesto(peli.peliaika_ms)),
            (kaanna("pasianssi.kierros"), kierros_teksti(lauta)),
            (
                kaanna("pasianssi.pakassa"),
                lauta.tila.pakka.len().to_string(),
            ),
        ];
        let peruutus = lauta.voi_perua();
        self.sivupaneeli(peli, tilastot, &OHJEET, &NAPIT, peruutus);
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
pub use tests::{napin_keskipiste, pakan_keskipiste};
