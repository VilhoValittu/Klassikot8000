use super::*;

#[test]
fn hiiri_alla_ja_painettuna_vain_alueen_sisalla() {
    unohda();
    assert_eq!(tila(10, 10, 20, 10), Tila::Lepo);
    kirjaa(Syote::Hiiri(15, 12));
    assert_eq!(tila(10, 10, 20, 10), Tila::Alla);
    assert_eq!(tila(30, 10, 20, 10), Tila::Lepo);
    kirjaa(Syote::Klikkaus(29, 19));
    assert_eq!(tila(10, 10, 20, 10), Tila::Painettu);
    // Painettuna vedetty hiiri ei paina muita nappeja kuin sen, jonka päällä se on.
    kirjaa(Syote::Hiiri(40, 12));
    assert_eq!(tila(10, 10, 20, 10), Tila::Lepo);
    assert_eq!(tila(30, 10, 20, 10), Tila::Painettu);
    kirjaa(Syote::Vapautus(40, 12));
    assert_eq!(tila(30, 10, 20, 10), Tila::Alla);
    kirjaa(Syote::Ammu);
    assert_eq!(tila(30, 10, 20, 10), Tila::Alla);
    unohda();
    assert_eq!(tila(30, 10, 20, 10), Tila::Lepo);
}
