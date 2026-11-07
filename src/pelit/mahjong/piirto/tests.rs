use super::*;
use crate::yhteinen::kieli::kaikilla_kielilla;
use crate::yhteinen::syote::Syote;
use klassikot8000::Arpa;

pub fn laatan_keskipiste(lauta: &Lauta, i: usize) -> (i32, i32) {
    kasvon_keski(lauta, origo(lauta), i)
}

pub fn napin_keskipiste(nappi: Nappi) -> (i32, i32) {
    let i = NAPIT.iter().position(|n| n.0 == nappi).unwrap() as i32;
    (SIVU_TEKSTI_X + 12, NAPIT_Y + i * NAPPI_ASKEL + NAPPI_K / 2)
}

pub fn vaikeuden_keskipiste(vaikeus: Vaikeus) -> (i32, i32) {
    let (_, teksti, x) = TASOT.iter().find(|t| t.0 == vaikeus).copied().unwrap();
    (x + napin_leveys(teksti) / 2, TASO_Y + NAPPI_K / 2)
}

#[test]
fn kaikki_pohjat_mahtuvat_alueelle() {
    let mut arpa = Arpa::new(1);
    for taso in Vaikeus::KAIKKI {
        let lauta = Lauta::new(taso, &mut arpa);
        let o = origo(&lauta);
        for i in 0..lauta.paikat.len() {
            let (x, y) = kasvo(&lauta, o, i);
            assert!(x - PAKSUUS - U >= ALUE_X, "{taso:?}");
            assert!(x + LAATTA_L + U <= ALUE_X + ALUE_L, "{taso:?}");
            assert!(y - U >= ALUE_Y, "{taso:?}");
            assert!(y + LAATTA_K + PAKSUUS + U <= ALUE_Y + ALUE_K, "{taso:?}");
            assert_eq!(x % U, 0);
            assert_eq!(y % U, 0);
        }
    }
    assert!(SIVU_PARHAAT_Y + PARHAAT_OTSIKKO + 3 * PARHAAT_RIVI < RUUDUN_KORKEUS as i32 - 30);
    assert!(SIVU_MERKKEJA as i32 * MERKKI <= SIVU_L - 36);
}

#[test]
fn jokainen_jaljella_oleva_laatta_loytyy_jostain_pisteesta() {
    let mut arpa = Arpa::new(3);
    for taso in Vaikeus::KAIKKI {
        let lauta = Lauta::new(taso, &mut arpa);
        let o = origo(&lauta);
        let mut loytyi = vec![false; lauta.paikat.len()];
        for y in (ALUE_Y..ALUE_Y + ALUE_K).step_by(U as usize) {
            for x in (ALUE_X..ALUE_X + ALUE_L).step_by(U as usize) {
                if let Some(i) = laatta_pisteesta(&lauta, x, y) {
                    loytyi[i] = true;
                }
            }
        }
        // Päällimmäinen laatta peittää alemman, joten vain näkyvät löytyvät; vapaat ainakin.
        for (i, vapaa) in lauta.vapaat().into_iter().enumerate() {
            if vapaa {
                assert!(loytyi[i], "{taso:?}: {i}");
                let (x, y) = kasvon_keski(&lauta, o, i);
                assert_eq!(laatta_pisteesta(&lauta, x, y), Some(i), "{taso:?}: {i}");
            }
        }
        assert_eq!(laatta_pisteesta(&lauta, 5, 5), None);
    }
}

#[test]
fn napit_ja_tasot_loytyvat() {
    for (nappi, _) in NAPIT {
        let (x, y) = napin_keskipiste(nappi);
        assert_eq!(nappi_pisteesta(x, y), Some(nappi));
    }
    assert_eq!(nappi_pisteesta(SIVU_TEKSTI_X - 1, NAPIT_Y + 3), None);
    assert_eq!(nappi_pisteesta(SIVU_TEKSTI_X + 3, NAPIT_Y - 1), None);
    for taso in Vaikeus::KAIKKI {
        let (x, y) = vaikeuden_keskipiste(taso);
        assert_eq!(vaikeus_pisteesta(x, y), Some(taso));
    }
}

#[test]
fn tekstit_mahtuvat_paikoilleen() {
    kaikilla_kielilla(|| {
        for (taso, teksti, x) in TASOT {
            assert!(x + napin_leveys(teksti) < LEVEYS as i32, "{taso:?}");
        }
        for rivi in SIVU_OHJEET {
            assert!(kaanna(rivi).chars().count() <= SIVU_MERKKEJA, "{rivi}");
        }
        for (_, teksti) in NAPIT {
            assert!(napin_leveys(teksti) <= SIVU_L - 36, "{teksti}");
        }
        for avain in [
            "yhteinen.pisteet",
            "mahjong.ketju",
            "mahjong.aika",
            "mahjong.pareja",
            "mahjong.laattoja",
            "mahjong.sekoituksia",
        ] {
            // Otsikko ja viisinumeroinen arvo samalla rivillä.
            let merkkeja = kaanna(avain).chars().count() as i32 + 1 + 5;
            assert!(merkkeja * OTSIKKO.2 <= SIVU_L - 36, "{avain}");
        }
        for taso in Vaikeus::KAIKKI {
            let kuvaus = muotoile(
                "mahjong.kuvaus",
                &[&pohjan_nimi(taso), &taso.pohja().len(), &taso.kerroin()],
            );
            assert!(kuvaus.chars().count() <= RIVILLE_MERKKEJA, "{kuvaus}");
            let otsikko = format!("{} - {}", kaanna("peli.mahjong"), pohjan_nimi(taso));
            assert!(otsikko.chars().count() <= 30, "{otsikko}");
        }
        for avain in TUULET {
            assert_eq!(kaanna(avain).chars().count(), 1, "{avain}");
        }
        // Viestit mahtuvat neljälle riville.
        for viesti in [
            kaanna("mahjong.aloita").to_string(),
            kaanna("mahjong.ei_vapaa").to_string(),
            kaanna("mahjong.ei_pari").to_string(),
            kaanna("mahjong.ei_sekoituksia").to_string(),
            kaanna("mahjong.peruttu").to_string(),
            kaanna("mahjong.ei_peruttavaa").to_string(),
            kaanna("mahjong.ei_perua_vaikealla").to_string(),
            muotoile("mahjong.pari", &[&999]),
            muotoile("mahjong.ketju_viesti", &[&5, &999]),
            muotoile("mahjong.vihje", &[&20]),
            muotoile("mahjong.jumissa", &[&50]),
            muotoile("mahjong.sekoitettu", &[&50]),
            muotoile("mahjong.voitto_viesti", &[&99999]),
            muotoile("mahjong.tappio_viesti", &[&99999]),
        ] {
            assert!(rivita(&viesti, SIVU_MERKKEJA).len() <= 4, "{viesti}");
        }
        let tila = LOPPU_L - 48;
        for otsikko in ["yhteinen.voitit", "mahjong.tappio"] {
            assert!(
                kaanna(otsikko).chars().count() as i32 * OTSIKKO.2 <= tila,
                "{otsikko}"
            );
        }
        let yhteensa = muotoile("mahjong.yhteensa", &[&99999]);
        assert!(yhteensa.chars().count() as i32 * OTSIKKO.2 <= tila);
        for rivi in [
            muotoile("mahjong.erittely_parit", &[&99999]),
            muotoile("mahjong.erittely_tyhjennys", &[&9999]),
            muotoile("mahjong.erittely_aika", &[&9999]),
            muotoile("mahjong.erittely_jaljella", &[&144]),
            kaanna("mahjong.loppu_ohje").to_string(),
        ] {
            assert!(rivi.chars().count() as i32 * MERKKI <= tila, "{rivi}");
        }
    });
    let rivit: Vec<&str> = ALKU_RIVIT
        .iter()
        .copied()
        .chain(["mahjong.kuvaus"])
        .collect();
    alkuohje_mahtuu(&ALKU_OSAT, &rivit, TASO_Y);
}

#[test]
fn jokainen_laatta_ja_vaihe_piirtyy_kaatumatta() {
    let mut ruutu = vec![0; RUUDUN_LEVEYS * RUUDUN_KORKEUS];
    let mut kuva = vec![0; LEVEYS * KORKEUS];
    for taso in Vaikeus::KAIKKI {
        let mut peli = Mahjong::new(1);
        peli.vaikeus = taso;
        piirra_alku(&peli, &mut kuva);
        peli.kasittele(Syote::Ammu);
        // Kaikki lajit näkyviin samalle laudalle.
        for (i, laatta) in peli.lauta.laatat.iter_mut().enumerate() {
            *laatta = Some(Laatta((i % 42) as u8));
        }
        let (a, b) = (0, 1);
        peli.valittu = Some(a);
        peli.vihje = Some((a, b));
        peli.hiiren_laatta = Some(b);
        piirra(&peli, &mut ruutu);
        peli.kursori_nakyy = true;
        piirra(&peli, &mut ruutu);
        let mut arpa = Arpa::new(9);
        peli.lauta = Lauta::new(taso, &mut arpa);
        let (a, b) = peli.lauta.ratkaisu[0];
        peli.valitse(a);
        peli.valitse(b);
        piirra(&peli, &mut ruutu);
        peli.edista(PONNAHDUS_MS / 2);
        piirra(&peli, &mut ruutu);
        for &(a, b) in &peli.lauta.ratkaisu.clone()[1..] {
            peli.valitse(a);
            peli.valitse(b);
        }
        assert_eq!(peli.vaihe, Vaihe::Nimi);
        piirra(&peli, &mut ruutu);
        peli.vahvista_nimi();
        for _ in 0..5 {
            peli.edista(JUHLA_MS / 4);
            piirra(&peli, &mut ruutu);
        }
        peli.lauta.tila = Tila::Tappio;
        peli.vaihe = Vaihe::Tappio;
        piirra(&peli, &mut ruutu);
    }
}
