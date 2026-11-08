use std::f32::consts::PI;

use klassikot8000::Arpa;
use klassikot8000::majakka::{
    AAMUN_TAUKO_MS, ASKEL_MS, ENITEN_KARIKOITA, KEILAN_KULMAT, KIINNIOTTO_MS, Kari, LEVEYS,
    LUOTSIN_SADE, Laiva, Laji, MAJAKKA, Meri, Ohjaus, RENKAITA, SATAMA, SUURIN_PISTEMAARA,
    TORVEN_HIDASTUS, TORVEN_KESTO_MS, TULO_LOPPUU_MS, Tapahtuma, ULAPPA, VAYLA, Vaikeus,
    Vuorokausi, YON_KESTO_MS, etaisyys, karikko_kelpaa, karikot, lahella_satamaa, ranta_y, suulla,
    suunta_majakalta,
};

/// Tyhjä, tyyni meri ilman uusia laivoja: testi päättää itse, mitä merellä on. Yö on niin
/// pitkällä, ettei laivoja enää tule, ja aamu sarastaa 15 sekunnin päästä.
fn tyyni(vaikeus: Vaikeus) -> (Meri, Arpa) {
    let mut arpa = Arpa::new(5);
    let mut meri = Meri::new(vaikeus, 0, &mut arpa);
    meri.karikot.clear();
    meri.virta = (0.0, 0.0);
    meri.aika_ms = TULO_LOPPUU_MS;
    (meri, arpa)
}

/// Askeleet ilman ohjausta, uusia laivoja odottamatta. Palauttaa tapahtumat.
fn aja(meri: &mut Meri, arpa: &mut Arpa, ms: u32, ohjaus: Ohjaus) -> Vec<Tapahtuma> {
    let mut kaikki = Vec::new();
    for _ in 0..ms / ASKEL_MS {
        kaikki.extend(meri.askel(ohjaus, arpa));
    }
    kaikki
}

/// Keila, joka valaisee koko meren.
fn kaikki_valossa(meri: &mut Meri) {
    meri.keila.puolikas = PI;
    meri.keila.kantama = 5000.0;
}

#[test]
fn karikot_jattavat_vaylat_ja_sataman_eteen_tilaa() {
    for siemen in 0..60 {
        let mut arpa = Arpa::new(siemen);
        for vaikeus in Vaikeus::KAIKKI {
            for yo in [0, 3, 10] {
                let karikot = karikot(vaikeus, yo, &mut arpa);
                assert_eq!(karikot.len(), vaikeus.karikoita(yo), "{siemen} {vaikeus:?}");
                assert!(karikot.len() <= ENITEN_KARIKOITA);
                for (i, kari) in karikot.iter().enumerate() {
                    assert!(karikko_kelpaa(kari, &karikot[..i]));
                    assert!(etaisyys(kari.paikka, ULAPPA) > 70.0);
                    assert!(kari.paikka.0 > 40.0 && kari.paikka.0 < LEVEYS - 40.0);
                    assert!(kari.paikka.1 + kari.sade < ranta_y(kari.paikka.0) - 40.0);
                    for toinen in &karikot[i + 1..] {
                        let vali = etaisyys(kari.paikka, toinen.paikka) - kari.sade - toinen.sade;
                        assert!(vali >= VAYLA);
                    }
                }
            }
        }
    }
}

#[test]
fn ranta_satama_ja_majakka_ovat_paikoillaan() {
    assert!(suulla(SATAMA.0) && !suulla(SATAMA.0 - 60.0) && !suulla(MAJAKKA.0));
    assert!(MAJAKKA.1 < ranta_y(MAJAKKA.0));
    assert!(ULAPPA.1 < ranta_y(ULAPPA.0 - 50.0) - 40.0);
    for x in (0..LEVEYS as i32).step_by(10) {
        let y = ranta_y(x as f32);
        assert!((470.0..=515.0).contains(&y), "{x}: {y}");
    }
}

#[test]
fn keila_kaantyy_rajatulla_nopeudella_ja_pysyy_merella() {
    for vaikeus in Vaikeus::KAIKKI {
        let (mut meri, mut arpa) = tyyni(vaikeus);
        let alku = meri.keila.kulma;
        let ohjaus = Ohjaus {
            kaanto: 1,
            tahtain: None,
        };
        aja(&mut meri, &mut arpa, 100, ohjaus);
        let odotettu = alku + vaikeus.kaantonopeus() * 0.1;
        assert!((meri.keila.kulma - odotettu).abs() < 0.001, "{vaikeus:?}");
        aja(&mut meri, &mut arpa, 5000, ohjaus);
        assert_eq!(meri.keila.kulma, KEILAN_KULMAT.1);

        // Tähtäin ei mene yli, ja maalle osoittava hiiri jää lähimpään reunaan.
        let tahtain = Some(-2.0);
        aja(&mut meri, &mut arpa, 5000, Ohjaus { kaanto: 0, tahtain });
        assert!((meri.keila.kulma + 2.0).abs() < 0.001);
    }
    assert_eq!(suunta_majakalta((0.0, MAJAKKA.1 + 300.0)), KEILAN_KULMAT.0);
    assert_eq!(
        suunta_majakalta((LEVEYS, MAJAKKA.1 + 300.0)),
        KEILAN_KULMAT.1
    );
    assert!((suunta_majakalta((MAJAKKA.0, 0.0)) + PI / 2.0).abs() < 0.001);
}

#[test]
fn keila_ylettyy_rannan_ja_satamansuun_laivoihin() {
    let (mut meri, _) = tyyni(Vaikeus::Vaikea);
    let rannalla = |x: f32| (x, ranta_y(x) - 10.0);
    let suulla = [SATAMA.0 - 30.0, SATAMA.0, SATAMA.0 + 30.0].map(|x| (x, SATAMA.1 - 12.0));
    let rannat = [20.0, 120.0, 220.0, 420.0, 520.0, 640.0].map(rannalla);
    for p in suulla.into_iter().chain(rannat) {
        meri.keila.kulma = suunta_majakalta(p);
        assert!(meri.keila.valaisee(p, Laji::Vene.sade()), "{p:?}");
    }
}

#[test]
fn keila_valaisee_vain_suuntaansa_ja_kantamaansa() {
    let (meri, _) = tyyni(Vaikeus::Vaikea);
    let keila = meri.keila;
    assert!(keila.valaisee((MAJAKKA.0, MAJAKKA.1 - 300.0), 6.0));
    assert!(!keila.valaisee((MAJAKKA.0 - 300.0, MAJAKKA.1 - 20.0), 6.0));
    assert!(!keila.valaisee((MAJAKKA.0, MAJAKKA.1 - 600.0), 6.0));
    // Iso laiva keilan reunan vieressä valaistuu kyljestään.
    let reuna = -PI / 2.0 + keila.puolikas + 0.04;
    let p = (
        MAJAKKA.0 + reuna.cos() * 200.0,
        MAJAKKA.1 + reuna.sin() * 200.0,
    );
    assert!(keila.valaisee(p, 12.0));
    assert!(!keila.valaisee(p, 1.0));
}

#[test]
fn pimeassa_laiva_pitaa_suuntansa_ja_valossa_kaantyy_satamaan() {
    let (mut meri, mut arpa) = tyyni(Vaikeus::Helppo);
    meri.keila.kantama = 0.0;
    meri.laivat = vec![Laiva::uusi(Laji::Kalastaja, (100.0, 100.0), 0.0)];
    aja(&mut meri, &mut arpa, 2000, Ohjaus::default());
    let laiva = meri.laivat[0];
    assert_eq!(laiva.suunta, 0.0);
    assert!((laiva.paikka.0 - (100.0 + Laji::Kalastaja.vauhti() * 2.0)).abs() < 0.5);
    assert!((laiva.paikka.1 - 100.0).abs() < 0.001);
    assert!(!laiva.valaistu);

    kaikki_valossa(&mut meri);
    aja(&mut meri, &mut arpa, 3000, Ohjaus::default());
    let laiva = meri.laivat[0];
    assert!(laiva.valaistu && laiva.tietaa());
    let kohti = (ULAPPA.1 - laiva.paikka.1).atan2(ULAPPA.0 - laiva.paikka.0);
    assert!((laiva.suunta - kohti).abs() < 0.2);

    // Valon sammuttua kapteeni muistaa reitin vielä hetken.
    meri.keila.kantama = 0.0;
    aja(
        &mut meri,
        &mut arpa,
        Vaikeus::Helppo.opastus_ms() - 100,
        Ohjaus::default(),
    );
    assert!(meri.laivat[0].tietaa());
    aja(&mut meri, &mut arpa, 200, Ohjaus::default());
    assert!(!meri.laivat[0].tietaa());
}

#[test]
fn sataman_lahella_laiva_loytaa_perille_pimeassakin() {
    let (mut meri, mut arpa) = tyyni(Vaikeus::Vaikea);
    meri.keila.kantama = 0.0;
    // Kummaltakin puolelta suuta suoraan kohti rantaa.
    for (laji, paikka) in [
        (Laji::Vene, (SATAMA.0 - 80.0, SATAMA.1 - 60.0)),
        (Laji::Rahtilaiva, (SATAMA.0 + 70.0, SATAMA.1 - 75.0)),
        (Laji::Kalastaja, (SATAMA.0, SATAMA.1 - LUOTSIN_SADE + 5.0)),
    ] {
        assert!(lahella_satamaa(paikka));
        meri.aika_ms = TULO_LOPPUU_MS;
        meri.laivat = vec![Laiva::uusi(laji, paikka, PI / 2.0)];
        assert!(meri.laivat[0].tietaa() && !meri.laivat[0].valaistu);
        let mut tapahtumat = aja(&mut meri, &mut arpa, 12_000, Ohjaus::default());
        assert_eq!(meri.vuorokausi, Vuorokausi::Yo);
        tapahtumat.retain(|t| *t != Tapahtuma::Puuska);
        assert!(
            matches!(tapahtumat[..], [Tapahtuma::Satamassa { laji: l, .. }] if l == laji),
            "{laji:?}: {tapahtumat:?}"
        );
    }
    // Kauempana pimeässä ei ole luotsia.
    let kaukana = (SATAMA.0 - 200.0, SATAMA.1 - 60.0);
    assert!(!lahella_satamaa(kaukana));
    assert!(!Laiva::uusi(Laji::Vene, kaukana, PI / 2.0).tietaa());
}

#[test]
fn valaistu_meri_tuo_jokaisen_laivan_satamaan() {
    for vaikeus in Vaikeus::KAIKKI {
        for siemen in 0..12 {
            let mut arpa = Arpa::new(siemen);
            let yo = (siemen % 6) as u32;
            let mut meri = Meri::new(vaikeus, yo, &mut arpa);
            let mut satamassa = 0;
            while meri.vuorokausi == Vuorokausi::Yo {
                kaikki_valossa(&mut meri);
                for tapahtuma in meri.askel(Ohjaus::default(), &mut arpa) {
                    match tapahtuma {
                        Tapahtuma::Satamassa { .. } => satamassa += 1,
                        Tapahtuma::Karille { laji, paikka } => {
                            panic!(
                                "{vaikeus:?} siemen {siemen} yö {yo}: {laji:?} karille {paikka:?}"
                            )
                        }
                        Tapahtuma::Eksyi { laji } => {
                            panic!("{vaikeus:?} siemen {siemen} yö {yo}: {laji:?} eksyi")
                        }
                        _ => {}
                    }
                }
            }
            assert!(satamassa >= 5, "{vaikeus:?} {siemen}: {satamassa}");
            assert_eq!(meri.renkaat, RENKAITA);
        }
    }
}

#[test]
fn karille_ajo_vie_renkaan_ja_nollaa_ketjun() {
    let (mut meri, mut arpa) = tyyni(Vaikeus::Normaali);
    meri.ketju = 5;
    meri.karikot = vec![Kari {
        paikka: (200.0, 200.0),
        sade: 15.0,
    }];
    for kerta in 1..=RENKAITA {
        meri.laivat = vec![Laiva::uusi(Laji::Vene, (170.0, 200.0), 0.0)];
        let tapahtumat = aja(&mut meri, &mut arpa, 2000, Ohjaus::default());
        assert!(matches!(
            tapahtumat[0],
            Tapahtuma::Karille {
                laji: Laji::Vene,
                ..
            }
        ));
        assert_eq!(meri.renkaat, RENKAITA - kerta);
        assert_eq!(meri.ketju, 0);
        assert_eq!(meri.rikkoja_yolla, kerta);
        let ohi = tapahtumat.contains(&Tapahtuma::Ohi);
        assert_eq!(ohi, kerta == RENKAITA);
        assert_eq!(meri.ohi, ohi);
    }
    // Päättynyt peli ei enää etene.
    assert!(meri.askel(Ohjaus::default(), &mut arpa).is_empty());
}

#[test]
fn rantaan_ajo_on_haaksirikko_ja_reunan_yli_eksyminen() {
    let (mut meri, mut arpa) = tyyni(Vaikeus::Helppo);
    meri.keila.kantama = 0.0;
    meri.laivat = vec![Laiva::uusi(Laji::Vene, (100.0, 450.0), PI / 2.0)];
    let tapahtumat = aja(&mut meri, &mut arpa, 5000, Ohjaus::default());
    assert!(matches!(tapahtumat[0], Tapahtuma::Karille { .. }));

    meri.ketju = 3;
    meri.laivat = vec![Laiva::uusi(Laji::Vene, (100.0, 100.0), PI)];
    let tapahtumat = aja(&mut meri, &mut arpa, 5000, Ohjaus::default());
    assert_eq!(tapahtumat, [Tapahtuma::Eksyi { laji: Laji::Vene }]);
    assert_eq!(meri.ketju, 0);
    assert_eq!(meri.renkaat, RENKAITA - 1);
}

#[test]
fn reunalta_tuleva_laiva_ei_eksy_ennen_kuin_on_kentalla() {
    let (mut meri, mut arpa) = tyyni(Vaikeus::Helppo);
    meri.keila.kantama = 0.0;
    meri.laivat = vec![Laiva::uusi(Laji::Rahtilaiva, (300.0, -8.0), PI / 2.0)];
    let tapahtumat = aja(&mut meri, &mut arpa, 1000, Ohjaus::default());
    assert!(tapahtumat.is_empty());
    assert_eq!(meri.laivat.len(), 1);
}

#[test]
fn satamaan_paasy_tuo_pisteet_ketjukertoimella() {
    let (mut meri, mut arpa) = tyyni(Vaikeus::Vaikea);
    let mut odotetut = 0;
    for i in 0..7 {
        let kerroin = 1 + i / 3;
        assert_eq!(meri.ketjukerroin(), kerroin);
        meri.laivat = vec![Laiva::uusi(
            Laji::Rahtilaiva,
            (SATAMA.0, SATAMA.1 - 10.0),
            PI / 2.0,
        )];
        let tapahtumat = aja(&mut meri, &mut arpa, 200, Ohjaus::default());
        let pisteet = 50 * kerroin * 3;
        odotetut += pisteet;
        assert!(tapahtumat.contains(&Tapahtuma::Satamassa {
            laji: Laji::Rahtilaiva,
            pisteet
        }));
    }
    assert_eq!(meri.pisteet, odotetut);
    assert_eq!(meri.satamassa_yolla, 7);
    meri.ketju = 1000;
    assert_eq!(meri.ketjukerroin(), 5);
}

#[test]
fn pisteet_eivat_ylita_rajaa() {
    let (mut meri, mut arpa) = tyyni(Vaikeus::Vaikea);
    meri.pisteet = SUURIN_PISTEMAARA - 10;
    meri.laivat = vec![Laiva::uusi(
        Laji::Vene,
        (SATAMA.0, SATAMA.1 - 10.0),
        PI / 2.0,
    )];
    aja(&mut meri, &mut arpa, 500, Ohjaus::default());
    assert_eq!(meri.pisteet, SUURIN_PISTEMAARA);
}

#[test]
fn salakuljettaja_jaa_kiinni_keilassa_mutta_ei_paivanvalossa() {
    let (mut meri, mut arpa) = tyyni(Vaikeus::Normaali);
    let alku = (MAJAKKA.0, 150.0);
    meri.laivat = vec![Laiva::salakuljettaja(alku, (MAJAKKA.0, 140.0))];
    meri.laivat[0].kohde = (-100.0, 150.0);
    meri.laivat[0].suunta = PI;
    // Keila seuraa salakuljettajaa.
    let mut tapahtumat = Vec::new();
    for _ in 0..(KIINNIOTTO_MS + 500) / ASKEL_MS {
        let tahtain = meri.laivat.first().map(|l| suunta_majakalta(l.paikka));
        tapahtumat.extend(meri.askel(Ohjaus { kaanto: 0, tahtain }, &mut arpa));
    }
    let pisteet = Laji::Salakuljettaja.pisteet() * 2;
    assert!(matches!(tapahtumat[0], Tapahtuma::Kiinni { pisteet: p, .. } if p == pisteet));
    assert_eq!(meri.pisteet, pisteet);
    assert!(meri.laivat.is_empty());

    // Aamun valossa salakuljettaja näkyy, mutta pakenee.
    meri.vuorokausi = Vuorokausi::Sarastus;
    meri.aika_ms = 0;
    meri.keila.kantama = 0.0;
    meri.laivat = vec![Laiva::salakuljettaja((600.0, 150.0), (700.0, 150.0))];
    let tapahtumat = aja(&mut meri, &mut arpa, 5000, Ohjaus::default());
    assert!(tapahtumat.contains(&Tapahtuma::Karkasi));
    assert_eq!(meri.pisteet, pisteet);
}

#[test]
fn salakuljettaja_ei_aja_karille() {
    let (mut meri, mut arpa) = tyyni(Vaikeus::Vaikea);
    meri.keila.kantama = 0.0;
    meri.karikot = vec![Kari {
        paikka: (330.0, 200.0),
        sade: 18.0,
    }];
    meri.laivat = vec![Laiva::salakuljettaja((-8.0, 200.0), (690.0, 200.0))];
    let tapahtumat = aja(&mut meri, &mut arpa, 30_000, Ohjaus::default());
    assert!(tapahtumat.contains(&Tapahtuma::Karkasi));
    assert!(
        !tapahtumat
            .iter()
            .any(|t| matches!(t, Tapahtuma::Karille { .. }))
    );
}

#[test]
fn torvi_hidastaa_laivat_ja_latautuu() {
    let (mut meri, mut arpa) = tyyni(Vaikeus::Normaali);
    meri.keila.kantama = 0.0;
    meri.laivat = vec![Laiva::uusi(Laji::Vene, (100.0, 100.0), 0.0)];
    assert!(meri.soita_torvi());
    assert!(!meri.soita_torvi());
    aja(&mut meri, &mut arpa, 1000, Ohjaus::default());
    let matka = meri.laivat[0].paikka.0 - 100.0;
    assert!((matka - Laji::Vene.vauhti() * TORVEN_HIDASTUS).abs() < 0.2);
    aja(&mut meri, &mut arpa, TORVEN_KESTO_MS, Ohjaus::default());
    assert_eq!(meri.torvi_ms, 0);
    assert!(meri.torven_valmius() < 1.0);

    let mut latautui = false;
    for _ in 0..Vaikeus::Normaali.torven_lataus_ms() / ASKEL_MS {
        meri.laivat.clear();
        latautui |= meri
            .askel(Ohjaus::default(), &mut arpa)
            .contains(&Tapahtuma::TorviLatautui);
    }
    assert!(latautui);
    assert_eq!(meri.torven_valmius(), 1.0);
    assert!(meri.soita_torvi());
}

#[test]
fn yo_vaihtuu_aamuksi_ja_uusi_yo_tuo_lisaa_karikoita() {
    let mut arpa = Arpa::new(3);
    let mut meri = Meri::new(Vaikeus::Normaali, 0, &mut arpa);
    let karikoita = meri.karikot.len();
    meri.karikot.clear();
    let mut tapahtumat = Vec::new();
    for _ in 0..YON_KESTO_MS / ASKEL_MS {
        kaikki_valossa(&mut meri);
        tapahtumat.extend(meri.askel(Ohjaus::default(), &mut arpa));
    }
    assert_eq!(meri.vuorokausi, Vuorokausi::Sarastus);
    assert!(meri.valoisuus() < 0.01);
    assert!(meri.yon_osuus() >= 1.0);
    // Sarastuksessa merellä olevat laivat löytävät kotiin itsestään.
    meri.keila.kantama = 0.0;
    meri.keila.puolikas = 0.0;
    while meri.vuorokausi == Vuorokausi::Sarastus {
        tapahtumat.extend(meri.askel(Ohjaus::default(), &mut arpa));
    }
    let satamassa = tapahtumat
        .iter()
        .filter(|t| matches!(t, Tapahtuma::Satamassa { .. }))
        .count() as u32;
    assert!(satamassa > 5);
    let bonus = 100 * 2;
    assert_eq!(
        tapahtumat.last(),
        Some(&Tapahtuma::Aamu { satamassa, bonus })
    );
    assert_eq!(meri.valoisuus(), 1.0);
    assert!(!meri.soita_torvi());

    let pisteet = meri.pisteet;
    let tapahtumat = aja(&mut meri, &mut arpa, AAMUN_TAUKO_MS, Ohjaus::default());
    assert!(tapahtumat.contains(&Tapahtuma::UusiYo));
    assert_eq!(meri.yo, 1);
    assert_eq!(meri.vuorokausi, Vuorokausi::Yo);
    assert_eq!(meri.karikot.len(), karikoita + 1);
    assert_eq!((meri.satamassa_yolla, meri.rikkoja_yolla), (0, 0));
    assert_eq!(meri.pisteet, pisteet);
}

#[test]
fn haaksirikko_vie_aamun_bonuksen() {
    let (mut meri, mut arpa) = tyyni(Vaikeus::Helppo);
    meri.satamassa_yolla = 4;
    meri.rikkoja_yolla = 1;
    meri.vuorokausi = Vuorokausi::Sarastus;
    meri.aika_ms = 0;
    let tapahtumat = aja(&mut meri, &mut arpa, 3000, Ohjaus::default());
    assert!(tapahtumat.contains(&Tapahtuma::Aamu {
        satamassa: 4,
        bonus: 0
    }));
}

#[test]
fn laivoja_tulee_lisaa_mutta_ei_liikaa_ja_myohemmin_salakuljettajia() {
    for siemen in 0..10 {
        let mut arpa = Arpa::new(siemen);
        let mut meri = Meri::new(Vaikeus::Helppo, 2, &mut arpa);
        let mut salakuljettajia = 0;
        let mut eniten = 0;
        for _ in 0..60_000 / ASKEL_MS {
            meri.askel(Ohjaus::default(), &mut arpa);
            meri.renkaat = RENKAITA;
            let tavallisia = meri
                .laivat
                .iter()
                .filter(|l| l.laji != Laji::Salakuljettaja)
                .count();
            eniten = eniten.max(tavallisia);
            salakuljettajia += meri
                .laivat
                .iter()
                .filter(|l| l.laji == Laji::Salakuljettaja)
                .count();
        }
        assert!((2..=5).contains(&eniten), "{siemen}: {eniten}");
        assert!(salakuljettajia > 0, "{siemen}");
    }
    let mut arpa = Arpa::new(1);
    let mut meri = Meri::new(Vaikeus::Helppo, 0, &mut arpa);
    for _ in 0..60_000 / ASKEL_MS {
        meri.askel(Ohjaus::default(), &mut arpa);
        meri.renkaat = RENKAITA;
        assert!(meri.laivat.iter().all(|l| l.laji != Laji::Salakuljettaja));
    }
}

#[test]
fn tuuli_puhaltaa_vain_tuulisina_oina() {
    for (vaikeus, yo, tuulinen) in [
        (Vaikeus::Helppo, 5, false),
        (Vaikeus::Normaali, 0, false),
        (Vaikeus::Normaali, 2, true),
        (Vaikeus::Vaikea, 0, true),
    ] {
        let mut arpa = Arpa::new(7);
        let mut meri = Meri::new(vaikeus, yo, &mut arpa);
        let mut puuskia = 0;
        for _ in 0..40_000 / ASKEL_MS {
            meri.laivat.clear();
            puuskia += meri
                .askel(Ohjaus::default(), &mut arpa)
                .iter()
                .filter(|t| **t == Tapahtuma::Puuska)
                .count();
        }
        assert_eq!(puuskia > 0, tuulinen, "{vaikeus:?} {yo}");
        assert_eq!(meri.virta == (0.0, 0.0), vaikeus == Vaikeus::Helppo);
    }
}

#[test]
fn sama_siemen_ja_ohjaus_toistavat_saman_yon() {
    let pelaa = || {
        let mut arpa = Arpa::new(42);
        let mut meri = Meri::new(Vaikeus::Vaikea, 1, &mut arpa);
        let mut tapahtumat = Vec::new();
        for i in 0..30_000 / ASKEL_MS {
            let kaanto = if (i / 400) % 2 == 0 { 1 } else { -1 };
            let ohjaus = Ohjaus {
                kaanto,
                tahtain: None,
            };
            tapahtumat.extend(meri.askel(ohjaus, &mut arpa));
        }
        (tapahtumat, meri.pisteet, meri.laivat.len())
    };
    assert_eq!(pelaa(), pelaa());
}
