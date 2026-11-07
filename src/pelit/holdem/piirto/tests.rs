use super::*;
use crate::yhteinen::kieli::kaikilla_kielilla;
use crate::yhteinen::syote::Syote;
use klassikot8000::holdem::ALKURAHA;

pub fn napin_keskipiste(napisto: Napisto, nappi: Nappi) -> (i32, i32) {
    let i = (napit(napisto).iter()).position(|&n| n == nappi).unwrap() as i32;
    (
        SIVU_TEKSTI_X + NAPIN_L / 2,
        NAPIT_Y + i * NAPPI_ASKEL + NAPPI_K / 2,
    )
}

const KAIKKI_NAPISTOT: [Napisto; 4] = [
    Napisto::Vuoro,
    Napisto::Odotus,
    Napisto::JakoOhi,
    Napisto::Loppu,
];

#[test]
fn poyta_mahtuu_ruudulle() {
    const { assert!(TEKO_Y + 24 + TULOS.1 <= PAIKKA_Y + PAIKKA_K) };
    const { assert!(POTTI_Y + OTSIKKO.1 < YHTEISET_Y) };
    const { assert!(YHTEISET_Y + KORTTI_K < OMA_Y) };
    const { assert!(OMA_Y + OMA_K <= KASI_RIVI_Y) };
    const { assert!(SAANTO_Y + 34 < RUUDUN_KORKEUS as i32) };
    const { assert!(5 * YHTEISET_ASKEL <= POYTA_L) };
    // Vastustajan kortit eivät peitä toisiaan, ja koko arvo näkyy näytössä.
    const { assert!(KORTTI_L < KORTTI_ASKEL) };
    const { assert!(KORTTI_L + KORTTI_ASKEL + 16 <= PAIKKA_L) };
    const { assert!(NAPIT_Y + VUORO_NAPIT.len() as i32 * NAPPI_ASKEL < RUUDUN_KORKEUS as i32) };
}

#[test]
fn tekstit_mahtuvat_kummallakin_kielella() {
    let tila = NAPIN_L - 24;
    assert!(SIVU_MERKKEJA as i32 * MERKKI <= SIVU_L - 36);
    kaikilla_kielilla(|| {
        let summa = 5 * ALKURAHA;
        for avain in [
            "holdem.nappi_maksa",
            "holdem.nappi_korota",
            "holdem.nappi_panosta",
        ] {
            let leveys = muotoile(avain, &[&summa]).chars().count() as i32 * MERKKI;
            assert!(leveys <= tila, "{avain}");
        }
        for avain in [
            "holdem.nappi_tarkista",
            "holdem.nappi_kippaa",
            "holdem.nappi_pienin",
            "holdem.nappi_puoli",
            "holdem.nappi_potti",
            "holdem.nappi_kaikki",
            "holdem.nappi_jako",
            "holdem.nappi_jatka",
            "yhteinen.uudestaan",
            "yhteinen.valikkoon",
        ] {
            assert!(
                kaanna(avain).chars().count() as i32 * MERKKI <= tila,
                "{avain}"
            );
        }
        for avain in ["holdem.sivu_nuolet", "holdem.sivu_aanet"] {
            assert!(kaanna(avain).chars().count() <= SIVU_MERKKEJA, "{avain}");
        }

        // Pisimmät viestit mahtuvat laatikkoon neljälle riville.
        let kaikki = format!(
            "{} +2500, {} +2500, {} +2500",
            pelaajan_nimi(1),
            pelaajan_nimi(2),
            pelaajan_nimi(3)
        );
        let nimet: Vec<&str> = (0..PELAAJIA).map(pelaajan_nimi).collect();
        let kaksi =
            |a: usize, b: usize| format!("{} +2500, {} +2500", pelaajan_nimi(a), pelaajan_nimi(b));
        let viestit = [
            muotoile(
                "holdem.voitot_kasi",
                &[&kaikki, &kaanna("holdem.kasi_kuningasvarisuora")],
            ),
            muotoile(
                "holdem.tasan",
                &[&nimet.join(", "), &kaanna("holdem.kasi_kuningasvarisuora")],
            ),
            muotoile("holdem.sivupotti_tulos", &[&kaksi(2, 4), &kaksi(1, 3)]),
            muotoile(
                "holdem.muut_kippasivat",
                &[&format!("{} +5000", pelaajan_nimi(1))],
            ),
            muotoile("holdem.havio", &[&5]),
            muotoile("holdem.voitto", &[&summa]),
            muotoile("holdem.maksettavaa", &[&summa]),
            muotoile("holdem.miettii", &[&pelaajan_nimi(1)]),
            kaanna("holdem.ei_korotusta").to_string(),
            kaanna("holdem.sinun_vuorosi").to_string(),
            kaanna("holdem.kortit_esiin").to_string(),
            kaanna("holdem.seuraava_kortti").to_string(),
        ];
        for viesti in viestit {
            assert!(rivita(&viesti, SIVU_MERKKEJA).len() <= 4, "{viesti}");
        }

        // Vastustajan kohtaan mahtuu pisin käsi ja nimi merkkien viereen.
        let tila = PAIKKA_L / TULOS.2;
        for avain in [
            "holdem.kasi_kuningasvarisuora",
            "holdem.kasi_kaksi_paria",
            "holdem.pudonnut",
            "holdem.teko_tarkisti",
            "holdem.teko_kippasi",
        ] {
            assert!(kaanna(avain).chars().count() as i32 <= tila, "{avain}");
        }
        for i in 1..PELAAJIA {
            let nimi = pelaajan_nimi(i).chars().count() as i32 * OTSIKKO.2;
            assert!(nimi + 2 * (MERKKI_L + 4) + 24 <= PAIKKA_L, "{i}");
        }
        for avain in [
            "holdem.merkki_iso",
            "holdem.merkki_pieni",
            "holdem.merkki_nappi",
        ] {
            assert!(
                kaanna(avain).chars().count() as i32 * MERKKI <= MERKKI_L,
                "{avain}"
            );
        }
        let oma = muotoile(
            "holdem.oma_kasi",
            &[&kaanna("holdem.kasi_kuningasvarisuora").to_uppercase()],
        );
        assert!(oma.chars().count() as i32 * OTSIKKO.2 <= POYTA_L);
        let saanto = muotoile("holdem.saanto", &[&999, &99, &10]);
        assert!(
            saanto.chars().count() as i32 * MERKKI <= POYTA_L,
            "{saanto}"
        );
        let potti = muotoile("holdem.potti_rivi", &[&summa]) + &muotoile("holdem.sivupotit", &[&3]);
        assert!(potti.chars().count() as i32 * OTSIKKO.2 <= POYTA_L);
    });
    alkuohje_mahtuu(&ALKU_OSAT, &ALKU_RIVIT, ALOITUS_Y);
}

#[test]
fn alkuohje_kertoo_oikeat_luvut() {
    kaikilla_kielilla(|| {
        let ohje = kaanna("holdem.ohje_2");
        assert!(ohje.contains(&ALKURAHA.to_string()), "{ohje}");
        assert!(ohje.contains("10/20"), "{ohje}");
    });
}

#[test]
fn napit_loytyvat_ja_valit_eivat_ole_kohteita() {
    for napisto in KAIKKI_NAPISTOT {
        for &nappi in napit(napisto) {
            let (x, y) = napin_keskipiste(napisto, nappi);
            assert_eq!(nappi_pisteesta(napisto, x, y), Some(nappi));
        }
    }
    let vali = NAPIT_Y + NAPPI_K;
    assert_eq!(nappi_pisteesta(Napisto::Vuoro, SIVU_TEKSTI_X, vali), None);
    assert_eq!(
        nappi_pisteesta(Napisto::Vuoro, SIVU_TEKSTI_X, NAPIT_Y - 1),
        None
    );
    assert_eq!(
        nappi_pisteesta(Napisto::Vuoro, SIVU_TEKSTI_X - 1, NAPIT_Y),
        None
    );
}

#[test]
fn koko_peli_piirtyy_kaatumatta() {
    let mut ruutu = vec![0; RUUDUN_LEVEYS * RUUDUN_KORKEUS];
    let mut klassinen = vec![0; LEVEYS * KORKEUS];
    for siemen in 0..3 {
        let mut peli = Holdem::new(siemen);
        piirra_alku(&peli, &mut klassinen);
        peli.kasittele(Syote::Ammu);
        for _ in 0..40 {
            piirra(&peli, &mut ruutu);
            peli.kasittele(Syote::Kirjain('R'));
            peli.edista(1000);
            piirra(&peli, &mut ruutu);
            peli.kasittele(Syote::Ammu);
            peli.edista(5000);
            if peli.vaihe != Vaihe::Pelissa {
                piirra(&peli, &mut ruutu);
                break;
            }
        }
        peli.vaihe = Vaihe::Nimi;
        peli.poyta.pelaajat[PELAAJA].rahat = 5 * ALKURAHA;
        piirra(&peli, &mut ruutu);
    }
}
