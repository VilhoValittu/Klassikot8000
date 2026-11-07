use super::*;

#[test]
fn nimi_ja_jasennys_vastaavat_toisiaan() {
    for rivi in 0..KOKO {
        for sarake in 0..KOKO {
            let ruutu = Ruutu::new(rivi, sarake).unwrap();
            assert_eq!(ruutu.to_string().parse(), Ok(ruutu));
        }
    }
    assert_eq!(Ruutu::new(9, 1).unwrap().to_string(), "B10");
}

#[test]
fn virheelliset_nimet_hylataan() {
    for teksti in ["", "A", "A0", "A11", "A01", "K1", "a1", "1A", "A1 "] {
        assert!(teksti.parse::<Ruutu>().is_err(), "{teksti}");
    }
}

#[test]
fn suora_jono_alkaa_ensimmaisesta_ja_pysyy_laudalla() {
    let alku = Ruutu::new(2, 7).unwrap();
    let vaaka = Ruudut::suora(alku, 3, true).unwrap();
    assert_eq!(vaaka.maara(), 3);
    assert_eq!(vaaka.ensimmainen(), Some(alku));
    assert!(vaaka.sisaltaa(Ruutu::new(2, 9).unwrap()));
    assert_eq!(Ruudut::suora(alku, 4, true), None);
    let pysty = Ruudut::suora(alku, 4, false).unwrap();
    assert!(pysty.sisaltaa(Ruutu::new(5, 7).unwrap()));
    assert_eq!(Ruudut::default().ensimmainen(), None);
}

#[test]
fn joukko_toimii() {
    let (a, b) = (Ruutu::new(0, 0).unwrap(), Ruutu::new(9, 9).unwrap());
    let mut joukko = Ruudut::default();
    joukko.lisaa(a);
    assert!(joukko.sisaltaa(a) && !joukko.sisaltaa(b));
    joukko.lisaa(b);
    assert_eq!(joukko.maara(), 2);
}
