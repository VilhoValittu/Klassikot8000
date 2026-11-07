use super::*;

pub fn aanikytkimen_keskipiste() -> (i32, i32) {
    (AANI_X + AANI_L / 2, AANI_Y + AANI_K / 2)
}

/// Ohjeet ja kehote eivät saa mennä päällekkäin eivätkä yli kuvan laidan millään kielellä.
pub fn alkuohje_mahtuu(osat: &[&str], rivit: &[&str], raja_y: i32) {
    crate::yhteinen::kieli::kaikilla_kielilla(|| {
        let osarivit: usize = osat
            .iter()
            .map(|osa| rivita(kaanna(osa), ALKU_MERKKEJA).len())
            .sum();
        let korkeus = (osarivit + rivit.len()) as i32 * RIVIN_KORKEUS + RIVIN_KORKEUS / 2;
        assert!(OHJE_Y + korkeus <= raja_y, "{osat:?}");
        for rivi in rivit.iter().chain(&[ALOITUS]) {
            let teksti = kaanna(rivi);
            assert!(teksti.chars().count() <= ALKU_MERKKEJA, "{teksti}");
        }
    });
}

const KAIKKI_RIVIT: [bool; RUUDUN_KORKEUS] = [true; RUUDUN_KORKEUS];

#[test]
fn nimi_ikkunan_tekstit_mahtuvat_kummallakin_kielella() {
    crate::yhteinen::kieli::kaikilla_kielilla(|| {
        let tila = NIMI_IKKUNA_L - 36;
        assert!(kaanna(NIMI_OHJE).chars().count() as i32 * MERKKI <= tila);
        let otsikko = format!("{} 999999", kaanna(NIMI_OTSIKKO));
        assert!(otsikko.chars().count() as i32 * OTSIKKO.2 <= tila);
    });
}

#[test]
pub fn rivitys_katkaisee_sanavalista_ja_kasittelee_aakkoset() {
    let teksti = "Vastustaja ampui B10: OSUMA! Lentotukialus upposi!";
    let rivit = rivita(teksti, RIVILLE_MERKKEJA);
    assert_eq!(
        rivit,
        ["Vastustaja ampui B10: OSUMA!", "Lentotukialus upposi!"]
    );
    assert_eq!(rivita("ääääää", 4), ["ääää", "ää"]);
    assert!(rivita("", 4).is_empty());
}

#[test]
fn skaalaus_tayttaa_ruudun_ja_hiiri_osuu_samaan_pikseliin() {
    let mut kehys = Kehys::default();
    for (i, pikseli) in kehys.kuva.iter_mut().enumerate() {
        *pikseli = i as u32 + 1;
    }
    kehys.venyta();
    let (kuva, ruutu) = (&kehys.kuva, &kehys.ruutu);
    assert_eq!(ruutu[0], PALKKI);
    assert_eq!(ruutu[RUUDUN_LEVEYS * RUUDUN_KORKEUS - 1], PALKKI);
    for (x, y) in [(285, 0), (700, 300), (1634, 1079), (1000, 1079)] {
        let (kx, ky) = ruudulta_kuvaan(x, y);
        let pikseli = ruutu[y as usize * RUUDUN_LEVEYS + x as usize];
        assert_eq!(pikseli, kuva[ky as usize * LEVEYS + kx as usize], "{x},{y}");
    }
    assert!(ruudulta_kuvaan(0, 0).0 < 0);
    assert!(ruudulta_kuvaan(1700, 0).0 >= LEVEYS as i32);
}

#[test]
fn muuttuneiden_rivien_venytys_vastaa_kokonaan_laskettua() {
    let mut kehys = Kehys::default();
    kehys.kuva.fill(0x123456);
    kehys.venyta();
    assert!(kehys.muuttuneet.iter().all(|&m| m));

    kehys.venyta();
    assert!(!kehys.muuttuneet.contains(&true));

    kehys.kuva[300 * LEVEYS + 500] = 0xABCDEF;
    kehys.venyta();
    let mut uusi = Kehys::default();
    uusi.kuva.copy_from_slice(&kehys.kuva);
    uusi.venyta();
    assert!(kehys.ruutu == uusi.ruutu);
    // Kuvan rivi 300 venyy ruudun riveille 422 ja 423 eikä muualle.
    assert!(kehys.muuttuneet[422] && kehys.muuttuneet[423]);
    assert_eq!(kehys.muuttuneet.iter().filter(|&&m| m).count(), 2);

    // Suora piirto sotkee ruudun, joten seuraava venytys laskee kaiken.
    kehys.ruutu.fill(0);
    kehys.piirretty_suoraan();
    assert!(kehys.muuttuneet.iter().all(|&m| m));
    kehys.venyta();
    assert!(kehys.muuttuneet.iter().all(|&m| m));
    assert!(kehys.ruutu == uusi.ruutu);
}

#[test]
fn sovitus_mahtuu_yleisiin_resoluutioihin_ja_hiiri_osuu_kuvaan() {
    for (l, k) in [
        (800, 450),
        (1024, 768),
        (1120, 630),
        (1280, 1024),
        (1760, 990),
        (1920, 1009),
        (1920, 1080),
        (3680, 2070),
        (1, 1),
    ] {
        let sovitus = Sovitus::new(l, k);
        assert!(
            sovitus.x + sovitus.l <= l && sovitus.y + sovitus.k <= k,
            "{l}x{k}"
        );
        assert!(sovitus.l == l || sovitus.k == k, "{l}x{k}");
        let (x, y) = (
            (sovitus.x + sovitus.l - 1) as i32,
            (sovitus.y + sovitus.k - 1) as i32,
        );
        let (rx, ry) = sovitus.ruudulle(x, y).unwrap();
        assert!((rx as usize) < RUUDUN_LEVEYS && (ry as usize) < RUUDUN_KORKEUS);
        assert_eq!(
            sovitus.ruudulle(sovitus.x as i32, sovitus.y as i32),
            Some((0, 0))
        );
    }
    // 1280x1024: palkit ylhäällä ja alhaalla eivät osu kuvaan
    let sovitus = Sovitus::new(1280, 1024);
    assert_eq!((sovitus.y, sovitus.k), (152, 720));
    assert_eq!(sovitus.ruudulle(640, 100), None);
    assert_eq!(sovitus.ruudulle(640, 900), None);
    assert_eq!(sovitus.ruudulle(-1, 300), None);
    assert_eq!(sovitus.ruudulle(640, 512), Some((960, 540)));
}

#[test]
fn rajattu_piste_jaa_kuvan_reunaan_ikkunan_ulkopuoleltakin() {
    // 1280x1024: kuva on rivit 152..872.
    let sovitus = Sovitus::new(1280, 1024);
    // Ulkopuolinen piste vastaa kuvan viimeistä pikseliä ikkunan sisällä.
    let reuna = |x, y| sovitus.ruudulle(x, y).unwrap();
    assert_eq!(sovitus.ruudulle_rajattu(-8, 512), (0, 540));
    assert_eq!(sovitus.ruudulle_rajattu(i32::MIN, i32::MIN), (0, 0));
    assert_eq!(sovitus.ruudulle_rajattu(5000, 512), reuna(1279, 512));
    assert_eq!(
        sovitus.ruudulle_rajattu(i32::MAX, i32::MAX),
        reuna(1279, 871)
    );
    assert_eq!(sovitus.ruudulle_rajattu(640, 100), (960, 0));
    assert_eq!(sovitus.ruudulle_rajattu(640, 2000), reuna(640, 871));
    // Kuvan sisällä tulos on sama kuin ilman rajausta.
    assert_eq!(sovitus.ruudulle_rajattu(640, 512), (960, 540));
    // Pienin mahdollinen ikkuna.
    assert_eq!(Sovitus::new(1, 1).ruudulle_rajattu(-5, 9), (0, 0));
}

#[test]
fn sovitus_tayttaa_palkit_ja_sailyttaa_ohuen_viivan() {
    let mut ruutu = vec![0; RUUDUN_LEVEYS * RUUDUN_KORKEUS];
    // pystyviiva x = 960, leveys yksi pikseli
    for rivi in ruutu.chunks_mut(RUUDUN_LEVEYS) {
        rivi[960] = 0xFFFFFF;
    }
    let (l, k) = (960, 700);
    let sovitus = Sovitus::new(l, k);
    assert_eq!(sovitus.k, 540);
    let mut kuva = Ikkunakuva::default();
    kuva.sovita(&ruutu, &KAIKKI_RIVIT, l, k, sovitus);
    let ikkuna = &kuva.pikselit;
    assert_eq!(ikkuna[0], PALKKI);
    assert_eq!(ikkuna[l * k - 1], PALKKI);
    let rivi = &ikkuna[sovitus.y * l..(sovitus.y + 1) * l];
    assert_eq!(rivi[480], 0x7F7F7F);
    assert_eq!(rivi[479], 0);

    // sama koko on suora kopio
    let sama = Sovitus::new(RUUDUN_LEVEYS, RUUDUN_KORKEUS);
    kuva.sovita(&ruutu, &KAIKKI_RIVIT, RUUDUN_LEVEYS, RUUDUN_KORKEUS, sama);
    assert_eq!(kuva.pikselit, ruutu);
}

#[test]
fn sovitus_kertoo_kuvan_muuttumisesta() {
    let mut ruutu = vec![0; RUUDUN_LEVEYS * RUUDUN_KORKEUS];
    let mut kuva = Ikkunakuva::default();
    let sovitus = Sovitus::new(960, 700);
    let kaikki = &KAIKKI_RIVIT;
    assert!(kuva.sovita(&ruutu, kaikki, 960, 700, sovitus));
    assert!(!kuva.sovita(&ruutu, kaikki, 960, 700, sovitus));
    ruutu[5] = 1;
    assert!(kuva.sovita(&ruutu, kaikki, 960, 700, sovitus));
    assert!(!kuva.sovita(&ruutu, kaikki, 960, 700, sovitus));
    // uusi ikkunan koko piirtää kaiken, vaikka kuva olisi sama
    let suurempi = Sovitus::new(1000, 700);
    assert!(kuva.sovita(&ruutu, kaikki, 1000, 700, suurempi));
}

#[test]
fn sovitus_etsii_muutoksia_vain_ehdokasriveilta() {
    let mut ruutu = vec![0; RUUDUN_LEVEYS * RUUDUN_KORKEUS];
    let mut kuva = Ikkunakuva::default();
    let sovitus = Sovitus::new(960, 700);
    kuva.sovita(&ruutu, &KAIKKI_RIVIT, 960, 700, sovitus);
    ruutu[5] = 1;
    assert!(!kuva.sovita(&ruutu, &[false; RUUDUN_KORKEUS], 960, 700, sovitus));
    assert!(kuva.sovita(&ruutu, &KAIKKI_RIVIT, 960, 700, sovitus));
    // Uusi koko laskee kaiken ehdokkaista välittämättä.
    let suurempi = Sovitus::new(1000, 700);
    assert!(kuva.sovita(&ruutu, &[false; RUUDUN_KORKEUS], 1000, 700, suurempi));
    let mut uusi = Ikkunakuva::default();
    uusi.sovita(&ruutu, &KAIKKI_RIVIT, 1000, 700, suurempi);
    assert!(kuva.pikselit == uusi.pikselit);
}

#[test]
fn muuttuneiden_rivien_sovitus_vastaa_kokonaan_laskettua() {
    let mut ruutu: Vec<u32> = (0..RUUDUN_LEVEYS * RUUDUN_KORKEUS)
        .map(|i| (i as u32).wrapping_mul(0x9E37_79B1) & 0xFFFFFF)
        .collect();
    let mut kuva = Ikkunakuva::default();
    // pienennys, palkit, suurennus ja takaisin ensimmäiseen kokoon
    for (l, k) in [(1760, 990), (960, 700), (2400, 1200), (1760, 990)] {
        let sovitus = Sovitus::new(l, k);
        kuva.sovita(&ruutu, &KAIKKI_RIVIT, l, k, sovitus);
        for (x, y) in [(0, 0), (1919, 1079), (700, 500), (701, 501)] {
            ruutu[y * RUUDUN_LEVEYS + x] ^= 0x00FF00;
            kuva.sovita(&ruutu, &KAIKKI_RIVIT, l, k, sovitus);
            let mut uusi = Ikkunakuva::default();
            uusi.sovita(&ruutu, &KAIKKI_RIVIT, l, k, sovitus);
            assert!(kuva.pikselit == uusi.pikselit, "{l}x{k} ({x}, {y})");
        }
        // Eri kohdat muuttuvat samassa ruudussa: pallo ylhäällä, maila alhaalla.
        ruutu[100 * RUUDUN_LEVEYS + 50] ^= 0x00FF00;
        ruutu[900 * RUUDUN_LEVEYS + 1800] ^= 0x00FF00;
        kuva.sovita(&ruutu, &KAIKKI_RIVIT, l, k, sovitus);
        let mut uusi = Ikkunakuva::default();
        uusi.sovita(&ruutu, &KAIKKI_RIVIT, l, k, sovitus);
        assert!(kuva.pikselit == uusi.pikselit, "{l}x{k}");
    }
}
