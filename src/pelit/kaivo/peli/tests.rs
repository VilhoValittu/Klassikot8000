use super::*;
use crate::yhteinen::tulokset::Tulos;
use klassikot8000::kaivo::{ALKU, HISSI, KYPARIA, PINTA, RIVEJA, SARAKKEITA};

/// Aikaa pienissä paloissa: yksi `edista` ajaa enintään 100 ms.
fn aja(peli: &mut Kaivo, ms: u32) {
    for _ in 0..ms.div_ceil(20) {
        peli.edista(20);
    }
}

fn pelissa() -> Kaivo {
    let mut peli = Kaivo::new(1);
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
    // Tasainen maa, jottei arvottu kenttä sotke testejä.
    for y in MAA_ALKAA..RIVEJA as i32 - 1 {
        for x in 0..SARAKKEITA as i32 {
            peli.kaivos.aseta((x, y), Solu::Maa);
        }
    }
    peli.aanet.jono.clear();
    peli
}

/// Viimeinen kypärä ja kivi kaivajan yläpuolella: peli päättyy kiven pudotessa.
fn paattyva(pisteet: u32) -> Kaivo {
    let mut peli = pelissa();
    peli.kaivos.pisteet = pisteet;
    peli.kaivos.kyparat = 1;
    for x in 8..12 {
        peli.kaivos.aseta((x, 6), Solu::Tyhja);
    }
    peli.kaivos.aseta((10, 5), Solu::Kivi);
    peli.kaivos.kaivaja.paikka = (10, 6);
    peli
}

#[test]
fn taso_valitaan_alussa_ja_peli_alkaa_silla() {
    let mut peli = Kaivo::new(1);
    peli.kasittele(Syote::Numero(3));
    assert_eq!(peli.vaikeus, Vaikeus::Vaikea);
    peli.kasittele(Syote::Nuoli(0, -1));
    assert_eq!(peli.vaikeus, Vaikeus::Normaali);
    let (x, y) = piirto::vaikeuden_keskipiste(Vaikeus::Helppo);
    peli.kasittele(Syote::Klikkaus(x, y));
    assert_eq!(peli.vaikeus, Vaikeus::Helppo);
    assert_eq!(peli.vaihe, Vaihe::Alku);
    peli.kasittele(Syote::Klikkaus(0, 300));
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
    assert_eq!(peli.kaivos.vaikeus, Vaikeus::Helppo);
    assert!(peli.kuva.is_some());
    assert_eq!(peli.aanet.jono, [(Aani::Alku, 0)]);

    let mut asetukset = Asetukset::default();
    peli.kirjaa_asetukset(&mut asetukset);
    assert_eq!(asetukset.kaivo, Vaikeus::Helppo);
}

#[test]
fn nuoli_kirjain_ja_klikkaus_kaivavat() {
    let mut peli = pelissa();
    peli.kasittele(Syote::Nuoli(1, 0));
    aja(&mut peli, 400);
    assert_eq!(peli.kaivos.kaivaja.paikka, (ALKU.0, MAA_ALKAA));
    assert!(peli.aanet.jono.contains(&(Aani::Kuokka, 0)));

    peli.kasittele(Syote::Kirjain('D'));
    aja(&mut peli, 400);
    assert_eq!(peli.kaivos.kaivaja.paikka, (ALKU.0 + 1, MAA_ALKAA));

    // Klikkaus kauas vasemmalle ja vähän alas: vaakasuunta voittaa.
    let (x, y) = piirto::ruudun_keskipiste((0, MAA_ALKAA + 1));
    peli.kasittele(Syote::Klikkaus(x, y));
    aja(&mut peli, 400);
    assert_eq!(peli.kaivos.kaivaja.paikka, (ALKU.0, MAA_ALKAA));
}

#[test]
fn yksi_painallus_vie_yhden_ruudun_maan_allakin() {
    let mut peli = pelissa();
    peli.kasittele(Syote::Nuoli(0, 1));
    aja(&mut peli, 1000);
    assert_eq!(peli.kaivos.kaivaja.paikka, (ALKU.0 + 1, PINTA));

    // Maan alla valmiissa tunnelissa.
    for x in 5..9 {
        peli.kaivos.aseta((x, 6), Solu::Tyhja);
    }
    peli.kaivos.kaivaja.paikka = (6, 6);
    peli.kasittele(Syote::Kirjain('D'));
    aja(&mut peli, 1000);
    assert_eq!(peli.kaivos.kaivaja.paikka, (7, 6));
    // Kävelyn aikana annettu painallus odottaa vuoroaan: kaksi painallusta, kaksi ruutua.
    peli.kasittele(Syote::Nuoli(0, 1));
    aja(&mut peli, 20);
    peli.kasittele(Syote::Nuoli(1, 0));
    aja(&mut peli, 1000);
    assert_eq!(peli.kaivos.kaivaja.paikka, (8, 7));
}

#[test]
fn pohjassa_pidetty_nuoli_jatkaa_vasta_viiveen_jalkeen() {
    let mut peli = pelissa();
    // Painallus ja pito alkavat samassa ruudussa, kuten oikeassa ikkunassa.
    peli.kasittele(Syote::Nuoli(0, 1));
    peli.aseta_nuolet(1, 0);
    aja(&mut peli, 380);
    assert_eq!(peli.kaivos.kaivaja.paikka, (ALKU.0 + 1, PINTA));
    aja(&mut peli, 1000);
    assert!(peli.kaivos.kaivaja.paikka.0 >= ALKU.0 + 5);
    peli.aseta_nuolet(0, 0);
    aja(&mut peli, 200);
    let pysahtyi = peli.kaivos.kaivaja.paikka;
    aja(&mut peli, 500);
    assert_eq!(peli.kaivos.kaivaja.paikka, pysahtyi);
}

#[test]
fn valilyonti_pystyttaa_ja_purkaa_tuen_ja_kertoo_kun_tuet_loppuvat() {
    let mut peli = pelissa();
    peli.kasittele(Syote::Nuoli(1, 0));
    aja(&mut peli, 400);
    peli.kasittele(Syote::Ammu);
    let paikka = peli.kaivos.kaivaja.paikka;
    assert_eq!(peli.kaivos.solu(paikka), Solu::Tuki);
    assert!(peli.aanet.jono.contains(&(Aani::Tiili, 0)));
    let versio = peli.kuva.as_ref().unwrap().versio;
    assert_eq!(versio, peli.kaivos.versio);
    peli.kasittele(Syote::OikeaKlikkaus(0, 0));
    assert_eq!(peli.kaivos.solu(paikka), Solu::Tyhja);

    peli.kaivos.tuet = 0;
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.viestit.last().unwrap(), "Tuet loppuivat.");
    assert!(peli.aanet.jono.contains(&(Aani::Virhe, 0)));
}

#[test]
fn r_heittaa_koyden_vain_maan_alla() {
    let mut peli = pelissa();
    peli.kasittele(Syote::Kirjain('R'));
    assert_eq!(peli.kaivos.kyparat, KYPARIA);
    peli.kasittele(Syote::Nuoli(1, 0));
    aja(&mut peli, 400);
    peli.kasittele(Syote::Kirjain('R'));
    assert_eq!(peli.kaivos.kyparat, KYPARIA - 1);
    assert_eq!(peli.kaivos.kaivaja.paikka, ALKU);
    assert!(peli.viestit.last().unwrap().contains('2'));
}

#[test]
fn kulta_hissille_antaa_pisteet_ja_kuvan_muutokset_seuraavat() {
    let mut peli = pelissa();
    peli.kaivos.kiintio = 1000;
    peli.kaivos.saalis = 40;
    peli.kaivos.kaivaja.paikka = (HISSI.0 + 1, PINTA);
    peli.kasittele(Syote::Nuoli(0, -1));
    aja(&mut peli, 300);
    assert_eq!(peli.kaivos.pisteet, 40);
    assert_eq!(peli.taulu.paras, 40);
    assert!(peli.aanet.jono.contains(&(Aani::Yhdistys, 0)));
    assert_eq!(peli.tehosteet[0].laji, Tehostelaji::Pisteet(40));
    assert_eq!(peli.viestit.last().unwrap(), "Nostettu 40, puuttuu 960.");
}

#[test]
fn sortuma_paattaa_viimeisen_kyparan_ja_kysyy_nimen() {
    let mut peli = paattyva(40);
    aja(&mut peli, 100);
    aja(&mut peli, 100);
    for _ in 0..20 {
        aja(&mut peli, 100);
    }
    assert_eq!(peli.vaihe, Vaihe::Nimi);
    assert!(peli.uusi_tulos);
    assert!(peli.aanet.jono.contains(&(Aani::Karille, 0)));
    assert!(peli.aanet.jono.contains(&(Aani::Ritina, 0)));
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
    for _ in 0..20 {
        aja(&mut peli, 100);
    }
    assert_eq!(peli.vaihe, Vaihe::Tappio);
    assert_eq!(peli.taulu.lahetys, Vec::new());
    peli.kasittele(Syote::Kirjain('K'));
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
    assert_eq!(peli.kaivos.pisteet, 0);
    assert_eq!(peli.kaivos.kyparat, KYPARIA);
}

#[test]
fn esc_kesken_pelin_pysayttaa_ja_toinen_kysyy_nimen() {
    let mut peli = paattyva(500);
    peli.kasittele(Syote::Peru);
    assert_eq!(peli.vaihe, Vaihe::Tauko);
    peli.kasittele(Syote::Peru);
    assert_eq!(peli.vaihe, Vaihe::Nimi);
    assert!(!peli.lopeta);
    // Kivi kaivajan yllä ei saa pudota nimen kirjoituksen aikana.
    aja(&mut peli, 3000);
    assert_eq!(peli.kaivos.kyparat, 1);
    peli.kasittele(Syote::Kirjain('B'));
    peli.kasittele(Syote::Ammu);
    assert!(peli.lopeta);
    assert_eq!(peli.taulu.parhaat[0].nimi, "B");
}

#[test]
fn tauko_pysayttaa_ja_ikkunan_vaihto_pysayttaa() {
    let mut peli = pelissa();
    peli.kasittele(Syote::Kirjain('P'));
    assert_eq!(peli.vaihe, Vaihe::Tauko);
    peli.kasittele(Syote::Nuoli(1, 0));
    aja(&mut peli, 400);
    assert_eq!(peli.kaivos.kaivaja.paikka, ALKU);
    peli.kasittele(Syote::Kirjain('P'));
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
    peli.keskeyta();
    assert_eq!(peli.vaihe, Vaihe::Tauko);
}

#[test]
fn lopun_napit_toimivat() {
    let mut peli = Kaivo::new(1);
    peli.vaihe = Vaihe::Tappio;
    let (x, y) = loppunapit::napin_keskipiste(Loppunappi::Uudestaan);
    peli.kasittele(Syote::Klikkaus(x, y));
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
    peli.vaihe = Vaihe::Tappio;
    let (x, y) = loppunapit::napin_keskipiste(Loppunappi::Valikko);
    peli.kasittele(Syote::Klikkaus(x, y));
    assert!(peli.lopeta);
}
