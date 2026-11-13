use super::*;
use crate::yhteinen::tulokset::Tulos;
use klassikot8000::tiilirikko::{KENTTIA, KORKEUS, LEVEYS, MAILA_L, RIVEJA, SARAKKEITA};

fn lennossa() -> Tiilirikko {
    let mut peli = Tiilirikko::new(1);
    peli.kasittele(Syote::Ammu);
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.lauta.tila, Tila::Lennossa);
    peli
}

/// Viimeinen pallo putoaa seuraavalla askeleella.
fn paattyva(pisteet: u32) -> Tiilirikko {
    let mut peli = lennossa();
    peli.lauta.pisteet = pisteet;
    peli.taulu.paras = pisteet;
    peli.lauta.elamat = 1;
    peli.lauta.pallot[0].paikka = (LEVEYS / 2.0, KORKEUS + 8.0);
    peli.lauta.pallot[0].nopeus = (0.0, 300.0);
    peli.aanet.jono.clear();
    peli
}

#[test]
fn valilyonti_aloittaa_ja_toinen_laukaisee() {
    let mut peli = Tiilirikko::new(1);
    peli.kasittele(Syote::Nuoli(0, 1));
    assert_eq!(peli.vaihe, Vaihe::Alku);
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
    assert_eq!(peli.lauta.tila, Tila::Kiinni);
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.lauta.tila, Tila::Lennossa);
}

#[test]
fn hiiri_ja_pohjassa_oleva_nuoli_liikuttavat_mailaa() {
    let mut peli = Tiilirikko::new(1);
    peli.kasittele(Syote::Ammu);
    peli.kasittele(Syote::Hiiri(piirto::KENTTA_X + 200, 300));
    assert_eq!(peli.lauta.maila_x, 200.0);
    peli.kasittele(Syote::Hiiri(-500, 300));
    assert_eq!(peli.lauta.maila_x, MAILA_L / 2.0);

    peli.ohjaus = 1;
    peli.edista(50);
    assert!(peli.lauta.maila_x > MAILA_L / 2.0);
    assert_eq!(peli.lauta.pallot[0].paikka.0, peli.lauta.maila_x);
}

#[test]
fn hiiri_ohjaa_ikkunan_ulkopuolelta_vain_pelin_aikana() {
    let mut peli = Tiilirikko::new(1);
    assert!(!peli.hiiri_ulkonakin());
    peli.kasittele(Syote::Ammu);
    assert!(peli.hiiri_ulkonakin());
    peli.kasittele(Syote::Kirjain('P'));
    assert!(!peli.hiiri_ulkonakin());
    peli.kasittele(Syote::Kirjain('P'));

    // Lennossakin, superpallon kanssa: tauolla tai lopussa hiiri ei liikuta mitään.
    peli.kasittele(Syote::Ammu);
    peli.lauta.pallot[0].superpallo = true;
    assert!(peli.hiiri_ulkonakin());
    peli.vaihe = Vaihe::Tappio;
    assert!(!peli.hiiri_ulkonakin());
}

#[test]
fn tauko_pysayttaa_pallon_ja_jatkuu_samasta() {
    let mut peli = lennossa();
    peli.kasittele(Syote::Kirjain('P'));
    let paikka = peli.lauta.pallot[0].paikka;
    peli.edista(100);
    assert_eq!(peli.lauta.pallot[0].paikka, paikka);
    peli.kasittele(Syote::Kirjain('P'));
    peli.edista(100);
    assert_ne!(peli.lauta.pallot[0].paikka, paikka);

    peli.keskeyta();
    assert_eq!(peli.vaihe, Vaihe::Tauko);
}

#[test]
fn kohdistuksen_menetys_ei_keskeyta_kiinni_olevaa_palloa() {
    let mut peli = Tiilirikko::new(1);
    peli.kasittele(Syote::Ammu);
    peli.keskeyta();
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
}

#[test]
fn iso_aikahyppy_ei_siirra_palloa_kerralla_pitkalle() {
    let mut peli = lennossa();
    let alku = peli.lauta.pallot[0].paikka;
    peli.edista(60_000);
    let matka = (peli.lauta.pallot[0].paikka.1 - alku.1).abs();
    assert!(matka < 100.0, "{matka}");
}

#[test]
fn viimeisen_pallon_menetys_paattaa_pelin_ja_paras_sailyy_uudessa() {
    let mut peli = paattyva(40);
    peli.taulu.parhaat = (0..3)
        .map(|_| Tulos {
            nimi: "A".to_string(),
            pisteet: 40,
            aika: None,
        })
        .collect();
    peli.edista(ASKEL_MS);
    assert_eq!(peli.vaihe, Vaihe::Tappio);
    assert!(peli.uusi_tulos);
    assert_eq!(
        peli.aanet.jono,
        [
            (Aani::Tappio, LOPPU_VIIVE_MS),
            (Aani::PeliOhi, LOPPU_VIIVE_MS + PUHE_VIIVE_MS)
        ]
    );

    peli.kasittele(Syote::Kirjain('K'));
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
    assert_eq!(peli.lauta.pisteet, 0);
    assert_eq!(peli.taulu.paras, 40);
}

#[test]
fn listalle_yltava_tulos_kysyy_nimen_ja_palaa_loppuruutuun() {
    let mut peli = paattyva(40);
    peli.edista(ASKEL_MS);
    assert_eq!(peli.vaihe, Vaihe::Nimi);

    // Laukaisuun painettu välilyönti ei kuittaa tyhjää nimeä, ja K on nimessä kirjain.
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.vaihe, Vaihe::Nimi);
    peli.kasittele(Syote::Kirjain('K'));
    peli.kasittele(Syote::VaihdaAanet);
    assert!(peli.aanet.paalla);
    peli.uusi_tulos = false;
    peli.kasittele(Syote::Ammu);

    assert_eq!(peli.vaihe, Vaihe::Tappio);
    assert!(peli.uusi_tulos);
    assert_eq!(peli.taulu.parhaat[0].nimi, "KM");
    assert_eq!(peli.taulu.parhaat[0].pisteet, 40);
}

#[test]
fn esc_ja_sulkeminen_kirjaavat_tuloksen_oletusnimella() {
    let mut peli = paattyva(40);
    peli.edista(ASKEL_MS);
    peli.kasittele(Syote::Peru);
    assert!(!peli.lopeta);
    assert_eq!(peli.vaihe, Vaihe::Tappio);
    assert_eq!(peli.taulu.parhaat[0].nimi, "NIMETÖN");

    let mut peli = paattyva(40);
    peli.edista(ASKEL_MS);
    peli.vahvista_nimi();
    assert_eq!(peli.taulu.parhaat.len(), 1);
    peli.vahvista_nimi();
    assert_eq!(peli.taulu.parhaat.len(), 1);
}

#[test]
fn nettilista_ratkaisee_nimen_kysymisen_kun_se_on_tiedossa() {
    let taysi = |pisteet| {
        (0..3)
            .map(|_| Tulos {
                nimi: "A".to_string(),
                pisteet,
                aika: None,
            })
            .collect::<Vec<_>>()
    };

    let mut peli = paattyva(40);
    peli.taulu.netti = Some(taysi(1000));
    peli.edista(ASKEL_MS);
    assert_eq!(peli.vaihe, Vaihe::Tappio);
    assert_eq!(peli.taulu.lahetys, Vec::new());

    let mut peli = paattyva(40);
    peli.taulu.parhaat = taysi(1000);
    peli.taulu.netti = Some(taysi(10));
    peli.edista(ASKEL_MS);
    assert_eq!(peli.vaihe, Vaihe::Nimi);
    peli.kasittele(Syote::Kirjain('B'));
    peli.kasittele(Syote::Ammu);
    let lista = peli.taulu.netti.as_ref().unwrap();
    assert_eq!(lista[0].nimi, "B");
    assert_eq!(peli.taulu.lahetys.last().unwrap().pisteet, 40);
}

#[test]
fn ilman_verkkoa_tulos_kirjataan_omalle_listalle_ja_lahetettavaksi() {
    let mut peli = paattyva(40);
    peli.edista(ASKEL_MS);
    peli.kasittele(Syote::Kirjain('B'));
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.taulu.nakyva()[0].nimi, "B");
    assert_eq!(peli.taulu.netti, None);
    assert_eq!(peli.taulu.lahetys.last().unwrap().nimi, "B");
}

#[test]
fn nollatulos_ei_paase_listalle() {
    let mut peli = paattyva(0);
    peli.edista(ASKEL_MS);
    assert_eq!(peli.vaihe, Vaihe::Tappio);
}

#[test]
fn viimeisen_kentan_voitto_palaa_nimen_jalkeen_voittoon_ja_bonus_nostaa_parasta() {
    let mut peli = Tiilirikko::new(1);
    peli.kasittele(Syote::Ammu);
    peli.lauta = Tiilipeli::new(KENTTIA - 1, &mut Arpa::new(1));
    peli.kasittele(Syote::Ammu);
    peli.lauta.tiilet = [[0; SARAKKEITA]; RIVEJA];
    peli.lauta.tiilet[0][0] = 1;
    peli.lauta.pallot[0].paikka = (30.0, 80.0);
    peli.lauta.pallot[0].nopeus = (0.0, -300.0);
    peli.edista(ASKEL_MS);
    assert_eq!(peli.vaihe, Vaihe::Nimi);
    assert!(peli.lauta.pisteet > 1000);
    assert_eq!(peli.taulu.paras, peli.lauta.pisteet);
    assert!(peli.viesti.contains("Aikabonus"));

    peli.kasittele(Syote::Kirjain('A'));
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.vaihe, Vaihe::Voitto);
}

#[test]
fn rikkoutuva_tiili_jattaa_kipinan_ja_soittaa_aanen() {
    let mut peli = lennossa();
    peli.lauta.tiilet = [[0; SARAKKEITA]; RIVEJA];
    peli.lauta.tiilet[0][0] = 1;
    peli.lauta.tiilet[7][10] = 1;
    peli.lauta.pallotiilet = [[false; SARAKKEITA]; RIVEJA];
    peli.lauta.supertiilet = [[false; SARAKKEITA]; RIVEJA];
    peli.lauta.pallot[0].paikka = (30.0, 80.0);
    peli.lauta.pallot[0].nopeus = (0.0, -300.0);
    peli.aanet.jono.clear();
    peli.edista(ASKEL_MS);
    assert_eq!(peli.kipinat.len(), 1);
    assert_eq!(peli.aanet.jono, [(Aani::Tiili, 0)]);
    assert_eq!(peli.taulu.paras, 10);
    peli.edista(KIPINA_MS);
    assert!(peli.kipinat.is_empty());
}

#[test]
fn toinen_pallo_kertoo_tulostaan_ja_menetys_ei_vie_elamaa() {
    let mut peli = lennossa();
    peli.lauta.tiilet = [[0; SARAKKEITA]; RIVEJA];
    peli.lauta.tiilet[0][0] = 1;
    peli.lauta.tiilet[7][10] = 1;
    peli.lauta.supertiilet = [[false; SARAKKEITA]; RIVEJA];
    peli.lauta.pallotiilet[0][0] = true;
    peli.lauta.pallot[0].paikka = (30.0, 80.0);
    peli.lauta.pallot[0].nopeus = (0.0, -300.0);
    peli.aanet.jono.clear();
    peli.edista(ASKEL_MS);
    assert_eq!(peli.lauta.pallot.len(), 2);
    assert_eq!(peli.aanet.jono, [(Aani::Tiili, 0), (Aani::Yhdistys, 0)]);
    assert_eq!(peli.viesti, kaanna("tiilirikko.uusi_pallo"));

    peli.lauta.pallot[1].paikka = (LEVEYS / 2.0, KORKEUS + 8.0);
    peli.lauta.pallot[1].nopeus = (0.0, 300.0);
    peli.aanet.jono.clear();
    peli.edista(ASKEL_MS);
    assert_eq!(peli.lauta.pallot.len(), 1);
    assert_eq!(peli.lauta.elamat, klassikot8000::tiilirikko::ELAMIA);
    assert_eq!(peli.aanet.jono, [(Aani::Puhkus, 0)]);
    assert_eq!(peli.viesti, kaanna("tiilirikko.toinen_meni"));
}

#[test]
fn esc_kesken_pelin_pysayttaa_ja_toinen_kysyy_nimen() {
    let mut peli = paattyva(500);
    peli.kasittele(Syote::Peru);
    assert_eq!(peli.vaihe, Vaihe::Tauko);
    assert!(!peli.lopeta);
    peli.kasittele(Syote::Peru);
    assert_eq!(peli.vaihe, Vaihe::Nimi);
    assert!(!peli.lopeta);
    peli.edista(1000);
    assert_eq!(peli.lauta.elamat, 1);

    peli.kasittele(Syote::Kirjain('B'));
    peli.kasittele(Syote::Ammu);
    assert!(peli.lopeta);
    assert_eq!(peli.taulu.parhaat[0].nimi, "B");
    assert_eq!(peli.taulu.lahetys.last().unwrap().pisteet, 500);
}

#[test]
fn listalle_yltamaton_tulos_poistuu_heti_ja_sulkeminen_kirjaa_oletusnimella() {
    let mut peli = paattyva(500);
    peli.taulu.netti = Some(vec![
        Tulos {
            nimi: "A".to_string(),
            pisteet: 1000,
            aika: None,
        };
        3
    ]);
    peli.kasittele(Syote::Peru);
    assert!(!peli.lopeta);
    peli.kasittele(Syote::Peru);
    assert!(peli.lopeta);
    assert_eq!(peli.taulu.lahetys, Vec::new());

    let mut peli = paattyva(500);
    peli.poistu();
    peli.vahvista_nimi();
    assert_eq!(peli.taulu.parhaat[0].nimi, "NIMETÖN");
}

#[test]
fn lopun_napit_ja_esc_toimivat() {
    let mut peli = Tiilirikko::new(1);
    peli.vaihe = Vaihe::Voitto;
    let (x, y) = loppunapit::napin_keskipiste(Loppunappi::Uudestaan);
    peli.kasittele(Syote::Klikkaus(x, y));
    assert_eq!(peli.vaihe, Vaihe::Pelissa);

    peli.vaihe = Vaihe::Tappio;
    let (x, y) = loppunapit::napin_keskipiste(Loppunappi::Valikko);
    peli.kasittele(Syote::Klikkaus(x, y));
    assert!(peli.lopeta);

    // Esc pysäyttää, välilyönti jatkaa ja tauon aikana Esc palaa valikkoon.
    let mut peli = lennossa();
    peli.kasittele(Syote::Peru);
    assert_eq!(peli.vaihe, Vaihe::Tauko);
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
    peli.kasittele(Syote::Peru);
    assert!(!peli.lopeta);
    peli.kasittele(Syote::Peru);
    assert!(peli.lopeta);
}
