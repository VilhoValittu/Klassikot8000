use super::*;

const KESKEN: Kesken = Kesken {
    ruudut: [[2, 4, 8, 16], [0; 4], [0, 1024, 0, 0], [0; 4]],
    pisteet: 1234,
    siirrot: 56,
    jatkettu: true,
};

fn tekstista(teksti: &str) -> Tallennus {
    Tallennus::tekstista(teksti).expect("otsake kelpaa")
}

fn tulos(nimi: &str, pisteet: u32) -> Tulos {
    Tulos {
        nimi: nimi.to_string(),
        pisteet,
        aika: None,
    }
}

fn kokonainen() -> Tallennus {
    let mut t = Tallennus {
        hiljaa: true,
        kieli: Kieli::Englanti,
        asetukset: Asetukset {
            valittu: 7,
            laivanupotus: laivanupotus::Vaikeus::Vaikea,
            pasianssi: pasianssi::Vaikeus::Helppo,
            variantti: Variantti::Pyramidi,
            miinaharava: miinaharava::Vaikeus::Normaali,
            mahjong: mahjong::Vaikeus::Vaikea,
            majakka: majakka::Vaikeus::Helppo,
            kaivo: kaivo::Vaikeus::Vaikea,
            jatsi: jatsi::Vaikeus::Helppo,
            suora: suora::Vaikeus::Vaikea,
        },
        kesken: Some(KESKEN),
        haastenimi: "HAASTAJA".to_string(),
        miinaajat: [12, 0, 345],
        ..Tallennus::default()
    };
    t.jatsipaikat = jatsi_peli::paikat_riveista(&["0 I VÄINÖ", "1 K", "2 I B", "3 -"]).unwrap();
    t.jatsikesken = JatsiKesken::riveista(
        "1,2,66123,10100",
        &[
            "I 3,-,-,-,-,-,-,-,-,-,-,-,-,-,50 VÄINÖ",
            "K -,-,-,-,-,-,12,-,-,-,-,-,-,-,-",
        ],
    );
    assert!(t.jatsikesken.is_some());
    t.haastepisteet[1][3] = 3500;
    t.haastepisteet[2][19] = 3500;
    t.laivatilasto[2] = Tilasto {
        voitot: 4,
        tappiot: 9,
        vahiten_laukauksia: 41,
    };
    // Jokaiselle listalle omat arvot, jotta sekaantuneet avaimet huomataan.
    for (i, peli) in Nettipeli::KAIKKI.into_iter().enumerate() {
        let pisteet = 100 * (i as u32 + 1);
        let lista = t.lista_mut(peli);
        if avaimet(peli).0.is_some() {
            lista.paras = pisteet;
        }
        // Vanhoista tallennuksista luetuilla tuloksilla ei ole aikaa.
        let vaino = Tulos {
            aika: Some(1_763_000_000 + i as u64),
            ..tulos("VÄINÖ", pisteet)
        };
        let nimeton = Tulos {
            aika: Some(1_763_000_100),
            ..tulos("NIMETÖN", pisteet + 1)
        };
        lista.parhaat = vec![vaino, tulos("A1", 3)];
        lista.jono = vec![nimeton];
    }
    t
}

#[test]
fn teksti_palautuu_samaksi() {
    let t = kokonainen();
    assert_eq!(tekstista(&t.tekstiksi()), t);
    let mut t = Tallennus::default();
    t.lista_mut(Nettipeli::Luvut).paras = 7;
    assert_eq!(tekstista(&t.tekstiksi()), t);
}

#[test]
fn vanhan_version_avaimet_luetaan_oikeille_listoille() {
    let t = tekstista(
        "klassikot8000 1\nparas 7\ntop 9 A\nlukujono 8 B\ntiilirikko 6\ntiilitop 5 C\n\
             tiilijono 4 D\nasteroidit 3\nsilmukkatop 2 E\npalikkajono 1 F\nspidertop 11 G\n\
             haastejono 12 H\nholdemtop 13 I\n",
    );
    let luvut = t.lista(Nettipeli::Luvut);
    assert_eq!(luvut.paras, 7);
    assert_eq!(luvut.parhaat, [tulos("A", 9)]);
    assert_eq!(luvut.jono, [tulos("B", 8)]);
    let tiili = t.lista(Nettipeli::Tiilirikko);
    assert_eq!(tiili.paras, 6);
    assert_eq!(tiili.parhaat, [tulos("C", 5)]);
    assert_eq!(tiili.jono, [tulos("D", 4)]);
    assert_eq!(t.lista(Nettipeli::Asteroidit).paras, 3);
    assert_eq!(t.lista(Nettipeli::Silmukka).parhaat, [tulos("E", 2)]);
    assert_eq!(t.lista(Nettipeli::Palikka).jono, [tulos("F", 1)]);
    assert_eq!(t.lista(Nettipeli::Spider).parhaat, [tulos("G", 11)]);
    assert_eq!(t.lista(Nettipeli::Haasteet).jono, [tulos("H", 12)]);
    assert_eq!(t.lista(Nettipeli::Holdem).parhaat, [tulos("I", 13)]);
    assert!(t.lista(Nettipeli::Pasianssi).parhaat.is_empty());
    // asetukset puuttuvat vanhasta tiedostosta
    assert_eq!(t.asetukset, Asetukset::default());
}

#[test]
fn vieras_tai_tyhja_tiedosto_ei_kelpaa() {
    assert_eq!(Tallennus::tekstista(""), None);
    assert_eq!(Tallennus::tekstista("paras 99\n"), None);
    assert_eq!(Tallennus::tekstista("klassikot8000 2\nparas 99\n"), None);
}

#[test]
fn muistion_bom_ei_haittaa() {
    let t = tekstista("\u{feff}klassikot8000 1\nparas 7\n");
    assert_eq!(t.lista(Nettipeli::Luvut).paras, 7);
}

#[test]
fn kelvoton_tulos_hylataan_ja_muokattu_lista_jarjestetaan() {
    let t = tekstista(
        "klassikot8000 1\ntop 5 PIENI\ntop 9 ISO\ntop 0 NOLLA\ntop x KISSA\n\
             top 7 pieni\ntop 7 LIIANPITKA\ntop 7\ntop 7 KAKSI OSAA\n",
    );
    let parhaat = &t.lista(Nettipeli::Luvut).parhaat;
    let pisteet: Vec<u32> = parhaat.iter().map(|t| t.pisteet).collect();
    assert_eq!(pisteet, [9, 5]);
    assert_eq!(parhaat[0].nimi, "ISO");
}

#[test]
fn vanhassa_tallennuksessa_aanet_ovat_paalla() {
    let t = tekstista("klassikot8000 1\nparas 7\n");
    assert_eq!(t.lista(Nettipeli::Luvut).paras, 7);
    assert!(!t.hiljaa);
    for peli in [
        Nettipeli::Tiilirikko,
        Nettipeli::Asteroidit,
        Nettipeli::Silmukka,
        Nettipeli::Palikka,
    ] {
        assert_eq!(t.lista(peli).paras, 0);
    }
    assert_eq!(t.miinaajat, [0; 3]);
    assert!(t.lista(Nettipeli::Luvut).parhaat.is_empty());
}

#[test]
fn kelvottomat_asetukset_hylataan() {
    let t = tekstista(
        "klassikot8000 1\nlaivataso 3\npasianssitaso -1\npasianssilaji x\nmiinataso 9\n\
             valittu kissa\n",
    );
    assert_eq!(t.asetukset, Asetukset::default());
    let t = tekstista("klassikot8000 1\nlaivataso 0\nmiinataso 2\nvalittu 4\n");
    assert_eq!(t.asetukset.laivanupotus, laivanupotus::Vaikeus::Helppo);
    assert_eq!(t.asetukset.miinaharava, miinaharava::Vaikeus::Vaikea);
    assert_eq!(t.asetukset.valittu, 4);
}

#[test]
fn kelvottomat_haastetiedot_hylataan() {
    let nollia = vec!["0"; HAASTEITA].join(",");
    let hyva = format!("klassikot8000 1\nhaastepisteet 2 5,{}\n", &nollia[2..]);
    assert_eq!(tekstista(&hyva).haastepisteet[2][0], 5);
    for huono in [
        format!("haastepisteet 3 {nollia}"),
        format!("haastepisteet 0 {nollia},0"),
        "haastepisteet 0 1,2".to_string(),
        format!("haastepisteet 0 99999999,{}", &nollia[2..]),
    ] {
        let t = tekstista(&format!("klassikot8000 1\n{huono}\n"));
        assert_eq!(t.haastepisteet, Haastepisteet::default(), "{huono}");
    }
    let t = tekstista("klassikot8000 1\nhaastenimi pieni\n");
    assert!(t.haastenimi.is_empty());
}

#[test]
fn kelvottomat_miina_ajat_ja_tilastot_hylataan() {
    let hyva = "klassikot8000 1\nmiinaajat 1,2,3\n";
    assert_eq!(tekstista(hyva).miinaajat, [1, 2, 3]);
    for huono in ["1,2", "1,2,3,4", "a,b,c", "-1,2,3", "1,2,4000000000"] {
        let t = tekstista(&hyva.replace("1,2,3", huono));
        assert_eq!(t.miinaajat, [0; 3], "{huono}");
    }
    for huono in ["3 1,2,3", "0 1,2", "x 1,2,3", "1 1,2,3,4", "1 -1,2,3"] {
        let t = tekstista(&format!("klassikot8000 1\nlaivatilasto {huono}\n"));
        assert_eq!(t.laivatilasto, [Tilasto::default(); 3], "{huono}");
    }
}

#[test]
fn kelvoton_lauta_hylataan_mutta_paras_sailyy() {
    let mut alku = kokonainen();
    alku.lista_mut(Nettipeli::Luvut).paras = 5000;
    let hyva = alku.tekstiksi();
    let lauta = "2,4,8,16,0,0,0,0,0,1024,0,0,0,0,0,0";
    for huono in [
        "3,4,8,16,0,0,0,0,0,1024,0,0,0,0,0,0",
        "1,4,8,16,0,0,0,0,0,1024,0,0,0,0,0,0",
        "4294967296,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0",
        "2147483648,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0",
        "2,4,8",
        "2,4,8,16,0,0,0,0,0,1024,0,0,0,0,0,0,0",
        "kissa",
        // täysi lauta ilman siirtoja
        "2,4,2,4,4,2,4,2,2,4,2,4,4,2,4,2",
    ] {
        let t = tekstista(&hyva.replace(lauta, huono));
        assert_eq!(t.lista(Nettipeli::Luvut).paras, 5000, "{huono}");
        assert_eq!(t.kesken, None, "{huono}");
    }
    let t = tekstista(&hyva.replace("paras 5000", "paras -1"));
    assert_eq!(t.lista(Nettipeli::Luvut).paras, 0);
    assert!(t.kesken.is_some());
}

#[test]
fn tiedosto_luodaan_ja_luetaan_ja_puuttuva_on_tyhja() {
    let kansio = std::env::temp_dir().join(format!("k8000_{}", std::process::id()));
    let polku = kansio.join("ala").join("tallennus.txt");
    assert_eq!(lue_tiedosto(&polku).unwrap(), Tallennus::default());
    kirjoita_tiedosto(&polku, &kokonainen()).unwrap();
    kirjoita_tiedosto(&polku, &kokonainen()).unwrap();
    assert_eq!(lue_tiedosto(&polku).unwrap(), kokonainen());
    // Ehjästä tiedostosta ei tehdä kopiota.
    assert_eq!(fs::read_dir(polku.parent().unwrap()).unwrap().count(), 1);
    fs::remove_dir_all(&kansio).unwrap();
}

/// Kansion tiedostot, joiden nimi päättyy `.bak`-päätteeseen, sisältöineen.
fn kopiot(kansio: &Path) -> Vec<Vec<u8>> {
    fs::read_dir(kansio)
        .unwrap()
        .map(|tiedosto| tiedosto.unwrap().path())
        .filter(|polku| polku.extension().is_some_and(|paate| paate == "bak"))
        .map(|polku| fs::read(polku).unwrap())
        .collect()
}

#[test]
fn kelvoton_tiedosto_kopioidaan_talteen_ennen_tyhjaa_tallennusta() {
    let kansio = std::env::temp_dir().join(format!("k8000_kopio_{}", std::process::id()));
    fs::create_dir_all(&kansio).unwrap();
    let polku = kansio.join("tallennus.txt");
    // Uudemman version otsake ja Muistion ANSI-merkistöllä tallentama Ä, joka ei ole UTF-8:aa.
    for sisalto in [
        &b"klassikot8000 2\nparas 900\n"[..],
        b"klassikot8000 1\ntop 9 \xC4\n",
    ] {
        fs::write(&polku, sisalto).unwrap();
        assert_eq!(lue_tiedosto(&polku).unwrap(), Tallennus::default());
        assert_eq!(kopiot(&kansio), [sisalto]);
        // Alkuperäinen jää paikalleen, kunnes peli kirjoittaa omansa sen päälle.
        assert_eq!(fs::read(&polku).unwrap(), sisalto);
        for kopio in fs::read_dir(&kansio).unwrap() {
            let kopio = kopio.unwrap().path();
            if kopio != polku {
                fs::remove_file(kopio).unwrap();
            }
        }
    }
    fs::remove_dir_all(&kansio).unwrap();
}

#[test]
fn lukukelvotonta_tiedostoa_ei_saa_kirjoittaa_yli() {
    // Kansio tallennuksen paikalla: lukeminen ja kopiointi epäonnistuvat kaikilla alustoilla.
    let kansio = std::env::temp_dir().join(format!("k8000_lukossa_{}", std::process::id()));
    let polku = kansio.join("tallennus.txt");
    fs::create_dir_all(&polku).unwrap();
    assert!(lue_tiedosto(&polku).is_err());
    fs::remove_dir_all(&kansio).unwrap();
}

#[test]
fn tallennuksen_lukon_saa_vain_yksi_kerrallaan() {
    let kansio = std::env::temp_dir().join(format!("k8000_lukko_{}", std::process::id()));
    // Kansiota ei vielä ole: lukitus luo sen, kuten ensimmäisellä käynnistyksellä.
    let polku = kansio.join("uusi").join("tallennus.txt");
    let Lukitus::Oma(lukko) = lukitse(&polku) else {
        panic!("vapaa lukko on saatava");
    };
    assert!(matches!(lukitse(&polku), Lukitus::Varattu));
    // Lukko ei estä tallennuksen kirjoittamista eikä lukemista.
    kirjoita_tiedosto(&polku, &kokonainen()).unwrap();
    assert_eq!(lue_tiedosto(&polku).unwrap(), kokonainen());
    drop(lukko);
    assert!(matches!(lukitse(&polku), Lukitus::Oma(_)));
    fs::remove_dir_all(&kansio).unwrap();
}

#[test]
fn lukko_jota_ei_voi_tehda_ei_ole_varattu() {
    // Tiedosto kansion paikalla: lukkotiedostoa ei voi luoda millään alustalla.
    let este = std::env::temp_dir().join(format!("k8000_eilukkoa_{}", std::process::id()));
    fs::write(&este, "").unwrap();
    match lukitse(&este.join("tallennus.txt")) {
        Lukitus::Puuttuu(virhe) => assert!(!virhe.to_string().is_empty()),
        muu => panic!("lukkoa ei pitänyt syntyä: {muu:?}"),
    }
    fs::remove_file(&este).unwrap();
}
