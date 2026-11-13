use super::*;
use crate::pelit::laivanupotus::peli::tulos_teksti;
use crate::yhteinen::kieli::kaikilla_kielilla;
use klassikot8000::laivanupotus::SUURIN_TULOS;

pub fn vaikeuden_keskipiste(vaikeus: Vaikeus) -> (i32, i32) {
    let (_, teksti, x) = TASOT.iter().find(|t| t.0 == vaikeus).copied().unwrap();
    (x + napin_leveys(teksti) / 2, TASO_Y + NAPPI_K / 2)
}

pub fn ruudun_keskipiste(ruutu: Ruutu) -> (i32, i32) {
    solun_keskipiste(VASTUSTAJA_X, ruutu)
}

pub fn oman_ruudun_keskipiste(ruutu: Ruutu) -> (i32, i32) {
    solun_keskipiste(OMA_X, ruutu)
}

pub fn napin_keskipiste(nappi: Nappi) -> (i32, i32) {
    let (_, teksti, x) = NAPIT
        .iter()
        .chain(&SIJOITUSNAPIT)
        .find(|n| n.0 == nappi)
        .copied()
        .unwrap();
    (x + napin_leveys(teksti) / 2, NAPPI_Y + NAPPI_K / 2)
}

#[test]
fn jokainen_ruutu_loytyy_keskipisteestaan() {
    for rivi in 0..KOKO {
        for sarake in 0..KOKO {
            let ruutu = Ruutu::new(rivi, sarake).unwrap();
            let (x, y) = ruudun_keskipiste(ruutu);
            assert_eq!(ruutu_pisteesta(x, y), Some(ruutu));
        }
    }
}

#[test]
fn oman_laudan_ruudut_loytyvat_eivatka_osu_vastustajan_laudalle() {
    for rivi in 0..KOKO {
        for sarake in 0..KOKO {
            let ruutu = Ruutu::new(rivi, sarake).unwrap();
            let (x, y) = oman_ruudun_keskipiste(ruutu);
            assert_eq!(oma_ruutu_pisteesta(x, y), Some(ruutu));
            let (x, y) = ruudun_keskipiste(ruutu);
            assert_eq!(oma_ruutu_pisteesta(x, y), None);
        }
    }
}

#[test]
fn oma_lauta_ja_reunat_eivat_ole_kohteita() {
    assert_eq!(ruutu_pisteesta(OMA_X + 5, LAUTA_Y + 5), None);
    assert_eq!(ruutu_pisteesta(VASTUSTAJA_X - 1, LAUTA_Y), None);
    assert_eq!(ruutu_pisteesta(VASTUSTAJA_X, LAUTA_Y + SOLU * 10), None);
}

#[test]
fn pisin_kierros_mahtuu_paneeliin() {
    kaikilla_kielilla(|| {
        let ruutu = Ruutu::new(9, 1).unwrap();
        for (nimi, _) in klassikot8000::laivanupotus::LAIVAT {
            // kaksi viestiä, lopputulos ja napit
            let tulos = Laukaustulos::Upposi(nimi);
            let a = tulos_teksti("laivanupotus.sina_ammuit", ruutu, tulos);
            let b = tulos_teksti("laivanupotus.vastustaja_ampui", ruutu, tulos);
            let rivit = rivita(&a, RIVILLE_MERKKEJA).len() + rivita(&b, RIVILLE_MERKKEJA).len();
            assert!(rivit + 2 <= PANEELI_RIVEJA, "{a} {b}");
            // Voitossa toisen viestin paikalla on voiton tulos.
            let voitto = muotoile("laivanupotus.voiton_pisteet", &[&100, &SUURIN_TULOS]);
            let rivit =
                rivita(&a, RIVILLE_MERKKEJA).len() + rivita(&voitto, RIVILLE_MERKKEJA).len();
            assert!(rivit + 2 <= PANEELI_RIVEJA, "{a} {voitto}");
            // Nimi ei saa osua tilaston palkkiin.
            let leveys = laivan_nimi(nimi).chars().count() as i32 * MERKKI;
            assert!(leveys < TILASTO_PALKKI_X, "{nimi}");
        }
        assert!(kaanna(OHJE).chars().count() <= RIVILLE_MERKKEJA);
        for rivi in SIJOITUS_OHJEET.map(kaanna).into_iter().chain([
            muotoile(TILASTO, &[&999, &999]).as_str(),
            muotoile(ENNATYS, &[&100]).as_str(),
        ]) {
            assert!(rivi.chars().count() <= RIVILLE_MERKKEJA, "{rivi}");
        }
        for pari in SIJOITUSNAPIT.windows(2) {
            assert!(pari[0].2 + napin_leveys(pari[0].1) < pari[1].2);
        }
        let (_, viimeinen, x) = SIJOITUSNAPIT[1];
        assert!(x + napin_leveys(viimeinen) <= LEVEYS as i32);
        assert!(kaanna(AMMUIT_JO).chars().count() <= RIVILLE_MERKKEJA);
        assert!(kaanna(TASO_OTSIKKO).chars().count() <= 6);
        for pari in NAPIT.windows(2) {
            assert!(pari[0].2 + napin_leveys(pari[0].1) < pari[1].2);
        }
        for pari in TASOT.windows(2) {
            assert!(pari[0].2 + napin_leveys(pari[0].1) < pari[1].2);
        }
    });
    alkuohje_mahtuu(&ALKU_OSAT, &ALKU_RIVIT, TASO_Y);
}

#[test]
fn kaikki_vaiheet_piirtyvat_kaatumatta_tehosteiden_aikana() {
    let mut kuva = vec![0; LEVEYS * KORKEUS];
    let mut peli = Peli::new(3);
    piirra(&peli, &mut kuva);
    peli.vaihe = Vaihe::Sijoitus;
    piirra(&peli, &mut kuva);
    peli.kasittele(crate::yhteinen::syote::Syote::Numero(1));
    peli.tilastot[1].vahiten_laukauksia = 40;
    piirra(&peli, &mut kuva);
    peli.vaihe = Vaihe::Ammunta;
    for i in 0..KOKO * KOKO {
        peli.ammu(Ruutu::new(i / KOKO, i % KOKO).unwrap());
        for _ in 0..4 {
            peli.edista(100);
            piirra(&peli, &mut kuva);
        }
    }
    peli.vaihe = Vaihe::Nimi;
    peli.nimi = "AMIRAALI".to_string();
    piirra(&peli, &mut kuva);
    peli.vaihe = Vaihe::Alku;
    peli.taulu.parhaat = vec![
        crate::yhteinen::tulokset::Tulos {
            nimi: "AMIRAALI".to_string(),
            pisteet: SUURIN_TULOS,
            aika: None,
        };
        3
    ];
    piirra(&peli, &mut kuva);
}
