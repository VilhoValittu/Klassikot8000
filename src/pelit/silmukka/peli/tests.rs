use super::*;
use crate::yhteinen::tulokset::Tulos;
use klassikot8000::silmukka::{Herkku, Laji, Otus};
use std::collections::VecDeque;

fn pelissa() -> Silmukka {
    let mut peli = Silmukka::new(1);
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
    peli.aanet.jono.clear();
    peli
}

/// Mato on juuri törmäämässä paikallaan olevaan itikkaan.
fn paattyva(pisteet: u32) -> Silmukka {
    let mut peli = pelissa();
    peli.lauta.pisteet = pisteet;
    let paa = peli.lauta.paa();
    let mut itikka = Otus::uusi(Laji::Itikka, (paa.0 + 1, paa.1), (1, 1), 0);
    itikka.odotus_ms = 100_000;
    itikka.jakso_ms = 100_000;
    peli.lauta.otukset = vec![itikka];
    peli.lauta.odotus_ms = ASKEL_MS;
    peli
}

/// Mato sulkee seuraavalla askeleella renkaan, jonka sisällä on `maara` itikkaa.
fn sulkeutuva(maara: i32) -> Silmukka {
    let mut peli = pelissa();
    let mut ruudut: VecDeque<_> = (5..=12).map(|x| (x, 5)).collect();
    ruudut.extend((6..=12).map(|y| (12, y)));
    ruudut.extend((5..=11).rev().map(|x| (x, 12)));
    ruudut.extend((6..=11).rev().map(|y| (5, y)));
    ruudut.push_front((4, 5));
    peli.lauta.mato = ruudut.into_iter().rev().collect();
    peli.lauta.suunta = Suunta::Ylos;
    peli.lauta.odotus_ms = ASKEL_MS;
    peli.lauta.herkut.clear();
    peli.lauta.otukset = (0..maara)
        .map(|i| {
            let mut otus = Otus::uusi(Laji::Itikka, (6 + i, 8), (1, 1), 0);
            otus.odotus_ms = 100_000;
            otus.jakso_ms = 100_000;
            otus
        })
        .collect();
    peli
}

#[test]
fn valilyonti_aloittaa_ja_nuolet_ja_kirjaimet_kaantavat() {
    let mut peli = Silmukka::new(1);
    peli.kasittele(Syote::Nuoli(-1, 0));
    assert_eq!(peli.vaihe, Vaihe::Alku);
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
    assert_eq!(peli.aanet.jono, [(Aani::Alku, 0)]);

    peli.kasittele(Syote::Nuoli(-1, 0));
    peli.kasittele(Syote::Kirjain('A'));
    assert_eq!(peli.lauta.jono, [Suunta::Ylos, Suunta::Vasen]);
    peli.lauta.jono.clear();
    peli.kasittele(Syote::Kirjain('S'));
    peli.kasittele(Syote::Nuoli(0, 1));
    assert_eq!(peli.lauta.jono, [Suunta::Alas, Suunta::Oikea]);
}

#[test]
fn hiiri_asettaa_tahtaimen_ruudun_ja_nuoli_poistaa_sen() {
    let mut peli = pelissa();
    let (x, y) = piirto::ruudun_keskipiste((3, 4));
    peli.kasittele(Syote::Hiiri(x, y));
    assert_eq!(peli.lauta.tahtain, Some((3, 4)));
    peli.kasittele(Syote::Nuoli(-1, 0));
    assert_eq!(peli.lauta.tahtain, None);
}

#[test]
fn saalis_antaa_pisteet_tehosteet_ja_nousevan_helinan() {
    let mut peli = sulkeutuva(3);
    peli.edista(ASKEL_MS);
    assert_eq!(peli.lauta.pisteet, 25 * 3 * 3);
    assert_eq!(peli.valahdykset.len(), 1);
    assert!(peli.valahdykset[0].saalis);
    assert_eq!(peli.pamaukset.len(), 3);
    assert_eq!(peli.leijuvat[0].teksti, "+225");
    assert_eq!(peli.kehu.map(|k| k.0), Some("KOLMOSET!"));
    assert!(peli.tarina.is_some());
    assert!(peli.viesti.contains("225"));
    assert_eq!(
        &peli.aanet.jono[..3],
        [
            (Aani::Helina1, 0),
            (Aani::Helina2, HELINAN_VALI_MS),
            (Aani::Helina3, 2 * HELINAN_VALI_MS)
        ]
    );
    assert_eq!(peli.taulu.paras, 25 * 3 * 3);

    peli.edista(KEHU_MS + VALAHDYS_MS);
    assert!(peli.valahdykset.is_empty());
    assert!(peli.pamaukset.is_empty());
    assert!(peli.leijuvat.is_empty());
    assert_eq!(peli.kehu, None);
    assert_eq!(peli.tarina, None);
}

#[test]
fn suuri_saalis_saa_suuren_kehun_ja_yksittainen_ei_mitaan() {
    assert_eq!(kehu(1), None);
    assert_eq!(kehu(2), Some("TUPLA!"));
    assert_eq!(kehu(16), Some("JUHANNUSTAIKAA!"));
    let mut peli = sulkeutuva(1);
    peli.edista(ASKEL_MS);
    assert_eq!(peli.kehu, None);
    assert_eq!(peli.lauta.pisteet, 25);
}

#[test]
fn tyhja_silmukka_soi_eri_aanen_eika_anna_pisteita() {
    let mut peli = sulkeutuva(0);
    peli.edista(ASKEL_MS);
    assert_eq!(peli.lauta.pisteet, 0);
    assert_eq!(peli.aanet.jono, [(Aani::Puhkus, 0)]);
    assert!(!peli.valahdykset[0].saalis);
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
}

#[test]
fn tauko_pysayttaa_madon_ja_jatkuu_samasta() {
    let mut peli = pelissa();
    peli.kasittele(Syote::Kirjain('P'));
    let paa = peli.lauta.paa();
    peli.edista(100);
    peli.edista(100);
    assert_eq!(peli.lauta.paa(), paa);
    peli.kasittele(Syote::Kirjain('P'));
    peli.edista(100);
    peli.edista(100);
    assert_ne!(peli.lauta.paa(), paa);

    peli.keskeyta();
    assert_eq!(peli.vaihe, Vaihe::Tauko);
}

#[test]
fn iso_aikahyppy_ei_siirra_matoa_kerralla_pitkalle() {
    let mut peli = pelissa();
    let (x, _) = peli.lauta.paa();
    peli.edista(60_000);
    assert!(peli.lauta.paa().0 - x <= 1);
}

#[test]
fn kuolema_kysyy_nimen_ja_palaa_loppuruutuun() {
    let mut peli = paattyva(40);
    peli.edista(ASKEL_MS);
    assert_eq!(peli.vaihe, Vaihe::Nimi);
    assert_eq!(peli.taulu.paras, 40);
    assert!(peli.uusi_tulos);
    assert_eq!(peli.aanet.jono[0], (Aani::Rajahdys, 0));

    // Viimeisenä painettu välilyönti ei kuittaa tyhjää nimeä, ja K on nimessä kirjain.
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.vaihe, Vaihe::Nimi);
    peli.kasittele(Syote::Kirjain('K'));
    peli.kasittele(Syote::VaihdaAanet);
    assert!(peli.aanet.paalla);
    peli.uusi_tulos = false;
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.vaihe, Vaihe::Tappio);
    assert!(peli.uusi_tulos);
    assert_eq!(peli.taulu.parhaat[0].nimi, "KM");
    assert_eq!(peli.taulu.lahetys.as_ref().unwrap().pisteet, 40);
}

#[test]
fn listalle_yltamaton_tulos_ei_kysy_nimea_ja_uusi_peli_alkaa_alusta() {
    let taysi = (0..3)
        .map(|_| Tulos {
            nimi: "A".to_string(),
            pisteet: 1000,
        })
        .collect::<Vec<_>>();
    let mut peli = paattyva(40);
    peli.taulu.netti = Some(taysi);
    peli.edista(ASKEL_MS);
    assert_eq!(peli.vaihe, Vaihe::Tappio);
    assert_eq!(peli.taulu.lahetys, None);

    peli.kasittele(Syote::Kirjain('K'));
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
    assert_eq!(peli.lauta.pisteet, 0);
    assert_eq!(peli.taulu.paras, 40);
    assert!(peli.pamaukset.is_empty());
}

#[test]
fn esc_kirjaa_nimen_oletuksella_ja_vahvistus_ei_toistu() {
    let mut peli = paattyva(40);
    peli.edista(ASKEL_MS);
    peli.kasittele(Syote::Peru);
    assert!(!peli.lopeta);
    assert_eq!(peli.vaihe, Vaihe::Tappio);
    assert_eq!(peli.taulu.parhaat[0].nimi, "NIMETÖN");
    peli.vahvista_nimi();
    assert_eq!(peli.taulu.parhaat.len(), 1);
}

#[test]
fn esc_kesken_pelin_pysayttaa_ja_toinen_kysyy_nimen() {
    let mut peli = paattyva(500);
    peli.kasittele(Syote::Peru);
    assert_eq!(peli.vaihe, Vaihe::Tauko);
    assert!(!peli.lopeta);
    peli.kasittele(Syote::Peru);
    assert_eq!(peli.vaihe, Vaihe::Nimi);
    assert!(!peli.lopeta);
    // Itikka madon edessä ei saa päättää peliä nimen kirjoituksen aikana.
    peli.edista(1000);
    assert!(!peli.lauta.ohi);

    peli.kasittele(Syote::Kirjain('B'));
    peli.kasittele(Syote::Ammu);
    assert!(peli.lopeta);
    assert_eq!(peli.taulu.parhaat[0].nimi, "B");
    assert_eq!(peli.taulu.lahetys.as_ref().unwrap().pisteet, 500);
}

#[test]
fn listalle_yltamaton_tulos_poistuu_heti_ja_sulkeminen_kirjaa_oletusnimella() {
    let mut peli = paattyva(500);
    peli.taulu.netti = Some(vec![
        Tulos {
            nimi: "A".to_string(),
            pisteet: 1000,
        };
        3
    ]);
    peli.kasittele(Syote::Peru);
    assert!(!peli.lopeta);
    peli.kasittele(Syote::Peru);
    assert!(peli.lopeta);
    assert_eq!(peli.taulu.lahetys, None);

    let mut peli = paattyva(500);
    peli.poistu();
    peli.vahvista_nimi();
    assert_eq!(peli.taulu.parhaat[0].nimi, "NIMETÖN");
}

#[test]
fn lopun_napit_ja_esc_toimivat() {
    let mut peli = Silmukka::new(1);
    peli.vaihe = Vaihe::Tappio;
    let (x, y) = piirto::napin_keskipiste(Nappi::Uudestaan);
    peli.kasittele(Syote::Klikkaus(x, y));
    assert_eq!(peli.vaihe, Vaihe::Pelissa);

    peli.vaihe = Vaihe::Tappio;
    let (x, y) = piirto::napin_keskipiste(Nappi::Valikko);
    peli.kasittele(Syote::Klikkaus(x, y));
    assert!(peli.lopeta);

    // Esc pysäyttää, välilyönti jatkaa ja tauon aikana Esc palaa valikkoon.
    let mut peli = pelissa();
    peli.kasittele(Syote::Peru);
    assert_eq!(peli.vaihe, Vaihe::Tauko);
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
    peli.kasittele(Syote::Peru);
    assert!(!peli.lopeta);
    peli.kasittele(Syote::Peru);
    assert!(peli.lopeta);
}

#[test]
fn kultaomena_soi_ja_antaa_lisaa_pisteita() {
    let mut peli = pelissa();
    let paa = peli.lauta.paa();
    peli.lauta.herkut = vec![Herkku {
        paikka: (paa.0 + 1, paa.1),
        kulta_ms: Some(1000),
    }];
    peli.lauta.odotus_ms = ASKEL_MS;
    peli.edista(ASKEL_MS);
    assert_eq!(peli.lauta.pisteet, 100);
    assert_eq!(peli.aanet.jono[0], (Aani::Yhdistys, 0));
}
