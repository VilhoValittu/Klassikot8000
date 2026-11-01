use klassikot8000::Arpa;
use klassikot8000::kortit::{KUNINGAS, Kortti, Maa};
use klassikot8000::pasianssi::Vaikeus;
use klassikot8000::pyramidi::{Lauta, PYRAMIDISSA, Paikka, RIVEJA, kierroksia};

fn k(arvo: u8) -> Kortti {
    Kortti {
        maa: Maa::Pata,
        arvo,
    }
}

fn pyr(rivi: usize, sarake: usize) -> Paikka {
    Paikka::Pyramidi { rivi, sarake }
}

fn tyhja(vaikeus: Vaikeus) -> Lauta {
    let mut lauta = Lauta::new(vaikeus, &mut Arpa::new(1));
    lauta.tila.rivit = [[None; RIVEJA]; RIVEJA];
    lauta.tila.pakka.clear();
    lauta
}

fn kaikki_kortit(lauta: &Lauta) -> Vec<(u8, u8)> {
    let tila = &lauta.tila;
    let mut kortit: Vec<(u8, u8)> = tila
        .rivit
        .iter()
        .flatten()
        .flatten()
        .chain(&tila.pakka)
        .chain(&tila.jate)
        .map(|k| (k.maa as u8, k.arvo))
        .collect();
    kortit.sort_unstable();
    kortit
}

/// Ahne pelaaja: poistaa minkä voi ja nostaa muuten. Palauttaa, tyhjenikö pyramidi.
fn pelaa_ahneesti(lauta: &mut Lauta) -> bool {
    for _ in 0..5000 {
        if lauta.on_voitettu() {
            return true;
        }
        if !lauta.on_siirtoja() {
            return false;
        }
        let mut paikat = vec![Paikka::Jate(0), Paikka::Jate(1)];
        for rivi in (0..RIVEJA).rev() {
            for sarake in 0..=rivi {
                paikat.push(pyr(rivi, sarake));
            }
        }
        let kuningas = paikat.iter().any(|&p| lauta.poista_kuningas(p));
        let pari = !kuningas
            && paikat
                .iter()
                .any(|&a| paikat.iter().any(|&b| lauta.poista_pari(a, b)));
        if !kuningas && !pari && !lauta.nosta() {
            return false;
        }
    }
    false
}

#[test]
fn jako_antaa_kaikki_kortit_kerran_ja_vain_alin_rivi_on_vapaa() {
    for siemen in 0..100 {
        let lauta = Lauta::new(Vaikeus::Normaali, &mut Arpa::new(siemen));
        let kortit = kaikki_kortit(&lauta);
        let mut erilaiset = kortit.clone();
        erilaiset.dedup();
        assert_eq!(erilaiset.len(), 52, "siemen {siemen}");
        assert_eq!(lauta.tila.pakka.len(), 52 - PYRAMIDISSA);
        for rivi in 0..RIVEJA {
            for sarake in 0..=rivi {
                assert!(lauta.tila.rivit[rivi][sarake].is_some());
                assert_eq!(lauta.on_vapaa(rivi, sarake), rivi == RIVEJA - 1);
            }
        }
    }
}

#[test]
fn kortti_vapautuu_kun_molemmat_paalta_on_poistettu() {
    let mut lauta = tyhja(Vaikeus::Helppo);
    lauta.tila.rivit[5][0] = Some(k(1));
    lauta.tila.rivit[6][0] = Some(k(12));
    lauta.tila.rivit[6][1] = Some(k(KUNINGAS));
    assert!(!lauta.on_vapaa(5, 0));
    assert_eq!(lauta.pelattava(pyr(5, 0)), None);
    assert!(lauta.poista_kuningas(pyr(6, 1)));
    assert!(!lauta.on_vapaa(5, 0));
    assert!(!lauta.poista_pari(pyr(6, 0), pyr(5, 0)));
    lauta.tila.rivit[6][0] = None;
    assert!(lauta.on_vapaa(5, 0));
}

#[test]
fn parin_summan_on_oltava_13_ja_kuningas_lahtee_yksin() {
    let mut lauta = tyhja(Vaikeus::Helppo);
    lauta.tila.rivit[6][0] = Some(k(6));
    lauta.tila.rivit[6][1] = Some(k(7));
    lauta.tila.rivit[6][2] = Some(k(7));
    lauta.tila.rivit[6][3] = Some(k(KUNINGAS));

    assert!(!lauta.poista_pari(pyr(6, 1), pyr(6, 2)));
    assert!(!lauta.poista_pari(pyr(6, 0), pyr(6, 0)));
    assert!(!lauta.poista_kuningas(pyr(6, 0)));
    assert!(lauta.poista_pari(pyr(6, 0), pyr(6, 1)));
    assert!(lauta.poista_kuningas(pyr(6, 3)));
    assert_eq!(lauta.tila.rivit[6][2], Some(k(7)));
    assert!(!lauta.on_voitettu());
    lauta.tila.rivit[6][2] = None;
    assert!(lauta.on_voitettu());
}

#[test]
fn jatteen_kaksi_paallimmaista_kelpaavat_ja_kolmas_ei() {
    let mut lauta = tyhja(Vaikeus::Helppo);
    lauta.tila.jate = vec![k(2), k(5), k(8)];
    lauta.tila.rivit[6][0] = Some(k(1));
    assert_eq!(lauta.pelattava(Paikka::Jate(0)), Some(k(8)));
    assert_eq!(lauta.pelattava(Paikka::Jate(1)), Some(k(5)));
    assert_eq!(lauta.pelattava(Paikka::Jate(2)), None);
    // Kumpikin järjestys poistaa samat kaksi korttia.
    assert!(lauta.poista_pari(Paikka::Jate(0), Paikka::Jate(1)));
    assert_eq!(lauta.tila.jate, [k(2)]);

    lauta.tila.jate = vec![k(2), k(5), k(8)];
    assert!(lauta.poista_pari(Paikka::Jate(1), Paikka::Jate(0)));
    assert_eq!(lauta.tila.jate, [k(2)]);

    lauta.tila.jate = vec![k(2), k(11), k(9)];
    assert!(!lauta.poista_pari(Paikka::Jate(1), Paikka::Jate(0)));
    lauta.tila.rivit[6][1] = Some(k(4));
    assert!(lauta.poista_pari(pyr(6, 1), Paikka::Jate(0)));
    assert_eq!(lauta.tila.jate, [k(2), k(11)]);
}

#[test]
fn kierrokset_rajaavat_pakan_kaantamista() {
    assert_eq!(kierroksia(Vaikeus::Helppo), None);
    assert_eq!(kierroksia(Vaikeus::Normaali), Some(3));
    assert_eq!(kierroksia(Vaikeus::Vaikea), Some(2));

    let mut lauta = Lauta::new(Vaikeus::Vaikea, &mut Arpa::new(1));
    let jarjestys: Vec<Kortti> = lauta.tila.pakka.iter().rev().copied().collect();
    while !lauta.tila.pakka.is_empty() {
        assert!(lauta.nosta());
    }
    assert_eq!(lauta.tila.jate, jarjestys);
    assert_eq!(lauta.kierros(), 1);
    assert!(lauta.nosta());
    assert_eq!(lauta.kierros(), 2);
    assert_eq!(
        lauta.tila.pakka.iter().rev().copied().collect::<Vec<_>>(),
        jarjestys
    );
    while !lauta.tila.pakka.is_empty() {
        assert!(lauta.nosta());
    }
    assert!(!lauta.nosta());

    let mut lauta = Lauta::new(Vaikeus::Helppo, &mut Arpa::new(1));
    for _ in 0..200 {
        assert!(lauta.nosta());
    }
}

#[test]
fn siirrot_loppuvat_vasta_kun_mikaan_ei_voi_enaa_tulla_vastaan() {
    let mut lauta = tyhja(Vaikeus::Vaikea);
    lauta.tila.rivit[6][0] = Some(k(1));
    assert!(!lauta.on_siirtoja());

    // Pakassa odottava pari riittää.
    lauta.tila.pakka = vec![k(12)];
    assert!(lauta.on_siirtoja());
    // Samoin kuningas.
    lauta.tila.pakka = vec![k(KUNINGAS)];
    assert!(lauta.on_siirtoja());
    // Peräkkäiset pakan kortit voi poistaa keskenään.
    lauta.tila.pakka = vec![k(3), k(10)];
    assert!(lauta.on_siirtoja());
    // Erillään olevat eivät koskaan tapaa.
    lauta.tila.pakka = vec![k(3), k(5), k(10)];
    assert!(!lauta.on_siirtoja());

    // Viimeisellä kierroksella syvällä jätepinossa oleva kortti ei enää tule esiin.
    lauta.tila.pakka.clear();
    lauta.tila.jate = vec![k(12), k(5), k(5)];
    lauta.tila.kierratykset = 1;
    assert!(!lauta.on_siirtoja());
    // Kierrättämällä se tulee.
    lauta.tila.kierratykset = 0;
    assert!(lauta.on_siirtoja());
}

#[test]
fn vaikealla_ei_voi_perua_ja_muualla_voi() {
    let mut lauta = Lauta::new(Vaikeus::Vaikea, &mut Arpa::new(1));
    assert!(lauta.nosta());
    assert!(!lauta.peruuta());
    assert_eq!(lauta.tila.jate.len(), 1);

    let mut lauta = Lauta::new(Vaikeus::Normaali, &mut Arpa::new(1));
    let alku = lauta.tila.clone();
    assert!(!lauta.peruuta());
    assert!(lauta.nosta());
    assert!(lauta.peruuta());
    assert_eq!(lauta.tila, alku);
}

#[test]
fn ahne_pelaaja_paasee_loppuun_ja_helppo_taso_on_voitettavissa() {
    let mut voittoja = 0;
    for siemen in 0..500 {
        let mut lauta = Lauta::new(Vaikeus::Helppo, &mut Arpa::new(siemen));
        let voitti = pelaa_ahneesti(&mut lauta);
        voittoja += voitti as u32;
        // Häviö tarkoittaa, että peli on oikeasti jumissa eikä pelaajan vuorot vain loppuneet.
        assert!(voitti || !lauta.on_siirtoja(), "siemen {siemen}");
    }
    assert!(voittoja > 0, "ahne pelaaja ei voittanut kertaakaan");
}
