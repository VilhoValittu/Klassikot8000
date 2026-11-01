//! Blackjackin näkymät. Alkuruutu käyttää muiden pelien kanssa samaa klassista kuvaa,
//! pöytä piirretään suoraan täysikokoiselle ruudulle.

use klassikot8000::blackjack::{Kasi, Kortti, Tulos, Vuoro, YKSI, summa};

use super::peli::{Blackjack, Nappi, Vaihe};
use crate::yhteinen::kieli::{self, Kieli, kaanna, muotoile};
use crate::yhteinen::kortit::{KORTTI_K, KORTTI_L, pelimerkki};
use crate::yhteinen::piirto::*;
use crate::yhteinen::vektori::{Kangas, Tahko, kuutio};

const POYTA_X: i32 = 48;
const SIVU_X: i32 = 1314;
const POYTA_L: i32 = SIVU_X - 12 - POYTA_X;
const SIVU_L: i32 = RUUDUN_LEVEYS as i32 - 24 - SIVU_X;
const SIVU_TEKSTI_X: i32 = SIVU_X + 18;
const SIVU_MERKKEJA: usize = 22;

const JAKAJA_OTSIKKO_Y: i32 = 48;
const JAKAJA_Y: i32 = 78;
const SAANTO_Y: i32 = 300;
const KASI_OTSIKKO_Y: i32 = 336;
const KASI_TULOS_Y: i32 = 366;
const KASI_Y: i32 = 396;

const VIUHKA_ASKEL: i32 = 60;
// Tätä kapeammassa viuhkassa kortin arvo jäisi seuraavan alle, joten kortit pinotaan alaspäin.
const VIUHKA_PIENIN: i32 = 45;
const PINO_ASKEL: i32 = 36;
const KASI_MARGINAALI: i32 = 12;
// Korttien punainen on liian tumma tekstiksi pöydän pohjalla.
const TAPPIO_VARI: u32 = 0xF06060;

const RAHAT_Y: i32 = 54;
const PANOS_Y: i32 = RAHAT_Y + TILASTO_ASKEL;
const TILASTO_ASKEL: i32 = 66;
const VIESTI_Y: i32 = RAHAT_Y + 2 * TILASTO_ASKEL + 6;
const VIESTI_K: i32 = 4 * RIVIN_KORKEUS + 18;
const SIVU_OHJE_Y: i32 = VIESTI_Y + VIESTI_K + 18;
const NAPIT_Y: i32 = SIVU_OHJE_Y + 3 * RIVIN_KORKEUS + 18;
const NAPPI_ASKEL: i32 = NAPPI_K + 12;

const PANOSTUS_NAPIT: [(Nappi, &str); 8] = [
    (Nappi::Jako, "blackjack.nappi_jako"),
    (Nappi::Lisaa(0), "blackjack.lisaa_1"),
    (Nappi::Lisaa(1), "blackjack.lisaa_5"),
    (Nappi::Lisaa(2), "blackjack.lisaa_25"),
    (Nappi::Lisaa(3), "blackjack.lisaa_100"),
    (Nappi::Kaikki, "blackjack.nappi_kaikki"),
    (Nappi::Pienin, "blackjack.nappi_pienin"),
    (Nappi::Valikko, "yhteinen.valikkoon"),
];
const VAKUUTUS_NAPIT: [(Nappi, &str); 3] = [
    (Nappi::Vakuuta, "blackjack.nappi_vakuuta"),
    (Nappi::EiVakuutusta, "blackjack.nappi_ei_vakuutusta"),
    (Nappi::Valikko, "yhteinen.valikkoon"),
];
const PELI_NAPIT: [(Nappi, &str); 5] = [
    (Nappi::Ota, "blackjack.nappi_ota"),
    (Nappi::Jaa, "blackjack.nappi_jaa"),
    (Nappi::Tuplaa, "blackjack.nappi_tuplaa"),
    (Nappi::Halkaise, "blackjack.nappi_halkaise"),
    (Nappi::Valikko, "yhteinen.valikkoon"),
];
const LOPPU_NAPIT: [(Nappi, &str); 2] = [
    (Nappi::Uudestaan, "yhteinen.uudestaan"),
    (Nappi::Valikko, "yhteinen.valikkoon"),
];

const ALKU_OSAT: [&str; 3] = [
    "blackjack.tervetuloa",
    "blackjack.ohje_1",
    "blackjack.ohje_2",
];
const ALKU_RIVIT: [&str; 6] = [
    "blackjack.nappain_panos",
    "blackjack.nappain_jako",
    "blackjack.nappain_kortit",
    "blackjack.nappain_vakuutus",
    "yhteinen.aanet_kytkin",
    "yhteinen.takaisin",
];

/// Rahamäärä näytettäväksi. Puolikas näkyy vain, kun sellainen on.
pub fn raha(maara: u32) -> String {
    let kokonaiset = maara / YKSI;
    if maara.is_multiple_of(YKSI) {
        return kokonaiset.to_string();
    }
    let erotin = if kieli::nykyinen() == Kieli::Suomi {
        ','
    } else {
        '.'
    };
    format!("{kokonaiset}{erotin}5")
}

fn napit(vaihe: Vaihe, vuoro: Vuoro) -> &'static [(Nappi, &'static str)] {
    match (vaihe, vuoro) {
        (Vaihe::Pelissa, Vuoro::Panostus) => &PANOSTUS_NAPIT,
        (Vaihe::Pelissa, Vuoro::Vakuutus) => &VAKUUTUS_NAPIT,
        (Vaihe::Pelissa, _) => &PELI_NAPIT,
        _ => &LOPPU_NAPIT,
    }
}

pub fn nappi_pisteesta(vaihe: Vaihe, vuoro: Vuoro, x: i32, y: i32) -> Option<Nappi> {
    let i = (y - NAPIT_Y).div_euclid(NAPPI_ASKEL);
    let sisalla = (y - NAPIT_Y).rem_euclid(NAPPI_ASKEL) < NAPPI_K;
    let napit = napit(vaihe, vuoro);
    if !sisalla || !(0..napit.len() as i32).contains(&i) {
        return None;
    }
    let (nappi, teksti) = napit[i as usize];
    (SIVU_TEKSTI_X..SIVU_TEKSTI_X + napin_leveys(teksti))
        .contains(&x)
        .then_some(nappi)
}

#[cfg(test)]
pub fn napin_keskipiste(vaihe: Vaihe, vuoro: Vuoro, nappi: Nappi) -> (i32, i32) {
    let i = (napit(vaihe, vuoro).iter())
        .position(|n| n.0 == nappi)
        .unwrap() as i32;
    (SIVU_TEKSTI_X + 12, NAPIT_Y + i * NAPPI_ASKEL + NAPPI_K / 2)
}

/// Pöydän näkymässä äänikytkin on täyden ruudun oikeassa laidassa.
pub fn aanikytkimella(vaihe: Vaihe, x: i32, y: i32) -> bool {
    let siirto = if vaihe == Vaihe::Alku {
        0
    } else {
        (RUUDUN_LEVEYS - LEVEYS) as i32
    };
    crate::yhteinen::piirto::aanikytkimella(x - siirto, y)
}

/// Käden ensimmäisen kortin x sekä askel kortista seuraavaan. Kädet jakavat pöydän tasan.
fn kaden_asettelu(kasia: usize, kasi: usize, kortteja: usize) -> (i32, i32, i32) {
    let leveys = POYTA_L / kasia as i32;
    let sarake_x = POYTA_X + kasi as i32 * leveys;
    let vali = kortteja.saturating_sub(1) as i32;
    let viuhka = if vali == 0 {
        VIUHKA_ASKEL
    } else {
        ((leveys - KORTTI_L - KASI_MARGINAALI) / vali).min(VIUHKA_ASKEL)
    };
    if viuhka >= VIUHKA_PIENIN {
        let koko = KORTTI_L + vali * viuhka;
        (sarake_x + (leveys - koko) / 2, viuhka, 0)
    } else {
        (sarake_x + (leveys - KORTTI_L) / 2, 0, PINO_ASKEL)
    }
}

/// Pehmeä käsi näyttää molemmat arvot, koska ässän arvo vielä ratkaisee.
fn summateksti(kortit: &[Kortti]) -> String {
    match summa(kortit) {
        (arvo, true) if arvo < 21 => format!("{} / {arvo}", arvo - 10),
        (arvo, _) => arvo.to_string(),
    }
}

/// Käden alle tuleva rivi ja sen väri.
fn kaden_rivi(kasi: &Kasi) -> (String, u32) {
    let panos = raha(kasi.panos);
    if kasi.on_yli() {
        return (muotoile("blackjack.tulos_yli", &[&panos]), TAPPIO_VARI);
    }
    match kasi.tulos {
        None => (muotoile("blackjack.kasi_panos", &[&panos]), VALKOINEN),
        Some(Tulos::Blackjack) => {
            let voitto = raha(Tulos::Blackjack.palautus(kasi.panos) - kasi.panos);
            (muotoile("blackjack.tulos_blackjack", &[&voitto]), KELTAINEN)
        }
        Some(Tulos::Voitto) => (muotoile("blackjack.tulos_voitto", &[&panos]), KELTAINEN),
        Some(Tulos::Tasan) => (kaanna("blackjack.tulos_tasan").to_string(), VALKOINEN),
        Some(Tulos::Tappio) => (muotoile("blackjack.tulos_tappio", &[&panos]), TAPPIO_VARI),
    }
}

/// Alkuruudun malli: pelimerkkipinot ja kaksi pystyssä olevaa korttia.
pub fn pelimerkkipinot() -> Vec<Tahko> {
    let mut tahkot = Vec::new();
    for (x, z, varit) in [
        (-1.4, -1.6, &[OSUMA, VALKOINEN, OSUMA, VALKOINEN, OSUMA][..]),
        (1.4, -1.2, &[WB_SININEN, VALKOINEN, WB_SININEN][..]),
    ] {
        for (taso, &vari) in varit.iter().enumerate() {
            pelimerkki(x, z, taso, vari, &mut tahkot);
        }
    }
    kuutio(
        [-2.9, -0.5],
        [0.0, 3.4],
        [0.9, 1.02],
        VALKOINEN,
        &mut tahkot,
    );
    kuutio([0.0, 2.4], [0.0, 3.4], [1.5, 1.62], OSUMA, &mut tahkot);
    tahkot
}

/// Alkuruutu piirretään klassiseen kuvaan, jonka käynnistin skaalaa ruudulle.
pub fn piirra_alku(peli: &Blackjack, kuva: &mut [u32]) {
    let mut p = Piirtaja {
        k: Kangas::new(kuva),
        kello: peli.kello_ms,
    };
    p.maisema(kaanna("blackjack.banneri"), &pelimerkkipinot());
    let otsikko = listan_otsikko(peli.netti_parhaat.is_some());
    p.parhaat_alkuruudulla(otsikko, peli.nakyva_lista());
    p.alkuohje(&ALKU_OSAT, &ALKU_RIVIT);
    p.aloituskehote(ALOITUS);
    p.aanikytkin(peli.aanet_paalla);
}

pub fn piirra(peli: &Blackjack, ruutu: &mut [u32]) {
    let mut p = Piirtaja {
        k: Kangas::kokoinen(ruutu, RUUDUN_LEVEYS, RUUDUN_KORKEUS),
        kello: peli.kello_ms,
    };
    p.bj_poyta(peli);
    p.aanikytkin(peli.aanet_paalla);
}

impl Piirtaja<'_> {
    fn bj_poyta(&mut self, peli: &Blackjack) {
        self.pelitausta();
        self.otsikkopalkki(&kaanna("peli.blackjack").to_uppercase(), "");

        self.bj_jakaja(peli);
        self.bj_saanto(peli.vaihe);
        let kadet = &peli.poyta.kadet;
        if kadet.is_empty() {
            self.bj_tyhjat_paikat(1, KASI_Y);
        }
        for i in 0..kadet.len() {
            self.bj_kasi(peli, i);
        }
        self.bj_sivu(peli);
        if peli.vaihe == Vaihe::Nimi {
            self.nimi_ikkuna(peli.poyta.pisteet(), &peli.nimi);
        }
    }

    fn bj_tyhjat_paikat(&mut self, kasia: usize, y: i32) {
        let (x, askel, _) = kaden_asettelu(kasia, 0, 2);
        for i in 0..2 {
            let kortti_x = x + i * askel;
            self.korttipaikka(kortti_x, y);
        }
    }

    fn bj_jakaja(&mut self, peli: &Blackjack) {
        let poyta = &peli.poyta;
        let kortit = &poyta.jakaja;
        // Piilokortin arvo ei saa näkyä summasta.
        let nakyvat = if poyta.piilossa {
            &kortit[..1]
        } else {
            &kortit[..]
        };
        let otsikko = if kortit.is_empty() {
            kaanna("blackjack.jakaja").to_string()
        } else {
            format!("{} {}", kaanna("blackjack.jakaja"), summateksti(nakyvat))
        };
        let keski = POYTA_X + POYTA_L / 2;
        let x = keski - otsikko.chars().count() as i32 * OTSIKKO.2 / 2;
        self.suurteksti(x, JAKAJA_OTSIKKO_Y, &otsikko, OTSIKKO, KELTAINEN, true);

        if kortit.is_empty() {
            return self.bj_tyhjat_paikat(1, JAKAJA_Y);
        }
        let (x, askel, _) = kaden_asettelu(1, 0, kortit.len());
        for (i, &kortti) in kortit.iter().enumerate() {
            let kortti_x = x + i as i32 * askel;
            if poyta.piilossa && i == 1 {
                self.kortin_tausta(kortti_x, JAKAJA_Y);
            } else {
                self.pelikortti(kortti_x, JAKAJA_Y, kortti);
            }
        }
    }

    fn bj_saanto(&mut self, vaihe: Vaihe) {
        if vaihe == Vaihe::Loppu {
            let teksti = kaanna("blackjack.rahat_loppu");
            let leveys = teksti.chars().count() as i32 * OTSIKKO.2;
            let x = POYTA_X + (POYTA_L - leveys) / 2;
            self.ikkuna(x - 18, SAANTO_Y - 12, leveys + 36, 42);
            self.suurteksti(x, SAANTO_Y - 2, teksti, OTSIKKO, KELTAINEN, true);
        } else {
            let teksti = kaanna("blackjack.saanto");
            let x = POYTA_X + (POYTA_L - teksti.chars().count() as i32 * MERKKI) / 2;
            self.teksti(x, SAANTO_Y, teksti, VALKOINEN);
        }
    }

    fn bj_kasi(&mut self, peli: &Blackjack, i: usize) {
        let poyta = &peli.poyta;
        let kasi = &poyta.kadet[i];
        let (x, dx, dy) = kaden_asettelu(poyta.kadet.len(), i, kasi.kortit.len());
        let vali = kasi.kortit.len() as i32 - 1;

        let leveys = POYTA_L / poyta.kadet.len() as i32;
        let keski = POYTA_X + i as i32 * leveys + leveys / 2;
        let summa = summateksti(&kasi.kortit);
        let summa_x = keski - summa.chars().count() as i32 * OTSIKKO.2 / 2;
        self.suurteksti(summa_x, KASI_OTSIKKO_Y, &summa, OTSIKKO, VALKOINEN, true);
        let (rivi, vari) = kaden_rivi(kasi);
        let rivi_x = keski - rivi.chars().count() as i32 * TULOS.2 / 2;
        self.suurteksti(rivi_x, KASI_TULOS_Y, &rivi, TULOS, vari, false);

        for (j, &kortti) in kasi.kortit.iter().enumerate() {
            let j = j as i32;
            self.pelikortti(x + j * dx, KASI_Y + j * dy, kortti);
        }
        // Korostus kertoo, mitä kättä pelataan, kun käsiä on useampi.
        if poyta.vuoro == Vuoro::Pelaaja && poyta.vuorossa == i && poyta.kadet.len() > 1 {
            let (l, k) = (KORTTI_L + vali * dx, KORTTI_K + vali * dy);
            for sisennys in 0..3 {
                self.k.kehys(
                    x - sisennys,
                    KASI_Y - sisennys,
                    l + 2 * sisennys,
                    k + 2 * sisennys,
                    WB_ORANSSI,
                );
            }
        }
    }

    fn bj_sivu(&mut self, peli: &Blackjack) {
        let poyta = &peli.poyta;
        self.ikkuna(SIVU_X - 12, 42, SIVU_L + 24, RUUDUN_KORKEUS as i32 - 66);
        let panostus = peli.vaihe == Vaihe::Pelissa && poyta.vuoro == Vuoro::Panostus;
        // Pelatessa näkyy pöydän koko panos, tuplaukset, halkaisut ja vakuutus mukaan lukien.
        let panos = if panostus {
            peli.panos
        } else {
            poyta.vakuutus + poyta.kadet.iter().map(|kasi| kasi.panos).sum::<u32>()
        };
        // Voitto näkyy rahojen vieressä, koska se on poistuessa saatava tulos.
        let rahat = match peli.poyta.pisteet() {
            0 => raha(poyta.rahat),
            voitto => format!("{} (+{voitto})", raha(poyta.rahat)),
        };
        for (y, otsikko, arvo) in [
            (RAHAT_Y, "blackjack.rahat", rahat),
            (PANOS_Y, "blackjack.panos", raha(panos)),
        ] {
            self.suurteksti(SIVU_TEKSTI_X, y, kaanna(otsikko), OTSIKKO, KELTAINEN, false);
            self.suurteksti(SIVU_TEKSTI_X, y + 30, &arvo, OTSIKKO, VALKOINEN, true);
        }

        self.ikkuna(SIVU_X, VIESTI_Y, SIVU_L, VIESTI_K);
        let mut y = VIESTI_Y + 9;
        for rivi in rivita(&peli.viesti, SIVU_MERKKEJA) {
            self.teksti(SIVU_TEKSTI_X, y, rivi, VALKOINEN);
            y += RIVIN_KORKEUS;
        }

        let ohjeet: &[&str] = if panostus {
            &["blackjack.sivu_nuolet", "blackjack.sivu_aanet"]
        } else {
            &["blackjack.sivu_aanet"]
        };
        let mut y = SIVU_OHJE_Y;
        for rivi in ohjeet {
            self.teksti(SIVU_TEKSTI_X, y, kaanna(rivi), HUTI);
            y += RIVIN_KORKEUS;
        }

        if peli.vaihe == Vaihe::Nimi {
            return;
        }
        for (i, &(nappi, teksti)) in napit(peli.vaihe, poyta.vuoro).iter().enumerate() {
            let y = NAPIT_Y + i as i32 * NAPPI_ASKEL;
            let kaytossa = match nappi {
                Nappi::Ota | Nappi::Jaa => poyta.vuoro == Vuoro::Pelaaja,
                Nappi::Tuplaa => poyta.voi_tuplata(),
                Nappi::Halkaise => poyta.voi_halkaista(),
                _ => true,
            };
            let vari = if kaytossa { WB_SININEN } else { KANSI };
            self.nappi(SIVU_TEKSTI_X, y, teksti, VALKOINEN, vari);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yhteinen::kieli::kaikilla_kielilla;
    use crate::yhteinen::syote::Syote;
    use klassikot8000::blackjack::{ALKURAHA, ENITEN_KASIA, Maa, PIENIN_PANOS};

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
}
