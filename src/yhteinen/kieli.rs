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
    #[default]
    Suomi,
    Englanti,
}

impl Kieli {
    #[cfg(test)]
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
    static NYKYINEN: Cell<Kieli> = const { Cell::new(Kieli::Suomi) };
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

/// Ajaa tarkistuksen kummallakin kielellä ja palauttaa suomen.
#[cfg(test)]
pub fn kaikilla_kielilla(mut tarkistus: impl FnMut()) {
    for kieli in Kieli::KAIKKI {
        aseta(kieli);
        tarkistus();
    }
    aseta(Kieli::Suomi);
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::yhteinen::piirto::{LEVEYS, SUURI};
    use crate::yhteinen::tulokset::{NIMEN_PITUUS, tulos_tekstista};

    fn upotetut() -> [Taulu; 2] {
        rakenna(None)
    }

    fn lahdekoodi(kansio: &Path, koodi: &mut String) {
        for tiedosto in fs::read_dir(kansio).unwrap() {
            let polku = tiedosto.unwrap().path();
            if polku.is_dir() {
                lahdekoodi(&polku, koodi);
            } else if polku.extension().is_some_and(|e| e == "rs") {
                *koodi += &fs::read_to_string(&polku).unwrap();
            }
        }
    }

    #[test]
    fn kielilla_on_samat_avaimet_ja_arvokohdat() {
        let [suomi, englanti] = upotetut();
        assert!(!suomi.is_empty());
        for (avain, teksti) in &suomi {
            let toinen = englanti.get(avain).unwrap_or_else(|| panic!("en: {avain}"));
            assert_eq!(
                teksti.matches("{}").count(),
                toinen.matches("{}").count(),
                "{avain}"
            );
        }
        for avain in englanti.keys() {
            assert!(suomi.contains_key(avain), "fi: {avain}");
        }
    }

    #[test]
    fn koodin_avaimet_loytyvat_ja_jokaista_tekstia_kaytetaan() {
        let [suomi, _] = upotetut();
        let mut koodi = String::new();
        lahdekoodi(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
            &mut koodi,
        );
        let etuliite = |avain: &str| avain.split('.').next().unwrap_or("").to_string();
        let etuliitteet: Vec<String> = suomi.keys().map(|a| etuliite(a)).collect();
        for pala in koodi.split('"') {
            let avaimen_muoto = pala.contains('.')
                && pala
                    .chars()
                    .all(|m| m == '.' || m == '_' || m.is_ascii_lowercase() || m.is_ascii_digit());
            if avaimen_muoto && etuliitteet.contains(&etuliite(pala)) {
                assert!(suomi.contains_key(pala), "kielet.txt: {pala} puuttuu");
            }
        }
        for avain in suomi.keys() {
            assert!(
                koodi.contains(&format!("\"{avain}\"")),
                "kielet.txt: {avain} on turha"
            );
        }
    }

    #[test]
    fn tekstit_piirtyvat_fonteilla() {
        for taulu in upotetut() {
            for (avain, teksti) in &taulu {
                let teksti = teksti.replace("{}", "");
                for merkki in teksti.chars() {
                    let tunnettu = matches!(merkki, ' '..='z' | 'Ä' | 'Ö' | 'ä' | 'ö');
                    assert!(tunnettu, "{avain}: {merkki}");
                }
            }
        }
    }

    #[test]
    fn bannerit_mahtuvat_kuvaan() {
        for taulu in upotetut() {
            for (avain, teksti) in taulu.iter().filter(|(a, _)| a.ends_with(".banneri")) {
                let leveys = teksti.chars().count() as i32 * SUURI.2;
                assert!(leveys <= LEVEYS as i32 - 48, "{avain}");
            }
        }
    }

    #[test]
    fn oletusnimi_kelpaa_tulokseksi() {
        kaikilla_kielilla(|| {
            let nimi = kaanna("yhteinen.oletusnimi");
            assert!(nimi.chars().count() <= NIMEN_PITUUS);
            assert!(tulos_tekstista(&format!("1 {nimi}")).is_some());
        });
    }

    #[test]
    fn tiedosto_korvaa_upotetun_ja_puuttuva_teksti_sailyy() {
        let teksti = "\u{feff}# kommentti\r\n[en]\r\npeli.luvut = Numbers = 2\r\nroskaa\r\n[xx]\r\npeli.pasianssi = X\r\n";
        let [suomi, englanti] = rakenna(Some(teksti));
        let [_, alkuperainen] = upotetut();
        assert_eq!(englanti["peli.luvut"], "Numbers = 2");
        assert_eq!(englanti["peli.pasianssi"], alkuperainen["peli.pasianssi"]);
        assert_eq!(suomi["peli.luvut"], "2048");
    }

    #[test]
    fn kielen_vaihto_ja_arvojen_sijoitus() {
        assert_eq!(nykyinen(), Kieli::Suomi);
        assert_eq!(kaanna("peli.pasianssi"), "Pasianssi");
        assert_eq!(muotoile("laivanupotus.upotettu", &[&1, &5]), "Upotettu 1/5");
        aseta(nykyinen().toinen());
        assert_eq!(kaanna("peli.pasianssi"), "Solitaire");
        assert_eq!(muotoile("laivanupotus.upotettu", &[&1, &5]), "Sunk 1/5");
        aseta(Kieli::Suomi);
        assert_eq!(
            Kieli::koodista(Kieli::Englanti.koodi()),
            Some(Kieli::Englanti)
        );
        assert_eq!(Kieli::koodista("sv"), None);
    }
}
