use super::*;
use crate::yhteinen::kieli::kaikilla_kielilla;

pub fn napin_keskipiste(nappi: Nappi) -> (i32, i32) {
    let (_, teksti, x) = NAPIT.iter().find(|n| n.0 == nappi).copied().unwrap();
    (x + napin_leveys(teksti) / 2, NAPPI_Y + NAPPI_K / 2)
}

pub fn uuden_pelin_keskipiste() -> (i32, i32) {
    (
        uuden_pelin_x() + napin_leveys(UUSI_PELI_NAPPI) / 2,
        UUSI_PELI_Y + NAPPI_K / 2,
    )
}

#[test]
fn laudan_neljannekset_antavat_suunnan_ja_ulkopuoli_ei() {
    for suunta in [Suunta::Ylos, Suunta::Alas, Suunta::Vasen, Suunta::Oikea] {
        let (x, y) = laudan_reuna(suunta);
        assert_eq!(suunta_pisteesta(x, y), Some(suunta));
    }
    assert_eq!(suunta_pisteesta(RUUDUT_X - 1, RUUDUT_Y), None);
    assert_eq!(suunta_pisteesta(RUUDUT_X, RUUDUT_Y + LAUDAN_SIVU), None);
}

#[test]
fn napit_eivat_mene_paallekkain_ja_mahtuvat_ruudulle() {
    kaikilla_kielilla(|| {
        for pari in NAPIT.windows(2) {
            assert!(pari[0].2 + napin_leveys(pari[0].1) < pari[1].2);
        }
        let (_, teksti, x) = NAPIT[2];
        assert!(x + napin_leveys(teksti) < LEVEYS as i32);
    });
}

#[test]
fn numerot_mahtuvat_laattaan_ja_tekstit_paneeliin() {
    for pituus in 1..=6 {
        let (l, _, askel) = numeron_koko(pituus);
        assert!(
            pituus as i32 * askel - (askel - l) <= LAATTA - 12,
            "{pituus}"
        );
    }
    kaikilla_kielilla(|| {
        for avain in [
            OHJE,
            EI_LIIKU,
            NIMI_OHJE,
            "luvut.jatketaan",
            "luvut.jatketaan_tallennettua",
        ] {
            let teksti = kaanna(avain);
            assert!(teksti.chars().count() <= RIVILLE_MERKKEJA, "{teksti}");
        }
        let siirto = muotoile("luvut.siirto_pisteet", &[&99999, &99999]);
        assert!(siirto.chars().count() <= RIVILLE_MERKKEJA, "{siirto}");
        assert!(kaanna(JATKA).chars().count() <= 36);
        // Nappi ei saa osua sen vieressä oleviin riveihin.
        for avain in [UUSI_PELI, "yhteinen.aanet_kytkin", "yhteinen.takaisin"] {
            let leveys = kaanna(avain).chars().count() as i32 * MERKKI;
            assert!(48 + leveys < uuden_pelin_x(), "{avain}");
        }
        assert!(kaanna(VOITTO).chars().count() as i32 * TULOS.2 < LEVEYS as i32 - TEKSTI_X);
    });
    // Tallennetun pelin rivi tulee näppäinohjeiden jälkeen.
    let rivit: Vec<&str> = ALKU_RIVIT.iter().copied().chain([UUSI_PELI]).collect();
    alkuohje_mahtuu(&ALKU_OSAT, &rivit, ALOITUS_Y);
}

#[test]
fn koko_peli_piirtyy_kaatumatta_animaatioiden_aikana() {
    let mut kuva = vec![0; LEVEYS * KORKEUS];
    let mut luvut = Luvut::new(3);
    piirra(&luvut, &mut kuva);
    luvut.vaihe = Vaihe::Pelissa;
    let suunnat = [Suunta::Vasen, Suunta::Ylos, Suunta::Oikea, Suunta::Alas];
    for i in 0..400 {
        luvut.hiiren_suunta = Some(suunnat[i % 4]);
        luvut.siirra(suunnat[i % 4]);
        for _ in 0..3 {
            luvut.edista(70);
            piirra(&luvut, &mut kuva);
        }
    }
    luvut.vaihe = Vaihe::Nimi;
    piirra(&luvut, &mut kuva);
}
