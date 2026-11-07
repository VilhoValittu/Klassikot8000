use super::*;
use crate::pelit::pasianssi::peli::{Nappi, Vaihe};
use crate::yhteinen::kieli::kaanna;
use crate::yhteinen::syote::Syote;
use klassikot8000::kortit::{Kortti, Maa};
use klassikot8000::pasianssi::{Vaikeus, Variantti};

fn k(arvo: u8) -> Kortti {
    Kortti {
        maa: Maa::Pata,
        arvo,
    }
}

fn pyr(rivi: usize, sarake: usize) -> Paikka {
    Paikka::Pyramidi { rivi, sarake }
}

fn pelissa() -> Pasianssi {
    let mut peli = Pasianssi::new(1);
    peli.variantti = Variantti::Pyramidi;
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
    peli.pyramidi.tila.rivit = [[None; RIVEJA]; RIVEJA];
    peli.pyramidi.tila.pakka = vec![k(2)];
    peli
}

fn keskipiste(peli: &Pasianssi, paikka: Paikka) -> (i32, i32) {
    let (x, y) = piirto::kortin_paikka(&peli.pyramidi, paikka);
    (x + 40, y + 40)
}

fn klikkaa(peli: &mut Pasianssi, paikka: Paikka) {
    let (x, y) = keskipiste(peli, paikka);
    peli.kasittele(Syote::Klikkaus(x, y));
    peli.kasittele(Syote::Vapautus(x, y));
}

#[test]
fn kaksi_klikkausta_poistaa_parin_ja_vaara_pari_ahkaisee() {
    let mut peli = pelissa();
    peli.pyramidi.tila.rivit[6][0] = Some(k(6));
    peli.pyramidi.tila.rivit[6][1] = Some(k(7));
    peli.pyramidi.tila.rivit[6][2] = Some(k(9));
    klikkaa(&mut peli, pyr(6, 0));
    assert_eq!(peli.pyramidi_valittu, Some(pyr(6, 0)));
    klikkaa(&mut peli, pyr(6, 2));
    assert_eq!(peli.viesti, kaanna(EI_PARIA));
    // Väärän parin jälkimmäinen kortti jää valituksi.
    assert_eq!(peli.pyramidi_valittu, Some(pyr(6, 2)));
    klikkaa(&mut peli, pyr(6, 2));
    assert_eq!(peli.pyramidi_valittu, None);

    klikkaa(&mut peli, pyr(6, 0));
    klikkaa(&mut peli, pyr(6, 1));
    assert_eq!(peli.pyramidi.tila.rivit[6][0], None);
    assert_eq!(peli.pyramidi.tila.rivit[6][1], None);
    assert_eq!(peli.pyramidi_valittu, None);
}

#[test]
fn veto_toiselle_kortille_poistaa_parin() {
    let mut peli = pelissa();
    peli.pyramidi.tila.rivit[6][0] = Some(k(6));
    peli.pyramidi.tila.rivit[6][1] = Some(k(7));
    peli.pyramidi.tila.rivit[6][2] = Some(k(9));
    let (x, y) = keskipiste(&peli, pyr(6, 0));
    peli.kasittele(Syote::Klikkaus(x, y));
    let (kx, ky) = keskipiste(&peli, pyr(6, 1));
    peli.kasittele(Syote::Hiiri(kx, ky));
    peli.kasittele(Syote::Vapautus(kx, ky));
    assert_eq!(peli.pyramidi.tila.rivit[6][1], None);
}

#[test]
fn kuningas_lahtee_klikkauksella_ja_peitetty_kortti_ei_reagoi() {
    let mut peli = pelissa();
    peli.pyramidi.tila.rivit[5][0] = Some(k(4));
    peli.pyramidi.tila.rivit[6][0] = Some(k(KUNINGAS));
    peli.pyramidi.tila.rivit[6][1] = Some(k(3));
    peli.pyramidi.tila.pakka = vec![k(10)];
    // Keskellä ylempää korttia, siltä kohdin, jota alemmat eivät peitä.
    let (x, y) = piirto::kortin_paikka(&peli.pyramidi, pyr(5, 0));
    peli.kasittele(Syote::Klikkaus(x + 5, y + 5));
    assert!(peli.pyramidi_pito.is_none());
    klikkaa(&mut peli, pyr(6, 0));
    assert_eq!(peli.pyramidi.tila.rivit[6][0], None);
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
}

#[test]
fn pakka_nostaa_ja_jatteen_kortti_kay_pariksi() {
    let mut peli = pelissa();
    peli.pyramidi.tila.rivit[6][0] = Some(k(11));
    let (x, y) = piirto::pakan_keskipiste();
    peli.kasittele(Syote::Klikkaus(x, y));
    assert_eq!(peli.pyramidi.tila.jate.last(), Some(&k(2)));
    klikkaa(&mut peli, Paikka::Jate(0));
    klikkaa(&mut peli, pyr(6, 0));
    assert!(peli.pyramidi.tila.jate.iter().all(|&kortti| kortti != k(2)));
}

#[test]
fn nappaimilla_poistetaan_pari() {
    let mut peli = pelissa();
    peli.pyramidi.tila.rivit[1][0] = Some(k(6));
    peli.pyramidi.tila.rivit[1][1] = Some(k(7));
    peli.pyramidi.tila.rivit[2][0] = None;
    peli.kasittele(Syote::Nuoli(1, 0));
    peli.kasittele(Syote::Nuoli(1, 0));
    assert_eq!(peli.pyramidi_kursorin_kohta(), Kohta::Paikka(pyr(1, 0)));
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.pyramidi_valittu, Some(pyr(1, 0)));
    peli.kasittele(Syote::Nuoli(0, 1));
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.pyramidi.tila.rivit[1], [None; RIVEJA]);
    assert_eq!(peli.vaihe, Vaihe::Nimi);
}

#[test]
fn ylarivilla_osoitin_nostaa_ja_osoittaa_jatetta() {
    let mut peli = pelissa();
    peli.pyramidi.tila.rivit[6][0] = Some(k(11));
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.pyramidi.tila.jate.last(), Some(&k(2)));
    peli.kasittele(Syote::Nuoli(0, 1));
    assert_eq!(
        peli.pyramidi_kursorin_kohta(),
        Kohta::Paikka(Paikka::Jate(1))
    );
    peli.kasittele(Syote::Nuoli(0, 1));
    assert_eq!(
        peli.pyramidi_kursorin_kohta(),
        Kohta::Paikka(Paikka::Jate(0))
    );
    peli.kasittele(Syote::Nuoli(0, 1));
    assert_eq!(
        peli.pyramidi_kursorin_kohta(),
        Kohta::Paikka(Paikka::Jate(0))
    );
}

#[test]
fn viimeinen_pari_voittaa_ja_tulos_menee_omalle_listalle() {
    let mut peli = pelissa();
    peli.pyramidi.tila.rivit[6][0] = Some(k(1));
    peli.pyramidi.tila.rivit[6][1] = Some(k(12));
    klikkaa(&mut peli, pyr(6, 0));
    klikkaa(&mut peli, pyr(6, 1));
    assert_eq!(peli.vaihe, Vaihe::Nimi);
    peli.kasittele(Syote::Kirjain('P'));
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.vaihe, Vaihe::Voitto);
    assert_eq!(
        peli.taulut[Variantti::Pyramidi as usize].parhaat[0].nimi,
        "P"
    );
    assert!(peli.taulut[Variantti::Klondike as usize].parhaat.is_empty());
}

#[test]
fn siirrot_loppuvat_ja_vaikealla_ei_voi_perua() {
    let mut peli = Pasianssi::new(1);
    peli.variantti = Variantti::Pyramidi;
    peli.vaikeus = Vaikeus::Vaikea;
    peli.kasittele(Syote::Ammu);
    peli.pyramidi.tila.rivit = [[None; RIVEJA]; RIVEJA];
    peli.pyramidi.tila.rivit[6][0] = Some(k(1));
    peli.pyramidi.tila.jate.clear();
    peli.pyramidi.tila.kierratykset = 1;
    peli.pyramidi.tila.pakka = vec![k(5)];
    peli.kasittele(Syote::Kirjain('U'));
    assert!(!peli.viesti.is_empty());
    peli.kasittele(Syote::Kirjain('N'));
    assert_eq!(peli.vaihe, Vaihe::Tappio);
    let (x, y) = piirto::napin_keskipiste(Nappi::Uusi);
    peli.kasittele(Syote::Klikkaus(x, y));
    assert_eq!(peli.vaihe, Vaihe::Alku);
}
