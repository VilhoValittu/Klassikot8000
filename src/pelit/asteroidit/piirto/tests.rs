use super::*;
use crate::yhteinen::kieli::{kaikilla_kielilla, muotoile};
use crate::yhteinen::syote::Syote;

#[test]
fn tekstit_ja_kentta_mahtuvat_paikoilleen() {
    kaikilla_kielilla(|| {
        for teksti in [
            kaanna(OHJE).to_string(),
            kaanna(TAUON_OHJE).to_string(),
            muotoile("asteroidit.alus_tuhoutui", &[&99]),
            muotoile("yhteinen.tulos", &[&999999]),
        ] {
            assert!(teksti.chars().count() <= RIVILLE_MERKKEJA, "{teksti}");
        }
        for avain in ["asteroidit.aalto", "asteroidit.alukset"] {
            let leveys = kaanna(avain).chars().count() as i32 * OTSIKKO.2;
            assert!(SARAKE_X + leveys < LEVEYS as i32, "{avain}");
        }
    });
    alkuohje_mahtuu(&ALKU_OSAT, &ALKU_RIVIT, ALOITUS_Y);
    const { assert!((2 + 8 + 1 + 6) * PARHAAT_KOKO.2 <= PARHAAT_L) };
    const { assert!(PARHAAT_Y + 30 + 3 * 26 < PANEELI_Y) };
    const { assert!(KENTTA_Y + KENTTA_K < PANEELI_Y) };
    assert!(SARAKE_X + "PISTEET".len() as i32 * OTSIKKO.2 < LEVEYS as i32);
    assert!(SARAKE_X + 6 * OTSIKKO.2 < LEVEYS as i32);
}

#[test]
fn laidan_yli_ulottuva_kappale_piirtyy_molemmille_puolille() {
    assert_eq!(kopiot((300.0, 300.0), 10.0).count(), 1);
    assert_eq!(kopiot((5.0, 300.0), 10.0).count(), 2);
    assert_eq!(kopiot((5.0, 5.0), 10.0).count(), 4);
}

#[test]
fn koko_peli_piirtyy_kaatumatta() {
    let mut kuva = vec![0; LEVEYS * KORKEUS];
    let mut peli = Asteroidit::new(3);
    piirra(&peli, &mut kuva);
    peli.kasittele(Syote::Ammu);
    // Aluksen paikallaan pysyessä kivet osuvat lopulta, ja peli päättyy.
    for i in 0..6000 {
        peli.edista(16);
        piirra(&peli, &mut kuva);
        if i % 10 == 0 {
            peli.kasittele(Syote::Ammu);
        }
    }
    peli.vaihe = Vaihe::Tauko;
    piirra(&peli, &mut kuva);
    peli.vaihe = Vaihe::Tappio;
    piirra(&peli, &mut kuva);
    peli.vaihe = Vaihe::Nimi;
    piirra(&peli, &mut kuva);
}
