use super::*;
use crate::pelit::pasianssi::piirto::{SIVU_ALKAA, sivu_mahtuu};
use klassikot8000::Arpa;
use klassikot8000::pasianssi::Vaikeus;

pub fn napin_keskipiste(nappi: Nappi) -> (i32, i32) {
    crate::pelit::pasianssi::piirto::napin_keskipiste_listasta(&NAPIT, nappi)
}

pub fn pakan_keskipiste() -> (i32, i32) {
    (PAKKA_X + KORTTI_L / 2, YLARIVI_Y + KORTTI_K / 2)
}

#[test]
fn pyramidi_mahtuu_poydalle_eika_osu_pakkaan() {
    let (vasen, alin) = pyramidin_paikka(RIVEJA - 1, 0);
    assert!(vasen > 0);
    assert!(alin + KORTTI_K < RUUDUN_KORKEUS as i32);
    let (oikea, _) = pyramidin_paikka(RIVEJA - 1, RIVEJA - 1);
    assert!(oikea + KORTTI_L < SIVU_ALKAA);
    // Toinen rivi on jätepinon korkeudella, joten sen on jäätävä sen oikealle puolelle.
    let (toinen, _) = pyramidin_paikka(1, 0);
    assert!(jatteen_x(0) + KORTTI_L < toinen);
    let (kolmas_x, kolmas_y) = pyramidin_paikka(2, 0);
    assert!(YLARIVI_Y + KORTTI_K < kolmas_y || jatteen_x(0) + KORTTI_L < kolmas_x);
    sivu_mahtuu(&OHJEET, &NAPIT);
}

#[test]
fn vain_vapaat_kortit_ja_pakka_loytyvat() {
    let lauta = Lauta::new(Vaikeus::Helppo, &mut Arpa::new(1));
    let (x, y) = pakan_keskipiste();
    assert_eq!(kohta_pisteesta(&lauta, x, y), Some(Kohta::Pakka));
    // Alimman rivin kortit ovat vapaita, ylempien ei.
    let (x, y) = pyramidin_paikka(RIVEJA - 1, 3);
    assert_eq!(
        kohta_pisteesta(&lauta, x + 5, y + 5),
        Some(Kohta::Paikka(Paikka::Pyramidi {
            rivi: RIVEJA - 1,
            sarake: 3
        }))
    );
    let (x, y) = pyramidin_paikka(0, 0);
    assert_eq!(kohta_pisteesta(&lauta, x + 5, y + 5), None);
    for (nappi, _) in NAPIT {
        let (x, y) = napin_keskipiste(nappi);
        assert_eq!(nappi_pisteesta(x, y), Some(nappi));
    }
}
