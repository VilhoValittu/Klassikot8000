use super::*;
use klassikot8000::pasianssi::{Kortti, Maa, Sarake};

fn pelissa() -> Pasianssi {
    let mut peli = Pasianssi::new(1);
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
    peli.lauta.tila.pakka.clear();
    peli.lauta.tila.sarakkeet = Default::default();
    peli
}

fn kortti(maa: Maa, arvo: u8) -> Kortti {
    Kortti { maa, arvo }
}

fn aseta(peli: &mut Pasianssi, sarake: usize, kortit: &[Kortti]) {
    peli.lauta.tila.sarakkeet[sarake] = Sarake {
        kortit: kortit.to_vec(),
        piilossa: 0,
    };
}

/// Kaksi saraketta, joiden välillä musta 6 siirtyy punaisen 7:n päälle.
fn kaksi_saraketta() -> Pasianssi {
    let mut peli = pelissa();
    aseta(&mut peli, 0, &[kortti(Maa::Pata, 6)]);
    aseta(&mut peli, 1, &[kortti(Maa::Hertta, 7)]);
    peli
}

fn kortin_keskipiste(peli: &Pasianssi, sarake: usize, kortti: usize) -> (i32, i32) {
    let (x, y) = piirto::kortin_paikka(&peli.lauta, Lahde::Sarake { sarake, kortti });
    (x + 20, y + 20)
}

/// Tyhjentää haasteen laudan niin, että seuraava keräys voittaa.
fn yhden_kortin_paassa_voitosta(peli: &mut Pasianssi) {
    peli.lauta.tila.pakka.clear();
    peli.lauta.tila.jate.clear();
    peli.lauta.tila.sarakkeet = Default::default();
    for (i, &maa) in Maa::KAIKKI.iter().enumerate() {
        peli.lauta.tila.perustat[i] = (1..=13).map(|arvo| kortti(maa, arvo)).collect();
    }
    let viimeinen = peli.lauta.tila.perustat[0].pop().unwrap();
    aseta(peli, 0, &[viimeinen]);
}

#[test]
fn haasteet_avautuvat_ja_esc_palaa_alkuruudulle() {
    let mut peli = Pasianssi::new(1);
    peli.kasittele(Syote::Kirjain('H'));
    assert_eq!(peli.vaihe, Vaihe::Haasteet);
    peli.kasittele(Syote::Peru);
    assert_eq!(peli.vaihe, Vaihe::Alku);
    assert!(!peli.lopeta);

    let (x, y) = piirto::haastenapin_keskipiste();
    peli.kasittele(Syote::Klikkaus(x, y));
    assert_eq!(peli.vaihe, Vaihe::Haasteet);
    let (x, y) = haastenakyma::takaisin_keskipiste();
    peli.kasittele(Syote::Klikkaus(x, y));
    assert_eq!(peli.vaihe, Vaihe::Alku);
}

#[test]
fn haastelistan_osoitin_liikkuu_ruudukossa_ja_lajit_vaihtuvat() {
    let mut peli = Pasianssi::new(1);
    peli.kasittele(Syote::Kirjain('H'));
    peli.kasittele(Syote::Nuoli(-1, 0));
    peli.kasittele(Syote::Nuoli(0, -1));
    assert_eq!(peli.haaste_kursori, 0);
    peli.kasittele(Syote::Nuoli(1, 0));
    assert_eq!(peli.haaste_kursori, HAASTERIVILLA);
    peli.kasittele(Syote::Nuoli(0, 1));
    assert_eq!(peli.haaste_kursori, HAASTERIVILLA + 1);
    for _ in 0..10 {
        peli.kasittele(Syote::Nuoli(1, 0));
    }
    assert!(peli.haaste_kursori < HAASTEITA);
    peli.kasittele(Syote::Numero(2));
    assert_eq!(peli.variantti, Variantti::Spider);
    peli.kasittele(Syote::Numero(9));
    assert_eq!(peli.variantti, Variantti::Spider);
    let (x, y) = haastenakyma::lajin_keskipiste(Variantti::Pyramidi);
    peli.kasittele(Syote::Klikkaus(x, y));
    assert_eq!(peli.variantti, Variantti::Pyramidi);
    assert_eq!(peli.vaihe, Vaihe::Haasteet);
}

#[test]
fn jokaisen_lajin_haaste_alkaa_klikkaamalla_ja_kello_nollautuu() {
    for laji in Variantti::KAIKKI {
        let mut peli = Pasianssi::new(1);
        peli.variantti = laji;
        peli.peliaika_ms = 5000;
        peli.kasittele(Syote::Kirjain('H'));
        let (x, y) = haastenakyma::haasteen_keskipiste(HAASTEITA - 1);
        peli.kasittele(Syote::Klikkaus(x, y));
        assert_eq!(peli.vaihe, Vaihe::Pelissa);
        assert_eq!(peli.haaste, Some(HAASTEITA - 1));
        assert_eq!(peli.peliaika_ms, 0);
        assert_eq!(peli.laudan_vaikeus(), Vaikeus::Vaikea);
        assert_eq!(peli.viesti, kaanna(HAASTE_VIESTI));
    }
}

#[test]
fn haasteessa_peruutus_toimii_vaikeallakin_ja_k_palaa_listaan() {
    let mut peli = Pasianssi::new(1);
    peli.aloita_haaste(HAASTEITA - 1);
    let alku = peli.lauta.tila.clone();
    peli.kasittele(Syote::Kirjain('N'));
    assert_ne!(peli.lauta.tila, alku);
    peli.kasittele(Syote::Kirjain('U'));
    assert_eq!(peli.lauta.tila, alku);
    peli.kasittele(Syote::Kirjain('K'));
    assert_eq!(peli.vaihe, Vaihe::Haasteet);
    assert_eq!(peli.haaste, None);

    // Tavallinen peli palaa alkuruudulle kuten ennenkin.
    peli.vaihe = Vaihe::Alku;
    peli.kasittele(Syote::Ammu);
    peli.kasittele(Syote::Kirjain('K'));
    assert_eq!(peli.vaihe, Vaihe::Alku);
}

/// Laudan tila lajista riippumatta, jotta testi voi verrata jakoja.
fn laudan_tila(peli: &Pasianssi) -> String {
    match peli.variantti {
        Variantti::Klondike => format!("{:?}", peli.lauta.tila),
        Variantti::Spider => format!("{:?}", peli.spider.tila),
        Variantti::Pyramidi => format!("{:?}", peli.pyramidi.tila),
    }
}

#[test]
fn r_aloittaa_saman_jaon_alusta_joka_lajissa() {
    for variantti in Variantti::KAIKKI {
        let mut peli = Pasianssi::new(3);
        peli.variantti = variantti;
        peli.kasittele(Syote::Ammu);
        let alku = laudan_tila(&peli);
        peli.kasittele(Syote::Kirjain('N'));
        peli.edista(5000);
        assert_ne!(laudan_tila(&peli), alku, "{variantti:?}");
        peli.kasittele(Syote::Kirjain('R'));
        assert_eq!(laudan_tila(&peli), alku, "{variantti:?}");
        assert_eq!(peli.vaihe, Vaihe::Pelissa);
        assert_eq!(peli.peliaika_ms, 0);

        // Napista sama, ja tappion jälkeenkin jako alkaa alusta.
        peli.kasittele(Syote::Kirjain('N'));
        let (x, y) = match variantti {
            Variantti::Klondike => piirto::napin_keskipiste(Nappi::Alusta),
            Variantti::Spider => spider::piirto::napin_keskipiste(Nappi::Alusta),
            Variantti::Pyramidi => pyramidi::piirto::napin_keskipiste(Nappi::Alusta),
        };
        peli.siirrot_loppuivat();
        peli.kasittele(Syote::Klikkaus(x, y));
        assert_eq!(laudan_tila(&peli), alku, "{variantti:?}");
        assert_eq!(peli.vaihe, Vaihe::Pelissa);

        // Uusi peli jakaa eri kortit.
        peli.kasittele(Syote::Kirjain('K'));
        peli.kasittele(Syote::Ammu);
        assert_ne!(laudan_tila(&peli), alku, "{variantti:?}");
    }
}

#[test]
fn r_aloittaa_haasteen_alusta() {
    let mut peli = Pasianssi::new(1);
    peli.aloita_haaste(HAASTEITA - 1);
    let alku = peli.lauta.tila.clone();
    peli.kasittele(Syote::Kirjain('N'));
    peli.kasittele(Syote::Kirjain('R'));
    assert_eq!(peli.lauta.tila, alku);
    assert_eq!(peli.haaste, Some(HAASTEITA - 1));
    assert_eq!(peli.viesti, kaanna(HAASTE_VIESTI));
}

#[test]
fn jumiutunut_haaste_ei_paaty_tappioon_vaan_voi_peruuttaa() {
    let mut peli = Pasianssi::new(1);
    peli.aloita_haaste(HAASTEITA - 1);
    peli.lauta.tila.pakka.clear();
    peli.lauta.tila.jate = vec![kortti(Maa::Hertta, 5)];
    peli.lauta.tila.kierratykset = 2;
    peli.lauta.tila.perustat = Default::default();
    aseta(&mut peli, 0, &[kortti(Maa::Pata, 9)]);
    aseta(
        &mut peli,
        1,
        &[kortti(Maa::Risti, 5), kortti(Maa::Hertta, 1)],
    );
    for sarake in 2..SARAKKEITA {
        aseta(&mut peli, sarake, &[kortti(Maa::Pata, 9)]);
    }
    peli.kasittele(Syote::Kirjain('A'));
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
    assert_eq!(peli.viesti, kaanna(HAASTE_JUMISSA));
    peli.kasittele(Syote::Kirjain('U'));
    assert_eq!(peli.lauta.tila.perustat[0].len(), 0);
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
}

#[test]
fn haasteen_voitto_kysyy_nimen_kerran_ja_summa_korvaa_rivin() {
    let mut peli = Pasianssi::new(1);
    peli.aloita_haaste(0);
    yhden_kortin_paassa_voitosta(&mut peli);
    peli.edista(10_000);
    peli.kasittele(Syote::Kirjain('A'));
    assert_eq!(peli.vaihe, Vaihe::Nimi);
    let eka = haasteet::pisteet(0, 10);
    assert_eq!(peli.nimen_pisteet(), eka);
    peli.kasittele(Syote::Kirjain('V'));
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.vaihe, Vaihe::Voitto);
    assert_eq!(peli.haastenimi, "V");
    assert_eq!(peli.haastepisteet[0][0], eka);
    let lahetetty = peli.haastetaulu.lahetys.take().unwrap();
    assert_eq!(lahetetty.pisteet, eka);
    // Tavallisten pelien listat eivät muutu.
    assert!(peli.taulut[0].parhaat.is_empty());
    assert_eq!(peli.taulu().lahetys, None);

    // Seuraava haaste ei enää kysy nimeä, ja rivi päivittyy saman nimen alle.
    peli.kasittele(Syote::Kirjain('K'));
    assert_eq!(peli.vaihe, Vaihe::Haasteet);
    peli.aloita_haaste(HAASTEITA - 1);
    yhden_kortin_paassa_voitosta(&mut peli);
    peli.kasittele(Syote::Kirjain('A'));
    assert_eq!(peli.vaihe, Vaihe::Voitto);
    let summa = eka + haasteet::pisteet(HAASTEITA - 1, 0);
    assert_eq!(peli.haasteiden_summa(), summa);
    assert_eq!(peli.haastetaulu.lahetys.take().unwrap().pisteet, summa);
    assert_eq!(peli.haastetaulu.parhaat.len(), 1);
    assert_eq!(peli.haastetaulu.parhaat[0].pisteet, summa);
    assert_eq!(peli.lapaistyja_haasteita(), 2);
}

#[test]
fn huonompi_uusinta_ei_muuta_tulosta_eika_laheta_mitaan() {
    let mut peli = Pasianssi::new(1);
    peli.haastenimi = "V".to_string();
    peli.aloita_haaste(0);
    yhden_kortin_paassa_voitosta(&mut peli);
    peli.kasittele(Syote::Kirjain('A'));
    let paras = peli.haastepisteet[0][0];
    peli.haastetaulu.lahetys = None;
    peli.uusi_tulos = false;

    peli.kasittele(Syote::Kirjain('K'));
    peli.aloita_haaste(0);
    yhden_kortin_paassa_voitosta(&mut peli);
    peli.edista(60_000);
    peli.kasittele(Syote::Kirjain('A'));
    assert_eq!(peli.vaihe, Vaihe::Voitto);
    assert_eq!(peli.haastepisteet[0][0], paras);
    assert_eq!(peli.haastetaulu.lahetys, None);
    assert!(!peli.uusi_tulos);
}

#[test]
fn listalle_yltamaton_summa_sailyy_mutta_ei_kysy_nimea() {
    let mut peli = Pasianssi::new(1);
    peli.haastetaulu.netti = Some(vec![
        Tulos {
            nimi: "A".to_string(),
            pisteet: 600_000,
        };
        3
    ]);
    peli.aloita_haaste(0);
    yhden_kortin_paassa_voitosta(&mut peli);
    peli.kasittele(Syote::Kirjain('A'));
    assert_eq!(peli.vaihe, Vaihe::Voitto);
    assert!(peli.haastepisteet[0][0] > 0);
    assert!(peli.uusi_tulos);
    assert_eq!(peli.haastetaulu.lahetys, None);
}

#[test]
fn laji_vaihtuu_alkuruudulla_ja_jokaisella_on_oma_listansa() {
    let mut peli = Pasianssi::new(1);
    peli.taulut[Variantti::Spider as usize].parhaat = vec![Tulos {
        nimi: "S".to_string(),
        pisteet: 5,
    }];
    assert!(peli.nakyva_lista().is_empty());
    peli.kasittele(Syote::Nuoli(1, 0));
    assert_eq!(peli.variantti, Variantti::Spider);
    assert_eq!(peli.nakyva_lista()[0].nimi, "S");
    peli.kasittele(Syote::Nuoli(1, 0));
    assert_eq!(peli.variantti, Variantti::Pyramidi);
    peli.kasittele(Syote::Nuoli(1, 0));
    assert_eq!(peli.variantti, Variantti::Pyramidi);
    assert_eq!(peli.vaikeus, Vaikeus::Normaali);

    let (x, y) = piirto::variantin_keskipiste(Variantti::Spider);
    peli.kasittele(Syote::Klikkaus(x, y));
    assert_eq!(peli.variantti, Variantti::Spider);
    assert_eq!(peli.vaihe, Vaihe::Alku);
    peli.kasittele(Syote::Numero(3));
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
    assert_eq!(peli.spider.vaikeus, Vaikeus::Vaikea);
}

#[test]
fn alussa_valitaan_taso_ja_peli_alkaa_sillä() {
    let mut peli = Pasianssi::new(1);
    peli.kasittele(Syote::Numero(3));
    assert_eq!(peli.vaikeus, Vaikeus::Vaikea);
    peli.kasittele(Syote::Nuoli(0, -1));
    assert_eq!(peli.vaikeus, Vaikeus::Normaali);
    let (x, y) = piirto::vaikeuden_keskipiste(Vaikeus::Helppo);
    peli.kasittele(Syote::Klikkaus(x, y));
    assert_eq!(peli.vaikeus, Vaikeus::Helppo);
    assert_eq!(peli.vaihe, Vaihe::Alku);
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
    assert_eq!(peli.lauta.vaikeus, Vaikeus::Helppo);
    assert_eq!(peli.aanet.jono, [(Aani::Alku, 0)]);
}

#[test]
fn veto_siirtaa_kortin_ja_laiton_pudotus_ahkaisee() {
    let mut peli = kaksi_saraketta();
    let (x, y) = kortin_keskipiste(&peli, 0, 0);
    peli.kasittele(Syote::Klikkaus(x, y));
    assert!(peli.pito.is_some());
    let (kx, ky) = kortin_keskipiste(&peli, 1, 0);
    peli.kasittele(Syote::Hiiri(kx, ky));
    assert!(peli.pito.is_some_and(|pito| pito.liikkui));
    peli.kasittele(Syote::Vapautus(kx, ky));
    assert_eq!(peli.lauta.tila.sarakkeet[1].kortit.len(), 2);
    assert!(peli.pito.is_none());

    let mut peli = pelissa();
    aseta(&mut peli, 0, &[kortti(Maa::Pata, 6)]);
    aseta(&mut peli, 1, &[kortti(Maa::Pata, 7)]);
    let (x, y) = kortin_keskipiste(&peli, 0, 0);
    peli.kasittele(Syote::Klikkaus(x, y));
    let (kx, ky) = kortin_keskipiste(&peli, 1, 0);
    peli.kasittele(Syote::Hiiri(kx, ky));
    peli.kasittele(Syote::Vapautus(kx, ky));
    assert_eq!(peli.lauta.tila.sarakkeet[0].kortit.len(), 1);
    assert_eq!(peli.viesti, kaanna(EI_KAY));
    assert!(peli.aanet.jono.contains(&(Aani::Virhe, 0)));
}

#[test]
fn klikkaus_valitsee_ja_toinen_klikkaus_siirtaa() {
    let mut peli = kaksi_saraketta();
    let (x, y) = kortin_keskipiste(&peli, 0, 0);
    peli.kasittele(Syote::Klikkaus(x, y));
    peli.kasittele(Syote::Vapautus(x, y));
    assert!(peli.valittu.is_some());

    let (kx, ky) = kortin_keskipiste(&peli, 1, 0);
    peli.kasittele(Syote::Klikkaus(kx, ky));
    assert_eq!(peli.lauta.tila.sarakkeet[1].kortit.len(), 2);
    assert!(peli.valittu.is_none());
}

#[test]
fn tuplaklikkaus_vie_perustalle_ja_hidas_ei() {
    let mut peli = pelissa();
    aseta(&mut peli, 0, &[kortti(Maa::Risti, 1)]);
    let (x, y) = kortin_keskipiste(&peli, 0, 0);
    peli.kasittele(Syote::Klikkaus(x, y));
    peli.kasittele(Syote::Vapautus(x, y));
    peli.edista(TUPLAKLIKKAUS_MS + 1);
    peli.kasittele(Syote::Klikkaus(x, y));
    peli.kasittele(Syote::Vapautus(x, y));
    assert_eq!(peli.lauta.tila.perustat[Maa::Risti as usize].len(), 0);

    peli.edista(100);
    peli.kasittele(Syote::Klikkaus(x, y));
    assert_eq!(peli.lauta.tila.perustat[Maa::Risti as usize].len(), 1);
}

#[test]
fn pakan_klikkaus_ja_n_nostavat_ja_lopussa_tulee_viesti() {
    let mut peli = Pasianssi::new(1);
    peli.vaikeus = Vaikeus::Vaikea;
    peli.kasittele(Syote::Ammu);
    let (x, y) = piirto::kortin_kulma_pakka();
    peli.kasittele(Syote::Klikkaus(x + 5, y + 5));
    assert_eq!(peli.lauta.tila.jate.len(), 3);
    peli.kasittele(Syote::Kirjain('N'));
    assert_eq!(peli.lauta.tila.jate.len(), 6);
    for _ in 0..100 {
        peli.kasittele(Syote::Kirjain('N'));
    }
    assert_eq!(peli.viesti, kaanna(PAKKA_LOPPU));
}

#[test]
fn kursori_liikkuu_jonon_ylos_ja_ylarivi_ohittaa_tyhjan_paikan() {
    let mut peli = pelissa();
    peli.kasittele(Syote::Nuoli(0, 1));
    assert_eq!(peli.kursorin_kohta(), Some(Kohta::Jate));
    peli.kasittele(Syote::Nuoli(0, 1));
    assert_eq!(peli.kursorin_kohta(), Some(Kohta::Perusta(0)));
    peli.kasittele(Syote::Nuoli(0, -1));
    assert_eq!(peli.kursorin_kohta(), Some(Kohta::Jate));

    aseta(
        &mut peli,
        1,
        &[kortti(Maa::Hertta, 9), kortti(Maa::Pata, 8)],
    );
    peli.kasittele(Syote::Nuoli(1, 0));
    let kohta = |kortti| Kohta::Sarake { sarake: 1, kortti };
    assert_eq!(peli.kursorin_kohta(), Some(kohta(Some(1))));
    peli.kasittele(Syote::Nuoli(-1, 0));
    assert_eq!(peli.kursorin_kohta(), Some(kohta(Some(0))));
    peli.kasittele(Syote::Nuoli(-1, 0));
    assert_eq!(peli.kursorin_kohta(), Some(Kohta::Jate));
}

#[test]
fn nappaimilla_siirto_ja_toinen_painallus_peruu_valinnan() {
    let mut peli = kaksi_saraketta();
    peli.kasittele(Syote::Nuoli(1, 0));
    peli.kasittele(Syote::Ammu);
    assert!(peli.valittu.is_some());
    peli.kasittele(Syote::Ammu);
    assert!(peli.valittu.is_none());

    peli.kasittele(Syote::Ammu);
    peli.kasittele(Syote::Nuoli(0, 1));
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.lauta.tila.sarakkeet[1].kortit.len(), 2);
}

#[test]
fn esc_peruu_ensin_valinnan_ja_sitten_palaa_valikkoon() {
    let mut peli = kaksi_saraketta();
    peli.kasittele(Syote::Nuoli(1, 0));
    peli.kasittele(Syote::Ammu);
    peli.kasittele(Syote::Peru);
    assert!(peli.valittu.is_none());
    assert!(!peli.lopeta);
    peli.kasittele(Syote::Peru);
    assert!(peli.lopeta);
}

#[test]
fn peruutus_ja_keraa_toimivat_napeista_ja_vaikealla_ei_voi_perua() {
    let mut peli = kaksi_saraketta();
    aseta(&mut peli, 2, &[kortti(Maa::Risti, 1)]);
    let (x, y) = piirto::napin_keskipiste(Nappi::Keraa);
    peli.kasittele(Syote::Klikkaus(x, y));
    assert_eq!(peli.lauta.tila.perustat[Maa::Risti as usize].len(), 1);
    let (x, y) = piirto::napin_keskipiste(Nappi::Peruuta);
    peli.kasittele(Syote::Klikkaus(x, y));
    assert_eq!(peli.lauta.tila.perustat[Maa::Risti as usize].len(), 0);

    let mut peli = Pasianssi::new(1);
    peli.vaikeus = Vaikeus::Vaikea;
    peli.kasittele(Syote::Ammu);
    peli.kasittele(Syote::Kirjain('N'));
    peli.kasittele(Syote::Kirjain('U'));
    assert_eq!(peli.viesti, kaanna(EI_PERUUTUSTA));
    assert_eq!(peli.lauta.tila.jate.len(), 3);
}

#[test]
fn viimeinen_kortti_voittaa_ja_kello_pysahtyy() {
    let mut peli = pelissa();
    for (i, &maa) in Maa::KAIKKI.iter().enumerate() {
        peli.lauta.tila.perustat[i] = (1..=13).map(|arvo| kortti(maa, arvo)).collect();
    }
    let viimeinen = peli.lauta.tila.perustat[0].pop().unwrap();
    aseta(&mut peli, 0, &[viimeinen]);
    peli.edista(1000);
    peli.kasittele(Syote::Kirjain('A'));
    assert_eq!(peli.vaihe, Vaihe::Nimi);
    assert!(peli.aanet.jono.contains(&(Aani::Voitto, VOITTO_VIIVE_MS)));
    peli.edista(1000);
    assert_eq!(peli.peliaika_ms, 1000);

    // Tulos riippuu vain ajasta, ja nimi kirjoitetaan ennen kuin loppu kuittaa mitään.
    let pisteet = aikapisteet(Vaikeus::Normaali, 1);
    assert_eq!(peli.viesti, muotoile(VOITTOVIESTI, &[&pisteet]));
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.vaihe, Vaihe::Nimi);
    peli.kasittele(Syote::Kirjain('K'));
    assert_eq!(peli.nimi, "K");
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.vaihe, Vaihe::Voitto);
    assert!(peli.uusi_tulos);
    assert_eq!(
        peli.taulut[Variantti::Klondike as usize].parhaat[0].pisteet,
        pisteet
    );
    assert_eq!(peli.taulu().lahetys.as_ref().unwrap().nimi, "K");

    peli.kasittele(Syote::Kirjain('K'));
    assert_eq!(peli.vaihe, Vaihe::Alku);
    peli.vaihe = Vaihe::Voitto;
    peli.kasittele(Syote::Kirjain('E'));
    assert!(peli.lopeta);
}

#[test]
fn listalle_yltamaton_voitto_ja_keskeytys_eivat_kysy_nimea() {
    let mut peli = pelissa();
    peli.taulut[Variantti::Klondike as usize].netti = Some(vec![
        Tulos {
            nimi: "A".to_string(),
            pisteet: 100_000,
        };
        3
    ]);
    for (i, &maa) in Maa::KAIKKI.iter().enumerate() {
        peli.lauta.tila.perustat[i] = (1..=13).map(|arvo| kortti(maa, arvo)).collect();
    }
    let viimeinen = peli.lauta.tila.perustat[0].pop().unwrap();
    aseta(&mut peli, 0, &[viimeinen]);
    peli.kasittele(Syote::Kirjain('A'));
    assert_eq!(peli.vaihe, Vaihe::Voitto);
    assert_eq!(peli.taulu().lahetys, None);
    // Listalle yltämätönkin voitto on oma ennätys, ja se tallennetaan.
    let pisteet = aikapisteet(Vaikeus::Normaali, 0);
    assert_eq!(peli.taulut.each_ref().map(|t| t.paras), [pisteet, 0, 0]);
    assert!(peli.uusi_tulos);

    let mut peli = pelissa();
    peli.kasittele(Syote::Peru);
    assert!(peli.lopeta);
    assert_eq!(peli.taulu().lahetys, None);
}

#[test]
fn pisteet_laskevat_ajan_kuluessa_ja_havio_ei_anna_mitaan() {
    let mut peli = pelissa();
    let alussa = peli.pisteet();
    peli.edista(1000);
    assert!(peli.pisteet() < alussa);
    peli.vaihe = Vaihe::Tappio;
    assert_eq!(peli.pisteet(), 0);
}

#[test]
fn peli_loppuu_kun_siirrot_loppuvat_ja_kello_pysahtyy() {
    let mut peli = pelissa();
    peli.lauta.vaikeus = Vaikeus::Vaikea;
    peli.lauta.tila.kierratykset = 2;
    peli.lauta.tila.jate = vec![kortti(Maa::Hertta, 5)];
    aseta(&mut peli, 0, &[kortti(Maa::Pata, 9)]);
    aseta(
        &mut peli,
        1,
        &[kortti(Maa::Risti, 5), kortti(Maa::Hertta, 1)],
    );
    peli.edista(1000);
    peli.kasittele(Syote::Kirjain('A'));
    assert_eq!(peli.vaihe, Vaihe::Tappio);
    assert_eq!(peli.viesti, kaanna(SIIRROT_LOPPU));
    assert!(peli.aanet.jono.contains(&(Aani::Tappio, VOITTO_VIIVE_MS)));
    peli.edista(1000);
    assert_eq!(peli.peliaika_ms, 1000);

    // Lopussa kortteja ei enää liikuteta, vain uusi peli tai valikko.
    peli.kasittele(Syote::Kirjain('N'));
    assert_eq!(peli.lauta.tila.jate.len(), 1);
    peli.kasittele(Syote::Kirjain('K'));
    assert_eq!(peli.vaihe, Vaihe::Alku);
}

#[test]
fn jonon_osan_siirto_pitaa_pelin_kaynnissa_ja_vihje_nakyy() {
    let mut peli = pelissa();
    aseta(
        &mut peli,
        0,
        &[
            kortti(Maa::Hertta, 9),
            kortti(Maa::Pata, 8),
            kortti(Maa::Hertta, 7),
        ],
    );
    aseta(&mut peli, 1, &[kortti(Maa::Risti, 8)]);
    aseta(&mut peli, 2, &[kortti(Maa::Hertta, 1)]);
    peli.kasittele(Syote::Kirjain('A'));
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
    assert_eq!(peli.viesti, kaanna(JUMISSA));
}
