use super::*;
use crate::yhteinen::tulokset::Tulos;

fn pelissa(vaikeus: Vaikeus) -> Suora {
    let mut peli = Suora::new(1);
    peli.vaikeus = vaikeus;
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
    peli.aanet.jono.clear();
    peli
}

fn klikkaa(peli: &mut Suora, ruutu: Paikka) {
    let (x, y) = piirto::ruudun_keskipiste(ruutu);
    peli.kasittele(Syote::Klikkaus(x, y));
}

fn taysi_lista() -> Vec<Tulos> {
    vec![
        Tulos {
            nimi: "A".to_string(),
            pisteet: 100_000,
            aika: None,
        };
        3
    ]
}

/// Pelaajalla neljä vaakaan rivillä 10 ja kone vastaa kaukana: seuraava risti voittaa.
fn voiton_partaalla(peli: &mut Suora) {
    peli.lauta = Lauta::merkeista(
        &[(10, 2), (10, 3), (10, 4), (10, 5)],
        &[(0, 0), (0, 4), (0, 8), (0, 12)],
    );
}

#[test]
fn taso_valitaan_alussa_ja_peli_alkaa_silla() {
    let mut peli = Suora::new(1);
    assert_eq!(peli.vaikeus, Vaikeus::Normaali);
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
    assert_eq!(peli.aanet.jono, [(Aani::Alku, 0)]);
    let mut asetukset = Asetukset::default();
    Pelinakyma::kirjaa_asetukset(&peli, &mut asetukset);
    assert_eq!(asetukset.suora, Vaikeus::Helppo);
}

#[test]
fn klikkaus_merkitsee_ja_kone_vastaa_viiveella() {
    let mut peli = pelissa(Vaikeus::Normaali);
    let avain = peli.kuvan_askel();
    klikkaa(&mut peli, (7, 7));
    assert_eq!(peli.lauta.ruutu((7, 7)), Some(PELAAJA));
    assert_eq!(peli.aanet.jono, [(Aani::Pomppu, 0)]);
    assert!(peli.kone_vuorossa());
    assert_ne!(peli.kuvan_askel(), avain);
    // Koneen vuorolla pelaaja ei voi merkitä.
    klikkaa(&mut peli, (3, 3));
    assert_eq!(peli.lauta.ruutu((3, 3)), None);
    peli.edista(KONE_VIIVE_MS - 1);
    assert_eq!(peli.lauta.siirrot.len(), 1);
    peli.edista(1);
    assert_eq!(peli.lauta.siirrot.len(), 2);
    assert!(!peli.kone_vuorossa());
    assert_eq!(peli.aanet.jono.last(), Some(&(Aani::Tiili, 0)));
    let koneen = peli.lauta.viimeisin().unwrap();
    assert_eq!(peli.lauta.ruutu(koneen), Some(Merkki::Nolla));
    // Varattu ruutu piippaa eikä anna vuoroa koneelle.
    klikkaa(&mut peli, koneen);
    assert_eq!(peli.aanet.jono.last(), Some(&(Aani::Virhe, 0)));
    assert!(!peli.kone_vuorossa());
}

#[test]
fn nappaimisto_liikuttaa_ja_merkitsee() {
    let mut peli = pelissa(Vaikeus::Helppo);
    peli.kasittele(Syote::Ammu);
    assert!(peli.kursori_nakyy);
    assert!(peli.lauta.siirrot.is_empty());
    peli.kasittele(Syote::Nuoli(-1, 0));
    peli.kasittele(Syote::Nuoli(0, 1));
    assert_eq!(peli.kursori, (6, 8));
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.lauta.ruutu((6, 8)), Some(PELAAJA));
    // Kursori pysyy laudalla.
    for _ in 0..20 {
        peli.kasittele(Syote::Nuoli(1, 1));
    }
    assert_eq!(peli.kursori, (KOKO - 1, KOKO - 1));
    peli.kasittele(Syote::Hiiri(0, 0));
    assert!(!peli.kursori_nakyy);
}

#[test]
fn listalle_yltava_voitto_kysyy_nimen_ja_lahtee_nettiin() {
    let mut peli = pelissa(Vaikeus::Vaikea);
    voiton_partaalla(&mut peli);
    klikkaa(&mut peli, (10, 6));
    assert_eq!(peli.vaihe, Vaihe::Nimi);
    assert_eq!(peli.ottelu, (1, 0));
    let pisteet = pisteet(Vaikeus::Vaikea, 5);
    assert_eq!(peli.pisteet(), pisteet);
    assert_eq!(peli.taulu.paras, pisteet);
    assert!(peli.viesti.contains(&pisteet.to_string()));
    assert!(peli.aanet.jono.contains(&(Aani::Voitto, LOPPU_VIIVE_MS)));
    peli.kasittele(Syote::Kirjain('V'));
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.vaihe, Vaihe::Voitto);
    assert_eq!(peli.taulu.parhaat[0].nimi, "V");
    assert_eq!(peli.taulu.lahetys.len(), 1);
    assert!(Pelinakyma::ota_uusi_tulos(&mut peli));
    // Voiton jälkeen lautaan ei voi merkitä, ja K aloittaa uuden pelin.
    klikkaa(&mut peli, (5, 5));
    assert_eq!(peli.lauta.ruutu((5, 5)), None);
    peli.kasittele(Syote::Kirjain('K'));
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
    assert!(peli.lauta.siirrot.is_empty());
    assert_eq!(peli.ottelu, (1, 0));
}

#[test]
fn heikompi_voitto_ei_kysy_nimea_eika_paranna_ennatysta() {
    let mut peli = pelissa(Vaikeus::Helppo);
    peli.taulu.parhaat = taysi_lista();
    peli.taulu.paras = 500;
    voiton_partaalla(&mut peli);
    klikkaa(&mut peli, (10, 6));
    assert_eq!(peli.vaihe, Vaihe::Voitto);
    assert!(peli.taulu.lahetys.is_empty());
    assert_eq!(peli.taulu.paras, 500);
    assert!(!peli.uusi_tulos);
}

#[test]
fn koneen_viisi_paattaa_pelin_tappioon() {
    let mut peli = pelissa(Vaikeus::Normaali);
    peli.lauta = Lauta::merkeista(
        &[(14, 0), (14, 4), (14, 8), (14, 12)],
        &[(3, 3), (3, 4), (3, 5), (3, 6)],
    );
    klikkaa(&mut peli, (12, 12));
    peli.edista(KONE_VIIVE_MS);
    assert_eq!(peli.vaihe, Vaihe::Tappio);
    assert_eq!(peli.ottelu, (0, 1));
    assert_eq!(peli.lauta.voittorivi.len(), 5);
    assert!(peli.aanet.jono.contains(&(Aani::Tappio, LOPPU_VIIVE_MS)));
    assert!(peli.taulu.lahetys.is_empty());
    peli.kasittele(Syote::Kirjain('E'));
    assert!(peli.lopeta);
}

#[test]
fn suljettu_ikkuna_kirjaa_nimea_odottavan_voiton() {
    let mut peli = pelissa(Vaikeus::Normaali);
    voiton_partaalla(&mut peli);
    klikkaa(&mut peli, (10, 1));
    assert_eq!(peli.vaihe, Vaihe::Nimi);
    Pelinakyma::viimeistele(&mut peli);
    assert_eq!(peli.taulu.parhaat[0].nimi, kaanna("yhteinen.oletusnimi"));
}

#[test]
fn kuva_on_paikallaan_kunnes_vilkku_tai_siirto_muuttaa_sita() {
    let mut peli = Suora::new(1);
    assert_eq!(peli.kuvan_askel(), None);
    peli.kasittele(Syote::Ammu);
    let alku = peli.kuvan_askel();
    peli.edista(VILKKU_MS / 2);
    assert_eq!(peli.kuvan_askel(), alku);
    peli.edista(VILKKU_MS);
    assert_ne!(peli.kuvan_askel(), alku);
}
