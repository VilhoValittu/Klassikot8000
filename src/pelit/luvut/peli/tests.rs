use super::*;
use crate::yhteinen::tulokset::Tulos;

fn pelissa(ruudut: [[u32; 4]; 4]) -> Luvut {
    let mut luvut = Luvut::new(1);
    luvut.kasittele(Syote::Ammu);
    assert_eq!(luvut.vaihe, Vaihe::Pelissa);
    luvut.lauta = Lukulauta::ruuduista(ruudut);
    luvut
}

const LAHES_VOITTO: [[u32; 4]; 4] = [[1024, 1024, 0, 0], [0; 4], [0; 4], [0; 4]];
// Ainoa mahdollinen siirto täyttää laudan ilman pareja.
const LAHES_LUKOSSA: [[u32; 4]; 4] = [
    [16, 8, 16, 0],
    [16, 8, 16, 8],
    [8, 16, 8, 16],
    [16, 8, 16, 8],
];
// Vasemmalle tuottaa 4 pistettä ja lukitsee laudan, oli uusi luku 2 tai 4.
const LUKKOON_PISTEILLA: [[u32; 4]; 4] = [
    [2, 2, 16, 8],
    [16, 32, 64, 16],
    [4, 16, 8, 32],
    [16, 32, 64, 128],
];

#[test]
fn tallennettu_peli_jatkuu_samasta_tilanteesta() {
    let mut luvut = pelissa([[2, 2, 0, 0], [0; 4], [0; 4], [0; 4]]);
    luvut.kasittele(Syote::Nuoli(0, -1));
    let kesken = luvut.kesken().unwrap();

    let mut uusi = Luvut::new(2);
    uusi.palauta(kesken);
    assert_eq!(uusi.vaihe, Vaihe::Alku);
    assert_eq!(uusi.taulu.paras, 4);
    uusi.kasittele(Syote::Ammu);
    assert_eq!(uusi.vaihe, Vaihe::Pelissa);
    assert_eq!(uusi.lauta, luvut.lauta);
    assert_eq!(uusi.siirtoja, 1);
    assert!(!uusi.jatkuu);
    assert!(uusi.viesti.contains("tallennettua"));
}

#[test]
fn k_hylkaa_tallennetun_mutta_parhaan_ei_ja_ilman_tallennusta_k_ei_tee_mitaan() {
    let mut luvut = Luvut::new(1);
    let alku = luvut.lauta.clone();
    luvut.kasittele(Syote::Kirjain('K'));
    assert_eq!(luvut.lauta, alku);

    luvut.palauta(Kesken {
        ruudut: [[64, 0, 0, 0], [0; 4], [0; 4], [0; 4]],
        pisteet: 500,
        siirrot: 9,
        jatkettu: false,
    });
    assert!(luvut.kesken().is_some());
    luvut.kasittele(Syote::Kirjain('K'));
    assert_eq!(luvut.vaihe, Vaihe::Nimi);
    assert!(luvut.kesken().is_some());
    luvut.kasittele(Syote::Ammu);
    assert_eq!(luvut.taulu.parhaat[0].pisteet, 500);
    assert_eq!(luvut.vaihe, Vaihe::Alku);
    assert!(!luvut.jatkuu);
    assert_eq!(luvut.lauta.pisteet, 0);
    assert_eq!(luvut.siirtoja, 0);
    assert_eq!(luvut.taulu.paras, 500);
    assert_eq!(luvut.kesken(), None);
}

#[test]
fn uuden_pelin_nappi_hylkaa_tallennetun_ja_muu_klikkaus_jatkaa() {
    let kesken = Kesken {
        ruudut: [[64, 0, 0, 0], [0; 4], [0; 4], [0; 4]],
        pisteet: 0,
        siirrot: 9,
        jatkettu: false,
    };
    let (x, y) = piirto::uuden_pelin_keskipiste();

    let mut luvut = Luvut::new(1);
    luvut.palauta(kesken);
    luvut.kasittele(Syote::Klikkaus(x, y));
    assert_eq!(luvut.vaihe, Vaihe::Alku);
    assert!(!luvut.jatkuu);
    assert_eq!(luvut.siirtoja, 0);
    // Ilman tallennusta nappia ei ole, joten sama klikkaus aloittaa pelin.
    luvut.kasittele(Syote::Klikkaus(x, y));
    assert_eq!(luvut.vaihe, Vaihe::Pelissa);

    let mut luvut = Luvut::new(1);
    luvut.palauta(kesken);
    luvut.kasittele(Syote::Klikkaus(0, 0));
    assert_eq!(luvut.vaihe, Vaihe::Pelissa);
    assert_eq!(luvut.siirtoja, 9);
}

#[test]
fn voitto_tallentuu_jatkettuna_ja_tappio_ei_tallennu() {
    let mut luvut = pelissa(LAHES_VOITTO);
    luvut.kasittele(Syote::Nuoli(0, -1));
    assert_eq!(luvut.vaihe, Vaihe::Voitto);
    assert!(luvut.kesken().unwrap().jatkettu);

    let mut luvut = pelissa(LAHES_LUKOSSA);
    luvut.kasittele(Syote::Nuoli(0, 1));
    assert_eq!(luvut.vaihe, Vaihe::Tappio);
    assert_eq!(luvut.kesken(), None);
}

#[test]
fn peli_alkaa_valilyonnilla_ja_nuoli_ei_siirra_alkuruudulla() {
    let mut luvut = Luvut::new(1);
    luvut.kasittele(Syote::Nuoli(0, -1));
    assert_eq!(luvut.vaihe, Vaihe::Alku);
    assert_eq!(luvut.siirtoja, 0);
    luvut.kasittele(Syote::Ammu);
    assert_eq!(luvut.vaihe, Vaihe::Pelissa);
}

#[test]
fn nuolet_siirtavat_ja_pisteet_kertyvat_parhaaseen() {
    let mut luvut = pelissa([[2, 2, 0, 0], [0; 4], [0; 4], [0; 4]]);
    luvut.kasittele(Syote::Nuoli(0, -1));
    assert_eq!(luvut.lauta.arvo((0, 0)), 4);
    assert_eq!(luvut.lauta.pisteet, 4);
    assert_eq!(luvut.taulu.paras, 4);
    assert_eq!(luvut.siirtoja, 1);
    assert!(luvut.viesti.contains("+4"));
    assert!(
        luvut
            .aanet
            .jono
            .contains(&(Aani::Yhdistys, YHDISTYS_VIIVE_MS))
    );
}

#[test]
fn liikkumaton_siirto_ahkaisee_eika_kuluta_vuoroa() {
    let mut luvut = pelissa([[2, 0, 0, 0], [0; 4], [0; 4], [0; 4]]);
    luvut.aanet.jono.clear();
    luvut.kasittele(Syote::Nuoli(0, -1));
    assert!(luvut.ei_liiku);
    assert_eq!(luvut.siirtoja, 0);
    assert_eq!(luvut.aanet.jono, [(Aani::Virhe, 0)]);
}

#[test]
fn voitto_tulee_kerran_ja_jatkaminen_toimii() {
    let mut luvut = pelissa(LAHES_VOITTO);
    luvut.kasittele(Syote::Nuoli(0, -1));
    assert_eq!(luvut.vaihe, Vaihe::Voitto);
    luvut.kasittele(Syote::Kirjain('J'));
    assert_eq!(luvut.vaihe, Vaihe::Pelissa);

    // jo yli tavoitteen: seuraava siirto ei voita uudelleen
    luvut.kasittele(Syote::Nuoli(1, 0));
    assert_eq!(luvut.vaihe, Vaihe::Pelissa);
}

#[test]
fn lukkoon_ajautuminen_on_tappio_eika_tappiosta_voi_jatkaa() {
    let mut luvut = pelissa(LAHES_LUKOSSA);
    // uusi luku voi olla 2 tai 4, ei kumpikaan yhdisty naapureihinsa
    luvut.kasittele(Syote::Nuoli(0, 1));
    assert_eq!(luvut.vaihe, Vaihe::Tappio);
    luvut.kasittele(Syote::Kirjain('J'));
    assert_eq!(luvut.vaihe, Vaihe::Tappio);
}

#[test]
fn uudestaan_nollaa_pelin_mutta_parhaan_ei() {
    let mut luvut = pelissa(LAHES_VOITTO);
    luvut.kasittele(Syote::Nuoli(0, -1));
    let paras = luvut.taulu.paras;
    assert!(paras >= 2048);
    luvut.kasittele(Syote::Kirjain('K'));
    assert_eq!(luvut.vaihe, Vaihe::Nimi);
    luvut.kasittele(Syote::Ammu);
    assert_eq!(luvut.vaihe, Vaihe::Alku);
    assert_eq!(luvut.lauta.pisteet, 0);
    assert_eq!(luvut.siirtoja, 0);
    assert_eq!(luvut.taulu.paras, paras);
}

#[test]
fn tappio_kysyy_nimen_ja_kirjaa_tuloksen_kerran() {
    let mut luvut = pelissa(LUKKOON_PISTEILLA);
    luvut.kasittele(Syote::Nuoli(0, -1));
    assert_eq!(luvut.vaihe, Vaihe::Nimi);
    assert_eq!(luvut.kesken(), None);

    // M on kirjain eikä äänikytkin, ja nimen pituudella on raja
    luvut.kasittele(Syote::VaihdaAanet);
    assert!(luvut.aanet.paalla);
    for kirjain in "ÄXXXXXXXXX".chars() {
        luvut.kasittele(Syote::Kirjain(kirjain));
    }
    assert_eq!(luvut.nimi, "MÄXXXXXX");
    luvut.kasittele(Syote::Pyyhi);
    luvut.kasittele(Syote::Numero(7));
    luvut.kasittele(Syote::Ammu);

    assert_eq!(luvut.vaihe, Vaihe::Tappio);
    assert!(luvut.uusi_tulos);
    assert_eq!(luvut.taulu.parhaat.len(), 1);
    assert_eq!(luvut.taulu.parhaat[0].nimi, "MÄXXXXX7");
    assert_eq!(luvut.taulu.parhaat[0].pisteet, 4);

    // tappion jälkeinen uusi peli ei kirjaa samaa tulosta toiseen kertaan
    luvut.kasittele(Syote::Kirjain('K'));
    assert_eq!(luvut.vaihe, Vaihe::Alku);
    assert_eq!(luvut.taulu.parhaat.len(), 1);
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

    let mut luvut = pelissa(LUKKOON_PISTEILLA);
    luvut.taulu.netti = Some(taysi(1000));
    luvut.kasittele(Syote::Nuoli(0, -1));
    assert_eq!(luvut.vaihe, Vaihe::Tappio);
    assert_eq!(luvut.taulu.lahetys, Vec::new());

    let mut luvut = pelissa(LUKKOON_PISTEILLA);
    luvut.taulu.parhaat = taysi(1000);
    luvut.taulu.netti = Some(taysi(2));
    luvut.kasittele(Syote::Nuoli(0, -1));
    assert_eq!(luvut.vaihe, Vaihe::Nimi);
    luvut.kasittele(Syote::Kirjain('B'));
    luvut.kasittele(Syote::Ammu);
    assert_eq!(luvut.taulu.netti.as_ref().unwrap()[0].nimi, "B");
    assert_eq!(luvut.taulu.lahetys.last().unwrap().pisteet, 4);
}

#[test]
fn ilman_verkkoa_tulos_kirjataan_omalle_listalle_ja_lahetettavaksi() {
    let mut luvut = pelissa(LUKKOON_PISTEILLA);
    luvut.kasittele(Syote::Nuoli(0, -1));
    luvut.kasittele(Syote::Kirjain('B'));
    luvut.kasittele(Syote::Ammu);
    assert_eq!(luvut.taulu.nakyva()[0].nimi, "B");
    assert_eq!(luvut.taulu.netti, None);
    assert_eq!(luvut.taulu.lahetys.last().unwrap().nimi, "B");
}

#[test]
fn listalle_yltamaton_tappio_ei_kysy_nimea() {
    let mut luvut = pelissa(LUKKOON_PISTEILLA);
    luvut.taulu.parhaat = (0..3)
        .map(|_| Tulos {
            nimi: "A".to_string(),
            pisteet: 4,
            aika: None,
        })
        .collect();
    luvut.kasittele(Syote::Nuoli(0, -1));
    assert_eq!(luvut.vaihe, Vaihe::Tappio);
}

#[test]
fn esc_ja_ikkunan_sulkeminen_kirjaavat_tuloksen_oletusnimella() {
    let mut luvut = pelissa(LUKKOON_PISTEILLA);
    luvut.kasittele(Syote::Nuoli(0, -1));
    luvut.kasittele(Syote::Peru);
    assert!(!luvut.lopeta);
    assert_eq!(luvut.taulu.parhaat[0].nimi, "NIMETÖN");

    let mut luvut = pelissa(LUKKOON_PISTEILLA);
    luvut.kasittele(Syote::Nuoli(0, -1));
    luvut.kasittele(Syote::Kirjain('A'));
    luvut.vahvista_nimi();
    assert_eq!(luvut.taulu.parhaat[0].nimi, "A");
    assert_eq!(luvut.vaihe, Vaihe::Tappio);
}

#[test]
fn napit_toimivat_hiirella() {
    let mut luvut = pelissa(LAHES_VOITTO);
    luvut.kasittele(Syote::Nuoli(0, -1));
    let (x, y) = piirto::napin_keskipiste(Nappi::Jatka);
    luvut.kasittele(Syote::Klikkaus(x, y));
    assert_eq!(luvut.vaihe, Vaihe::Pelissa);

    let mut luvut = pelissa(LAHES_VOITTO);
    luvut.kasittele(Syote::Nuoli(0, -1));
    let (x, y) = piirto::napin_keskipiste(Nappi::Valikko);
    luvut.kasittele(Syote::Klikkaus(x, y));
    assert!(luvut.lopeta);
}

#[test]
fn hiiri_nayttaa_suunnan_ja_klikkaus_siirtaa_sinne() {
    let mut luvut = pelissa([[0, 0, 0, 0], [0, 8, 0, 0], [0; 4], [0; 4]]);
    let (x, y) = piirto::laudan_reuna(Suunta::Oikea);
    luvut.kasittele(Syote::Hiiri(x, y));
    assert_eq!(luvut.hiiren_suunta, Some(Suunta::Oikea));
    luvut.kasittele(Syote::Klikkaus(x, y));
    assert_eq!(luvut.lauta.arvo((1, 3)), 8);

    luvut.kasittele(Syote::Klikkaus(0, 0));
    assert_eq!(luvut.siirtoja, 1);
}

#[test]
fn animaatio_paattyy_ja_esc_pyytaa_paluuta() {
    let mut luvut = pelissa([[2, 2, 0, 0], [0; 4], [0; 4], [0; 4]]);
    luvut.kasittele(Syote::Nuoli(0, -1));
    assert!(luvut.viime_siirto.is_some());
    luvut.edista(LIUKU_MS + KIPINA_MS);
    assert!(luvut.viime_siirto.is_none());
    luvut.kasittele(Syote::Peru);
    assert!(luvut.lopeta);
}

#[test]
fn levossa_kuva_ei_vaihdu_ajasta_mutta_animaatio_ja_vilkku_vaihtavat() {
    // alkuruudun malli pyörii
    assert_eq!(Luvut::new(1).kuvan_avain(), None);
    let mut luvut = pelissa([[2, 2, 0, 0], [0; 4], [0; 4], [0; 4]]);
    let lepo = luvut.kuvan_avain();
    assert!(lepo.is_some());
    luvut.edista(60_000);
    assert_eq!(luvut.kuvan_avain(), lepo);

    // siirto liukuu, ja sen jälkeen kuva asettuu taas lepoon
    luvut.kasittele(Syote::Nuoli(0, -1));
    assert_eq!(luvut.kuvan_avain(), None);
    luvut.edista(LIUKU_MS + KIPINA_MS);
    assert!(luvut.kuvan_avain().is_some());

    // hiiren suuntanuoli vilkkuu
    luvut.hiiren_suunta = Some(Suunta::Oikea);
    let ennen = luvut.kuvan_avain();
    luvut.edista(VILKKU_MS);
    assert_ne!(luvut.kuvan_avain(), ennen);
}
