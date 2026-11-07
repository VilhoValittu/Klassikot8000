use super::*;
use crate::yhteinen::kieli::{kaikilla_kielilla, muotoile};
use crate::yhteinen::syote::Syote;
use klassikot8000::Arpa;
use klassikot8000::majakka::{Ohjaus, RENKAITA, Vuorokausi};

pub fn napin_keskipiste(nappi: Nappi) -> (i32, i32) {
    let (_, teksti, x) = NAPIT.iter().find(|n| n.0 == nappi).copied().unwrap();
    (x + napin_leveys(teksti) / 2, NAPPI_Y + NAPPI_K / 2)
}

pub fn vaikeuden_keskipiste(vaikeus: Vaikeus) -> (i32, i32) {
    let (_, teksti, x) = TASOT.iter().find(|t| t.0 == vaikeus).copied().unwrap();
    (x + napin_leveys(teksti) / 2, TASO_Y + NAPPI_K / 2)
}

#[test]
fn napit_ja_tasot_loytyvat_eivatka_mene_paallekkain() {
    for (nappi, ..) in NAPIT {
        let (x, y) = napin_keskipiste(nappi);
        assert_eq!(nappi_pisteesta(x, y), Some(nappi));
    }
    assert_eq!(nappi_pisteesta(0, 0), None);
    for taso in Vaikeus::KAIKKI {
        let (x, y) = vaikeuden_keskipiste(taso);
        assert_eq!(vaikeus_pisteesta(x, y), Some(taso));
    }
    kaikilla_kielilla(|| {
        assert!(NAPIT[0].2 + napin_leveys(NAPIT[0].1) < NAPIT[1].2);
        for pari in TASOT.windows(2) {
            assert!(pari[0].2 + napin_leveys(pari[0].1) < pari[1].2);
        }
        let viimeinen = TASOT[2];
        assert!(viimeinen.2 + napin_leveys(viimeinen.1) < LEVEYS as i32 - 24);
    });
}

#[test]
fn tekstit_mahtuvat_paikoilleen() {
    kaikilla_kielilla(|| {
        let pisin = kaanna("majakka.kalastaja");
        for teksti in [
            kaanna(OHJE).to_string(),
            kaanna(TAUON_OHJE).to_string(),
            muotoile("majakka.satamassa", &[&pisin, &750]),
            muotoile("majakka.karille", &[&kaanna("majakka.rahtilaiva"), &2]),
            muotoile("majakka.karille", &[&pisin, &2]),
            muotoile("majakka.eksyi", &[&pisin]),
            muotoile("majakka.eksyi", &[&kaanna("majakka.rahtilaiva")]),
            muotoile("majakka.kiinni", &[&300]),
            kaanna("majakka.karkasi").to_string(),
            kaanna("majakka.puuska").to_string(),
            muotoile("majakka.aamu", &[&99]),
            muotoile("majakka.bonus", &[&99999]),
            muotoile("majakka.uusi_yo", &[&999]),
            muotoile("yhteinen.tulos", &[&999999]),
        ] {
            assert!(teksti.chars().count() <= RIVILLE_MERKKEJA, "{teksti}");
        }
        for avain in [
            "yhteinen.pisteet",
            "yhteinen.paras",
            "majakka.yo_otsikko",
            "majakka.ketju",
            "majakka.renkaat",
            "majakka.torvi",
        ] {
            let leveys = kaanna(avain).chars().count() as i32 * OTSIKKO.2;
            assert!(leveys <= PARHAAT_L, "{avain}");
        }
    });
    let rivit: Vec<&str> = ALKU_RIVIT
        .iter()
        .copied()
        .chain(Vaikeus::KAIKKI.map(tason_kuvaus).into_iter().take(1))
        .collect();
    alkuohje_mahtuu(&ALKU_OSAT, &rivit, TASO_Y);
    kaikilla_kielilla(|| {
        for taso in Vaikeus::KAIKKI {
            assert!(kaanna(tason_kuvaus(taso)).chars().count() <= 36);
        }
    });
    const { assert!((2 + 8 + 1 + 6) * PARHAAT_KOKO.2 <= PARHAAT_L) };
    const { assert!(PARHAAT_Y + PARHAAT_OTSIKKO + 3 * PARHAAT_RIVI < PANEELI_Y) };
    const { assert!(KENTTA_Y + KENTTA_K < PANEELI_Y) };
    const { assert!(RENKAITA as i32 * (2 * RENKAAN_SADE + 12) <= PARHAAT_L) };
}

#[test]
fn keila_valaisee_suuntaansa_eika_taaksepain() {
    let mut arpa = Arpa::new(1);
    let mut meri = Meri::new(Vaikeus::Normaali, 0, &mut arpa);
    meri.keila.kulma = -PI / 2.0;
    let alue = Keilanalue::new(&meri);
    assert_eq!(alue.osuus(MAJAKKA.0, MAJAKKA.1 - 200.0), 1.0);
    assert_eq!(alue.osuus(MAJAKKA.0 - 300.0, MAJAKKA.1 - 10.0), 0.0);
    assert_eq!(alue.osuus(MAJAKKA.0, MAJAKKA.1 + 50.0), 0.0);
    // Kantaman päässä valo hiipuu.
    let kauka = alue.osuus(MAJAKKA.0, MAJAKKA.1 - 700.0);
    assert!(kauka > 0.0 && kauka < 0.2);
    // Logiikan valaisema laiva on myös kuvassa valossa.
    let p = (MAJAKKA.0 + 20.0, MAJAKKA.1 - 250.0);
    assert!(meri.keila.valaisee(p, 1.0));
    assert!(alue.osuus(p.0, p.1) > 0.0);
}

#[test]
fn maasto_tuntee_sataman_rannan_ja_karikon() {
    let karikot = [Kari {
        paikka: (100.0, 200.0),
        sade: 12.0,
    }];
    assert!(matches!(maasto(100.0, 200.0, &karikot), Maasto::Kari(_)));
    assert!(matches!(maasto(100.0, 214.0, &karikot), Maasto::Vaahto(_)));
    assert!(maasto(400.0, 100.0, &karikot) == Maasto::Vesi);
    assert!(maasto(SATAMA.0, SATAMA.1 + 20.0, &karikot) == Maasto::Satama);
    assert!(maasto(60.0, 539.0, &karikot) == Maasto::Maa);
    for &(x, y) in &TALOT {
        assert!(y > ranta_y(x) + 5.0 && y > ranta_y(x + TALON_L) + 5.0);
        assert!(y + TALON_K < majakka::KORKEUS && x + TALON_L < majakka::LEVEYS);
        assert!(!suulla(x) && !suulla(x + TALON_L));
    }
}

#[test]
fn koko_peli_piirtyy_kaatumatta() {
    let mut kuva = vec![0; LEVEYS * KORKEUS];
    for taso in Vaikeus::KAIKKI {
        let mut peli = Majakka::new(3);
        peli.vaikeus = taso;
        piirra(&peli, &mut kuva);
        peli.kasittele(Syote::Ammu);
        peli.meri
            .laivat
            .push(Laiva::salakuljettaja((300.0, 200.0), (700.0, 200.0)));
        peli.meri.laivat[0].keilassa_ms = 700;
        for i in 0..3000 {
            peli.edista(16);
            if i % 7 == 0 {
                piirra(&peli, &mut kuva);
            }
        }
        for vuorokausi in [Vuorokausi::Sarastus, Vuorokausi::Aamu] {
            peli.meri.vuorokausi = vuorokausi;
            piirra(&peli, &mut kuva);
        }
        peli.meri.askel(Ohjaus::default(), &mut Arpa::new(1));
        for vaihe in [Vaihe::Tauko, Vaihe::Tappio, Vaihe::Nimi] {
            peli.vaihe = vaihe;
            piirra(&peli, &mut kuva);
        }
    }
}
