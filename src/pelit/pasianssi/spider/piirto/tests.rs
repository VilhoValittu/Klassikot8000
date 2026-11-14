use super::*;
use crate::yhteinen::kieli::kaikilla_kielilla;
use klassikot8000::Arpa;
use klassikot8000::kortit::Maa;
use klassikot8000::pasianssi::Vaikeus;

const VIESTIN_RIVEJA: usize = 5;

pub fn napin_keskipiste(nappi: Nappi) -> (i32, i32) {
    let i = NAPIT.iter().position(|n| n.0 == nappi).unwrap() as i32;
    (NAPIT_X + 12, NAPIT_Y + i * NAPPI_ASKEL + NAPPI_K / 2)
}

pub fn pakan_keskipiste() -> (i32, i32) {
    (SARAKE_X + KORTTI_L / 2, YLARIVI_Y + KORTTI_K / 2)
}

#[test]
fn kaikki_mahtuu_ruudulle_ja_pitkakin_sarake_tiivistyy() {
    assert!(sarakkeen_x(SARAKKEITA - 1) + KORTTI_L < RUUDUN_LEVEYS as i32);
    const { assert!(SARAKE_X + PAKAN_LEVEYS < VALMIIT_X) };
    assert!(VALMIIT_X + (SARJOJA as i32 - 1) * VALMIS_ASKEL + KORTTI_L < PANEELI_X);
    const { assert!(PANEELI_YLA + PANEELI_KORKEUS <= TAULU_Y) };
    assert!(NAPIT_Y + NAPIT.len() as i32 * NAPPI_ASKEL <= PANEELI_YLA + PANEELI_KORKEUS);
    assert!(VIESTI_Y + VIESTIN_RIVEJA as i32 * SIVU_RIVI <= PANEELI_YLA + PANEELI_KORKEUS);

    for (piilossa, nakyvia) in [(0, 1), (5, 13), (5, 40), (0, 104), (5, 99)] {
        let tiedot = Sarake {
            kortit: vec![
                Kortti {
                    maa: Maa::Pata,
                    arvo: 1
                };
                piilossa + nakyvia
            ],
            piilossa,
        };
        let alin = kortin_y(&tiedot, tiedot.kortit.len() - 1) + KORTTI_K;
        assert!(alin <= TAULU_ALA, "{piilossa} {nakyvia}: {alin}");
    }
}

#[test]
fn napit_ja_tekstit_mahtuvat_paneeliin() {
    kaikilla_kielilla(|| {
        for (_, teksti) in NAPIT {
            assert!(
                NAPIT_X + napin_leveys(teksti) <= PANEELI_X + PANEELI_L - 12,
                "{teksti}"
            );
        }
        for avain in ["yhteinen.pisteet", "yhteinen.siirrot", "spider.jakoja"] {
            assert!(kaanna(avain).chars().count() as i32 * TULOS.2 < TILASTO_ASKEL);
        }
        for avain in [
            "spider.ei_jakoa",
            "pasianssi.ei_kay",
            "pasianssi.pakka_loppu",
            "pasianssi.ei_peruttavaa",
            "pasianssi.jumissa",
            "pasianssi.voittoviesti",
            "pasianssi.siirrot_loppu",
            "pasianssi.haaste_viesti",
            "pasianssi.haaste_voitto",
            "pasianssi.haaste_jumissa",
        ] {
            let rivit = rivita(kaanna(avain), VIESTIN_MERKKEJA).len();
            assert!(rivit <= VIESTIN_RIVEJA, "{avain}");
        }
    });
    assert!(TILASTO_X + VIESTIN_MERKKEJA as i32 * MERKKI <= NAPIT_X);
}

#[test]
fn kohdat_ja_napit_tunnistetaan() {
    let lauta = Lauta::new(Vaikeus::Helppo, &mut Arpa::new(1));
    let (x, y) = pakan_keskipiste();
    assert_eq!(kohta_pisteesta(&lauta, x, y), Some(Kohta::Pakka));
    assert_eq!(kohta_pisteesta(&lauta, VALMIIT_X + 5, y), None);
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
    assert_eq!(kohde_pisteesta(sarakkeen_x(9) + 1, TAULU_Y), Some(9));
    assert_eq!(kohde_pisteesta(sarakkeen_x(9) + 1, TAULU_Y - 1), None);
}
