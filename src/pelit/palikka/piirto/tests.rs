use super::*;
use crate::yhteinen::kieli::{kaikilla_kielilla, muotoile};
use crate::yhteinen::syote::Syote;
use klassikot8000::palikka::{SUURIN_TULOS, VAREJA};

pub fn ruudun_keskipiste(ruutu: Paikka) -> (i32, i32) {
    keskipiste(ruutu)
}

#[test]
fn lauta_ja_sarake_mahtuvat_kuvaan() {
    const { assert!(RUUDUT_Y + LAUDAN_SIVU < PANEELI_Y) };
    const { assert!(SARAKE_X + EDISTYS_PALOJA as i32 * PALKIN_PALA < LEVEYS as i32) };
    const { assert!(PARHAAT_Y + 30 + 3 * 26 < PANEELI_Y) };
    assert_eq!(MUODOT.len(), VAREJA);
    // sijoitus, kahdeksan merkin nimi ja kuusinumeroiset pisteet samalle riville
    assert!((2 + 8 + 1 + 6) * TULOS.2 <= PARHAAT_L);
}

#[test]
fn ruudut_loytyvat_ja_ulkopuoli_ei() {
    for rivi in 0..SIVU {
        for sarake in 0..SIVU {
            let (x, y) = ruudun_keskipiste((rivi, sarake));
            assert_eq!(ruutu_pisteesta(x, y), Some((rivi, sarake)));
        }
    }
    assert_eq!(ruutu_pisteesta(RUUDUT_X - 1, RUUDUT_Y), None);
    assert_eq!(ruutu_pisteesta(RUUDUT_X, RUUDUT_Y + LAUDAN_SIVU), None);
}

#[test]
fn tekstit_mahtuvat_paikoilleen() {
    kaikilla_kielilla(|| {
        let viestit = [
            kaanna(OHJE).to_string(),
            kaanna("palikka.huono").to_string(),
            muotoile("palikka.siirto", &[&SUURIN_TULOS]),
            muotoile("palikka.ketju", &[&99, &SUURIN_TULOS]),
            muotoile("palikka.uusi_taso", &[&999]),
            muotoile("palikka.ei_siirtoja", &[&SUURIN_TULOS]),
        ];
        for viesti in viestit {
            assert!(viesti.chars().count() <= RIVILLE_MERKKEJA, "{viesti}");
        }
    });
    alkuohje_mahtuu(&ALKU_OSAT, &ALKU_RIVIT, ALOITUS_Y);
}

#[test]
fn koko_peli_piirtyy_kaatumatta_toiston_aikana() {
    let mut kuva = vec![0; LEVEYS * KORKEUS];
    let mut peli = Palikkapeli::new(3);
    piirra(&peli, &mut kuva);
    peli.kasittele(Syote::Ammu);
    for _ in 0..60 {
        let Some((a, b)) = peli.lauta.vihje() else {
            break;
        };
        peli.vaihda(b, a);
        while peli.animoi() {
            peli.edista(40);
            piirra(&peli, &mut kuva);
        }
    }
    peli.kursori_nakyy = true;
    peli.valittu = Some((0, 0));
    peli.edista(10_000);
    piirra(&peli, &mut kuva);
    // Erikoispalikat ja hutivaihto
    peli.lauta = klassikot8000::palikka::Palikkalauta::ruuduista({
        let mut ruudut = *peli.lauta.ruudut();
        ruudut[0][0] = Some(Palikka::Pommi(1));
        ruudut[0][1] = Some(Palikka::Kaikki);
        ruudut
    });
    piirra(&peli, &mut kuva);
    peli.vaihe = Vaihe::Tappio;
    piirra(&peli, &mut kuva);
    peli.vaihe = Vaihe::Nimi;
    piirra(&peli, &mut kuva);
}
