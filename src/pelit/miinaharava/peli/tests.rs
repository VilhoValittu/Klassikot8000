use super::*;
use crate::yhteinen::tulokset::Tulos;
use klassikot8000::miinaharava::Tila;

fn pelissa(vaikeus: Vaikeus, miinat: &[Paikka]) -> Miinaharava {
    let mut peli = Miinaharava::new(1);
    peli.vaikeus = vaikeus;
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
    peli.aanet.jono.clear();
    peli.lauta = Miinakentta::miinoista(vaikeus, miinat);
    peli
}

#[test]
fn listalle_yltava_voitto_kysyy_nimen_ja_lahtee_nettiin() {
    let mut peli = pelissa(Vaikeus::Normaali, &[(0, 0)]);
    peli.avaa((1, 1));
    peli.edista(40_000);
    peli.avaa((15, 15));
    assert_eq!(peli.vaihe, Vaihe::Nimi);
    let pisteet = aikapisteet(Vaikeus::Normaali, 40);
    assert_eq!(peli.pisteet(), pisteet);
    // tyhjää nimeä ei kuitata välilyönnillä
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.vaihe, Vaihe::Nimi);
    peli.kasittele(Syote::Kirjain('V'));
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.vaihe, Vaihe::Voitto);
    let tulos = Tulos {
        nimi: "V".to_string(),
        pisteet,
    };
    assert_eq!(peli.taulu.parhaat, std::slice::from_ref(&tulos));
    assert_eq!(peli.taulu.lahetys, Some(tulos));
    assert!(peli.uusi_tulos);

    // täydelle listalle heikompi tulos ei pääse, eikä nimeä kysytä
    let mut peli = pelissa(Vaikeus::Helppo, &[(0, 0)]);
    peli.taulu.parhaat = vec![
        Tulos {
            nimi: "A".to_string(),
            pisteet: 5000,
        };
        3
    ];
    peli.avaa((1, 1));
    peli.avaa((8, 8));
    assert_eq!(peli.vaihe, Vaihe::Voitto);
    assert!(peli.taulu.lahetys.is_none());
}

#[test]
fn suljettu_ikkuna_kirjaa_nimea_odottavan_voiton() {
    let mut peli = pelissa(Vaikeus::Helppo, &[(0, 0)]);
    peli.avaa((1, 1));
    peli.avaa((8, 8));
    assert_eq!(peli.vaihe, Vaihe::Nimi);
    Pelinakyma::viimeistele(&mut peli);
    assert_eq!(peli.taulu.parhaat[0].nimi, kaanna("yhteinen.oletusnimi"));
}

fn klikkaa(peli: &mut Miinaharava, ruutu: Paikka) {
    let (x, y) = piirto::ruudun_keskipiste(peli.vaikeus, ruutu);
    peli.kasittele(Syote::Klikkaus(x, y));
}

#[test]
fn taso_valitaan_alussa_ja_peli_alkaa_silla() {
    let mut peli = Miinaharava::new(1);
    peli.kasittele(Syote::Numero(3));
    assert_eq!(peli.vaikeus, Vaikeus::Vaikea);
    peli.kasittele(Syote::Nuoli(0, -1));
    assert_eq!(peli.vaikeus, Vaikeus::Normaali);
    peli.kasittele(Syote::Numero(9));
    assert_eq!(peli.vaikeus, Vaikeus::Normaali);
    let (x, y) = piirto::vaikeuden_keskipiste(Vaikeus::Helppo);
    peli.kasittele(Syote::Klikkaus(x, y));
    assert_eq!(peli.vaikeus, Vaikeus::Helppo);
    assert_eq!(peli.vaihe, Vaihe::Alku);
    peli.kasittele(Syote::Klikkaus(0, 300));
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
    assert_eq!(peli.lauta.vaikeus, Vaikeus::Helppo);
    assert_eq!(peli.aanet.jono, [(Aani::Alku, 0)]);
}

#[test]
fn vasen_avaa_ja_oikea_liputtaa() {
    let mut peli = pelissa(Vaikeus::Helppo, &[(0, 0), (8, 8)]);
    klikkaa(&mut peli, (1, 1));
    assert!(peli.lauta.ruutu((1, 1)).auki);
    assert_eq!(peli.aanet.jono, [(Aani::Pomppu, 0)]);

    let (x, y) = piirto::ruudun_keskipiste(Vaikeus::Helppo, (0, 0));
    peli.kasittele(Syote::OikeaKlikkaus(x, y));
    assert!(peli.lauta.ruutu((0, 0)).lippu);
    // liputettu ei aukea, joten miina ei räjähdä
    peli.kasittele(Syote::Klikkaus(x, y));
    assert_eq!(peli.vaihe, Vaihe::Pelissa);

    // numeron klikkaus avaa ympäriltä, kun liput täsmäävät
    klikkaa(&mut peli, (1, 1));
    assert!(peli.lauta.ruutu((2, 2)).auki);
}

#[test]
fn nappaimisto_liikuttaa_avaa_ja_liputtaa() {
    let mut peli = pelissa(Vaikeus::Helppo, &[(0, 0), (8, 8)]);
    peli.kasittele(Syote::Ammu);
    assert!(peli.kursori_nakyy);
    assert!(!peli.lauta.ruutu(peli.kursori).auki);
    peli.kursori = (0, 1);
    peli.kasittele(Syote::Ammu);
    assert!(peli.lauta.ruutu((0, 1)).auki);
    peli.kasittele(Syote::Nuoli(0, -1));
    assert_eq!(peli.kursori, (0, 0));
    peli.kasittele(Syote::Nuoli(0, -1));
    peli.kasittele(Syote::Nuoli(-1, 0));
    assert_eq!(peli.kursori, (0, 0));
    peli.kasittele(Syote::Kirjain('F'));
    assert!(peli.lauta.ruutu((0, 0)).lippu);
    assert_eq!(peli.aanet.jono.last(), Some(&(Aani::Tiili, 0)));
    peli.kasittele(Syote::Hiiri(0, 0));
    assert!(!peli.kursori_nakyy);
}

#[test]
fn kello_kay_ensimmaisesta_avauksesta_ja_pysahtyy_loppuun() {
    let mut peli = pelissa(Vaikeus::Helppo, &[(0, 0), (8, 8)]);
    peli.edista(5000);
    assert_eq!(peli.aika(), 0);
    klikkaa(&mut peli, (1, 1));
    peli.edista(3000);
    assert_eq!(peli.aika(), 3);
    klikkaa(&mut peli, (0, 0));
    assert_eq!(peli.vaihe, Vaihe::Tappio);
    peli.edista(5000);
    assert_eq!(peli.aika(), 3);
    assert!(peli.rajahdys.is_none());
}

#[test]
fn miina_soi_ja_paattaa_pelin_ilman_ennatysta() {
    let mut peli = pelissa(Vaikeus::Helppo, &[(0, 0), (8, 8)]);
    klikkaa(&mut peli, (0, 0));
    assert_eq!(peli.vaihe, Vaihe::Tappio);
    assert_eq!(peli.aanet.jono[0], (Aani::Rajahdys, 0));
    assert!(peli.rajahdys.is_some());
    assert!(!peli.uusi_tulos);
    assert_eq!(peli.ajat, [0; 3]);
}

/// Numeroruutu käynnistää kellon, ja kaukainen tyhjä ruutu avaa kerralla kaiken muun.
fn pelaa_voittoon(peli: &mut Miinaharava, aika_ms: u32) {
    peli.lauta = Miinakentta::miinoista(peli.vaikeus, &[(0, 0)]);
    klikkaa(peli, (1, 1));
    peli.edista(aika_ms);
    peli.aanet.jono.clear();
    klikkaa(peli, (8, 8));
    assert_eq!(peli.vaihe, Vaihe::Voitto);
}

#[test]
fn voitto_kirjaa_parhaan_ajan_vain_jos_se_paranee() {
    let mut peli = pelissa(Vaikeus::Helppo, &[(0, 0)]);
    // Täysi top 3, jottei nimikysely tule väliin.
    peli.taulu.parhaat = vec![
        Tulos {
            nimi: "A".to_string(),
            pisteet: 100_000,
        };
        3
    ];
    pelaa_voittoon(&mut peli, 40_000);
    assert_eq!(peli.ajat, [40, 0, 0]);
    assert!(peli.uusi_tulos);
    assert_eq!(peli.paras_aika(), Some(40));
    assert!(peli.viesti.contains("40"));
    assert!(peli.aanet.jono.contains(&(Aani::Voitto, LOPPU_VIIVE_MS)));
    assert_eq!(
        peli.aanet.jono.last(),
        Some(&(Aani::Voitit, LOPPU_VIIVE_MS + PUHE_VIIVE_MS))
    );

    // hitaampi ei kirjaudu
    peli.uusi_tulos = false;
    peli.aloita();
    pelaa_voittoon(&mut peli, 90_000);
    assert_eq!(peli.ajat, [40, 0, 0]);
    assert!(!peli.uusi_tulos);

    // nopeampi kirjautuu
    peli.aloita();
    pelaa_voittoon(&mut peli, 9_000);
    assert_eq!(peli.ajat, [9, 0, 0]);
    assert!(peli.uusi_tulos);

    // heti aukeava kenttä ei kirjaa nollaa, joka tarkoittaisi läpäisemätöntä tasoa
    peli.ajat = [0; 3];
    peli.aloita();
    peli.lauta = Miinakentta::miinoista(Vaikeus::Helppo, &[(0, 0)]);
    klikkaa(&mut peli, (8, 8));
    assert_eq!(peli.ajat, [1, 0, 0]);
}

#[test]
fn taso_pitaa_oman_aikansa_ja_kellolla_on_kattonsa() {
    let mut peli = pelissa(Vaikeus::Normaali, &[(0, 0)]);
    peli.ajat = [50, 0, 7];
    assert_eq!(peli.paras_aika(), None);
    klikkaa(&mut peli, (1, 1));
    peli.edista(5_000_000);
    assert_eq!(peli.aika(), SUURIN_AIKA);
}

#[test]
fn lopun_napit_ja_kirjaimet_toimivat() {
    let mut peli = pelissa(Vaikeus::Normaali, &[(0, 0)]);
    klikkaa(&mut peli, (0, 0));
    let (x, y) = piirto::napin_keskipiste(Nappi::Uudestaan);
    peli.kasittele(Syote::Klikkaus(x, y));
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
    assert_eq!(peli.lauta.tila, Tila::Alussa);
    assert_eq!(peli.lauta.vaikeus, Vaikeus::Normaali);
    assert_eq!(peli.aika(), 0);

    peli.vaihe = Vaihe::Tappio;
    let (x, y) = piirto::napin_keskipiste(Nappi::Valikko);
    peli.kasittele(Syote::Klikkaus(x, y));
    assert!(peli.lopeta);

    let mut peli = pelissa(Vaikeus::Helppo, &[(0, 0)]);
    peli.kasittele(Syote::Peru);
    assert!(peli.lopeta);
}

#[test]
fn kuva_paivittyy_vain_vilkun_ja_rajahdyksen_tahdissa() {
    let mut peli = Miinaharava::new(1);
    assert_eq!(peli.kuvan_askel(), None);
    peli.kasittele(Syote::Ammu);
    let lepo = peli.kuvan_askel();
    peli.edista(1);
    assert_eq!(peli.kuvan_askel(), lepo);
    peli.edista(VILKKU_MS);
    assert_ne!(peli.kuvan_askel(), lepo);

    peli.rajahdys = Some(peli.kello_ms);
    let rajahdys = peli.kuvan_askel();
    peli.edista(1);
    assert_ne!(peli.kuvan_askel(), rajahdys);
}
