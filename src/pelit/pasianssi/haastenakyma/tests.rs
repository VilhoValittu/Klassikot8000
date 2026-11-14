use super::*;
use crate::yhteinen::kieli::kaikilla_kielilla;

pub fn lajin_keskipiste(laji: Variantti) -> (i32, i32) {
    let (_, teksti, x) = VARIANTIT[laji as usize];
    (lajin_x(x) + napin_leveys(teksti) / 2, LAJI_Y + NAPPI_K / 2)
}

pub fn haasteen_keskipiste(haaste: usize) -> (i32, i32) {
    let (x, y) = solun_paikka(haaste);
    (x + SOLU_L / 2, y + SOLU_K / 2)
}

pub fn takaisin_keskipiste() -> (i32, i32) {
    (SIVU_TEKSTI_X + 12, NAPIT_Y + NAPPI_K / 2)
}

#[test]
fn lista_paivamaarineen_mahtuu_ohjeen_ylapuolelle() {
    const { assert!(LISTA_Y + PARHAAT_OTSIKKO + 3 * parhaiden_askel(OTSIKKO, Some(TULOS)) < SUMMA_Y) };
}

#[test]
fn ruudukko_mahtuu_ja_solut_loytyvat() {
    let (oikea, alin) = solun_paikka(HAASTEITA - 1);
    assert!(oikea + SOLU_L < SIVU_X - 12);
    assert!(alin + SOLU_K < RUUDUN_KORKEUS as i32);
    assert!(HAASTEITA.is_multiple_of(HAASTERIVILLA));
    for haaste in 0..HAASTEITA {
        let (x, y) = haasteen_keskipiste(haaste);
        assert_eq!(haaste_pisteesta(x, y), Some(haaste));
    }
    assert_eq!(haaste_pisteesta(SOLU_X - 1, SOLU_Y), None);
    for laji in Variantti::KAIKKI {
        let (x, y) = lajin_keskipiste(laji);
        assert_eq!(laji_pisteesta(x, y), Some(laji));
        assert_eq!(haaste_pisteesta(x, y), None);
    }
    let (x, y) = takaisin_keskipiste();
    assert!(takaisin_pisteessa(x, y));
}

#[test]
fn sivun_tekstit_mahtuvat_kaikilla_kielilla() {
    kaikilla_kielilla(|| {
        let ohje = rivita(kaanna("pasianssi.haaste_ohje"), SIVU_MERKKEJA).len() as i32;
        // Ohjeen ja näppäinrivien jälkeen on vielä tilaa takaisin-napille.
        assert!(OHJE_Y + (ohje + 3) * SIVU_RIVI <= NAPIT_Y);
        for avain in [
            "pasianssi.haaste_nappaimet_1",
            "pasianssi.haaste_nappaimet_2",
        ] {
            assert!(kaanna(avain).chars().count() <= SIVU_MERKKEJA, "{avain}");
        }
        let otsikko = format!(
            "{} - {}",
            kaanna("pasianssi.haasteet_otsikko"),
            listan_otsikko(true)
        );
        assert!(
            otsikko.chars().count() as i32 * MERKKI <= SIVU_L - 36,
            "{otsikko}"
        );
        let puolikas = (SIVU_L - 36) / 2;
        for avain in ["pasianssi.haaste_yhteensa", "pasianssi.haaste_lapi"] {
            assert!(
                kaanna(avain).chars().count() as i32 * OTSIKKO.2 < puolikas,
                "{avain}"
            );
        }
        for pari in VARIANTIT.windows(2) {
            let oikea = lajin_x(pari[0].2) + napin_leveys(pari[0].1);
            assert!(oikea < lajin_x(pari[1].2));
        }
        let (_, viimeinen, x) = VARIANTIT[VARIANTIT.len() - 1];
        assert!(lajin_x(x) + napin_leveys(viimeinen) < SIVU_X - 12);
    });
}
