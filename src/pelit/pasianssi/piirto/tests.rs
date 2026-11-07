use super::*;
use crate::yhteinen::kieli::kaikilla_kielilla;
use klassikot8000::Arpa;

pub const SIVU_ALKAA: i32 = SIVU_X - 12;

pub fn haastenapin_keskipiste() -> (i32, i32) {
    (
        haastenapin_x() + napin_leveys(HAASTENAPPI) / 2,
        ALOITUS_Y + NAPPI_K / 2,
    )
}

pub fn vaikeuden_keskipiste(vaikeus: Vaikeus) -> (i32, i32) {
    let (_, teksti, x) = TASOT.iter().find(|t| t.0 == vaikeus).copied().unwrap();
    (x + napin_leveys(teksti) / 2, TASO_Y + NAPPI_K / 2)
}

pub fn variantin_keskipiste(variantti: Variantti) -> (i32, i32) {
    let (_, teksti, x) = VARIANTIT[variantti as usize];
    (x + napin_leveys(teksti) / 2, VARIANTTI_Y + NAPPI_K / 2)
}

pub fn napin_keskipiste(nappi: Nappi) -> (i32, i32) {
    napin_keskipiste_listasta(&NAPIT, nappi)
}

pub fn napin_keskipiste_listasta(napit: &[(Nappi, &str)], nappi: Nappi) -> (i32, i32) {
    let i = napit.iter().position(|n| n.0 == nappi).unwrap() as i32;
    (SIVU_TEKSTI_X + 12, NAPIT_Y + i * NAPPI_ASKEL + NAPPI_K / 2)
}

/// Sivupaneelin ohjeet ja napit mahtuvat tilaansa millä tahansa kielellä.
pub fn sivu_mahtuu(ohjeet: &[&str], napit: &[(Nappi, &str)]) {
    use crate::yhteinen::kieli::kaikilla_kielilla;
    let tila = SIVU_L - 36;
    assert!(SIVU_OHJE_Y + ohjeet.len() as i32 * RIVIN_KORKEUS <= NAPIT_Y);
    assert!(NAPIT_Y + napit.len() as i32 * NAPPI_ASKEL < RUUDUN_KORKEUS as i32);
    kaikilla_kielilla(|| {
        for avain in ohjeet {
            let rivi = kaanna(avain);
            assert!(rivi.chars().count() <= SIVU_MERKKEJA, "{rivi}");
        }
        for &(_, teksti) in napit {
            assert!(napin_leveys(teksti) <= tila, "{teksti}");
        }
    });
}

pub fn kortin_kulma_pakka() -> (i32, i32) {
    (sarakkeen_x(0), YLARIVI_Y)
}

#[test]
fn kortit_mahtuvat_ruudulle_pisimmassakin_sarakkeessa() {
    // Kuudes sarake: kuusi piilossa ja täysi jono kuninkaasta ässään.
    let alin = TAULU_Y + 6 * PIILO_ASKEL + 12 * NAKYVA_ASKEL + KORTTI_K;
    assert!(alin < RUUDUN_KORKEUS as i32);
    assert!(sarakkeen_x(SARAKKEITA - 1) + KORTTI_L < SIVU_X);
    assert!(SIVU_X + SIVU_L < RUUDUN_LEVEYS as i32);
    let viimeinen_nappi = NAPIT_Y + NAPIT.len() as i32 * NAPPI_ASKEL;
    assert!(viimeinen_nappi < RUUDUN_KORKEUS as i32);
}

#[test]
fn sivun_tekstit_mahtuvat_paneeliin() {
    assert!(SIVU_MERKKEJA as i32 * MERKKI <= SIVU_L - 36);
    sivu_mahtuu(&SIVU_OHJEET, &NAPIT);
    kaikilla_kielilla(|| {
        // Viesti rivitetään sivupalkkiin, jossa on tilaa neljälle riville.
        for avain in [
            "pasianssi.ei_kay",
            "pasianssi.pakka_loppu",
            "pasianssi.ei_peruutusta",
            "pasianssi.ei_peruttavaa",
            "pasianssi.ei_kerattavaa",
            "pasianssi.jumissa",
            "pasianssi.voittoviesti",
            "pasianssi.siirrot_loppu",
            "pyramidi.voittoviesti",
            "pyramidi.ei_paria",
        ] {
            assert!(rivita(kaanna(avain), SIVU_MERKKEJA).len() <= 4, "{avain}");
        }
        for avain in [
            "pasianssi.haaste_viesti",
            "pasianssi.haaste_voitto",
            "pasianssi.haaste_jumissa",
        ] {
            assert!(rivita(kaanna(avain), SIVU_MERKKEJA).len() <= 4, "{avain}");
        }
        assert!(napin_leveys(NAPPI_HAASTEET) <= napin_leveys("pasianssi.nappi_uusi"));
        // Kehote ja sen vilkkuva kursori jäävät haastenapin vasemmalle puolelle.
        let kehote = kaanna(PASIANSSI_ALOITUS).chars().count() as i32 + 1;
        assert!(48 + kehote * MERKKI < haastenapin_x());
        for pari in TASOT.windows(2) {
            assert!(pari[0].2 + napin_leveys(pari[0].1) < pari[1].2);
        }
        for pari in VARIANTIT.windows(2) {
            assert!(pari[0].2 + napin_leveys(pari[0].1) < pari[1].2);
        }
        let (_, viimeinen, x) = VARIANTIT[VARIANTIT.len() - 1];
        assert!(x + napin_leveys(viimeinen) < LEVEYS as i32);
        for laji in Variantti::KAIKKI {
            for taso in Vaikeus::KAIKKI {
                let kuvaus = kaanna(tason_kuvaus(laji, taso));
                assert!(kuvaus.chars().count() <= 36, "{kuvaus}");
            }
        }
        assert!(kaanna("pasianssi.laji").chars().count() as i32 * MERKKI < TASO_X - 48);
    });
    // Vaikeustason kuvaus tulee näppäinohjeiden jälkeen.
    for (osat, rivit) in [
        (&ALKU_OSAT, &ALKU_RIVIT),
        (&SPIDER_OSAT, &SPIDER_RIVIT),
        (&PYRAMIDI_OSAT, &PYRAMIDI_RIVIT),
    ] {
        let mut rivit = rivit.to_vec();
        rivit.extend(["pasianssi.kuvaus_normaali", "pasianssi.kuvaus_normaali"]);
        alkuohje_mahtuu(osat, &rivit, VARIANTTI_Y);
    }
}

#[test]
fn lajinappi_loytyy_ja_taso_pysyy_erillaan() {
    for laji in Variantti::KAIKKI {
        let (x, y) = variantin_keskipiste(laji);
        assert_eq!(variantti_pisteesta(x, y), Some(laji));
        assert_eq!(vaikeus_pisteesta(x, y), None);
    }
    let (x, y) = vaikeuden_keskipiste(Vaikeus::Vaikea);
    assert_eq!(variantti_pisteesta(x, y), None);
}

#[test]
fn jokainen_kohta_loytyy_ja_napit_tunnistetaan() {
    let lauta = Lauta::new(Vaikeus::Normaali, &mut Arpa::new(1));
    assert_eq!(
        kohta_pisteesta(&lauta, sarakkeen_x(0) + 5, YLARIVI_Y + 5),
        Some(Kohta::Pakka)
    );
    assert_eq!(
        kohta_pisteesta(&lauta, sarakkeen_x(3) + 5, YLARIVI_Y + 5),
        Some(Kohta::Perusta(0))
    );
    for sarake in 0..SARAKKEITA {
        let tiedot = &lauta.tila.sarakkeet[sarake];
        let viimeinen = tiedot.kortit.len() - 1;
        let y = kortin_y(tiedot, viimeinen) + KORTTI_K - 1;
        assert_eq!(
            kohta_pisteesta(&lauta, sarakkeen_x(sarake) + 5, y),
            Some(Kohta::Sarake {
                sarake,
                kortti: Some(viimeinen)
            })
        );
    }
    for (nappi, _) in NAPIT {
        let (x, y) = napin_keskipiste(nappi);
        assert_eq!(nappi_pisteesta(x, y), Some(nappi));
    }
    assert_eq!(nappi_pisteesta(SIVU_TEKSTI_X, NAPIT_Y - 1), None);
    // Osuma-alue seuraa kunkin napin omaa leveyttä.
    for (i, (nappi, teksti)) in NAPIT.into_iter().enumerate() {
        let y = NAPIT_Y + i as i32 * NAPPI_ASKEL;
        let reuna = SIVU_TEKSTI_X + napin_leveys(teksti);
        assert_eq!(nappi_pisteesta(reuna - 1, y), Some(nappi));
        assert_eq!(nappi_pisteesta(reuna, y), None);
    }
}

#[test]
fn pudotus_tunnistaa_perustat_ja_sarakkeet() {
    assert_eq!(
        kohde_pisteesta(sarakkeen_x(4) + 1, YLARIVI_Y + 1),
        Some(Kohde::Perusta(1))
    );
    assert_eq!(kohde_pisteesta(sarakkeen_x(0) + 1, YLARIVI_Y + 1), None);
    assert_eq!(
        kohde_pisteesta(sarakkeen_x(2) + 1, TAULU_Y + 700),
        Some(Kohde::Sarake(2))
    );
}

#[test]
fn koko_peli_piirtyy_kaatumatta() {
    let mut ruutu = vec![0; RUUDUN_LEVEYS * RUUDUN_KORKEUS];
    let mut klassinen = vec![0; LEVEYS * KORKEUS];
    for (vaikeus, laji) in Vaikeus::KAIKKI
        .into_iter()
        .flat_map(|v| Variantti::KAIKKI.map(|l| (v, l)))
    {
        let mut peli = Pasianssi::new(3);
        peli.vaikeus = vaikeus;
        peli.variantti = laji;
        piirra_alku(&peli, &mut klassinen);
        peli.aloita();
        peli.kursori_nakyy = true;
        // Uudet lajit riittää käydä läpi lyhyesti: Spiderin pakkakin loppuu viidessä jaossa.
        let nostoja = if laji == Variantti::Klondike { 30 } else { 6 };
        for _ in 0..nostoja {
            peli.lauta.nosta();
            peli.lauta.keraa();
            peli.spider.jaa();
            peli.pyramidi.nosta();
            piirra(&peli, &mut ruutu);
        }
        peli.vaihe = Vaihe::Voitto;
        for maa in 0..MAITA {
            peli.lauta.tila.perustat[maa] = (1..=KUNINGAS)
                .map(|arvo| Kortti {
                    maa: Maa::KAIKKI[maa],
                    arvo,
                })
                .collect();
        }
        for _ in 0..5 {
            peli.edista(1500);
            piirra(&peli, &mut ruutu);
        }
        peli.vaihe = Vaihe::Nimi;
        piirra(&peli, &mut ruutu);
    }
}

#[test]
fn haastelista_ja_jokainen_haaste_piirtyvat_kaatumatta() {
    let mut ruutu = vec![0; RUUDUN_LEVEYS * RUUDUN_KORKEUS];
    for laji in Variantti::KAIKKI {
        let mut peli = Pasianssi::new(1);
        peli.variantti = laji;
        peli.haastepisteet[laji as usize][2] = 3400;
        peli.vaihe = Vaihe::Haasteet;
        piirra(&peli, &mut ruutu);
        for haaste in 0..HAASTEITA {
            peli.aloita_haaste(haaste);
            peli.kursori_nakyy = true;
            piirra(&peli, &mut ruutu);
        }
        peli.vaihe = Vaihe::Voitto;
        piirra(&peli, &mut ruutu);
    }
}
