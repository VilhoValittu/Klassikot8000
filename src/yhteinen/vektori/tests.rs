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
fn viiva_on_konepikselin_paksuinen_ja_vino_reuna_pehmea() {
    let mut kuva = tyhja();
    Kangas::new(&mut kuva).viiva((10, 10), (40, 10), 0xFFFFFF);
    // Vaakaviiva on tasan kolme pikseliä paksu, joten sen reunat ovat terävät.
    for y in 9..12 {
        assert_eq!(pikseli(&kuva, 20, y), 0xFFFFFF);
    }
    assert_eq!(pikseli(&kuva, 20, 8), 0);
    assert_eq!(pikseli(&kuva, 20, 12), 0);

    let mut kuva = tyhja();
    Kangas::new(&mut kuva).viiva((10, 10), (40, 25), 0xFFFFFF);
    let valisavyja = kuva.iter().filter(|&&v| v != 0 && v != 0xFFFFFF).count();
    assert!(valisavyja > 30, "{valisavyja}");
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
fn monikulmion_vino_reuna_saa_valisavyn() {
    let mut kuva = tyhja();
    Kangas::new(&mut kuva).monikulmio(&[(0, 0), (30, 0), (0, 30)], 0xFFFFFF);
    assert_eq!(pikseli(&kuva, 5, 5), 0xFFFFFF);
    // Lävistäjällä oleva pikseli on puoliksi kolmion sisällä.
    let puolikas = pikseli(&kuva, 15, 14);
    assert!(puolikas != 0 && puolikas != 0xFFFFFF, "{puolikas:06X}");
    assert_eq!(pikseli(&kuva, 20, 20), 0);
}

#[test]
fn kiekko_ja_ympyra_pehmenevat_reunoilta() {
    let mut kuva = tyhja();
    let mut k = Kangas::new(&mut kuva);
    k.kiekko((100, 100), 20, 0xFFFFFF);
    k.ympyra((300, 100), 20.0, 0xFFFFFF);
    assert_eq!(pikseli(&kuva, 100, 100), 0xFFFFFF);
    assert_eq!(pikseli(&kuva, 100, 125), 0);
    assert!((0..LEVEYS).any(|x| {
        let v = pikseli(&kuva, x, 100);
        v != 0 && v != 0xFFFFFF
    }));
    // Kehä on ontto.
    assert_eq!(pikseli(&kuva, 300, 100), 0);
    assert_eq!(pikseli(&kuva, 320, 100), 0xFFFFFF);
}

#[test]
fn sekoitus_kokonaisluvuin_osuu_paihin() {
    assert_eq!(sekoita_alfa(0x123456, 0xABCDEF, 0), 0x123456);
    assert_eq!(sekoita_alfa(0x123456, 0xABCDEF, 255), 0xABCDEF);
    assert_eq!(sekoita_alfa(0x000000, 0xFFFFFF, 128), 0x808080);
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
fn tarkka_rivi_leikkautuu_reunoilla() {
    let mut kuva = tyhja();
    let mut k = Kangas::new(&mut kuva);
    k.tarkka_rivi(-1, 0, &[1, 2, 3]);
    k.tarkka_rivi(LEVEYS as i32 - 2, 1, &[4, 5, 6]);
    k.tarkka_rivi(0, -1, &[9]);
    k.tarkka_rivi(0, KORKEUS as i32, &[9]);
    assert_eq!(&kuva[..3], &[2, 3, 0]);
    assert_eq!(&kuva[2 * LEVEYS - 2..2 * LEVEYS], &[4, 5]);
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
fn liuku_alkaa_ylavarista_ja_paattyy_alavariin() {
    let mut kuva = tyhja();
    Kangas::new(&mut kuva).liukulaatikko(0, 0, 30, 60, 0x000000, 0xF0F0F0);
    assert_eq!(pikseli(&kuva, 0, 0), 0);
    assert_eq!(pikseli(&kuva, 0, 59), 0xF0F0F0);
    assert!(pikseli(&kuva, 0, 30) > pikseli(&kuva, 0, 29));
    assert_eq!(pikseli(&kuva, 30, 0), 0);
}

#[test]
fn sprite_ohittaa_lapinakyvat_merkit_ja_yhdistaa_vinon_portaan() {
    let mut kuva = tyhja();
    let vari = |m| (m == b'#').then_some(0xFFFFFF);
    Kangas::new(&mut kuva).sprite((6, 6), &[".#", "#."], 3, vari);
    // Ruutujen keskukset: tyhjä jää tyhjäksi, täysi täydeksi.
    assert_eq!(pikseli(&kuva, 10, 7), 0xFFFFFF);
    assert_eq!(pikseli(&kuva, 7, 10), 0xFFFFFF);
    assert_eq!(pikseli(&kuva, 6, 6), 0);
    // Vino porras täyttyy osin tyhjän ruudun kulmasta.
    let kulma = pikseli(&kuva, 8, 8);
    assert!(kulma != 0, "{kulma:06X}");
}

fn suorakulmio(vasen: f32, yla: f32, oikea: f32, ala: f32, vari: u32) -> Ruututahko {
    Ruututahko {
        pisteet: [
            (vasen, yla),
            (oikea, yla),
            (oikea, ala),
            (vasen, ala),
            (vasen, ala),
        ],
        maara: 4,
        vari,
    }
}

#[test]
fn mallin_reuna_pehmenee_mutta_sisus_sailyy() {
    let mut kuva = tyhja();
    let tahko = suorakulmio(9.0, 9.0, 40.5, 39.0, 0xFF0000);
    piirra_naytteina(&mut Kangas::new(&mut kuva), &[tahko]);
    assert_eq!(pikseli(&kuva, 18, 18), 0xFF0000);
    assert_eq!(pikseli(&kuva, 39, 18), 0xFF0000);
    // Pikseli 40 on puoliksi tahkoa: väri on tahkon ja taustan välissä.
    let reuna = pikseli(&kuva, 40, 18);
    assert!(reuna != 0 && reuna != 0xFF0000, "{reuna:06X}");
    // Tahkon ulkopuoli ei muutu.
    assert_eq!(pikseli(&kuva, 41, 18), 0);
    assert_eq!(pikseli(&kuva, 18, 45), 0);
}

#[test]
fn vierekkaisten_tahkojen_saumasta_ei_nay_taustaa() {
    let mut kuva = tyhja();
    let vari = 0x80C040;
    let tahkot = [
        suorakulmio(9.0, 9.0, 25.5, 39.0, vari),
        suorakulmio(25.5, 9.0, 42.0, 39.0, vari),
    ];
    piirra_naytteina(&mut Kangas::new(&mut kuva), &tahkot);
    for x in 9..42 {
        assert_eq!(pikseli(&kuva, x, 20), vari, "{x}");
    }
}
