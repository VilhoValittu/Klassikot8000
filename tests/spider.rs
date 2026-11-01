use klassikot8000::Arpa;
use klassikot8000::kortit::{KUNINGAS, Kortti, Maa};
use klassikot8000::pasianssi::{Sarake, Vaikeus};
use klassikot8000::spider::{Jono, KORTTEJA, Lauta, SARAKKEITA, SARJOJA};

fn k(maa: Maa, arvo: u8) -> Kortti {
    Kortti { maa, arvo }
}

fn sarake(kortit: &[Kortti]) -> Sarake {
    Sarake {
        kortit: kortit.to_vec(),
        piilossa: 0,
    }
}

/// Lauta ilman kortteja: jokainen testi rakentaa oman tilanteensa.
fn tyhja() -> Lauta {
    let mut lauta = Lauta::new(Vaikeus::Helppo, &mut Arpa::new(1));
    lauta.tila.pakka.clear();
    lauta.tila.sarakkeet = Default::default();
    lauta
}

fn pinon_maara(lauta: &Lauta) -> usize {
    lauta.tila.pakka.len()
        + lauta
            .tila
            .sarakkeet
            .iter()
            .map(|s| s.kortit.len())
            .sum::<usize>()
        + lauta.tila.valmiit.len() * KUNINGAS as usize
}

/// Sarja kuninkaasta alaspäin arvoon `loppu` asti.
fn sarja(maa: Maa, loppu: u8) -> Vec<Kortti> {
    (loppu..=KUNINGAS).rev().map(|arvo| k(maa, arvo)).collect()
}

#[test]
fn jako_antaa_kortit_tasoittain_ja_vain_paallimmainen_nakyy() {
    for (vaikeus, maita) in [
        (Vaikeus::Helppo, 1),
        (Vaikeus::Normaali, 2),
        (Vaikeus::Vaikea, 4),
    ] {
        for siemen in 0..50 {
            let lauta = Lauta::new(vaikeus, &mut Arpa::new(siemen));
            assert_eq!(pinon_maara(&lauta), KORTTEJA);
            assert_eq!(lauta.tila.pakka.len(), 50);
            assert_eq!(lauta.jakoja(), 5);
            let mut kaikki: Vec<Kortti> = lauta.tila.pakka.clone();
            for (i, tiedot) in lauta.tila.sarakkeet.iter().enumerate() {
                assert_eq!(tiedot.kortit.len(), if i < 4 { 6 } else { 5 });
                assert_eq!(tiedot.piilossa, tiedot.kortit.len() - 1);
                kaikki.extend(&tiedot.kortit);
            }
            for maa in Maa::KAIKKI {
                for arvo in 1..=KUNINGAS {
                    let maara = kaikki.iter().filter(|&&x| x == k(maa, arvo)).count();
                    assert!(maara == 0 || maara == SARJOJA / maita, "{maa:?} {arvo}");
                }
            }
        }
    }
}

#[test]
fn siirto_kay_maasta_riippumatta_mutta_jono_vaatii_saman_maan() {
    let mut lauta = tyhja();
    lauta.tila.sarakkeet[0] = sarake(&[k(Maa::Hertta, 7)]);
    lauta.tila.sarakkeet[1] = sarake(&[k(Maa::Pata, 6), k(Maa::Pata, 5)]);
    lauta.tila.sarakkeet[2] = sarake(&[k(Maa::Hertta, 6), k(Maa::Pata, 5)]);

    assert!(
        lauta
            .siirrettavat(Jono {
                sarake: 1,
                kortti: 0
            })
            .is_some()
    );
    assert!(
        lauta
            .siirrettavat(Jono {
                sarake: 2,
                kortti: 0
            })
            .is_none()
    );
    assert!(
        lauta
            .siirrettavat(Jono {
                sarake: 2,
                kortti: 1
            })
            .is_some()
    );
    assert!(!lauta.siirra(
        Jono {
            sarake: 2,
            kortti: 0
        },
        0
    ));
    assert!(lauta.siirra(
        Jono {
            sarake: 1,
            kortti: 0
        },
        0
    ));
    assert_eq!(lauta.tila.sarakkeet[0].kortit.len(), 3);
    assert!(lauta.tila.sarakkeet[1].kortit.is_empty());
    // Tyhjään sarakkeeseen käy mikä tahansa kortti.
    assert!(lauta.siirra(
        Jono {
            sarake: 2,
            kortti: 1
        },
        1
    ));
    // Saman sarakkeen sisällä ei voi siirtää.
    assert!(!lauta.voi_siirtaa(
        Jono {
            sarake: 0,
            kortti: 1
        },
        0
    ));
}

#[test]
fn nurinpain_oleva_kortti_ei_liiku_ja_paljastuu_siirrossa() {
    let mut lauta = tyhja();
    lauta.tila.sarakkeet[0] = Sarake {
        kortit: vec![k(Maa::Pata, 2), k(Maa::Pata, 9)],
        piilossa: 1,
    };
    lauta.tila.sarakkeet[1] = sarake(&[k(Maa::Hertta, 10)]);
    assert!(
        lauta
            .siirrettavat(Jono {
                sarake: 0,
                kortti: 0
            })
            .is_none()
    );
    assert!(lauta.siirra(
        Jono {
            sarake: 0,
            kortti: 1
        },
        1
    ));
    assert_eq!(lauta.tila.sarakkeet[0].piilossa, 0);
}

#[test]
fn valmis_sarja_poistuu_paljastaa_kortin_ja_voittaa() {
    let mut lauta = tyhja();
    lauta.tila.valmiit = vec![Maa::Pata; SARJOJA - 1];
    lauta.tila.sarakkeet[0] = Sarake {
        kortit: [vec![k(Maa::Risti, 4)], sarja(Maa::Pata, 2)].concat(),
        piilossa: 1,
    };
    lauta.tila.sarakkeet[1] = sarake(&[k(Maa::Pata, 1)]);
    assert!(lauta.siirra(
        Jono {
            sarake: 1,
            kortti: 0
        },
        0
    ));
    assert_eq!(lauta.tila.valmiit.len(), SARJOJA);
    assert_eq!(lauta.tila.sarakkeet[0].kortit, [k(Maa::Risti, 4)]);
    assert_eq!(lauta.tila.sarakkeet[0].piilossa, 0);
    assert!(lauta.on_voitettu());
}

#[test]
fn eri_maiden_sarja_ei_poistu() {
    let mut lauta = tyhja();
    let mut kortit = sarja(Maa::Pata, 2);
    kortit[5].maa = Maa::Hertta;
    lauta.tila.sarakkeet[0] = sarake(&kortit);
    lauta.tila.sarakkeet[1] = sarake(&[k(Maa::Pata, 1)]);
    assert!(lauta.siirra(
        Jono {
            sarake: 1,
            kortti: 0
        },
        0
    ));
    assert!(lauta.tila.valmiit.is_empty());
    assert_eq!(lauta.tila.sarakkeet[0].kortit.len(), 13);
}

#[test]
fn jako_antaa_kortin_joka_sarakkeeseen_ja_vaatii_taydet_sarakkeet() {
    let mut lauta = Lauta::new(Vaikeus::Helppo, &mut Arpa::new(3));
    let ennen: Vec<usize> = lauta
        .tila
        .sarakkeet
        .iter()
        .map(|s| s.kortit.len())
        .collect();
    assert!(lauta.jaa());
    assert_eq!(lauta.tila.pakka.len(), 40);
    for (i, tiedot) in lauta.tila.sarakkeet.iter().enumerate() {
        assert_eq!(tiedot.kortit.len(), ennen[i] + 1);
    }

    lauta.tila.sarakkeet[4].kortit.clear();
    lauta.tila.sarakkeet[4].piilossa = 0;
    assert!(!lauta.voi_jakaa());
    assert!(!lauta.jaa());
    assert_eq!(lauta.tila.pakka.len(), 40);
}

#[test]
fn jako_voi_valmistaa_sarjan() {
    let mut lauta = tyhja();
    lauta.tila.sarakkeet[0] = sarake(&sarja(Maa::Pata, 2));
    for sarake_i in 1..SARAKKEITA {
        lauta.tila.sarakkeet[sarake_i] = sarake(&[k(Maa::Hertta, 9)]);
    }
    lauta.tila.pakka = (0..SARAKKEITA)
        .map(|i| k(Maa::Pata, if i == SARAKKEITA - 1 { 1 } else { 5 }))
        .collect();
    assert!(lauta.jaa());
    assert_eq!(lauta.tila.valmiit, [Maa::Pata]);
    assert!(lauta.tila.sarakkeet[0].kortit.is_empty());
}

#[test]
fn liian_vahat_kortit_eivat_lukitse_jakoa() {
    let mut lauta = tyhja();
    lauta.tila.sarakkeet[0] = sarake(&[k(Maa::Pata, 9)]);
    lauta.tila.pakka = vec![k(Maa::Pata, 3); SARAKKEITA];
    assert!(lauta.voi_jakaa());
    assert!(lauta.on_siirtoja());
}

#[test]
fn siirrot_loppuvat_kun_pakka_on_tyhja_eika_mikaan_kay() {
    let mut lauta = tyhja();
    for i in 0..SARAKKEITA {
        lauta.tila.sarakkeet[i] = sarake(&[k(Maa::Pata, 1), k(Maa::Hertta, 5 + (i % 2) as u8 * 2)]);
    }
    assert!(!lauta.on_siirtoja());
    assert!(!lauta.on_hyodyllisia_siirtoja());

    lauta.tila.pakka = vec![k(Maa::Pata, 3); SARAKKEITA];
    assert!(lauta.on_siirtoja());

    // Yksinäisen sarakkeen siirto tyhjästä toiseen ei muuta tilannetta.
    let mut lauta = tyhja();
    lauta.tila.sarakkeet[0] = sarake(&[k(Maa::Pata, 9)]);
    assert!(!lauta.on_siirtoja());
}

#[test]
fn vihje_tunnistaa_turhat_siirrot() {
    // Sama siirto edestakaisin on laillinen muttei hyödyllinen.
    let mut lauta = tyhja();
    lauta.tila.sarakkeet[0] = sarake(&[k(Maa::Pata, 1), k(Maa::Hertta, 6), k(Maa::Pata, 5)]);
    lauta.tila.sarakkeet[1] = sarake(&[k(Maa::Pata, 1), k(Maa::Risti, 6)]);
    for i in 2..SARAKKEITA {
        lauta.tila.sarakkeet[i] = sarake(&[k(Maa::Pata, 1), k(Maa::Hertta, 13)]);
    }
    assert!(lauta.on_siirtoja());
    assert!(!lauta.on_hyodyllisia_siirtoja());

    // Saman maan päälle yhdistäminen on hyödyllistä.
    lauta.tila.sarakkeet[1] = sarake(&[k(Maa::Pata, 1), k(Maa::Pata, 6)]);
    assert!(lauta.on_hyodyllisia_siirtoja());
}

#[test]
fn peruutus_palauttaa_jaon_ja_valmiin_sarjan() {
    let mut lauta = Lauta::new(Vaikeus::Vaikea, &mut Arpa::new(5));
    let alku = lauta.tila.clone();
    assert!(!lauta.peruuta());
    assert!(lauta.jaa());
    assert!(lauta.peruuta());
    assert_eq!(lauta.tila, alku);

    let mut lauta = tyhja();
    lauta.tila.sarakkeet[0] = sarake(&sarja(Maa::Pata, 2));
    lauta.tila.sarakkeet[1] = sarake(&[k(Maa::Pata, 1)]);
    let ennen = lauta.tila.clone();
    assert!(lauta.siirra(
        Jono {
            sarake: 1,
            kortti: 0
        },
        0
    ));
    assert_eq!(lauta.tila.valmiit.len(), 1);
    assert!(lauta.peruuta());
    assert_eq!(lauta.tila, ennen);
}

#[test]
fn kaikki_kortit_sailyvat_satunnaisessa_pelissa() {
    for siemen in 0..30 {
        let mut arpa = Arpa::new(siemen);
        let mut lauta = Lauta::new(Vaikeus::Normaali, &mut arpa);
        for _ in 0..400 {
            let sarake = arpa.alle(SARAKKEITA);
            let pituus = lauta.tila.sarakkeet[sarake].kortit.len();
            let kortti = pituus.saturating_sub(1 + arpa.alle(3));
            if !lauta.siirra(Jono { sarake, kortti }, arpa.alle(SARAKKEITA)) && arpa.alle(8) == 0 {
                lauta.jaa();
            }
            assert_eq!(pinon_maara(&lauta), KORTTEJA);
        }
    }
}
