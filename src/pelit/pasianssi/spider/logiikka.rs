//! Spider-pasianssin säännöt ilman ikkunaa, piirtoa tai syötettä.

use crate::Arpa;
use crate::kortit::{KUNINGAS, Kortti, Maa, sekoita};
use crate::pasianssi::{Sarake, Vaikeus};

pub const SARAKKEITA: usize = 10;
pub const SARJOJA: usize = 8;
pub const KORTTEJA: usize = SARJOJA * KUNINGAS as usize;
const SARJAN_PITUUS: usize = KUNINGAS as usize;
const PITKIA_SARAKKEITA: usize = 4;
const PITKA_SARAKE: usize = 6;
const LYHYT_SARAKE: usize = 5;

/// Vaikeus on maiden määrä: mitä vähemmän maita, sitä helpommin jonot pysyvät ehjinä.
fn maat(vaikeus: Vaikeus) -> &'static [Maa] {
    match vaikeus {
        Vaikeus::Helppo => &[Maa::Pata],
        Vaikeus::Normaali => &[Maa::Pata, Maa::Hertta],
        Vaikeus::Vaikea => &Maa::KAIKKI,
    }
}

/// Sarakkeen korttijono. `kortti` on jonon alin kortti.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Jono {
    pub sarake: usize,
    pub kortti: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tila {
    /// Jakamattomat kortit. Seuraavaksi jaettava on viimeisenä.
    pub pakka: Vec<Kortti>,
    pub sarakkeet: [Sarake; SARAKKEITA],
    /// Valmiiden sarjojen maat valmistumisjärjestyksessä.
    pub valmiit: Vec<Maa>,
    pub siirrot: u32,
}

pub struct Lauta {
    pub vaikeus: Vaikeus,
    pub tila: Tila,
    historia: Vec<Tila>,
}

impl Lauta {
    pub fn new(vaikeus: Vaikeus, arpa: &mut Arpa) -> Self {
        let maat = maat(vaikeus);
        let mut pakka: Vec<Kortti> = (0..SARJOJA)
            .map(|sarja| maat[sarja % maat.len()])
            .flat_map(|maa| (1..=KUNINGAS).map(move |arvo| Kortti { maa, arvo }))
            .collect();
        sekoita(&mut pakka, arpa);
        let mut sarakkeet: [Sarake; SARAKKEITA] = Default::default();
        for (i, sarake) in sarakkeet.iter_mut().enumerate() {
            let maara = if i < PITKIA_SARAKKEITA {
                PITKA_SARAKE
            } else {
                LYHYT_SARAKE
            };
            sarake.kortit = pakka.split_off(pakka.len() - maara);
            sarake.piilossa = maara - 1;
        }
        Self {
            vaikeus,
            tila: Tila {
                pakka,
                sarakkeet,
                valmiit: Vec::new(),
                siirrot: 0,
            },
            historia: Vec::new(),
        }
    }

    /// Montako jakoa pakassa on jäljellä.
    pub fn jakoja(&self) -> usize {
        self.tila.pakka.len().div_ceil(SARAKKEITA)
    }

    /// Jonon kortit, jos se on ehjä: kuvapuolella, samaa maata ja laskevassa järjestyksessä.
    pub fn siirrettavat(&self, jono: Jono) -> Option<&[Kortti]> {
        let sarake = self.tila.sarakkeet.get(jono.sarake)?;
        if jono.kortti < sarake.piilossa || jono.kortti >= sarake.kortit.len() {
            return None;
        }
        let kortit = &sarake.kortit[jono.kortti..];
        kortit
            .windows(2)
            .all(|pari| pari[1].maa == pari[0].maa && pari[1].arvo + 1 == pari[0].arvo)
            .then_some(kortit)
    }

    /// Jonon saa tyhjään sarakkeeseen tai kortille, joka on arvoltaan yhtä suurempi. Maalla ei ole väliä.
    pub fn voi_siirtaa(&self, jono: Jono, kohde: usize) -> bool {
        let Some(kortit) = self.siirrettavat(jono) else {
            return false;
        };
        let Some(sarake) = self.tila.sarakkeet.get(kohde) else {
            return false;
        };
        jono.sarake != kohde
            && sarake
                .kortit
                .last()
                .is_none_or(|paalla| kortit[0].arvo + 1 == paalla.arvo)
    }

    /// Tekee laillisen siirron. Laittomasta palauttaa epätoden eikä muuta mitään.
    pub fn siirra(&mut self, jono: Jono, kohde: usize) -> bool {
        if !self.voi_siirtaa(jono, kohde) {
            return false;
        }
        self.muista();
        let kortit = self.tila.sarakkeet[jono.sarake]
            .kortit
            .split_off(jono.kortti);
        paljasta(&mut self.tila.sarakkeet[jono.sarake]);
        self.tila.sarakkeet[kohde].kortit.extend(kortit);
        self.keraa_sarja(kohde);
        self.tila.siirrot += 1;
        true
    }

    /// Jakaa saa vain, kun jokaisessa sarakkeessa on kortti. Jos kortteja on pöydällä liian
    /// vähän sarakkeiden täyttämiseen, jako sallitaan, ettei peli jää lukkoon.
    pub fn voi_jakaa(&self) -> bool {
        let sarakkeet = &self.tila.sarakkeet;
        let poydalla: usize = sarakkeet.iter().map(|s| s.kortit.len()).sum();
        !self.tila.pakka.is_empty()
            && (sarakkeet.iter().all(|s| !s.kortit.is_empty()) || poydalla < SARAKKEITA)
    }

    /// Jakaa jokaiseen sarakkeeseen yhden kortin.
    pub fn jaa(&mut self) -> bool {
        if !self.voi_jakaa() {
            return false;
        }
        self.muista();
        for sarake in 0..SARAKKEITA {
            if let Some(kortti) = self.tila.pakka.pop() {
                self.tila.sarakkeet[sarake].kortit.push(kortti);
                self.keraa_sarja(sarake);
            }
        }
        self.tila.siirrot += 1;
        true
    }

    /// Poistaa sarakkeen päältä valmiin sarjan kuninkaasta ässään.
    fn keraa_sarja(&mut self, sarake: usize) {
        let tiedot = &mut self.tila.sarakkeet[sarake];
        let Some(alku) = tiedot.kortit.len().checked_sub(SARJAN_PITUUS) else {
            return;
        };
        let sarja = &tiedot.kortit[alku..];
        let valmis = alku >= tiedot.piilossa
            && sarja[0].arvo == KUNINGAS
            && sarja
                .windows(2)
                .all(|pari| pari[1].maa == pari[0].maa && pari[1].arvo + 1 == pari[0].arvo);
        if valmis {
            let maa = sarja[0].maa;
            tiedot.kortit.truncate(alku);
            paljasta(tiedot);
            self.tila.valmiit.push(maa);
        }
    }

    pub fn on_voitettu(&self) -> bool {
        self.tila.valmiit.len() == SARJOJA
    }

    /// Peruu viimeisimmän siirron. Kello ei peruudu, joten peruutus maksaa aikaa.
    pub fn peruuta(&mut self) -> bool {
        let Some(edellinen) = self.historia.pop() else {
            return false;
        };
        self.tila = edellinen;
        true
    }

    /// Kaikki siirrot, joista `ehto` hyväksyy jonon ja kohteen.
    fn on_siirto(&self, ehto: impl Fn(Jono, usize) -> bool) -> bool {
        (0..SARAKKEITA).any(|sarake| {
            let tiedot = &self.tila.sarakkeet[sarake];
            (tiedot.piilossa..tiedot.kortit.len()).any(|kortti| {
                let jono = Jono { sarake, kortti };
                (0..SARAKKEITA).any(|kohde| self.voi_siirtaa(jono, kohde) && ehto(jono, kohde))
            })
        })
    }

    /// Onko laudalla enää yhtään siirtoa, joka muuttaa tilannetta. Kokonaisen sarakkeen
    /// siirto tyhjästä paikasta toiseen ei ole sellainen.
    pub fn on_siirtoja(&self) -> bool {
        self.voi_jakaa()
            || self.on_siirto(|jono, kohde| {
                jono.kortti != 0 || !self.tila.sarakkeet[kohde].kortit.is_empty()
            })
    }

    /// Onko näkyvissä vielä hyödyllistä siirtoa. Vain vihje pelaajalle: peli loppuu vasta,
    /// kun `on_siirtoja` on epätosi.
    pub fn on_hyodyllisia_siirtoja(&self) -> bool {
        self.voi_jakaa()
            || self.on_siirto(|jono, kohde| {
                let lahde = &self.tila.sarakkeet[jono.sarake];
                let paljastaa = jono.kortti > 0 && jono.kortti == lahde.piilossa;
                let (kortit, alla) = (&lahde.kortit, self.tila.sarakkeet[kohde].kortit.last());
                let tyhjentaa = jono.kortti == 0 && alla.is_some();
                let katkaisee_jonon = jono.kortti > lahde.piilossa
                    && self
                        .siirrettavat(Jono {
                            kortti: jono.kortti - 1,
                            ..jono
                        })
                        .is_some();
                let samaan_maahan = alla.is_some_and(|k| k.maa == kortit[jono.kortti].maa);
                paljastaa || tyhjentaa || (samaan_maahan && !katkaisee_jonon)
            })
    }

    fn muista(&mut self) {
        self.historia.push(self.tila.clone());
    }
}

/// Kääntää sarakkeen päällimmäisen kortin, kun sen päältä lähti viimeinen näkyvä kortti.
fn paljasta(sarake: &mut Sarake) {
    if sarake.piilossa > 0 && sarake.piilossa == sarake.kortit.len() {
        sarake.piilossa -= 1;
    }
}
