//! 2048-pelin näkymät.

use klassikot8000::luvut::{Lukulauta, Paikka, SIVU, Suunta, TAVOITE};

use super::peli::{KIPINA_MS, LIUKU_MS, Luvut, Nappi, Vaihe};
use crate::yhteinen::kieli::{kaanna, muotoile};
use crate::yhteinen::piirto::*;
use crate::yhteinen::vektori::{Kangas, Tahko, kuutio};

const POMPAHDUS_MS: u32 = 140;

const RUUTU: i32 = 132;
const LAATTA: i32 = RUUTU - 12;
const LAUDAN_SIVU: i32 = RUUTU * SIVU as i32;
const RUUDUT_X: i32 = 30;
const RUUDUT_Y: i32 = 48;
const SARAKE_X: i32 = RUUDUT_X + LAUDAN_SIVU + 30;
const TILASTON_ASKEL: i32 = 87;
const TAVOITE_Y: i32 = RUUDUT_Y + 4 * TILASTON_ASKEL;
const ASKELIA_TAVOITTEESEEN: i32 = 11;
const NUOLEN_KOKO: i32 = 15;
const PARHAAT_Y: i32 = TAVOITE_Y + 66;
const PARHAAT_L: i32 = LEVEYS as i32 - 12 - SARAKE_X;

const OHJE: &str = "luvut.ohje";
const EI_LIIKU: &str = "luvut.ei_liiku";
const UUSI_PELI: &str = "luvut.uusi_peli";
const UUSI_PELI_NAPPI: &str = "luvut.uusi_nappi";
const UUSI_PELI_Y: i32 = ALOITUS_Y - NAPPI_K - 6;
const JATKA: &str = "luvut.jatka";
const VOITTO: &str = "luvut.voitto";
const TAPPIO: &str = "yhteinen.peli_paattyi";

const NAPIT: [(Nappi, &str, i32); 3] = [
    (Nappi::Uudestaan, "yhteinen.uudestaan", NAPPI_X),
    (
        Nappi::Valikko,
        "yhteinen.valikkoon",
        NAPPI_X + 13 * MERKKI + 48,
    ),
    (
        Nappi::Jatka,
        "luvut.jatka_nappi",
        NAPPI_X + 24 * MERKKI + 84,
    ),
];

const ALKU_OSAT: [&str; 5] = [
    "luvut.tervetuloa",
    "yhteinen.ohjeet",
    "luvut.ohje_1",
    "luvut.ohje_2",
    "luvut.ohje_3",
];
const ALKU_RIVIT: [&str; 4] = [
    "luvut.nappain_nuolet",
    "luvut.nappain_hiiri",
    "yhteinen.aanet_kytkin",
    "yhteinen.takaisin",
];

/// Laudan pikseli kertoo suunnan: se puoli keskipisteestä, jota kohti on pisin matka.
pub fn suunta_pisteesta(x: i32, y: i32) -> Option<Suunta> {
    let sisalla = (RUUDUT_X..RUUDUT_X + LAUDAN_SIVU).contains(&x)
        && (RUUDUT_Y..RUUDUT_Y + LAUDAN_SIVU).contains(&y);
    if !sisalla {
        return None;
    }
    let (dx, dy) = (
        x - RUUDUT_X - LAUDAN_SIVU / 2,
        y - RUUDUT_Y - LAUDAN_SIVU / 2,
    );
    Some(if dx.abs() >= dy.abs() {
        if dx > 0 { Suunta::Oikea } else { Suunta::Vasen }
    } else if dy > 0 {
        Suunta::Alas
    } else {
        Suunta::Ylos
    })
}

pub fn nappi_pisteesta(x: i32, y: i32) -> Option<Nappi> {
    NAPIT
        .iter()
        .find(|&&(_, teksti, nx)| {
            (nx..nx + napin_leveys(teksti)).contains(&x)
                && (NAPPI_Y..NAPPI_Y + NAPPI_K).contains(&y)
        })
        .map(|&(nappi, ..)| nappi)
}

#[cfg(test)]
pub fn napin_keskipiste(nappi: Nappi) -> (i32, i32) {
    let (_, teksti, x) = NAPIT.iter().find(|n| n.0 == nappi).copied().unwrap();
    (x + napin_leveys(teksti) / 2, NAPPI_Y + NAPPI_K / 2)
}

/// Tallennetun pelin rinnalla oleva nappi on alkuruudun oikeassa laidassa.
fn uuden_pelin_x() -> i32 {
    LEVEYS as i32 - 12 - napin_leveys(UUSI_PELI_NAPPI)
}

pub fn uusi_peli_pisteessa(x: i32, y: i32) -> bool {
    let leveys = napin_leveys(UUSI_PELI_NAPPI);
    (uuden_pelin_x()..uuden_pelin_x() + leveys).contains(&x)
        && (UUSI_PELI_Y..UUSI_PELI_Y + NAPPI_K).contains(&y)
}

#[cfg(test)]
pub fn uuden_pelin_keskipiste() -> (i32, i32) {
    (
        uuden_pelin_x() + napin_leveys(UUSI_PELI_NAPPI) / 2,
        UUSI_PELI_Y + NAPPI_K / 2,
    )
}

/// Piste laudan reunan keskellä: sinne piirretään myös hiiren suuntanuoli.
pub fn laudan_reuna(suunta: Suunta) -> (i32, i32) {
    let (kx, ky) = (RUUDUT_X + LAUDAN_SIVU / 2, RUUDUT_Y + LAUDAN_SIVU / 2);
    let matka = LAUDAN_SIVU / 2 - NUOLEN_KOKO;
    match suunta {
        Suunta::Ylos => (kx, ky - matka),
        Suunta::Alas => (kx, ky + matka),
        Suunta::Vasen => (kx - matka, ky),
        Suunta::Oikea => (kx + matka, ky),
    }
}

/// Laatan väri: kylläiset sävyt sinisestä kultaan.
fn laatan_vari(arvo: u32) -> u32 {
    match arvo {
        2 => 0x7468E0,
        4 => 0x3898E8,
        8 => 0x28B8B8,
        16 => 0x48B048,
        32 => 0xD8B820,
        64 => 0xF08010,
        128 => TAIVAS_ALA,
        256 => OSUMA,
        512 => 0x8B3FC0,
        1024 => WB_SININEN,
        2048 => 0xFFC800,
        _ => 0x302848,
    }
}

/// Merkin koko numeron pituuden mukaan: (leveys, korkeus, askel).
fn numeron_koko(pituus: usize) -> (i32, i32, i32) {
    match pituus {
        0..=2 => (33, 54, 42),
        3 => (27, 42, 33),
        4 => (21, 33, 27),
        5 => (16, 27, 21),
        _ => (13, 21, 18),
    }
}

fn ruudun_keskipiste((rivi, sarake): Paikka) -> (i32, i32) {
    (
        RUUDUT_X + sarake as i32 * RUUTU + RUUTU / 2,
        RUUDUT_Y + rivi as i32 * RUUTU + RUUTU / 2,
    )
}

/// Montako tuplausta luvusta 2 tarvitaan. Palkki täyttyy samassa tahdissa.
fn tuplauksia(arvo: u32) -> i32 {
    arvo.max(1).trailing_zeros() as i32
}

/// Alkuruudun malli: neljä laattaa kasvavina pinoina. Mitat kuin laivalla.
pub fn lukupino() -> Vec<Tahko> {
    let varit = [2, 8, 64, 2048].map(laatan_vari);
    let mut tahkot = Vec::new();
    for (i, vari) in varit.into_iter().enumerate() {
        let x = -4.6 + i as f32 * 2.4;
        let korkeus = 0.8 + i as f32 * 0.8;
        kuutio([x, x + 2.0], [0.0, korkeus], [-1.0, 1.0], vari, &mut tahkot);
    }
    tahkot
}

pub fn piirra(luvut: &Luvut, kuva: &mut [u32]) {
    let mut p = Piirtaja {
        k: Kangas::new(kuva),
        kello: luvut.kello_ms,
    };
    if luvut.vaihe == Vaihe::Alku {
        p.luvut_alkuruutu(luvut);
    } else {
        p.luvut_nakyma(luvut);
    }
    p.aanikytkin(luvut.aanet_paalla);
}

impl Piirtaja<'_> {
    fn luvut_alkuruutu(&mut self, luvut: &Luvut) {
        let jatkuu = luvut.jatkuu;
        self.maisema(kaanna("luvut.banneri"), &lukupino());
        let otsikko = listan_otsikko(luvut.netti_parhaat.is_some());
        self.parhaat_alkuruudulla(otsikko, luvut.nakyva_lista());
        let y = self.alkuohje(&ALKU_OSAT, &ALKU_RIVIT);
        if jatkuu {
            self.teksti(48, y, kaanna(UUSI_PELI), KELTAINEN);
            let x = uuden_pelin_x();
            self.nappi(x, UUSI_PELI_Y, UUSI_PELI_NAPPI, VALKOINEN, WB_SININEN);
        }
        self.aloituskehote(if jatkuu { JATKA } else { ALOITUS });
    }

    fn luvut_nakyma(&mut self, luvut: &Luvut) {
        self.pelitausta();
        self.otsikkopalkki(kaanna("peli.luvut"), "");

        self.luku_lauta();
        self.laatat(luvut);
        if luvut.vaihe == Vaihe::Pelissa
            && let Some(suunta) = luvut.hiiren_suunta
        {
            self.suuntanuoli(suunta);
        }
        self.luku_tilasto(luvut);
        self.luku_paneeli(luvut);
    }

    /// Lauta on tummaa liukuväriä, ja jokaisella ruudulla on oma kuoppansa.
    fn luku_lauta(&mut self) {
        self.k.liukulaatikko(
            RUUDUT_X,
            RUUDUT_Y,
            LAUDAN_SIVU,
            LAUDAN_SIVU,
            0x1C2470,
            0x0C1040,
        );
        self.upotus(RUUDUT_X, RUUDUT_Y, LAUDAN_SIVU, LAUDAN_SIVU);
        // Jokaisella ruudulla on oma kuoppansa, johon laatta asettuu.
        for rivi in 0..SIVU as i32 {
            for sarake in 0..SIVU as i32 {
                let x = RUUDUT_X + sarake * RUUTU + 6;
                let y = RUUDUT_Y + rivi * RUUTU + 6;
                self.k.laatikko(x, y, LAATTA, LAATTA, 0x080C30);
                self.k.laatikko(x, y + LAATTA - 3, LAATTA, 3, 0x2C3890);
                self.k.laatikko(x + LAATTA - 3, y, 3, LAATTA, 0x2C3890);
            }
        }
    }

    fn laatat(&mut self, luvut: &Luvut) {
        let kulunut = luvut
            .viime_siirto
            .as_ref()
            .map(|(_, alku)| self.kello.saturating_sub(*alku));
        if let (Some((siirto, _)), Some(kulunut)) = (&luvut.viime_siirto, kulunut)
            && kulunut < LIUKU_MS
        {
            let t = kulunut as f32 / LIUKU_MS as f32;
            for liike in &siirto.liikkeet {
                let (ax, ay) = ruudun_keskipiste(liike.mista);
                let (bx, by) = ruudun_keskipiste(liike.mihin);
                let keski = (
                    ax + ((bx - ax) as f32 * t) as i32,
                    ay + ((by - ay) as f32 * t) as i32,
                );
                self.laatta(keski, LAATTA, liike.arvo);
            }
            return;
        }

        let pompahdus = kulunut.map_or(1.0, |kulunut| {
            ((kulunut.saturating_sub(LIUKU_MS)) as f32 / POMPAHDUS_MS as f32).min(1.0)
        });
        for rivi in 0..SIVU {
            for sarake in 0..SIVU {
                let paikka = (rivi, sarake);
                let arvo = luvut.lauta.arvo(paikka);
                if arvo == 0 {
                    continue;
                }
                let siirto = luvut.viime_siirto.as_ref().map(|(siirto, _)| siirto);
                // uusi luku kasvaa esiin, yhdistynyt paisuu ja palautuu
                let kerroin = match siirto {
                    Some(s) if s.uusi == paikka => 0.3 + 0.7 * pompahdus,
                    Some(s) if s.yhdistyneet.contains(&paikka) => 1.0 + 0.12 * (1.0 - pompahdus),
                    _ => 1.0,
                };
                let sivu = (LAATTA as f32 * kerroin) as i32;
                self.laatta(ruudun_keskipiste(paikka), sivu, arvo);
            }
        }
        if let (Some((siirto, _)), Some(kulunut)) = (&luvut.viime_siirto, kulunut) {
            let ika = (kulunut - LIUKU_MS) as f32 / KIPINA_MS as f32;
            for &paikka in &siirto.yhdistyneet {
                let arvo = luvut.lauta.arvo(paikka);
                let koko = if arvo >= 128 { 105.0 } else { 66.0 };
                self.rajahdys(ruudun_keskipiste(paikka), ika, 12, koko);
            }
        }
    }

    /// Kupera, kiiltävä laatta. `sivu` muuttuu animaatioissa.
    fn laatta(&mut self, keski: (i32, i32), sivu: i32, arvo: u32) {
        let (x, y) = (keski.0 - sivu / 2, keski.1 - sivu / 2);
        self.k.laatikko(x + 6, y + 6, sivu - 3, sivu - 3, 0x04061C);
        self.nappitausta(x, y, sivu, sivu, laatan_vari(arvo));

        let teksti = arvo.to_string();
        let pituus = teksti.chars().count();
        let (l, k, askel) = numeron_koko(pituus);
        let mittaa = |arvo: i32| arvo * sivu / LAATTA;
        let koko = (mittaa(l), mittaa(k), mittaa(askel));
        let leveys = pituus as i32 * koko.2 - (koko.2 - koko.0);
        let (tx, ty) = (keski.0 - leveys / 2, keski.1 - koko.1 / 2);
        self.suurteksti(tx, ty, &teksti, koko, VALKOINEN, true);
    }

    fn suuntanuoli(&mut self, suunta: Suunta) {
        let (x, y) = laudan_reuna(suunta);
        let vari = if self.vilkku() { KELTAINEN } else { WB_ORANSSI };
        let n = NUOLEN_KOKO;
        let kolmio = match suunta {
            Suunta::Ylos => [(x, y - n), (x + n, y + n), (x - n, y + n)],
            Suunta::Alas => [(x, y + n), (x + n, y - n), (x - n, y - n)],
            Suunta::Vasen => [(x - n, y), (x + n, y - n), (x + n, y + n)],
            Suunta::Oikea => [(x + n, y), (x - n, y - n), (x - n, y + n)],
        };
        self.k.monikulmio(&kolmio, vari);
        for (i, &a) in kolmio.iter().enumerate() {
            self.k.viiva(a, kolmio[(i + 1) % kolmio.len()], MUSTA);
        }
    }

    fn luku_tilasto(&mut self, luvut: &Luvut) {
        let lauta: &Lukulauta = &luvut.lauta;
        for (i, (otsikko, arvo)) in [
            (kaanna("yhteinen.pisteet"), lauta.pisteet),
            (kaanna("yhteinen.paras"), luvut.paras),
            (kaanna("luvut.suurin"), lauta.suurin()),
            (kaanna("yhteinen.siirrot"), luvut.siirtoja),
        ]
        .into_iter()
        .enumerate()
        {
            let y = RUUDUT_Y + i as i32 * TILASTON_ASKEL;
            self.suurteksti(SARAKE_X, y, otsikko, OTSIKKO, KELTAINEN, false);
            let numero = arvo.to_string();
            self.suurteksti(SARAKE_X, y + 30, &numero, OTSIKKO, VALKOINEN, true);
        }

        // Palkki kertoo matkan tavoitteeseen samoilla väreillä kuin laatat.
        let otsikko = muotoile("luvut.tavoite", &[&TAVOITE]);
        self.teksti(SARAKE_X, TAVOITE_Y, &otsikko, KELTAINEN);
        let saavutettu = tuplauksia(lauta.suurin());
        for i in 1..=ASKELIA_TAVOITTEESEEN {
            let x = SARAKE_X + (i - 1) * PALKIN_PALA;
            let vari = if i <= saavutettu {
                laatan_vari(1 << i)
            } else {
                KANSI
            };
            let (y, korkeus) = if i <= saavutettu {
                (TAVOITE_Y + 30, 24)
            } else {
                (TAVOITE_Y + 42, 12)
            };
            self.pala(x, y, PALKIN_PALA - 3, korkeus, vari);
        }
        self.parhaat_lista(
            SARAKE_X,
            PARHAAT_Y,
            PARHAAT_L,
            TULOS,
            listan_otsikko(luvut.netti_parhaat.is_some()),
            luvut.nakyva_lista(),
        );
    }

    fn luku_paneeli(&mut self, luvut: &Luvut) {
        self.paneelin_tausta();
        self.teksti(TEKSTI_X, PANEELI_RIVI_Y, &luvut.viesti, VALKOINEN);

        let alarivi = PANEELI_RIVI_Y + (PANEELI_RIVEJA as i32 - 1) * RIVIN_KORKEUS;
        let otsikko_y = alarivi - RIVIN_KORKEUS - 12;
        match luvut.vaihe {
            Vaihe::Pelissa if luvut.ei_liiku => {
                self.teksti(TEKSTI_X, alarivi, kaanna(EI_LIIKU), WB_ORANSSI);
            }
            Vaihe::Pelissa => self.teksti(TEKSTI_X, alarivi, kaanna(OHJE), HUTI),
            Vaihe::Voitto => {
                self.suurteksti(TEKSTI_X, otsikko_y, kaanna(VOITTO), TULOS, KELTAINEN, true);
                for (_, teksti, x) in NAPIT {
                    self.nappi(x, NAPPI_Y, teksti, VALKOINEN, WB_SININEN);
                }
            }
            Vaihe::Tappio => {
                self.suurteksti(TEKSTI_X, otsikko_y, kaanna(TAPPIO), TULOS, KELTAINEN, true);
                for (_, teksti, x) in &NAPIT[..2] {
                    self.nappi(*x, NAPPI_Y, teksti, VALKOINEN, WB_SININEN);
                }
            }
            Vaihe::Nimi => self.nimikysely(luvut.lauta.pisteet, &luvut.nimi),
            Vaihe::Alku => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yhteinen::kieli::kaikilla_kielilla;

    #[test]
    fn laudan_neljannekset_antavat_suunnan_ja_ulkopuoli_ei() {
        for suunta in [Suunta::Ylos, Suunta::Alas, Suunta::Vasen, Suunta::Oikea] {
            let (x, y) = laudan_reuna(suunta);
            assert_eq!(suunta_pisteesta(x, y), Some(suunta));
        }
        assert_eq!(suunta_pisteesta(RUUDUT_X - 1, RUUDUT_Y), None);
        assert_eq!(suunta_pisteesta(RUUDUT_X, RUUDUT_Y + LAUDAN_SIVU), None);
    }

    #[test]
    fn napit_eivat_mene_paallekkain_ja_mahtuvat_ruudulle() {
        kaikilla_kielilla(|| {
            for pari in NAPIT.windows(2) {
                assert!(pari[0].2 + napin_leveys(pari[0].1) < pari[1].2);
            }
            let (_, teksti, x) = NAPIT[2];
            assert!(x + napin_leveys(teksti) < LEVEYS as i32);
        });
    }

    #[test]
    fn numerot_mahtuvat_laattaan_ja_tekstit_paneeliin() {
        for pituus in 1..=6 {
            let (l, _, askel) = numeron_koko(pituus);
            assert!(
                pituus as i32 * askel - (askel - l) <= LAATTA - 12,
                "{pituus}"
            );
        }
        kaikilla_kielilla(|| {
            for avain in [
                OHJE,
                EI_LIIKU,
                NIMI_OHJE,
                "luvut.jatketaan",
                "luvut.jatketaan_tallennettua",
            ] {
                let teksti = kaanna(avain);
                assert!(teksti.chars().count() <= RIVILLE_MERKKEJA, "{teksti}");
            }
            let siirto = muotoile("luvut.siirto_pisteet", &[&99999, &99999]);
            assert!(siirto.chars().count() <= RIVILLE_MERKKEJA, "{siirto}");
            assert!(kaanna(JATKA).chars().count() <= 36);
            // Nappi ei saa osua sen vieressä oleviin riveihin.
            for avain in [UUSI_PELI, "yhteinen.aanet_kytkin", "yhteinen.takaisin"] {
                let leveys = kaanna(avain).chars().count() as i32 * MERKKI;
                assert!(48 + leveys < uuden_pelin_x(), "{avain}");
            }
            assert!(kaanna(VOITTO).chars().count() as i32 * TULOS.2 < LEVEYS as i32 - TEKSTI_X);
        });
        // Tallennetun pelin rivi tulee näppäinohjeiden jälkeen.
        let rivit: Vec<&str> = ALKU_RIVIT.iter().copied().chain([UUSI_PELI]).collect();
        alkuohje_mahtuu(&ALKU_OSAT, &rivit, ALOITUS_Y);
    }

    #[test]
    fn koko_peli_piirtyy_kaatumatta_animaatioiden_aikana() {
        let mut kuva = vec![0; LEVEYS * KORKEUS];
        let mut luvut = Luvut::new(3);
        piirra(&luvut, &mut kuva);
        luvut.vaihe = Vaihe::Pelissa;
        let suunnat = [Suunta::Vasen, Suunta::Ylos, Suunta::Oikea, Suunta::Alas];
        for i in 0..400 {
            luvut.hiiren_suunta = Some(suunnat[i % 4]);
            luvut.siirra(suunnat[i % 4]);
            for _ in 0..3 {
                luvut.edista(70);
                piirra(&luvut, &mut kuva);
            }
        }
        luvut.vaihe = Vaihe::Nimi;
        piirra(&luvut, &mut kuva);
    }
}
