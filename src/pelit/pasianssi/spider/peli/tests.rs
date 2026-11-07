use super::*;
use crate::pelit::pasianssi::peli::{Nappi, Vaihe};
use crate::yhteinen::kieli::kaanna;
use crate::yhteinen::syote::Syote;
use crate::yhteinen::tulokset::Tulos;
use klassikot8000::kortit::{Kortti, Maa};
use klassikot8000::pasianssi::{Sarake, Variantti};
use klassikot8000::spider::SARJOJA;

fn k(maa: Maa, arvo: u8) -> Kortti {
    Kortti { maa, arvo }
}

fn pelissa() -> Pasianssi {
    let mut peli = Pasianssi::new(1);
    peli.variantti = Variantti::Spider;
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
    peli.spider.tila.pakka.clear();
    peli.spider.tila.sarakkeet = Default::default();
    peli
}

fn aseta(peli: &mut Pasianssi, sarake: usize, kortit: &[Kortti]) {
    peli.spider.tila.sarakkeet[sarake] = Sarake {
        kortit: kortit.to_vec(),
        piilossa: 0,
    };
}

fn kortin_keskipiste(peli: &Pasianssi, sarake: usize, kortti: usize) -> (i32, i32) {
    let (x, y) = piirto::kortin_paikka(&peli.spider, Jono { sarake, kortti });
    (x + 20, y + 10)
}

#[test]
fn veto_ja_klikkaus_siirtavat_jonon() {
    let mut peli = pelissa();
    aseta(&mut peli, 0, &[k(Maa::Pata, 6), k(Maa::Pata, 5)]);
    aseta(&mut peli, 1, &[k(Maa::Hertta, 7)]);
    let (x, y) = kortin_keskipiste(&peli, 0, 0);
    peli.kasittele(Syote::Klikkaus(x, y));
    assert!(peli.spider_pito.is_some());
    let (kx, ky) = kortin_keskipiste(&peli, 1, 0);
    peli.kasittele(Syote::Hiiri(kx, ky));
    peli.kasittele(Syote::Vapautus(kx, ky));
    assert_eq!(peli.spider.tila.sarakkeet[1].kortit.len(), 3);
    assert!(peli.spider_pito.is_none());

    let mut peli = pelissa();
    aseta(&mut peli, 0, &[k(Maa::Pata, 6)]);
    aseta(&mut peli, 1, &[k(Maa::Hertta, 7)]);
    let (x, y) = kortin_keskipiste(&peli, 0, 0);
    peli.kasittele(Syote::Klikkaus(x, y));
    peli.kasittele(Syote::Vapautus(x, y));
    assert!(peli.spider_valittu.is_some());
    let (kx, ky) = kortin_keskipiste(&peli, 1, 0);
    peli.kasittele(Syote::Klikkaus(kx, ky));
    assert_eq!(peli.spider.tila.sarakkeet[1].kortit.len(), 2);
}

#[test]
fn laiton_pudotus_ahkaisee_ja_rikkinainen_jono_ei_lahde() {
    let mut peli = pelissa();
    aseta(&mut peli, 0, &[k(Maa::Pata, 6)]);
    aseta(&mut peli, 1, &[k(Maa::Hertta, 9)]);
    let (x, y) = kortin_keskipiste(&peli, 0, 0);
    peli.kasittele(Syote::Klikkaus(x, y));
    let (kx, ky) = kortin_keskipiste(&peli, 1, 0);
    peli.kasittele(Syote::Hiiri(kx, ky));
    peli.kasittele(Syote::Vapautus(kx, ky));
    assert_eq!(peli.spider.tila.sarakkeet[0].kortit.len(), 1);
    assert_eq!(peli.viesti, kaanna(EI_KAY));

    let mut peli = pelissa();
    aseta(&mut peli, 0, &[k(Maa::Hertta, 6), k(Maa::Pata, 5)]);
    let (x, y) = kortin_keskipiste(&peli, 0, 0);
    peli.kasittele(Syote::Klikkaus(x, y));
    assert!(peli.spider_pito.is_none());
}

#[test]
fn jako_toimii_pakasta_napista_ja_n_nappaimesta() {
    let mut peli = Pasianssi::new(1);
    peli.variantti = Variantti::Spider;
    peli.kasittele(Syote::Ammu);
    let (x, y) = piirto::pakan_keskipiste();
    peli.kasittele(Syote::Klikkaus(x, y));
    assert_eq!(peli.spider.jakoja(), 4);
    peli.kasittele(Syote::Kirjain('N'));
    assert_eq!(peli.spider.jakoja(), 3);
    let (x, y) = piirto::napin_keskipiste(Nappi::Nosta);
    peli.kasittele(Syote::Klikkaus(x, y));
    assert_eq!(peli.spider.jakoja(), 2);

    peli.spider.tila.sarakkeet[3].kortit.clear();
    peli.spider.tila.sarakkeet[3].piilossa = 0;
    peli.kasittele(Syote::Kirjain('N'));
    assert_eq!(peli.spider.jakoja(), 2);
    assert_eq!(peli.viesti, kaanna(EI_JAKOA));
}

#[test]
fn nappaimilla_poimitaan_ja_siirretaan_jono() {
    let mut peli = pelissa();
    aseta(&mut peli, 0, &[k(Maa::Pata, 6), k(Maa::Pata, 5)]);
    aseta(&mut peli, 1, &[k(Maa::Hertta, 7)]);
    peli.kasittele(Syote::Nuoli(-1, 0));
    peli.kasittele(Syote::Ammu);
    assert_eq!(
        peli.spider_valittu,
        Some(Jono {
            sarake: 0,
            kortti: 0
        })
    );
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.spider_valittu, None);
    peli.kasittele(Syote::Ammu);
    peli.kasittele(Syote::Nuoli(0, 1));
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.spider.tila.sarakkeet[1].kortit.len(), 3);
    assert_eq!(peli.kursori.syvyys, 0);
}

#[test]
fn peruutus_toimii_vaikeallakin_ja_viimeinen_sarja_voittaa() {
    let mut peli = Pasianssi::new(1);
    peli.variantti = Variantti::Spider;
    peli.vaikeus = klassikot8000::pasianssi::Vaikeus::Vaikea;
    peli.kasittele(Syote::Ammu);
    peli.kasittele(Syote::Kirjain('N'));
    peli.kasittele(Syote::Kirjain('U'));
    assert_eq!(peli.spider.jakoja(), 5);

    let mut peli = pelissa();
    peli.spider.tila.valmiit = vec![Maa::Pata; SARJOJA - 1];
    let sarja: Vec<Kortti> = (2..=13).rev().map(|arvo| k(Maa::Pata, arvo)).collect();
    aseta(&mut peli, 0, &sarja);
    aseta(&mut peli, 1, &[k(Maa::Pata, 1)]);
    peli.edista(2000);
    let (x, y) = kortin_keskipiste(&peli, 1, 0);
    peli.kasittele(Syote::Klikkaus(x, y));
    peli.kasittele(Syote::Vapautus(x, y));
    let (kx, ky) = kortin_keskipiste(&peli, 0, 11);
    peli.kasittele(Syote::Klikkaus(kx, ky));
    assert_eq!(peli.vaihe, Vaihe::Nimi);

    // Tulos menee Spiderin omalle listalle.
    peli.kasittele(Syote::Kirjain('S'));
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.vaihe, Vaihe::Voitto);
    let oma = &peli.taulut[Variantti::Spider as usize].parhaat;
    assert_eq!(
        oma.iter().map(|t: &Tulos| &t.nimi[..]).collect::<Vec<_>>(),
        ["S"]
    );
    assert!(peli.taulut[Variantti::Klondike as usize].parhaat.is_empty());
}

#[test]
fn peli_loppuu_kun_siirrot_loppuvat_ja_kello_pysahtyy() {
    let mut peli = pelissa();
    // Vain viitosen siirto kuutosen päälle on mahdollinen, ja sen jälkeen mikään ei käy.
    for i in 2..SARAKKEITA {
        aseta(&mut peli, i, &[k(Maa::Pata, 1), k(Maa::Hertta, 13)]);
    }
    aseta(
        &mut peli,
        0,
        &[k(Maa::Pata, 1), k(Maa::Hertta, 9), k(Maa::Pata, 5)],
    );
    aseta(&mut peli, 1, &[k(Maa::Pata, 1), k(Maa::Risti, 6)]);
    peli.edista(1000);
    let (x, y) = kortin_keskipiste(&peli, 0, 2);
    peli.kasittele(Syote::Klikkaus(x, y));
    let (kx, ky) = kortin_keskipiste(&peli, 1, 1);
    peli.kasittele(Syote::Hiiri(kx, ky));
    peli.kasittele(Syote::Vapautus(kx, ky));
    assert_eq!(peli.spider.tila.sarakkeet[1].kortit.len(), 3);
    assert_eq!(peli.vaihe, Vaihe::Tappio);
    assert_eq!(peli.pisteet(), 0);
    peli.edista(1000);
    assert_eq!(peli.peliaika_ms, 1000);
    peli.kasittele(Syote::Kirjain('K'));
    assert_eq!(peli.vaihe, Vaihe::Alku);
}
