//! Tiilirikon säännöt ilman ikkunaa, piirtoa tai syötettä. Mitat ovat kentän pikseleitä,
//! ja aika etenee kiinteinä askelina, joten sama siemen ja ohjaus toistavat saman pelin.

use crate::Arpa;

mod kentat;

use kentat::KENTAT;

pub const LEVEYS: f32 = 660.0;
pub const KORKEUS: f32 = 540.0;
pub const SARAKKEITA: usize = 11;
pub const RIVEJA: usize = 8;
pub const TIILI_L: f32 = LEVEYS / SARAKKEITA as f32;
pub const TIILI_K: f32 = 24.0;
pub const TIILET_Y: f32 = 48.0;
pub const MAILA_L: f32 = 108.0;
/// Mailan leveys, kun pelissä on kaksi palloa.
pub const LEVEA_MAILA_L: f32 = MAILA_L * 1.5;
pub const MAILA_K: f32 = 15.0;
pub const MAILA_Y: f32 = KORKEUS - 39.0;
pub const PALLON_SADE: f32 = 7.0;
pub const ASKEL_MS: u32 = 5;
pub const ELAMIA: u32 = 3;
pub const PALLOJA_ENINTAAN: usize = 2;
pub const SUURIN_PISTEMAARA: u32 = 999_999;
/// Superpallon vauhti pallon omaan vauhtiin verrattuna.
pub const SUPERVAUHTI: f32 = 1.9;
/// Superpallon osuman pistekerroin.
pub const SUPERKERROIN: u32 = 4;

const ASKEL_S: f32 = ASKEL_MS as f32 / 1000.0;
const MAILAN_VAUHTI: f32 = 780.0;
const PERUSVAUHTI: f32 = 330.0;
// Viisikymmentä kenttää: kasvu pysyy pienenä, jotta viimeisetkin ovat pelattavissa.
const VAUHTI_KENTTAA_KOHTI: f32 = 3.0;
const VAUHTI_TIILTA_KOHTI: f32 = 4.0;
// Askel on pallon halkaisijaa lyhyempi superpallonkin vauhdilla, joten pallo ei hyppää tiilen yli.
const SUURIN_VAUHTI: f32 = 660.0;
const SUURIN_KULMA: f32 = 1.05;
const UUDEN_PALLON_KULMA: f32 = 0.5;
const PISTEET_OSUMASTA: u32 = 10;
// Pistekerroin kasvaa kymmenen kentän välein, joten tulos mahtuu kuuteen numeroon.
const KENTTIA_KERROINTA_KOHTI: usize = 10;
// Bonus putoaa tasaisesti nollaan aikarajan aikana ja kerrotaan pistekertoimella.
const AIKABONUS_ALUSSA: u32 = 1000;
const AIKARAJA_MS: u32 = 240_000;
const SUPERTIILIA: usize = 1;

pub const KENTTIA: usize = KENTAT.len();

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tila {
    /// Pallo odottaa mailan päällä laukaisua.
    Kiinni,
    Lennossa,
    Voitto,
    Ohi,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tapahtuma {
    Seina,
    Maila,
    Tiili {
        rivi: usize,
        sarake: usize,
        rikki: bool,
    },
    /// Pallotiili rikkoutui ja kentälle tuli toinen pallo.
    UusiPallo,
    /// Supertiili rikkoutui ja sen rikkonut pallo muuttui superpalloksi.
    Superpallo,
    /// Toinen kahdesta pallosta putosi, joten elämä säilyy.
    ToinenMeni,
    Menetys,
    KenttaLapi {
        bonus: u32,
    },
    Voitto {
        bonus: u32,
    },
    Ohi,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pallo {
    pub paikka: (f32, f32),
    pub nopeus: (f32, f32),
    /// Kasvaa vain tämän pallon omista tiiliosumista.
    pub vauhti: f32,
    /// Superpallo kulkee tiilien läpi, kunnes se putoaa tai kenttä vaihtuu.
    pub superpallo: bool,
}

impl Pallo {
    /// Vauhti, jolla pallo todella liikkuu.
    pub fn lentovauhti(&self) -> f32 {
        if self.superpallo {
            self.vauhti * SUPERVAUHTI
        } else {
            self.vauhti
        }
    }
}

pub struct Tiilipeli {
    pub tiilet: [[u8; SARAKKEITA]; RIVEJA],
    /// Tiilet, joiden rikkoutuminen tuo toisen pallon.
    pub pallotiilet: [[bool; SARAKKEITA]; RIVEJA],
    /// Tiilet, joiden rikkoutuminen tekee rikkojasta superpallon.
    pub supertiilet: [[bool; SARAKKEITA]; RIVEJA],
    pub maila_x: f32,
    /// Pelissä olevat pallot. Tyhjä vain, kun peli on päättynyt tappioon.
    pub pallot: Vec<Pallo>,
    pub tila: Tila,
    pub elamat: u32,
    pub pisteet: u32,
    /// Nollasta alkava kentän numero.
    pub kentta: usize,
    /// Kentällä kulunut aika. Kello käy vain pallon lentäessä, joten tauko ja
    /// laukaisun odotus eivät vie pisteitä.
    pub kentan_ms: u32,
}

fn kentan_tiilet(kentta: usize) -> [[u8; SARAKKEITA]; RIVEJA] {
    let mut tiilet = [[0; SARAKKEITA]; RIVEJA];
    for (rivi, kuvio) in tiilet.iter_mut().zip(KENTAT[kentta]) {
        for (tiili, merkki) in rivi.iter_mut().zip(kuvio.chars()) {
            *tiili = merkki.to_digit(10).unwrap_or(0) as u8;
        }
    }
    tiilet
}

impl Tiilipeli {
    /// Peli alkaa annetusta kentästä, joka rajataan olemassa oleviin.
    pub fn new(kentta: usize, arpa: &mut Arpa) -> Self {
        let kentta = kentta.min(KENTTIA - 1);
        let mut peli = Self {
            tiilet: [[0; SARAKKEITA]; RIVEJA],
            pallotiilet: [[false; SARAKKEITA]; RIVEJA],
            supertiilet: [[false; SARAKKEITA]; RIVEJA],
            maila_x: LEVEYS / 2.0,
            pallot: Vec::with_capacity(PALLOJA_ENINTAAN),
            tila: Tila::Kiinni,
            elamat: ELAMIA,
            pisteet: 0,
            kentta,
            kentan_ms: 0,
        };
        peli.lataa_kentta(arpa);
        peli.kiinnita_pallo();
        peli
    }

    /// Asettaa kentän tiilet ja arpoo niistä palkintotiilet, jotka kestävät yhden osuman.
    /// Palkintotiilet eivät kosketa toisiaan edes kulmittain.
    fn lataa_kentta(&mut self, arpa: &mut Arpa) {
        self.tiilet = kentan_tiilet(self.kentta);
        self.pallotiilet = [[false; SARAKKEITA]; RIVEJA];
        self.supertiilet = [[false; SARAKKEITA]; RIVEJA];
        let mut ehdokkaat = Vec::new();
        for rivi in 0..RIVEJA {
            for sarake in 0..SARAKKEITA {
                if self.tiilet[rivi][sarake] > 0 {
                    ehdokkaat.push((rivi, sarake));
                }
            }
        }
        // Loppupään kentissä on yksi pallotiili enemmän.
        let pallotiilia = 3 + self.kentta * 2 / KENTTIA;
        for i in 0..pallotiilia + SUPERTIILIA {
            if ehdokkaat.is_empty() {
                break;
            }
            let (rivi, sarake) = ehdokkaat.swap_remove(arpa.alle(ehdokkaat.len()));
            self.tiilet[rivi][sarake] = 1;
            if i < pallotiilia {
                self.pallotiilet[rivi][sarake] = true;
            } else {
                self.supertiilet[rivi][sarake] = true;
            }
            ehdokkaat
                .retain(|&(r, s): &(usize, usize)| r.abs_diff(rivi) > 1 || s.abs_diff(sarake) > 1);
        }
    }

    fn perusvauhti(&self) -> f32 {
        PERUSVAUHTI + self.kentta as f32 * VAUHTI_KENTTAA_KOHTI
    }

    /// Tiilipisteiden ja aikabonuksen kerroin.
    fn kerroin(&self) -> u32 {
        (self.kentta / KENTTIA_KERROINTA_KOHTI) as u32 + 1
    }

    fn lisaa_pisteita(&mut self, maara: u32) {
        self.pisteet = (self.pisteet + maara).min(SUURIN_PISTEMAARA);
    }

    pub fn tiilia_jaljella(&self) -> usize {
        self.tiilet.iter().flatten().filter(|&&t| t > 0).count()
    }

    /// Bonus, jonka kentän läpäisy juuri nyt antaisi.
    pub fn aikabonus(&self) -> u32 {
        let jaljella = AIKARAJA_MS.saturating_sub(self.kentan_ms);
        AIKABONUS_ALUSSA * self.kerroin() * (jaljella / 1000) / (AIKARAJA_MS / 1000)
    }

    /// Kahden pallon aikana maila on leveämpi.
    pub fn maila_l(&self) -> f32 {
        if self.pallot.len() >= 2 {
            LEVEA_MAILA_L
        } else {
            MAILA_L
        }
    }

    fn kiinnita_pallo(&mut self) {
        self.tila = Tila::Kiinni;
        self.pallot.clear();
        self.pallot.push(Pallo {
            paikka: (self.maila_x, MAILA_Y - PALLON_SADE),
            nopeus: (0.0, 0.0),
            vauhti: 0.0,
            superpallo: false,
        });
    }

    /// Maila pysyy kokonaan kentällä, ja kiinni oleva pallo kulkee sen mukana.
    pub fn aseta_maila(&mut self, x: f32) {
        let puolikas = self.maila_l() / 2.0;
        self.maila_x = x.clamp(puolikas, LEVEYS - puolikas);
        if self.tila == Tila::Kiinni {
            self.pallot[0].paikka.0 = self.maila_x;
        }
    }

    pub fn laukaise(&mut self, arpa: &mut Arpa) {
        if self.tila != Tila::Kiinni {
            return;
        }
        self.tila = Tila::Lennossa;
        let vauhti = self.perusvauhti();
        // Arvottu kulma estää lähdön suoraan ylös, jolloin pallo toistaisi samaa rataa.
        let kulma = (10 + arpa.alle(26)) as f32 * std::f32::consts::PI / 180.0;
        let suunta = if arpa.alle(2) == 0 { -1.0 } else { 1.0 };
        self.pallot[0].vauhti = vauhti;
        self.pallot[0].nopeus = (suunta * vauhti * kulma.sin(), -vauhti * kulma.cos());
    }

    /// Etenee yhden `ASKEL_MS`:n verran. `ohjaus` on -1 vasemmalle, 1 oikealle, 0 paikallaan.
    /// Arpaa tarvitaan, kun kenttä vaihtuu ja seuraavan pallotiilet arvotaan.
    pub fn askel(&mut self, ohjaus: i8, arpa: &mut Arpa) -> Vec<Tapahtuma> {
        let mut tapahtumat = Vec::new();
        if matches!(self.tila, Tila::Voitto | Tila::Ohi) {
            return tapahtumat;
        }
        self.aseta_maila(self.maila_x + ohjaus as f32 * MAILAN_VAUHTI * ASKEL_S);
        if self.tila != Tila::Lennossa {
            return tapahtumat;
        }
        self.kentan_ms = self.kentan_ms.saturating_add(ASKEL_MS);

        // Takaperin, koska poisto tai lisäys ei siirrä vielä liikkumattomia palloja.
        for i in (0..self.pallot.len()).rev() {
            self.liikuta_palloa(i, &mut tapahtumat, arpa);
            if self.tila != Tila::Lennossa {
                break;
            }
        }
        tapahtumat
    }

    fn liikuta_palloa(&mut self, i: usize, tapahtumat: &mut Vec<Tapahtuma>, arpa: &mut Arpa) {
        // Akselit erikseen: törmäyksen suunta on aina selvä, eikä pallo jää tiilen sisään.
        self.pallot[i].paikka.0 += self.pallot[i].nopeus.0 * ASKEL_S;
        let x = self.pallot[i].paikka.0;
        let sivuseina = if x < PALLON_SADE {
            Some(PALLON_SADE)
        } else if x > LEVEYS - PALLON_SADE {
            Some(LEVEYS - PALLON_SADE)
        } else {
            None
        };
        if let Some(x) = sivuseina {
            self.pallot[i].paikka.0 = x;
            self.pallot[i].nopeus.0 = -self.pallot[i].nopeus.0;
            tapahtumat.push(Tapahtuma::Seina);
        } else if self.pallot[i].superpallo {
            self.lapaise_tiilet(i, tapahtumat, arpa);
            if self.tila != Tila::Lennossa {
                return;
            }
        } else if let Some(paikka) = self.osuva_tiili(self.pallot[i].paikka) {
            self.pallot[i].paikka.0 -= self.pallot[i].nopeus.0 * ASKEL_S;
            self.pallot[i].nopeus.0 = -self.pallot[i].nopeus.0;
            self.osu_tiileen(i, paikka, tapahtumat, arpa);
            if self.tila != Tila::Lennossa {
                return;
            }
        }

        self.pallot[i].paikka.1 += self.pallot[i].nopeus.1 * ASKEL_S;
        if self.pallot[i].paikka.1 < PALLON_SADE {
            self.pallot[i].paikka.1 = PALLON_SADE;
            self.pallot[i].nopeus.1 = -self.pallot[i].nopeus.1;
            tapahtumat.push(Tapahtuma::Seina);
        } else if self.pallot[i].superpallo {
            self.lapaise_tiilet(i, tapahtumat, arpa);
            if self.tila != Tila::Lennossa {
                return;
            }
        } else if let Some(paikka) = self.osuva_tiili(self.pallot[i].paikka) {
            self.pallot[i].paikka.1 -= self.pallot[i].nopeus.1 * ASKEL_S;
            self.pallot[i].nopeus.1 = -self.pallot[i].nopeus.1;
            self.osu_tiileen(i, paikka, tapahtumat, arpa);
            if self.tila != Tila::Lennossa {
                return;
            }
        }

        if self.osuu_mailaan(self.pallot[i]) {
            self.pallot[i].paikka.1 = MAILA_Y - PALLON_SADE;
            // Mailan laita kääntää pallon jyrkemmin, joten pelaaja voi tähdätä.
            let osuma =
                (self.pallot[i].paikka.0 - self.maila_x) / (self.maila_l() / 2.0 + PALLON_SADE);
            let kulma = osuma.clamp(-1.0, 1.0) * SUURIN_KULMA;
            let vauhti = self.pallot[i].lentovauhti();
            self.pallot[i].nopeus = (vauhti * kulma.sin(), -vauhti * kulma.cos());
            tapahtumat.push(Tapahtuma::Maila);
        } else if self.pallot[i].paikka.1 - PALLON_SADE > KORKEUS {
            self.poista_pallo(i, tapahtumat);
        }
    }

    /// Elämä kuluu vasta, kun kentältä putoaa viimeinen pallo.
    fn poista_pallo(&mut self, i: usize, tapahtumat: &mut Vec<Tapahtuma>) {
        self.pallot.remove(i);
        if !self.pallot.is_empty() {
            tapahtumat.push(Tapahtuma::ToinenMeni);
            return;
        }
        self.elamat -= 1;
        if self.elamat == 0 {
            self.tila = Tila::Ohi;
            tapahtumat.push(Tapahtuma::Ohi);
        } else {
            self.kiinnita_pallo();
            tapahtumat.push(Tapahtuma::Menetys);
        }
    }

    fn osuu_mailaan(&self, pallo: Pallo) -> bool {
        let (x, y) = pallo.paikka;
        pallo.nopeus.1 > 0.0
            && y + PALLON_SADE >= MAILA_Y
            && y - PALLON_SADE < MAILA_Y + MAILA_K
            && (x - self.maila_x).abs() <= self.maila_l() / 2.0 + PALLON_SADE
    }

    /// Ensimmäinen ehjä tiili pallon neliön alla.
    fn osuva_tiili(&self, (x, y): (f32, f32)) -> Option<(usize, usize)> {
        let rivit = (y - PALLON_SADE - TIILET_Y) / TIILI_K..(y + PALLON_SADE - TIILET_Y) / TIILI_K;
        let sarakkeet = (x - PALLON_SADE) / TIILI_L..(x + PALLON_SADE) / TIILI_L;
        let viimeinen = |arvo: f32| arvo.ceil() as i32;
        for rivi in rivit.start.floor() as i32..viimeinen(rivit.end) {
            for sarake in sarakkeet.start.floor() as i32..viimeinen(sarakkeet.end) {
                if !(0..RIVEJA as i32).contains(&rivi) || !(0..SARAKKEITA as i32).contains(&sarake)
                {
                    continue;
                }
                let paikka = (rivi as usize, sarake as usize);
                if self.tiilet[paikka.0][paikka.1] > 0 {
                    return Some(paikka);
                }
            }
        }
        None
    }

    /// Superpallo rikkoo kaikki alleen jäävät tiilet eikä kimpoa niistä.
    fn lapaise_tiilet(&mut self, i: usize, tapahtumat: &mut Vec<Tapahtuma>, arpa: &mut Arpa) {
        while self.tila == Tila::Lennossa {
            let Some(paikka) = self.osuva_tiili(self.pallot[i].paikka) else {
                return;
            };
            self.osu_tiileen(i, paikka, tapahtumat, arpa);
        }
    }

    fn osu_tiileen(
        &mut self,
        i: usize,
        (rivi, sarake): (usize, usize),
        tapahtumat: &mut Vec<Tapahtuma>,
        arpa: &mut Arpa,
    ) {
        let superpallo = self.pallot[i].superpallo;
        let tiili = &mut self.tiilet[rivi][sarake];
        // Läpi kulkeva pallo ei palaa tiileen, joten kestäväkin tiili rikkoutuu kerralla.
        *tiili = if superpallo { 0 } else { *tiili - 1 };
        let rikki = *tiili == 0;
        // Kahta palloa on vaikeampi hallita, joten kumpikin kertoo osuman pisteet.
        let kerroin = if superpallo { SUPERKERROIN } else { 1 };
        self.lisaa_pisteita(PISTEET_OSUMASTA * self.kerroin() * self.pallot.len() as u32 * kerroin);
        tapahtumat.push(Tapahtuma::Tiili {
            rivi,
            sarake,
            rikki,
        });

        // Superpallon vauhti pysyy tasaisena, muuten läpi ajetut rivit kiihdyttäisivät sen heti.
        if !superpallo {
            let pallo = &mut self.pallot[i];
            let uusi = (pallo.vauhti + VAUHTI_TIILTA_KOHTI).min(SUURIN_VAUHTI);
            pallo.nopeus = (
                pallo.nopeus.0 * uusi / pallo.vauhti,
                pallo.nopeus.1 * uusi / pallo.vauhti,
            );
            pallo.vauhti = uusi;
        }

        if self.tiilia_jaljella() > 0 {
            if rikki && self.supertiilet[rivi][sarake] && !superpallo {
                let pallo = &mut self.pallot[i];
                pallo.superpallo = true;
                pallo.nopeus = (pallo.nopeus.0 * SUPERVAUHTI, pallo.nopeus.1 * SUPERVAUHTI);
                tapahtumat.push(Tapahtuma::Superpallo);
            }
            if rikki && self.pallotiilet[rivi][sarake] && self.pallot.len() < PALLOJA_ENINTAAN {
                self.lisaa_pallo(i, (rivi, sarake));
                tapahtumat.push(Tapahtuma::UusiPallo);
            }
            return;
        }
        let bonus = self.aikabonus();
        self.lisaa_pisteita(bonus);
        if self.kentta + 1 == KENTTIA {
            self.tila = Tila::Voitto;
            tapahtumat.push(Tapahtuma::Voitto { bonus });
        } else {
            self.kentta += 1;
            self.kentan_ms = 0;
            self.lataa_kentta(arpa);
            self.kiinnita_pallo();
            tapahtumat.push(Tapahtuma::KenttaLapi { bonus });
        }
    }

    /// Uusi pallo lähtee rikkoutuneesta tiilestä alas kentän perusvauhdilla, eri suuntaan
    /// kuin osunut pallo. Superpallo tuplaantuu superpalloksi.
    fn lisaa_pallo(&mut self, osunut: usize, (rivi, sarake): (usize, usize)) {
        let suunta = if self.pallot[osunut].nopeus.0 > 0.0 {
            -1.0
        } else {
            1.0
        };
        let mut pallo = Pallo {
            paikka: (
                (sarake as f32 + 0.5) * TIILI_L,
                TIILET_Y + (rivi as f32 + 0.5) * TIILI_K,
            ),
            nopeus: (0.0, 0.0),
            vauhti: self.perusvauhti(),
            superpallo: self.pallot[osunut].superpallo,
        };
        let vauhti = pallo.lentovauhti();
        pallo.nopeus = (
            suunta * vauhti * UUDEN_PALLON_KULMA.sin(),
            vauhti * UUDEN_PALLON_KULMA.cos(),
        );
        self.pallot.push(pallo);
        // Leveämpi maila ei saa jäädä seinän sisään.
        self.aseta_maila(self.maila_x);
    }
}
