use super::*;

#[test]
fn pelin_merkit_loytyvat_ja_tuntematon_on_kysymysmerkki() {
    assert_ne!(kuvio('ä'), kuvio('a'));
    assert_ne!(kuvio('Ö'), kuvio('O'));
    assert_eq!(kuvio('z')[2], 0x3F);
    assert_eq!(kuvio('€'), kuvio('?'));
}
