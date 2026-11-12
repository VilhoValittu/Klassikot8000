use super::*;
use crate::yhteinen::tulokset::Tulos;
use klassikot8000::mahjong::{PARIN_PISTEET, Paikka, TYHJENNYSBONUS};

/// Neljä erillistä laattaa, kaksi paria: kaikki ovat vapaita.
fn pieni_lauta(vaikeus: Vaikeus, laatat: &[u8]) -> Lauta {
    let paikat = (0..laatat.len())
        .map(|i| Paikka {
            x: 4 * i as i8,
            y: 0,
            z: 0,
        })
        .collect();
    let laatat = laatat.iter().map(|&l| Some(Laatta(l))).collect();
    Lauta::laatoista(vaikeus, paikat, laatat)
}

fn pelissa(laatat: &[u8]) -> Mahjong {
    let mut peli = Mahjong::new(1);
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
    peli.aanet.jono.clear();
    peli.lauta = pieni_lauta(Vaikeus::Helppo, laatat);
    peli
}

fn klikkaa(peli: &mut Mahjong, i: usize) {
    let (x, y) = piirto::laatan_keskipiste(&peli.lauta, i);
    peli.kasittele(Syote::Klikkaus(x, y));
}

#[test]
fn taso_valitaan_alussa_ja_peli_alkaa_silla() {
    let mut peli = Mahjong::new(1);
    peli.kasittele(Syote::Numero(3));
    assert_eq!(peli.vaikeus, Vaikeus::Vaikea);
    peli.kasittele(Syote::Nuoli(0, -1));
    assert_eq!(peli.vaikeus, Vaikeus::Normaali);
    let (x, y) = piirto::vaikeuden_keskipiste(Vaikeus::Helppo);
    peli.kasittele(Syote::Klikkaus(x, y));
    assert_eq!(peli.vaikeus, Vaikeus::Helppo);
    assert_eq!(peli.vaihe, Vaihe::Alku);
    peli.kasittele(Syote::Numero(2));
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
    assert_eq!(peli.lauta.vaikeus, Vaikeus::Normaali);
    assert_eq!(peli.lauta.jaljella(), 124);
    assert_eq!(peli.aanet.jono, [(Aani::Alku, 0)]);
}

#[test]
fn klikkaukset_valitsevat_ja_ottavat_parin() {
    let mut peli = pelissa(&[1, 2, 1, 2]);
    klikkaa(&mut peli, 0);
    assert_eq!(peli.valittu, Some(0));
    // Väärä pari vaihtaa valinnan.
    klikkaa(&mut peli, 1);
    assert_eq!(peli.valittu, Some(1));
    assert_eq!(peli.lauta.jaljella(), 4);
    // Sama laatta uudelleen poistaa valinnan.
    klikkaa(&mut peli, 1);
    assert_eq!(peli.valittu, None);
    klikkaa(&mut peli, 0);
    klikkaa(&mut peli, 2);
    assert_eq!(peli.lauta.jaljella(), 2);
    assert_eq!(peli.lauta.pisteet, PARIN_PISTEET);
    assert!(peli.ponnahdus.is_some());
    assert!(peli.aanet.jono.contains(&(Aani::Yhdistys, 0)));
    assert_eq!(peli.kuvan_askel(), None);
    peli.edista(PONNAHDUS_MS);
    assert!(peli.ponnahdus.is_none());
    assert!(peli.kuvan_askel().is_some());
}

#[test]
fn peitetty_laatta_ei_kelpaa() {
    let mut peli = pelissa(&[1, 1]);
    peli.lauta = Lauta::laatoista(
        Vaikeus::Helppo,
        vec![Paikka { x: 0, y: 0, z: 0 }, Paikka { x: 0, y: 0, z: 1 }],
        vec![Some(Laatta(1)), Some(Laatta(1))],
    );
    peli.valitse(0);
    assert_eq!(peli.valittu, None);
    assert_eq!(peli.viesti, kaanna("mahjong.ei_vapaa"));
    // Päällimmäinen laatta löytyy pisteestä, vaikka alempi on samassa kohdassa.
    let (x, y) = piirto::laatan_keskipiste(&peli.lauta, 1);
    assert_eq!(piirto::laatta_pisteesta(&peli.lauta, x, y), Some(1));
}

#[test]
fn voitto_kysyy_nimen_ja_kirjaa_tuloksen() {
    let mut peli = pelissa(&[5, 5]);
    peli.valitse(0);
    peli.valitse(1);
    assert_eq!(peli.vaihe, Vaihe::Nimi);
    let pisteet = PARIN_PISTEET + TYHJENNYSBONUS + Vaikeus::Helppo.aikaraja_s();
    assert_eq!(peli.lauta.pisteet, pisteet);
    assert_eq!(peli.taulu.paras, pisteet);
    assert!(peli.uusi_tulos);
    // Juhla elää ensin joka ruudussa.
    assert_eq!(peli.kuvan_askel(), None);
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.vaihe, Vaihe::Nimi);
    peli.kasittele(Syote::Kirjain('Ä'));
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.vaihe, Vaihe::Voitto);
    let tulos = Tulos {
        nimi: "Ä".to_string(),
        pisteet,
    };
    assert_eq!(peli.taulu.parhaat, std::slice::from_ref(&tulos));
    assert_eq!(peli.taulu.lahetys, vec![tulos]);
    peli.edista(JUHLA_MS);
    assert!(peli.kuvan_askel().is_some());
    peli.kasittele(Syote::Kirjain('K'));
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
    assert_eq!(peli.peliaika_ms, 0);
}

#[test]
fn jumi_tarjoaa_sekoituksen_ja_tappio_kirjaa_pisteet() {
    let mut peli = pelissa(&[1, 1]);
    // Rivissä vain päät ovat vapaita.
    let paikat = (0..6)
        .map(|i| Paikka {
            x: 2 * i,
            y: 0,
            z: 0,
        })
        .collect();
    // Päät 1 ja 2 eivät ole pari, joten jumissa ollaan heti.
    let laatat = [1, 2, 1, 3, 3, 2].map(|l| Some(Laatta(l))).to_vec();
    peli.lauta = Lauta::laatoista(Vaikeus::Vaikea, paikat, laatat);
    assert_eq!(peli.lauta.tila, Tila::Jumissa);
    peli.kasittele(Syote::Kirjain('H'));
    assert_eq!(peli.vihje, None);
    peli.kasittele(Syote::Kirjain('U'));
    assert_eq!(peli.viesti, kaanna("mahjong.ei_perua_vaikealla"));
    peli.kasittele(Syote::Kirjain('S'));
    assert_eq!(peli.lauta.sekoituksia, 0);
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
    assert_eq!(peli.lauta.tila, Tila::Kesken);
    peli.kasittele(Syote::Kirjain('S'));
    assert_eq!(peli.viesti, kaanna("mahjong.ei_sekoituksia"));
}

#[test]
fn vihje_ja_peruutus_napeista() {
    let mut peli = pelissa(&[1, 2, 1, 2]);
    peli.lauta.pisteet = 100;
    let (x, y) = piirto::napin_keskipiste(Nappi::Vihje);
    peli.kasittele(Syote::Klikkaus(x, y));
    assert_eq!(peli.vihje, Some((0, 2)));
    assert_eq!(peli.lauta.pisteet, 100 - VIHJEEN_HINTA);
    // Esc poistaa vihjeen ensin, eikä poistu pelistä.
    peli.kasittele(Syote::Peru);
    assert_eq!(peli.vihje, None);
    assert!(!peli.lopeta);
    peli.valitse(0);
    peli.valitse(2);
    let (x, y) = piirto::napin_keskipiste(Nappi::Peruuta);
    peli.kasittele(Syote::Klikkaus(x, y));
    assert_eq!(peli.lauta.jaljella(), 4);
    assert_eq!(peli.lauta.pisteet, 100 - VIHJEEN_HINTA);
    let (x, y) = piirto::napin_keskipiste(Nappi::Valikko);
    peli.kasittele(Syote::Klikkaus(x, y));
    assert!(peli.lopeta);
}

#[test]
fn nappaimisto_liikkuu_vapailla_laatoilla_ja_valitsee() {
    let mut peli = pelissa(&[1, 2, 1, 2]);
    peli.kasittele(Syote::Nuoli(0, 1));
    assert!(peli.kursori_nakyy);
    let alku = peli.kursori;
    peli.kasittele(Syote::Nuoli(0, 1));
    peli.kasittele(Syote::Nuoli(0, -1));
    assert_eq!(peli.kursori, alku);
    peli.kursori = 0;
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.valittu, Some(0));
    peli.kasittele(Syote::Nuoli(0, 1));
    assert_eq!(peli.kursori, 1);
    peli.kasittele(Syote::Nuoli(0, 1));
    assert_eq!(peli.kursori, 2);
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.lauta.jaljella(), 2);
    peli.kasittele(Syote::Hiiri(0, 0));
    assert!(!peli.kursori_nakyy);
}

#[test]
fn kello_kay_vain_pelissa() {
    let mut peli = Mahjong::new(1);
    peli.edista(5000);
    assert_eq!(peli.peliaika_ms, 0);
    peli.kasittele(Syote::Ammu);
    peli.edista(3000);
    assert_eq!(peli.peliaika_ms, 3000);
    assert_eq!(peli.kuvan_askel().map(|a| a.0), Some(3));
    peli.vaihe = Vaihe::Tappio;
    peli.edista(3000);
    assert_eq!(peli.peliaika_ms, 3000);
}

#[test]
fn suljettu_ikkuna_kirjaa_nimea_odottavan_tuloksen() {
    let mut peli = pelissa(&[5, 5]);
    peli.valitse(0);
    peli.valitse(1);
    assert_eq!(peli.vaihe, Vaihe::Nimi);
    Pelinakyma::viimeistele(&mut peli);
    assert_eq!(peli.taulu.parhaat[0].nimi, kaanna("yhteinen.oletusnimi"));
}
