use super::*;
use crate::yhteinen::kieli::kaikilla_kielilla;
use crate::yhteinen::syote::Syote;
use klassikot8000::miinaharava::Miinakentta;

pub fn ruudun_keskipiste(vaikeus: Vaikeus, (rivi, sarake): Paikka) -> (i32, i32) {
    let (ox, oy) = origo(vaikeus);
    let koko = ruudun_koko(vaikeus);
    (
        ox + sarake as i32 * koko + koko / 2,
        oy + rivi as i32 * koko + koko / 2,
    )
}

pub fn napin_keskipiste(nappi: Nappi) -> (i32, i32) {
    let (_, teksti, x) = NAPIT.iter().find(|n| n.0 == nappi).copied().unwrap();
    (x + napin_leveys(teksti) / 2, NAPPI_Y + NAPPI_K / 2)
}

pub fn vaikeuden_keskipiste(vaikeus: Vaikeus) -> (i32, i32) {
    let (_, teksti, x) = TASOT.iter().find(|t| t.0 == vaikeus).copied().unwrap();
    (x + napin_leveys(teksti) / 2, TASO_Y + NAPPI_K / 2)
}

#[test]
fn kaikki_laudat_mahtuvat_kuvaan() {
    for taso in Vaikeus::KAIKKI {
        let (x, y) = origo(taso);
        let koko = ruudun_koko(taso);
        assert!(x > 0);
        assert!(y >= ALUE_Y);
        assert!(x + koko * (taso.leveys() as i32) < LEVEYS as i32);
        assert!(y + koko * taso.korkeus() as i32 <= ALUE_Y + ALUE_K);
    }
}

#[test]
fn ruudut_loytyvat_ja_ulkopuoli_ei() {
    for taso in Vaikeus::KAIKKI {
        let (ox, oy) = origo(taso);
        for rivi in 0..taso.korkeus() {
            for sarake in 0..taso.leveys() {
                let (x, y) = ruudun_keskipiste(taso, (rivi, sarake));
                assert_eq!(ruutu_pisteesta(taso, x, y), Some((rivi, sarake)));
            }
        }
        assert_eq!(ruutu_pisteesta(taso, ox - 1, oy), None);
        assert_eq!(ruutu_pisteesta(taso, ox, oy - 1), None);
        let koko = ruudun_koko(taso);
        assert_eq!(
            ruutu_pisteesta(taso, ox, oy + koko * taso.korkeus() as i32),
            None
        );
        assert_eq!(
            ruutu_pisteesta(taso, ox + koko * taso.leveys() as i32, oy),
            None
        );
    }
}

#[test]
fn tekstit_mahtuvat_paikoilleen() {
    kaikilla_kielilla(|| {
        for (taso, teksti, x) in TASOT {
            assert!(x + napin_leveys(teksti) < LEVEYS as i32, "{taso:?}");
            assert_eq!(vaikeus_pisteesta(x, TASO_Y), Some(taso));
        }
        for pari in NAPIT.windows(2) {
            assert!(pari[0].2 + napin_leveys(pari[0].1) < pari[1].2);
        }
        for viesti in [
            kaanna(OHJE).to_string(),
            kaanna("miinaharava.aloita").to_string(),
            kaanna("miinaharava.pamaus").to_string(),
            muotoile("miinaharava.ennatys", &[&999, &2997]),
            muotoile("miinaharava.aika_viesti", &[&999, &2997]),
            muotoile("miinaharava.kuvaus", &[&30, &16, &99]),
        ] {
            assert!(viesti.chars().count() <= RIVILLE_MERKKEJA, "{viesti}");
        }
        let otsikot = [kaanna("yhteinen.voitit"), kaanna("miinaharava.tappio")];
        for otsikko in otsikot {
            assert!(otsikko.chars().count() as i32 * TULOS.2 < LEVEYS as i32 - TEKSTI_X);
        }
        // Pisin tilasto ei saa ulottua seuraavan päälle.
        for avain in ["miinaharava.miinat", "miinaharava.aika", "yhteinen.paras"] {
            let merkkeja = kaanna(avain).chars().count() as i32 + 1 + 4;
            assert!(merkkeja * OTSIKKO.2 < TILASTON_ASKEL, "{avain}");
        }
    });
    // Kuvausrivi tulee näppäinohjeiden jälkeen.
    let rivit: Vec<&str> = ALKU_RIVIT.iter().copied().chain([OHJE]).collect();
    alkuohje_mahtuu(&ALKU_OSAT, &rivit, TASO_Y);
}

#[test]
fn jokainen_ruudun_tila_piirtyy_kaatumatta_kaikilla_tasoilla() {
    let mut kuva = vec![0; LEVEYS * KORKEUS];
    for taso in Vaikeus::KAIKKI {
        let mut peli = Miinaharava::new(1);
        peli.vaikeus = taso;
        piirra(&peli, &mut kuva);
        peli.kasittele(Syote::Ammu);
        piirra(&peli, &mut kuva);
        peli.avaa((0, 0));
        peli.liputa((taso.korkeus() - 1, taso.leveys() - 1));
        let (x, y) = ruudun_keskipiste(taso, (1, 1));
        peli.kasittele(Syote::Hiiri(x, y));
        piirra(&peli, &mut kuva);
        peli.kasittele(Syote::Nuoli(0, 1));
        peli.kasittele(Syote::Nuoli(0, 1));
        piirra(&peli, &mut kuva);
        // Kaikki numerot, väärä lippu ja räjähdys samalla laudalla.
        peli.lauta = Miinakentta::miinoista(
            taso,
            &[
                (0, 0),
                (0, 1),
                (0, 2),
                (1, 0),
                (1, 2),
                (2, 0),
                (2, 1),
                (2, 2),
                (5, 5),
            ],
        );
        peli.avaa((1, 1));
        peli.liputa((4, 4));
        peli.avaa((5, 5));
        assert_eq!(peli.vaihe, Vaihe::Tappio);
        peli.edista(100);
        piirra(&peli, &mut kuva);
        peli.edista(10_000);
        piirra(&peli, &mut kuva);
        peli.vaihe = Vaihe::Voitto;
        piirra(&peli, &mut kuva);
        peli.vaihe = Vaihe::Nimi;
        peli.nimi = "VÄINÖ".to_string();
        piirra(&peli, &mut kuva);
    }
}
