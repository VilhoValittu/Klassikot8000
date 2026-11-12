use klassikot8000::Arpa;
use klassikot8000::jatsi::{
    BONUS, HEITTOJA, Nopat, Poyta, Poytakirja, RIVEJA, Rivi, SUURIN_SUMMA, Toimi, Vaikeus,
    koneen_toimi, mahdollinen, pisteet,
};

#[test]
fn rivien_pisteet_pohjoismaisin_saannoin() {
    let tapaukset: [(Rivi, Nopat, u32); 26] = [
        (Rivi::Ykkoset, [1, 1, 3, 1, 6], 3),
        (Rivi::Kuutoset, [6, 6, 6, 6, 2], 24),
        (Rivi::Vitoset, [1, 2, 3, 4, 6], 0),
        (Rivi::Pari, [3, 3, 5, 5, 1], 10),
        (Rivi::Pari, [1, 2, 3, 4, 6], 0),
        (Rivi::Pari, [4, 4, 4, 4, 4], 8),
        (Rivi::KaksiParia, [3, 3, 5, 5, 1], 16),
        // Neljä samaa ei ole kaksi paria.
        (Rivi::KaksiParia, [2, 2, 2, 2, 6], 0),
        (Rivi::KaksiParia, [6, 6, 6, 1, 1], 14),
        (Rivi::KolmeSamaa, [2, 5, 5, 5, 6], 15),
        (Rivi::KolmeSamaa, [2, 5, 5, 6, 6], 0),
        (Rivi::KolmeSamaa, [4, 4, 4, 4, 1], 12),
        (Rivi::NeljaSamaa, [4, 4, 4, 4, 1], 16),
        (Rivi::NeljaSamaa, [6, 6, 6, 6, 6], 24),
        (Rivi::NeljaSamaa, [6, 6, 6, 1, 1], 0),
        (Rivi::PieniSuora, [5, 3, 1, 2, 4], 15),
        (Rivi::PieniSuora, [2, 3, 4, 5, 6], 0),
        (Rivi::SuuriSuora, [6, 2, 4, 3, 5], 20),
        (Rivi::SuuriSuora, [1, 2, 3, 4, 5], 0),
        (Rivi::Tayskasi, [2, 2, 6, 6, 6], 22),
        (Rivi::Tayskasi, [3, 3, 3, 3, 3], 0),
        (Rivi::Tayskasi, [2, 2, 6, 6, 1], 0),
        (Rivi::Sattuma, [1, 2, 3, 4, 6], 16),
        (Rivi::Jatsi, [5, 5, 5, 5, 5], 50),
        (Rivi::Jatsi, [5, 5, 5, 5, 4], 0),
        (Rivi::Kolmoset, [3, 3, 3, 3, 3], 15),
    ];
    for (rivi, nopat, odotettu) in tapaukset {
        assert_eq!(pisteet(rivi, &nopat), odotettu, "{rivi:?} {nopat:?}");
    }
}

/// Jokaisen rivin suurin pistemäärä: yhteensä 324 ja bonus.
const PARHAAT: [u8; RIVEJA] = [5, 10, 15, 20, 25, 30, 12, 22, 18, 24, 15, 20, 28, 30, 50];

#[test]
fn bonus_tulee_rajalta_ja_suurin_summa_on_374() {
    let mut kirja = Poytakirja::default();
    // Kolme kutakin silmälukua on tasan raja.
    for (i, rivi) in Rivi::KAIKKI[..6].iter().enumerate() {
        assert_eq!(kirja.bonus(), 0);
        assert!(kirja.kirjaa(*rivi, 3 * (i as u32 + 1)));
    }
    assert_eq!(kirja.ylaosa(), 63);
    assert_eq!(kirja.bonus(), BONUS);
    assert_eq!(kirja.summa(), 63 + BONUS);
    assert!(
        !kirja.kirjaa(Rivi::Ykkoset, 5),
        "täytettyyn riviin ei kirjata"
    );

    let rivit = PARHAAT.map(Some);
    let taysi = Poytakirja::riveista(rivit).expect("suurimmat pisteet ovat mahdollisia");
    assert!(taysi.taynna());
    assert_eq!(taysi.summa(), SUURIN_SUMMA);
}

#[test]
fn mahdottomat_pisteet_hylataan() {
    assert!(mahdollinen(Rivi::Tayskasi, 28));
    assert!(!mahdollinen(Rivi::Tayskasi, 29));
    assert!(!mahdollinen(Rivi::Ykkoset, 6));
    assert!(!mahdollinen(Rivi::Pari, 7));
    assert!(!mahdollinen(Rivi::Jatsi, 49));
    assert!(mahdollinen(Rivi::Jatsi, 0));
    let mut rivit = [None; RIVEJA];
    rivit[Rivi::Pari.indeksi()] = Some(13);
    assert_eq!(Poytakirja::riveista(rivit), None);
    rivit[Rivi::Pari.indeksi()] = Some(12);
    assert!(Poytakirja::riveista(rivit).is_some());
}

#[test]
fn heitot_lukitukset_ja_kirjaus_noudattavat_vuoroa() {
    let mut arpa = Arpa::new(7);
    let mut poyta = Poyta::new(vec![false, true], 0);
    assert!(!poyta.voi_kirjata(Rivi::Sattuma), "kirjaus vaatii heiton");
    assert!(!poyta.lukitse(0), "ennen heittoa ei lukita");
    assert!(poyta.heita(&mut arpa));
    assert!(poyta.nopat.iter().all(|n| (1..=6).contains(n)));
    assert!(poyta.lukitse(2));
    let lukittu = poyta.nopat[2];
    for _ in 1..HEITTOJA {
        assert!(poyta.heita(&mut arpa));
        assert_eq!(poyta.nopat[2], lukittu);
    }
    assert!(!poyta.heita(&mut arpa), "neljättä heittoa ei ole");
    assert!(!poyta.lukitse(1), "viimeisen heiton jälkeen ei lukita");
    let odotettu = pisteet(Rivi::Sattuma, &poyta.nopat);
    let kirjaus = poyta.kirjaa(Rivi::Sattuma).unwrap();
    assert_eq!((kirjaus.pelaaja, kirjaus.pisteet), (0, odotettu));
    assert_eq!(poyta.vuorossa, 1);
    assert!(poyta.kone_vuorossa());
    assert_eq!(poyta.heitetty, 0);
    assert_eq!(poyta.lukitut, [false; 5]);
    // Uuden vuoron ensimmäinen heitto heittää kaikki, vaikka lukitus jäisi päälle.
    poyta.lukitut = [true; 5];
    poyta.heita(&mut arpa);
    assert_eq!(poyta.lukitut, [false; 5]);
}

/// Pelaa pöydän loppuun koneiden päätöksillä ja tarkistaa, ettei mikään toimi ole laiton.
fn pelaa(poyta: &mut Poyta, tasot: &[Vaikeus], arpa: &mut Arpa) {
    let mut toimia = 0;
    while !poyta.ohi() {
        toimia += 1;
        assert!(toimia < 10_000, "peli ei etene");
        let vaikeus = tasot[poyta.vuorossa];
        match koneen_toimi(vaikeus, poyta) {
            Toimi::Heita => assert!(poyta.heita(arpa)),
            Toimi::Lukitse(lukitut) => {
                assert!(poyta.voi_lukita());
                assert_ne!(lukitut, poyta.lukitut, "lukitus muuttuu");
                poyta.lukitut = lukitut;
                assert!(poyta.heita(arpa));
            }
            Toimi::Kirjaa(rivi) => {
                assert!(poyta.kirja().vapaa(rivi), "{rivi:?} on jo kirjattu");
                assert!(poyta.kirjaa(rivi).is_some());
            }
        }
    }
}

#[test]
fn koneet_pelaavat_kaikki_rivit_vuorotellen() {
    for siemen in 0..12 {
        let mut arpa = Arpa::new(siemen);
        let mut poyta = Poyta::new(vec![true; 4], siemen as usize);
        let tasot = [
            Vaikeus::Helppo,
            Vaikeus::Normaali,
            Vaikeus::Vaikea,
            Vaikeus::Helppo,
        ];
        pelaa(&mut poyta, &tasot, &mut arpa);
        for kirja in &poyta.kirjat {
            assert!(kirja.taynna());
            assert!(kirja.summa() <= SUURIN_SUMMA);
        }
        let voittajat = poyta.voittajat();
        assert!(!voittajat.is_empty());
        for p in 0..4 {
            assert_eq!(poyta.sija(p) == 1, voittajat.contains(&p));
        }
    }
}

#[test]
fn vaikeampi_kone_saa_enemman_pisteita() {
    const PELEJA: u64 = 30;
    let keskiarvo = |vaikeus: Vaikeus| {
        let summa: u32 = (0..PELEJA)
            .map(|siemen| {
                let mut arpa = Arpa::new(siemen);
                let mut poyta = Poyta::new(vec![true], 0);
                pelaa(&mut poyta, &[vaikeus], &mut arpa);
                poyta.kirjat[0].summa()
            })
            .sum();
        summa / PELEJA as u32
    };
    let [helppo, normaali, vaikea] = Vaikeus::KAIKKI.map(keskiarvo);
    assert!(helppo < normaali, "{helppo} {normaali}");
    assert!(normaali < vaikea, "{normaali} {vaikea}");
    assert!(vaikea > 220, "vaikea pelaa hyvin: {vaikea}");
}

#[test]
fn kone_ei_heita_valmista_jatsia() {
    let mut poyta = Poyta::new(vec![true], 0);
    poyta.heitetty = 1;
    poyta.nopat = [4; 5];
    for vaikeus in Vaikeus::KAIKKI {
        assert_eq!(
            koneen_toimi(vaikeus, &poyta),
            Toimi::Kirjaa(Rivi::Jatsi),
            "{vaikeus:?}"
        );
    }
}

#[test]
fn jaettu_karki_on_kaikkien_voitto() {
    let mut poyta = Poyta::new(vec![false, false, true], 0);
    for kirja in &mut poyta.kirjat {
        kirja.kirjaa(Rivi::Sattuma, 20);
    }
    poyta.kirjat[2].kirjaa(Rivi::Ykkoset, 1);
    assert_eq!(poyta.voittajat(), vec![2]);
    poyta.kirjat[0].kirjaa(Rivi::Kakkoset, 2);
    assert_eq!(poyta.voittajat(), vec![0]);
    poyta.kirjat[1].kirjaa(Rivi::Ykkoset, 2);
    assert_eq!(poyta.voittajat(), vec![0, 1]);
    assert_eq!(poyta.sija(2), 3);
}
