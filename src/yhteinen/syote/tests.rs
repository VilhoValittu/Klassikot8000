use super::*;

#[test]
fn yleinen_syote_lajitellaan_nimen_aanten_ja_pelin_kesken() {
    let kytkin = |x: i32, y: i32| (x, y) == (1, 1);
    let mut aanet = true;
    let lajittele = |syote, nimi, aanet: &mut bool| yleinen(syote, nimi, aanet, kytkin);

    // Nimeä kirjoittaessa kirjaimet, M ja Esc kuuluvat nimikentälle.
    for syote in [
        Syote::Kirjain('M'),
        Syote::VaihdaAanet,
        Syote::Peru,
        Syote::Ammu,
    ] {
        assert_eq!(lajittele(syote, true, &mut aanet), Yleinen::Nimi);
    }
    assert!(aanet);
    // Klikkaus ohittaa nimikentän: kytkin vaihtaa äänet, muu kohta ei tee mitään.
    assert_eq!(
        lajittele(Syote::Klikkaus(1, 1), true, &mut aanet),
        Yleinen::Aanet
    );
    assert!(!aanet);
    assert_eq!(
        lajittele(Syote::Klikkaus(5, 5), true, &mut aanet),
        Yleinen::Ohita
    );

    assert_eq!(lajittele(Syote::Peru, false, &mut aanet), Yleinen::Peru);
    assert_eq!(
        lajittele(Syote::VaihdaAanet, false, &mut aanet),
        Yleinen::Aanet
    );
    assert!(aanet);
    assert_eq!(
        lajittele(Syote::Klikkaus(5, 5), false, &mut aanet),
        Yleinen::Peli
    );
    assert_eq!(lajittele(Syote::Ammu, false, &mut aanet), Yleinen::Peli);
    assert!(aanet);
}
