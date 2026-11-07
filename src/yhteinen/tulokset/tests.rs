use super::*;

#[test]
fn parhaat_pysyvat_jarjestyksessa_ja_tasapisteilla_vanhempi_sailyy() {
    let tulos = |nimi: &str, pisteet| Tulos {
        nimi: nimi.to_string(),
        pisteet,
    };
    let mut parhaat = Vec::new();
    assert!(!mahtuu_parhaisiin(&parhaat, 0));
    for (nimi, pisteet) in [("B", 200), ("C", 100), ("A", 300)] {
        assert!(mahtuu_parhaisiin(&parhaat, pisteet));
        lisaa_tulos(&mut parhaat, tulos(nimi, pisteet));
    }
    assert!(!mahtuu_parhaisiin(&parhaat, 100));
    assert!(mahtuu_parhaisiin(&parhaat, 101));
    lisaa_tulos(&mut parhaat, tulos("D", 200));
    assert_eq!(parhaat, [tulos("A", 300), tulos("B", 200), tulos("D", 200)]);
}

#[test]
fn lista_luetaan_tekstista_ja_kelvottomat_rivit_ohitetaan() {
    let lista = lista_tekstista("5 PIENI\n9 VÄINÖ\n0 NOLLA\nkissa\n7 pieni\n2000000000 ISO\n");
    let pisteet: Vec<u32> = lista.iter().map(|t| t.pisteet).collect();
    assert_eq!(pisteet, [9, 5]);
    assert_eq!(lista[0].nimi, "VÄINÖ");
}

#[test]
fn saman_nimen_tulos_korvataan() {
    let tulos = |nimi: &str, pisteet| Tulos {
        nimi: nimi.to_string(),
        pisteet,
    };
    let mut lista = vec![tulos("A", 300), tulos("B", 200), tulos("C", 100)];
    paivita_nimen_tulos(&mut lista, tulos("C", 400));
    assert_eq!(lista, [tulos("C", 400), tulos("A", 300), tulos("B", 200)]);
}

#[test]
fn nettipelien_jarjestys_ja_tunnukset_tasmaavat() {
    for (i, peli) in Nettipeli::KAIKKI.into_iter().enumerate() {
        assert_eq!(peli as usize, i);
        let samat = Nettipeli::KAIKKI
            .iter()
            .filter(|p| p.tunnus() == peli.tunnus());
        assert_eq!(samat.count(), 1, "{peli:?}");
    }
}

#[test]
fn taulu_kirjaa_tuloksen_kaikille_listoille_ja_lahtemaan() {
    let mut taulu = Tulostaulu::new(Nettipeli::Luvut);
    let mut nimi = "A".to_string();
    taulu.kirjaa(&mut nimi, 50);
    assert!(nimi.is_empty());
    assert_eq!(taulu.nakyva(), taulu.parhaat);
    taulu.netti = Some(Vec::new());
    assert!(taulu.nakyva().is_empty());
    nimi.push('B');
    taulu.kirjaa(&mut nimi, 40);
    let b = Tulos {
        nimi: "B".to_string(),
        pisteet: 40,
    };
    assert_eq!(taulu.parhaat.len(), 2);
    assert_eq!(taulu.nakyva(), std::slice::from_ref(&b));
    assert_eq!(taulu.lahetys, Some(b));

    // Yhden rivin listalla saman nimen uusi tulos korvaa vanhan.
    let mut haasteet = Tulostaulu::new(Nettipeli::Haasteet);
    for pisteet in [10, 20] {
        nimi.push('C');
        haasteet.kirjaa(&mut nimi, pisteet);
    }
    assert_eq!(haasteet.parhaat.len(), 1);
    assert_eq!(haasteet.parhaat[0].pisteet, 20);
}

#[test]
fn tyhja_nimi_saa_oletuksen() {
    let mut nimi = String::new();
    assert_eq!(ota_nimi(&mut nimi), kaanna(OLETUSNIMI));
    muokkaa_nimea(&mut nimi, Syote::Kirjain('A'));
    assert_eq!(ota_nimi(&mut nimi), "A");
    assert!(nimi.is_empty());
}
