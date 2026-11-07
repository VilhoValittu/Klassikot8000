use super::*;
use crate::yhteinen::kieli::kaikilla_kielilla;
use crate::yhteinen::syote::Syote;
use klassikot8000::Arpa;
use klassikot8000::kaivo::{ALKU, PINTA, Putoava};

pub fn vaikeuden_keskipiste(vaikeus: Vaikeus) -> (i32, i32) {
    let (_, teksti, x) = TASOT.iter().find(|t| t.0 == vaikeus).copied().unwrap();
    (x + napin_leveys(teksti) / 2, TASO_Y + NAPPI_K / 2)
}

pub fn ruudun_keskipiste((x, y): Ruutu) -> (i32, i32) {
    (
        KENTTA_X + x * SOLU + SOLU / 2,
        KENTTA_Y + y * SOLU + SOLU / 2,
    )
}

#[test]
fn tasot_loytyvat_eivatka_mene_paallekkain() {
    for taso in Vaikeus::KAIKKI {
        let (x, y) = vaikeuden_keskipiste(taso);
        assert_eq!(vaikeus_pisteesta(x, y), Some(taso));
    }
    kaikilla_kielilla(|| {
        for pari in TASOT.windows(2) {
            assert!(pari[0].2 + napin_leveys(pari[0].1) < pari[1].2);
        }
        let viimeinen = TASOT[2];
        assert!(viimeinen.2 + napin_leveys(viimeinen.1) < LEVEYS as i32 - 24);
    });
}

#[test]
fn ruudut_loytyvat_pisteista() {
    for ruutu in [(0, 0), ALKU, (SARAKKEITA as i32 - 1, RIVEJA as i32 - 1)] {
        let (x, y) = ruudun_keskipiste(ruutu);
        assert_eq!(ruutu_pisteesta(x, y), Some(ruutu));
    }
    assert_eq!(ruutu_pisteesta(KENTTA_X - 1, KENTTA_Y), None);
    assert_eq!(ruutu_pisteesta(KENTTA_X + KENTTA_L, KENTTA_Y), None);
}

#[test]
fn tekstit_mahtuvat_paikoilleen() {
    kaikilla_kielilla(|| {
        for teksti in [
            kaanna(OHJE).to_string(),
            kaanna(TAUON_OHJE).to_string(),
            muotoile("kaivo.kulta", &[&9999]),
            muotoile("kaivo.timantti", &[&9999]),
            kaanna("kaivo.oljy").to_string(),
            muotoile("kaivo.puut", &[&4]),
            kaanna("kaivo.kiintio_taynna").to_string(),
            muotoile("kaivo.murskautui", &[&99999, &2]),
            muotoile("kaivo.putosi", &[&2]),
            muotoile("kaivo.koysi", &[&2]),
            muotoile("kaivo.nosto", &[&99999, &99999]),
            muotoile("kaivo.taso_valmis", &[&99999]),
            muotoile("kaivo.uusi_taso", &[&99]),
            kaanna("kaivo.oljy_loppui").to_string(),
            kaanna("kaivo.ei_tukia").to_string(),
            kaanna("kaivo.ei_tikkaita").to_string(),
            muotoile("yhteinen.tulos", &[&999999]),
        ] {
            assert!(teksti.chars().count() <= RIVILLE_MERKKEJA, "{teksti}");
        }
        for avain in [
            "yhteinen.pisteet",
            "yhteinen.paras",
            "kaivo.taso_otsikko",
            "kaivo.kiintio",
            "kaivo.sakki",
            "kaivo.tuet",
            "kaivo.tikkaat",
            "kaivo.kyparat",
            "kaivo.lamppu",
        ] {
            let leveys = kaanna(avain).chars().count() as i32 * OTSIKKO.2;
            assert!(leveys <= PARHAAT_L, "{avain}");
        }
        // Tukien ja tikkaiden nimen perään mahtuu välin kanssa kaksinumeroinen määrä: tikkaita on
        // enintään 3 * 20 ja tukia 20 ja kolme nippua.
        assert!(Vaikeus::Helppo.tikkaita() < 100);
        for avain in ["kaivo.tuet", "kaivo.tikkaat"] {
            let leveys = (kaanna(avain).chars().count() as i32 + 1 + 2) * OTSIKKO.2;
            assert!(leveys <= PARHAAT_L, "{avain}");
        }
        let otsikko = kaanna("kaivo.valmis_otsikko").chars().count() as i32 * OTSIKKO.2;
        assert!(otsikko <= 520);
        assert!(muotoile("kaivo.seuraava", &[&99]).chars().count() as i32 * MERKKI <= 520);
    });
    let rivit: Vec<&str> = ALKU_RIVIT
        .iter()
        .copied()
        .chain(Vaikeus::KAIKKI.map(tason_kuvaus).into_iter().take(1))
        .collect();
    alkuohje_mahtuu(&ALKU_OSAT, &rivit, TASO_Y);
    kaikilla_kielilla(|| {
        for taso in Vaikeus::KAIKKI {
            assert!(kaanna(tason_kuvaus(taso)).chars().count() <= 36);
        }
    });
    const { assert!((2 + 8 + 1 + 6) * PARHAAT_KOKO.2 <= PARHAAT_L) };
    const { assert!(PARHAAT_Y + PARHAAT_OTSIKKO + 3 * PARHAAT_RIVI < PANEELI_Y) };
    const { assert!(KENTTA_Y + KENTTA_K < PANEELI_Y) };
    const { assert!(SARAKE_X + PARHAAT_L <= LEVEYS as i32) };
    const { assert!(KYPARIA as i32 * 33 <= PARHAAT_L) };
}

#[test]
fn lamppu_valaisee_lahelta_eika_kaukaa_ja_taivas_on_aina_valossa() {
    let mut peli = Kaivo::new(1);
    peli.kasittele(Syote::Ammu);
    peli.kaivos.kaivaja.paikka = (10, 10);
    let lamppu = Lamppu::new(&peli);
    let (gx, gy) = (10 * SOLU_P + 5, 10 * SOLU_P + 4);
    assert_eq!(lamppu.osuus(gx, gy), 1.0);
    assert_eq!(lamppu.osuus(gx + 9 * SOLU_P, gy), 0.0);
    assert_eq!(lamppu.osuus(0, 0), 1.0);
    // Öljyn loputtua valo kutistuu.
    let ennen = lamppu.osuus(gx + 3 * SOLU_P, gy);
    peli.kaivos.oljy_ms = 0;
    let himmea = Lamppu::new(&peli);
    assert!(himmea.osuus(gx + 3 * SOLU_P, gy) < ennen);
}

#[test]
fn kuvan_pikselit_tuntevat_materiaalit() {
    let mut peli = Kaivo::new(1);
    peli.kasittele(Syote::Ammu);
    let k = &mut peli.kaivos;
    k.aseta((5, 5), Solu::Tyhja);
    k.aseta((6, 5), Solu::Tuki);
    k.aseta((7, 5), Solu::Kivi);
    k.aseta((8, 5), Solu::Tikkaat);
    let keski = |x: i32, y: i32| (x * SOLU_P + 5, y * SOLU_P + 5);
    let seina = pikselin_vari(k, keski(5, 5).0, keski(5, 5).1);
    assert_eq!(seina, {
        let (gx, gy) = keski(5, 5);
        seinan_vari(5, gx, gy)
    });
    // Tuen pylväs ja kiven keskus ovat omaa väriään, tuen keskellä näkyy seinä.
    assert_eq!(pikselin_vari(k, 6 * SOLU_P + 1, 5 * SOLU_P + 4), 0xC8904C);
    let (gx, gy) = (6 * SOLU_P + 4, 5 * SOLU_P + 3);
    assert_eq!(pikselin_vari(k, gx, gy), seinan_vari(5, gx, gy));
    // Tikkaiden johde ja puola ovat omaa väriään, puolien välissä näkyy seinä.
    assert_eq!(pikselin_vari(k, 8 * SOLU_P + 2, 5 * SOLU_P + 5), 0xE8C888);
    assert_eq!(pikselin_vari(k, 8 * SOLU_P + 5, 5 * SOLU_P + 2), 0xD0A868);
    let (gx, gy) = (8 * SOLU_P + 5, 5 * SOLU_P + 5);
    assert_eq!(pikselin_vari(k, gx, gy), seinan_vari(5, gx, gy));
    assert_ne!(
        pikselin_vari(k, keski(7, 5).0, keski(7, 5).1),
        seinan_vari(5, 0, 0)
    );
    // Taivas ei pimene.
    let kuva = Kaivoskuva::new(k);
    assert_eq!(kuva.valo.rivi(0), kuva.pimea.rivi(0));
    assert_ne!(kuva.valo.rivi(100), kuva.pimea.rivi(100));
    assert_eq!(kuva.versio, k.versio);
}

#[test]
fn koko_peli_piirtyy_kaatumatta() {
    let mut kuva = vec![0; LEVEYS * KORKEUS];
    for taso in Vaikeus::KAIKKI {
        let mut peli = Kaivo::new(3);
        peli.vaikeus = taso;
        piirra(&peli, &mut kuva);
        peli.kasittele(Syote::Ammu);
        peli.kaivos.putoavat.push(Putoava {
            solu: Solu::Kivi,
            paikka: (10, 8),
            ms: 30,
        });
        peli.kaivos.saalis = 50;
        for i in 0..600 {
            // Kaivaja kaivaa vuorotellen alas ja sivuille.
            if i % 20 == 0 {
                let suunnat = [(0, 1), (1, 0), (0, 1), (-1, 0)];
                peli.kaivos.paina(suunnat[(i / 40 % 4) as usize]);
            }
            peli.edista(16);
            if i % 7 == 0 {
                piirra(&peli, &mut kuva);
            }
        }
        peli.kaivos.oljy_ms = 0;
        piirra(&peli, &mut kuva);
        peli.kaivos.kaivaja.paikka = (HISSI.0 + 1, PINTA);
        peli.kaivos.kaivaja.toimi = klassikot8000::kaivo::Toimi::Seisoo;
        peli.kaivos.kyparat = KYPARIA;
        peli.kaivos.saalis = 10;
        peli.kaivos.kiintio = 1;
        peli.kaivos.paina((-1, 0));
        for _ in 0..15 {
            peli.edista(20);
        }
        assert!(peli.kaivos.hissi_ms.is_some());
        for _ in 0..30 {
            peli.edista(100);
            piirra(&peli, &mut kuva);
        }
        assert_eq!(peli.kaivos.taso, 1);
        peli.kaivos.askel(&mut Arpa::new(1));
        for vaihe in [Vaihe::Tauko, Vaihe::Tappio, Vaihe::Nimi] {
            peli.vaihe = vaihe;
            piirra(&peli, &mut kuva);
        }
    }
}
