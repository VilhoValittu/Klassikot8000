//! Spiderin näkymä. Kymmenen saraketta vie koko ruudun leveyden, joten pakka, valmiit
//! sarjat, tilastot ja napit ovat yläreunassa eivätkä sivupaneelissa.

use klassikot8000::kortit::{KUNINGAS, Kortti};
use klassikot8000::pasianssi::Sarake;
use klassikot8000::spider::{Jono, Lauta, SARAKKEITA, SARJOJA};

use crate::pelit::pasianssi::peli::{Nappi, Pasianssi, Vaihe};
use crate::pelit::pasianssi::piirto::kesto;
use crate::yhteinen::kieli::kaanna;
use crate::yhteinen::kortit::{KORTTI_K, KORTTI_L};
use crate::yhteinen::piirto::*;

const SARAKE_X: i32 = 48;
const SARAKE_ASKEL: i32 = 186;
const YLARIVI_Y: i32 = 54;
const TAULU_Y: i32 = YLARIVI_Y + KORTTI_K + 24;
const TAULU_ALA: i32 = RUUDUN_KORKEUS as i32 - 12;
const PIILO_ASKEL: i32 = 15;
const NAKYVA_ASKEL: i32 = 36;

const PAKKA_ASKEL: i32 = 12;
const PAKAN_LEVEYS: i32 = KORTTI_L + 4 * PAKKA_ASKEL;
const VALMIIT_X: i32 = 270;
const VALMIS_ASKEL: i32 = 66;

const PANEELI_X: i32 = 900;
const PANEELI_L: i32 = SARAKE_X + (SARAKKEITA as i32 - 1) * SARAKE_ASKEL + KORTTI_L - PANEELI_X;
const PANEELI_YLA: i32 = 48;
const PANEELI_KORKEUS: i32 = 222;
const TILASTO_X: i32 = PANEELI_X + 18;
const TILASTO_ASKEL: i32 = 150;
const TILASTOJA: i32 = 4;
const VIESTI_Y: i32 = YLARIVI_Y + 72;
const VIESTIN_MERKKEJA: usize = 24;
#[cfg(test)]
const VIESTIN_RIVEJA: usize = 5;
const NAPIT_X: i32 = TILASTO_X + TILASTOJA * TILASTO_ASKEL;
const NAPIT_Y: i32 = YLARIVI_Y + 6;
const NAPPI_ASKEL: i32 = NAPPI_K + 12;

const NAPIT: [(Nappi, &str); 4] = [
    (Nappi::Nosta, "spider.nappi_jaa"),
    (Nappi::Peruuta, "pasianssi.nappi_peruuta"),
    (Nappi::Uusi, "pasianssi.nappi_uusi"),
    (Nappi::Valikko, "yhteinen.valikkoon"),
];

/// Laudan paikka hiiren alla. `kortti` on `None` tyhjässä sarakkeessa.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kohta {
    Pakka,
    Sarake {
        sarake: usize,
        kortti: Option<usize>,
    },
}

fn sarakkeen_x(sarake: usize) -> i32 {
    SARAKE_X + sarake as i32 * SARAKE_ASKEL
}

/// Pitkä sarake tiivistyy niin, että se mahtuu aina ruudulle.
fn nakyva_askel(tiedot: &Sarake) -> i32 {
    let valeja = (tiedot.kortit.len() - tiedot.piilossa) as i32 - 1;
    if valeja <= 0 {
        return NAKYVA_ASKEL;
    }
    let tila = TAULU_ALA - TAULU_Y - KORTTI_K - tiedot.piilossa as i32 * PIILO_ASKEL;
    (tila / valeja).clamp(1, NAKYVA_ASKEL)
}

fn kortin_y(tiedot: &Sarake, i: usize) -> i32 {
    let piilossa = i.min(tiedot.piilossa) as i32;
    TAULU_Y + piilossa * PIILO_ASKEL + (i as i32 - piilossa) * nakyva_askel(tiedot)
}

/// Jonon alimman kortin vasen yläkulma.
pub fn kortin_paikka(lauta: &Lauta, jono: Jono) -> (i32, i32) {
    (
        sarakkeen_x(jono.sarake),
        kortin_y(&lauta.tila.sarakkeet[jono.sarake], jono.kortti),
    )
}

fn sarake_pisteesta(x: i32) -> Option<usize> {
    let sarake = (x - SARAKE_X).div_euclid(SARAKE_ASKEL);
    let sisalla = (x - SARAKE_X).rem_euclid(SARAKE_ASKEL) < KORTTI_L;
    (sisalla && (0..SARAKKEITA as i32).contains(&sarake)).then_some(sarake as usize)
}

pub fn kohta_pisteesta(lauta: &Lauta, x: i32, y: i32) -> Option<Kohta> {
    if (YLARIVI_Y..YLARIVI_Y + KORTTI_K).contains(&y) {
        let pakalla = (SARAKE_X..SARAKE_X + PAKAN_LEVEYS).contains(&x);
        return pakalla.then_some(Kohta::Pakka);
    }
    let sarake = sarake_pisteesta(x)?;
    let tiedot = &lauta.tila.sarakkeet[sarake];
    let kortti = (0..tiedot.kortit.len()).rev().find(|&i| {
        let kortti_y = kortin_y(tiedot, i);
        (kortti_y..kortti_y + KORTTI_K).contains(&y)
    });
    let tyhja_kohta = tiedot.kortit.is_empty() && (TAULU_Y..TAULU_Y + KORTTI_K).contains(&y);
    (kortti.is_some() || tyhja_kohta).then_some(Kohta::Sarake { sarake, kortti })
}

/// Pudotuskohde on anteliaampi kuin poiminta: koko sarakkeen kaista kelpaa.
pub fn kohde_pisteesta(x: i32, y: i32) -> Option<usize> {
    sarake_pisteesta(x).filter(|_| y >= TAULU_Y)
}

/// Vedettävän jonon keskikohta, kun osoitin on jäänyt sarakkeiden väliin.
pub fn vedetyn_keskipiste(x: i32) -> i32 {
    x + KORTTI_L / 2
}

pub fn nappi_pisteesta(x: i32, y: i32) -> Option<Nappi> {
    let i = (y - NAPIT_Y).div_euclid(NAPPI_ASKEL);
    let sisalla = (y - NAPIT_Y).rem_euclid(NAPPI_ASKEL) < NAPPI_K;
    if !sisalla || !(0..NAPIT.len() as i32).contains(&i) {
        return None;
    }
    let (nappi, teksti) = NAPIT[i as usize];
    (NAPIT_X..NAPIT_X + napin_leveys(teksti))
        .contains(&x)
        .then_some(nappi)
}

#[cfg(test)]
pub fn napin_keskipiste(nappi: Nappi) -> (i32, i32) {
    let i = NAPIT.iter().position(|n| n.0 == nappi).unwrap() as i32;
    (NAPIT_X + 12, NAPIT_Y + i * NAPPI_ASKEL + NAPPI_K / 2)
}

#[cfg(test)]
pub fn pakan_keskipiste() -> (i32, i32) {
    (SARAKE_X + KORTTI_L / 2, YLARIVI_Y + KORTTI_K / 2)
}

impl Piirtaja<'_> {
    pub fn spider_lauta(&mut self, peli: &Pasianssi) {
        self.spider_ylarivi(peli);
        self.spider_paneeli(peli);
        for sarake in 0..SARAKKEITA {
            self.spider_sarake(peli, sarake);
        }
        self.spider_kursori(peli);
        if let Some(pito) = peli.spider_pito.filter(|pito| pito.liikkui)
            && let Some(kortit) = peli.spider.siirrettavat(pito.lahde)
        {
            let x = peli.hiiri.0 - pito.tartunta.0;
            let y = peli.hiiri.1 - pito.tartunta.1;
            for (i, &kortti) in kortit.iter().enumerate() {
                self.pelikortti(x, y + i as i32 * NAKYVA_ASKEL, kortti);
            }
        }
    }

    fn spider_ylarivi(&mut self, peli: &Pasianssi) {
        let jakoja = peli.spider.jakoja() as i32;
        if jakoja == 0 {
            self.korttipaikka(SARAKE_X, YLARIVI_Y);
        }
        for i in 0..jakoja {
            self.kortin_tausta(SARAKE_X + i * PAKKA_ASKEL, YLARIVI_Y);
        }
        for i in 0..SARJOJA {
            let x = VALMIIT_X + i as i32 * VALMIS_ASKEL;
            match peli.spider.tila.valmiit.get(i) {
                Some(&maa) => self.pelikortti(
                    x,
                    YLARIVI_Y,
                    Kortti {
                        maa,
                        arvo: KUNINGAS,
                    },
                ),
                None => self.korttipaikka(x, YLARIVI_Y),
            }
        }
    }

    fn spider_paneeli(&mut self, peli: &Pasianssi) {
        let tila = &peli.spider.tila;
        self.ikkuna(PANEELI_X, PANEELI_YLA, PANEELI_L, PANEELI_KORKEUS);
        for (i, (otsikko, arvo)) in [
            (kaanna("yhteinen.pisteet"), peli.pisteet().to_string()),
            (kaanna("yhteinen.siirrot"), tila.siirrot.to_string()),
            (kaanna("pasianssi.aika"), kesto(peli.peliaika_ms)),
            (kaanna("spider.jakoja"), peli.spider.jakoja().to_string()),
        ]
        .into_iter()
        .enumerate()
        {
            let x = TILASTO_X + i as i32 * TILASTO_ASKEL;
            self.suurteksti(x, YLARIVI_Y + 6, otsikko, TULOS, KELTAINEN, false);
            self.suurteksti(x, YLARIVI_Y + 33, &arvo, OTSIKKO, VALKOINEN, true);
        }
        let mut y = VIESTI_Y;
        for rivi in rivita(&peli.viesti, VIESTIN_MERKKEJA) {
            self.teksti(TILASTO_X, y, rivi, VALKOINEN);
            y += RIVIN_KORKEUS;
        }
        for (i, (nappi, teksti)) in NAPIT.into_iter().enumerate() {
            let y = NAPIT_Y + i as i32 * NAPPI_ASKEL;
            let kaytossa = peli.vaihe == Vaihe::Pelissa
                || (peli.vaihe != Vaihe::Nimi && matches!(nappi, Nappi::Uusi | Nappi::Valikko));
            let vari = if kaytossa { WB_SININEN } else { KANSI };
            self.nappi(NAPIT_X, y, teksti, VALKOINEN, vari);
        }
    }

    fn spider_sarake(&mut self, peli: &Pasianssi, sarake: usize) {
        let x = sarakkeen_x(sarake);
        let tiedot = &peli.spider.tila.sarakkeet[sarake];
        if tiedot.kortit.is_empty() {
            self.korttipaikka(x, TAULU_Y);
            return;
        }
        // Vedettävä jono jää piirtämättä: se kulkee osoittimen mukana.
        let piirrettavia = match peli.spider_pito {
            Some(pito) if pito.liikkui && pito.lahde.sarake == sarake => pito.lahde.kortti,
            _ => tiedot.kortit.len(),
        };
        for (i, &kortti) in tiedot.kortit[..piirrettavia].iter().enumerate() {
            let y = kortin_y(tiedot, i);
            if i < tiedot.piilossa {
                self.kortin_tausta(x, y);
            } else {
                self.pelikortti(x, y, kortti);
            }
        }
        if let Some(valittu) = peli.spider_valittu
            && valittu.sarake == sarake
            && valittu.kortti < tiedot.kortit.len()
        {
            let y = kortin_y(tiedot, valittu.kortti);
            let korkeus = kortin_y(tiedot, tiedot.kortit.len() - 1) + KORTTI_K - y;
            self.korostus(x, y, korkeus, WB_ORANSSI);
        }
    }

    fn spider_kursori(&mut self, peli: &Pasianssi) {
        if !peli.kursori_nakyy || peli.vaihe != Vaihe::Pelissa || !self.vilkku() {
            return;
        }
        let Kohta::Sarake { sarake, kortti } = peli.spider_kursorin_kohta() else {
            return;
        };
        let tiedot = &peli.spider.tila.sarakkeet[sarake];
        let y = kortin_y(tiedot, kortti.unwrap_or(0));
        let viimeinen = tiedot.kortit.len().saturating_sub(1);
        let korkeus = kortin_y(tiedot, viimeinen) + KORTTI_K - y;
        self.korostus(sarakkeen_x(sarake), y, korkeus, KELTAINEN);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yhteinen::kieli::kaikilla_kielilla;
    use klassikot8000::Arpa;
    use klassikot8000::kortit::Maa;
    use klassikot8000::pasianssi::Vaikeus;

    #[test]
    fn kaikki_mahtuu_ruudulle_ja_pitkakin_sarake_tiivistyy() {
        assert!(sarakkeen_x(SARAKKEITA - 1) + KORTTI_L < RUUDUN_LEVEYS as i32);
        const { assert!(SARAKE_X + PAKAN_LEVEYS < VALMIIT_X) };
        assert!(VALMIIT_X + (SARJOJA as i32 - 1) * VALMIS_ASKEL + KORTTI_L < PANEELI_X);
        const { assert!(PANEELI_YLA + PANEELI_KORKEUS <= TAULU_Y) };
        assert!(NAPIT_Y + NAPIT.len() as i32 * NAPPI_ASKEL <= PANEELI_YLA + PANEELI_KORKEUS);
        assert!(VIESTI_Y + VIESTIN_RIVEJA as i32 * RIVIN_KORKEUS <= PANEELI_YLA + PANEELI_KORKEUS);

        for (piilossa, nakyvia) in [(0, 1), (5, 13), (5, 40), (0, 104), (5, 99)] {
            let tiedot = Sarake {
                kortit: vec![
                    Kortti {
                        maa: Maa::Pata,
                        arvo: 1
                    };
                    piilossa + nakyvia
                ],
                piilossa,
            };
            let alin = kortin_y(&tiedot, tiedot.kortit.len() - 1) + KORTTI_K;
            assert!(alin <= TAULU_ALA, "{piilossa} {nakyvia}: {alin}");
        }
    }

    #[test]
    fn napit_ja_tekstit_mahtuvat_paneeliin() {
        kaikilla_kielilla(|| {
            for (_, teksti) in NAPIT {
                assert!(
                    NAPIT_X + napin_leveys(teksti) <= PANEELI_X + PANEELI_L - 12,
                    "{teksti}"
                );
            }
            for avain in ["yhteinen.pisteet", "yhteinen.siirrot", "spider.jakoja"] {
                assert!(kaanna(avain).chars().count() as i32 * TULOS.2 < TILASTO_ASKEL);
            }
            for avain in [
                "spider.ei_jakoa",
                "pasianssi.ei_kay",
                "pasianssi.pakka_loppu",
                "pasianssi.ei_peruttavaa",
                "pasianssi.jumissa",
                "pasianssi.voittoviesti",
                "pasianssi.siirrot_loppu",
            ] {
                let rivit = rivita(kaanna(avain), VIESTIN_MERKKEJA).len();
                assert!(rivit <= VIESTIN_RIVEJA, "{avain}");
            }
        });
        assert!(TILASTO_X + VIESTIN_MERKKEJA as i32 * MERKKI <= NAPIT_X);
    }

    #[test]
    fn kohdat_ja_napit_tunnistetaan() {
        let lauta = Lauta::new(Vaikeus::Helppo, &mut Arpa::new(1));
        let (x, y) = pakan_keskipiste();
        assert_eq!(kohta_pisteesta(&lauta, x, y), Some(Kohta::Pakka));
        assert_eq!(kohta_pisteesta(&lauta, VALMIIT_X + 5, y), None);
        for sarake in 0..SARAKKEITA {
            let tiedot = &lauta.tila.sarakkeet[sarake];
            let viimeinen = tiedot.kortit.len() - 1;
            let y = kortin_y(tiedot, viimeinen) + KORTTI_K - 1;
            assert_eq!(
                kohta_pisteesta(&lauta, sarakkeen_x(sarake) + 5, y),
                Some(Kohta::Sarake {
                    sarake,
                    kortti: Some(viimeinen)
                })
            );
        }
        for (nappi, _) in NAPIT {
            let (x, y) = napin_keskipiste(nappi);
            assert_eq!(nappi_pisteesta(x, y), Some(nappi));
        }
        assert_eq!(kohde_pisteesta(sarakkeen_x(9) + 1, TAULU_Y), Some(9));
        assert_eq!(kohde_pisteesta(sarakkeen_x(9) + 1, TAULU_Y - 1), None);
    }
}
