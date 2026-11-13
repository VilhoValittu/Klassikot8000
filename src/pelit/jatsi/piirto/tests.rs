use super::*;
use crate::yhteinen::kieli::kaikilla_kielilla;

pub fn rivin_keskipiste(rivi: Rivi) -> (i32, i32) {
    (kirja_x(PELAAJIA) + NIMI_L / 2, rivin_y(rivi) + RIVI_K / 2)
}

pub fn nopan_keskipiste(noppa: usize) -> (i32, i32) {
    (nopan_x(noppa) + NOPPA / 2, NOPAT_Y + NOPPA / 2)
}

pub fn heiton_keskipiste() -> (i32, i32) {
    (HEITTO_X + HEITTO_L / 2, HEITTO_Y + HEITTO_K / 2)
}

pub fn istujan_keskipiste(paikka: usize) -> (i32, i32) {
    (ISTUJA_X + ISTUJA_L / 2, paikan_y(paikka) + NAPPI_K / 2)
}

pub fn loppunapin_keskipiste(i: usize) -> (i32, i32) {
    let leveys = napin_leveys(LOPPUNAPIT[i].1);
    (
        SIVU_TEKSTI_X + leveys / 2,
        NAPIT_Y + i as i32 * NAPPI_ASKEL + NAPPI_K / 2,
    )
}

#[test]
fn osumat_loytavat_rivit_nopat_ja_napit() {
    for rivi in Rivi::KAIKKI {
        let (x, y) = rivin_keskipiste(rivi);
        assert_eq!(rivi_pisteesta(PELAAJIA, x, y), Some(rivi));
    }
    // Välisumma ja bonus ovat rivien välissä, eikä niitä voi kirjata.
    let valisumma = KIRJA_Y + OTSIKKO_K + 6 * RIVI_K + 3;
    assert_eq!(
        rivi_pisteesta(PELAAJIA, kirja_x(PELAAJIA) + 6, valisumma),
        None
    );
    let oikea = kirja_x(1) + kirjan_leveys(1);
    assert!(rivi_pisteesta(1, oikea - 1, rivin_y(Rivi::Pari)).is_some());
    assert_eq!(rivi_pisteesta(1, oikea, rivin_y(Rivi::Pari)), None);
    for i in 0..NOPPIA {
        let (x, y) = nopan_keskipiste(i);
        assert_eq!(noppa_pisteesta(x, y), Some(i));
        assert_eq!(
            noppa_pisteesta(x, NOPAT_Y - NOUSU),
            Some(i),
            "noussut noppa"
        );
        assert!(!heitto_pisteessa(x, y));
    }
    let (x, y) = heiton_keskipiste();
    assert!(heitto_pisteessa(x, y));
    assert_eq!(noppa_pisteesta(x, y), None);
    for i in 0..PELAAJIA {
        let (x, y) = istujan_keskipiste(i);
        assert_eq!(istuja_pisteesta(x, y), Some(i));
        assert_eq!(nimi_pisteesta(PAIKAN_NIMI_X + 6, y), Some(i));
    }
    for (i, (nappi, _)) in LOPPUNAPIT.iter().enumerate() {
        let (x, y) = loppunapin_keskipiste(i);
        assert_eq!(loppunappi_pisteesta(x, y), Some(*nappi));
    }
}

#[test]
fn poytakirja_ja_sivupaneeli_mahtuvat_ruudulle() {
    for n in 1..=PELAAJIA {
        assert!(kirja_x(n) >= KIRJA_X);
        assert!(kirja_x(n) + kirjan_leveys(n) + 18 < SIVU_X - 12, "{n}");
    }
    const { assert!(KIRJA_Y + KIRJAN_K + 12 < RUUDUN_KORKEUS as i32) };
    assert!(nopan_x(NOPPIA - 1) + NOPPA < SIVU_X + SIVU_L);
    const { assert!(OHJE_Y + 8 * RIVIN_KORKEUS < NAPIT_Y) };
    const { assert!(OHJE_Y + PARHAAT_OTSIKKO + 3 * parhaiden_askel(TULOS, Some(PVM)) < NAPIT_Y) };
    assert_eq!(
        rivin_y(Rivi::Jatsi) + RIVI_K,
        KIRJA_Y + OTSIKKO_K + 17 * RIVI_K
    );
    const { assert!(PAIKAN_OHJE_Y + RIVIN_KORKEUS <= TASO_Y) };
}

#[test]
fn tekstit_mahtuvat_paikoilleen() {
    kaikilla_kielilla(|| {
        for rivi in Rivi::KAIKKI {
            let nimi = rivin_nimi(rivi);
            assert!(
                nimi.chars().count() as i32 * MERKKI + 12 <= NIMI_L,
                "{nimi}"
            );
        }
        for avain in ["jatsi.ylaosa", "jatsi.bonus", "jatsi.summa"] {
            assert!(kaanna(avain).chars().count() as i32 * MERKKI + 12 <= NIMI_L);
        }
        for istuja in [Istuja::Ihminen, Istuja::Kone, Istuja::Tyhja] {
            let nimi = istujan_nimi(istuja);
            assert!(
                nimi.chars().count() as i32 * MERKKI + 12 <= ISTUJA_L,
                "{nimi}"
            );
        }
        assert!(Jatsi::oletusnimi(3).chars().count() <= 8);
        let heitto = muotoile("jatsi.heita_nappi", &[&3]);
        assert!(
            heitto.chars().count() as i32 * MERKKI + 24 <= HEITTO_L,
            "{heitto}"
        );
        // Sivupaneelin ohjeet rivitetään: niille on varattu kahdeksan riviä.
        let ohjeen_rivit: usize = [
            "jatsi.ohje_heitto",
            "jatsi.ohje_kirjaus",
            "yhteinen.takaisin",
        ]
        .iter()
        .map(|avain| rivita(kaanna(avain), SIVU_MERKKEJA).len())
        .sum();
        assert!(ohjeen_rivit <= 7);
        for avain in ["jatsi.vuorossa_otsikko", "jatsi.listalle", "jatsi.hoputa"] {
            let teksti = kaanna(avain);
            assert!(rivita(teksti, SIVU_MERKKEJA).len() <= 2, "{teksti}");
        }
        for avain in [
            "jatsi.paikka_ohje",
            "jatsi.jatka",
            "jatsi.kesken",
            "jatsi.pelaajat",
        ] {
            let teksti = kaanna(avain);
            assert!(teksti.chars().count() <= ALKU_MERKIT, "{teksti}");
        }
        assert!(rivita(kaanna("jatsi.ohje_1"), ALKU_MERKIT).len() <= 2);
        // Pisin viesti: kirjaus, bonus ja seuraavan vuoro.
        let viesti = format!(
            "{} {} {}",
            muotoile(
                "jatsi.kirjasi",
                &[&"PELAAJA4", &rivin_nimi(Rivi::KaksiParia), &22]
            ),
            muotoile("jatsi.bonus_viesti", &[&50]),
            muotoile("jatsi.vuorossa", &[&"PELAAJA4"]),
        );
        assert!(
            rivita(&viesti, SIVU_MERKKEJA).len() <= VIESTIN_RIVEJA,
            "{viesti}"
        );
        let viivaus = muotoile("jatsi.viivataanko", &[&rivin_nimi(Rivi::PieniSuora)]);
        assert!(rivita(&viivaus, SIVU_MERKKEJA).len() <= VIESTIN_RIVEJA);
        for otsikko in [
            muotoile("jatsi.voitti", &[&"PELAAJA4"]),
            kaanna("jatsi.havisit").to_string(),
            kaanna("yhteinen.peli_paattyi").to_string(),
        ] {
            assert!(
                otsikko.chars().count() as i32 * OTSIKKO.2 < SIVU_L - 36,
                "{otsikko}"
            );
        }
        let juhla = kaanna("jatsi.juhla").chars().count() as i32 * SUURI.2 * 2;
        assert!(juhla < kirjan_leveys(1));
        let kehote = kaanna("jatsi.uusi_peli").chars().count() as i32 * MERKKI;
        assert!(48 + kehote < uuden_pelin_x());
    });
}
