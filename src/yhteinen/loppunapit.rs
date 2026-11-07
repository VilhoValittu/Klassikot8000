//! Pelin lopun napit "Uudestaan" ja "Valikkoon" näppäimineen. Ne ovat samat kaikissa
//! peleissä, joiden loppuruudulla ei ole muita valintoja.

use super::piirto::{MERKKI, NAPPI_X, NAPPI_Y, Piirtaja, VALKOINEN, WB_SININEN, rivin_nappi};
use super::syote::Syote;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Loppunappi {
    Uudestaan,
    Valikko,
}

const NAPIT: [(Loppunappi, &str, i32); 2] = [
    (Loppunappi::Uudestaan, "yhteinen.uudestaan", NAPPI_X),
    (
        Loppunappi::Valikko,
        "yhteinen.valikkoon",
        NAPPI_X + 13 * MERKKI + 48,
    ),
];

pub fn nappi_pisteesta(x: i32, y: i32) -> Option<Loppunappi> {
    rivin_nappi(&NAPIT, NAPPI_Y, x, y)
}

/// K aloittaa uudestaan, E palaa valikkoon, ja klikkaus valitsee napin.
pub fn valinta(syote: Syote) -> Option<Loppunappi> {
    match syote {
        Syote::Kirjain('K') => Some(Loppunappi::Uudestaan),
        Syote::Kirjain('E') => Some(Loppunappi::Valikko),
        Syote::Klikkaus(x, y) => nappi_pisteesta(x, y),
        _ => None,
    }
}

impl Piirtaja<'_> {
    pub fn loppunapit(&mut self) {
        for (_, teksti, x) in NAPIT {
            self.nappi(x, NAPPI_Y, teksti, VALKOINEN, WB_SININEN);
        }
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
pub use tests::napin_keskipiste;
