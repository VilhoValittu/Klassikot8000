use super::*;
use crate::kaynnistin::tallennus::Tallennus;
use klassikot8000::jatsi::HEITTOJA;

fn peli() -> Jatsi {
    let mut peli = Jatsi::new(3);
    peli.aanet.paalla = false;
    peli
}

/// Pelaa pöydän loppuun: ihmisen vuorolla heittää kolmesti ja kirjaa kursorin rivin.
fn pelaa_loppuun(peli: &mut Jatsi) {
    for _ in 0..10_000 {
        match peli.vaihe {
            Vaihe::Pelissa if peli.poyta.kone_vuorossa() => {
                peli.edista(KONE_ASKEL_MS + HEITTO_MS);
            }
            Vaihe::Pelissa => {
                peli.kasittele(Syote::Ammu);
                peli.edista(HEITTO_MS);
            }
            _ => return,
        }
    }
    panic!("peli ei päättynyt");
}

#[test]
fn oletuspoydassa_on_pelaaja_ja_kone() {
    let mut peli = peli();
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
    assert_eq!(peli.poyta.koneet, vec![false, true]);
    assert_eq!(peli.nimet[0], Jatsi::oletusnimi(0));
    assert_eq!(peli.nimi(1), muotoile("jatsi.kone", &[&1]));
}

#[test]
fn paikat_vaihtuvat_nuolilla_ja_tyhja_paikka_jaa_pois() {
    let mut peli = peli();
    // Ensimmäinen paikka on aina ihmisen.
    peli.kasittele(Syote::Nuoli(0, 1));
    assert_eq!(peli.paikat[0].istuja, Istuja::Ihminen);
    peli.kasittele(Syote::Nuoli(1, 0));
    peli.kasittele(Syote::Nuoli(0, 1));
    assert_eq!(peli.paikat[1].istuja, Istuja::Ihminen);
    peli.kasittele(Syote::Nuoli(1, 0));
    peli.kasittele(Syote::Nuoli(0, -1));
    assert_eq!(peli.paikat[2].istuja, Istuja::Ihminen);
    peli.kasittele(Syote::Nuoli(0, -1));
    assert_eq!(peli.paikat[2].istuja, Istuja::Kone);
    peli.kasittele(Syote::Nuoli(1, 0));
    peli.kasittele(Syote::Nuoli(1, 0));
    assert_eq!(peli.valittu_paikka, 3, "valinta pysähtyy viimeiseen");
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.poyta.koneet, vec![false, false, true]);
    assert_eq!(peli.nimet[1], Jatsi::oletusnimi(1));
}

#[test]
fn nimi_kirjoitetaan_alkuruudulla_ja_m_on_kirjain() {
    let mut peli = peli();
    peli.kasittele(Syote::Kirjain('N'));
    assert!(peli.nimetaan);
    for syote in [Syote::Kirjain('A'), Syote::VaihdaAanet, Syote::Kirjain('X')] {
        peli.kasittele(syote);
    }
    assert!(!peli.aanet.paalla, "M ei vaihda ääniä nimeä kirjoittaessa");
    peli.kasittele(Syote::Ammu);
    assert!(!peli.nimetaan);
    assert_eq!(peli.paikat[0].nimi, "AMX");
    // Koneen paikalla ei ole nimeä kirjoitettavaksi.
    peli.kasittele(Syote::Nuoli(1, 0));
    peli.kasittele(Syote::Kirjain('N'));
    assert!(!peli.nimetaan);
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.nimet[0], "AMX");
}

#[test]
fn taso_valitaan_numerolla() {
    let mut peli = peli();
    peli.kasittele(Syote::Numero(3));
    assert_eq!(peli.vaikeus, Vaikeus::Vaikea);
    peli.kasittele(Syote::Numero(1));
    assert_eq!(peli.vaikeus, Vaikeus::Helppo);
}

/// Yksinpeli ihmisen vuoron alkuun.
fn yksin() -> Jatsi {
    let mut peli = peli();
    peli.paikat[1].istuja = Istuja::Tyhja;
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.poyta.pelaajia(), 1);
    peli
}

#[test]
fn heitot_lukitus_ja_kirjaus_nappaimilla() {
    let mut peli = yksin();
    peli.kasittele(Syote::Kirjain('K'));
    assert_eq!(
        peli.poyta.kirja().kirjattuja(),
        0,
        "ennen heittoa ei kirjata"
    );
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.poyta.heitetty, 1);
    // Pyörivää noppaa ei lukita.
    peli.kasittele(Syote::Numero(2));
    assert!(!peli.poyta.lukitut[1]);
    peli.edista(HEITTO_MS);
    peli.kasittele(Syote::Numero(2));
    assert!(peli.poyta.lukitut[1]);
    let lukittu = peli.poyta.nopat[1];
    for _ in 1..HEITTOJA {
        peli.kasittele(Syote::Ammu);
        peli.edista(HEITTO_MS);
    }
    assert_eq!(peli.poyta.nopat[1], lukittu);
    assert!(!peli.poyta.voi_heittaa());
    // Kursori on parhaalla rivillä, joten se ei ole nolla.
    let rivi = peli.kursori;
    let odotettu = pisteet(rivi, &peli.poyta.nopat);
    assert!(odotettu > 0);
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.poyta.kirja().arvo(rivi), Some(odotettu));
    assert_eq!(peli.poyta.heitetty, 0);
}

#[test]
fn nolla_viivataan_vasta_toisella_painalluksella() {
    let mut peli = yksin();
    peli.kasittele(Syote::Ammu);
    peli.edista(HEITTO_MS);
    peli.poyta.nopat = [1, 2, 3, 4, 6];
    peli.kursori = Rivi::Ykkoset;
    peli.kasittele(Syote::Nuoli(-1, 0));
    assert_eq!(peli.kursori, Rivi::Jatsi, "kursori kiertää ympäri");
    peli.kasittele(Syote::Kirjain('K'));
    assert_eq!(peli.viivaus, Some(Rivi::Jatsi));
    assert!(peli.poyta.kirja().vapaa(Rivi::Jatsi));
    peli.kasittele(Syote::Kirjain('K'));
    assert_eq!(peli.poyta.kirja().arvo(Rivi::Jatsi), Some(0));
    // Kursori ohittaa kirjatun rivin.
    peli.kasittele(Syote::Ammu);
    peli.edista(HEITTO_MS);
    peli.kursori = Rivi::Sattuma;
    peli.kasittele(Syote::Nuoli(1, 0));
    assert_eq!(peli.kursori, Rivi::Ykkoset);
}

#[test]
fn kone_pelaa_omassa_tahdissaan_ja_valilyonti_hoputtaa() {
    let mut peli = peli();
    peli.kasittele(Syote::Ammu);
    // Ihmisen vuoro pois alta, jotta kone on vuorossa.
    if !peli.poyta.kone_vuorossa() {
        peli.kasittele(Syote::Ammu);
        peli.edista(HEITTO_MS);
        peli.poyta.nopat = [6, 6, 6, 6, 6];
        peli.kursori = Rivi::Jatsi;
        peli.kasittele(Syote::Kirjain('K'));
    }
    assert!(peli.poyta.kone_vuorossa());
    let ennen = peli.poyta.kirjat[1].kirjattuja();
    peli.edista(KONE_ASKEL_MS);
    assert_eq!(peli.poyta.heitetty, 1, "ensimmäinen askel heittää");
    assert!(peli.kuvan_avain().is_none(), "nopat pyörivät");
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.poyta.kirjat[1].kirjattuja(), ennen + 1);
    assert!(!peli.poyta.kone_vuorossa());
}

#[test]
fn peli_paattyy_ja_voittaja_saa_kertoimen() {
    for siemen in 0..6 {
        let mut peli = Jatsi::new(siemen);
        peli.aanet.paalla = false;
        peli.vaikeus = Vaikeus::Vaikea;
        peli.kasittele(Syote::Ammu);
        pelaa_loppuun(&mut peli);
        assert_eq!(peli.vaihe, Vaihe::Loppu);
        let summa = peli.poyta.summat()[0];
        let voitti = peli.poyta.voittajat().contains(&0);
        let odotettu = if voitti { summa * 3 } else { summa };
        assert_eq!(peli.tulokset, vec![(0, odotettu)]);
        assert_eq!(peli.taulu.parhaat[0].pisteet, odotettu);
        assert_eq!(peli.taulu.lahetys.len(), 1);
        assert!(peli.kesken().is_none(), "päättynyttä peliä ei tallenneta");
        assert!(peli.ota_uusi_tulos());
        peli.kasittele(Syote::Kirjain('K'));
        assert_eq!(peli.vaihe, Vaihe::Pelissa);
    }
}

#[test]
fn moninpelissa_jokainen_ihminen_paasee_listalle_ilman_kerrointa() {
    let mut peli = peli();
    peli.paikat[1].istuja = Istuja::Ihminen;
    peli.paikat[1].nimi = "B".to_string();
    peli.kasittele(Syote::Ammu);
    pelaa_loppuun(&mut peli);
    let summat = peli.poyta.summat();
    assert_eq!(peli.tulokset, vec![(0, summat[0]), (1, summat[1])]);
    let nimet: Vec<&str> = peli.taulu.lahetys.iter().map(|t| t.nimi.as_str()).collect();
    assert_eq!(nimet.len(), 2);
    assert!(nimet.contains(&"B"));
}

#[test]
fn kesken_jaanyt_peli_tallentuu_ja_jatkuu() {
    let mut peli = peli();
    peli.paikat[0].nimi = "AA".to_string();
    peli.kasittele(Syote::Ammu);
    for _ in 0..4 {
        pelaa_vuoro(&mut peli);
    }
    peli.kasittele(Syote::Ammu);
    peli.edista(HEITTO_MS);
    let mut tallennus = Tallennus::default();
    peli.kirjaa_tallennukseen(&mut tallennus);
    let luettu = Tallennus::tekstista(&tallennus.tekstiksi()).unwrap();
    assert_eq!(luettu.jatsikesken, tallennus.jatsikesken);
    assert_eq!(luettu.jatsipaikat, peli.paikat);

    let mut uusi = Jatsi::new(9);
    uusi.lue_tallennuksesta(&luettu);
    assert!(uusi.jatkuu);
    assert_eq!(uusi.poyta, peli.poyta);
    assert_eq!(uusi.nimet, peli.nimet);
    assert!(uusi.kesken().is_some(), "aloittamaton jatko säilyy");
    uusi.kasittele(Syote::Ammu);
    assert_eq!(uusi.vaihe, Vaihe::Pelissa);

    // K hylkää tallennetun pelin.
    let mut hylatty = Jatsi::new(9);
    hylatty.lue_tallennuksesta(&luettu);
    hylatty.kasittele(Syote::Kirjain('K'));
    assert!(!hylatty.jatkuu);
    assert!(hylatty.kesken().is_none());
}

/// Yksi kirjaus: ihminen heittää ja kirjaa, kone pelaa vuoronsa.
fn pelaa_vuoro(peli: &mut Jatsi) {
    if peli.poyta.kone_vuorossa() {
        peli.kasittele(Syote::Ammu);
        return;
    }
    let vuorossa = peli.poyta.vuorossa;
    while peli.poyta.vuorossa == vuorossa {
        peli.kasittele(Syote::Ammu);
        peli.edista(HEITTO_MS);
    }
}

#[test]
fn kelvoton_tallennus_hylataan() {
    let kirja = "I 3,-,-,-,-,-,-,-,-,-,-,-,-,-,- AA";
    assert!(Kesken::riveista("0,1,12345,00000", &[kirja]).is_some());
    // Mahdoton arvo, väärä noppa, vuoro ohi pöydän ja pelkät koneet.
    let mahdoton = "I 6,-,-,-,-,-,-,-,-,-,-,-,-,-,- AA";
    assert!(Kesken::riveista("0,1,12345,00000", &[mahdoton]).is_none());
    assert!(Kesken::riveista("0,1,12347,00000", &[kirja]).is_none());
    assert!(Kesken::riveista("1,1,12345,00000", &[kirja]).is_none());
    assert!(Kesken::riveista("0,4,12345,00000", &[kirja]).is_none());
    let kone = "K 3,-,-,-,-,-,-,-,-,-,-,-,-,-,-";
    assert!(Kesken::riveista("0,1,12345,00000", &[kone]).is_none());
    assert!(Kesken::riveista("0,1,12345,00000", &[kirja, kone]).is_some());
    assert!(
        paikat_riveista(&["0 K"]).is_none(),
        "ensimmäinen paikka on ihmisen"
    );
    assert!(
        paikat_riveista(&["1 I ab"]).is_none(),
        "pienet kirjaimet eivät kelpaa nimeksi"
    );
    let paikat = paikat_riveista(&["1 I BB", "2 K"]).unwrap();
    assert_eq!(paikat[1].nimi, "BB");
    assert_eq!(paikat[2].istuja, Istuja::Kone);
}

#[test]
fn kuva_elaa_vain_alkuruudulla_ja_animaatioissa() {
    let mut peli = yksin();
    assert!(peli.kuvan_avain().is_some());
    peli.kasittele(Syote::Ammu);
    assert!(peli.kuvan_avain().is_none());
    peli.edista(HEITTO_MS);
    assert!(peli.kuvan_avain().is_some());
}
