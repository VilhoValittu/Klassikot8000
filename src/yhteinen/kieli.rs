//! Pelin tekstit suomeksi ja englanniksi. Tekstit luetaan käynnistyksessä exen vieressä olevasta
//! tiedostosta `kielet.txt`. Sama tiedosto on upotettu exeen, joten peli toimii ilman sitäkin,
//! ja muokatusta tiedostosta puuttuvat tekstit tulevat upotetusta.

use std::cell::Cell;
use std::collections::HashMap;
use std::fmt::Display;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

const TIEDOSTO: &str = "kielet.txt";
const UPOTETUT: &str = include_str!("../../kielet.txt");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Kieli {
    Suomi,
    #[default]
    Englanti,
}

impl Kieli {
    #[cfg(debug_assertions)]
    pub const KAIKKI: [Kieli; 2] = [Kieli::Suomi, Kieli::Englanti];

    pub fn koodi(self) -> &'static str {
        match self {
            Kieli::Suomi => "fi",
            Kieli::Englanti => "en",
        }
    }

    pub fn koodista(koodi: &str) -> Option<Self> {
        match koodi {
            "fi" => Some(Kieli::Suomi),
            "en" => Some(Kieli::Englanti),
            _ => None,
        }
    }

    pub fn toinen(self) -> Self {
        match self {
            Kieli::Suomi => Kieli::Englanti,
            Kieli::Englanti => Kieli::Suomi,
        }
    }
}

type Taulu = HashMap<&'static str, &'static str>;

static TEKSTIT: OnceLock<[Taulu; 2]> = OnceLock::new();

thread_local! {
    // Tekstejä käyttää vain ikkunan säie, ja näin testit voivat vaihtaa kieltä häiritsemättä toisiaan.
    // Pelin oletus on englanti, mutta testien odotetut tekstit ovat suomeksi.
    static NYKYINEN: Cell<Kieli> = const {
        Cell::new(if cfg!(test) { Kieli::Suomi } else { Kieli::Englanti })
    };
}

/// Lisää tekstit tauluihin. Kelvottomat rivit ohitetaan, jottei muokattu tiedosto estä pelaamista.
fn lisaa_tekstit(teksti: &'static str, taulut: &mut [Taulu; 2]) {
    let mut kieli = None;
    // Muistio voi lisätä tiedoston alkuun BOM-merkin.
    for rivi in teksti.trim_start_matches('\u{feff}').lines() {
        let rivi = rivi.trim();
        if rivi.is_empty() || rivi.starts_with('#') {
            continue;
        }
        if let Some(koodi) = rivi.strip_prefix('[').and_then(|r| r.strip_suffix(']')) {
            kieli = Kieli::koodista(koodi);
        } else if let (Some(kieli), Some((avain, arvo))) = (kieli, rivi.split_once('=')) {
            taulut[kieli as usize].insert(avain.trim(), arvo.trim());
        }
    }
}

fn rakenna(tiedostosta: Option<&'static str>) -> [Taulu; 2] {
    let mut taulut = [Taulu::new(), Taulu::new()];
    lisaa_tekstit(UPOTETUT, &mut taulut);
    if let Some(teksti) = tiedostosta {
        lisaa_tekstit(teksti, &mut taulut);
    }
    taulut
}

pub fn oletuspolku() -> Option<PathBuf> {
    Some(std::env::current_exe().ok()?.parent()?.join(TIEDOSTO))
}

/// Luetaan kerran ennen ensimmäistä tekstiä, joten tiedoston muisti saa jäädä varatuksi.
pub fn lue_tiedosto(polku: &Path) {
    let teksti = fs::read_to_string(polku).ok();
    TEKSTIT.get_or_init(|| rakenna(teksti.map(|t| &*t.leak())));
}

pub fn nykyinen() -> Kieli {
    NYKYINEN.get()
}

pub fn aseta(kieli: Kieli) {
    NYKYINEN.set(kieli);
}

pub fn kaanna(avain: &str) -> &'static str {
    let taulut = TEKSTIT.get_or_init(|| rakenna(None));
    let teksti = taulut[nykyinen() as usize].get(avain).copied();
    debug_assert!(teksti.is_some(), "kielet.txt: {avain} puuttuu");
    teksti.unwrap_or("?")
}

/// Sijoittaa arvot järjestyksessä tekstin `{}`-kohtiin.
pub fn muotoile(avain: &str, arvot: &[&dyn Display]) -> String {
    let mut teksti = String::new();
    for (i, osa) in kaanna(avain).split("{}").enumerate() {
        if let Some(arvo) = i.checked_sub(1).and_then(|i| arvot.get(i)) {
            teksti += &arvo.to_string();
        }
        teksti += osa;
    }
    teksti
}

#[cfg(test)]
mod tests;
#[cfg(test)]
pub use tests::kaikilla_kielilla;
