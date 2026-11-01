//! Tavallinen 52 kortin pakka, jota korttipelit käyttävät.

use crate::Arpa;

pub const MAITA: usize = 4;
pub const KUNINGAS: u8 = 13;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Maa {
    Hertta,
    Ruutu,
    Risti,
    Pata,
}

impl Maa {
    pub const KAIKKI: [Maa; MAITA] = [Maa::Hertta, Maa::Ruutu, Maa::Risti, Maa::Pata];

    pub fn on_punainen(self) -> bool {
        matches!(self, Maa::Hertta | Maa::Ruutu)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Kortti {
    pub maa: Maa,
    /// Ässä on 1 ja kuningas 13.
    pub arvo: u8,
}

/// Sekoitetut kortit `pakkoja` pakasta. Päällimmäinen kortti on viimeisenä.
pub fn sekoitettu(pakkoja: usize, arpa: &mut Arpa) -> Vec<Kortti> {
    let mut kortit: Vec<Kortti> = (0..pakkoja)
        .flat_map(|_| Maa::KAIKKI)
        .flat_map(|maa| (1..=KUNINGAS).map(move |arvo| Kortti { maa, arvo }))
        .collect();
    sekoita(&mut kortit, arpa);
    kortit
}

pub fn sekoita(kortit: &mut [Kortti], arpa: &mut Arpa) {
    for i in (1..kortit.len()).rev() {
        kortit.swap(i, arpa.alle(i + 1));
    }
}
