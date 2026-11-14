use super::*;

fn ruudukko(kuva: &'static [&'static str]) -> impl Fn(i32, i32) -> Option<u32> {
    move |rivi, sarake| (kuva[rivi as usize].as_bytes()[sarake as usize] == b'#').then_some(7)
}

fn alfa(peitto: &Peitto, x: usize, y: usize) -> u8 {
    peitto.pikselit[y * peitto.leveys + x].1
}

#[test]
fn taysi_solu_peittaa_ja_tyhja_jaa_lapinakyvaksi() {
    let kuva: &[&str] = &["##..", "##..", "....", "...."];
    let peitto = pehmenna(4, 4, (3, 3), ruudukko(kuva));
    assert_eq!((peitto.leveys, peitto.korkeus), (12, 12));
    assert_eq!(peitto.pikselit[3 * 12 + 3], (7, 255));
    assert_eq!(alfa(&peitto, 9, 9), 0);
}

#[test]
fn ohuen_viivan_paa_pysyy_teravana() {
    // Vaakaviivan pään yläkulmaa ei pyöristetä: alla ei ole mitään, joten kulma ei ole porras.
    let kuva: &[&str] = &["....", "###.", "....", "...."];
    let peitto = pehmenna(4, 4, (3, 3), ruudukko(kuva));
    assert_eq!(alfa(&peitto, 0, 3), 255);
    assert_eq!(alfa(&peitto, 8, 5), 255);
}

#[test]
fn vino_porras_tayttyy_ja_kupera_kulma_pyoristyy() {
    // Kahden pikselin levyinen vino viiva: sisäkulma täyttyy osin ja ulkokulma lohkeaa.
    let kuva: &[&str] = &["##..", ".##.", "..##", "...."];
    let peitto = pehmenna(4, 4, (3, 3), ruudukko(kuva));
    // Tyhjän solun (0, 2) vasen alakulma.
    let sisakulma = alfa(&peitto, 6, 2);
    assert!(sisakulma > 0 && sisakulma < 255, "{sisakulma}");
    // Täyden solun (1, 1) vasen alakulma on porras: se lohkeaa.
    let ulkokulma = alfa(&peitto, 3, 5);
    assert!(ulkokulma < 255, "{ulkokulma}");
    // Solujen keskiosa ei muutu.
    assert_eq!(alfa(&peitto, 4, 4), 255);
}

#[test]
fn erilliset_osat_eivat_yhdisty() {
    // Ääkkösen pisteet ovat kirjaimesta tyhjän rivin päässä.
    let kuva: &[&str] = &["#.#.", "....", "###.", "...."];
    let peitto = pehmenna(4, 4, (3, 3), ruudukko(kuva));
    for x in 0..12 {
        assert_eq!(alfa(&peitto, x, 4), 0, "{x}");
    }
}
