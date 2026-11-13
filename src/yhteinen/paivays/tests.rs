use super::*;

#[test]
fn paivamaara_on_suomalaisessa_muodossa() {
    assert_eq!(pvm(0), "01.01.1970");
    assert_eq!(pvm(951_825_600), "29.02.2000");
    // 31.12.2026 klo 23.59.59 ja minuuttia myöhemmin uusi vuosi Suomen ajassa.
    assert_eq!(pvm(1_798_754_399), "31.12.2026");
    assert_eq!(pvm(1_798_754_400), "01.01.2027");
}

#[test]
fn kesaaika_siirtaa_paivan_vaihdetta() {
    // 30.6. klo 21.30 UTC on kesäaikana jo 1.7. klo 0.30.
    assert_eq!(pvm(1_782_855_000), "01.07.2026");
    // Lokakuun viimeisen sunnuntain jälkeen ero on taas kaksi tuntia.
    assert_eq!(pvm(1_792_877_400), "25.10.2026");
    assert_eq!(pvm(1_792_963_800), "25.10.2026");
    // Kesäaika alkaa 29.3.2026 klo 1.00 UTC.
    assert!(!kesaaikaa(1_774_745_999));
    assert!(kesaaikaa(1_774_746_000));
}

#[test]
fn paivanumero_kaantyy_takaisin() {
    for numero in -1_000..60_000 {
        let (vuosi, kuukausi, paiva) = paiva_numerosta(numero);
        assert_eq!(numero_paivasta(vuosi, kuukausi, paiva), numero);
    }
}

#[test]
fn valtava_aika_ei_kaada() {
    let _ = pvm(u64::MAX);
}
