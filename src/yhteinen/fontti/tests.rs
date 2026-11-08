use super::*;

#[test]
fn pelin_merkit_loytyvat_ja_tuntematon_on_kysymysmerkki() {
    assert_ne!(kuvio('ä'), kuvio('a'));
    assert_ne!(kuvio('Ö'), kuvio('O'));
    assert_ne!(kuvio('ö'), kuvio('o'));
    assert_eq!(kuvio('z')[2], 0x3F);
    assert_eq!(kuvio('€'), kuvio('?'));
    assert_eq!(kuvio(' '), &[0; 8]);
}

#[test]
fn jokainen_tulostuva_ascii_merkki_ja_aakkonen_on_piirretty() {
    let kirjasin = &*KIRJASIN;
    for merkki in '!'..='z' {
        assert!(kirjasin.ascii[merkki as usize].is_some(), "{merkki}");
    }
    for merkki in ['Ä', 'Ö', 'ä', 'ö'] {
        assert!(kirjasin.muut.iter().any(|(m, _)| *m == merkki), "{merkki}");
    }
}

/// Tyhjä oikea sarake erottaa vierekkäiset merkit toisistaan.
#[test]
fn oikea_sarake_on_aina_tyhja() {
    for merkki in ('!'..='z').chain(['Ä', 'Ö', 'ä', 'ö']) {
        assert!(
            kuvio(merkki).iter().all(|rivi| rivi & 0x80 == 0),
            "{merkki}"
        );
    }
}

#[test]
fn isojen_ja_pienten_kirjainten_korkeudet_ovat_yhtenaiset() {
    let rivit = |merkki: char| {
        let kuvio = kuvio(merkki);
        let ylin = kuvio.iter().position(|&r| r != 0).unwrap_or(0);
        let alin = kuvio.iter().rposition(|&r| r != 0).unwrap_or(0);
        (ylin, alin)
    };
    for merkki in 'A'..='Z' {
        assert_eq!(rivit(merkki), (0, 6), "{merkki}");
    }
    for merkki in "acemnorsuvwxz".chars() {
        assert_eq!(rivit(merkki), (2, 6), "{merkki}");
    }
    for merkki in "gpqy".chars() {
        assert_eq!(rivit(merkki).1, 7, "{merkki}");
    }
}
