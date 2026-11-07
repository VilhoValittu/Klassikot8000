use super::*;
use crate::yhteinen::kieli::kaikilla_kielilla;
use crate::yhteinen::piirto::{NAPPI_K, napin_leveys};

pub fn napin_keskipiste(nappi: Loppunappi) -> (i32, i32) {
    let (_, teksti, x) = NAPIT.iter().find(|n| n.0 == nappi).copied().unwrap();
    (x + napin_leveys(teksti) / 2, NAPPI_Y + NAPPI_K / 2)
}

#[test]
fn napit_loytyvat_eivatka_mene_paallekkain() {
    for (nappi, ..) in NAPIT {
        let (x, y) = napin_keskipiste(nappi);
        assert_eq!(nappi_pisteesta(x, y), Some(nappi));
        assert_eq!(valinta(Syote::Klikkaus(x, y)), Some(nappi));
    }
    assert_eq!(nappi_pisteesta(0, 0), None);
    kaikilla_kielilla(|| assert!(NAPIT[0].2 + napin_leveys(NAPIT[0].1) < NAPIT[1].2));
}

#[test]
fn kirjaimet_valitsevat_napin() {
    assert_eq!(valinta(Syote::Kirjain('K')), Some(Loppunappi::Uudestaan));
    assert_eq!(valinta(Syote::Kirjain('E')), Some(Loppunappi::Valikko));
    assert_eq!(valinta(Syote::Kirjain('X')), None);
    assert_eq!(valinta(Syote::Ammu), None);
}
