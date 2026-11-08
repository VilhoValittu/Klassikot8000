use super::*;
use crate::yhteinen::kieli::kaikilla_kielilla;
use crate::yhteinen::tulokset::NIMEN_PITUUS;

const KUVAUS_RIVEJA: usize = 3;

/// Painikkeen alin paikka, kun kuvaus vie kaikki rivit.
const PELAA_ALIN_Y: i32 = KUVAUS_Y + KUVAUS_RIVEJA as i32 * KUVAUS_RIVI - KUVAUS_VALI + PELAA_YLA;

fn sirun_keskipiste((vasen, leveys): (i32, i32)) -> (i32, i32) {
    (vasen + leveys / 2, SIRU_Y + SIRU_K / 2)
}

pub fn ilmoituksen_keskipiste(versio: &str) -> (i32, i32) {
    sirun_keskipiste(ilmoituksen_alue(versio))
}

pub fn aanikytkimen_keskipiste() -> (i32, i32) {
    sirun_keskipiste(aanikytkimen_alue())
}

pub fn kielikytkimen_keskipiste() -> (i32, i32) {
    sirun_keskipiste(kielikytkimen_alue())
}

pub fn pelaa_napin_keskipiste(valittu: usize) -> (i32, i32) {
    (
        ESITTELY_X + pelaa_napin_leveys() / 2,
        pelaa_y(valittu) + PELAA_K / 2,
    )
}

pub fn kortin_keskipiste(i: usize) -> (i32, i32) {
    (
        KORTTI_X + KORTTI_L / 2,
        KORTTI_Y + i as i32 * KORTTI_ASKEL + KORTTI_K / 2,
    )
}

/// Ohjepalkin näppäinten ja selitteiden leveys. Selitteistä lasketaan pidempi muoto, koska
/// lopetuksen selite pitenee ensimmäisestä Esc-painalluksesta.
fn ohjeiden_leveys() -> i32 {
    OHJEET
        .iter()
        .map(|(nappaimet, selite)| {
            let nappaimet: i32 = nappaimet
                .iter()
                .map(|nimi| nappaimen_leveys(nimi) + NAPPAINTEN_VALI)
                .sum();
            let selite = [false, true]
                .map(|kysyy| pieni_leveys(ohjeen_selite(selite, kysyy).0))
                .into_iter()
                .max()
                .unwrap_or(0);
            nappaimet - NAPPAINTEN_VALI + SELITTEEN_VALI + selite
        })
        .sum::<i32>()
        + (OHJEET.len() as i32 - 1) * RYHMIEN_VALI
}

/// Top 3 -laatikon sisällön korkeus otsikosta oman parhaan riviin.
fn listan_korkeus(laji: bool) -> i32 {
    (1 + laji as i32) * LAATIKON_OTSIKKO
        + PARHAITA as i32 * PARHAAT_RIVI
        + OMA_VALI
        + P
        + 2 * OMA_VALI
        + PIENI_MERKKI
}

const PISIN_VERSIO: &str = "100.100.100";

#[test]
fn osat_mahtuvat_kuvaan_eivatka_mene_paallekkain() {
    const { assert!(LISTAN_ALA <= ALAPALKKI_Y) };
    const { assert!(KORTTI_X + KORTTI_L + VALITUN_SIIRTO + VARJO < NUOLI_X - NUOLI_L / 2) };
    const { assert!(NUOLI_X + NUOLI_L / 2 < ESITTELY_X) };
    const { assert!(PELAA_ALIN_Y + PELAA_K + VARJO <= LAATIKKO_Y) };
    const { assert!(LAATIKKO_Y + LAATIKKO_K + VARJO <= ALAPALKKI_Y) };
    const { assert!(SIRU_Y + SIRU_K < KORTTI_Y) };
    const { assert!(LOGO_Y + 40 < KORTTI_Y) };
    // Lasipinnat kopioidaan taustan ruudukosta, joten niiden pitää osua siihen.
    for arvo in [KORTTI_X, KORTTI_L, KORTTI_Y, KORTTI_ASKEL, KORTTI_K] {
        assert_eq!(arvo % P, 0);
    }
    for arvo in [
        LAATIKKO_Y, LAATIKKO_K, LISTA_X, LISTA_L, TILASTO_X, TILASTO_L,
    ] {
        assert_eq!(arvo % P, 0);
    }
    assert_eq!(SIRU_Y % P, 0);
    assert_eq!(SIRU_K % P, 0);
    kaikilla_kielilla(|| {
        for alue in [aanikytkimen_alue(), kielikytkimen_alue()] {
            assert_eq!(alue.0 % P, 0);
            assert_eq!(alue.1 % P, 0);
        }
    });
}

#[test]
fn kytkimet_ja_ilmoitus_loytyvat_eivatka_mene_logon_tai_toistensa_paalle() {
    kaikilla_kielilla(|| {
        let (x, y) = kielikytkimen_keskipiste();
        assert!(kielikytkimella(x, y));
        assert!(!aanikytkimella(x, y));
        assert!(!ilmoituksella(x, y, PISIN_VERSIO));
        let (aani_x, aani_y) = aanikytkimen_keskipiste();
        assert!(aanikytkimella(aani_x, aani_y));
        assert!(!kielikytkimella(aani_x, aani_y));
        let (ix, iy) = ilmoituksen_keskipiste(PISIN_VERSIO);
        assert!(ilmoituksella(ix, iy, PISIN_VERSIO));
        assert!(!kielikytkimella(ix, iy));
        assert!(!ilmoituksella(ix, SIRU_Y - 1, PISIN_VERSIO));
        for (x, y) in [(x, y), (aani_x, aani_y), (ix, iy)] {
            assert_eq!(kortti_pisteesta(x, y), None);
            assert!((0..PELIT.len()).all(|i| !pelaa_napilla(x, y, i)));
        }
        let logo = kaanna("valikko.banneri").chars().count() as i32 * LOGO_KOKO.2;
        assert!(VASEN_REUNA + logo + SIRU_VALI <= ilmoituksen_alue(PISIN_VERSIO).0);
    });
}

#[test]
fn tekstit_mahtuvat_paikoilleen_kummallakin_kielella() {
    kaikilla_kielilla(|| {
        for kortti in &PELIT {
            let nimi = kaanna(kortti.nimi).to_uppercase();
            let leveys = nimi.chars().count() as i32 * NIMI_KOKO.2;
            assert!(
                NIMI_X + leveys <= KORTTI_L - KORTIN_NUOLI,
                "{}",
                kortti.nimi
            );
            let koko = nimen_koko(&nimi);
            assert!(nimi.chars().count() as i32 * koko.2 <= ESITTELY_L, "{nimi}");
            let rivit = rivita(kaanna(kortti.kuvaus), KUVAUS_MERKKEJA).len();
            assert!(rivit <= KUVAUS_RIVEJA, "{}", kortti.kuvaus);
        }
        assert!(pelaa_napin_leveys() <= ESITTELY_L);
        let versio = muotoile("valikko.versio", &[&PISIN_VERSIO]);
        let ohjeet = VASEN_REUNA + ohjeiden_leveys() + RYHMIEN_VALI;
        assert!(ohjeet <= OIKEA_REUNA - pieni_leveys(&versio));
        let jarjestys = muotoile("valikko.peli_nro", &[&PELIT.len(), &PELIT.len()]);
        assert!(pieni_leveys(&jarjestys) <= ESITTELY_L);
    });
}

#[test]
fn laatikoiden_sisalto_mahtuu_kummallakin_kielella() {
    let sisa = LAATIKKO_K - 2 * LAATIKKO_SISENNYS;
    assert!(listan_korkeus(true) <= sisa);
    assert!(4 * TAULUKON_RIVI + PIENI_MERKKI <= sisa);
    let listan_sisa = LISTA_L - 2 * LAATIKKO_SISENNYS;
    let tilaston_sisa = TILASTO_L - 2 * LAATIKKO_SISENNYS;
    kaikilla_kielilla(|| {
        // pisin nimi ja seitsennumeroiset pisteet samalle riville
        assert!((2 + NIMEN_PITUUS as i32 + 7) * LISTA_KOKO.2 <= listan_sisa);
        for netista in [false, true] {
            assert!(pieni_leveys(listan_otsikko(netista)) <= listan_sisa);
        }
        let oma = pieni_leveys(kaanna(OMA_PARAS)) + PIENI_MERKKI + pieni_leveys("1000000");
        assert!(oma <= listan_sisa);
        for variantti in klassikot8000::pasianssi::Variantti::KAIKKI {
            let laji = crate::pelit::pasianssi::piirto::lajin_nimi(variantti);
            assert!(pieni_leveys(laji) <= listan_sisa, "{laji}");
        }
        // Sarakkeiden väliin jää vähintään merkin verran tilaa, myös nelinumeroisille luvuille.
        let tasot = TASOJEN_NIMET.map(|avain| pieni_leveys(kaanna(avain)));
        let mut edellinen = *tasot.iter().max().unwrap();
        for (avain, oikea) in TAULUKON_SARAKKEET {
            let sarake = pieni_leveys(kaanna(avain)).max(pieni_leveys("9999"));
            assert!(edellinen + PIENI_MERKKI <= oikea - sarake, "{avain}");
            edellinen = oikea;
        }
        assert!(edellinen <= tilaston_sisa);
    });
}

#[test]
fn pelaa_nappi_loytyy_eika_osu_kortteihin() {
    kaikilla_kielilla(|| {
        for i in 0..PELIT.len() {
            let (x, y) = pelaa_napin_keskipiste(i);
            assert!(pelaa_napilla(x, y, i));
            assert_eq!(kortti_pisteesta(x, y), None);
            assert!(!pelaa_napilla(x, pelaa_y(i) - 1, i));
            assert!(!pelaa_napilla(ESITTELY_X - 1, y, i));
            assert!(pelaa_y(i) <= PELAA_ALIN_Y);
        }
    });
}

#[test]
fn jokainen_kortti_loytyy_ja_valit_eivat_ole_kohteita() {
    for i in 0..NAKYVIA_KORTTEJA as usize {
        let (x, y) = kortin_keskipiste(i);
        assert_eq!(kortti_pisteesta(x, y), Some(i));
    }
    assert_eq!(kortti_pisteesta(KORTTI_X, KORTTI_Y + KORTTI_K), None);
    assert_eq!(kortti_pisteesta(KORTTI_X, KORTTI_Y - 1), None);
    assert_eq!(kortti_pisteesta(KORTTI_X - 1, KORTTI_Y), None);
    assert_eq!(kortti_pisteesta(KORTTI_X, LISTAN_ALA + KORTTI_ASKEL), None);
}

#[test]
fn kuvakkeet_ovat_12x12_ja_tunnetuin_varein() {
    for kortti in &PELIT {
        let (_, kuva) = kuvake(kortti.laji);
        for rivi in kuva {
            assert_eq!(rivi.len(), 12, "{}", kortti.nimi);
            for merkki in rivi.bytes().filter(|&m| m != b'.') {
                assert!(kuvakkeen_vari(merkki).is_some(), "{}", kortti.nimi);
            }
        }
    }
}
