use super::*;
use crate::yhteinen::tulokset;
use klassikot8000::blackjack::{Kortti, Maa};

/// Peli panostusvaiheessa. Kortit nousevat annetussa järjestyksessä.
fn pelissa(arvot: &[u8]) -> Blackjack {
    let mut peli = Blackjack::new(1);
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
    let mut kenka = vec![
        Kortti {
            maa: Maa::Pata,
            arvo: 2,
        };
        100
    ];
    kenka.extend(arvot.iter().rev().map(|&arvo| Kortti {
        maa: Maa::Hertta,
        arvo,
    }));
    peli.poyta.kenka = kenka;
    peli
}

#[test]
fn panos_muuttuu_nuolilla_numeroilla_ja_napeilla_mutta_pysyy_rajoissa() {
    let mut peli = pelissa(&[]);
    assert_eq!(peli.panos, ALKUPANOS);
    peli.kasittele(Syote::Nuoli(0, 1));
    assert_eq!(peli.panos, 11 * YKSI);
    peli.kasittele(Syote::Nuoli(-1, 0));
    assert_eq!(peli.panos, 21 * YKSI);
    peli.kasittele(Syote::Numero(4));
    assert_eq!(peli.panos, 121 * YKSI);
    peli.kasittele(Syote::Numero(9));
    assert_eq!(peli.panos, 121 * YKSI);
    peli.kasittele(Syote::Kirjain('T'));
    assert_eq!(peli.panos, PIENIN_PANOS);
    peli.kasittele(Syote::Nuoli(1, 0));
    assert_eq!(peli.panos, PIENIN_PANOS);
    peli.kasittele(Syote::Kirjain('K'));
    assert_eq!(peli.panos, ALKURAHA);
    peli.kasittele(Syote::Numero(4));
    assert_eq!(peli.panos, ALKURAHA);

    let (x, y) = piirto::napin_keskipiste(Vaihe::Pelissa, Vuoro::Panostus, Nappi::Pienin);
    peli.kasittele(Syote::Klikkaus(x, y));
    assert_eq!(peli.panos, PIENIN_PANOS);
    let (x, y) = piirto::napin_keskipiste(Vaihe::Pelissa, Vuoro::Panostus, Nappi::Lisaa(2));
    peli.kasittele(Syote::Klikkaus(x, y));
    assert_eq!(peli.panos, 26 * YKSI);
}

#[test]
fn jakaja_pelaa_ajan_kuluessa_ja_tulos_nakyy_vasta_lopussa() {
    let mut peli = pelissa(&[10, 5, 10, 6, 4, 2]);
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.poyta.vuoro, Vuoro::Pelaaja);
    assert_eq!(peli.poyta.rahat, ALKURAHA - ALKUPANOS);
    let lepo = peli.kuvan_askel();
    peli.edista(10_000);
    assert_eq!(peli.kuvan_askel(), lepo);

    peli.kasittele(Syote::Kirjain('J'));
    assert_eq!(peli.poyta.vuoro, Vuoro::Jakaja);
    peli.edista(JAKAJAN_SIIRTO_MS - 1);
    assert!(peli.poyta.piilossa);
    // syöte ei hoputa jakajaa eikä muuta panosta kesken kierroksen
    peli.kasittele(Syote::Ammu);
    peli.kasittele(Syote::Nuoli(0, 1));
    assert_eq!(peli.panos, ALKUPANOS);
    assert_eq!(peli.poyta.vuoro, Vuoro::Jakaja);

    let ennen = peli.kuvan_askel();
    peli.edista(1);
    assert!(!peli.poyta.piilossa);
    assert_ne!(peli.kuvan_askel(), ennen);
    assert_eq!(peli.poyta.rahat, ALKURAHA - ALKUPANOS);
    assert!(peli.viesti.is_empty());

    peli.edista(2 * JAKAJAN_SIIRTO_MS);
    assert_eq!(peli.poyta.vuoro, Vuoro::Panostus);
    assert_eq!(peli.poyta.rahat, ALKURAHA + ALKUPANOS);
    assert_eq!(
        peli.viesti,
        muotoile("blackjack.viesti_voitto", &[&raha(ALKUPANOS)])
    );
    assert!(peli.aanet.jono.contains(&(Aani::Yhdistys, LOPPU_VIIVE_MS)));
}

#[test]
fn blackjack_paattyy_heti_ja_soi_voitolle() {
    let mut peli = pelissa(&[1, 9, 13, 8]);
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.poyta.vuoro, Vuoro::Panostus);
    assert_eq!(peli.poyta.rahat, ALKURAHA + 15 * YKSI);
    assert!(peli.aanet.jono.contains(&(Aani::Voitto, LOPPU_VIIVE_MS)));
}

#[test]
fn pienimman_panoksen_blackjack_maksaa_puolikkaan_ja_nakyy_viestissa() {
    let mut peli = pelissa(&[1, 9, 13, 8]);
    peli.kasittele(Syote::Kirjain('T'));
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.poyta.rahat, ALKURAHA + 3);
    assert_eq!(
        peli.viesti,
        muotoile("blackjack.viesti_blackjack", &[&"1,5"])
    );
}

#[test]
fn kaikki_rahat_havittyaan_peli_loppuu_ja_alkaa_alusta() {
    let mut peli = pelissa(&[10, 10, 7, 10]);
    peli.kasittele(Syote::Kirjain('K'));
    peli.kasittele(Syote::Ammu);
    peli.kasittele(Syote::Kirjain('J'));
    peli.edista(10_000);
    assert_eq!(peli.vaihe, Vaihe::Loppu);
    assert_eq!(peli.poyta.rahat, 0);
    assert!(peli.aanet.jono.contains(&(Aani::Tappio, LOPPU_VIIVE_MS)));

    // lopussa panos tai jako eivät enää toimi
    peli.kasittele(Syote::Ammu);
    peli.kasittele(Syote::Nuoli(0, 1));
    assert_eq!(peli.vaihe, Vaihe::Loppu);
    peli.kasittele(Syote::Kirjain('K'));
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
    assert_eq!(peli.poyta.rahat, ALKURAHA);
    assert_eq!(peli.panos, ALKUPANOS);
    assert!(peli.poyta.kadet.is_empty());
}

#[test]
fn pelkka_puolikas_ei_riita_panokseen() {
    let mut peli = pelissa(&[10, 10, 7, 10]);
    peli.poyta.rahat = PIENIN_PANOS + 1;
    peli.panos = PIENIN_PANOS;
    peli.kasittele(Syote::Ammu);
    peli.kasittele(Syote::Kirjain('J'));
    peli.edista(10_000);
    assert_eq!(peli.poyta.rahat, 1);
    assert_eq!(peli.vaihe, Vaihe::Loppu);
}

#[test]
fn panos_pienenee_jaljella_oleviin_kokonaisiin_rahoihin() {
    let mut peli = pelissa(&[10, 10, 7, 10]);
    peli.panos = 295 * YKSI;
    peli.poyta.rahat = ALKURAHA + 1;
    peli.kasittele(Syote::Ammu);
    peli.kasittele(Syote::Kirjain('J'));
    peli.edista(10_000);
    assert_eq!(peli.poyta.rahat, 5 * YKSI + 1);
    assert_eq!(peli.panos, 5 * YKSI);
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
}

#[test]
fn vakuutus_kysytaan_jakajan_assasta_ja_kierros_jatkuu_ilman_blackjackia() {
    let mut peli = pelissa(&[10, 1, 9, 6]);
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.poyta.vuoro, Vuoro::Vakuutus);
    assert_eq!(
        peli.viesti,
        muotoile("blackjack.vakuutus_tarjous", &[&raha(ALKUPANOS / 2)])
    );
    // muut näppäimet eivät tee mitään ennen päätöstä
    peli.kasittele(Syote::Kirjain('O'));
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.poyta.kadet[0].kortit.len(), 2);
    assert_eq!(peli.poyta.vuoro, Vuoro::Vakuutus);

    peli.kasittele(Syote::Kirjain('V'));
    assert_eq!(peli.poyta.vuoro, Vuoro::Pelaaja);
    assert!(peli.poyta.piilossa);
    assert_eq!(peli.poyta.rahat, ALKURAHA - ALKUPANOS - ALKUPANOS / 2);
    assert_eq!(peli.viesti, kaanna("blackjack.vakuutus_havisi"));
}

#[test]
fn vakuutus_maksaa_jakajan_blackjackista_ja_kieltaytyminen_haviaa() {
    let mut peli = pelissa(&[10, 1, 9, 13]);
    peli.kasittele(Syote::Ammu);
    let (x, y) = piirto::napin_keskipiste(Vaihe::Pelissa, Vuoro::Vakuutus, Nappi::Vakuuta);
    peli.kasittele(Syote::Klikkaus(x, y));
    assert_eq!(peli.poyta.vuoro, Vuoro::Panostus);
    assert_eq!(peli.poyta.rahat, ALKURAHA);
    assert_eq!(
        peli.viesti,
        muotoile("blackjack.viesti_vakuutus", &[&raha(ALKUPANOS), &raha(0)])
    );

    let mut peli = pelissa(&[10, 1, 9, 13]);
    peli.kasittele(Syote::Ammu);
    peli.kasittele(Syote::Kirjain('N'));
    assert_eq!(peli.poyta.rahat, ALKURAHA - ALKUPANOS);
    assert_eq!(
        peli.viesti,
        muotoile("blackjack.viesti_tappio", &[&raha(ALKUPANOS)])
    );
}

#[test]
fn blackjack_kadella_vakuutus_on_tasaraha() {
    let mut peli = pelissa(&[1, 1, 13, 6]);
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.poyta.vuoro, Vuoro::Vakuutus);
    assert_eq!(
        peli.viesti,
        muotoile("blackjack.tasaraha_tarjous", &[&raha(ALKUPANOS)])
    );
    peli.kasittele(Syote::Kirjain('V'));
    assert_eq!(peli.poyta.rahat, ALKURAHA + ALKUPANOS);
}

#[test]
fn mahdoton_tuplaus_ja_halkaisu_antavat_viestin_eivatka_muuta_tilaa() {
    let mut peli = pelissa(&[5, 10, 6, 9]);
    peli.kasittele(Syote::Kirjain('K'));
    peli.kasittele(Syote::Ammu);
    peli.kasittele(Syote::Kirjain('T'));
    assert_eq!(peli.viesti, kaanna("blackjack.ei_tuplausta"));
    peli.kasittele(Syote::Kirjain('H'));
    assert_eq!(peli.viesti, kaanna("blackjack.ei_halkaisua"));
    assert_eq!(peli.poyta.kadet[0].kortit.len(), 2);
    assert_eq!(peli.poyta.vuoro, Vuoro::Pelaaja);
}

#[test]
fn voitolla_poistuva_kirjoittaa_nimen_ja_tulos_on_voitto() {
    let mut peli = pelissa(&[]);
    peli.poyta.rahat = ALKURAHA + 120 * YKSI;
    peli.kasittele(Syote::Kirjain('E'));
    assert_eq!(peli.vaihe, Vaihe::Nimi);
    assert!(!peli.lopeta);
    // Nappikirjain menee nimeen, eikä klikkaus paina piilossa olevia nappeja.
    peli.kasittele(Syote::Kirjain('E'));
    let (x, y) = piirto::napin_keskipiste(Vaihe::Nimi, Vuoro::Panostus, Nappi::Valikko);
    peli.kasittele(Syote::Klikkaus(x, y));
    assert!(!peli.lopeta);
    peli.kasittele(Syote::Ammu);
    assert!(peli.lopeta);
    assert_eq!(peli.taulu.parhaat[0].nimi, "E");
    assert_eq!(peli.taulu.parhaat[0].pisteet, 120);
    assert!(peli.uusi_tulos);
    assert_eq!(peli.taulu.lahetys.last().unwrap().pisteet, 120);
}

#[test]
fn ilman_voittoa_tai_listapaikkaa_poistutaan_heti() {
    let mut peli = pelissa(&[]);
    peli.kasittele(Syote::Peru);
    assert!(peli.lopeta);
    assert_eq!(peli.taulu.lahetys, Vec::new());

    let mut peli = pelissa(&[]);
    peli.poyta.rahat = ALKURAHA + 5 * YKSI;
    peli.taulu.netti = Some(vec![
        tulokset::Tulos {
            nimi: "A".to_string(),
            pisteet: 100,
        };
        3
    ]);
    peli.kasittele(Syote::Peru);
    assert!(peli.lopeta);
    assert_eq!(peli.taulu.lahetys, Vec::new());
}

#[test]
fn kesken_kierroksen_poistuva_jattaa_panoksen_eika_jakaja_jatka() {
    let mut peli = pelissa(&[10, 5, 10, 6, 4, 2]);
    peli.poyta.rahat = ALKURAHA + 50 * YKSI;
    peli.kasittele(Syote::Ammu);
    peli.kasittele(Syote::Kirjain('J'));
    assert_eq!(peli.poyta.vuoro, Vuoro::Jakaja);
    peli.kasittele(Syote::Peru);
    assert_eq!(peli.vaihe, Vaihe::Nimi);
    let kuva = peli.kuvan_askel();
    peli.edista(10_000);
    assert_eq!(peli.poyta.vuoro, Vuoro::Jakaja);
    assert_ne!(peli.kuvan_askel(), kuva);
    peli.kasittele(Syote::Kirjain('A'));
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.taulu.parhaat[0].pisteet, 40);
}

#[test]
fn napit_toimivat_pelatessa_ja_esc_palaa_valikkoon() {
    let mut peli = pelissa(&[8, 10, 8, 9, 3, 2]);
    peli.kasittele(Syote::Ammu);
    let (x, y) = piirto::napin_keskipiste(Vaihe::Pelissa, Vuoro::Pelaaja, Nappi::Halkaise);
    peli.kasittele(Syote::Klikkaus(x, y));
    assert_eq!(peli.poyta.kadet.len(), 2);
    let (x, y) = piirto::napin_keskipiste(Vaihe::Pelissa, Vuoro::Pelaaja, Nappi::Ota);
    peli.kasittele(Syote::Klikkaus(x, y));
    assert_eq!(peli.poyta.kadet[0].kortit.len(), 3);
    peli.kasittele(Syote::Peru);
    assert!(peli.lopeta);

    let mut peli = pelissa(&[]);
    peli.kasittele(Syote::Kirjain('E'));
    assert!(peli.lopeta);
}
