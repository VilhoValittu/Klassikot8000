use super::*;
use crate::yhteinen::kieli::{kaikilla_kielilla, muotoile};
use crate::yhteinen::syote::Syote;
use klassikot8000::suora::{Lauta, SIIRTOJA_ENINTAAN, SUURIN_TULOS};

pub fn ruudun_keskipiste(ruutu: Paikka) -> (i32, i32) {
    let (x, y) = ruudun_kulma(ruutu);
    (x + RUUTU / 2, y + RUUTU / 2)
}

pub fn vastuksen_keskipiste(vastus: Vastus) -> (i32, i32) {
    let (_, teksti, x) = VASTUKSET.iter().find(|t| t.0 == vastus).copied().unwrap();
    (x + napin_leveys(teksti) / 2, VASTUS_Y + NAPPI_K / 2)
}

pub fn vaikeuden_keskipiste(vaikeus: Vaikeus) -> (i32, i32) {
    let (_, teksti, x) = TASOT.iter().find(|t| t.0 == vaikeus).copied().unwrap();
    (x + napin_leveys(teksti) / 2, TASO_Y + NAPPI_K / 2)
}

#[test]
fn lauta_ja_sivuikkunat_mahtuvat_konepikselin_ruudukkoon() {
    let (x, y) = ORIGO;
    assert_eq!((x % 3, y % 3, RUUTU % 3), (0, 0, 0));
    assert!(y >= ALUE_Y);
    assert!(y + LAUTA + 3 <= ALUE_Y + ALUE_K);
    // Sivuikkunat valintakehyksineen jäävät laudan ja kuvan reunan väliin.
    assert!(SIVU_X[0] - 12 >= 0);
    assert!(SIVU_X[0] + SIVU_L + 12 < x - 3);
    assert!(SIVU_X[1] - 12 > x + LAUTA + 3);
    assert!(SIVU_X[1] + SIVU_L + 12 <= LEVEYS as i32);
    const { assert!(SIVU_Y - 12 > 36) };
    // Iso merkki ja sen varjo mahtuvat ikkunan sisään voittolukujen yläpuolelle.
    const { assert!(9 * ISO_KERROIN + ISO_KERROIN + 6 <= SIVU_L) };
    const { assert!(45 + 10 * ISO_KERROIN <= SIVU_K - 48) };
}

#[test]
fn ruudut_loytyvat_ja_ulkopuoli_ei() {
    for rivi in 0..KOKO {
        for sarake in 0..KOKO {
            let (x, y) = ruudun_keskipiste((rivi, sarake));
            assert_eq!(ruutu_pisteesta(x, y), Some((rivi, sarake)));
        }
    }
    let (x, y) = ORIGO;
    assert_eq!(ruutu_pisteesta(x - 1, y), None);
    assert_eq!(ruutu_pisteesta(x, y - 1), None);
    assert_eq!(ruutu_pisteesta(x + LAUTA, y), None);
    assert_eq!(ruutu_pisteesta(x, y + LAUTA), None);
}

#[test]
fn tekstit_mahtuvat_paikoilleen() {
    kaikilla_kielilla(|| {
        for (taso, teksti, x) in TASOT {
            assert!(x + napin_leveys(teksti) < LEVEYS as i32, "{taso:?}");
            assert_eq!(vaikeus_pisteesta(x, TASO_Y), Some(taso));
        }
        // Vastusnapit eivät mene päällekkäin, ja kumpikin löytyy.
        let mut oikea = 0;
        for (vastus, teksti, x) in VASTUKSET {
            assert!(x > oikea, "{vastus:?}");
            oikea = x + napin_leveys(teksti);
            assert!(oikea < LEVEYS as i32, "{vastus:?}");
            assert_eq!(vastus_pisteesta(x, VASTUS_Y), Some(vastus));
        }
        let otsikko = kaanna("suora.peli_rivi").chars().count() as i32 * MERKKI;
        assert!(48 + otsikko < TASO_X);
        let mut viestit = vec![
            kaanna(OHJE).to_string(),
            kaanna("suora.aloita").to_string(),
            kaanna("suora.kone_miettii").to_string(),
            kaanna("suora.vuorosi").to_string(),
            kaanna("suora.kone_voitti").to_string(),
            kaanna("suora.lauta_taynna").to_string(),
            kaanna("suora.vuoro_risti").to_string(),
            kaanna("suora.vuoro_nolla").to_string(),
            kaanna("suora.risti_sai").to_string(),
            kaanna("suora.nolla_sai").to_string(),
            muotoile("suora.ennatys", &[&SIIRTOJA_ENINTAAN, &SUURIN_TULOS]),
            muotoile("suora.voitto_viesti", &[&SIIRTOJA_ENINTAAN, &SUURIN_TULOS]),
        ];
        for vastus in Vastus::KAIKKI {
            viestit.extend(Vaikeus::KAIKKI.map(|taso| kaanna(kuvaus(vastus, taso)).to_string()));
        }
        for viesti in viestit {
            assert!(viesti.chars().count() <= RIVILLE_MERKKEJA, "{viesti}");
        }
        for otsikko in [
            "yhteinen.voitit",
            "suora.tappio",
            "suora.tasapeli",
            "suora.risti_voitti",
            "suora.nolla_voitti",
        ] {
            let merkkeja = kaanna(otsikko).chars().count() as i32;
            assert!(merkkeja * TULOS.2 < LEVEYS as i32 - TEKSTI_X, "{otsikko}");
        }
        // Nimet mahtuvat sivuikkunaan reunuksineen.
        for nimi in ["suora.sina", "suora.kone", "suora.risti", "suora.nolla"] {
            let leveys = kaanna(nimi).chars().count() as i32 * MERKKI;
            assert!(leveys <= SIVU_L - 24, "{nimi}");
        }
        // Pisin tilasto ei saa ulottua seuraavan päälle.
        let paras = format!("{} {SUURIN_TULOS}", kaanna("yhteinen.paras"));
        let siirrot = format!("{} {SIIRTOJA_ENINTAAN}", kaanna("yhteinen.siirrot"));
        for teksti in [paras, siirrot] {
            assert!(
                teksti.chars().count() as i32 * OTSIKKO.2 < TILASTON_ASKEL,
                "{teksti}"
            );
        }
        let kaksin = kaanna("suora.vastus_kaksin").to_uppercase();
        assert!(
            TEKSTI_X + TILASTON_ASKEL + kaksin.chars().count() as i32 * OTSIKKO.2 < LEVEYS as i32
        );
        for taso in Vaikeus::KAIKKI {
            let nimi = kaanna(tason_nimi(taso));
            let loppu = TEKSTI_X + 2 * TILASTON_ASKEL + nimi.chars().count() as i32 * OTSIKKO.2;
            assert!(loppu < LEVEYS as i32, "{nimi}");
        }
    });
    // Kuvausrivi tulee näppäinohjeiden jälkeen puolen rivin välillä, jota apu ei laske.
    let rivit: Vec<&str> = ALKU_RIVIT.iter().copied().chain([OHJE]).collect();
    alkuohje_mahtuu(&ALKU_OSAT, &rivit, VASTUS_Y - RIVIN_KORKEUS / 2);
}

#[test]
fn jokainen_vaihe_piirtyy_kaatumatta() {
    let mut kuva = vec![0; LEVEYS * KORKEUS];
    for taso in Vaikeus::KAIKKI {
        let mut peli = Suora::new(1);
        peli.vaikeus = taso;
        piirra(&peli, &mut kuva);
        peli.kasittele(Syote::Ammu);
        piirra(&peli, &mut kuva);
        let (x, y) = ruudun_keskipiste((7, 7));
        peli.kasittele(Syote::Hiiri(x, y));
        piirra(&peli, &mut kuva);
        peli.kasittele(Syote::Klikkaus(x, y));
        piirra(&peli, &mut kuva);
        peli.edista(10_000);
        peli.kasittele(Syote::Nuoli(0, 1));
        peli.kasittele(Syote::Nuoli(0, 1));
        piirra(&peli, &mut kuva);
        // Voittorivi vinoon laudan kulmasta kulmaan ja täysi lauta.
        peli.lauta = Lauta::merkeista(
            &[(10, 10), (11, 11), (12, 12), (13, 13)],
            &[(0, 0), (0, 1), (0, 2), (0, 3)],
        );
        peli.merkitse((14, 14));
        assert_eq!(peli.vaihe, Vaihe::Nimi);
        peli.nimi = "VÄINÖ".to_string();
        piirra(&peli, &mut kuva);
        peli.vahvista_nimi();
        piirra(&peli, &mut kuva);
        peli.vaihe = Vaihe::Tappio;
        piirra(&peli, &mut kuva);
        peli.vaihe = Vaihe::Tasapeli;
        piirra(&peli, &mut kuva);
    }
}

#[test]
fn kaksinpeli_piirtyy_alusta_voittoon_kaatumatta() {
    let mut kuva = vec![0; LEVEYS * KORKEUS];
    let mut peli = Suora::new(1);
    peli.valitse_vastus(Vastus::Kaksin);
    piirra(&peli, &mut kuva);
    peli.kasittele(Syote::Ammu);
    peli.merkitse((7, 7));
    let (x, y) = ruudun_keskipiste((7, 8));
    peli.kasittele(Syote::Hiiri(x, y));
    piirra(&peli, &mut kuva);
    peli.lauta = Lauta::merkeista(
        &[(0, 0), (0, 2), (0, 4), (0, 6)],
        &[(5, 5), (6, 6), (7, 7), (8, 8)],
    );
    peli.lauta.vuoro = Merkki::Nolla;
    peli.merkitse((9, 9));
    assert_eq!(peli.vaihe, Vaihe::Voitto);
    piirra(&peli, &mut kuva);
}

#[test]
fn malli_rakentuu() {
    let malli = suoramalli();
    assert!(!malli.is_empty());
    // Malli piirretään joka ruudussa: pidetään tahkojen määrä kohtuullisena.
    assert!(malli.len() < 600, "{}", malli.len());
}
