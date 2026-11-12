use super::*;
use crate::yhteinen::tulokset::Tulos;
use klassikot8000::majakka::{Kari, Laiva, MAJAKKA, SATAMA, TORVEN_KESTO_MS, ULAPPA, Vuorokausi};

fn pelissa() -> Majakka {
    let mut peli = Majakka::new(1);
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
    peli.aanet.jono.clear();
    peli
}

/// Viimeinen rengas ja karikko laivan edessä: laiva haaksirikkoutuu heti.
fn paattyva(pisteet: u32) -> Majakka {
    let mut peli = pelissa();
    peli.meri.pisteet = pisteet;
    peli.meri.renkaat = 1;
    peli.meri.karikot = vec![Kari {
        paikka: (200.0, 200.0),
        sade: 15.0,
    }];
    peli.meri.laivat = vec![Laiva::uusi(Laji::Vene, (200.0, 190.0), 0.0)];
    peli.aanet.jono.clear();
    peli
}

#[test]
fn taso_valitaan_alussa_ja_peli_alkaa_silla() {
    let mut peli = Majakka::new(1);
    peli.kasittele(Syote::Numero(3));
    assert_eq!(peli.vaikeus, Vaikeus::Vaikea);
    peli.kasittele(Syote::Nuoli(0, -1));
    assert_eq!(peli.vaikeus, Vaikeus::Normaali);
    peli.kasittele(Syote::Numero(9));
    assert_eq!(peli.vaikeus, Vaikeus::Normaali);
    let (x, y) = piirto::vaikeuden_keskipiste(Vaikeus::Helppo);
    peli.kasittele(Syote::Klikkaus(x, y));
    assert_eq!(peli.vaikeus, Vaikeus::Helppo);
    assert_eq!(peli.vaihe, Vaihe::Alku);
    peli.kasittele(Syote::Klikkaus(0, 300));
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
    assert_eq!(peli.meri.vaikeus, Vaikeus::Helppo);
    assert!(peli.kuva.is_some());
    assert_eq!(peli.aanet.jono, [(Aani::Alku, 0)]);

    let mut asetukset = Asetukset::default();
    peli.kirjaa_asetukset(&mut asetukset);
    assert_eq!(asetukset.majakka, Vaikeus::Helppo);
}

#[test]
fn hiiri_tahtaa_keilan_ja_nuolet_ohittavat_sen() {
    let mut peli = pelissa();
    let kohde = (100.0, 100.0);
    peli.kasittele(Syote::Hiiri(
        piirto::KENTTA_X + kohde.0 as i32,
        piirto::KENTTA_Y + kohde.1 as i32,
    ));
    assert_eq!(peli.ohjaus.tahtain, Some(suunta_majakalta(kohde)));
    for _ in 0..100 {
        peli.edista(20);
    }
    let odotettu = suunta_majakalta(kohde);
    assert!((peli.meri.keila.kulma - odotettu).abs() < 0.01);

    peli.aseta_nuolet(1);
    assert_eq!(peli.ohjaus.tahtain, None);
    let ennen = peli.meri.keila.kulma;
    peli.edista(100);
    assert!(peli.meri.keila.kulma > ennen);
}

#[test]
fn valilyonti_soittaa_torven_kerran_latauksen_aikana() {
    let mut peli = pelissa();
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.meri.torvi_ms, TORVEN_KESTO_MS);
    assert_eq!(peli.aanet.jono, [(Aani::Sumutorvi, 0)]);
    peli.kasittele(Syote::Klikkaus(300, 300));
    assert_eq!(peli.aanet.jono.len(), 1);
}

#[test]
fn tauko_pysayttaa_meren_ja_jatkuu_samasta() {
    let mut peli = pelissa();
    peli.meri.laivat = vec![Laiva::uusi(Laji::Vene, (100.0, 100.0), 0.0)];
    peli.kasittele(Syote::Kirjain('P'));
    let paikka = peli.meri.laivat[0].paikka;
    peli.edista(100);
    assert_eq!(peli.meri.laivat[0].paikka, paikka);
    peli.kasittele(Syote::Kirjain('P'));
    peli.edista(100);
    assert_ne!(peli.meri.laivat[0].paikka, paikka);

    peli.keskeyta();
    assert_eq!(peli.vaihe, Vaihe::Tauko);
}

#[test]
fn iso_aikahyppy_ei_siirra_laivoja_kerralla_pitkalle() {
    let mut peli = pelissa();
    peli.meri.karikot.clear();
    peli.meri.laivat = vec![Laiva::uusi(Laji::Vene, (100.0, 100.0), 0.0)];
    peli.edista(60_000);
    assert!(peli.meri.laivat[0].paikka.0 < 110.0);
}

#[test]
fn satamaan_paassyt_laiva_soi_kelloa_ja_nayttaa_pisteet() {
    let mut peli = pelissa();
    peli.meri.laivat = vec![Laiva::uusi(
        Laji::Kalastaja,
        (SATAMA.0, SATAMA.1 - 8.0),
        1.57,
    )];
    peli.edista(ASKEL_MS * 4);
    assert!(peli.meri.laivat.is_empty());
    // Normaalilla tasolla kerroin on kaksi.
    assert_eq!(peli.meri.pisteet, 40);
    assert_eq!(peli.taulu.paras, 40);
    assert!(peli.aanet.jono.contains(&(Aani::Laivakello, 0)));
    assert_eq!(peli.tehosteet[0].laji, Tehostelaji::Pisteet(40));
    assert_eq!(peli.viestit.last().unwrap(), "Kalastaja satamassa! +40");
}

#[test]
fn viesteja_mahtuu_kaksi_ja_uusi_on_viimeisena() {
    let mut peli = pelissa();
    for _ in 0..3 {
        peli.meri.laivat = vec![Laiva::uusi(Laji::Vene, (SATAMA.0, SATAMA.1 - 8.0), 1.57)];
        peli.edista(ASKEL_MS * 4);
    }
    assert_eq!(peli.viestit.len(), VIESTEJA);
    assert!(peli.viestit[1].ends_with("+20"));
}

#[test]
fn viimeinen_haaksirikko_kysyy_nimen_ja_palaa_loppuruutuun() {
    let mut peli = paattyva(40);
    peli.edista(ASKEL_MS);
    assert_eq!(peli.vaihe, Vaihe::Nimi);
    assert!(peli.uusi_tulos);
    assert_eq!(peli.taulu.paras, 40);
    assert!(peli.aanet.jono.contains(&(Aani::Karille, 0)));
    assert_eq!(peli.tehosteet.len(), 1);

    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.vaihe, Vaihe::Nimi);
    peli.kasittele(Syote::Kirjain('K'));
    peli.uusi_tulos = false;
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.vaihe, Vaihe::Tappio);
    assert!(peli.uusi_tulos);
    assert_eq!(peli.taulu.parhaat[0].nimi, "K");
    assert_eq!(peli.taulu.lahetys.last().unwrap().pisteet, 40);
}

#[test]
fn listalle_yltamaton_tulos_ei_kysy_nimea_ja_uusi_peli_alkaa_alusta() {
    let mut peli = paattyva(40);
    peli.taulu.netti = Some(vec![
        Tulos {
            nimi: "A".to_string(),
            pisteet: 1000,
        };
        3
    ]);
    peli.edista(ASKEL_MS);
    assert_eq!(peli.vaihe, Vaihe::Tappio);
    assert_eq!(peli.taulu.lahetys, Vec::new());

    peli.kasittele(Syote::Kirjain('K'));
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
    assert_eq!(peli.meri.pisteet, 0);
    assert_eq!(peli.meri.renkaat, 3);
    assert_eq!(peli.taulu.paras, 40);
}

#[test]
fn esc_kesken_pelin_pysayttaa_ja_toinen_kysyy_nimen() {
    let mut peli = paattyva(500);
    peli.kasittele(Syote::Peru);
    assert_eq!(peli.vaihe, Vaihe::Tauko);
    peli.kasittele(Syote::Peru);
    assert_eq!(peli.vaihe, Vaihe::Nimi);
    assert!(!peli.lopeta);
    // Karikko laivan edessä ei saa päättää peliä nimen kirjoituksen aikana.
    peli.edista(1000);
    assert_eq!(peli.meri.renkaat, 1);

    peli.kasittele(Syote::Kirjain('B'));
    peli.kasittele(Syote::Ammu);
    assert!(peli.lopeta);
    assert_eq!(peli.taulu.parhaat[0].nimi, "B");
}

#[test]
fn lopun_napit_toimivat() {
    let mut peli = Majakka::new(1);
    peli.vaihe = Vaihe::Tappio;
    let (x, y) = loppunapit::napin_keskipiste(Loppunappi::Uudestaan);
    peli.kasittele(Syote::Klikkaus(x, y));
    assert_eq!(peli.vaihe, Vaihe::Pelissa);

    peli.vaihe = Vaihe::Tappio;
    let (x, y) = loppunapit::napin_keskipiste(Loppunappi::Valikko);
    peli.kasittele(Syote::Klikkaus(x, y));
    assert!(peli.lopeta);
}

#[test]
fn uusi_yo_laskee_meren_kuvan_uudelleen() {
    let mut peli = pelissa();
    peli.meri.vuorokausi = Vuorokausi::Aamu;
    peli.meri.aika_ms = ASKEL_MS;
    let vanhat = peli.meri.karikot.clone();
    peli.edista(ASKEL_MS);
    assert_eq!(peli.meri.yo, 1);
    assert_ne!(peli.meri.karikot, vanhat);
    assert_eq!(peli.viestit.last().unwrap(), "Yö 2 alkaa.");
    assert!(peli.aanet.jono.contains(&(Aani::Alku, 0)));
}

#[test]
fn majakan_ja_sataman_paikat_ovat_kentalla() {
    for p in [MAJAKKA, SATAMA, ULAPPA] {
        assert!(p.0 > 0.0 && p.1 > 0.0);
    }
}
