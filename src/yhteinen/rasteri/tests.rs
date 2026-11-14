use super::*;
use crate::yhteinen::piirto::{KORKEUS, LEVEYS};

#[test]
fn valisavy_antaa_paat_tarkasti_ja_puolivalin() {
    assert_eq!(valisavy(0x102030, 0xF0E0D0, 0.0), 0x102030);
    assert_eq!(valisavy(0x102030, 0xF0E0D0, 1.0), 0xF0E0D0);
    assert_eq!(valisavy(0x000000, 0xFEFEFE, 0.5), 0x7F7F7F);
    assert_eq!(valisavy(1, 2, -3.0), 1);
}

#[test]
fn liuku_vaihtuu_portaatta() {
    let a = liukuvari(&[0x000000, 0xFF0000], 0.50);
    let b = liukuvari(&[0x000000, 0xFF0000], 0.51);
    assert!(b > a, "{a:06X} {b:06X}");
}

#[test]
fn liuku_kestaa_paat_ja_kohina_pysyy_valilla() {
    for t in [-1.0, 0.0, 0.5, 1.0, 2.0] {
        liukuvari(&[1, 0xFFFFFF, 3], t);
    }
    assert_eq!(liukuvari(&[0x101010, 0xFFFFFF], 0.0), 0x101010);
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

#[test]
fn tarkka_kuva_pehmentaa_reunan_ja_piirtyy_pikseleittain() {
    // Vino reuna: vasemmalla musta, oikealla valkoinen.
    let rasteri = Rasteri::tarkka(4, 4, 3, |x, y| if x > y + 0.3 { 0xFFFFFF } else { 0 });
    assert_eq!((rasteri.tiheys(), rasteri.korkeus()), (3, 4));
    assert_eq!(rasteri.rivi(0).len(), 12);
    // Reunapikselit saavat välisävyn, kaukana reunasta väri on puhdas.
    let rivi = rasteri.rivi(5);
    assert_eq!(rivi[0], 0);
    assert_eq!(rivi[11], 0xFFFFFF);
    assert!(rivi.iter().any(|&v| v != 0 && v != 0xFFFFFF), "{rivi:06X?}");

    let mut kuva = vec![0x123456; LEVEYS * KORKEUS];
    let mut kangas = Kangas::new(&mut kuva);
    rasteri.piirra_alue(&mut kangas, 30, 60, (1, 1, 2, 2));
    // Alue alkaa konepikselistä (1, 1) eli tarkan kuvan pikselistä (3, 3).
    assert_eq!(kuva[60 * LEVEYS + 30], rasteri.rivi(3)[3]);
    assert_eq!(kuva[65 * LEVEYS + 35], rasteri.rivi(8)[8]);
    assert_eq!(kuva[66 * LEVEYS + 30], 0x123456);
    assert_eq!(kuva[60 * LEVEYS + 36], 0x123456);
}
