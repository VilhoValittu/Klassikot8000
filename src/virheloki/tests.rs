use super::*;

#[test]
fn rivit_lisataan_ja_liian_iso_loki_aloitetaan_alusta() {
    let kansio = std::env::temp_dir().join(format!("klassikot8000_loki_{}", std::process::id()));
    let polku = kansio.join("virheet.txt");
    let _ = fs::remove_dir_all(&kansio);

    lisaa_riville(&polku, "eka").unwrap();
    lisaa_riville(&polku, "toka").unwrap();
    assert_eq!(fs::read_to_string(&polku).unwrap(), "eka\ntoka\n");

    fs::write(&polku, "x".repeat(SUURIN_KOKO as usize + 1)).unwrap();
    lisaa_riville(&polku, "uusi").unwrap();
    assert_eq!(fs::read_to_string(&polku).unwrap(), "uusi\n");

    fs::remove_dir_all(&kansio).unwrap();
}
