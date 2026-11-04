use klassikot8000::Arpa;
use klassikot8000::mahjong::{
    BONUSPARIN_PISTEET, KETJU_MS, LAJEJA, Laatta, Lauta, PARIN_PISTEET, Paikka, SEKOITUKSEN_HINTA,
    SUURIN_KETJU, Sarja, TYHJENNYSBONUS, Tila, VIHJEEN_HINTA, Vaikeus, suurin_tulos,
};

#[test]
fn pohjissa_on_parillinen_maara_laattoja_eika_paallekkaisia() {
    let koot: Vec<usize> = Vaikeus::KAIKKI.iter().map(|t| t.pohja().len()).collect();
    assert_eq!(koot, [72, 124, 144]);
    for taso in Vaikeus::KAIKKI {
        let pohja = taso.pohja();
        for (i, p) in pohja.iter().enumerate() {
            for q in &pohja[i + 1..] {
                let sama_kerros = p.z == q.z;
                let limittain = (p.x - q.x).abs() < 2 && (p.y - q.y).abs() < 2;
                assert!(!(sama_kerros && limittain), "{taso:?}: {p:?} ja {q:?}");
            }
            // Ylempi laatta lepää aina alemman päällä.
            if p.z > 0 {
                let alla = pohja
                    .iter()
                    .any(|q| q.z + 1 == p.z && (p.x - q.x).abs() < 2 && (p.y - q.y).abs() < 2);
                assert!(alla, "{taso:?}: {p:?} leijuu");
            }
        }
    }
}

#[test]
fn lajit_ja_parit() {
    let sarjat: Vec<Sarja> = (0..LAJEJA).map(|l| Laatta(l).sarja()).collect();
    assert_eq!(sarjat.iter().filter(|&&s| s == Sarja::Pallot).count(), 9);
    assert_eq!(sarjat.iter().filter(|&&s| s == Sarja::Tuulet).count(), 4);
    assert_eq!(
        sarjat.iter().filter(|&&s| s == Sarja::Lohikaarmeet).count(),
        3
    );
    assert_eq!(Laatta(0).arvo(), 1);
    assert_eq!(Laatta(26).arvo(), 9);
    assert_eq!(Laatta(41).arvo(), 4);
    assert!(Laatta(5).sopii(Laatta(5)));
    assert!(!Laatta(5).sopii(Laatta(6)));
    // Kukat käyvät toisilleen, mutta eivät vuodenajoille.
    assert!(Laatta(34).sopii(Laatta(37)));
    assert!(Laatta(38).sopii(Laatta(41)));
    assert!(!Laatta(37).sopii(Laatta(38)));
}

/// Jokainen jako ratkeaa poistojärjestyksellään, ja täysi peli käyttää jokaista lajia neljästi.
#[test]
fn jako_ratkeaa_aina_omalla_ratkaisullaan() {
    for taso in Vaikeus::KAIKKI {
        for siemen in 1..=40 {
            let mut arpa = Arpa::new(siemen);
            let mut lauta = Lauta::new(taso, &mut arpa);
            assert_eq!(lauta.jaljella(), lauta.paikat.len());
            assert_eq!(lauta.tila, Tila::Kesken);
            if taso == Vaikeus::Vaikea {
                let mut maarat = [0; LAJEJA as usize];
                for laatta in lauta.laatat.iter().flatten() {
                    maarat[laatta.0 as usize] += 1;
                }
                assert!(maarat[..34].iter().all(|&m| m == 4), "{maarat:?}");
                assert!(maarat[34..].iter().all(|&m| m == 1), "{maarat:?}");
            }
            let ratkaisu = lauta.ratkaisu.clone();
            assert_eq!(ratkaisu.len(), lauta.paikat.len() / 2);
            for (i, &(a, b)) in ratkaisu.iter().enumerate() {
                assert!(
                    lauta.poista(a, b, i as u32 * 60_000).is_some(),
                    "{taso:?} siemen {siemen}: pari {i}"
                );
            }
            assert_eq!(lauta.tila, Tila::Voitto);
        }
    }
}

/// Laatat vierekkäin samalla rivillä: vain päät ovat vapaita.
fn rivi(laatat: &[u8]) -> (Vec<Paikka>, Vec<Option<Laatta>>) {
    let paikat = (0..laatat.len())
        .map(|i| Paikka {
            x: 2 * i as i8,
            y: 0,
            z: 0,
        })
        .collect();
    (paikat, laatat.iter().map(|&l| Some(Laatta(l))).collect())
}

/// Laatat erillään toisistaan: kaikki ovat vapaita.
fn erilliset(laatat: &[u8]) -> (Vec<Paikka>, Vec<Option<Laatta>>) {
    let (mut paikat, laatat) = rivi(laatat);
    for (i, paikka) in paikat.iter_mut().enumerate() {
        paikka.y = 4 * i as i8;
    }
    (paikat, laatat)
}

#[test]
fn vain_vapaa_laatta_kelpaa() {
    // Rivi A B A B ja päällä laatta, joka peittää kolmannen.
    let (mut paikat, mut laatat) = rivi(&[1, 2, 1, 2]);
    paikat.push(Paikka { x: 4, y: 0, z: 1 });
    laatat.push(Some(Laatta(7)));
    paikat.push(Paikka { x: 4, y: 0, z: 2 });
    laatat.push(Some(Laatta(7)));
    let mut lauta = Lauta::laatoista(Vaikeus::Helppo, paikat, laatat);
    // Päät ovat vapaita, keskimmäiset eivät.
    assert_eq!(lauta.vapaat(), [true, false, false, true, false, true]);
    assert!(!lauta.kelpaa(0, 2));
    assert!(!lauta.kelpaa(1, 3));
    // Päällekkäiset samat laatat: alempi ei ole vapaa.
    assert!(!lauta.kelpaa(4, 5));
    assert_eq!(lauta.vapaat_parit(), []);
    assert_eq!(lauta.tila, Tila::Jumissa);
    assert_eq!(lauta.poista(0, 3, 0), None);
    assert_eq!(lauta.pisteet, 0);
    assert!(!lauta.kelpaa(0, 0));
}

#[test]
fn ketju_kasvaa_nopeista_pareista_ja_katkeaa_viiveesta() {
    let (paikat, laatat) = erilliset(&[1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 34, 35]);
    let mut lauta = Lauta::laatoista(Vaikeus::Normaali, paikat, laatat);
    let kerroin = Vaikeus::Normaali.kerroin();
    assert_eq!(lauta.poista(0, 1, 0), Some(PARIN_PISTEET * kerroin));
    assert_eq!(lauta.ketju, 1);
    assert_eq!(
        lauta.poista(2, 3, KETJU_MS),
        Some(2 * PARIN_PISTEET * kerroin)
    );
    let mut aika = KETJU_MS;
    for (a, b) in [(4, 5), (6, 7), (8, 9), (10, 11)] {
        aika += 1000;
        lauta.poista(a, b, aika);
    }
    assert_eq!(lauta.ketju, SUURIN_KETJU);
    // Viive katkaisee ketjun.
    assert_eq!(
        lauta.poista(12, 13, aika + KETJU_MS + 1),
        Some(PARIN_PISTEET * kerroin)
    );
    // Kukkapari on arvokkaampi.
    let ennen = lauta.pisteet;
    let saatu = lauta.poista(14, 15, aika + KETJU_MS + 2).unwrap();
    assert_eq!(saatu, 2 * BONUSPARIN_PISTEET * kerroin);
    assert_eq!(lauta.tila, Tila::Voitto);
    let aikabonus = (Vaikeus::Normaali.aikaraja_s() - (aika + KETJU_MS + 2) / 1000) * kerroin;
    assert_eq!(lauta.bonus.tyhjennys, TYHJENNYSBONUS * kerroin);
    assert_eq!(lauta.bonus.aika, aikabonus);
    assert_eq!(
        lauta.pisteet,
        ennen + saatu + TYHJENNYSBONUS * kerroin + aikabonus
    );
    // Päättynyttä peliä ei voi jatkaa.
    assert!(!lauta.peru());
    assert_eq!(lauta.vihje(), None);
}

#[test]
fn peruutus_palauttaa_parin_ja_pisteet_paitsi_vaikealla() {
    let (paikat, laatat) = erilliset(&[1, 1, 2, 2]);
    let mut lauta = Lauta::laatoista(Vaikeus::Helppo, paikat.clone(), laatat.clone());
    assert!(!lauta.voi_perua());
    lauta.poista(0, 1, 0);
    assert!(lauta.peru());
    assert_eq!(lauta.pisteet, 0);
    assert_eq!(lauta.ketju, 0);
    assert_eq!(lauta.jaljella(), 4);
    assert!(!lauta.peru());

    let mut vaikea = Lauta::laatoista(Vaikeus::Vaikea, paikat, laatat);
    vaikea.poista(0, 1, 0);
    assert!(!vaikea.voi_perua());
    assert!(!vaikea.peru());
}

#[test]
fn vihje_loytaa_parin_ja_maksaa() {
    let (paikat, laatat) = erilliset(&[3, 1, 2, 4, 1, 3]);
    let mut lauta = Lauta::laatoista(Vaikeus::Helppo, paikat, laatat);
    lauta.pisteet = 100;
    assert_eq!(lauta.vihje(), Some((0, 5)));
    assert_eq!(lauta.pisteet, 100 - VIHJEEN_HINTA);
    assert_eq!(lauta.vihjeita, 1);
    lauta.pisteet = 5;
    lauta.vihje();
    assert_eq!(lauta.pisteet, 0);
}

#[test]
fn sekoitus_avaa_jumin_ja_loppuu_lopulta() {
    // Rivi A B A B jumittuu, mutta sekoitettuna päistä löytyy pari.
    for siemen in 1..20 {
        let (paikat, laatat) = rivi(&[1, 2, 1, 2]);
        let mut lauta = Lauta::laatoista(Vaikeus::Vaikea, paikat, laatat);
        lauta.pisteet = 80;
        assert_eq!(lauta.tila, Tila::Jumissa);
        let mut arpa = Arpa::new(siemen);
        assert!(lauta.sekoita(&mut arpa));
        assert_eq!(lauta.tila, Tila::Kesken);
        assert_eq!(lauta.pisteet, 80 - SEKOITUKSEN_HINTA);
        assert_eq!(lauta.sekoituksia, 0);
        assert_eq!(lauta.jaljella(), 4);
        let mut laatat: Vec<u8> = lauta.laatat.iter().flatten().map(|l| l.0).collect();
        laatat.sort();
        assert_eq!(laatat, [1, 1, 2, 2]);
        assert!(!lauta.sekoita(&mut arpa));
    }
    // Ilman sekoituksia jumi on tappio.
    let (mut paikat, mut laatat) = rivi(&[1, 2, 1, 2]);
    let (erilliset_paikat, erilliset_laatat) = erilliset(&[3, 3]);
    for (mut paikka, laatta) in erilliset_paikat.into_iter().zip(erilliset_laatat) {
        paikka.y += 10;
        paikat.push(paikka);
        laatat.push(laatta);
    }
    let mut lauta = Lauta::laatoista(Vaikeus::Vaikea, paikat, laatat);
    lauta.sekoituksia = 0;
    assert_eq!(lauta.tila, Tila::Kesken);
    lauta.poista(4, 5, 0);
    assert_eq!(lauta.tila, Tila::Tappio);
}

#[test]
fn mahdoton_jaanne_sekoittuu_silti() {
    // Kaksi laattaa päällekkäin ei koskaan ratkea, mutta sekoitus ei saa jäädä silmukkaan.
    let paikat = vec![Paikka { x: 0, y: 0, z: 0 }, Paikka { x: 0, y: 0, z: 1 }];
    let laatat = vec![Some(Laatta(3)), Some(Laatta(3))];
    let mut lauta = Lauta::laatoista(Vaikeus::Helppo, paikat, laatat);
    assert_eq!(lauta.tila, Tila::Jumissa);
    let mut arpa = Arpa::new(5);
    while lauta.sekoita(&mut arpa) {}
    assert_eq!(lauta.tila, Tila::Tappio);
    assert_eq!(lauta.jaljella(), 2);
}

#[test]
fn suurin_tulos_mahtuu_palvelimen_rajaan() {
    assert!(suurin_tulos() <= 30_000, "{}", suurin_tulos());
}
