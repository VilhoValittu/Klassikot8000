use std::collections::VecDeque;

use klassikot8000::Arpa;
use klassikot8000::silmukka::{
    ALKUPITUUS, ASKEL_MS, Herkku, Kohde, LEPO_MS, Laji, Niitty, Otus, RIVEJA, Ruutu, SARAKKEITA,
    Suunta, Tapahtuma, kentalla, otuksia_enintaan, saaliin_pisteet, sisapuoli,
};

/// Rengas vasemmasta yläkulmasta myötäpäivään. Pää jää kahden ruudun päähän sulkeutumisesta,
/// ja häntä jatkuu renkaan ulkopuolelle, jotta se pysyy paikoillaan koko kokeen ajan.
fn rengas() -> Vec<Ruutu> {
    let mut ruudut = vec![(1, 5), (2, 5), (3, 5), (4, 5)];
    ruudut.extend((5..=15).map(|x| (x, 5)));
    ruudut.extend((6..=15).map(|y| (15, y)));
    ruudut.extend((5..=14).rev().map(|x| (x, 15)));
    ruudut.extend((8..=14).rev().map(|y| (5, y)));
    ruudut
}

/// Tyhjä niitty, jonka mato kulkee annetut ruudut hännästä päähän.
fn niitty(hannasta: &[Ruutu], suunta: Suunta) -> (Niitty, Arpa) {
    let mut arpa = Arpa::new(1);
    let mut niitty = Niitty::new(&mut arpa);
    niitty.mato = hannasta.iter().rev().copied().collect::<VecDeque<_>>();
    niitty.jalki = hannasta[0];
    niitty.suunta = suunta;
    niitty.otukset.clear();
    niitty.herkut.clear();
    (niitty, arpa)
}

fn itikka(paikka: Ruutu) -> Otus {
    Otus::uusi(Laji::Itikka, paikka, (1, 1), 0)
}

fn omena(paikka: Ruutu) -> Herkku {
    Herkku {
        paikka,
        kulta_ms: None,
    }
}

/// Ajaa, kunnes mato on ottanut `n` askelta, ja palauttaa sinä aikana syntyneet tapahtumat.
fn askeleita(niitty: &mut Niitty, arpa: &mut Arpa, n: usize) -> Vec<Tapahtuma> {
    let mut tapahtumat = Vec::new();
    for _ in 0..n {
        niitty.odotus_ms = ASKEL_MS;
        tapahtumat.extend(niitty.askel(arpa));
    }
    tapahtumat
}

fn tarkista_mato(niitty: &Niitty) {
    let mato: Vec<Ruutu> = niitty.mato.iter().copied().collect();
    for (i, &ruutu) in mato.iter().enumerate() {
        assert!(kentalla(ruutu), "{ruutu:?}");
        assert!(!mato[i + 1..].contains(&ruutu), "{ruutu:?} kahdesti");
    }
    for pari in mato.windows(2) {
        let vali = (pari[0].0 - pari[1].0).abs() + (pari[0].1 - pari[1].1).abs();
        assert_eq!(vali, 1, "{pari:?}");
    }
    for otus in &niitty.otukset {
        assert!(kentalla(otus.paikka));
        // Pään ruutuun otus pääsee vain lopettamalla pelin.
        if !niitty.ohi {
            assert!(!mato.contains(&otus.paikka), "{otus:?} madossa");
        }
    }
}

#[test]
fn alussa_mato_on_keskella_ja_kentalla_on_omenat_ja_muna() {
    for siemen in 0..50 {
        let mut arpa = Arpa::new(siemen);
        let niitty = Niitty::new(&mut arpa);
        assert_eq!(niitty.mato.len(), ALKUPITUUS);
        assert_eq!(niitty.herkut.len(), 3);
        assert_eq!(niitty.otukset.len(), 1);
        assert!(niitty.otukset[0].on_muna());
        tarkista_mato(&niitty);
    }
}

#[test]
fn sisapuoli_loytaa_renkaan_sisuksen_myos_seinan_vierella() {
    let rengas: Vec<Ruutu> = (2..=5)
        .flat_map(|x| [(x, 2), (x, 5)])
        .chain((3..=4).flat_map(|y| [(2, y), (5, y)]))
        .collect();
    assert_eq!(sisapuoli(&rengas), [(3, 3), (4, 3), (3, 4), (4, 4)]);
    assert!(sisapuoli(&[(1, 1), (2, 1), (2, 2), (1, 2)]).is_empty());

    // nurkkaan piirretty rengas ei rajaudu seinästä, vaan vain omasta vartalostaan
    let nurkka: Vec<Ruutu> = (0..=2)
        .flat_map(|x| [(x, 0), (x, 2)])
        .chain([(0, 1), (2, 1)])
        .collect();
    assert_eq!(sisapuoli(&nurkka), [(1, 1)]);

    // pelkkä seinään nojaava kaari ei sulje mitään
    let kaari = [(0, 3), (1, 3), (1, 4), (1, 5), (0, 5)];
    assert!(sisapuoli(&kaari).is_empty());
}

#[test]
fn nuolet_jonottavat_kaksi_kaannosta_eika_peruutus_kelpaa() {
    let mut arpa = Arpa::new(1);
    let mut niitty = Niitty::new(&mut arpa);
    niitty.otukset.clear();
    niitty.kaanny(Suunta::Vasen);
    assert!(niitty.jono.is_empty());
    niitty.kaanny(Suunta::Oikea);
    assert!(niitty.jono.is_empty());
    niitty.kaanny(Suunta::Ylos);
    niitty.kaanny(Suunta::Ylos);
    niitty.kaanny(Suunta::Vasen);
    niitty.kaanny(Suunta::Alas);
    assert_eq!(niitty.jono, [Suunta::Ylos, Suunta::Vasen]);

    let (x, y) = niitty.paa();
    askeleita(&mut niitty, &mut arpa, 2);
    assert_eq!(niitty.paa(), (x - 1, y - 1));
    assert_eq!(niitty.suunta, Suunta::Vasen);
    tarkista_mato(&niitty);
}

#[test]
fn seina_antaa_yhden_askeleen_armon_ja_sitten_tappaa() {
    let (mut n, mut arpa) = niitty(&[(23, 3), (24, 3), (25, 3), (26, 3)], Suunta::Oikea);
    assert_eq!(
        askeleita(&mut n, &mut arpa, 1),
        [Tapahtuma::Seina],
        "ensimmäinen osuma"
    );
    assert_eq!(n.paa(), (26, 3));
    assert!(!n.ohi);
    n.kaanny(Suunta::Alas);
    assert!(askeleita(&mut n, &mut arpa, 1).is_empty());
    assert_eq!(n.paa(), (26, 4));

    // armo palautuu seuraavaa seinää varten
    n.kaanny(Suunta::Oikea);
    askeleita(&mut n, &mut arpa, 1);
    assert!(!n.ohi);
    let tapahtumat = askeleita(&mut n, &mut arpa, 1);
    assert!(n.ohi);
    assert_eq!(tapahtumat, [Tapahtuma::Kuolema { paikka: (26, 4) }]);
}

#[test]
fn silmukka_pyydystaa_sisalle_jaavat_ja_lyhentaa_matoa() {
    let (mut n, mut arpa) = niitty(&rengas(), Suunta::Ylos);
    let pituus = n.mato.len();
    n.otukset = vec![
        itikka((10, 10)),
        Otus::uusi(Laji::Paarma, (8, 12), (0, 0), 0),
        Otus::uusi(Laji::Itikka, (7, 7), (0, 0), 900),
        // ulkopuolella säilyy
        itikka((20, 18)),
    ];
    n.herkut = vec![omena((12, 12)), omena((22, 2))];
    // Otukset eivät saa ehtiä liikkua kokeen aikana.
    for otus in &mut n.otukset {
        otus.odotus_ms = 1000;
        otus.jakso_ms = 1000;
    }

    assert!(askeleita(&mut n, &mut arpa, 2).is_empty());
    let tapahtumat = askeleita(&mut n, &mut arpa, 1);
    let [Tapahtuma::Silmukka(saalis)] = tapahtumat.as_slice() else {
        panic!("{tapahtumat:?}");
    };
    let arvot = Kohde::Itikka.arvo() + Kohde::Paarma.arvo() + Kohde::Muna.arvo();
    let odotetut = (arvot + Kohde::Omena.arvo()) * 4;
    assert_eq!(saalis.pisteet, odotetut);
    assert_eq!(n.pisteet, odotetut);
    assert_eq!(saalis.kohteet.len(), 4);
    assert_eq!(saalis.reitti.first(), Some(&(5, 6)));
    assert_eq!(saalis.reitti.last(), Some(&(5, 5)));
    assert_eq!(saalis.sisus.len(), 9 * 9);

    // Pää on risteyksessä, jäljellä on vain renkaan ulkopuolinen häntä, joka lyheni kaksi askelta.
    assert_eq!(n.paa(), (5, 5));
    assert_eq!(n.mato.len(), 3);
    assert!(n.mato.len() < pituus);
    assert_eq!(n.otukset.len(), 1);
    assert_eq!(n.otukset[0].paikka, (20, 18));
    assert!(n.herkut.iter().any(|h| h.paikka == (22, 2)));
    // Syöty omena tulee takaisin jonnekin muualle.
    assert_eq!(n.herkut.len(), 3);
    // 39 ruudun silmukka maksaa neljänneksensä (9), loput 30 ja saaliin 4 * 2 kasvavat takaisin.
    assert_eq!(n.kasvu, 30 + 8);
    assert_eq!(n.mato.len() + n.kasvu as usize, pituus - 9 + 8);
    assert!(n.paikallaan);
    tarkista_mato(&n);

    // Häntä odottaa, kunnes pituus on kasvanut takaisin.
    let hanta = *n.mato.back().unwrap();
    n.kaanny(Suunta::Oikea);
    askeleita(&mut n, &mut arpa, 1);
    assert_eq!(*n.mato.back().unwrap(), hanta);
    assert_eq!(n.mato.len(), 4);
    tarkista_mato(&n);
}

#[test]
fn pisteet_kasvavat_saaliin_nelion_mukaan() {
    assert_eq!(saaliin_pisteet([]), 0);
    assert_eq!(saaliin_pisteet([Kohde::Itikka]), 25);
    assert_eq!(saaliin_pisteet([Kohde::Itikka; 2]), 100);
    assert_eq!(saaliin_pisteet([Kohde::Itikka; 10]), 2500);
}

#[test]
fn tyhja_silmukka_ei_tapa_mutta_kuluttaa_pituuden_ja_jattaa_alkupituuden_kasvuksi() {
    let (mut n, mut arpa) = niitty(
        &[
            (5, 3),
            (6, 3),
            (6, 4),
            (5, 4),
            (4, 4),
            (4, 3),
            (4, 2),
            (5, 2),
        ],
        Suunta::Oikea,
    );
    n.kaanny(Suunta::Alas);
    // (5, 2) -> (5, 3) on häntä, joka lähtee alta samalla askeleella: ei silmukkaa.
    assert!(askeleita(&mut n, &mut arpa, 1).is_empty());
    assert_eq!(n.paa(), (5, 3));
    assert_eq!(n.mato.len(), 8);

    // Seuraava askel osuu vartaloon, ja silmukka on tyhjä.
    let tapahtumat = askeleita(&mut n, &mut arpa, 1);
    assert!(matches!(
        tapahtumat.as_slice(),
        [Tapahtuma::TyhjaSilmukka { .. }]
    ));
    assert!(!n.ohi);
    assert_eq!(n.pisteet, 0);
    assert_eq!(n.paa(), (5, 4));
    assert_eq!(n.mato.len() + n.kasvu as usize, 5);
    tarkista_mato(&n);
}

#[test]
fn kasvavan_madon_hanta_ei_vapauta_ruutuaan() {
    let hannasta = [
        (5, 3),
        (6, 3),
        (6, 4),
        (5, 4),
        (4, 4),
        (4, 3),
        (4, 2),
        (5, 2),
    ];
    let (mut n, mut arpa) = niitty(&hannasta, Suunta::Oikea);
    n.kasvu = 1;
    n.kaanny(Suunta::Alas);
    let tapahtumat = askeleita(&mut n, &mut arpa, 1);
    assert!(matches!(
        tapahtumat.as_slice(),
        [Tapahtuma::TyhjaSilmukka { .. }]
    ));
    assert_eq!(n.paa(), (5, 3));
}

#[test]
fn omena_kasvattaa_ja_palaa_kentalle_ja_muna_litistyy() {
    let (mut n, mut arpa) = niitty(&[(5, 5), (6, 5), (7, 5)], Suunta::Oikea);
    n.herkut = vec![omena((8, 5))];
    n.otukset = vec![Otus::uusi(Laji::Itikka, (9, 5), (1, 1), 1000)];
    let tapahtumat = askeleita(&mut n, &mut arpa, 1);
    assert_eq!(
        tapahtumat,
        [Tapahtuma::Omena {
            paikka: (8, 5),
            pisteet: 10
        }]
    );
    assert_eq!(n.kasvu, 3);
    assert_eq!(n.herkut.len(), 3);

    let tapahtumat = askeleita(&mut n, &mut arpa, 1);
    assert_eq!(
        tapahtumat,
        [Tapahtuma::Muna {
            paikka: (9, 5),
            pisteet: 15
        }]
    );
    assert_eq!(n.mato.len(), 4);
    assert!(n.otukset.is_empty());
    assert_eq!(n.pisteet, 25);
}

#[test]
fn itikkaan_tormays_tappaa_ja_peli_pysahtyy() {
    let (mut n, mut arpa) = niitty(&[(5, 5), (6, 5), (7, 5)], Suunta::Oikea);
    n.otukset = vec![itikka((8, 5))];
    n.otukset[0].odotus_ms = 1000;
    n.otukset[0].jakso_ms = 1000;
    let tapahtumat = askeleita(&mut n, &mut arpa, 1);
    assert_eq!(tapahtumat, [Tapahtuma::Kuolema { paikka: (7, 5) }]);
    assert!(n.ohi);
    assert_eq!(n.paa(), (7, 5));

    // Pelin jälkeen mato ei liiku, vaikka otukset lentävät.
    n.otukset[0].odotus_ms = ASKEL_MS;
    let (paikka, mato) = (n.otukset[0].paikka, n.mato.clone());
    assert!(n.askel(&mut arpa).is_empty());
    assert_ne!(n.otukset[0].paikka, paikka);
    assert_eq!(n.mato, mato);
}

#[test]
fn itikka_kimpoaa_seinasta_ja_vartalosta_eika_pujahda_nurkan_lapi() {
    let (mut n, mut arpa) = niitty(&[(5, 5), (6, 5), (7, 5)], Suunta::Oikea);
    // Ei liikuteta matoa tässä kokeessa.
    n.odotus_ms = 1000;
    n.otukset = vec![Otus::uusi(Laji::Itikka, (0, 10), (-1, 1), 0)];
    n.otukset[0].odotus_ms = ASKEL_MS;
    n.askel(&mut arpa);
    assert_eq!(n.otukset[0].paikka, (1, 11));
    assert_eq!(n.otukset[0].suunta, (1, 1));

    // Vartalo kulkee itikan edestä vaakasuunnassa.
    n.otukset = vec![Otus::uusi(Laji::Itikka, (6, 7), (1, -1), 0)];
    n.mato = VecDeque::from([(7, 6), (6, 6), (5, 6)]);
    n.otukset[0].odotus_ms = ASKEL_MS;
    n.askel(&mut arpa);
    assert_eq!(n.otukset[0].suunta.1, 1);
    assert!(n.otukset[0].paikka.1 > 7);

    // Ahtaassa kolossa itikka pysyy paikallaan.
    n.mato = VecDeque::from([
        (10, 9),
        (11, 9),
        (11, 10),
        (11, 11),
        (10, 11),
        (9, 11),
        (9, 10),
        (9, 9),
    ]);
    n.otukset = vec![Otus::uusi(Laji::Itikka, (10, 10), (1, 1), 0)];
    n.otukset[0].odotus_ms = ASKEL_MS;
    n.askel(&mut arpa);
    assert_eq!(n.otukset[0].paikka, (10, 10));
}

#[test]
fn itikka_lentaa_ja_laskeutuu_lepaamaan_mutta_pistaa_levossakin() {
    let (mut n, mut arpa) = niitty(&[(20, 20), (21, 20), (22, 20)], Suunta::Oikea);
    n.odotus_ms = 1_000_000;
    n.otukset = vec![itikka((2, 2))];
    let mut askeleet = 0;
    while n.otukset[0].lepo_ms == 0 {
        n.askel(&mut arpa);
        askeleet += 1;
        assert!(askeleet < 2000, "itikka ei laskeudu");
    }
    let paikka = n.otukset[0].paikka;
    assert_ne!(paikka, (2, 2));
    let lepo = n.otukset[0].lepo_ms;
    for _ in 0..lepo / ASKEL_MS {
        n.askel(&mut arpa);
        assert_eq!(n.otukset[0].paikka, paikka);
    }
    assert_eq!(n.otukset[0].lepo_ms, 0);
    for _ in 0..200 {
        n.askel(&mut arpa);
    }
    assert_ne!(n.otukset[0].paikka, paikka, "lepo loppuu");

    // Levossa oleva itikka tappaa yhä törmääjän.
    let (mut n, mut arpa) = niitty(&[(5, 5), (6, 5), (7, 5)], Suunta::Oikea);
    n.otukset = vec![itikka((8, 5))];
    n.otukset[0].lepo_ms = 10_000;
    askeleita(&mut n, &mut arpa, 1);
    assert!(n.ohi);
}

/// Ajaa pelin omalla aikataululla ja kääntää madon kullakin sen askeleella.
fn aja_reitti(n: &mut Niitty, arpa: &mut Arpa, reitti: &[Suunta]) -> Vec<Tapahtuma> {
    let mut tapahtumat = Vec::new();
    for &suunta in reitti {
        n.kaanny(suunta);
        loop {
            let viimeinen = n.odotus_ms <= ASKEL_MS;
            tapahtumat.extend(n.askel(arpa));
            if viimeinen {
                break;
            }
        }
    }
    tapahtumat
}

#[test]
fn levossa_olevan_itikan_ehtii_kiertaa_ensimmaisella_tasolla() {
    use Suunta::{Alas, Oikea, Vasen, Ylos};
    // Ensimmäisen tason hitaalla madolla: tiukka rengas ja väljempi, jonka sisään itikka jää.
    let tiukka: &[Suunta] = &[Oikea, Oikea, Alas, Alas, Vasen, Vasen, Ylos, Ylos];
    let valjempi: &[Suunta] = &[
        Oikea, Oikea, Oikea, Oikea, Alas, Alas, Alas, Alas, Vasen, Vasen, Vasen, Vasen, Ylos, Ylos,
        Ylos, Ylos,
    ];
    for (reitti, itikka_paikka) in [(tiukka, (10, 10)), (valjempi, (11, 11))] {
        let hannasta: Vec<Ruutu> = (0..=9).map(|x| (x, 9)).collect();
        let (mut n, mut arpa) = niitty(&hannasta, Suunta::Oikea);
        n.kasvu = 8;
        let mut otus = itikka(itikka_paikka);
        otus.lepo_ms = LEPO_MS;
        n.otukset = vec![otus];
        let tapahtumat = aja_reitti(&mut n, &mut arpa, reitti);
        assert!(
            tapahtumat.iter().any(|t| matches!(
                t,
                Tapahtuma::Silmukka(saalis) if saalis.kohteet.len() == 1
            )),
            "{tapahtumat:?}"
        );
        assert!(n.otukset.is_empty());
        assert_eq!(n.pisteet, Kohde::Itikka.arvo());
        assert!(!n.ohi);
    }
}

#[test]
fn paarma_jahtaa_paata_ja_saa_sen_kiinni_mutta_ei_kulje_vartalon_lapi() {
    let (mut n, mut arpa) = niitty(&[(5, 5), (6, 5), (7, 5)], Suunta::Oikea);
    n.odotus_ms = 1_000_000;
    n.otukset = vec![Otus::uusi(Laji::Paarma, (7, 9), (0, 0), 0)];
    let (mut paikka, mut tapahtumat) = (n.otukset[0].paikka, Vec::new());
    for _ in 0..100_000 {
        tapahtumat.extend(n.askel(&mut arpa));
        let uusi = n.otukset[0].paikka;
        assert!(uusi == paikka || (uusi.0 - paikka.0).abs() + (uusi.1 - paikka.1).abs() == 1);
        paikka = uusi;
        if n.ohi {
            break;
        }
    }
    assert!(n.ohi);
    assert!(tapahtumat.contains(&Tapahtuma::Kuolema { paikka: (7, 5) }));

    // Vartalon takana paarma jää odottamaan.
    let (mut n, mut arpa) = niitty(&[(5, 5), (6, 5), (7, 5)], Suunta::Oikea);
    n.odotus_ms = 1_000_000;
    n.otukset = vec![Otus::uusi(Laji::Paarma, (5, 8), (0, 0), 0)];
    n.mato = VecDeque::from([(5, 3), (5, 4), (5, 5), (5, 6)]);
    for _ in 0..400 {
        n.askel(&mut arpa);
    }
    assert!(!n.ohi);
    assert_eq!(n.otukset[0].paikka, (5, 7));
}

#[test]
fn ennuste_vastaa_toteutunutta_silmukkaa() {
    let (mut n, mut arpa) = niitty(&rengas(), Suunta::Ylos);
    n.herkut = vec![omena((10, 10)), omena((22, 2))];
    n.otukset = vec![Otus::uusi(Laji::Itikka, (7, 7), (0, 0), 100_000)];
    let ennuste = n.ennuste().expect("edessä on oma vartalo");
    assert_eq!(ennuste.suora, 2);
    assert_eq!(ennuste.saalis, 2);
    assert_eq!(
        ennuste.pisteet,
        saaliin_pisteet([Kohde::Omena, Kohde::Muna])
    );
    let tapahtumat = askeleita(&mut n, &mut arpa, 3);
    let saalis = tapahtumat
        .iter()
        .find_map(|t| match t {
            Tapahtuma::Silmukka(saalis) => Some(saalis),
            _ => None,
        })
        .expect("silmukka sulkeutui");
    assert_eq!(saalis.pisteet, ennuste.pisteet);
    assert_eq!(saalis.sisus, ennuste.sisus);
}

#[test]
fn ennustetta_ei_ole_kun_edessa_on_seina_tai_hanta_ehtii_alta() {
    let (n, _) = niitty(&[(5, 5), (6, 5), (7, 5)], Suunta::Oikea);
    assert_eq!(n.ennuste(), None);

    // Vartalo edessä, mutta se ehtii pois ennen kuin pää saapuu.
    let (n, _) = niitty(
        &[(8, 2), (8, 3), (8, 4), (7, 4), (6, 4), (5, 4), (5, 3)],
        Suunta::Oikea,
    );
    assert_eq!(n.ennuste(), None);
}

#[test]
fn tahtain_ohjaa_paata_eika_kaanna_seinaan_tai_peruuta() {
    let (mut n, _) = niitty(&[(5, 5), (6, 5), (7, 5)], Suunta::Oikea);
    n.tahtain = Some((7, 15));
    assert_eq!(n.seuraava_suunta(), Suunta::Alas);
    n.tahtain = Some((20, 6));
    assert_eq!(n.seuraava_suunta(), Suunta::Oikea);
    // suoraan takana: kierretään sivun kautta eikä peruuteta
    n.tahtain = Some((2, 5));
    assert!(matches!(n.seuraava_suunta(), Suunta::Ylos | Suunta::Alas));

    n.kaanny(Suunta::Ylos);
    assert_eq!(n.tahtain, None);
    assert_eq!(n.seuraava_suunta(), Suunta::Ylos);

    // Tähtäimen ollessa kohdalla seinän viereltä käännytään sivuun.
    let (mut n, _) = niitty(&[(24, 3), (25, 3), (26, 3)], Suunta::Oikea);
    n.tahtain = Some((26, 3));
    assert!(matches!(n.seuraava_suunta(), Suunta::Ylos | Suunta::Alas));
}

#[test]
fn paikallaan_pysyva_tahtain_ei_syo_matoa() {
    let mut arpa = Arpa::new(5);
    let mut n = Niitty::new(&mut arpa);
    n.otukset.clear();
    n.herkut.clear();
    let pituus = n.mato.len();
    n.aseta_tahtain((13, 11));
    let tapahtumat = askeleita(&mut n, &mut arpa, 200);
    assert!(
        !tapahtumat
            .iter()
            .any(|t| matches!(t, Tapahtuma::TyhjaSilmukka { .. })),
        "{tapahtumat:?}"
    );
    assert_eq!(n.mato.len(), pituus);
    assert!(!n.ohi);
}

#[test]
fn tahtain_vartalon_takana_sulkee_silmukan() {
    let (mut n, mut arpa) = niitty(&rengas(), Suunta::Ylos);
    n.aseta_tahtain((5, 2));
    let tapahtumat = askeleita(&mut n, &mut arpa, 3);
    assert!(matches!(
        tapahtumat.as_slice(),
        [Tapahtuma::TyhjaSilmukka { .. }]
    ));
    assert_eq!(n.paa(), (5, 5));
}

#[test]
fn taso_nousee_ajan_kuluessa_ja_vauhti_kasvaa() {
    let mut arpa = Arpa::new(2);
    let mut n = Niitty::new(&mut arpa);
    let alussa = n.jakso_ms;
    n.peliaika_ms = 60_000 - ASKEL_MS;
    n.otukset.clear();
    let tapahtumat = n.askel(&mut arpa);
    assert!(tapahtumat.contains(&Tapahtuma::UusiTaso));
    assert_eq!(n.taso, 4);
    n.odotus_ms = ASKEL_MS;
    n.askel(&mut arpa);
    assert!(n.jakso_ms < alussa);
}

/// Yksinkertainen pelaaja: hakee lähintä omenaa ja väistää seiniä ja otuksia.
fn varova_suunta(n: &Niitty) -> Suunta {
    let paa = n.paa();
    let kohde = n.herkut.first().map_or((13, 11), |h| h.paikka);
    let turvallinen = |s: Suunta| {
        let (dx, dy) = s.siirto();
        let kohta = (paa.0 + dx, paa.1 + dy);
        kentalla(kohta)
            && !n.mato.contains(&kohta)
            && !n
                .otukset
                .iter()
                .any(|o| (o.paikka.0 - kohta.0).abs() + (o.paikka.1 - kohta.1).abs() <= 1)
    };
    let mut suunnat = [Suunta::Ylos, Suunta::Alas, Suunta::Vasen, Suunta::Oikea];
    let etaisyys = |s: Suunta| {
        let (dx, dy) = s.siirto();
        (paa.0 + dx - kohde.0).abs() + (paa.1 + dy - kohde.1).abs()
    };
    suunnat.sort_by_key(|&s| etaisyys(s));
    suunnat
        .into_iter()
        .find(|&s| turvallinen(s))
        .unwrap_or(n.suunta)
}

#[test]
fn parvi_kasvaa_vasta_nopeuksien_jalkeen_ja_katto_tulee_vastaan() {
    assert_eq!(otuksia_enintaan(1), 16);
    assert_eq!(otuksia_enintaan(13), 16);
    assert_eq!(otuksia_enintaan(14), 17);
    assert_eq!(otuksia_enintaan(1000), 30);
}

#[test]
fn pitka_peli_pitaa_saannot_voimassa_ja_paattyy_aikanaan() {
    for siemen in 0..40 {
        let mut arpa = Arpa::new(siemen);
        let mut n = Niitty::new(&mut arpa);
        let mut askeleet = 0u32;
        while !n.ohi {
            if n.odotus_ms <= ASKEL_MS {
                n.kaanny(varova_suunta(&n));
            }
            for tapahtuma in n.askel(&mut arpa) {
                if let Tapahtuma::Silmukka(saalis) = tapahtuma {
                    assert!(!saalis.kohteet.is_empty());
                    assert!(saalis.pisteet > 0);
                }
            }
            tarkista_mato(&n);
            assert!(n.mato.len() >= 2);
            assert!(n.otukset.len() <= otuksia_enintaan(n.taso));
            askeleet += 1;
            assert!(askeleet < 20_000_000, "siemen {siemen}: peli ei pääty");
        }
        assert!(n.pisteet > 0, "siemen {siemen}");
    }
}

#[test]
fn sama_siemen_ja_ohjaus_toistavat_saman_pelin() {
    let aja = || {
        let mut arpa = Arpa::new(9);
        let mut n = Niitty::new(&mut arpa);
        let mut tapahtumat = Vec::new();
        for i in 0..40_000 {
            if i % 700 == 0 {
                n.kaanny([Suunta::Ylos, Suunta::Vasen, Suunta::Alas, Suunta::Oikea][i / 700 % 4]);
            }
            tapahtumat.extend(n.askel(&mut arpa));
        }
        (tapahtumat, n.pisteet, n.mato)
    };
    assert_eq!(aja(), aja());
}

#[test]
fn kentan_mitat_ovat_jarkevat() {
    const { assert!(SARAKKEITA > 2 * ALKUPITUUS as i32) };
    const { assert!(RIVEJA > 8) };
}
