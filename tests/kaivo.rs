use klassikot8000::Arpa;
use klassikot8000::kaivo::{
    ALKU, ASKEL_MS, HISSI, HISSIN_MATKA_MS, KYPARIA, Kaivos, Loyto, MAA_ALKAA, PIENIN_VALO, PINTA,
    RIVEJA, Ruutu, SARAKKEITA, SUURIN_PISTEMAARA, Solu, Tapahtuma, Toimi, Vaikeus, kullan_arvo,
    luo_kentta, tuenta,
};

/// Tasainen maa ilman aarteita ja vaaroja, iso kiintiö.
fn kaivos() -> Kaivos {
    let mut k = Kaivos::new(Vaikeus::Normaali, 0, &mut Arpa::new(1));
    for y in 0..RIVEJA as i32 {
        for x in 0..SARAKKEITA as i32 {
            let solu = if y < MAA_ALKAA {
                Solu::Tyhja
            } else if y == RIVEJA as i32 - 1 {
                Solu::Kallio
            } else {
                Solu::Maa
            };
            k.aseta((x, y), solu);
        }
    }
    k.kiintio = 1000;
    k
}

fn aja(k: &mut Kaivos, ms: u32) -> Vec<Tapahtuma> {
    let mut arpa = Arpa::new(2);
    let mut tapahtumat = Vec::new();
    for _ in 0..ms / ASKEL_MS {
        tapahtumat.extend(k.askel(&mut arpa));
    }
    tapahtumat
}

/// Painallus ja aikaa toimen loppuun.
fn paina(k: &mut Kaivos, suunta: (i8, i8)) -> Vec<Tapahtuma> {
    k.paina(suunta);
    aja(k, 600)
}

fn tunneli(k: &mut Kaivos, rivi: i32, sarakkeet: std::ops::Range<i32>) {
    for x in sarakkeet {
        k.aseta((x, rivi), Solu::Tyhja);
    }
}

fn kaikki_ruudut() -> impl Iterator<Item = Ruutu> {
    (0..RIVEJA as i32).flat_map(|y| (0..SARAKKEITA as i32).map(move |x| (x, y)))
}

#[test]
fn arvottu_kentta_on_ehja_ja_kantaa_itsensa() {
    for siemen in 1..40 {
        for vaikeus in Vaikeus::KAIKKI {
            for taso in [0, 1, 3, 8] {
                let k = Kaivos::new(vaikeus, taso, &mut Arpa::new(siemen));
                for (x, y) in kaikki_ruudut() {
                    let solu = k.solu((x, y));
                    if y < MAA_ALKAA {
                        assert_eq!(solu, Solu::Tyhja);
                    }
                    if y == RIVEJA as i32 - 1 {
                        assert_eq!(solu, Solu::Kallio);
                    }
                    // Hissin ja alun ympäristö on turvallista maata.
                    if x <= 5 && (MAA_ALKAA + 1..=MAA_ALKAA + 2).contains(&y) {
                        assert_eq!(solu, Solu::Maa, "{siemen} {taso} ({x}, {y})");
                    }
                }
                for x in 0..=2 {
                    assert_eq!(k.solu((x, MAA_ALKAA)), Solu::Kallio);
                }
                assert!(k.solu((ALKU.0, ALKU.1 + 1)).kiintea());
                let tuettu = tuenta(&k.solut);
                for (x, y) in kaikki_ruudut() {
                    if k.solu((x, y)).kiintea() {
                        assert!(tuettu[y as usize][x as usize], "{siemen} {taso} ({x}, {y})");
                    }
                }
                let kultaa: u32 = kaikki_ruudut()
                    .filter(|&p| k.solu(p) == Solu::Kulta)
                    .map(|(_, y)| kullan_arvo(y) * vaikeus.kerroin())
                    .sum();
                assert!(k.kiintio > 0 && k.kiintio <= kultaa, "{siemen} {taso}");
                assert_eq!(k.tuet, vaikeus.tukia());
                assert_eq!(k.kyparat, KYPARIA);
            }
        }
    }
}

#[test]
fn sama_siemen_antaa_saman_kentan_ja_luolat_tulevat_myohemmin() {
    let a = luo_kentta(Vaikeus::Vaikea, 2, &mut Arpa::new(9));
    let b = luo_kentta(Vaikeus::Vaikea, 2, &mut Arpa::new(9));
    assert_eq!(a, b);
    let maan_alla = |kentta: &[[Solu; SARAKKEITA]; RIVEJA]| {
        kentta[MAA_ALKAA as usize..]
            .iter()
            .flatten()
            .filter(|&&s| s == Solu::Tyhja)
            .count()
    };
    assert_eq!(
        maan_alla(&luo_kentta(Vaikeus::Normaali, 0, &mut Arpa::new(3))),
        0
    );
    assert!(maan_alla(&a) > 0);
}

#[test]
fn neljan_levyinen_tunneli_kestaa_mutta_viiden_sortuu_keskelta() {
    let mut k = kaivos();
    tunneli(&mut k, 6, 5..9);
    let tapahtumat = aja(&mut k, 4000);
    assert!(tapahtumat.is_empty(), "{tapahtumat:?}");
    assert!((5..9).all(|x| k.solu((x, 5)) == Solu::Maa));

    let mut k = kaivos();
    tunneli(&mut k, 6, 5..10);
    let tapahtumat = aja(&mut k, 300);
    assert_eq!(tapahtumat, [Tapahtuma::Ritisee((7, 5))]);
    assert!(k.ritina[5][7] > 0);
    let tapahtumat = aja(&mut k, 4000);
    assert!(tapahtumat.contains(&Tapahtuma::Sortui {
        solu: Solu::Maa,
        paikka: (7, 6)
    }));
    // Keskimmäinen putosi tunnelin pohjalle, ja sen yläpuolinen holvi jäi kantamaan.
    assert_eq!(k.solu((7, 5)), Solu::Tyhja);
    assert_eq!(k.solu((7, 6)), Solu::Maa);
    assert_eq!(k.solu((7, 4)), Solu::Maa);
    assert!(k.putoavat.is_empty());
    assert!(aja(&mut k, 3000).is_empty());
}

#[test]
fn savi_kantaa_leveamman_katon() {
    let mut k = kaivos();
    for x in 0..SARAKKEITA as i32 {
        k.aseta((x, 5), Solu::Savi);
    }
    tunneli(&mut k, 6, 5..11);
    assert!(aja(&mut k, 4000).is_empty());

    tunneli(&mut k, 6, 11..12);
    let tapahtumat = aja(&mut k, 5000);
    assert!(tapahtumat.contains(&Tapahtuma::Ritisee((8, 5))));
    assert_eq!(k.solu((8, 6)), Solu::Savi);
}

#[test]
fn tuki_pelastaa_ritisevan_katon() {
    let mut k = kaivos();
    tunneli(&mut k, 6, 5..10);
    aja(&mut k, 500);
    assert!(k.ritina[5][7] > 0);
    k.aseta((7, 6), Solu::Tuki);
    aja(&mut k, ASKEL_MS);
    assert_eq!(k.ritina[5][7], 0);
    assert!(aja(&mut k, 4000).is_empty());
    assert_eq!(k.solu((7, 5)), Solu::Maa);
}

#[test]
fn hiekka_valuu_heti_alta_kaivettuna_ja_pino_putoaa_yhdessa() {
    let mut k = kaivos();
    k.aseta((10, 4), Solu::Hiekka);
    k.aseta((10, 5), Solu::Hiekka);
    k.aseta((10, 6), Solu::Tyhja);
    k.aseta((10, 7), Solu::Tyhja);
    let tapahtumat = aja(&mut k, 800);
    assert_eq!(k.solu((10, 7)), Solu::Hiekka);
    assert_eq!(k.solu((10, 6)), Solu::Hiekka);
    assert_eq!(k.solu((10, 5)), Solu::Tyhja);
    assert_eq!(k.solu((10, 4)), Solu::Tyhja);
    let sortumia = tapahtumat
        .iter()
        .filter(|t| matches!(t, Tapahtuma::Sortui { .. }))
        .count();
    assert_eq!(sortumia, 2);
}

#[test]
fn putoava_kivi_murskaa_kaivajan_ja_sakki_katoaa() {
    let mut k = kaivos();
    tunneli(&mut k, 6, 8..12);
    k.aseta((10, 5), Solu::Kivi);
    k.kaivaja.paikka = (10, 6);
    k.saalis = 60;
    let tapahtumat = aja(&mut k, 1500);
    assert!(tapahtumat.contains(&Tapahtuma::Murskautui {
        paikka: (10, 6),
        menetetty: 60
    }));
    assert_eq!(k.kyparat, KYPARIA - 1);
    assert_eq!(k.saalis, 0);
    assert_eq!(k.kaivaja.paikka, ALKU);
    assert_eq!(k.solu((10, 6)), Solu::Kivi);
}

#[test]
fn tuki_kantaa_kiven_ja_suojaa_kaivajaa() {
    let mut k = kaivos();
    tunneli(&mut k, 6, 8..12);
    k.aseta((10, 5), Solu::Kivi);
    k.aseta((10, 6), Solu::Tuki);
    k.kaivaja.paikka = (10, 6);
    assert!(aja(&mut k, 3000).is_empty());
    assert_eq!(k.kyparat, KYPARIA);
    assert_eq!(k.solu((10, 5)), Solu::Kivi);
}

#[test]
fn kaivaja_kaivaa_alas_ja_sivulle_mutta_ei_kalliota_eika_kivea() {
    let mut k = kaivos();
    let tapahtumat = paina(&mut k, (0, 1));
    assert!(tapahtumat.contains(&Tapahtuma::Kaivoi(Solu::Maa)));
    assert_eq!(k.kaivaja.paikka, (ALKU.0, MAA_ALKAA));
    paina(&mut k, (0, 1));
    assert_eq!(k.kaivaja.paikka, (ALKU.0, MAA_ALKAA + 1));
    paina(&mut k, (1, 0));
    assert_eq!(k.kaivaja.paikka, (ALKU.0 + 1, MAA_ALKAA + 1));
    assert_eq!(k.kaivaja.katse, 1);

    k.aseta((ALKU.0 + 2, MAA_ALKAA + 1), Solu::Kivi);
    k.aseta((ALKU.0 + 1, MAA_ALKAA + 2), Solu::Kallio);
    assert!(paina(&mut k, (1, 0)).is_empty());
    assert!(paina(&mut k, (0, 1)).is_empty());
    assert_eq!(k.kaivaja.paikka, (ALKU.0 + 1, MAA_ALKAA + 1));
    paina(&mut k, (-1, 0));
    assert_eq!(k.kaivaja.katse, -1);
}

#[test]
fn ylos_paasee_vain_tukea_pitkin() {
    let mut k = kaivos();
    paina(&mut k, (0, 1));
    assert_eq!(k.kaivaja.paikka.1, MAA_ALKAA);
    paina(&mut k, (0, -1));
    assert_eq!(k.kaivaja.paikka.1, MAA_ALKAA);

    assert!(k.vaihda_tuki());
    assert_eq!(k.tuet, Vaikeus::Normaali.tukia() - 1);
    // Tuessa seisova ei kaiva jalkojensa alta.
    assert!(paina(&mut k, (0, 1)).is_empty());
    paina(&mut k, (0, -1));
    assert_eq!(k.kaivaja.paikka, ALKU);
    // Pinnalle ei pysty tukea.
    assert!(!k.vaihda_tuki());

    // Alas tukea pitkin ja tuki takaisin.
    paina(&mut k, (0, 1));
    assert_eq!(k.kaivaja.paikka.1, MAA_ALKAA);
    assert!(k.vaihda_tuki());
    assert_eq!(k.solu(k.kaivaja.paikka), Solu::Tyhja);
    assert_eq!(k.tuet, Vaikeus::Normaali.tukia());
}

#[test]
fn ylos_kaivaminen_kaivaa_mutta_ei_nosta() {
    let mut k = kaivos();
    tunneli(&mut k, 6, 8..10);
    k.kaivaja.paikka = (8, 6);
    let tapahtumat = paina(&mut k, (0, -1));
    assert!(tapahtumat.contains(&Tapahtuma::Kaivoi(Solu::Maa)));
    assert_eq!(k.solu((8, 5)), Solu::Tyhja);
    assert_eq!(k.kaivaja.paikka, (8, 6));
}

#[test]
fn pitka_pudotus_satuttaa_mutta_lyhyt_ei() {
    let mut k = kaivos();
    tunneli(&mut k, 2, 10..11);
    for y in MAA_ALKAA..MAA_ALKAA + 3 {
        k.aseta((10, y), Solu::Tyhja);
    }
    k.kaivaja.paikka = (10, PINTA);
    assert!(aja(&mut k, 1000).is_empty());
    assert_eq!(k.kaivaja.paikka, (10, MAA_ALKAA + 2));

    let mut k = kaivos();
    for y in MAA_ALKAA..MAA_ALKAA + 9 {
        k.aseta((10, y), Solu::Tyhja);
    }
    k.kaivaja.paikka = (10, PINTA);
    k.saalis = 30;
    let tapahtumat = aja(&mut k, 1500);
    assert!(tapahtumat.contains(&Tapahtuma::Putosi {
        paikka: (10, MAA_ALKAA + 8),
        menetetty: 30
    }));
    assert_eq!(k.kyparat, KYPARIA - 1);
    assert_eq!(k.kaivaja.paikka, ALKU);
}

#[test]
fn kulta_menee_sakkiin_ja_hissilla_pisteiksi() {
    let mut k = kaivos();
    k.aseta((ALKU.0, MAA_ALKAA), Solu::Kulta);
    let tapahtumat = paina(&mut k, (0, 1));
    let arvo = kullan_arvo(MAA_ALKAA) * 2;
    assert!(tapahtumat.contains(&Tapahtuma::Loysi {
        loyto: Loyto::Kulta(arvo),
        paikka: (ALKU.0, MAA_ALKAA)
    }));
    assert_eq!(k.saalis, arvo);
    assert_eq!(k.pisteet, 0);
    k.vaihda_tuki();
    paina(&mut k, (0, -1));
    paina(&mut k, (-1, 0));
    let tapahtumat = paina(&mut k, (-1, 0));
    assert_eq!(k.kaivaja.paikka, HISSI);
    assert!(tapahtumat.contains(&Tapahtuma::Nosti { pisteet: arvo }));
    assert_eq!(k.pisteet, arvo);
    assert_eq!(k.saalis, 0);
    assert_eq!(k.nostettu, arvo);
    assert_eq!(k.hissi_ms, None);
}

#[test]
fn taysi_kiintio_vie_hissilla_seuraavalle_tasolle() {
    let mut k = kaivos();
    k.kiintio = 20;
    k.aseta((ALKU.0, MAA_ALKAA), Solu::Kulta);
    let tapahtumat = paina(&mut k, (0, 1));
    assert!(tapahtumat.contains(&Tapahtuma::KiintioTaynna));
    k.vaihda_tuki();
    paina(&mut k, (0, -1));
    paina(&mut k, (-1, 0));
    let tapahtumat = paina(&mut k, (-1, 0));
    let tuet = Vaikeus::Normaali.tukia() - 1;
    let bonus = 2 * (50 + 10 * tuet);
    assert!(tapahtumat.contains(&Tapahtuma::TasoValmis { bonus }));
    assert_eq!(k.pisteet, 20 + bonus);
    assert!(k.hissi_ms.is_some());
    // Hissin matkalla kaivaja ei liiku.
    k.paina((1, 0));
    let tapahtumat = aja(&mut k, HISSIN_MATKA_MS);
    assert_eq!(tapahtumat, [Tapahtuma::UusiTaso]);
    assert_eq!(k.taso, 1);
    assert_eq!(k.kaivaja.paikka, ALKU);
    assert_eq!(k.tuet, Vaikeus::Normaali.tukia());
    assert_eq!(k.pisteet, 20 + bonus);
    assert_eq!(k.nostettu, 0);
    assert_eq!(k.kaivaja.toimi, Toimi::Seisoo);
}

#[test]
fn loydot_tayttavat_lamppua_ja_tukipinoa() {
    let mut k = kaivos();
    k.oljy_ms = 1000;
    k.aseta((ALKU.0, MAA_ALKAA), Solu::Oljy);
    k.aseta((ALKU.0, MAA_ALKAA + 1), Solu::Puut);
    paina(&mut k, (0, 1));
    assert!(k.oljy_ms > 1000);
    let tapahtumat = paina(&mut k, (0, 1));
    assert!(tapahtumat.contains(&Tapahtuma::Loysi {
        loyto: Loyto::Puut(4),
        paikka: (ALKU.0, MAA_ALKAA + 1)
    }));
    assert_eq!(k.tuet, Vaikeus::Normaali.tukia() + 4);
}

#[test]
fn lamppu_palaa_maan_alla_ja_tayttyy_pinnalla() {
    let mut k = kaivos();
    let taysi = Vaikeus::Normaali.oljya_ms();
    let suurin = k.valon_sade();
    tunneli(&mut k, 5, 10..11);
    k.kaivaja.paikka = (10, 5);
    aja(&mut k, 1000);
    assert_eq!(k.oljy_ms, taysi - 1000);
    assert!(k.valon_sade() < suurin);

    k.oljy_ms = 10;
    let tapahtumat = aja(&mut k, 100);
    assert_eq!(tapahtumat, [Tapahtuma::OljyLoppui]);
    assert_eq!(k.valon_sade(), PIENIN_VALO);
    assert!(aja(&mut k, 1000).is_empty());

    k.kaivaja.paikka = ALKU;
    aja(&mut k, taysi / 4);
    assert_eq!(k.oljy_ms, taysi);
}

#[test]
fn hatakoysi_maksaa_kyparan_ja_viimeinen_paattaa_pelin() {
    let mut k = kaivos();
    let mut tapahtumat = Vec::new();
    assert!(!k.heita_koysi(&mut tapahtumat));
    tunneli(&mut k, 5, 10..11);
    k.kaivaja.paikka = (10, 5);
    k.saalis = 40;
    assert!(k.heita_koysi(&mut tapahtumat));
    assert_eq!(tapahtumat, [Tapahtuma::Koysi { menetetty: 40 }]);
    assert_eq!(k.kaivaja.paikka, ALKU);
    assert_eq!(k.kyparat, KYPARIA - 1);

    k.kyparat = 1;
    k.kaivaja.paikka = (10, 5);
    tapahtumat.clear();
    k.heita_koysi(&mut tapahtumat);
    assert_eq!(
        tapahtumat,
        [Tapahtuma::Koysi { menetetty: 0 }, Tapahtuma::Ohi]
    );
    assert!(k.ohi);
    k.paina((1, 0));
    assert!(aja(&mut k, 500).is_empty());
    assert_eq!(k.kaivaja.paikka, ALKU);
}

#[test]
fn pisteet_eivat_ylita_rajaa() {
    let mut k = kaivos();
    k.pisteet = SUURIN_PISTEMAARA - 5;
    k.saalis = 500;
    k.kaivaja.paikka = HISSI;
    aja(&mut k, ASKEL_MS);
    assert_eq!(k.pisteet, SUURIN_PISTEMAARA);
}

#[test]
fn painallus_vie_yhden_ruudun_ja_viimeisin_odottaa_vuoroaan() {
    let mut k = kaivos();
    k.paina((1, 0));
    aja(&mut k, 1000);
    assert_eq!(k.kaivaja.paikka, (ALKU.0 + 1, PINTA));
    // Kaivu ja siirto kaivettuun ruutuun ovat yksi toimi.
    k.paina((0, 1));
    aja(&mut k, 1000);
    assert_eq!(k.kaivaja.paikka, (ALKU.0 + 1, MAA_ALKAA));
    k.paina((1, 0));
    aja(&mut k, 1000);
    assert_eq!(k.kaivaja.paikka, (ALKU.0 + 2, MAA_ALKAA));

    let mut k = kaivos();
    k.paina((1, 0));
    k.paina((-1, 0));
    aja(&mut k, 300);
    assert_eq!(k.kaivaja.paikka, (ALKU.0 - 1, PINTA));
}

#[test]
fn sama_siemen_ja_ohjaus_toistavat_saman_pelin() {
    let pelaa = || {
        let mut arpa = Arpa::new(5);
        let mut k = Kaivos::new(Vaikeus::Vaikea, 2, &mut arpa);
        let mut tapahtumat = Vec::new();
        for i in 0..4000u32 {
            if i % 60 == 0 {
                k.paina([(1, 0), (0, 1), (-1, 0), (0, 1)][(i / 300 % 4) as usize]);
            }
            tapahtumat.extend(k.askel(&mut arpa));
        }
        (tapahtumat, k.solut, k.kaivaja, k.pisteet)
    };
    assert_eq!(pelaa(), pelaa());
}
