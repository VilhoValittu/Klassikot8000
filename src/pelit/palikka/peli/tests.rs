use super::*;
use crate::yhteinen::tulokset::Tulos;
use klassikot8000::palikka::{Palikka, Ruudut};

/// Lauta, jossa (2,2) ja (3,2) vaihtamalla syntyy kolmen suora.
fn yksi_siirto() -> Ruudut {
    let mut ruudut: Ruudut = [[None; SIVU]; SIVU];
    for (r, rivi) in ruudut.iter_mut().enumerate() {
        for (s, ruutu) in rivi.iter_mut().enumerate() {
            let vari = (2 + (r % 3) * 2 + s % 2 + (s / 2) % 2 * 3) % 6;
            *ruutu = Some(Palikka::Tavallinen(vari as u8));
        }
    }
    for (r, s) in [(2, 0), (2, 1), (3, 2)] {
        ruudut[r][s] = Some(Palikka::Tavallinen(0));
    }
    ruudut
}

fn pelissa() -> Palikkapeli {
    let mut peli = Palikkapeli::new(1);
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
    peli.aanet.jono.clear();
    peli.lauta = Palikkalauta::ruuduista(yksi_siirto());
    peli
}

fn klikkaa(peli: &mut Palikkapeli, ruutu: Paikka) {
    let (x, y) = piirto::ruudun_keskipiste(ruutu);
    peli.kasittele(Syote::Klikkaus(x, y));
    peli.kasittele(Syote::Vapautus(x, y));
}

/// Lauta, jossa ei ole siirtoja. Peli päättyy, kun toisto on ohi.
fn jumissa(peli: &mut Palikkapeli, pisteet: u32) {
    peli.lauta.pisteet = pisteet;
    peli.lauta.ohi = true;
}

#[test]
fn klikkaus_valitsee_ja_naapurin_klikkaus_vaihtaa() {
    let mut peli = pelissa();
    klikkaa(&mut peli, (2, 2));
    assert_eq!(peli.valittu, Some((2, 2)));
    klikkaa(&mut peli, (2, 2));
    assert_eq!(peli.valittu, None);

    klikkaa(&mut peli, (2, 2));
    klikkaa(&mut peli, (5, 5));
    assert_eq!(peli.valittu, Some((5, 5)));
    klikkaa(&mut peli, (2, 2));
    klikkaa(&mut peli, (3, 2));
    assert!(peli.lauta.pisteet >= 30);
    assert_eq!(peli.valittu, None);
    assert!(peli.animoi());
    assert_eq!(peli.taulu.paras, peli.lauta.pisteet);
    assert_eq!(peli.aanet.jono[0], (Aani::Liuku, 0));
    assert_eq!(peli.aanet.jono[1], (Aani::Helina1, VAIHTO_MS));
    assert!(peli.viesti.contains('+'));
}

#[test]
fn raahaus_naapuriin_vaihtaa_ja_vapautus_perii_raahauksen() {
    let mut peli = pelissa();
    let (x, y) = piirto::ruudun_keskipiste((2, 2));
    peli.kasittele(Syote::Klikkaus(x, y));
    let (x2, y2) = piirto::ruudun_keskipiste((3, 2));
    peli.kasittele(Syote::Hiiri(x2, y2));
    assert!(peli.lauta.pisteet >= 30);

    let mut peli = pelissa();
    peli.kasittele(Syote::Klikkaus(x, y));
    peli.kasittele(Syote::Vapautus(x, y));
    peli.kasittele(Syote::Hiiri(x2, y2));
    assert_eq!(peli.lauta.pisteet, 0);
    assert_eq!(peli.valittu, Some((2, 2)));
}

#[test]
fn hyodyton_vaihto_ahkaisee_ja_palaa() {
    let mut peli = pelissa();
    let lauta = peli.lauta.clone();
    klikkaa(&mut peli, (6, 6));
    klikkaa(&mut peli, (6, 7));
    assert_eq!(peli.lauta, lauta);
    assert_eq!(peli.aanet.jono, [(Aani::Virhe, 0)]);
    assert!(matches!(peli.animaatio, Some(Animaatio::Huti { .. })));
    peli.edista(HUTI_MS);
    assert!(!peli.animoi());
}

#[test]
fn syote_odottaa_toiston_loppua() {
    let mut peli = pelissa();
    klikkaa(&mut peli, (2, 2));
    klikkaa(&mut peli, (3, 2));
    assert!(peli.animoi());
    klikkaa(&mut peli, (5, 5));
    assert_eq!(peli.valittu, None);
    // Pudonneet palikat voivat sattua lukitsemaan laudan, mutta tässä peli jatkuu.
    peli.lauta.ohi = false;
    peli.edista(100_000);
    assert!(!peli.animoi());
    klikkaa(&mut peli, (5, 5));
    assert_eq!(peli.valittu, Some((5, 5)));
}

#[test]
fn nappaimisto_nayttaa_kursorin_valitsee_ja_vaihtaa() {
    let mut peli = pelissa();
    // Ensimmäinen painallus vain tuo kursorin näkyviin.
    peli.kasittele(Syote::Nuoli(0, 1));
    assert!(peli.kursori_nakyy);
    assert_eq!(peli.kursori, (4, 4));
    peli.kasittele(Syote::Nuoli(-1, 0));
    peli.kasittele(Syote::Nuoli(-1, 0));
    peli.kasittele(Syote::Nuoli(0, -1));
    peli.kasittele(Syote::Nuoli(0, -1));
    assert_eq!(peli.kursori, (2, 2));
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.valittu, Some((2, 2)));
    peli.kasittele(Syote::Nuoli(1, 0));
    assert_eq!(peli.kursori, (3, 2));
    assert!(peli.lauta.pisteet >= 30);
    assert_eq!(peli.valittu, None);

    // Laudan reuna pysäyttää kursorin.
    let mut peli = pelissa();
    peli.kursori_nakyy = true;
    peli.kursori = (0, 0);
    peli.kasittele(Syote::Nuoli(-1, 0));
    peli.kasittele(Syote::Nuoli(0, -1));
    assert_eq!(peli.kursori, (0, 0));
}

#[test]
fn valilyonti_ilman_kursoria_ei_valitse_vaan_nayttaa_sen() {
    let mut peli = pelissa();
    peli.kasittele(Syote::Ammu);
    assert!(peli.kursori_nakyy);
    assert_eq!(peli.valittu, None);
    peli.kasittele(Syote::Hiiri(0, 0));
    assert!(!peli.kursori_nakyy);
}

#[test]
fn vihje_tulee_odotuksen_jalkeen_ja_syote_nollaa_sen() {
    let mut peli = pelissa();
    assert_eq!(peli.vihje(), None);
    peli.edista(VIHJEEN_ODOTUS_MS);
    assert_eq!(peli.vihje(), peli.lauta.vihje());
    assert!(peli.vihje().is_some());
    peli.kasittele(Syote::Hiiri(1, 1));
    assert_eq!(peli.vihje(), None);
}

#[test]
fn toiston_kohdat_seuraavat_toisiaan() {
    assert!(matches!(kohta(0, 2), Kohta::Vaihdetaan(_)));
    assert!(matches!(kohta(VAIHTO_MS, 2), Kohta::Poksuu(0, _)));
    assert!(matches!(
        kohta(VAIHTO_MS + POKSAHDUS_MS, 2),
        Kohta::Putoaa(0, _)
    ));
    let toinen = VAIHTO_MS + POKSAHDUS_MS + PUTOAMINEN_MS;
    assert_eq!(kohta(toinen, 2), Kohta::Poksuu(1, 0.0));
    // Viimeisen kuvan jälkeen pysytään viimeisessä kierroksessa.
    assert_eq!(kohta(100_000, 2), Kohta::Putoaa(1, 1.0));
}

#[test]
fn kuva_paivittyy_toistossa_mutta_ei_levossa() {
    let mut peli = Palikkapeli::new(1);
    assert_eq!(peli.kuvan_askel(), None);
    peli.kasittele(Syote::Ammu);
    peli.lauta = Palikkalauta::ruuduista(yksi_siirto());
    let lepo = peli.kuvan_askel();
    peli.edista(1);
    assert_eq!(peli.kuvan_askel(), lepo);
    peli.vaihda((2, 2), (3, 2));
    let toisto = peli.kuvan_askel();
    peli.edista(1);
    assert_ne!(peli.kuvan_askel(), toisto);
}

#[test]
fn jumi_paattaa_pelin_vasta_toiston_jalkeen_ja_kysyy_nimen() {
    let mut peli = pelissa();
    klikkaa(&mut peli, (2, 2));
    klikkaa(&mut peli, (3, 2));
    jumissa(&mut peli, 40);
    peli.edista(1);
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
    peli.edista(100_000);
    assert_eq!(peli.vaihe, Vaihe::Nimi);
    assert!(peli.uusi_tulos);

    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.vaihe, Vaihe::Nimi);
    peli.kasittele(Syote::Kirjain('K'));
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.vaihe, Vaihe::Tappio);
    assert_eq!(peli.taulu.parhaat[0].nimi, "K");
    assert_eq!(peli.taulu.lahetys.last().unwrap().pisteet, 40);
}

#[test]
fn listalle_yltamaton_loppu_ja_uusi_peli() {
    let mut peli = pelissa();
    peli.taulu.netti = Some(vec![
        Tulos {
            nimi: "A".to_string(),
            pisteet: 1000,
        };
        3
    ]);
    jumissa(&mut peli, 40);
    peli.taulu.paras = 40;
    peli.edista(1);
    assert_eq!(peli.vaihe, Vaihe::Tappio);
    assert_eq!(peli.taulu.lahetys, Vec::new());

    peli.kasittele(Syote::Kirjain('K'));
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
    assert_eq!(peli.lauta.pisteet, 0);
    assert_eq!(peli.taulu.paras, 40);
}

#[test]
fn esc_kesken_pelin_pysayttaa_ja_toinen_kysyy_nimen() {
    let mut peli = pelissa();
    peli.lauta.pisteet = 500;
    peli.kasittele(Syote::Peru);
    assert_eq!(peli.vaihe, Vaihe::Tauko);
    // Tauolla lauta ei ota siirtoja, ja klikkaus jatkaa peliä.
    peli.kasittele(Syote::Klikkaus(0, 0));
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
    peli.kasittele(Syote::Peru);
    peli.kasittele(Syote::Nuoli(0, 1));
    assert_eq!(peli.vaihe, Vaihe::Tauko);
    assert!(!peli.kursori_nakyy);
    peli.kasittele(Syote::Peru);
    assert_eq!(peli.vaihe, Vaihe::Nimi);
    assert!(!peli.lopeta);
    peli.kasittele(Syote::Kirjain('B'));
    peli.kasittele(Syote::Ammu);
    assert!(peli.lopeta);
    assert_eq!(peli.taulu.parhaat[0].nimi, "B");

    // Ilman pisteitä toinen Esc poistuu heti.
    let mut peli = pelissa();
    peli.kasittele(Syote::Peru);
    assert!(!peli.lopeta);
    peli.kasittele(Syote::Peru);
    assert!(peli.lopeta);
    assert_eq!(peli.taulu.lahetys, Vec::new());
}

#[test]
fn lopun_napit_toimivat() {
    let mut peli = Palikkapeli::new(1);
    peli.vaihe = Vaihe::Tappio;
    let (x, y) = loppunapit::napin_keskipiste(Loppunappi::Uudestaan);
    peli.kasittele(Syote::Klikkaus(x, y));
    assert_eq!(peli.vaihe, Vaihe::Pelissa);

    peli.vaihe = Vaihe::Tappio;
    let (x, y) = loppunapit::napin_keskipiste(Loppunappi::Valikko);
    peli.kasittele(Syote::Klikkaus(x, y));
    assert!(peli.lopeta);
}
