use klassikot8000::Arpa;
use klassikot8000::miinaharava::{Avaus, Miinakentta, Paikka, Tila, Vaikeus};

fn paikat(kentta: &Miinakentta) -> Vec<Paikka> {
    let vaikeus = kentta.vaikeus;
    (0..vaikeus.korkeus())
        .flat_map(|r| (0..vaikeus.leveys()).map(move |s| (r, s)))
        .collect()
}

#[test]
fn ensimmainen_avaus_on_turvallinen_ja_miinoja_on_oikea_maara() {
    for vaikeus in Vaikeus::KAIKKI {
        for siemen in 0..50 {
            let mut arpa = Arpa::new(siemen);
            let mut kentta = Miinakentta::new(vaikeus);
            assert_eq!(kentta.tila, Tila::Alussa);
            let eka = (siemen as usize % vaikeus.korkeus(), 3);
            assert!(matches!(kentta.avaa(eka, &mut arpa), Avaus::Auki(_)));
            assert_eq!(kentta.tila, Tila::Kesken);
            let kaikki = paikat(&kentta);
            let miinoja = kaikki.iter().filter(|&&p| kentta.ruutu(p).miina).count();
            assert_eq!(miinoja, vaikeus.miinoja());
            assert_eq!(kentta.ruutu(eka).naapureita, 0);
            assert!(kentta.naapurit(eka).all(|q| !kentta.ruutu(q).miina));
            assert!(kentta.naapurit(eka).all(|q| kentta.ruutu(q).auki));
            // jokainen numero vastaa naapurien miinoja
            for p in kaikki {
                let laskettu = kentta
                    .naapurit(p)
                    .filter(|&q| kentta.ruutu(q).miina)
                    .count();
                assert_eq!(kentta.ruutu(p).naapureita as usize, laskettu);
            }
        }
    }
}

/// Kulmamiina ja rivin 4 täyttävä miinaseinä, jonka yli avaus ei leviä.
fn seinallinen() -> Miinakentta {
    let mut miinat = vec![(0, 0)];
    miinat.extend((0..9).map(|s| (4, s)));
    Miinakentta::miinoista(Vaikeus::Helppo, &miinat)
}

#[test]
fn avaus_leviaa_tyhjista_ruuduista_ja_pysahtyy_numeroihin() {
    let mut arpa = Arpa::new(1);
    let mut kentta = seinallinen();
    assert_eq!(kentta.avaa((1, 1), &mut arpa), Avaus::Auki(1));
    assert!(!kentta.ruutu((1, 2)).auki);
    assert_eq!(kentta.avaa((1, 1), &mut arpa), Avaus::Ei);
    assert_eq!(kentta.avaa((8, 0), &mut arpa), Avaus::Auki(36));
    assert!(!kentta.ruutu((4, 4)).auki);
    assert!(!kentta.ruutu((3, 3)).auki);
    assert!(kentta.ruutu((5, 3)).auki);
    assert_eq!(kentta.ruutu((5, 3)).naapureita, 3);
}

#[test]
fn lippu_estaa_avauksen_ja_laskuri_voi_menna_miinukselle() {
    let mut arpa = Arpa::new(1);
    let mut kentta = Miinakentta::miinoista(Vaikeus::Helppo, &[(0, 0)]);
    assert!(kentta.liputa((0, 0)));
    assert_eq!(kentta.avaa((0, 0), &mut arpa), Avaus::Ei);
    assert_eq!(kentta.tila, Tila::Kesken);
    assert_eq!(kentta.miinoja_jaljella(), 0);
    assert!(kentta.liputa((0, 0)));
    assert_eq!(kentta.miinoja_jaljella(), 1);

    let mut paljon = Miinakentta::new(Vaikeus::Helppo);
    for s in 0..9 {
        paljon.liputa((0, s));
        paljon.liputa((1, s));
    }
    assert_eq!(paljon.miinoja_jaljella(), -8);

    let mut kentta = Miinakentta::miinoista(Vaikeus::Helppo, &[(0, 0), (4, 4)]);
    kentta.avaa((1, 1), &mut arpa);
    assert!(!kentta.liputa((1, 1)));
}

#[test]
fn miina_paattaa_pelin_eika_mikaan_muutu_sen_jalkeen() {
    let mut arpa = Arpa::new(1);
    let mut kentta = Miinakentta::miinoista(Vaikeus::Helppo, &[(0, 0), (4, 4)]);
    assert_eq!(kentta.avaa((4, 4), &mut arpa), Avaus::Miina);
    assert_eq!(kentta.tila, Tila::Tappio);
    assert_eq!(kentta.rajahtanyt, Some((4, 4)));
    assert_eq!(kentta.avaa((8, 0), &mut arpa), Avaus::Ei);
    assert!(!kentta.liputa((0, 8)));
    assert!(!kentta.ruutu((8, 0)).auki);
}

#[test]
fn voitto_liputtaa_miinat_ja_nollaa_laskurin() {
    let mut arpa = Arpa::new(1);
    let mut kentta = Miinakentta::miinoista(Vaikeus::Helppo, &[(0, 0), (8, 8)]);
    let mut viimeinen = Avaus::Ei;
    for p in paikat(&kentta) {
        if !kentta.ruutu(p).miina {
            viimeinen = match kentta.avaa(p, &mut arpa) {
                Avaus::Ei => viimeinen,
                avaus => avaus,
            };
        }
    }
    assert_eq!(viimeinen, Avaus::Voitto);
    assert_eq!(kentta.tila, Tila::Voitto);
    assert!(kentta.ruutu((0, 0)).lippu && kentta.ruutu((8, 8)).lippu);
    assert_eq!(kentta.miinoja_jaljella(), 0);
}

#[test]
fn numeron_avaus_vaatii_oikean_lippumaaran() {
    let mut arpa = Arpa::new(1);
    let mut kentta = seinallinen();
    kentta.avaa((1, 1), &mut arpa);
    assert_eq!(kentta.avaa_ymparilta((1, 1), &mut arpa), Avaus::Ei);
    kentta.liputa((0, 0));
    assert!(matches!(kentta.avaa_ymparilta((1, 1), &mut arpa), Avaus::Auki(n) if n > 0));
    assert!(kentta.ruutu((0, 1)).auki);
    assert!(kentta.ruutu((2, 2)).auki);
    // avaamaton ruutu ei kelpaa keskipisteeksi
    assert_eq!(kentta.avaa_ymparilta((0, 0), &mut arpa), Avaus::Ei);
}

#[test]
fn vaara_lippu_rajayttaa_numeron_avauksessa() {
    let mut arpa = Arpa::new(1);
    let mut kentta = Miinakentta::miinoista(Vaikeus::Helppo, &[(0, 0), (4, 4)]);
    kentta.avaa((1, 1), &mut arpa);
    kentta.liputa((2, 2));
    assert_eq!(kentta.avaa_ymparilta((1, 1), &mut arpa), Avaus::Miina);
    assert_eq!(kentta.rajahtanyt, Some((0, 0)));
}

#[test]
fn kulman_naapureita_on_kolme_ja_keskella_kahdeksan() {
    let kentta = Miinakentta::new(Vaikeus::Vaikea);
    assert_eq!(kentta.naapurit((0, 0)).count(), 3);
    assert_eq!(kentta.naapurit((15, 29)).count(), 3);
    assert_eq!(kentta.naapurit((5, 5)).count(), 8);
    assert_eq!(kentta.naapurit((0, 5)).count(), 5);
}

#[test]
fn nopeampi_voitto_ja_vaikeampi_taso_antavat_enemman_pisteita() {
    use klassikot8000::miinaharava::{SUURIN_AIKA, aikapisteet};

    for vaikeus in Vaikeus::KAIKKI {
        assert!(aikapisteet(vaikeus, 10) > aikapisteet(vaikeus, 11));
        assert!(aikapisteet(vaikeus, SUURIN_AIKA) > 0);
        assert!(aikapisteet(vaikeus, u32::MAX) > 0);
    }
    assert!(aikapisteet(Vaikeus::Vaikea, 100) > aikapisteet(Vaikeus::Normaali, 100));
    assert!(aikapisteet(Vaikeus::Normaali, 100) > aikapisteet(Vaikeus::Helppo, 100));
    // Palvelin hylkää tätä suuremmat tulokset.
    assert!(aikapisteet(Vaikeus::Vaikea, 1) <= 3000);
}
