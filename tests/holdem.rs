use klassikot8000::Arpa;
use klassikot8000::holdem::{
    ALKURAHA, ISO_ALUSSA, Kortti, Kulku, Luokka, Maa, PELAAJIA, PIENI_ALUSSA, Potti, Poyta,
    TASON_JAKOJA, Teko, Tila, Toimi, paras_kasi,
};
use klassikot8000::kortit::sekoitettu;

/// Kortti tekstistä, esimerkiksi "As", "Td", "2h" tai "Kc".
fn k(teksti: &str) -> Kortti {
    let mut merkit = teksti.chars();
    let arvo = match merkit.next().unwrap() {
        'A' => 1,
        'T' => 10,
        'J' => 11,
        'Q' => 12,
        'K' => 13,
        numero => numero.to_digit(10).unwrap() as u8,
    };
    let maa = match merkit.next().unwrap() {
        's' => Maa::Pata,
        'h' => Maa::Hertta,
        'd' => Maa::Ruutu,
        _ => Maa::Risti,
    };
    Kortti { maa, arvo }
}

fn kortit(teksti: &str) -> Vec<Kortti> {
    teksti.split(' ').map(k).collect()
}

fn luokka(teksti: &str) -> Luokka {
    paras_kasi(&kortit(teksti)).unwrap().0.luokka
}

/// Vertaa käsiä: paras viiden kortin käsi kummastakin.
fn parempi(a: &str, b: &str) -> bool {
    paras_kasi(&kortit(a)).unwrap().0 > paras_kasi(&kortit(b)).unwrap().0
}

fn tasan(a: &str, b: &str) -> bool {
    paras_kasi(&kortit(a)).unwrap().0 == paras_kasi(&kortit(b)).unwrap().0
}

/// Pöytä, jossa `iso_ennen` oli edellisen jaon iso blindi ja rahat ovat annetut.
/// Seuraavassa jaossa hänestä tulee pieni blindi ja hänen vasemmasta naapuristaan iso.
fn poyta(iso_ennen: usize, rahat: [u32; PELAAJIA]) -> Poyta {
    let mut arpa = Arpa::new(1);
    let mut poyta = Poyta::new(&mut arpa);
    poyta.iso = iso_ennen;
    poyta.pieni = (iso_ennen + PELAAJIA - 1) % PELAAJIA;
    for (pelaaja, rahat) in poyta.pelaajat.iter_mut().zip(rahat) {
        pelaaja.rahat = rahat;
    }
    poyta
}

/// Pakka, josta nousevat ensin annetut kortit ja niiden jälkeen muut, sekoitettuina.
fn pakka(ylimmat: &str) -> Vec<Kortti> {
    let ylimmat = if ylimmat.is_empty() {
        Vec::new()
    } else {
        kortit(ylimmat)
    };
    let mut pakka: Vec<Kortti> = sekoitettu(1, &mut Arpa::new(9))
        .into_iter()
        .filter(|kortti| !ylimmat.contains(kortti))
        .collect();
    pakka.extend(ylimmat.iter().rev());
    pakka
}

fn jaettu(iso_ennen: usize, rahat: [u32; PELAAJIA]) -> Poyta {
    let mut p = poyta(iso_ennen, rahat);
    assert!(p.jaa_pakasta(pakka("")));
    p
}

const TAYSI: [u32; PELAAJIA] = [ALKURAHA; PELAAJIA];

fn rahat_yhteensa(p: &Poyta) -> u32 {
    p.pelaajat.iter().map(|x| x.rahat).sum::<u32>() + p.potti()
}

/// Etenee, kunnes joku saa päättää tai jako on ohi. Kortit ja näytöt vain, ei koneen tekoja.
fn kortit_esiin(p: &mut Poyta) {
    let mut arpa = Arpa::new(2);
    while matches!(p.kulku, Kulku::Kaanto | Kulku::Naytto) {
        p.etene(&mut arpa);
    }
}

fn kaikki_tarkistavat_kadun(p: &mut Poyta) {
    while p.kulku == Kulku::Panostus {
        assert!(p.toimi(Toimi::Maksa));
    }
}

// Käsien arvotus

#[test]
fn luokat_ovat_oikeassa_jarjestyksessa() {
    let jarjestys = [
        ("Ks Qh 9d 4c 2s", Luokka::Hai),
        ("Ks Kh 9d 4c 2s", Luokka::Pari),
        ("Ks Kh 9d 9c 2s", Luokka::KaksiParia),
        ("Ks Kh Kd 4c 2s", Luokka::Kolmoset),
        ("9s 8h 7d 6c 5s", Luokka::Suora),
        ("Ks Qs 9s 4s 2s", Luokka::Vari),
        ("Ks Kh Kd 4c 4s", Luokka::Tayskasi),
        ("Ks Kh Kd Kc 2s", Luokka::Neloset),
        ("9s 8s 7s 6s 5s", Luokka::Varisuora),
    ];
    for (kasi, odotettu) in jarjestys {
        assert_eq!(luokka(kasi), odotettu, "{kasi}");
    }
    for pari in jarjestys.windows(2) {
        assert!(
            parempi(pari[1].0, pari[0].0),
            "{} > {}",
            pari[1].0,
            pari[0].0
        );
    }
    assert!(
        paras_kasi(&kortit("As Ks Qs Js Ts"))
            .unwrap()
            .0
            .on_kuningasvarisuora()
    );
    assert!(
        !paras_kasi(&kortit("9s 8s 7s 6s 5s"))
            .unwrap()
            .0
            .on_kuningasvarisuora()
    );
}

#[test]
fn assa_kay_suoran_kumpaankin_paahan_muttei_kierra() {
    assert_eq!(luokka("As Kh Qd Jc Ts"), Luokka::Suora);
    assert_eq!(luokka("As 2h 3d 4c 5s"), Luokka::Suora);
    assert_eq!(luokka("Qs Kh Ad 2c 3s"), Luokka::Hai);
    // pyörä on pienin suora ja korkein on ässäkärkinen
    assert!(parempi("2s 3h 4d 5c 6s", "As 2h 3d 4c 5s"));
    assert!(parempi("As Kh Qd Jc Ts", "Ks Qh Jd Tc 9s"));
    assert_eq!(luokka("As 2s 3s 4s 5s"), Luokka::Varisuora);
    assert!(parempi("2s 3s 4s 5s 6s", "Ah 2h 3h 4h 5h"));
}

#[test]
fn potkurit_ratkaisevat_ja_maa_ei_koskaan() {
    assert!(parempi("Ks Kh Ad 4c 2s", "Kd Kc Qd 4h 2h"));
    assert!(parempi("Ks Kh Ad 5c 2s", "Kd Kc Ah 4h 3h"));
    assert!(parempi("Ks Kh 9d 9c 3s", "Kd Kc 9h 9s 2h"));
    assert!(parempi("Ks Kh 2d 2c 3s", "Qd Qc Jh Js As"));
    assert!(parempi("3s 3h 3d 2c 2s", "2d 2h 2s Ac Ad"));
    assert!(parempi("As Js 9s 4s 2s", "Ah Th 9h 4h 2h"));
    assert!(parempi("Ks Kh Kd Kc 3s", "Qs Qh Qd Qc As"));
    assert!(tasan("As Ks 9d 4c 2s", "Ah Kh 9s 4d 2c"));
    assert!(tasan("As Ks Qs Js Ts", "Ah Kh Qh Jh Th"));
}

#[test]
fn seitsemasta_kortista_valitaan_paras_viisi() {
    // kolmesta parista kaksi parasta ja korkein viides kortti
    let (arvo, viisi) = paras_kasi(&kortit("As Ah Kd Kc 2s 2h Qd")).unwrap();
    assert_eq!(arvo.luokka, Luokka::KaksiParia);
    assert_eq!(arvo.arvot, [14, 13, 12, 0, 0]);
    assert!(viisi.contains(&k("Qd")));
    // kahdet kolmoset ovat täyskäsi
    let (arvo, _) = paras_kasi(&kortit("9s 9h 9d 4c 4s 4h Ad")).unwrap();
    assert_eq!(arvo.luokka, Luokka::Tayskasi);
    assert_eq!(arvo.arvot, [9, 4, 0, 0, 0]);
    // kuudesta samasta maasta viisi korkeinta
    let (arvo, _) = paras_kasi(&kortit("As Ks 9s 7s 4s 2s Ad")).unwrap();
    assert_eq!(arvo.luokka, Luokka::Vari);
    assert_eq!(arvo.arvot, [14, 13, 9, 7, 4]);
    // yksi oma kortti riittää
    let (arvo, viisi) = paras_kasi(&kortit("4s 2d 5s 6s 7s 8s Kh")).unwrap();
    assert_eq!(arvo.luokka, Luokka::Varisuora);
    assert!(!viisi.contains(&k("2d")));
    assert!(paras_kasi(&kortit("As Ks Qs Js")).is_none());
}

#[test]
fn poydan_pelaaminen_jakaa_potin() {
    // Kummankaan omat kortit eivät paranna pöydän suoraa.
    let poydalla = "Ts 9h 8d 7c 6s";
    assert!(tasan(
        &format!("2h 3h {poydalla}"),
        &format!("2d 4c {poydalla}")
    ));
}

// Jako

#[test]
fn kortit_jaetaan_yksi_kerrallaan_napista_vasemmalle_ja_kortti_poltetaan_ennen_katua() {
    let mut p = poyta(1, TAYSI);
    // Nappi 0, pieni 1, iso 2: jako alkaa pienestä blindistä ja päättyy nappiin.
    let ylimmat = "2s 3s 4s 5s 6s 7h 8h 9h Th Jh Ad Kd Qd Jd Td 9d 8d 7d";
    assert!(p.jaa_pakasta(pakka(ylimmat)));
    assert_eq!((p.nappi, p.pieni, p.iso), (0, 1, 2));
    assert_eq!(p.pelaajat[1].kortit, kortit("2s 7h"));
    assert_eq!(p.pelaajat[2].kortit, kortit("3s 8h"));
    assert_eq!(p.pelaajat[3].kortit, kortit("4s 9h"));
    assert_eq!(p.pelaajat[4].kortit, kortit("5s Th"));
    assert_eq!(p.pelaajat[0].kortit, kortit("6s Jh"));

    kaikki_tarkistavat_kadun(&mut p);
    assert_eq!(p.kulku, Kulku::Kaanto);
    kortit_esiin(&mut p);
    // Ad poltettu
    assert_eq!(p.yhteiset, kortit("Kd Qd Jd"));
    kaikki_tarkistavat_kadun(&mut p);
    kortit_esiin(&mut p);
    // Td poltettu
    assert_eq!(p.yhteiset, kortit("Kd Qd Jd 9d"));
    kaikki_tarkistavat_kadun(&mut p);
    kortit_esiin(&mut p);
    // 8d poltettu, mutta 7d ei
    let viides = p.yhteiset[4];
    assert_eq!(viides, k("7d"));
    assert_eq!(p.yhteiset.len(), 5);
}

// Blindit ja vuorojärjestys

#[test]
fn blindit_maksetaan_ja_ensimmaisena_toimii_ison_blindin_vasen_naapuri() {
    let p = jaettu(1, TAYSI);
    assert_eq!(p.pelaajat[1].panos, PIENI_ALUSSA);
    assert_eq!(p.pelaajat[2].panos, ISO_ALUSSA);
    assert_eq!(p.pelaajat[1].rahat, ALKURAHA - PIENI_ALUSSA);
    assert_eq!(p.vuorossa, Some(3));
    assert_eq!(p.maksu(3), ISO_ALUSSA);
    assert_eq!(p.maksu(1), ISO_ALUSSA - PIENI_ALUSSA);
    assert_eq!(p.maksu(2), 0);
}

#[test]
fn floppia_edeltavan_jalkeen_ensimmaisena_toimii_napin_vasen_naapuri() {
    let mut p = jaettu(1, TAYSI);
    kaikki_tarkistavat_kadun(&mut p);
    kortit_esiin(&mut p);
    assert_eq!(p.vuorossa, Some(1));
    assert_eq!(p.korkein, 0);
    assert_eq!(p.pelaajat[1].panos, 0);
    // pieni blindi kippasi: seuraava mukana oleva aloittaa
    let mut p = jaettu(1, TAYSI);
    for toimi in [Toimi::Maksa, Toimi::Maksa, Toimi::Maksa, Toimi::Kippaa] {
        assert!(p.toimi(toimi));
    }
    assert!(p.toimi(Toimi::Maksa));
    kortit_esiin(&mut p);
    assert_eq!(p.vuorossa, Some(2));
}

#[test]
fn isolla_blindilla_on_optio_kun_kukaan_ei_korottanut() {
    let mut p = jaettu(1, TAYSI);
    for _ in 0..4 {
        assert!(p.toimi(Toimi::Maksa));
    }
    assert_eq!(p.vuorossa, Some(2));
    assert_eq!(p.maksu(2), 0);
    assert!(p.voi_korottaa(2));
    // optio: iso blindi korottaa, ja muut saavat vastata
    assert!(p.toimi(Toimi::Korota(60)));
    assert_eq!(p.vuorossa, Some(3));
    assert_eq!(p.maksu(3), 40);

    // tarkistus päättää kierroksen
    let mut p = jaettu(1, TAYSI);
    for _ in 0..5 {
        assert!(p.toimi(Toimi::Maksa));
    }
    assert_eq!(p.kulku, Kulku::Kaanto);
    assert_eq!(p.pelaajat[2].teko, Some(Teko::Tarkisti));
}

#[test]
fn nappi_siirtyy_ja_blindit_kiertavat_myotapaivaan() {
    let mut p = jaettu(1, TAYSI);
    assert_eq!((p.nappi, p.pieni, p.iso), (0, 1, 2));
    for odotettu in [(1, 2, 3), (2, 3, 4), (3, 4, 0), (4, 0, 1), (0, 1, 2)] {
        p.jaa_pakasta(pakka(""));
        assert_eq!((p.nappi, p.pieni, p.iso), odotettu);
    }
}

#[test]
fn blindit_tuplaantuvat_vain_uuden_jaon_alussa() {
    let mut p = jaettu(1, TAYSI);
    assert_eq!(p.blindit(), (10, 20));
    assert_eq!(p.jakoja_tasolla(), TASON_JAKOJA - 1);
    for _ in 1..TASON_JAKOJA {
        p.jaa_pakasta(pakka(""));
    }
    assert_eq!(p.jako, TASON_JAKOJA);
    assert_eq!(p.blindit(), (10, 20));
    assert_eq!(p.jakoja_tasolla(), 0);
    p.jaa_pakasta(pakka(""));
    assert_eq!(p.blindit(), (20, 40));
    assert_eq!(p.jakoja_tasolla(), TASON_JAKOJA - 1);
    assert_eq!(p.pienin_korotus, 40);
}

// Panostus

#[test]
fn pienin_panos_on_iso_blindi_ja_korotus_vahintaan_edellisen_kokoinen() {
    let mut p = jaettu(1, TAYSI);
    // korotus tasoon 40 on pienin sallittu
    assert_eq!(p.pienin_taso(3), 40);
    assert!(!p.toimi(Toimi::Korota(39)));
    assert!(!p.toimi(Toimi::Korota(ALKURAHA + 1)));
    assert_eq!(p.vuorossa, Some(3));
    assert!(p.toimi(Toimi::Korota(100)));
    assert_eq!(p.pelaajat[3].teko, Some(Teko::Korotti(100)));
    // korotus oli 80, joten seuraava on vähintään tasoon 180
    assert_eq!(p.pienin_korotus, 80);
    assert_eq!(p.pienin_taso(4), 180);
    assert!(!p.toimi(Toimi::Korota(179)));
    assert!(p.toimi(Toimi::Korota(180)));
    assert_eq!(p.pienin_taso(0), 260);

    let mut p = jaettu(1, TAYSI);
    kaikki_tarkistavat_kadun(&mut p);
    kortit_esiin(&mut p);
    // kadun ensimmäinen panos: vähintään iso blindi
    assert_eq!(p.pienin_taso(1), ISO_ALUSSA);
    assert!(!p.toimi(Toimi::Korota(19)));
    assert!(p.toimi(Toimi::Korota(50)));
    assert_eq!(p.pelaajat[1].teko, Some(Teko::Panosti(50)));
    assert_eq!(p.pienin_taso(2), 100);
}

#[test]
fn korotusten_maaraa_ei_ole_rajattu() {
    let mut p = jaettu(1, [100_000; PELAAJIA]);
    let mut taso = 20;
    for kerta in 1..=20 {
        taso += 20;
        assert!(p.toimi(Toimi::Korota(taso)), "{kerta}");
    }
}

#[test]
fn vajaa_all_in_ei_avaa_panostusta_jo_toimineelle() {
    let mut p = jaettu(1, [1000, 1000, 1000, 1000, 150]);
    // 3 korottaa tasoon 100 (korotus 80), 4 all-in 150 (vain 50 lisää)
    assert!(p.toimi(Toimi::Korota(100)));
    assert_eq!(p.pienin_taso(4), 150);
    assert!(p.toimi(Toimi::Korota(150)));
    assert_eq!(p.korkein, 150);
    assert_eq!(p.pienin_korotus, 80);
    // 0 ei ole vielä toiminut: se saa korottaa, ja pienin mitataan täydestä korotuksesta
    assert!(p.voi_korottaa(0));
    assert_eq!(p.pienin_taso(0), 230);
    assert!(p.toimi(Toimi::Maksa));
    assert!(p.toimi(Toimi::Kippaa));
    assert!(p.toimi(Toimi::Kippaa));
    // 3 on jo toiminut eikä edessä ole täyttä korotusta: maksa tai kippaa
    assert_eq!(p.vuorossa, Some(3));
    assert_eq!(p.maksu(3), 50);
    assert!(!p.voi_korottaa(3));
    assert!(!p.toimi(Toimi::Korota(230)));
    assert!(!p.toimi(Toimi::Korota(1000)));
    assert_eq!(p.vuorossa, Some(3));
    assert!(p.toimi(Toimi::Maksa));
    assert_eq!(p.kulku, Kulku::Kaanto);
}

#[test]
fn useampi_vajaa_all_in_yhdessa_avaa_panostuksen() {
    // Pelaaja 1 pysyy mukana, jotta korotukselle on vastaaja.
    let mut p = jaettu(1, [150, 1000, 1000, 1000, 120]);
    // 3 tasoon 100 (korotus 80), 4 all-in 120, 0 all-in 150: yhteensä 50, ei vielä täysi
    assert!(p.toimi(Toimi::Korota(100)));
    assert!(p.toimi(Toimi::Korota(120)));
    assert!(p.toimi(Toimi::Korota(150)));
    assert!(p.toimi(Toimi::Maksa));
    assert!(p.toimi(Toimi::Kippaa));
    assert_eq!(p.vuorossa, Some(3));
    assert!(!p.voi_korottaa(3));

    let mut p = jaettu(1, [190, 1000, 1000, 1000, 150]);
    // nyt vajaat all-init ovat yhteensä 90, joten edessä on täysi korotus
    assert!(p.toimi(Toimi::Korota(100)));
    assert!(p.toimi(Toimi::Korota(150)));
    assert!(p.toimi(Toimi::Korota(190)));
    assert!(p.toimi(Toimi::Maksa));
    assert!(p.toimi(Toimi::Kippaa));
    assert_eq!(p.vuorossa, Some(3));
    assert!(p.voi_korottaa(3));
    // pienin korotus on yhä viimeisin täysi korotus
    assert_eq!(p.pienin_korotus, 80);
    assert_eq!(p.pienin_taso(3), 270);
}

#[test]
fn korotus_ei_kay_kun_kukaan_ei_voi_enaa_vastata() {
    // 4 ja 0 ovat all-in, 3 ja 1 kippasivat: iso blindi on viimeinen, jolla on rahaa.
    let mut p = jaettu(1, [300, 1000, 1000, 1000, 300]);
    assert!(p.toimi(Toimi::Kippaa));
    assert!(p.toimi(Toimi::Korota(300)));
    assert!(p.toimi(Toimi::Maksa));
    assert!(p.toimi(Toimi::Kippaa));
    assert_eq!(p.vuorossa, Some(2));
    assert!(!p.voi_korottaa(2));
    assert!(!p.toimi(Toimi::Korota(600)));
    assert_eq!(p.maksu(2), 280);
    assert!(p.toimi(Toimi::Maksa));
    assert_eq!(p.kulku, Kulku::Kaanto);
}

#[test]
fn vajaa_iso_blindi_maksetaan_silti_taytena() {
    let mut p = jaettu(1, [1000, 1000, 15, 1000, 1000]);
    assert_eq!(p.pelaajat[2].panos, 15);
    assert_eq!(p.korkein, ISO_ALUSSA);
    assert_eq!(p.maksu(3), ISO_ALUSSA);
    assert_eq!(p.pienin_taso(3), 2 * ISO_ALUSSA);
    assert!(p.toimi(Toimi::Maksa));
    assert_eq!(p.pelaajat[3].panos, ISO_ALUSSA);
    // Vain pelaaja 3 on enää potissa, ja sen ylimenevä viisi palaa kadun lopussa.
    assert!(p.toimi(Toimi::Kippaa));
    assert!(p.toimi(Toimi::Kippaa));
    assert!(p.toimi(Toimi::Kippaa));
    assert_eq!(p.kulku, Kulku::Kaanto);
    assert_eq!(p.pelaajat[3].yhteensa, 15);
    assert_eq!(p.pelaajat[3].rahat, ALKURAHA - 15);
}

#[test]
fn all_in_maksu_rajoittuu_rahoihin() {
    let mut p = jaettu(1, [1000, 1000, 1000, 15, 1000]);
    assert_eq!(p.maksu(3), 15);
    assert!(!p.voi_korottaa(3));
    assert!(p.toimi(Toimi::Maksa));
    assert_eq!(p.pelaajat[3].rahat, 0);
    assert_eq!(p.pelaajat[3].tila, Tila::Mukana);
    assert_eq!(p.vuorossa, Some(4));
}

// Potit

fn panokset(iso_ennen: usize, panokset: [u32; PELAAJIA]) -> Poyta {
    let mut p = poyta(iso_ennen, TAYSI);
    p.jaa_pakasta(pakka(""));
    for (pelaaja, panos) in p.pelaajat.iter_mut().zip(panokset) {
        pelaaja.yhteensa = panos;
    }
    p
}

#[test]
fn sivupotit_syntyvat_all_inien_tasoille_ja_kippaajan_rahat_jaavat_pottiin() {
    let mut p = panokset(1, [100, 300, 300, 500, 50]);
    p.pelaajat[4].tila = Tila::Kippasi;
    assert_eq!(
        p.potit(),
        [
            Potti {
                summa: 450,
                oikeutetut: vec![0, 1, 2, 3],
            },
            Potti {
                summa: 600,
                oikeutetut: vec![1, 2, 3],
            },
            Potti {
                summa: 200,
                oikeutetut: vec![3],
            },
        ]
    );
    let summa: u32 = p.potit().iter().map(|x| x.summa).sum();
    assert_eq!(summa, p.potti());
}

#[test]
fn kippaajan_suurempi_panos_ei_katoa() {
    let mut p = panokset(1, [50, 200, 100, 0, 0]);
    p.pelaajat[1].tila = Tila::Kippasi;
    p.pelaajat[3].tila = Tila::Kippasi;
    p.pelaajat[4].tila = Tila::Kippasi;
    let summa: u32 = p.potit().iter().map(|x| x.summa).sum();
    assert_eq!(summa, 350);
}

#[test]
fn maksamaton_osa_palaa_eika_kukaan_voita_sita() {
    let mut p = jaettu(1, TAYSI);
    // 3 korottaa kaikilla rahoillaan, ja 4 pystyy maksamaan siitä vain 300
    p.pelaajat[4].rahat = 300;
    assert!(p.toimi(Toimi::Korota(1000)));
    assert!(p.toimi(Toimi::Maksa));
    assert_eq!(p.pelaajat[4].rahat, 0);
    assert!(p.toimi(Toimi::Kippaa));
    assert!(p.toimi(Toimi::Kippaa));
    assert!(p.toimi(Toimi::Kippaa));
    assert_eq!(p.kulku, Kulku::Kaanto);
    assert_eq!(p.pelaajat[3].rahat, 700);
    assert_eq!(p.pelaajat[3].yhteensa, 300);
    assert_eq!(p.pelaajat[4].yhteensa, 300);
}

#[test]
fn kaikkien_kipatessa_viimeinen_voittaa_ilman_korttien_nayttoa() {
    let mut p = jaettu(1, TAYSI);
    for _ in 0..4 {
        assert!(p.toimi(Toimi::Kippaa));
    }
    assert_eq!(p.kulku, Kulku::Ohi);
    assert_eq!(p.vuorossa, None);
    // iso blindi sai maksamattoman osansa takaisin
    assert_eq!(p.pelaajat[2].voitto, 2 * PIENI_ALUSSA);
    assert_eq!(p.pelaajat[2].rahat, ALKURAHA + PIENI_ALUSSA);
    assert!(!p.pelaajat.iter().any(|x| x.naytetty));
    assert!(p.yhteiset.is_empty());
    assert_eq!(rahat_yhteensa(&p), 5 * ALKURAHA);
}

/// Jako, jossa nappi 0, pieni 1, iso 2, ja kortit jaetaan 1, 2, 3, 4, 0.
fn jako_korteilla(rahat: [u32; PELAAJIA], kadet: [&str; 5], poyta_kortit: &str) -> Poyta {
    let mut p = poyta(1, rahat);
    let kasi: Vec<Vec<Kortti>> = kadet.iter().map(|k| kortit(k)).collect();
    let mut ylimmat: Vec<Kortti> = (0..2)
        .flat_map(|kortti| [1, 2, 3, 4, 0].map(|i| kasi[i][kortti]))
        .collect();
    let yhteiset = kortit(poyta_kortit);
    let polttokortit = kortit("Qc Qd Th");
    ylimmat.push(polttokortit[0]);
    ylimmat.extend_from_slice(&yhteiset[..3]);
    ylimmat.push(polttokortit[1]);
    ylimmat.push(yhteiset[3]);
    ylimmat.push(polttokortit[2]);
    ylimmat.push(yhteiset[4]);
    let mut loput: Vec<Kortti> = sekoitettu(1, &mut Arpa::new(9));
    loput.retain(|kortti| !ylimmat.contains(kortti));
    loput.extend(ylimmat.iter().rev());
    assert!(p.jaa_pakasta(loput));
    p
}

fn pelaa_loppuun(p: &mut Poyta) {
    let mut arpa = Arpa::new(3);
    while p.kulku != Kulku::Ohi {
        if p.kulku == Kulku::Panostus {
            assert!(p.toimi(Toimi::Maksa));
        } else {
            p.etene(&mut arpa);
        }
    }
}

#[test]
fn all_in_saa_vain_paapotin_ja_sivupotti_menee_seuraavaksi_parhaalle() {
    // 3:lla on paras käsi, mutta vain 100 rahaa. 4 voittaa sivupotin 0:aa vastaan.
    let mut p = jako_korteilla(
        [1000, 1000, 1000, 100, 1000],
        ["7h 8h", "5d 5s", "2d 7c", "As Ad", "Ks Kd"],
        "Ah Kh 9c 6s Jd",
    );
    assert!(p.toimi(Toimi::Korota(100)));
    assert_eq!(p.pelaajat[3].rahat, 0);
    assert!(p.toimi(Toimi::Korota(300)));
    assert!(p.toimi(Toimi::Maksa));
    assert!(p.toimi(Toimi::Kippaa));
    assert!(p.toimi(Toimi::Kippaa));
    assert_eq!(p.sivupotteja(), 1);
    pelaa_loppuun(&mut p);
    assert_eq!(p.kulku, Kulku::Ohi);

    // pääpotti: 100 * 3 + blindit 10 + 20. sivupotti: 200 * 2.
    assert_eq!(p.pelaajat[3].voitto, 330);
    assert_eq!(p.pelaajat[3].paapotista, 330);
    assert_eq!(p.pelaajat[3].rahat, 330);
    assert_eq!(p.pelaajat[4].voitto, 400);
    assert_eq!(p.pelaajat[4].paapotista, 0);
    assert_eq!(p.pelaajat[0].voitto, 0);
    assert_eq!(p.pelaajat[4].rahat, 1100);
    assert_eq!(rahat_yhteensa(&p), 5000 - 900);
}

#[test]
fn pariton_merkki_menee_ensimmaiselle_napista_vasemmalle() {
    // Pöydän väri on kaikille paras. Potti on 90 (neljä kertaa 20 ja kipanneen pienen 10).
    let mut p = jako_korteilla(
        TAYSI,
        ["2c 3d", "2d 3c", "4c 5d", "4d 5c", "6c 6d"],
        "As Ks Qs Js 9s",
    );
    assert!(p.toimi(Toimi::Maksa));
    assert!(p.toimi(Toimi::Maksa));
    assert!(p.toimi(Toimi::Maksa));
    assert!(p.toimi(Toimi::Kippaa));
    pelaa_loppuun(&mut p);
    // 90 / 4 = 22 ja kaksi paritonta: nappi on 0, ja pieni 1 kippasi, joten 2 ja 3 saavat ne.
    let voitot: Vec<u32> = p.pelaajat.iter().map(|x| x.voitto).collect();
    assert_eq!(voitot, [22, 0, 23, 23, 22]);
    assert_eq!(p.pelaajat.iter().map(|x| x.rahat).sum::<u32>(), 5000);
}

#[test]
fn jokainen_sivupotti_jaetaan_erikseen() {
    // 3 ja 4 ovat tasan pääpotissa, ja 4 voittaa sivupotin yksin.
    let mut p = jako_korteilla(
        [1000, 1000, 1000, 100, 1000],
        ["7h 8h", "5d 5s", "2d 7c", "Ah Kd", "Ad Kc"],
        "As Kh 9c 6s Jd",
    );
    assert!(p.toimi(Toimi::Korota(100)));
    assert!(p.toimi(Toimi::Korota(300)));
    assert!(p.toimi(Toimi::Maksa));
    assert!(p.toimi(Toimi::Kippaa));
    assert!(p.toimi(Toimi::Kippaa));
    pelaa_loppuun(&mut p);
    // pääpotti 100 * 3 + blindit 30 = 330 kahtia, sivupotti 200 * 2 = 400
    assert_eq!(p.pelaajat[3].voitto, 165);
    assert_eq!(p.pelaajat[4].voitto, 165 + 400);
    assert_eq!(p.pelaajat[0].voitto, 0);
}

/// Vain 0 ja 2 jatkavat näyttöön asti. Potti on 50: kaksi isoa blindiä ja kipanneen pieni.
fn kaksi_nayttoon(oma: &str, toinen: &str, poyta_kortit: &str) -> Poyta {
    let mut p = jako_korteilla(
        TAYSI,
        [oma, "2d 3c", toinen, "4h 4c", "2h 3h"],
        poyta_kortit,
    );
    assert!(p.toimi(Toimi::Kippaa));
    assert!(p.toimi(Toimi::Kippaa));
    assert!(p.toimi(Toimi::Maksa));
    assert!(p.toimi(Toimi::Kippaa));
    pelaa_loppuun(&mut p);
    p
}

#[test]
fn korkeampi_suora_vie_koko_potin() {
    let p = kaksi_nayttoon("9h Tc", "9d 4c", "5s 6h 7d 8c Kd");
    assert_eq!(p.pelaajat[0].kasi.unwrap().0.luokka, Luokka::Suora);
    assert_eq!(p.pelaajat[2].kasi.unwrap().0.luokka, Luokka::Suora);
    assert_eq!(p.pelaajat[0].voitto, 50);
    assert_eq!(p.pelaajat[2].voitto, 0);
}

#[test]
fn sama_suora_jakaa_potin_vaikka_omat_kortit_olisivat_korkeammat() {
    // Pöydän suora on molempien paras käsi, eikä viiden kortin ulkopuolinen ässä ratkaise.
    let p = kaksi_nayttoon("As Kh", "2c 3d", "5s 6h 7d 8c 9s");
    assert_eq!(p.pelaajat[0].voitto, 25);
    assert_eq!(p.pelaajat[2].voitto, 25);
    assert_eq!(p.pelaajat[0].paapotista, 25);
    assert_eq!(p.pelaajat[2].paapotista, 25);
}

#[test]
fn nayttojarjestys_alkaa_viimeisesta_panostajasta_tai_napin_vasemmalta() {
    let mut p = jaettu(1, TAYSI);
    kaikki_tarkistavat_kadun(&mut p);
    kortit_esiin(&mut p);
    kaikki_tarkistavat_kadun(&mut p);
    kortit_esiin(&mut p);
    kaikki_tarkistavat_kadun(&mut p);
    kortit_esiin(&mut p);
    assert_eq!(p.yhteiset.len(), 5);
    assert_eq!(p.vuorossa, Some(1));
    assert_eq!(p.nayttojarjestys(), [1, 2, 3, 4, 0]);
    // 2 panostaa, 3 korottaa, muut maksavat: 3 näyttää ensin
    assert!(p.toimi(Toimi::Maksa));
    assert!(p.toimi(Toimi::Korota(40)));
    assert!(p.toimi(Toimi::Korota(100)));
    assert!(p.toimi(Toimi::Kippaa));
    assert!(p.toimi(Toimi::Maksa));
    assert!(p.toimi(Toimi::Maksa));
    assert!(p.toimi(Toimi::Maksa));
    assert_eq!(p.kulku, Kulku::Naytto);
    assert_eq!(p.nayttojarjestys(), [3, 0, 1, 2]);

    let mut arpa = Arpa::new(1);
    assert!(!p.pelaajat[3].naytetty);
    p.etene(&mut arpa);
    assert!(p.pelaajat[3].naytetty && !p.pelaajat[0].naytetty);
    p.etene(&mut arpa);
    assert!(p.pelaajat[0].naytetty && !p.pelaajat[1].naytetty);
    assert!(p.pelaajat[4].kasi.is_none() && !p.pelaajat[4].naytetty);
}

#[test]
fn kortit_kaannetaan_esiin_kun_panostus_loppuu_all_inien_takia() {
    let mut p = jaettu(1, TAYSI);
    assert!(p.toimi(Toimi::Korota(1000)));
    assert!(p.toimi(Toimi::Maksa));
    for _ in 0..3 {
        assert!(p.toimi(Toimi::Kippaa));
    }
    assert_eq!(p.kulku, Kulku::Kaanto);
    assert!(p.pelaajat[3].naytetty && p.pelaajat[4].naytetty);
    assert!(!p.pelaajat[0].naytetty);
    let mut arpa = Arpa::new(1);
    p.etene(&mut arpa);
    assert_eq!(p.yhteiset.len(), 3);
    assert_eq!(p.vuorossa, None);
    assert_eq!(p.kulku, Kulku::Kaanto);
    p.etene(&mut arpa);
    p.etene(&mut arpa);
    assert_eq!(p.yhteiset.len(), 5);
    assert_eq!(p.kulku, Kulku::Naytto);
    p.etene(&mut arpa);
    assert_eq!(p.kulku, Kulku::Ohi);
    assert_eq!(p.pelaajat.iter().map(|x| x.rahat).sum::<u32>(), 5000);
}

// Kaksinpeli ja kuollut nappi

#[test]
fn kaksinpelissa_nappi_on_pieni_blindi_toimii_ensin_floppia_ennen_ja_viimeisena_sen_jalkeen() {
    let mut p = poyta(1, [0, 0, 500, 0, 500]);
    assert!(p.jaa_pakasta(pakka("2s 3s 4s 5s")));
    // edellinen iso oli tyhjä paikka 1: iso siirtyy paikalle 2, ja toinen on nappi
    assert_eq!((p.nappi, p.pieni, p.iso), (4, 4, 2));
    assert_eq!(p.pelaajat[4].panos, PIENI_ALUSSA);
    assert_eq!(p.pelaajat[2].panos, ISO_ALUSSA);
    // nappi saa viimeisen kortin
    assert_eq!(p.pelaajat[2].kortit, kortit("2s 4s"));
    assert_eq!(p.pelaajat[4].kortit, kortit("3s 5s"));
    assert_eq!(p.vuorossa, Some(4));
    assert!(p.toimi(Toimi::Maksa));
    assert_eq!(p.vuorossa, Some(2));
    assert!(p.toimi(Toimi::Maksa));
    kortit_esiin(&mut p);
    assert_eq!(p.vuorossa, Some(2));
    assert!(p.toimi(Toimi::Maksa));
    assert_eq!(p.vuorossa, Some(4));

    // blindit vaihtuvat, eikä kukaan saa isoa kahdesti peräkkäin
    p.jaa_pakasta(pakka(""));
    assert_eq!((p.nappi, p.pieni, p.iso), (2, 2, 4));
}

#[test]
fn siirtyma_kaksinpeliin_ei_anna_isoa_blindia_kahdesti() {
    for iso_ennen in 0..PELAAJIA {
        for a in 0..PELAAJIA {
            for b in 0..PELAAJIA {
                if a == b {
                    continue;
                }
                let mut rahat = [0; PELAAJIA];
                rahat[a] = 500;
                rahat[b] = 500;
                let mut p = poyta(iso_ennen, rahat);
                assert!(p.jaa_pakasta(pakka("")));
                assert_ne!(p.iso, iso_ennen);
                assert_eq!(p.nappi, p.pieni);
                assert!(p.mukana_testissa(p.iso) && p.mukana_testissa(p.pieni));
                assert_ne!(p.iso, p.pieni);
            }
        }
    }
}

trait Mukana {
    fn mukana_testissa(&self, i: usize) -> bool;
}

impl Mukana for Poyta {
    fn mukana_testissa(&self, i: usize) -> bool {
        self.pelaajat[i].tila == Tila::Mukana
    }
}

#[test]
fn pudonnut_iso_blindi_tekee_pienesta_kuolleen_ja_iso_siirtyy_seuraavalle() {
    // Edellinen iso oli paikka 2, ja se putosi: pieni blindi jää maksamatta.
    let mut p = poyta(2, [1000, 1000, 0, 1000, 1000]);
    assert!(p.jaa_pakasta(pakka("")));
    assert_eq!(p.pieni, 2);
    assert_eq!(p.pelaajat[2].tila, Tila::Poissa);
    assert_eq!(p.iso, 3);
    assert_eq!(p.nappi, 1);
    assert_eq!(p.potti(), ISO_ALUSSA);
    assert!(p.pelaajat[2].kortit.is_empty());
    assert_eq!(p.vuorossa, Some(4));
    kaikki_tarkistavat_kadun(&mut p);
    kortit_esiin(&mut p);
    // iso on ensimmäinen napista vasemmalle mukana oleva
    assert_eq!(p.vuorossa, Some(3));
}

#[test]
fn pudonnut_pieni_blindi_jattaa_napin_tyhjalle_paikalle() {
    // TDA 36-B: edellinen pieni 1 putosi, joten nappi siirtyy pienen viereen.
    let mut p = poyta(2, [1000, 0, 1000, 1000, 1000]);
    assert!(p.jaa_pakasta(pakka("")));
    assert_eq!((p.pieni, p.iso), (2, 3));
    assert_eq!(p.nappi, 1);
    assert_eq!(p.pelaajat[1].tila, Tila::Poissa);
    assert_eq!(p.pelaajat[2].panos, PIENI_ALUSSA);
    // kortit alkavat pienestä blindistä, ja tyhjä nappipaikka ei saa kortteja
    assert_eq!(p.vuorossa, Some(4));
    kaikki_tarkistavat_kadun(&mut p);
    kortit_esiin(&mut p);
    assert_eq!(p.vuorossa, Some(2));
}

#[test]
fn tyhjat_paikat_ohitetaan_ison_blindin_siirtyessa() {
    let mut p = poyta(0, [1000, 0, 0, 1000, 1000]);
    assert!(p.jaa_pakasta(pakka("")));
    assert_eq!(p.pieni, 0);
    assert_eq!(p.iso, 3);
    assert_eq!(p.vuorossa, Some(4));
}

#[test]
fn peli_ei_jatku_kun_vain_yksi_on_rahoissa() {
    let mut p = poyta(0, [5000, 0, 0, 0, 0]);
    assert!(!p.jaa_pakasta(pakka("")));
    assert_eq!(p.kulku, Kulku::Ohi);
}

#[test]
fn blindi_voi_vieda_koko_pinon_ja_jako_etenee_silti() {
    // Molemmat all-in pelkillä blindeillä: kukaan ei päätä mitään.
    let mut p = poyta(0, [5, 5, 0, 0, 0]);
    assert!(p.jaa_pakasta(pakka("")));
    assert_eq!(p.vuorossa, None);
    assert_eq!(p.kulku, Kulku::Kaanto);
    pelaa_loppuun(&mut p);
    assert_eq!(p.pelaajat[0].rahat + p.pelaajat[1].rahat, 10);

    // Toinen on yksin rahoissa, mutta jo mukana enemmällä kuin all-in: ei päätöstä.
    let mut p = poyta(0, [100, 5, 0, 0, 0]);
    assert!(p.jaa_pakasta(pakka("")));
    assert_eq!((p.pieni, p.iso), (0, 1));
    assert_eq!(p.vuorossa, None);
    assert_eq!(p.pelaajat[0].rahat, 95);
    // ylimenevä pieni blindi palaa
    assert_eq!(p.pelaajat[0].yhteensa, 5);
    pelaa_loppuun(&mut p);
    assert_eq!(p.pelaajat[0].rahat + p.pelaajat[1].rahat, 105);
}

#[test]
fn vajaata_blindia_vastaan_viimeinen_maksaa_vain_sen_verran() {
    let mut p = poyta(0, [100, 15, 0, 0, 0]);
    assert!(p.jaa_pakasta(pakka("")));
    assert_eq!(p.vuorossa, Some(0));
    assert_eq!(p.maksu(0), 5);
    assert!(!p.voi_korottaa(0));
    assert!(p.toimi(Toimi::Maksa));
    assert_eq!(p.pelaajat[0].yhteensa, 15);
    assert_eq!(p.kulku, Kulku::Kaanto);
}

// Tietokone

fn koneiden_turnaus(siemen: u64) -> (u32, usize) {
    let mut arpa = Arpa::new(siemen);
    let mut p = Poyta::new(&mut arpa);
    let mut floppeja = 0;
    while p.jaa(&mut arpa) {
        assert!(p.jako < 5000, "peli ei pääty, siemen {siemen}");
        let mut askelia = 0;
        let mut flopissa = false;
        while p.kulku != Kulku::Ohi {
            assert_eq!(rahat_yhteensa(&p), 5 * ALKURAHA, "{siemen}");
            if let (Kulku::Panostus, Some(i)) = (p.kulku, p.vuorossa) {
                let toimi = p.koneen_paatos(i, &mut arpa);
                assert!(p.toimi(toimi), "laiton {toimi:?}, siemen {siemen}");
            } else {
                p.etene(&mut arpa);
            }
            flopissa |= p.yhteiset.len() >= 3;
            askelia += 1;
            assert!(askelia < 500, "jako ei pääty, siemen {siemen}");
        }
        floppeja += flopissa as usize;
        assert_eq!(p.pelaajat.iter().map(|x| x.rahat).sum::<u32>(), 5000);
    }
    assert_eq!(p.rahoissa(), 1);
    (p.jako, floppeja)
}

/// Debug-käännöksessä koneen arviot ovat hitaita, joten koko turnauksia ajetaan vähän.
#[test]
fn koneiden_turnaus_paattyy_rahat_tasmaavat_ja_koneet_pelaavat_kasia() {
    let (mut jakoja, mut floppeja) = (0, 0);
    for siemen in 0..2 {
        let (j, f) = koneiden_turnaus(siemen);
        jakoja += j as usize;
        floppeja += f;
    }
    assert!(floppeja * 3 >= jakoja, "{floppeja} floppia, {jakoja} jakoa");
}

/// Laillinen satunnainen teko: mukana myös pienimmät ja suurimmat korotukset.
fn satunnainen_toimi(p: &Poyta, i: usize, arpa: &mut Arpa) -> Toimi {
    match arpa.alle(5) {
        0 => Toimi::Kippaa,
        1 | 2 => Toimi::Maksa,
        _ if !p.voi_korottaa(i) => Toimi::Maksa,
        3 => Toimi::Korota(p.pienin_taso(i)),
        _ => {
            let (a, b) = (p.pienin_taso(i), p.suurin_taso(i));
            Toimi::Korota(a + arpa.alle((b - a + 1) as usize) as u32)
        }
    }
}

#[test]
fn satunnaisilla_teoilla_saannot_pitavat_kaikissa_tilanteissa() {
    for siemen in 0..400 {
        let mut arpa = Arpa::new(siemen);
        let mut p = Poyta::new(&mut arpa);
        while p.jaa(&mut arpa) {
            assert!(p.jako < 5000, "{siemen}");
            // iso blindi on aina mukana, eikä nappi voi olla sama kuin iso
            assert_eq!(p.pelaajat[p.iso].tila, Tila::Mukana);
            assert_ne!(p.nappi, p.iso, "{siemen}");
            let mut askelia = 0;
            while p.kulku != Kulku::Ohi {
                assert_eq!(rahat_yhteensa(&p), 5 * ALKURAHA, "{siemen}");
                match (p.kulku, p.vuorossa) {
                    (Kulku::Panostus, Some(i)) => {
                        assert!(p.pelaajat[i].rahat > 0, "{siemen}");
                        assert_eq!(p.pelaajat[i].tila, Tila::Mukana);
                        let toimi = satunnainen_toimi(&p, i, &mut arpa);
                        assert!(p.toimi(toimi), "{toimi:?}, siemen {siemen}");
                    }
                    (Kulku::Panostus, None) => panic!("vuoro puuttuu, siemen {siemen}"),
                    _ => p.etene(&mut arpa),
                }
                if p.kulku != Kulku::Ohi {
                    let potit: u32 = p.potit().iter().map(|x| x.summa).sum();
                    assert_eq!(potit, p.potti(), "{siemen}");
                }
                askelia += 1;
                assert!(askelia < 500, "{siemen}");
            }
            assert_eq!(
                p.pelaajat.iter().map(|x| x.rahat).sum::<u32>(),
                5 * ALKURAHA,
                "{siemen}"
            );
        }
        assert_eq!(p.rahoissa(), 1, "{siemen}");
    }
}

#[test]
fn kone_ei_nae_muiden_kortteja_eika_pakkaa() {
    let tee = |vastustajan: &str, pakan: &str| {
        let mut p = jako_korteilla(
            TAYSI,
            ["2c 3d", "4d 5c", vastustajan, "Qh Jh", "Tc 9c"],
            "As Ks Qs Js 9s",
        );
        // sama ylin kortti, vaikka muu pakka muuttuisi
        p.pakka = kortit(pakan);
        p
    };
    for siemen in 0..50 {
        let a = tee("Ad Kd", "2h 3h 4h");
        let b = tee("7h 7d", "8h 8d 6c");
        let toimi_a = a.koneen_paatos(a.vuorossa.unwrap(), &mut Arpa::new(siemen));
        let toimi_b = b.koneen_paatos(b.vuorossa.unwrap(), &mut Arpa::new(siemen));
        assert_eq!(toimi_a, toimi_b, "{siemen}");
    }
}

#[test]
fn voittotodennakoisyys_vastaa_tunnettuja_kasia() {
    let mut arpa = Arpa::new(11);
    let aloita = |kasi: &str| {
        jako_korteilla(
            TAYSI,
            ["2c 3d", "4d 5c", "6h 6d", kasi, "Tc 9c"],
            "2h 3h 4h 5h 6c",
        )
    };
    let assat = aloita("As Ad");
    let huono = aloita("7c 2d");
    let (a, b) = (
        assat.voittotodennakoisyys(3, 1, 3000, &mut arpa),
        huono.voittotodennakoisyys(3, 1, 3000, &mut arpa),
    );
    // Tunnetut arvot kaksinpelissä: ässät noin 85 %, 7-2 noin 35 %.
    assert!((0.80..0.90).contains(&a), "{a}");
    assert!((0.30..0.40).contains(&b), "{b}");
}

#[test]
fn kone_kippaa_heikon_kaden_valtavaa_panosta_vastaan_ja_ei_vahvaa() {
    let (mut vahva_kippasi, mut heikko_maksoi) = (0, 0);
    for siemen in 0..40 {
        let mut arpa = Arpa::new(siemen);
        let mut p = jako_korteilla(
            TAYSI,
            ["2c 3d", "4d 5c", "6h 6d", "Tc 9s", "As Ad"],
            "2h 3h 4h 5h 6c",
        );
        assert!(p.toimi(Toimi::Korota(1000)));
        if p.koneen_paatos(4, &mut arpa) == Toimi::Kippaa {
            vahva_kippasi += 1;
        }
        let mut p = jako_korteilla(
            TAYSI,
            ["2c 3d", "4d 5c", "6h 6d", "Tc 9s", "7s 2d"],
            "2h 3h 4h 5h 6c",
        );
        assert!(p.toimi(Toimi::Korota(1000)));
        if p.koneen_paatos(4, &mut arpa) != Toimi::Kippaa {
            heikko_maksoi += 1;
        }
    }
    assert_eq!(vahva_kippasi, 0);
    assert_eq!(heikko_maksoi, 0);
}

#[test]
fn tulos_on_oma_voitto_alkurahoihin_nahden() {
    use klassikot8000::holdem::{ALKURAHA, PELAAJA, PELAAJIA};
    let mut poyta = klassikot8000::holdem::Poyta::new(&mut klassikot8000::Arpa::new(1));
    assert_eq!(poyta.pisteet(), 0);
    poyta.pelaajat[PELAAJA].rahat = ALKURAHA - 1;
    assert_eq!(poyta.pisteet(), 0);
    poyta.pelaajat[PELAAJA].rahat = ALKURAHA * PELAAJIA as u32;
    assert_eq!(poyta.pisteet(), ALKURAHA * (PELAAJIA as u32 - 1));
}
