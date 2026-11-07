use super::*;
use crate::yhteinen::kieli::{kaikilla_kielilla, muotoile};

pub fn napin_keskipiste(nappi: Nappi) -> (i32, i32) {
    let (_, teksti, x) = NAPIT.iter().find(|n| n.0 == nappi).copied().unwrap();
    (x + napin_leveys(teksti) / 2, NAPPI_Y + NAPPI_K / 2)
}

#[test]
fn napit_loytyvat_eivatka_mene_paallekkain() {
    for (nappi, ..) in NAPIT {
        let (x, y) = napin_keskipiste(nappi);
        assert_eq!(nappi_pisteesta(x, y), Some(nappi));
    }
    assert_eq!(nappi_pisteesta(0, 0), None);
    kaikilla_kielilla(|| assert!(NAPIT[0].2 + napin_leveys(NAPIT[0].1) < NAPIT[1].2));
}

#[test]
fn tekstit_ja_kentta_mahtuvat_paikoilleen() {
    kaikilla_kielilla(|| {
        for teksti in [
            kaanna(OHJE).to_string(),
            kaanna(LAUKAISU).to_string(),
            kaanna(TAUON_OHJE).to_string(),
            kaanna("tiilirikko.uusi_pallo").to_string(),
            kaanna("tiilirikko.superpallo").to_string(),
            kaanna("tiilirikko.toinen_meni").to_string(),
            muotoile("tiilirikko.pallo_meni", &[&3]),
            muotoile("tiilirikko.kentta_lapi", &[&5000]),
            muotoile("tiilirikko.loppubonus", &[&5000, &999999]),
            muotoile("yhteinen.tulos", &[&999999]),
        ] {
            assert!(teksti.chars().count() <= RIVILLE_MERKKEJA, "{teksti}");
        }
        let voitto = kaanna(VOITTO).chars().count() as i32;
        assert!(voitto * TULOS.2 < LEVEYS as i32 - TEKSTI_X);
        for avain in ["tiilirikko.kentta", "tiilirikko.pallot"] {
            let leveys = kaanna(avain).chars().count() as i32 * OTSIKKO.2;
            assert!(SARAKE_X + leveys < LEVEYS as i32, "{avain}");
        }
        let bonus = kaanna("tiilirikko.aikabonus").chars().count() as i32 * TULOS.2;
        assert!(SARAKE_X + bonus < LEVEYS as i32);
    });
    alkuohje_mahtuu(&ALKU_OSAT, &ALKU_RIVIT, ALOITUS_Y);
    const { assert!((2 + 8 + 1 + 6) * PARHAAT_KOKO.2 <= PARHAAT_L) };
    const { assert!(PARHAAT_Y + 30 + 3 * 26 < PANEELI_Y) };
    const { assert!(KENTTA_Y + KENTTA_K < PANEELI_Y) };
    assert!(SARAKE_X + "PISTEET".len() as i32 * OTSIKKO.2 < LEVEYS as i32);
}

#[test]
fn koko_peli_piirtyy_kaatumatta() {
    let mut kuva = vec![0; LEVEYS * KORKEUS];
    let mut peli = Tiilirikko::new(3);
    piirra(&peli, &mut kuva);
    peli.kasittele(crate::yhteinen::syote::Syote::Ammu);
    peli.kasittele(crate::yhteinen::syote::Syote::Ammu);
    // Mailaa ei liikuteta, joten pallot putoavat kentän ali ja peli päättyy.
    for _ in 0..3000 {
        peli.edista(16);
        piirra(&peli, &mut kuva);
        if peli.lauta.tila == Tila::Kiinni {
            peli.kasittele(crate::yhteinen::syote::Syote::Ammu);
        }
    }
    peli.vaihe = Vaihe::Tauko;
    piirra(&peli, &mut kuva);
    peli.vaihe = Vaihe::Voitto;
    piirra(&peli, &mut kuva);
    peli.vaihe = Vaihe::Nimi;
    piirra(&peli, &mut kuva);
}
