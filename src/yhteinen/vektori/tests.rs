use super::*;

fn tyhja() -> Vec<u32> {
    vec![0; LEVEYS * KORKEUS]
}

fn pikseli(kuva: &[u32], x: usize, y: usize) -> u32 {
    kuva[y * LEVEYS + x]
}

#[test]
fn viiva_ulottuu_paasta_paahan_ja_kuvan_ulkopuoli_ohitetaan() {
    let mut kuva = tyhja();
    let mut k = Kangas::new(&mut kuva);
    k.viiva((-50, -50), (LEVEYS as i32 + 50, KORKEUS as i32 + 50), 2);
    k.viiva((10, 10), (40, 25), 1);
    assert_eq!(pikseli(&kuva, 10, 10), 1);
    assert_eq!(pikseli(&kuva, 40, 25), 1);
}

#[test]
fn viiva_pysyy_konepikselin_ruudukossa() {
    let mut kuva = tyhja();
    Kangas::new(&mut kuva).viiva((10, 10), (10, 10), 1);
    assert_eq!(pikseli(&kuva, 9, 9), 1);
    assert_eq!(pikseli(&kuva, 11, 11), 1);
    assert_eq!(pikseli(&kuva, 12, 12), 0);
}

#[test]
fn monikulmio_tayttaa_sisaosan_muttei_ulkopuolta() {
    let mut kuva = tyhja();
    Kangas::new(&mut kuva).monikulmio(&[(10, 10), (30, 10), (20, 30)], 7);
    assert_eq!(pikseli(&kuva, 20, 15), 7);
    assert_eq!(pikseli(&kuva, 11, 25), 0);
}

#[test]
fn monikulmio_kuvan_reunan_yli_ei_kaada() {
    let mut kuva = tyhja();
    Kangas::new(&mut kuva).monikulmio(&[(-20, -20), (LEVEYS as i32 + 20, 0), (0, 2000)], 7);
    assert_eq!(pikseli(&kuva, 0, 0), 7);
}

#[test]
fn ruudutettu_monikulmio_vuorottelee_kahta_varia() {
    let mut kuva = tyhja();
    let nelio = [(0, 0), (30, 0), (30, 30), (0, 30)];
    Kangas::new(&mut kuva).ruudutettu_monikulmio(&nelio, 1, 2);
    assert_eq!(pikseli(&kuva, 0, 0), 1);
    assert_eq!(pikseli(&kuva, 3, 0), 2);
    assert_eq!(pikseli(&kuva, 0, 3), 2);
    assert_eq!(pikseli(&kuva, 3, 3), 1);
}

#[test]
fn malli_piirtyy_kaikissa_kulmissa() {
    let malli = taistelulaiva(0x111111, 0x222222, 0x333333);
    for i in 0..32 {
        let mut kuva = tyhja();
        Kangas::new(&mut kuva).malli(&malli, i as f32 * 0.2, (480, 225), 33.0);
        assert!(kuva.iter().filter(|&&p| p != 0).count() > 500);
    }
}

#[test]
fn varjostus_ei_ylita_kanavaa() {
    assert_eq!(varjosta(0xFF8040, 2.0), 0xFFFF80);
    assert_eq!(varjosta(0xFF8040, 0.5), 0x7F4020);
}

#[test]
fn karkea_rivi_venyy_konepikseleiksi_ja_leikkautuu_reunoilla() {
    let mut kuva = tyhja();
    let mut k = Kangas::new(&mut kuva);
    k.karkea_rivi(-3, -1, &[1, 2, 3]);
    k.karkea_rivi(LEVEYS as i32 - 4, 30, &[4, 5, 6]);
    k.karkea_rivi(0, KORKEUS as i32 + 1, &[9]);
    assert_eq!(pikseli(&kuva, 0, 0), 2);
    assert_eq!(pikseli(&kuva, 2, 1), 2);
    assert_eq!(pikseli(&kuva, 3, 0), 3);
    assert_eq!(pikseli(&kuva, 0, 2), 0);
    assert_eq!(pikseli(&kuva, LEVEYS - 1, 32), 5);
}

#[test]
fn liuku_alkaa_ylavarista_ja_lahestyy_alavaria() {
    let mut kuva = tyhja();
    Kangas::new(&mut kuva).liukulaatikko(0, 0, 30, 60, 0x000000, 0xF0F0F0);
    assert_eq!(pikseli(&kuva, 0, 0), 0);
    assert!(pikseli(&kuva, 0, 59) > 0x800000);
    assert_eq!(pikseli(&kuva, 30, 0), 0);
}

#[test]
fn sprite_ohittaa_lapinakyvat_merkit() {
    let mut kuva = tyhja();
    let vari = |m| (m == b'#').then_some(5);
    Kangas::new(&mut kuva).sprite((6, 6), &[".#", "#."], 3, vari);
    assert_eq!(pikseli(&kuva, 6, 6), 0);
    assert_eq!(pikseli(&kuva, 9, 6), 5);
    assert_eq!(pikseli(&kuva, 6, 9), 5);
}

fn suorakulmio(vasen: f32, yla: f32, oikea: f32, ala: f32, a: u32, b: u32) -> Ruututahko {
    Ruututahko {
        pisteet: [
            (vasen, yla),
            (oikea, yla),
            (oikea, ala),
            (vasen, ala),
            (vasen, ala),
        ],
        maara: 4,
        a,
        b,
    }
}

#[test]
fn mallin_reuna_pehmenee_mutta_sisus_ja_ditterointi_sailyvat() {
    let mut kuva = tyhja();
    let tahko = suorakulmio(9.0, 9.0, 40.5, 39.0, 0xFF0000, 0x0000FF);
    piirra_naytteina(&mut Kangas::new(&mut kuva), &[tahko]);
    // Konepikseli on 3: sisuksen ruudutus vuorottelee konepikseleittäin.
    assert_eq!(pikseli(&kuva, 18, 18), 0xFF0000);
    assert_eq!(pikseli(&kuva, 21, 18), 0x0000FF);
    assert_eq!(pikseli(&kuva, 20, 20), 0xFF0000);
    // Konepikseli 39..42 on puoliksi tahkoa: väri on tahkon ja taustan välissä.
    let reuna = pikseli(&kuva, 40, 18);
    assert!(
        reuna != 0 && reuna != 0xFF0000 && reuna != 0x0000FF,
        "{reuna:06X}"
    );
    assert_eq!(pikseli(&kuva, 41, 18), reuna);
    // Tahkon ulkopuoli ei muutu.
    assert_eq!(pikseli(&kuva, 45, 18), 0);
    assert_eq!(pikseli(&kuva, 18, 45), 0);
}

#[test]
fn vierekkaisten_tahkojen_saumasta_ei_nay_taustaa() {
    let mut kuva = tyhja();
    let vari = 0x80C040;
    let tahkot = [
        suorakulmio(9.0, 9.0, 25.5, 39.0, vari, vari),
        suorakulmio(25.5, 9.0, 42.0, 39.0, vari, vari),
    ];
    piirra_naytteina(&mut Kangas::new(&mut kuva), &tahkot);
    for x in 9..42 {
        assert_eq!(pikseli(&kuva, x, 20), vari, "{x}");
    }
}
