use klassikot8000::Arpa;
use klassikot8000::tiilirikko::{
    ASKEL_MS, ELAMIA, KENTTIA, KORKEUS, LEVEA_MAILA_L, LEVEYS, MAILA_L, MAILA_Y, PALLOJA_ENINTAAN,
    PALLON_SADE, Pallo, RIVEJA, SARAKKEITA, SUPERVAUHTI, SUURIN_PISTEMAARA, TIILET_Y, TIILI_K,
    TIILI_L, Tapahtuma, Tiilipeli, Tila,
};

fn uusi_peli(kentta: usize) -> Tiilipeli {
    Tiilipeli::new(kentta, &mut Arpa::new(1))
}

/// Peli, jossa on vain yksi tavallinen tiili kaukana pallosta: kenttä ei vaihdu kesken testin.
fn lahes_tyhja() -> Tiilipeli {
    let mut peli = uusi_peli(0);
    peli.tiilet = [[0; SARAKKEITA]; RIVEJA];
    peli.pallotiilet = [[false; SARAKKEITA]; RIVEJA];
    peli.supertiilet = [[false; SARAKKEITA]; RIVEJA];
    peli.tiilet[0][0] = 1;
    peli.tila = Tila::Lennossa;
    peli
}

/// Kutsuu pelin `laukaise`-metodia, jotta sisäinen vauhti on asetettu.
fn lennossa_tyhjalla() -> Tiilipeli {
    let mut peli = lahes_tyhja();
    peli.tila = Tila::Kiinni;
    peli.laukaise(&mut Arpa::new(1));
    peli
}

#[test]
fn jokaisessa_kentassa_on_tiilia_ja_ylisuuri_numero_rajataan() {
    for kentta in 0..KENTTIA {
        let peli = uusi_peli(kentta);
        assert!(peli.tiilia_jaljella() > 0, "{kentta}");
        assert_eq!(peli.tila, Tila::Kiinni);
        assert_eq!(peli.elamat, ELAMIA);
    }
    assert_eq!(uusi_peli(99).kentta, KENTTIA - 1);
}

#[test]
fn maila_pysyy_kentalla_ja_kiinni_oleva_pallo_seuraa() {
    let mut peli = uusi_peli(0);
    peli.aseta_maila(-100.0);
    assert_eq!(peli.maila_x, MAILA_L / 2.0);
    peli.aseta_maila(10_000.0);
    assert_eq!(peli.maila_x, LEVEYS - MAILA_L / 2.0);
    assert_eq!(peli.pallot[0].paikka.0, peli.maila_x);

    let alku = peli.maila_x;
    peli.askel(-1, &mut Arpa::new(1));
    assert!(peli.maila_x < alku);
    assert_eq!(peli.pallot[0].paikka.0, peli.maila_x);
}

#[test]
fn laukaisu_lahtee_ylos_vinoon_eika_toistu_lennossa() {
    for siemen in 0..100 {
        let mut arpa = Arpa::new(siemen);
        let mut peli = uusi_peli(0);
        peli.laukaise(&mut arpa);
        assert_eq!(peli.tila, Tila::Lennossa);
        assert!(peli.pallot[0].nopeus.1 < 0.0);
        assert!(peli.pallot[0].nopeus.0 != 0.0);
        let nopeus = peli.pallot[0].nopeus;
        peli.laukaise(&mut arpa);
        assert_eq!(peli.pallot[0].nopeus, nopeus);
    }
}

#[test]
fn seinat_kaantavat_pallon() {
    let mut peli = lennossa_tyhjalla();
    peli.pallot[0].paikka = (PALLON_SADE + 0.1, 300.0);
    peli.pallot[0].nopeus = (-300.0, -100.0);
    assert_eq!(peli.askel(0, &mut Arpa::new(1)), [Tapahtuma::Seina]);
    assert!(peli.pallot[0].nopeus.0 > 0.0);

    peli.pallot[0].paikka = (LEVEYS - PALLON_SADE - 0.1, 300.0);
    peli.pallot[0].nopeus = (300.0, -100.0);
    assert_eq!(peli.askel(0, &mut Arpa::new(1)), [Tapahtuma::Seina]);
    assert!(peli.pallot[0].nopeus.0 < 0.0);

    peli.pallot[0].paikka = (300.0, PALLON_SADE + 0.1);
    peli.pallot[0].nopeus = (100.0, -300.0);
    assert_eq!(peli.askel(0, &mut Arpa::new(1)), [Tapahtuma::Seina]);
    assert!(peli.pallot[0].nopeus.1 > 0.0);
}

#[test]
fn mailan_laita_ohjaa_palloa_ja_vauhti_sailyy() {
    for (poikkeama, suunta) in [(-40.0, -1.0), (40.0, 1.0)] {
        let mut peli = lennossa_tyhjalla();
        let vauhti = peli.pallot[0].nopeus.0.hypot(peli.pallot[0].nopeus.1);
        peli.aseta_maila(300.0);
        peli.pallot[0].paikka = (300.0 + poikkeama, MAILA_Y - PALLON_SADE - 0.1);
        peli.pallot[0].nopeus = (0.0, vauhti);
        assert_eq!(peli.askel(0, &mut Arpa::new(1)), [Tapahtuma::Maila]);
        assert!(peli.pallot[0].nopeus.1 < 0.0);
        assert!(peli.pallot[0].nopeus.0 * suunta > 0.0);
        assert!((peli.pallot[0].nopeus.0.hypot(peli.pallot[0].nopeus.1) - vauhti).abs() < 0.01);
    }
}

#[test]
fn nouseva_pallo_kulkee_mailan_lapi() {
    let mut peli = lennossa_tyhjalla();
    peli.aseta_maila(300.0);
    peli.pallot[0].paikka = (300.0, MAILA_Y + 5.0);
    peli.pallot[0].nopeus = (0.0, -300.0);
    assert!(peli.askel(0, &mut Arpa::new(1)).is_empty());
    assert!(peli.pallot[0].nopeus.1 < 0.0);
}

#[test]
fn kestava_tiili_vaatii_useamman_osuman_ja_kaantaa_pallon() {
    let mut peli = lennossa_tyhjalla();
    peli.tiilet[3][5] = 2;
    let y = TIILET_Y + 4.0 * TIILI_K + PALLON_SADE + 0.5;
    let x = 5.5 * TIILI_L;
    let lauo = |peli: &mut Tiilipeli| {
        peli.pallot[0].paikka = (x, y);
        peli.pallot[0].nopeus = (0.0, -300.0);
        peli.askel(0, &mut Arpa::new(1))
    };
    assert_eq!(
        lauo(&mut peli),
        [Tapahtuma::Tiili {
            rivi: 3,
            sarake: 5,
            rikki: false
        }]
    );
    assert!(peli.pallot[0].nopeus.1 > 0.0);
    assert_eq!(peli.pisteet, 10);
    assert_eq!(
        lauo(&mut peli),
        [Tapahtuma::Tiili {
            rivi: 3,
            sarake: 5,
            rikki: true
        }]
    );
    assert_eq!(peli.tiilet[3][5], 0);
    assert_eq!(peli.pisteet, 20);
}

#[test]
fn sivusta_tuleva_osuma_kaantaa_vaakasuunnan_ja_vauhti_kasvaa() {
    let mut peli = lennossa_tyhjalla();
    peli.tiilet[3][5] = 1;
    let vauhti = peli.pallot[0].nopeus.0.hypot(peli.pallot[0].nopeus.1);
    peli.pallot[0].paikka = (5.0 * TIILI_L - PALLON_SADE - 0.5, TIILET_Y + 3.5 * TIILI_K);
    let suunta = (vauhti * 0.8, vauhti * 0.6);
    peli.pallot[0].nopeus = suunta;
    peli.askel(0, &mut Arpa::new(1));
    assert!(peli.pallot[0].nopeus.0 < 0.0);
    assert!(peli.pallot[0].nopeus.1 > 0.0);
    assert!(peli.pallot[0].nopeus.0.hypot(peli.pallot[0].nopeus.1) > vauhti);
}

#[test]
fn pallon_menetys_kuluttaa_elaman_ja_viimeinen_paattaa_pelin() {
    let mut peli = lennossa_tyhjalla();
    let pudota = |peli: &mut Tiilipeli| {
        peli.pallot[0].paikka = (LEVEYS / 2.0, KORKEUS + PALLON_SADE + 0.1);
        peli.pallot[0].nopeus = (0.0, 300.0);
        peli.askel(0, &mut Arpa::new(1))
    };
    assert_eq!(pudota(&mut peli), [Tapahtuma::Menetys]);
    assert_eq!(peli.tila, Tila::Kiinni);
    assert_eq!(peli.elamat, ELAMIA - 1);

    peli.tila = Tila::Lennossa;
    assert_eq!(pudota(&mut peli), [Tapahtuma::Menetys]);
    peli.tila = Tila::Lennossa;
    assert_eq!(pudota(&mut peli), [Tapahtuma::Ohi]);
    assert_eq!(peli.tila, Tila::Ohi);
    assert_eq!(peli.elamat, 0);

    // päättynyt peli ei enää etene
    assert!(peli.askel(1, &mut Arpa::new(1)).is_empty());
}

#[test]
fn viimeinen_tiili_avaa_seuraavan_kentan_ja_viimeisessa_kentassa_voittaa() {
    let mut peli = uusi_peli(KENTTIA - 2);
    peli.tiilet = [[0; SARAKKEITA]; RIVEJA];
    peli.tiilet[0][0] = 1;
    peli.laukaise(&mut Arpa::new(1));
    let viimeistele = |peli: &mut Tiilipeli| {
        peli.tiilet = [[0; SARAKKEITA]; RIVEJA];
        peli.tiilet[0][0] = 1;
        peli.pallot[0].paikka = (TIILI_L / 2.0, TIILET_Y + TIILI_K + PALLON_SADE + 0.5);
        peli.pallot[0].nopeus = (0.0, -300.0);
        peli.askel(0, &mut Arpa::new(1))
    };
    let tapahtumat = viimeistele(&mut peli);
    assert!(matches!(
        tapahtumat.last(),
        Some(Tapahtuma::KenttaLapi { .. })
    ));
    assert_eq!(peli.kentta, KENTTIA - 1);
    assert_eq!(peli.tila, Tila::Kiinni);
    assert!(peli.tiilia_jaljella() > 0);

    peli.laukaise(&mut Arpa::new(1));
    let tapahtumat = viimeistele(&mut peli);
    assert!(matches!(tapahtumat.last(), Some(Tapahtuma::Voitto { .. })));
    assert_eq!(peli.tila, Tila::Voitto);
}

/// Rikkoo kentän ainoan tiilen, kun kentällä on kulunut `kulunut_ms`. Palauttaa bonuksen ja pisteiden kasvun.
fn lapaisy(kentta: usize, kulunut_ms: u32) -> (u32, u32) {
    let mut peli = uusi_peli(kentta);
    peli.tiilet = [[0; SARAKKEITA]; RIVEJA];
    peli.tiilet[0][0] = 1;
    peli.laukaise(&mut Arpa::new(1));
    // Osuman askel kuluttaa vielä oman aikansa.
    peli.kentan_ms = kulunut_ms.saturating_sub(ASKEL_MS);
    peli.pallot[0].paikka = (TIILI_L / 2.0, TIILET_Y + TIILI_K + PALLON_SADE + 0.5);
    peli.pallot[0].nopeus = (0.0, -300.0);
    let tapahtumat = peli.askel(0, &mut Arpa::new(1));
    let bonus = match tapahtumat.last() {
        Some(Tapahtuma::KenttaLapi { bonus } | Tapahtuma::Voitto { bonus }) => *bonus,
        muu => panic!("{muu:?}"),
    };
    (bonus, peli.pisteet)
}

#[test]
fn nopeampi_lapaisy_antaa_enemman_pisteita() {
    let (alussa, pisteet) = lapaisy(0, 0);
    assert!((990..1000).contains(&alussa), "{alussa}");
    assert_eq!(pisteet, 10 + alussa);
    assert_eq!(lapaisy(0, 120_000).0, 500);
    assert_eq!(lapaisy(0, 240_000).0, 0);
    assert_eq!(lapaisy(0, 999_000).0, 0);
    assert!(lapaisy(0, 60_000).1 > lapaisy(0, 180_000).1);
    // Kerroin kasvaa kymmenen kentän välein, joten viimeinen kenttä on viisinkertainen.
    assert_eq!(lapaisy(9, 120_000).0, 500);
    assert_eq!(lapaisy(10, 120_000).0, 1000);
    assert_eq!(lapaisy(KENTTIA - 1, 120_000).0, 2500);
}

#[test]
fn kentan_kello_kay_vain_lennossa_ja_nollautuu_kentan_vaihtuessa() {
    let mut peli = uusi_peli(0);
    peli.askel(0, &mut Arpa::new(1));
    assert_eq!(peli.kentan_ms, 0);
    peli.laukaise(&mut Arpa::new(1));
    peli.askel(0, &mut Arpa::new(1));
    assert_eq!(peli.kentan_ms, ASKEL_MS);

    peli.tiilet = [[0; SARAKKEITA]; RIVEJA];
    peli.tiilet[0][0] = 1;
    peli.pallot[0].paikka = (TIILI_L / 2.0, TIILET_Y + TIILI_K + PALLON_SADE + 0.5);
    peli.pallot[0].nopeus = (0.0, -300.0);
    peli.askel(0, &mut Arpa::new(1));
    assert_eq!(peli.kentta, 1);
    assert_eq!(peli.kentan_ms, 0);
}

/// Pallotiili riville 0 ja ylimääräinen tiili, joka pitää kentän kesken.
fn pallotiilella() -> Tiilipeli {
    let mut peli = lennossa_tyhjalla();
    peli.tiilet[7][10] = 1;
    peli.pallotiilet[0][0] = true;
    peli
}

fn riko_ensimmainen(peli: &mut Tiilipeli) -> Vec<Tapahtuma> {
    peli.pallot[0].paikka = (TIILI_L / 2.0, TIILET_Y + TIILI_K + PALLON_SADE + 0.5);
    peli.pallot[0].nopeus = (100.0, -300.0);
    peli.askel(0, &mut Arpa::new(1))
}

fn pudota(peli: &mut Tiilipeli, i: usize) -> Vec<Tapahtuma> {
    peli.pallot[i].paikka = (LEVEYS / 2.0, KORKEUS + PALLON_SADE + 0.1);
    peli.pallot[i].nopeus = (0.0, 300.0);
    peli.askel(0, &mut Arpa::new(1))
}

/// Kentässä on useita pallotiiliä ja yksi supertiili. Jokainen on ehjä yhden osuman tiili,
/// eivätkä ne kosketa toisiaan edes kulmittain.
fn tarkista_pallotiilet(peli: &Tiilipeli) {
    let mut palkinnot = Vec::new();
    let mut supertiilia = 0;
    for rivi in 0..RIVEJA {
        for sarake in 0..SARAKKEITA {
            let pallotiili = peli.pallotiilet[rivi][sarake];
            let supertiili = peli.supertiilet[rivi][sarake];
            assert!(!(pallotiili && supertiili), "kenttä {}", peli.kentta);
            if pallotiili || supertiili {
                palkinnot.push((rivi, sarake));
                assert_eq!(peli.tiilet[rivi][sarake], 1, "kenttä {}", peli.kentta);
            }
            supertiilia += supertiili as usize;
        }
    }
    assert_eq!(supertiilia, 1, "kenttä {}", peli.kentta);
    let maara = palkinnot.len() - supertiilia;
    assert!((3..=4).contains(&maara), "kenttä {}: {maara}", peli.kentta);
    for (i, a) in palkinnot.iter().enumerate() {
        for b in &palkinnot[i + 1..] {
            assert!(
                a.0.abs_diff(b.0) > 1 || a.1.abs_diff(b.1) > 1,
                "kenttä {}: {a:?} ja {b:?} vierekkäin",
                peli.kentta
            );
        }
    }
}

#[test]
fn kenttia_on_viisikymmenta_ja_jokainen_on_erilainen() {
    assert_eq!(KENTTIA, 50);
    let kuviot: Vec<_> = (0..KENTTIA)
        .map(|kentta| {
            let mut peli = uusi_peli(kentta);
            // Palkintotiilet ovat arvottuja ja kestävät yhden osuman, joten ne eivät kuulu vertailuun.
            for rivi in 0..RIVEJA {
                for sarake in 0..SARAKKEITA {
                    if peli.tiilet[rivi][sarake] > 0 {
                        peli.tiilet[rivi][sarake] = 1;
                    }
                }
            }
            peli.tiilet
        })
        .collect();
    for (i, kuvio) in kuviot.iter().enumerate() {
        for (j, toinen) in kuviot.iter().enumerate().skip(i + 1) {
            assert_ne!(kuvio, toinen, "kentät {i} ja {j}");
        }
    }
}

#[test]
fn jokaisessa_kentassa_on_useita_yhden_osuman_pallotiilia() {
    for siemen in 0..200 {
        let mut arpa = Arpa::new(siemen);
        for kentta in 0..KENTTIA {
            tarkista_pallotiilet(&Tiilipeli::new(kentta, &mut arpa));
        }
    }
}

#[test]
fn pallotiilet_arvotaan_joka_latauksella() {
    let mut arpa = Arpa::new(1);
    for kentta in 0..KENTTIA {
        let eka = Tiilipeli::new(kentta, &mut arpa).pallotiilet;
        assert!((0..20).any(|_| Tiilipeli::new(kentta, &mut arpa).pallotiilet != eka));
    }
}

#[test]
fn kentan_vaihtuessa_uudet_pallotiilet_arvotaan_annetulla_arvalla() {
    let lapaise = |siemen| {
        let mut peli = lennossa_tyhjalla();
        peli.pallot[0].paikka = (TIILI_L / 2.0, TIILET_Y + TIILI_K + PALLON_SADE + 0.5);
        peli.pallot[0].nopeus = (0.0, -300.0);
        peli.askel(0, &mut Arpa::new(siemen));
        assert_eq!(peli.kentta, 1);
        tarkista_pallotiilet(&peli);
        peli.pallotiilet
    };
    let eka = lapaise(0);
    assert!((1..20).any(|siemen| lapaise(siemen) != eka));
}

#[test]
fn pallotiili_tuo_toisen_pallon_ja_leventaa_mailan() {
    let mut peli = pallotiilella();
    assert_eq!(peli.maila_l(), MAILA_L);
    let vauhti = |p: (f32, f32)| p.0.hypot(p.1);
    let alkuvauhti = vauhti(peli.pallot[0].nopeus);
    assert_eq!(
        riko_ensimmainen(&mut peli),
        [
            Tapahtuma::Tiili {
                rivi: 0,
                sarake: 0,
                rikki: true
            },
            Tapahtuma::UusiPallo
        ]
    );
    assert_eq!(peli.pallot.len(), 2);
    assert_eq!(LEVEA_MAILA_L, MAILA_L * 1.5);
    assert_eq!(peli.maila_l(), LEVEA_MAILA_L);

    // Uusi pallo lähtee tiilen kohdalta alas eri suuntaan kuin osunut, kentän perusvauhdilla.
    let [vanha, uusi] = peli.pallot[..] else {
        panic!("palloja pitää olla kaksi");
    };
    assert_eq!(uusi.paikka, (TIILI_L / 2.0, TIILET_Y + TIILI_K / 2.0));
    assert!(uusi.nopeus.1 > 0.0);
    assert!(uusi.nopeus.0 * vanha.nopeus.0 < 0.0);
    assert!((vauhti(uusi.nopeus) - alkuvauhti).abs() < 0.01);
    assert_eq!(uusi.vauhti, alkuvauhti);
    assert_eq!(vanha.vauhti, alkuvauhti + 4.0);
}

#[test]
fn kahdella_pallolla_tiiliosuma_antaa_tuplapisteet() {
    let mut peli = pallotiilella();
    // Toisen pallon tuova osuma lasketaan vielä yhdellä pallolla.
    riko_ensimmainen(&mut peli);
    assert_eq!(peli.pisteet, 10);

    let osu = |peli: &mut Tiilipeli| {
        peli.tiilet[3][5] = 1;
        peli.pallot[0].paikka = (5.5 * TIILI_L, TIILET_Y + 4.0 * TIILI_K + PALLON_SADE + 0.5);
        peli.pallot[0].nopeus = (0.0, -300.0);
        if let Some(toinen) = peli.pallot.get_mut(1) {
            toinen.paikka = (100.0, 300.0);
        }
        peli.askel(0, &mut Arpa::new(1));
    };
    osu(&mut peli);
    assert_eq!(peli.pisteet, 30);

    // Kerroin päättyy, kun toinen pallo putoaa.
    assert_eq!(pudota(&mut peli, 1), [Tapahtuma::ToinenMeni]);
    osu(&mut peli);
    assert_eq!(peli.pisteet, 40);
}

#[test]
fn tuplapisteet_kasvavat_kentan_kertoimella_mutta_aikabonus_ei_tuplaannu() {
    let mut peli = pallotiilella();
    riko_ensimmainen(&mut peli);
    let ennen = peli.pisteet;

    peli.tiilet = [[0; SARAKKEITA]; RIVEJA];
    peli.tiilet[0][0] = 1;
    let tapahtumat = riko_ensimmainen(&mut peli);
    let Some(Tapahtuma::KenttaLapi { bonus }) = tapahtumat.last().copied() else {
        panic!("{tapahtumat:?}");
    };
    assert_eq!(peli.pisteet, ennen + 20 + bonus);

    // Kentästä 11 alkaen osuma on valmiiksi kaksinkertainen, ja kaksi palloa tuplaa sen.
    let mut peli = Tiilipeli::new(10, &mut Arpa::new(1));
    peli.laukaise(&mut Arpa::new(1));
    peli.tiilet = [[0; SARAKKEITA]; RIVEJA];
    peli.pallotiilet = [[false; SARAKKEITA]; RIVEJA];
    peli.supertiilet = [[false; SARAKKEITA]; RIVEJA];
    peli.tiilet[0][0] = 1;
    peli.tiilet[7][10] = 1;
    peli.pallotiilet[0][0] = true;
    riko_ensimmainen(&mut peli);
    assert_eq!(peli.pisteet, 20);
    peli.tiilet[3][5] = 1;
    peli.pallot[1].paikka = (100.0, 300.0);
    peli.pallot[0].paikka = (5.5 * TIILI_L, TIILET_Y + 4.0 * TIILI_K + PALLON_SADE + 0.5);
    peli.pallot[0].nopeus = (0.0, -300.0);
    peli.askel(0, &mut Arpa::new(1));
    assert_eq!(peli.pisteet, 60);
}

#[test]
fn tavallinen_ja_kestava_tiili_eivat_tuo_palloa() {
    let mut peli = pallotiilella();
    peli.pallotiilet[0][0] = false;
    riko_ensimmainen(&mut peli);
    assert_eq!(peli.pallot.len(), 1);

    let mut peli = pallotiilella();
    peli.tiilet[0][0] = 2;
    riko_ensimmainen(&mut peli);
    assert_eq!(peli.pallot.len(), 1);
}

#[test]
fn palloja_on_enintaan_kaksi() {
    let mut peli = pallotiilella();
    riko_ensimmainen(&mut peli);
    peli.tiilet[0][0] = 1;
    let tapahtumat = riko_ensimmainen(&mut peli);
    assert!(!tapahtumat.contains(&Tapahtuma::UusiPallo));
    assert_eq!(peli.pallot.len(), PALLOJA_ENINTAAN);
    assert_eq!(peli.tiilet[0][0], 0);
}

#[test]
fn levennyt_maila_pysyy_kentalla_ja_saa_kiinni_kapeaa_kauempaa() {
    let mut peli = pallotiilella();
    peli.aseta_maila(0.0);
    riko_ensimmainen(&mut peli);
    assert_eq!(peli.maila_x, LEVEA_MAILA_L / 2.0);
    peli.aseta_maila(-100.0);
    assert_eq!(peli.maila_x, LEVEA_MAILA_L / 2.0);
    peli.aseta_maila(10_000.0);
    assert_eq!(peli.maila_x, LEVEYS - LEVEA_MAILA_L / 2.0);

    // Kohta on kapean mailan ulottumattomissa mutta leveän päällä.
    peli.aseta_maila(300.0);
    peli.pallot[0].paikka = (
        300.0 + MAILA_L / 2.0 + PALLON_SADE + 10.0,
        MAILA_Y - PALLON_SADE,
    );
    peli.pallot[0].nopeus = (0.0, 300.0);
    peli.pallot[1].paikka = (100.0, 300.0);
    assert!(peli.askel(0, &mut Arpa::new(1)).contains(&Tapahtuma::Maila));
    assert!(peli.pallot[0].nopeus.1 < 0.0);
}

#[test]
fn toisen_pallon_menetys_kaventaa_mailan_eika_vie_elamaa() {
    for pudotettava in 0..2 {
        let mut peli = pallotiilella();
        riko_ensimmainen(&mut peli);
        let jaava = peli.pallot[1 - pudotettava].nopeus;

        assert_eq!(pudota(&mut peli, pudotettava), [Tapahtuma::ToinenMeni]);
        assert_eq!(peli.elamat, ELAMIA);
        assert_eq!(peli.tila, Tila::Lennossa);
        assert_eq!(peli.pallot.len(), 1);
        assert_eq!(peli.pallot[0].nopeus, jaava);
        assert_eq!(peli.maila_l(), MAILA_L);

        assert_eq!(pudota(&mut peli, 0), [Tapahtuma::Menetys]);
        assert_eq!(peli.elamat, ELAMIA - 1);
        assert_eq!(peli.tila, Tila::Kiinni);
    }
}

#[test]
fn kaksi_palloa_samalla_askeleella_vie_yhden_elaman() {
    let mut peli = pallotiilella();
    riko_ensimmainen(&mut peli);
    peli.pallot[1].paikka = (100.0, KORKEUS + PALLON_SADE + 0.1);
    peli.pallot[1].nopeus = (0.0, 300.0);
    assert_eq!(
        pudota(&mut peli, 0),
        [Tapahtuma::ToinenMeni, Tapahtuma::Menetys]
    );
    assert_eq!(peli.elamat, ELAMIA - 1);
    assert_eq!(peli.pallot.len(), 1);
}

#[test]
fn viimeisen_elaman_peli_jatkuu_toisella_pallolla() {
    let mut peli = pallotiilella();
    peli.elamat = 1;
    riko_ensimmainen(&mut peli);
    assert_eq!(pudota(&mut peli, 0), [Tapahtuma::ToinenMeni]);
    assert_eq!(peli.tila, Tila::Lennossa);
    assert_eq!(pudota(&mut peli, 0), [Tapahtuma::Ohi]);
    assert_eq!(peli.tila, Tila::Ohi);
    assert!(peli.pallot.is_empty());
    assert!(peli.askel(1, &mut Arpa::new(1)).is_empty());
}

#[test]
fn kentan_vaihtuessa_jaa_yksi_pallo_ja_kapea_maila() {
    let mut peli = pallotiilella();
    riko_ensimmainen(&mut peli);
    assert_eq!(peli.pallot.len(), 2);

    peli.tiilet = [[0; SARAKKEITA]; RIVEJA];
    peli.tiilet[0][0] = 1;
    let tapahtumat = riko_ensimmainen(&mut peli);
    assert!(matches!(
        tapahtumat.last(),
        Some(Tapahtuma::KenttaLapi { .. })
    ));
    assert_eq!(peli.pallot.len(), 1);
    assert_eq!(peli.maila_l(), MAILA_L);
    assert_eq!(peli.tila, Tila::Kiinni);
    assert_eq!(peli.pallot[0].paikka.0, peli.maila_x);
}

#[test]
fn kentan_viimeinen_pallotiili_ei_tuo_palloa() {
    let mut peli = lennossa_tyhjalla();
    peli.pallotiilet[0][0] = true;
    let tapahtumat = riko_ensimmainen(&mut peli);
    assert!(!tapahtumat.contains(&Tapahtuma::UusiPallo));
    assert_eq!(peli.pallot.len(), 1);
}

#[test]
fn tiiliosuma_kiihdyttaa_vain_osunutta_palloa() {
    for osuva in 0..2 {
        let mut peli = pallotiilella();
        riko_ensimmainen(&mut peli);
        peli.tiilet[3][5] = 1;
        let toinen = 1 - osuva;
        peli.pallot[toinen].paikka = (100.0, 300.0);
        let (toisen_vauhti, toisen_nopeus) =
            (peli.pallot[toinen].vauhti, peli.pallot[toinen].nopeus);
        let osuvan_vauhti = peli.pallot[osuva].vauhti;
        peli.pallot[osuva].paikka = (5.5 * TIILI_L, TIILET_Y + 4.0 * TIILI_K + PALLON_SADE + 0.5);
        peli.pallot[osuva].nopeus = (0.0, -osuvan_vauhti);
        peli.askel(0, &mut Arpa::new(1));
        assert_eq!(peli.pallot[osuva].vauhti, osuvan_vauhti + 4.0);
        assert_eq!(peli.pallot[osuva].nopeus, (0.0, osuvan_vauhti + 4.0));
        assert_eq!(peli.pallot[toinen].vauhti, toisen_vauhti);
        assert_eq!(peli.pallot[toinen].nopeus, toisen_nopeus);
    }
}

#[test]
fn maila_kimmottaa_kummankin_pallon_omalla_vauhdilla() {
    let mut peli = pallotiilella();
    riko_ensimmainen(&mut peli);
    assert_ne!(peli.pallot[0].vauhti, peli.pallot[1].vauhti);
    peli.aseta_maila(300.0);
    for (i, x) in [(0, 280.0), (1, 320.0)] {
        peli.pallot[i].paikka = (x, MAILA_Y - PALLON_SADE - 0.1);
        peli.pallot[i].nopeus = (0.0, 300.0);
    }
    peli.askel(0, &mut Arpa::new(1));
    for pallo in &peli.pallot {
        assert!(pallo.nopeus.1 < 0.0);
        assert!((pallo.nopeus.0.hypot(pallo.nopeus.1) - pallo.vauhti).abs() < 0.01);
    }
}

#[test]
fn jaljelle_jaava_pallo_sailyttaa_oman_vauhtinsa() {
    let mut peli = pallotiilella();
    riko_ensimmainen(&mut peli);
    let vanhan_vauhti = peli.pallot[0].vauhti;
    assert_eq!(pudota(&mut peli, 1), [Tapahtuma::ToinenMeni]);
    assert_eq!(peli.pallot[0].vauhti, vanhan_vauhti);
}

/// Supertiili riville 0 ja ylimääräinen tiili, joka pitää kentän kesken.
fn supertiilella() -> Tiilipeli {
    let mut peli = lennossa_tyhjalla();
    peli.tiilet[7][10] = 1;
    peli.supertiilet[0][0] = true;
    peli
}

fn vauhti(pallo: &Pallo) -> f32 {
    pallo.nopeus.0.hypot(pallo.nopeus.1)
}

#[test]
fn supertiili_tekee_rikkojasta_supervauhtisen_superpallon() {
    let mut peli = supertiilella();
    assert_eq!(
        riko_ensimmainen(&mut peli),
        [
            Tapahtuma::Tiili {
                rivi: 0,
                sarake: 0,
                rikki: true
            },
            Tapahtuma::Superpallo
        ]
    );
    let pallo = peli.pallot[0];
    assert!(pallo.superpallo);
    assert_eq!(peli.pallot.len(), 1);
    assert!((pallo.lentovauhti() - pallo.vauhti * SUPERVAUHTI).abs() < 0.01);
    assert!(
        (vauhti(&pallo)
            - (100.0f32.hypot(300.0) * SUPERVAUHTI * pallo.vauhti / (pallo.vauhti - 4.0)))
            .abs()
            < 0.1
    );
    // Muuttumisen tuova osuma lasketaan vielä tavallisena.
    assert_eq!(peli.pisteet, 10);
}

#[test]
fn superpallo_menee_tiilien_lapi_rikkoo_kestavatkin_ja_antaa_nelinkertaiset_pisteet() {
    let mut peli = supertiilella();
    riko_ensimmainen(&mut peli);
    let ennen = peli.pisteet;
    let nopeus = (0.0, -300.0);
    peli.tiilet[3][5] = 3;
    peli.pallot[0].paikka = (5.5 * TIILI_L, TIILET_Y + 4.0 * TIILI_K + PALLON_SADE + 0.5);
    peli.pallot[0].nopeus = nopeus;
    let vauhti_ennen = peli.pallot[0].vauhti;
    assert_eq!(
        peli.askel(0, &mut Arpa::new(1)),
        [Tapahtuma::Tiili {
            rivi: 3,
            sarake: 5,
            rikki: true
        }]
    );
    assert_eq!(peli.tiilet[3][5], 0);
    assert_eq!(peli.pisteet, ennen + 40);
    // Pallo jatkaa samaan suuntaan eikä kiihdy.
    assert_eq!(peli.pallot[0].nopeus, nopeus);
    assert_eq!(peli.pallot[0].vauhti, vauhti_ennen);
}

#[test]
fn superpallo_rikkoo_kerralla_kaikki_alleen_jaavat_tiilet() {
    let mut peli = supertiilella();
    riko_ensimmainen(&mut peli);
    let ennen = peli.pisteet;
    // Pallo on neljän tiilen kulmassa.
    for (rivi, sarake) in [(2, 4), (2, 5), (3, 4), (3, 5)] {
        peli.tiilet[rivi][sarake] = 1;
    }
    peli.pallot[0].paikka = (5.0 * TIILI_L, TIILET_Y + 3.0 * TIILI_K);
    peli.pallot[0].nopeus = (0.0, -1.0);
    peli.askel(0, &mut Arpa::new(1));
    for (rivi, sarake) in [(2, 4), (2, 5), (3, 4), (3, 5)] {
        assert_eq!(peli.tiilet[rivi][sarake], 0);
    }
    assert_eq!(peli.pisteet, ennen + 4 * 40);
}

#[test]
fn pallotiili_tuplaa_superpallon_ja_kahdesta_superpallosta_saa_kahdeksankertaiset() {
    let mut peli = supertiilella();
    riko_ensimmainen(&mut peli);
    peli.tiilet[3][5] = 1;
    peli.pallotiilet[3][5] = true;
    peli.pallot[0].paikka = (5.5 * TIILI_L, TIILET_Y + 4.0 * TIILI_K + PALLON_SADE + 0.5);
    peli.pallot[0].nopeus = (100.0, -300.0);
    let tapahtumat = peli.askel(0, &mut Arpa::new(1));
    assert!(tapahtumat.contains(&Tapahtuma::UusiPallo));

    let [vanha, uusi] = peli.pallot[..] else {
        panic!("palloja pitää olla kaksi");
    };
    assert!(vanha.superpallo && uusi.superpallo);
    assert!((vauhti(&uusi) - uusi.vauhti * SUPERVAUHTI).abs() < 0.01);

    let ennen = peli.pisteet;
    peli.tiilet[5][2] = 1;
    peli.pallot[1].paikka = (100.0, 300.0);
    peli.pallot[0].paikka = (2.5 * TIILI_L, TIILET_Y + 5.5 * TIILI_K);
    peli.askel(0, &mut Arpa::new(1));
    assert_eq!(peli.pisteet, ennen + 80);
}

#[test]
fn tavallinen_pallo_ei_muutu_toisen_supertiilesta() {
    let mut peli = pallotiilella();
    riko_ensimmainen(&mut peli);
    peli.tiilet[3][5] = 1;
    peli.supertiilet[3][5] = true;
    peli.pallot[1].paikka = (100.0, 300.0);
    peli.pallot[0].paikka = (5.5 * TIILI_L, TIILET_Y + 4.0 * TIILI_K + PALLON_SADE + 0.5);
    peli.pallot[0].nopeus = (0.0, -300.0);
    assert!(
        peli.askel(0, &mut Arpa::new(1))
            .contains(&Tapahtuma::Superpallo)
    );
    assert!(peli.pallot[0].superpallo);
    assert!(!peli.pallot[1].superpallo);
}

#[test]
fn superpallo_kimpoaa_mailasta_nopeammin_ja_loppuu_putoamiseen_ja_kentan_vaihtoon() {
    let mut peli = supertiilella();
    riko_ensimmainen(&mut peli);
    peli.aseta_maila(300.0);
    peli.pallot[0].paikka = (300.0, MAILA_Y - PALLON_SADE - 0.1);
    peli.pallot[0].nopeus = (0.0, 300.0);
    assert_eq!(peli.askel(0, &mut Arpa::new(1)), [Tapahtuma::Maila]);
    let pallo = peli.pallot[0];
    assert!((vauhti(&pallo) - pallo.vauhti * SUPERVAUHTI).abs() < 0.01);

    assert_eq!(pudota(&mut peli, 0), [Tapahtuma::Menetys]);
    assert!(!peli.pallot[0].superpallo);

    let mut peli = supertiilella();
    riko_ensimmainen(&mut peli);
    peli.tiilet = [[0; SARAKKEITA]; RIVEJA];
    peli.tiilet[0][0] = 1;
    let tapahtumat = riko_ensimmainen(&mut peli);
    assert!(matches!(
        tapahtumat.last(),
        Some(Tapahtuma::KenttaLapi { .. })
    ));
    assert!(!peli.pallot[0].superpallo);
}

#[test]
fn superpallo_lapaisee_kentan_viimeisen_tiilen_kaatumatta() {
    let mut peli = uusi_peli(KENTTIA - 1);
    peli.laukaise(&mut Arpa::new(1));
    peli.pallot[0].superpallo = true;
    peli.tiilet = [[0; SARAKKEITA]; RIVEJA];
    peli.tiilet[2][4] = 1;
    peli.tiilet[2][5] = 1;
    peli.pallot[0].paikka = (5.0 * TIILI_L, TIILET_Y + 2.5 * TIILI_K);
    peli.pallot[0].nopeus = (0.0, -1.0);
    let tapahtumat = peli.askel(0, &mut Arpa::new(1));
    assert!(matches!(tapahtumat.last(), Some(Tapahtuma::Voitto { .. })));
    assert_eq!(peli.tila, Tila::Voitto);
}

#[test]
fn pisteet_eivat_ylita_kattoa() {
    let mut peli = supertiilella();
    peli.pisteet = SUURIN_PISTEMAARA - 5;
    riko_ensimmainen(&mut peli);
    assert_eq!(peli.pisteet, SUURIN_PISTEMAARA);
}

#[test]
fn nopeimmankin_pallon_askel_on_halkaisijaa_lyhyempi() {
    let mut peli = supertiilella();
    riko_ensimmainen(&mut peli);
    peli.pallot[0].vauhti = 660.0;
    assert!(peli.pallot[0].lentovauhti() * ASKEL_MS as f32 / 1000.0 < 2.0 * PALLON_SADE);
}

fn pallo_tiilen_sisalla(peli: &Tiilipeli, (x, y): (f32, f32)) -> bool {
    (0..RIVEJA).any(|rivi| {
        (0..SARAKKEITA).any(|sarake| {
            let vasen = sarake as f32 * TIILI_L;
            let yla = TIILET_Y + rivi as f32 * TIILI_K;
            peli.tiilet[rivi][sarake] > 0
                && x + PALLON_SADE < vasen + TIILI_L + PALLON_SADE
                && x - PALLON_SADE > vasen - PALLON_SADE
                && y + PALLON_SADE < yla + TIILI_K + PALLON_SADE
                && y - PALLON_SADE > yla - PALLON_SADE
        })
    })
}

/// Automaatti pitää mailan pallon alla vaihtelevalla poikkeamalla, joten se pelaa koko pelin.
#[test]
fn automaatti_lapaisee_kaikki_kentat_eika_pallo_karkaa() {
    for siemen in 0..5 {
        let mut arpa = Arpa::new(siemen);
        let mut peli = Tiilipeli::new(0, &mut arpa);
        let mut poikkeama = 0.0;
        let mut askelia = 0u32;
        let mut toista_palloa_nahty = false;
        let mut superpalloa_nahty = false;
        while peli.tila != Tila::Voitto {
            askelia += 1;
            assert!(askelia < 200_000_000, "siemen {siemen}: peli ei pääty");
            if peli.tila == Tila::Kiinni {
                peli.laukaise(&mut arpa);
            }
            peli.aseta_maila(peli.pallot[0].paikka.0 + poikkeama);
            for tapahtuma in peli.askel(0, &mut arpa) {
                if tapahtuma == Tapahtuma::Maila {
                    poikkeama = arpa.alle(81) as f32 - 40.0;
                }
                assert_ne!(tapahtuma, Tapahtuma::Menetys, "siemen {siemen}");
            }
            assert!(peli.pallot.len() <= PALLOJA_ENINTAAN, "siemen {siemen}");
            toista_palloa_nahty |= peli.pallot.len() == 2;
            superpalloa_nahty |= peli.pallot.iter().any(|pallo| pallo.superpallo);
            for pallo in &peli.pallot {
                assert!(
                    !pallo_tiilen_sisalla(&peli, pallo.paikka),
                    "siemen {siemen}"
                );
                assert!(
                    (PALLON_SADE..=LEVEYS - PALLON_SADE).contains(&pallo.paikka.0),
                    "siemen {siemen}: x {}",
                    pallo.paikka.0
                );
                assert!(pallo.paikka.1 >= PALLON_SADE, "siemen {siemen}");
            }
            // Maila seuraa ensimmäistä palloa, joten vain toinen voi pudota.
            assert!(peli.pallot[0].paikka.1 <= MAILA_Y, "siemen {siemen}");
        }
        assert!(toista_palloa_nahty, "siemen {siemen}");
        assert!(superpalloa_nahty, "siemen {siemen}");
        assert!(peli.pisteet <= SUURIN_PISTEMAARA);
        assert_eq!(peli.tiilia_jaljella(), 0);
        assert_eq!(peli.kentta, KENTTIA - 1);
    }
}
