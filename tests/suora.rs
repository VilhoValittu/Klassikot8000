use klassikot8000::Arpa;
use klassikot8000::suora::{
    KOKO, Lauta, Merkki, Muoto, Paikka, RUUTUJA, SIIRTOJA_ENINTAAN, SUUNNAT, SUURIN_TULOS, Tila,
    Vaikeus, koneen_siirto, muodot, pisteet,
};

/// Viisi ruutua suunnassa `d` alkaen kohdasta `alku`.
fn rivi(alku: Paikka, (dr, ds): (i32, i32), pituus: usize) -> Vec<Paikka> {
    (0..pituus as i32)
        .map(|i| {
            (
                (alku.0 as i32 + dr * i) as usize,
                (alku.1 as i32 + ds * i) as usize,
            )
        })
        .collect()
}

#[test]
fn viisi_suoraan_voittaa_joka_suunnassa_ja_keskelta_taydennettyna() {
    for d in SUUNNAT {
        let alku = if d.1 < 0 { (3, 10) } else { (3, 3) };
        let ruudut = rivi(alku, d, 5);
        // Viimeinen merkki täydentää suoran keskeltä, jolloin molemmat puolet lasketaan.
        for viimeinen in 0..5 {
            let ristit: Vec<Paikka> = (0..5)
                .filter(|&i| i != viimeinen)
                .map(|i| ruudut[i])
                .collect();
            let nollat = [(12, 0), (12, 2), (12, 4), (12, 6)];
            let mut lauta = Lauta::merkeista(&ristit, &nollat);
            assert_eq!(lauta.vuoro, Merkki::Risti);
            assert_eq!(
                lauta.aseta(ruudut[viimeinen]),
                Some(Tila::Voitto(Merkki::Risti))
            );
            assert_eq!(lauta.voittorivi, ruudut, "{d:?}");
            // Ohi olevaan peliin ei voi enää merkitä.
            assert_eq!(lauta.aseta((0, 0)), None);
        }
    }
}

#[test]
fn nelja_ei_voita_mutta_kuusi_voittaa() {
    let mut lauta = Lauta::merkeista(&[(5, 1), (5, 2), (5, 3)], &[(0, 0), (0, 1), (0, 2)]);
    assert_eq!(lauta.aseta((5, 4)), Some(Tila::Kesken));
    assert_eq!(lauta.vuoro, Merkki::Nolla);
    // Kuuden suora: aukko keskeltä täyttyy.
    let mut lauta = Lauta::merkeista(
        &[(5, 0), (5, 1), (5, 3), (5, 4), (5, 5)],
        &[(0, 0), (0, 1), (0, 2), (0, 3), (0, 4)],
    );
    assert_eq!(lauta.vuoro, Merkki::Risti);
    assert_eq!(lauta.aseta((5, 2)), Some(Tila::Voitto(Merkki::Risti)));
    assert_eq!(lauta.voittorivi.len(), 6);
}

#[test]
fn varattuun_tai_laudan_ulkopuolelle_ei_voi_merkita() {
    let mut lauta = Lauta::new();
    assert_eq!(lauta.aseta((7, 7)), Some(Tila::Kesken));
    assert_eq!(lauta.aseta((7, 7)), None);
    assert_eq!(lauta.aseta((KOKO, 0)), None);
    assert_eq!(lauta.aseta((0, KOKO)), None);
    assert_eq!(lauta.vuoro, Merkki::Nolla);
    assert_eq!(lauta.siirrot, vec![(7, 7)]);
    assert_eq!(lauta.viimeisin(), Some((7, 7)));
}

#[test]
fn taysi_lauta_ilman_suoraa_on_tasapeli() {
    // Kahden ruudun raidat, joiden vaihe siirtyy joka rivillä: pisin suora joka suuntaan on kaksi.
    let ristin = |r: usize, s: usize| (s + 2 * r) % 4 < 2;
    let viimeinen = (KOKO - 1, KOKO - 1);
    let (mut ristit, mut nollat) = (Vec::new(), Vec::new());
    for r in 0..KOKO {
        for s in 0..KOKO {
            if (r, s) != viimeinen {
                if ristin(r, s) {
                    &mut ristit
                } else {
                    &mut nollat
                }
                .push((r, s));
            }
        }
    }
    assert_eq!(ristit.len() as u32, SIIRTOJA_ENINTAAN);
    let mut lauta = Lauta::merkeista(&ristit, &nollat);
    assert_eq!(lauta.vuoro, Merkki::Nolla);
    assert_eq!(lauta.aseta(viimeinen), Some(Tila::Tasapeli));
    assert!(lauta.voittorivi.is_empty());
    assert_eq!(lauta.siirrot.len(), RUUTUJA);
}

#[test]
fn muodot_tunnistetaan() {
    // Avoin kolmonen: _XX?_ keskellä tyhjää.
    let lauta = Lauta::merkeista(&[(7, 5), (7, 6)], &[(0, 0), (0, 14)]);
    assert_eq!(muodot(&lauta, (7, 7), Merkki::Risti)[0], Muoto::AvoinKolme);
    // Toisesta päästä suljettu kolmonen.
    let lauta = Lauta::merkeista(&[(7, 5), (7, 6)], &[(7, 4), (0, 14)]);
    assert_eq!(muodot(&lauta, (7, 7), Merkki::Risti)[0], Muoto::Kolme);
    // Laudan reuna sulkee kuten vastustaja.
    let lauta = Lauta::merkeista(&[(7, 0), (7, 1), (7, 2)], &[(0, 0), (0, 14), (14, 0)]);
    assert_eq!(muodot(&lauta, (7, 3), Merkki::Risti)[0], Muoto::Nelja);
    let lauta = Lauta::merkeista(&[(7, 4), (7, 5), (7, 6)], &[(0, 0), (0, 14), (14, 0)]);
    assert_eq!(muodot(&lauta, (7, 7), Merkki::Risti)[0], Muoto::AvoinNelja);
    // Rakoinen nelonen X_XX? on yksi puuttuva ruutu.
    let lauta = Lauta::merkeista(&[(7, 3), (7, 5), (7, 6)], &[(0, 0), (0, 14), (14, 0)]);
    assert_eq!(muodot(&lauta, (7, 7), Merkki::Risti)[0], Muoto::Nelja);
    // Pysty ja vino samasta ruudusta, vaaka tyhjä.
    let lauta = Lauta::merkeista(
        &[(5, 7), (6, 7), (8, 8), (9, 9)],
        &[(0, 0), (0, 14), (14, 0), (14, 14)],
    );
    let m = muodot(&lauta, (7, 7), Merkki::Risti);
    assert_eq!(m[0], Muoto::Ei);
    assert_eq!(m[1], Muoto::AvoinKolme);
    assert_eq!(m[2], Muoto::AvoinKolme);
    // Avoin kakkonen.
    let lauta = Lauta::merkeista(&[(7, 6)], &[(0, 0)]);
    assert_eq!(muodot(&lauta, (7, 7), Merkki::Risti)[0], Muoto::AvoinKaksi);
    let lauta = Lauta::merkeista(
        &[(7, 3), (7, 4), (7, 5), (7, 6)],
        &[(0, 0), (0, 1), (0, 2), (0, 3)],
    );
    assert_eq!(muodot(&lauta, (7, 7), Merkki::Risti)[0], Muoto::Viisi);
}

#[test]
fn pisteet_suosivat_nopeaa_voittoa_ja_vaikeaa_tasoa() {
    assert_eq!(SIIRTOJA_ENINTAAN, 113);
    assert_eq!(SUURIN_TULOS, 327);
    assert!(pisteet(Vaikeus::Normaali, 10) > pisteet(Vaikeus::Normaali, 11));
    assert!(pisteet(Vaikeus::Vaikea, 30) > pisteet(Vaikeus::Normaali, 30));
    assert_eq!(pisteet(Vaikeus::Helppo, 500), 1);
    for taso in Vaikeus::KAIKKI {
        assert!(pisteet(taso, 5) <= SUURIN_TULOS);
    }
}

#[test]
fn kone_voittaa_kun_voi_ja_torjuu_nelosen() {
    for taso in Vaikeus::KAIKKI {
        for siemen in 0..20 {
            let mut arpa = Arpa::new(siemen);
            // Nollalla on neljä, ristillä kolme: oma voitto ennen torjuntaa.
            let lauta = Lauta::merkeista(
                &[(2, 2), (2, 3), (2, 4), (10, 10), (11, 11)],
                &[(6, 1), (6, 2), (6, 3), (6, 4)],
            );
            let p = koneen_siirto(taso, &lauta, &mut arpa).unwrap();
            assert!(p == (6, 0) || p == (6, 5), "{taso:?} {p:?}");
        }
    }
    for taso in [Vaikeus::Normaali, Vaikeus::Vaikea] {
        for siemen in 0..20 {
            let mut arpa = Arpa::new(siemen);
            let lauta = Lauta::merkeista(
                &[(4, 4), (4, 5), (4, 6), (4, 7)],
                &[(4, 3), (9, 9), (9, 10)],
            );
            assert_eq!(
                koneen_siirto(taso, &lauta, &mut arpa),
                Some((4, 8)),
                "{taso:?}"
            );
            // Avoin kolmonen torjutaan sen päästä.
            let lauta = Lauta::merkeista(&[(4, 5), (4, 6), (4, 7)], &[(9, 9), (10, 10)]);
            let p = koneen_siirto(taso, &lauta, &mut arpa).unwrap();
            assert!(
                [(4, 4), (4, 8), (4, 3), (4, 9)].contains(&p),
                "{taso:?} {p:?}"
            );
        }
    }
}

#[test]
fn vaikea_kone_loytaa_nelossarjan() {
    // Nolla voittaa nelosilla: (8,4) pakottaa torjunnan ja tekee sitten kaksi nelosta.
    // Pystyrivi 4: (5,4) (6,4) (7,4) ja vaakarivi 8: (8,5) (8,6) (8,7), kulma (8,4) yhdistää.
    let ristit = [(5, 3), (8, 8), (4, 4), (0, 0), (0, 2), (0, 4), (0, 6)];
    let nollat = [(5, 4), (6, 4), (7, 4), (8, 5), (8, 6), (8, 7)];
    let mut lauta = Lauta::merkeista(&ristit, &nollat);
    lauta.vuoro = Merkki::Nolla;
    let p = koneen_siirto(Vaikeus::Vaikea, &lauta, &mut Arpa::new(1)).unwrap();
    assert_eq!(p, (8, 4));
}

/// Pelaa koneet toisiaan vastaan. Palauttaa voittajan, jos peli ratkesi.
fn ottelu(risti: Vaikeus, nolla: Vaikeus, siemen: u64) -> Tila {
    let mut arpa = Arpa::new(siemen);
    let mut lauta = Lauta::new();
    while lauta.tila == Tila::Kesken {
        let taso = if lauta.vuoro == Merkki::Risti {
            risti
        } else {
            nolla
        };
        let p = koneen_siirto(taso, &lauta, &mut arpa).expect("siirto puuttuu");
        assert!(lauta.ruutu(p).is_none(), "varattu {p:?}");
        assert!(lauta.aseta(p).is_some());
    }
    assert_eq!(koneen_siirto(risti, &lauta, &mut arpa), None);
    lauta.tila
}

#[test]
fn koneen_siirto_on_aina_laillinen_ja_peli_ratkeaa() {
    for siemen in 0..6 {
        for taso in Vaikeus::KAIKKI {
            let tila = ottelu(taso, taso, siemen);
            assert_ne!(tila, Tila::Kesken);
        }
    }
}

#[test]
fn vaikeampi_kone_voittaa_helpomman_useimmiten() {
    // Voitot ja tappiot vaikeamman kannalta; koneiden välillä tasapeli on yleinen.
    let mut tulokset = [[0; 2]; 2];
    for siemen in 0..10 {
        // Kumpikin vuorollaan aloittaa, ettei aloitusetu ratkaise.
        for (i, (vahva, heikko)) in [
            (Vaikeus::Normaali, Vaikeus::Helppo),
            (Vaikeus::Vaikea, Vaikeus::Normaali),
        ]
        .into_iter()
        .enumerate()
        {
            for (risti, nolla, vahvan) in [
                (vahva, heikko, Merkki::Risti),
                (heikko, vahva, Merkki::Nolla),
            ] {
                match ottelu(risti, nolla, siemen) {
                    Tila::Voitto(m) if m == vahvan => tulokset[i][0] += 1,
                    Tila::Voitto(_) => tulokset[i][1] += 1,
                    _ => {}
                }
            }
        }
    }
    let [helppoa, normaalia] = tulokset;
    assert!(helppoa[0] >= 15, "normaali helppoa vastaan {helppoa:?}");
    assert!(
        normaalia[0] > normaalia[1],
        "vaikea normaalia vastaan {normaalia:?}"
    );
}
