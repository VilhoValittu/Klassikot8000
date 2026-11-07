use super::*;
use crate::yhteinen::piirto::{KORKEUS, LEVEYS};

#[test]
fn ditterointi_antaa_paat_tarkasti() {
    for y in 0..4 {
        for x in 0..4 {
            assert_eq!(ditteroi(x, y, 1, 2, 0.0), 1);
            assert_eq!(ditteroi(x, y, 1, 2, 1.0), 2);
        }
    }
    let puolet = (0..16)
        .filter(|i| ditteroi(i % 4, i / 4, 1, 2, 0.5) == 2)
        .count();
    assert_eq!(puolet, 8);
}

#[test]
fn liuku_kestaa_paat_ja_kohina_pysyy_valilla() {
    for t in [-1.0, 0.0, 0.5, 1.0, 2.0] {
        liukuvari(&[1, 0xFFFFFF, 3], t, 0, 0);
    }
    assert_eq!(liukuvari(&[0x101010, 0xFFFFFF], 0.0, 0, 0), 0x101010);
    for i in 0..1000 {
        assert!((0.0..1.0).contains(&kohina(i, i * 3)));
    }
}

#[test]
fn piirto_paitsi_alue_jattaa_alueen_koskematta() {
    let mut kuva = vec![0; LEVEYS * KORKEUS];
    let rasteri = Rasteri::luo(320, 256, |_, _| 7);
    let mut kangas = Kangas::new(&mut kuva);
    rasteri.piirra_paitsi(&mut kangas, (10, 10, 100, 100), 256);
    let pikseli = |x: usize, y: usize| kuva[y * LEVEYS + x];
    assert_eq!(pikseli(0, 0), 7);
    assert_eq!(pikseli(29, 150), 7);
    assert_eq!(pikseli(30, 30), 0);
    assert_eq!(pikseli(329, 329), 0);
    assert_eq!(pikseli(330, 150), 7);
    assert_eq!(pikseli(150, 330), 7);
    assert_eq!(pikseli(LEVEYS - 1, KORKEUS - 1), 7);
}
