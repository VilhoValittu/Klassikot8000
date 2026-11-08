use super::*;

fn kanavat(vari: u32) -> [i32; 3] {
    [16, 8, 0].map(|siirto| (vari >> siirto & 0xFF) as i32)
}

fn kirkkaus(vari: u32) -> i32 {
    let [r, g, b] = kanavat(vari);
    r * 3 + g * 6 + b
}

#[test]
fn hsv_muunnos_palaa_samaksi_vareiksi() {
    for vari in [
        0xFFE860, 0x0848B8, 0x2C2070, 0xFFFFFF, 0x000000, 0xC8382A, 0x1C8050,
    ] {
        let (h, s, v) = hsv(vari);
        let takaisin = kanavat(rgb(h, s, v));
        for (a, b) in kanavat(vari).iter().zip(takaisin) {
            assert!((a - b).abs() <= 1, "{vari:06X}");
        }
    }
}

#[test]
fn porras_tummenee_varjoon_ja_vaalenee_valoon() {
    for vari in [0xFFE860, 0x0848B8, 0xFFFFFF, 0xC8382A, 0x8C84F0] {
        let porras = ramppi(vari);
        assert_eq!(porras[2], vari);
        // Varjot tummenevat aina, mutta valkoista ei voi vaalentaa.
        for (i, pari) in porras.windows(2).enumerate() {
            let (a, b) = (kirkkaus(pari[0]), kirkkaus(pari[1]));
            assert!(a < b || (i >= 2 && a == b), "{vari:06X}: {porras:06X?}");
        }
    }
}

#[test]
fn keltainen_tummuu_oranssiksi_ja_valkoinen_laventeliksi() {
    let [r, g, b] = kanavat(ramppi(0xFFE860)[0]);
    assert!(r > g && g > b, "oranssi: {r} {g} {b}");
    let [r, g, b] = kanavat(ramppi(0xFFFFFF)[1]);
    assert!(b > r && b > g, "laventeli: {r} {g} {b}");
}

#[test]
fn savy_kiertyy_lyhinta_tieta_eika_ohita_kohdetta() {
    assert_eq!(kierra_kohti(10.0, 350.0, 5.0), 5.0);
    assert_eq!(kierra_kohti(340.0, 20.0, 50.0), 20.0);
    assert_eq!(kierra_kohti(100.0, 50.0, 8.0), 92.0);
}

#[test]
fn varjo_tummentaa_osuuden_mukaan() {
    assert_eq!(varjoon(0xFFFFFF, 0), 0xFFFFFF);
    assert_eq!(varjoon(0xFFFFFF, 256), VARJO);
    let puoli = varjoon(0xFFFFFF, 128);
    assert!(kirkkaus(puoli) < kirkkaus(0xFFFFFF) && kirkkaus(puoli) > kirkkaus(VARJO));
}
