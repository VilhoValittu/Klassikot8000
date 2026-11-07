use super::*;
use crate::yhteinen::tulokset::Tulos;
use klassikot8000::asteroidit::{Kivi, Koko};

fn pelissa() -> Asteroidit {
    let mut peli = Asteroidit::new(1);
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
    peli.aanet.jono.clear();
    peli
}

/// Paikallaan oleva kivi aluksen kohdalla: viimeinen alus tuhoutuu seuraavalla askeleella.
fn paattyva(pisteet: u32) -> Asteroidit {
    let mut peli = pelissa();
    peli.lauta.pisteet = pisteet;
    peli.lauta.elamat = 1;
    peli.lauta.kivet = vec![Kivi {
        paikka: peli.lauta.alus.paikka,
        nopeus: (0.0, 0.0),
        koko: Koko::Pieni,
        kulma: 0.0,
        pyorinta: 0.0,
        muoto: 0,
    }];
    peli.aanet.jono.clear();
    peli
}

#[test]
fn valilyonti_aloittaa_ja_toinen_ampuu() {
    let mut peli = Asteroidit::new(1);
    peli.kasittele(Syote::Nuoli(0, 1));
    assert_eq!(peli.vaihe, Vaihe::Alku);
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
    assert_eq!(peli.aanet.jono, [(Aani::Alku, 0)]);
    peli.aanet.jono.clear();
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.lauta.ammukset.len(), 1);
    assert_eq!(peli.aanet.jono, [(Aani::Ammus, 0)]);
}

#[test]
fn klikkaus_ampuu_ja_hiiri_kaantaa_alusta_kentan_koordinaateissa() {
    let mut peli = pelissa();
    let (x, y) = peli.lauta.alus.paikka;
    peli.kasittele(Syote::Hiiri(
        piirto::KENTTA_X + x as i32 + 100,
        piirto::KENTTA_Y + y as i32,
    ));
    assert_eq!(peli.ohjaus.tahtain, Some((x + 100.0, y)));
    peli.lauta.kivet.clear();
    for _ in 0..10 {
        peli.edista(50);
    }
    assert!(peli.lauta.alus.kulma.sin().abs() < 0.01);
    peli.kasittele(Syote::Klikkaus(300, 300));
    assert_eq!(peli.lauta.ammukset.len(), 1);

    peli.aseta_nuolet(1, false);
    assert_eq!(peli.ohjaus.tahtain, None);
}

#[test]
fn hiiri_tahtaa_ikkunan_ulkopuolelta_vain_pelin_aikana() {
    let mut peli = Asteroidit::new(1);
    assert!(!peli.hiiri_ulkonakin());
    peli.kasittele(Syote::Ammu);
    assert!(peli.hiiri_ulkonakin());
    peli.kasittele(Syote::Kirjain('P'));
    assert!(!peli.hiiri_ulkonakin());
    peli.kasittele(Syote::Kirjain('P'));
    assert!(peli.hiiri_ulkonakin());
    peli.vaihe = Vaihe::Tappio;
    assert!(!peli.hiiri_ulkonakin());
}

#[test]
fn tauko_pysayttaa_kivet_ja_jatkuu_samasta() {
    let mut peli = pelissa();
    peli.kasittele(Syote::Kirjain('P'));
    let paikka = peli.lauta.kivet[0].paikka;
    peli.edista(100);
    assert_eq!(peli.lauta.kivet[0].paikka, paikka);
    peli.kasittele(Syote::Kirjain('P'));
    peli.edista(100);
    assert_ne!(peli.lauta.kivet[0].paikka, paikka);

    peli.keskeyta();
    assert_eq!(peli.vaihe, Vaihe::Tauko);
}

#[test]
fn iso_aikahyppy_ei_siirra_kivia_kerralla_pitkalle() {
    let mut peli = pelissa();
    peli.lauta.kivet.truncate(1);
    peli.lauta.kivet[0].nopeus = (100.0, 0.0);
    peli.lauta.kivet[0].paikka = (0.0, 0.0);
    peli.edista(60_000);
    assert!(peli.lauta.kivet[0].paikka.0 < 100.0);
}

#[test]
fn moottori_ja_lautanen_soivat_uudelleen_valein() {
    let mut peli = pelissa();
    peli.lauta.kivet.clear();
    peli.aseta_nuolet(0, true);
    peli.edista(16);
    peli.edista(16);
    assert_eq!(peli.aanet.jono, [(Aani::Moottori, 0)]);
    peli.edista(MOOTTORIN_VALI_MS);
    assert_eq!(peli.aanet.jono.len(), 2);
}

#[test]
fn viimeisen_aluksen_tuho_kysyy_nimen_ja_palaa_loppuruutuun() {
    let mut peli = paattyva(40);
    peli.edista(ASKEL_MS);
    assert_eq!(peli.vaihe, Vaihe::Nimi);
    assert_eq!(peli.lauta.pisteet, 140);
    assert_eq!(peli.taulu.paras, 140);
    assert!(peli.uusi_tulos);
    assert_eq!(peli.rajahdykset.len(), 2);

    // Ampumiseen painettu välilyönti ei kuittaa tyhjää nimeä, ja K on nimessä kirjain.
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
    assert_eq!(peli.taulu.lahetys.as_ref().unwrap().pisteet, 140);
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
    assert_eq!(peli.taulu.paras, 140);
}

#[test]
fn kivet_ajelehtivat_pelin_paatyttya_mutta_tulos_ei_muutu() {
    let mut peli = paattyva(40);
    peli.edista(ASKEL_MS);
    peli.vahvista_nimi();
    // Osunut kivi hajosi, joten kentälle tuodaan uusi.
    peli.lauta.kivet = vec![Kivi {
        paikka: (100.0, 100.0),
        nopeus: (100.0, 0.0),
        koko: Koko::Iso,
        kulma: 0.0,
        pyorinta: 0.0,
        muoto: 0,
    }];
    let paikka = peli.lauta.kivet[0].paikka;
    let pisteet = peli.lauta.pisteet;
    peli.edista(50);
    assert_ne!(peli.lauta.kivet[0].paikka, paikka);
    assert_eq!(peli.lauta.pisteet, pisteet);
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
    // Kivi aluksen päällä ei saa päättää peliä nimen kirjoituksen aikana.
    peli.edista(1000);
    assert_eq!(peli.lauta.pisteet, 500);
    assert_eq!(peli.lauta.elamat, 1);

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
    let mut peli = Asteroidit::new(1);
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
