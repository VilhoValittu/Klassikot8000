use super::*;
use crate::yhteinen::kieli::kaikilla_kielilla;
use crate::yhteinen::syote::Syote;
use klassikot8000::blackjack::{ALKURAHA, ENITEN_KASIA, Maa, PIENIN_PANOS};

pub fn napin_keskipiste(vaihe: Vaihe, vuoro: Vuoro, nappi: Nappi) -> (i32, i32) {
    let i = (napit(vaihe, vuoro).iter())
        .position(|n| n.0 == nappi)
        .unwrap() as i32;
    (SIVU_TEKSTI_X + 12, NAPIT_Y + i * NAPPI_ASKEL + NAPPI_K / 2)
}

fn kortit(arvot: &[u8]) -> Vec<Kortti> {
    arvot
        .iter()
        .map(|&arvo| Kortti {
            maa: Maa::Risti,
            arvo,
        })
        .collect()
}

#[test]
fn kadet_mahtuvat_poydalle_kaikissa_asetteluissa() {
    // Kuudesta pakasta saa enintään 13 kortin käden ennen kuin 21 täyttyy.
    for kasia in 1..=ENITEN_KASIA {
        for kortteja in 1..=13 {
            for kasi in 0..kasia {
                let (x, dx, dy) = kaden_asettelu(kasia, kasi, kortteja);
                let vali = kortteja as i32 - 1;
                assert!(x >= POYTA_X, "{kasia} {kortteja}");
                assert!(
                    x + KORTTI_L + vali * dx <= SIVU_X - 12,
                    "{kasia} {kortteja}"
                );
                let ala = KASI_Y + KORTTI_K + vali * dy;
                assert!(ala < RUUDUN_KORKEUS as i32, "{kasia} {kortteja}");
                assert!(dx == 0 || dx >= VIUHKA_PIENIN);
            }
        }
    }
    // Yksi käsi on aina viuhkana, jotta tavallinen kierros näyttää pöydältä.
    for kortteja in 1..=13 {
        assert_eq!(kaden_asettelu(1, 0, kortteja).2, 0);
    }
    const { assert!(JAKAJA_Y + KORTTI_K < SAANTO_Y) };
    const { assert!(SAANTO_Y + MERKKI < KASI_OTSIKKO_Y) };
    const { assert!(KASI_TULOS_Y + TULOS.1 < KASI_Y) };
}

#[test]
fn sivun_ja_poydan_tekstit_mahtuvat_kummallakin_kielella() {
    let tila = SIVU_L - 36;
    assert!(SIVU_MERKKEJA as i32 * MERKKI <= tila);
    let alin = NAPIT_Y + PANOSTUS_NAPIT.len() as i32 * NAPPI_ASKEL;
    assert!(alin < RUUDUN_KORKEUS as i32);
    kaikilla_kielilla(|| {
        for avain in ["blackjack.sivu_nuolet", "blackjack.sivu_aanet"] {
            assert!(kaanna(avain).chars().count() <= SIVU_MERKKEJA, "{avain}");
        }
        for &(_, avain) in PANOSTUS_NAPIT
            .iter()
            .chain(&VAKUUTUS_NAPIT)
            .chain(&PELI_NAPIT)
            .chain(&LOPPU_NAPIT)
        {
            assert!(napin_leveys(avain) <= tila, "{avain}");
        }
        for avain in [
            "blackjack.aseta_panos",
            "blackjack.vakuutus_havisi",
            "blackjack.ei_tuplausta",
            "blackjack.ei_halkaisua",
            "blackjack.viesti_tasan",
        ] {
            assert!(rivita(kaanna(avain), SIVU_MERKKEJA).len() <= 4, "{avain}");
        }
        for (avain, arvo) in [
            ("blackjack.konkurssi", ALKURAHA),
            ("blackjack.viesti_voitto", 2_000_000_001),
            ("blackjack.viesti_tappio", 2_000_000_001),
            ("blackjack.viesti_blackjack", 2_000_000_001),
            ("blackjack.vakuutus_tarjous", 2_000_000_001),
            ("blackjack.tasaraha_tarjous", 2_000_000_001),
        ] {
            let teksti = muotoile(avain, &[&raha(arvo)]);
            assert!(rivita(&teksti, SIVU_MERKKEJA).len() <= 4, "{avain}");
        }
        let teksti = muotoile("blackjack.viesti_vakuutus", &[&"99999,5", &"99999,5"]);
        assert!(rivita(&teksti, SIVU_MERKKEJA).len() <= 4);
        let saanto = kaanna("blackjack.saanto").chars().count() as i32 * MERKKI;
        assert!(saanto <= POYTA_L);
        // Neljännes pöydästä riittää käden riville kuusinumeroisella panoksella.
        let kaista = POYTA_L / ENITEN_KASIA as i32;
        for avain in [
            "blackjack.kasi_panos",
            "blackjack.tulos_voitto",
            "blackjack.tulos_tappio",
            "blackjack.tulos_yli",
        ] {
            let teksti = muotoile(avain, &[&"99999,5"]);
            assert!(teksti.chars().count() as i32 * TULOS.2 <= kaista, "{avain}");
        }
        let otsikko = format!("{} 7 / 17", kaanna("blackjack.jakaja"));
        assert!(otsikko.chars().count() as i32 * OTSIKKO.2 <= POYTA_L);
    });
    alkuohje_mahtuu(&ALKU_OSAT, &ALKU_RIVIT, ALOITUS_Y);
}

#[test]
fn alkuohje_kertoo_oikeat_rahat_ja_panoksen() {
    kaikilla_kielilla(|| {
        let ohje = kaanna("blackjack.ohje_2");
        assert!(ohje.contains(&raha(ALKURAHA)), "{ohje}");
        assert!(ohje.contains(&raha(PIENIN_PANOS)), "{ohje}");
    });
}

#[test]
fn napit_loytyvat_vaiheittain_ja_valit_eivat_ole_kohteita() {
    for (vaihe, vuoro) in [
        (Vaihe::Pelissa, Vuoro::Panostus),
        (Vaihe::Pelissa, Vuoro::Vakuutus),
        (Vaihe::Pelissa, Vuoro::Pelaaja),
        (Vaihe::Pelissa, Vuoro::Jakaja),
        (Vaihe::Loppu, Vuoro::Panostus),
    ] {
        for &(nappi, _) in napit(vaihe, vuoro) {
            let (x, y) = napin_keskipiste(vaihe, vuoro, nappi);
            assert_eq!(nappi_pisteesta(vaihe, vuoro, x, y), Some(nappi));
        }
    }
    let (panostus, vali) = (Vuoro::Panostus, NAPIT_Y + NAPPI_K);
    assert_eq!(
        nappi_pisteesta(Vaihe::Pelissa, panostus, SIVU_TEKSTI_X, vali),
        None
    );
    assert_eq!(
        nappi_pisteesta(Vaihe::Pelissa, panostus, SIVU_TEKSTI_X, NAPIT_Y - 1),
        None
    );
}

#[test]
fn puolikas_nakyy_kielen_mukaan_ja_vain_tarvittaessa() {
    assert_eq!(raha(ALKURAHA), "300");
    assert_eq!(raha(3), "1,5");
    kieli::aseta(Kieli::Englanti);
    assert_eq!(raha(3), "1.5");
    assert_eq!(raha(0), "0");
    kieli::aseta(Kieli::Suomi);
}

#[test]
fn pehmea_summa_nayttaa_molemmat_arvot_ja_yli_mennyt_tappion() {
    assert_eq!(summateksti(&kortit(&[1, 6])), "7 / 17");
    assert_eq!(summateksti(&kortit(&[1, 13])), "21");
    assert_eq!(summateksti(&kortit(&[10, 6])), "16");
    let kasi = Kasi {
        kortit: kortit(&[10, 6, 9]),
        panos: 5,
        tuplattu: false,
        halkaistu: false,
        tulos: None,
    };
    assert_eq!(kaden_rivi(&kasi).1, TAPPIO_VARI);
}

#[test]
fn koko_peli_piirtyy_kaatumatta() {
    let mut ruutu = vec![0; RUUDUN_LEVEYS * RUUDUN_KORKEUS];
    let mut klassinen = vec![0; LEVEYS * KORKEUS];
    let mut peli = Blackjack::new(3);
    piirra_alku(&peli, &mut klassinen);
    peli.kasittele(Syote::Ammu);
    for _ in 0..200 {
        piirra(&peli, &mut ruutu);
        peli.kasittele(Syote::Ammu);
        piirra(&peli, &mut ruutu);
        peli.kasittele(Syote::Kirjain('N'));
        peli.kasittele(Syote::Kirjain('H'));
        piirra(&peli, &mut ruutu);
        peli.kasittele(Syote::Kirjain('O'));
        peli.kasittele(Syote::Kirjain('J'));
        peli.kasittele(Syote::Kirjain('J'));
        peli.kasittele(Syote::Kirjain('J'));
        piirra(&peli, &mut ruutu);
        peli.edista(5000);
    }
    peli.poyta.rahat = u32::MAX / 2;
    piirra(&peli, &mut ruutu);
    peli.vaihe = Vaihe::Nimi;
    peli.nimi = "ÄÖ".to_string();
    piirra(&peli, &mut ruutu);
}

#[test]
fn rahat_ja_voitto_mahtuvat_sivulle() {
    let rivi = format!(
        "{} (+{})",
        raha(2_000_000_001),
        klassikot8000::blackjack::SUURIN_TULOS
    );
    assert!(rivi.chars().count() as i32 * OTSIKKO.2 <= SIVU_L - 36);
}
