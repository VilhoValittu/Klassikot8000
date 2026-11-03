use klassikot8000::Arpa;
use klassikot8000::laivanupotus::{
    KOKO, LAIVARUUTUJA, LAIVAT, Laivasto, Laukaustulos, Ruudut, Ruutu, SUURIN_TULOS, Vaikeus,
    pisteet, vastustaja,
};

fn kaikki_ruudut() -> impl Iterator<Item = Ruutu> {
    (0..KOKO * KOKO).filter_map(|i| Ruutu::new(i / KOKO, i % KOKO))
}

#[test]
fn laivasto_on_saantojen_mukainen_kaikilla_siemenilla() {
    for siemen in 0..500 {
        let laivasto = Laivasto::satunnainen(&mut Arpa::new(siemen));
        let mut kaikki = Ruudut::default();
        for (laiva, (nimi, pituus)) in laivasto.laivat().iter().zip(LAIVAT) {
            assert_eq!(laiva.nimi, nimi);
            assert_eq!(laiva.ruudut.maara(), pituus);
            assert!(!laiva.ruudut.leikkaako(kaikki));
            kaikki = kaikki.yhdiste(laiva.ruudut);

            // suora ja yhtenäinen: toinen akseli on vakio, toisen vaihteluväli on pituus
            let ruudut: Vec<_> = kaikki_ruudut()
                .filter(|&r| laiva.ruudut.sisaltaa(r))
                .collect();
            let vali = |a: Vec<u8>| (a.iter().max().unwrap() - a.iter().min().unwrap()) as usize;
            let dr = vali(ruudut.iter().map(|r| r.rivi).collect());
            let ds = vali(ruudut.iter().map(|r| r.sarake).collect());
            assert!((dr, ds) == (0, pituus - 1) || (dr, ds) == (pituus - 1, 0));
        }
    }
}

#[test]
fn laivat_eivat_koske_toisiaan_edes_kulmittain() {
    for siemen in 0..5000 {
        let laivasto = Laivasto::satunnainen(&mut Arpa::new(siemen));
        let ruudut = |i: usize| {
            let laiva = laivasto.laivat()[i];
            kaikki_ruudut().filter(move |&r| laiva.ruudut.sisaltaa(r))
        };
        for a in 0..LAIVAT.len() {
            for b in a + 1..LAIVAT.len() {
                for (ra, rb) in ruudut(a).flat_map(|ra| ruudut(b).map(move |rb| (ra, rb))) {
                    let etaisyys = ra.rivi.abs_diff(rb.rivi).max(ra.sarake.abs_diff(rb.sarake));
                    assert!(etaisyys > 1, "siemen {siemen}: {ra} ja {rb}");
                }
            }
        }
    }
}

#[test]
fn molemmat_suunnat_esiintyvat() {
    let mut suunnat = [false; 2];
    for siemen in 0..50 {
        let laivasto = Laivasto::satunnainen(&mut Arpa::new(siemen));
        let laiva = laivasto.laivat()[0];
        let alku: Vec<_> = kaikki_ruudut()
            .filter(|&r| laiva.ruudut.sisaltaa(r))
            .take(2)
            .collect();
        suunnat[(alku[0].rivi == alku[1].rivi) as usize] = true;
    }
    assert_eq!(suunnat, [true, true]);
}

#[test]
fn laiva_uppoaa_vasta_viimeisesta_ruudusta_ja_voitto_17_osumasta() {
    for siemen in 0..100 {
        let mut laivasto = Laivasto::satunnainen(&mut Arpa::new(siemen));
        let mut osumia = 0;
        let mut upotettuja = 0;
        for ruutu in kaikki_ruudut() {
            let oli_laiva = laivasto.on_laiva(ruutu);
            match laivasto.kasittele_laukaus(ruutu).unwrap() {
                Laukaustulos::Ohi => assert!(!oli_laiva),
                Laukaustulos::Osuma => osumia += 1,
                Laukaustulos::Upposi(nimi) => {
                    osumia += 1;
                    upotettuja += 1;
                    let laiva = laivasto.laivat().iter().find(|l| l.nimi == nimi).unwrap();
                    assert!(laiva.ruudut.sisaltaa(ruutu));
                    assert!(laiva.ruudut.on_osajoukko(laivasto.ammutut()));
                }
            }
            assert_eq!(laivasto.kaikki_upotettu(), osumia == 17);
        }
        assert_eq!((osumia, upotettuja), (17, 5));
    }
}

#[test]
fn upotettujen_laskuri_seuraa_uponneita_laivoja() {
    for siemen in 0..100 {
        let mut laivasto = Laivasto::satunnainen(&mut Arpa::new(siemen));
        let mut uponneet = Vec::new();
        for ruutu in kaikki_ruudut() {
            if let Some(Laukaustulos::Upposi(nimi)) = laivasto.kasittele_laukaus(ruutu) {
                uponneet.push(nimi);
            }
            assert_eq!(laivasto.upotettuja(), uponneet.len());
            for laiva in laivasto.laivat() {
                assert_eq!(laivasto.on_upotettu(laiva), uponneet.contains(&laiva.nimi));
            }
        }
    }
}

#[test]
fn sama_ruutu_hylataan_toisella_kerralla() {
    let mut laivasto = Laivasto::satunnainen(&mut Arpa::new(1));
    let ruutu = Ruutu::new(4, 4).unwrap();
    assert!(laivasto.kasittele_laukaus(ruutu).is_some());
    assert_eq!(laivasto.kasittele_laukaus(ruutu), None);
    assert_eq!(laivasto.ammutut().maara(), 1);
}

#[test]
fn vastustaja_ei_ammu_samaan_ja_kattaa_koko_laudan() {
    for vaikeus in Vaikeus::KAIKKI {
        for siemen in 0..50 {
            let mut arpa = Arpa::new(siemen);
            let mut laivasto = Laivasto::satunnainen(&mut arpa);
            for _ in 0..KOKO * KOKO {
                let ruutu = vastustaja::valitse(vaikeus, &laivasto.tiedot(), &mut arpa).unwrap();
                assert!(laivasto.kasittele_laukaus(ruutu).is_some(), "{vaikeus:?}");
            }
            assert_eq!(
                vastustaja::valitse(vaikeus, &laivasto.tiedot(), &mut arpa),
                None
            );
        }
    }
}

/// Montako laukausta vastustaja tarvitsee koko laivaston upottamiseen, keskimäärin.
fn keskimaarin_laukauksia(vaikeus: Vaikeus) -> f32 {
    const PELEJA: u64 = 100;
    let mut laukauksia = 0;
    for siemen in 0..PELEJA {
        let mut arpa = Arpa::new(siemen);
        let mut laivasto = Laivasto::satunnainen(&mut arpa);
        while !laivasto.kaikki_upotettu() {
            let ruutu = vastustaja::valitse(vaikeus, &laivasto.tiedot(), &mut arpa).unwrap();
            laivasto.kasittele_laukaus(ruutu);
            laukauksia += 1;
        }
    }
    laukauksia as f32 / PELEJA as f32
}

#[test]
fn vaikeampi_taso_upottaa_laivaston_nopeammin() {
    let helppo = keskimaarin_laukauksia(Vaikeus::Helppo);
    let normaali = keskimaarin_laukauksia(Vaikeus::Normaali);
    let vaikea = keskimaarin_laukauksia(Vaikeus::Vaikea);
    println!("helppo {helppo}, normaali {normaali}, vaikea {vaikea}");
    assert!(helppo > 85.0, "helppo {helppo}");
    assert!(normaali < helppo - 15.0, "normaali {normaali}");
    assert!(vaikea < normaali - 10.0, "vaikea {vaikea}");
}

#[test]
fn normaali_ja_vaikea_ampuvat_osuman_viereen() {
    for vaikeus in [Vaikeus::Normaali, Vaikeus::Vaikea] {
        for siemen in 0..200 {
            let mut arpa = Arpa::new(siemen);
            let mut laivasto = Laivasto::satunnainen(&mut arpa);
            let laiva = laivasto.laivat()[0];
            let osuma = kaikki_ruudut().find(|&r| laiva.ruudut.sisaltaa(r)).unwrap();
            laivasto.kasittele_laukaus(osuma);
            let ruutu = vastustaja::valitse(vaikeus, &laivasto.tiedot(), &mut arpa).unwrap();
            let etaisyys = ruutu.rivi.abs_diff(osuma.rivi) + ruutu.sarake.abs_diff(osuma.sarake);
            assert_eq!(etaisyys, 1, "{vaikeus:?}, siemen {siemen}");
        }
    }
}

#[test]
fn vaikea_ei_ammu_uponneen_laivan_viereen() {
    for siemen in 0..100 {
        let mut arpa = Arpa::new(siemen);
        let mut laivasto = Laivasto::satunnainen(&mut arpa);
        while !laivasto.kaikki_upotettu() {
            let tiedot = laivasto.tiedot();
            let ruutu = vastustaja::valitse(Vaikeus::Vaikea, &tiedot, &mut arpa).unwrap();
            assert!(
                !tiedot.upotetut.ymparys().sisaltaa(ruutu),
                "siemen {siemen}"
            );
            laivasto.kasittele_laukaus(ruutu);
        }
    }
}

#[test]
fn tiedot_eivat_paljasta_kelluvia_laivoja() {
    let mut laivasto = Laivasto::satunnainen(&mut Arpa::new(1));
    assert_eq!(laivasto.tiedot().osumat.maara(), 0);
    assert_eq!(laivasto.tiedot().jaljella, [5, 4, 3, 3, 2]);

    let partioalus = laivasto.laivat()[4];
    let ruudut: Vec<_> = kaikki_ruudut()
        .filter(|&r| partioalus.ruudut.sisaltaa(r))
        .collect();
    laivasto.kasittele_laukaus(ruudut[0]);
    assert_eq!(laivasto.tiedot().osumat.maara(), 1);
    laivasto.kasittele_laukaus(ruudut[1]);
    let tiedot = laivasto.tiedot();
    assert_eq!(tiedot.osumat.maara(), 0);
    assert_eq!(tiedot.upotetut, partioalus.ruudut);
    assert_eq!(tiedot.jaljella, [5, 4, 3, 3]);
}

#[test]
fn siirto_kelpaa_vain_koskematta_muihin_ja_ennen_laukauksia() {
    for siemen in 0..300 {
        let mut arpa = Arpa::new(siemen);
        let mut laivasto = Laivasto::satunnainen(&mut arpa);
        // Satunnaisia siirtoyrityksiä: hyväksytyn jälkeen sääntö pätee yhä.
        for _ in 0..40 {
            let i = arpa.alle(LAIVAT.len());
            let pituus = LAIVAT[i].1;
            let alku = Ruutu::new(arpa.alle(KOKO), arpa.alle(KOKO)).unwrap();
            let Some(ruudut) = Ruudut::suora(alku, pituus, arpa.alle(2) == 0) else {
                continue;
            };
            let kelpaa = laivasto.mahtuu(i, ruudut);
            assert_eq!(laivasto.siirra(i, ruudut), kelpaa);
            if kelpaa {
                assert_eq!(laivasto.laivat()[i].ruudut, ruudut);
                assert_eq!(laivasto.laivat()[i].alku(), Some(alku));
            }
            for (j, laiva) in laivasto.laivat().iter().enumerate() {
                for (k, muu) in laivasto.laivat().iter().enumerate() {
                    assert!(j == k || !laiva.ruudut.ymparys().leikkaako(muu.ruudut));
                }
            }
        }
    }

    // väärän pituinen jono ja ammuttu laivasto eivät kelpaa
    let mut laivasto = Laivasto::satunnainen(&mut Arpa::new(1));
    let alku = laivasto.laivat()[4].alku().unwrap();
    let vaaka = laivasto.laivat()[4].vaaka();
    assert!(!laivasto.siirra(4, Ruudut::suora(alku, 3, vaaka).unwrap_or_default()));
    let paikallaan = laivasto.laivat()[4].ruudut;
    assert!(laivasto.siirra(4, paikallaan));
    assert_eq!(laivasto.laiva_ruudussa(alku), Some(4));
    laivasto.kasittele_laukaus(Ruutu::new(0, 0).unwrap());
    assert!(!laivasto.siirra(4, paikallaan));
}

#[test]
fn voiton_pisteet_kasvavat_saastetyista_laukauksista_ja_tasosta() {
    let ruutuja: usize = LAIVAT.iter().map(|&(_, pituus)| pituus).sum();
    assert_eq!(ruutuja as u32, LAIVARUUTUJA);
    assert_eq!(pisteet(Vaikeus::Helppo, 100), 1);
    assert_eq!(pisteet(Vaikeus::Helppo, 60), 41);
    assert_eq!(pisteet(Vaikeus::Normaali, 60), 82);
    assert_eq!(pisteet(Vaikeus::Vaikea, 60), 123);
    assert_eq!(SUURIN_TULOS, 252);
    // mahdoton laukausmäärä ei voi ylivuotaa
    assert_eq!(pisteet(Vaikeus::Vaikea, 1000), 3);
    for vaikeus in Vaikeus::KAIKKI {
        for laukauksia in LAIVARUUTUJA..=KOKO as u32 * KOKO as u32 {
            let nyt = pisteet(vaikeus, laukauksia);
            assert!(nyt > 0 && nyt <= SUURIN_TULOS);
            assert!(nyt > pisteet(vaikeus, laukauksia + 1) || laukauksia == 100);
        }
    }
}
