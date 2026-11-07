use super::*;
use crate::yhteinen::kieli::{kaikilla_kielilla, muotoile};
use crate::yhteinen::syote::Syote;
use klassikot8000::silmukka::Suunta;

pub fn ruudun_keskipiste(ruutu: Ruutu) -> (i32, i32) {
    keski((KENTTA_X, KENTTA_Y), liukuluvuksi(ruutu))
}

#[test]
fn tekstit_ja_kentta_mahtuvat_paikoilleen() {
    kaikilla_kielilla(|| {
        for teksti in [
            kaanna(OHJE).to_string(),
            kaanna(TAUON_OHJE).to_string(),
            kaanna("silmukka.tyhja").to_string(),
            kaanna("silmukka.muna_litistyi").to_string(),
            kaanna("silmukka.kulta_ilmestyi").to_string(),
            muotoile("silmukka.uusi_taso", &[&99]),
            muotoile("silmukka.saalis", &[&19, &999999]),
            muotoile("yhteinen.tulos", &[&999999]),
        ] {
            assert!(teksti.chars().count() <= RIVILLE_MERKKEJA, "{teksti}");
        }
        // Pisin kehu mahtuu kentän leveydelle.
        for avain in [
            "silmukka.kehu_2",
            "silmukka.kehu_3",
            "silmukka.kehu_4",
            "silmukka.kehu_5",
            "silmukka.kehu_7",
            "silmukka.kehu_9",
        ] {
            assert!(
                kaanna(avain).chars().count() as i32 * SUURI.2 < KENTTA_L,
                "{avain}"
            );
        }
        for avain in ["silmukka.taso", "silmukka.pituus"] {
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
fn hiiri_osuu_ruutuun_ja_ulkopuoli_rajataan_kentalle() {
    for ruutu in [(0, 0), (5, 7), (SARAKKEITA - 1, RIVEJA - 1)] {
        let (x, y) = ruudun_keskipiste(ruutu);
        assert_eq!(ruutu_pisteesta(x, y), ruutu);
    }
    assert_eq!(ruutu_pisteesta(-100, -100), (0, 0));
    assert_eq!(ruutu_pisteesta(5000, 5000), (SARAKKEITA - 1, RIVEJA - 1));
}

#[test]
fn suunnat_ovat_kaikki_ruudun_mittaisia() {
    for suunta in [Suunta::Ylos, Suunta::Alas, Suunta::Vasen, Suunta::Oikea] {
        let (x, y) = suunta.siirto();
        assert_eq!(x.abs() + y.abs(), 1);
    }
}

#[test]
fn koko_peli_piirtyy_kaatumatta() {
    let mut kuva = vec![0; LEVEYS * KORKEUS];
    let mut peli = Silmukka::new(3);
    piirra(&peli, &mut kuva);
    peli.kasittele(Syote::Ammu);
    // Kierros kiertää omia jälkiään, joten ennuste, silmukat ja kuolema tulevat kaikki vastaan.
    let nuolet = [(1, 0), (0, -1), (-1, 0), (0, 1)];
    for i in 0..6000 {
        peli.edista(16);
        piirra(&peli, &mut kuva);
        if i % 12 == 0 {
            let (rivi, sarake) = nuolet[i / 12 % 4];
            peli.kasittele(Syote::Nuoli(rivi, sarake));
        }
    }
    peli.vaihe = Vaihe::Tauko;
    piirra(&peli, &mut kuva);
    peli.vaihe = Vaihe::Tappio;
    piirra(&peli, &mut kuva);
    peli.vaihe = Vaihe::Nimi;
    piirra(&peli, &mut kuva);
}
