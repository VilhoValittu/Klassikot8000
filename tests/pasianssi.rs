use klassikot8000::Arpa;
use klassikot8000::pasianssi::{
    KORTTEJA, KUNINGAS, Kohde, Kortti, Lahde, Lauta, MAITA, Maa, SARAKKEITA, SUURIN_TULOS, Sarake,
    Vaikeus, aikapisteet,
};

fn kaikki_kortit(lauta: &Lauta) -> Vec<(u8, u8)> {
    let tila = &lauta.tila;
    let mut kortit: Vec<(u8, u8)> = tila
        .pakka
        .iter()
        .chain(&tila.jate)
        .chain(tila.perustat.iter().flatten())
        .chain(tila.sarakkeet.iter().flat_map(|s| &s.kortit))
        .map(|k| (k.maa as u8, k.arvo))
        .collect();
    kortit.sort_unstable();
    kortit
}

fn tyhja(vaikeus: Vaikeus) -> Lauta {
    let mut lauta = Lauta::new(vaikeus, &mut Arpa::new(1));
    lauta.tila.pakka.clear();
    lauta.tila.sarakkeet = Default::default();
    lauta
}

fn k(maa: Maa, arvo: u8) -> Kortti {
    Kortti { maa, arvo }
}

fn sarake(kortit: &[Kortti]) -> Sarake {
    Sarake {
        kortit: kortit.to_vec(),
        piilossa: 0,
    }
}

const JATE: Lahde = Lahde::Jate;

#[test]
fn jako_antaa_kaikki_kortit_kerran_ja_sarakkeet_kasvavat() {
    for siemen in 0..200 {
        let lauta = Lauta::new(Vaikeus::Normaali, &mut Arpa::new(siemen));
        let kortit = kaikki_kortit(&lauta);
        assert_eq!(kortit.len(), KORTTEJA);
        let mut erilaiset = kortit.clone();
        erilaiset.dedup();
        assert_eq!(erilaiset.len(), KORTTEJA, "siemen {siemen}");
        assert_eq!(lauta.tila.pakka.len(), 24);
        for (i, sarake) in lauta.tila.sarakkeet.iter().enumerate() {
            assert_eq!(sarake.kortit.len(), i + 1);
            assert_eq!(sarake.piilossa, i);
        }
    }
}

#[test]
fn eri_siemenet_jakavat_eri_tavalla() {
    let a = Lauta::new(Vaikeus::Helppo, &mut Arpa::new(1));
    let b = Lauta::new(Vaikeus::Helppo, &mut Arpa::new(2));
    assert_ne!(a.tila.pakka, b.tila.pakka);
}

#[test]
fn vaikeus_maaraa_noston_kierrokset_ja_peruutuksen() {
    let mut helppo = Lauta::new(Vaikeus::Helppo, &mut Arpa::new(1));
    assert!(helppo.nosta());
    assert_eq!(helppo.tila.jate.len(), 1);

    let mut normaali = Lauta::new(Vaikeus::Normaali, &mut Arpa::new(1));
    assert!(normaali.nosta());
    assert_eq!(normaali.tila.jate.len(), 3);
    assert!(normaali.peruuta());
    assert!(normaali.tila.jate.is_empty());

    let mut vaikea = Lauta::new(Vaikeus::Vaikea, &mut Arpa::new(1));
    assert!(vaikea.nosta());
    assert!(!vaikea.peruuta());
    assert_eq!(vaikea.tila.jate.len(), 3);
}

#[test]
fn vaikealla_pakka_kiertaa_kolmesti_ja_muilla_rajatta() {
    let mut vaikea = Lauta::new(Vaikeus::Vaikea, &mut Arpa::new(1));
    let mut kierroksia = 1;
    while vaikea.nosta() {
        kierroksia = vaikea.kierros();
    }
    assert_eq!(kierroksia, 3);
    assert!(vaikea.tila.pakka.is_empty());

    let mut normaali = Lauta::new(Vaikeus::Normaali, &mut Arpa::new(1));
    for _ in 0..200 {
        assert!(normaali.nosta());
    }
    assert!(normaali.kierros() > 3);
}

#[test]
fn kierratys_palauttaa_pakan_samaan_jarjestykseen() {
    let mut lauta = Lauta::new(Vaikeus::Helppo, &mut Arpa::new(5));
    let alku = lauta.tila.pakka.clone();
    while !lauta.tila.pakka.is_empty() {
        lauta.nosta();
    }
    assert!(lauta.nosta());
    assert_eq!(lauta.tila.pakka, alku);
    assert!(lauta.tila.jate.is_empty());
}

#[test]
fn sarakkeeseen_kay_vain_vastavari_yhta_pienempana() {
    let mut lauta = tyhja(Vaikeus::Helppo);
    lauta.tila.sarakkeet[0] = sarake(&[k(Maa::Pata, 7)]);
    for (kortti, sallittu) in [
        (k(Maa::Hertta, 6), true),
        (k(Maa::Ruutu, 6), true),
        (k(Maa::Risti, 6), false),
        (k(Maa::Hertta, 5), false),
        (k(Maa::Hertta, 7), false),
    ] {
        lauta.tila.jate = vec![kortti];
        assert_eq!(lauta.voi_siirtaa(JATE, Kohde::Sarake(0)), sallittu);
    }
}

#[test]
fn tyhjaan_sarakkeeseen_kay_vain_kuningas() {
    let mut lauta = tyhja(Vaikeus::Helppo);
    lauta.tila.jate = vec![k(Maa::Hertta, 12)];
    assert!(!lauta.siirra(JATE, Kohde::Sarake(0)));
    lauta.tila.jate = vec![k(Maa::Hertta, KUNINGAS)];
    assert!(lauta.siirra(JATE, Kohde::Sarake(0)));
    assert_eq!(lauta.tila.sarakkeet[0].kortit.len(), 1);
}

#[test]
fn perusta_kasvaa_assasta_ylos_omassa_maassaan() {
    let mut lauta = tyhja(Vaikeus::Helppo);
    lauta.tila.jate = vec![k(Maa::Pata, 2), k(Maa::Pata, 1)];
    let pata = Maa::Pata as usize;
    assert!(!lauta.siirra(JATE, Kohde::Perusta(0)));
    assert!(lauta.siirra(JATE, Kohde::Perusta(pata)));
    assert!(lauta.perustalle(JATE));
    assert_eq!(lauta.tila.perustat[pata].len(), 2);

    lauta.tila.jate = vec![k(Maa::Pata, 4)];
    assert!(!lauta.perustalle(JATE));
}

#[test]
fn jono_siirtyy_kokonaan_ja_alta_kaantyy_kortti() {
    let mut lauta = tyhja(Vaikeus::Helppo);
    lauta.tila.sarakkeet[0] = Sarake {
        kortit: vec![k(Maa::Risti, 2), k(Maa::Hertta, 9), k(Maa::Pata, 8)],
        piilossa: 1,
    };
    lauta.tila.sarakkeet[1] = sarake(&[k(Maa::Pata, 10)]);
    let jono = Lahde::Sarake {
        sarake: 0,
        kortti: 1,
    };
    assert!(lauta.siirra(jono, Kohde::Sarake(1)));
    assert_eq!(lauta.tila.sarakkeet[1].kortit.len(), 3);
    assert_eq!(lauta.tila.sarakkeet[0].kortit.len(), 1);
    assert_eq!(lauta.tila.sarakkeet[0].piilossa, 0);
}

#[test]
fn piilossa_olevaa_tai_rikkinaista_jonoa_ei_voi_siirtaa() {
    let mut lauta = tyhja(Vaikeus::Helppo);
    lauta.tila.sarakkeet[0] = Sarake {
        kortit: vec![k(Maa::Hertta, 9), k(Maa::Pata, 5), k(Maa::Risti, 3)],
        piilossa: 1,
    };
    lauta.tila.sarakkeet[1] = sarake(&[k(Maa::Pata, 10)]);
    for kortti in [0, 1, 9] {
        let lahde = Lahde::Sarake { sarake: 0, kortti };
        assert!(!lauta.siirra(lahde, Kohde::Sarake(1)), "{kortti}");
    }
    let omaan = Lahde::Sarake {
        sarake: 0,
        kortti: 2,
    };
    assert!(!lauta.siirra(omaan, Kohde::Sarake(0)));
}

#[test]
fn nopeampi_voitto_ja_vaikeampi_taso_antavat_enemman() {
    assert!(aikapisteet(Vaikeus::Normaali, 100) > aikapisteet(Vaikeus::Normaali, 101));
    assert!(aikapisteet(Vaikeus::Vaikea, 100) > aikapisteet(Vaikeus::Normaali, 100));
    assert!(aikapisteet(Vaikeus::Normaali, 100) > aikapisteet(Vaikeus::Helppo, 100));
    assert_eq!(aikapisteet(Vaikeus::Vaikea, 0), SUURIN_TULOS);
    // Hidaskin voitto kelpaa listalle.
    for vaikeus in Vaikeus::KAIKKI {
        assert!(aikapisteet(vaikeus, u32::MAX) > 0);
    }
}

#[test]
fn peruutus_palauttaa_tilanteen() {
    let mut lauta = tyhja(Vaikeus::Helppo);
    lauta.tila.jate = vec![k(Maa::Hertta, 1)];
    assert!(lauta.perustalle(JATE));
    assert!(lauta.peruuta());
    assert_eq!(lauta.tila.jate.len(), 1);
    assert!(lauta.tila.perustat[0].is_empty());
    assert!(!lauta.peruuta());
}

#[test]
fn keraa_vie_kaikki_ja_voitto_tunnistetaan_viimeisesta_kortista() {
    let mut lauta = tyhja(Vaikeus::Vaikea);
    lauta.tila.jate.clear();
    for (i, &maa) in Maa::KAIKKI.iter().enumerate() {
        let kortit: Vec<Kortti> = (1..=KUNINGAS).rev().map(|arvo| k(maa, arvo)).collect();
        lauta.tila.sarakkeet[i] = sarake(&kortit);
    }
    assert!(!lauta.on_voitettu());
    assert_eq!(lauta.keraa(), KORTTEJA);
    assert!(lauta.on_voitettu());
    assert_eq!(lauta.keraa(), 0);
}

#[test]
fn laillisten_siirtojen_jalkeen_kortit_sailyvat() {
    for siemen in 0..50 {
        let mut arpa = Arpa::new(siemen);
        for vaikeus in Vaikeus::KAIKKI {
            let mut lauta = Lauta::new(vaikeus, &mut arpa);
            for _ in 0..500 {
                let sarake = arpa.alle(SARAKKEITA);
                let kortti = arpa.alle(13);
                let lahde = match arpa.alle(3) {
                    0 => Lahde::Jate,
                    1 => Lahde::Perusta(arpa.alle(MAITA)),
                    _ => Lahde::Sarake { sarake, kortti },
                };
                let kohde = if arpa.alle(2) == 0 {
                    Kohde::Perusta(arpa.alle(MAITA))
                } else {
                    Kohde::Sarake(arpa.alle(SARAKKEITA))
                };
                match arpa.alle(5) {
                    0 => {
                        lauta.nosta();
                    }
                    1 => {
                        lauta.peruuta();
                    }
                    2 => {
                        lauta.keraa();
                    }
                    _ => {
                        lauta.siirra(lahde, kohde);
                    }
                }
            }
            assert_eq!(kaikki_kortit(&lauta).len(), KORTTEJA);
            let mut erilaiset = kaikki_kortit(&lauta);
            erilaiset.dedup();
            assert_eq!(erilaiset.len(), KORTTEJA);
            for sarake in &lauta.tila.sarakkeet {
                assert!(sarake.piilossa < sarake.kortit.len().max(1));
            }
        }
    }
}

#[test]
fn siirrot_loppuvat_vasta_kun_mikaan_siirto_ei_kay() {
    let mut lauta = tyhja(Vaikeus::Vaikea);
    lauta.tila.kierratykset = 2;
    lauta.tila.jate = vec![k(Maa::Hertta, 5)];
    lauta.tila.sarakkeet[0] = sarake(&[k(Maa::Pata, 9)]);
    assert!(!lauta.on_siirtoja());

    // Jonon osa pääsee toisen sarakkeen päälle.
    lauta.tila.sarakkeet[1] = sarake(&[k(Maa::Hertta, 10)]);
    assert!(lauta.on_siirtoja());

    // Perustan päällimmäisen saa takaisin sarakkeeseen.
    lauta.tila.sarakkeet[1] = Sarake::default();
    lauta.tila.perustat[Maa::Hertta as usize] = (1..=8).map(|a| k(Maa::Hertta, a)).collect();
    assert!(lauta.on_siirtoja());
}

#[test]
fn kuninkaan_siirto_tyhjasta_paikasta_toiseen_ei_ole_siirto() {
    let mut lauta = tyhja(Vaikeus::Helppo);
    lauta.tila.sarakkeet[0] = sarake(&[k(Maa::Pata, KUNINGAS)]);
    assert!(!lauta.on_siirtoja());

    // Piilokortin päältä kuningas kannattaa siirtää.
    lauta.tila.sarakkeet[0] = Sarake {
        kortit: vec![k(Maa::Risti, 2), k(Maa::Pata, KUNINGAS)],
        piilossa: 1,
    };
    assert!(lauta.on_siirtoja());
}

#[test]
fn pakasta_lasketaan_vain_kortit_jotka_noston_tahdissa_saa_esiin() {
    let mut lauta = tyhja(Vaikeus::Normaali);
    lauta.tila.sarakkeet[0] = sarake(&[k(Maa::Pata, 9)]);
    // Sopiva hertta 8 jää kolmen noston keskelle, joten sitä ei koskaan saa päällimmäiseksi.
    let huono = k(Maa::Hertta, 5);
    lauta.tila.pakka = vec![huono, k(Maa::Hertta, 8), huono];
    assert!(!lauta.on_siirtoja());

    lauta.tila.pakka = vec![k(Maa::Hertta, 8), huono, huono];
    assert!(lauta.on_siirtoja());

    // Sama kortti jätepinossa tulee esiin vain, jos pakan saa vielä kääntää.
    let mut vaikea = tyhja(Vaikeus::Vaikea);
    vaikea.tila.sarakkeet[0] = sarake(&[k(Maa::Pata, 9)]);
    vaikea.tila.jate = vec![huono, huono, k(Maa::Hertta, 8), huono];
    assert!(vaikea.on_siirtoja());
    vaikea.tila.kierratykset = 2;
    assert!(!vaikea.on_siirtoja());
}

#[test]
fn vihje_tunnistaa_pakan_ja_jumiutuneen_tilanteen() {
    let mut lauta = tyhja(Vaikeus::Vaikea);
    lauta.tila.kierratykset = 2;
    lauta.tila.jate = vec![k(Maa::Hertta, 5)];
    lauta.tila.sarakkeet[0] = sarake(&[k(Maa::Pata, 9)]);
    assert!(!lauta.on_hyodyllisia_siirtoja());

    lauta.tila.jate = vec![k(Maa::Hertta, 8)];
    assert!(lauta.on_hyodyllisia_siirtoja());

    // Sama kortti pakan pohjalla kelpaa niin kauan, kuin pakka vielä kiertää.
    lauta.tila.jate = vec![k(Maa::Hertta, 5), k(Maa::Hertta, 8), k(Maa::Hertta, 5)];
    assert!(!lauta.on_hyodyllisia_siirtoja());
    lauta.tila.kierratykset = 0;
    assert!(lauta.on_hyodyllisia_siirtoja());
}
