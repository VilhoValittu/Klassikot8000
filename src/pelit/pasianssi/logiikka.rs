//! Klondike-pasianssin säännöt ilman ikkunaa, piirtoa tai syötettä.

use crate::Arpa;
use crate::kortit::sekoitettu;
pub use crate::kortit::{KUNINGAS, Kortti, MAITA, Maa};

pub const SARAKKEITA: usize = 7;
pub const KORTTEJA: usize = MAITA * KUNINGAS as usize;

/// Jokainen sekunti vie tasokertoimen verran pisteitä, joten tunnin jälkeen voitosta saa enää pienimmän tuloksen.
const AIKAKATTO_S: u32 = 3600;
pub const SUURIN_TULOS: u32 = AIKAKATTO_S * 3;

impl Kortti {
    fn sopii_paalle(self, alla: Kortti) -> bool {
        self.maa.on_punainen() != alla.maa.on_punainen() && self.arvo + 1 == alla.arvo
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vaikeus {
    Helppo,
    Normaali,
    Vaikea,
}

impl Vaikeus {
    pub const KAIKKI: [Vaikeus; 3] = [Vaikeus::Helppo, Vaikeus::Normaali, Vaikeus::Vaikea];

    pub fn nimi(self) -> &'static str {
        match self {
            Vaikeus::Helppo => "Helppo",
            Vaikeus::Normaali => "Normaali",
            Vaikeus::Vaikea => "Vaikea",
        }
    }

    /// Montako korttia pakasta nostetaan kerralla.
    pub fn nosto(self) -> usize {
        match self {
            Vaikeus::Helppo => 1,
            Vaikeus::Normaali | Vaikeus::Vaikea => 3,
        }
    }

    /// Montako kertaa pakan saa käydä läpi. `None` tarkoittaa rajatonta.
    pub fn kierroksia(self) -> Option<u32> {
        match self {
            Vaikeus::Vaikea => Some(3),
            Vaikeus::Helppo | Vaikeus::Normaali => None,
        }
    }

    pub fn peruutus_sallittu(self) -> bool {
        self != Vaikeus::Vaikea
    }

    /// Tasot jakavat saman listan, joten vaikeampi taso painaa enemmän kuin helpon nopeus.
    fn kerroin(self) -> u32 {
        match self {
            Vaikeus::Helppo => 1,
            Vaikeus::Normaali => 2,
            Vaikeus::Vaikea => 3,
        }
    }
}

/// Pasianssin laji. Jokaisella on oma tuloslistansa.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Variantti {
    Klondike,
    Spider,
    Pyramidi,
}

impl Variantti {
    pub const KAIKKI: [Variantti; 3] =
        [Variantti::Klondike, Variantti::Spider, Variantti::Pyramidi];
}

/// Voitosta saatava tulos: mitä nopeammin, sitä enemmän. Voitto antaa aina jotain.
pub fn aikapisteet(vaikeus: Vaikeus, sekunnit: u32) -> u32 {
    AIKAKATTO_S.saturating_sub(sekunnit).max(1) * vaikeus.kerroin()
}

/// Sarakkeen kortit alimmasta päällimmäiseen. Alimmat `piilossa` korttia ovat nurin.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Sarake {
    pub kortit: Vec<Kortti>,
    pub piilossa: usize,
}

/// Mistä kortti tai korttijono otetaan. `kortti` on jonon alin kortti sarakkeessa.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lahde {
    Jate,
    Perusta(usize),
    Sarake { sarake: usize, kortti: usize },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kohde {
    Perusta(usize),
    Sarake(usize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tila {
    /// Päällimmäinen kortti on viimeisenä.
    pub pakka: Vec<Kortti>,
    pub jate: Vec<Kortti>,
    /// Yksi perusta kullekin maalle `Maa::KAIKKI`-järjestyksessä.
    pub perustat: [Vec<Kortti>; MAITA],
    pub sarakkeet: [Sarake; SARAKKEITA],
    pub kierratykset: u32,
    pub siirrot: u32,
}

pub struct Lauta {
    pub vaikeus: Vaikeus,
    pub tila: Tila,
    historia: Vec<Tila>,
}

impl Lauta {
    pub fn new(vaikeus: Vaikeus, arpa: &mut Arpa) -> Self {
        let mut pakka = sekoitettu(1, arpa);
        let mut sarakkeet: [Sarake; SARAKKEITA] = Default::default();
        for (i, sarake) in sarakkeet.iter_mut().enumerate() {
            sarake.kortit = pakka.split_off(pakka.len() - (i + 1));
            sarake.piilossa = i;
        }
        Self {
            vaikeus,
            tila: Tila {
                pakka,
                jate: Vec::new(),
                perustat: Default::default(),
                sarakkeet,
                kierratykset: 0,
                siirrot: 0,
            },
            historia: Vec::new(),
        }
    }

    /// Monesko pakan läpikäynti on menossa, ensimmäinen on 1.
    pub fn kierros(&self) -> u32 {
        self.tila.kierratykset + 1
    }

    pub fn voi_kierrattaa(&self) -> bool {
        !self.tila.jate.is_empty()
            && self
                .vaikeus
                .kierroksia()
                .is_none_or(|kierroksia| self.kierros() < kierroksia)
    }

    /// Nostaa pakasta tai kääntää tyhjän pakan ympäri. Palauttaa, tapahtuiko mitään.
    pub fn nosta(&mut self) -> bool {
        if self.tila.pakka.is_empty() {
            if !self.voi_kierrattaa() {
                return false;
            }
            self.muista();
            let tila = &mut self.tila;
            tila.pakka = std::mem::take(&mut tila.jate);
            tila.pakka.reverse();
            tila.kierratykset += 1;
        } else {
            self.muista();
            let nosto = self.vaikeus.nosto().min(self.tila.pakka.len());
            for _ in 0..nosto {
                if let Some(kortti) = self.tila.pakka.pop() {
                    self.tila.jate.push(kortti);
                }
            }
        }
        self.tila.siirrot += 1;
        true
    }

    /// Kortit, jotka lähde antaa siirrettäväksi. Sarakkeesta vain ehjä, kuvapuolella oleva jono.
    pub fn siirrettavat(&self, lahde: Lahde) -> Option<&[Kortti]> {
        let viimeinen = |kortit: &'_ [Kortti]| kortit.len().checked_sub(1);
        match lahde {
            Lahde::Jate => viimeinen(&self.tila.jate).map(|i| &self.tila.jate[i..]),
            Lahde::Perusta(perusta) => {
                let kortit = self.tila.perustat.get(perusta)?;
                viimeinen(kortit).map(|i| &kortit[i..])
            }
            Lahde::Sarake { sarake, kortti } => {
                let sarake = self.tila.sarakkeet.get(sarake)?;
                if kortti < sarake.piilossa || kortti >= sarake.kortit.len() {
                    return None;
                }
                let jono = &sarake.kortit[kortti..];
                jono.windows(2)
                    .all(|pari| pari[1].sopii_paalle(pari[0]))
                    .then_some(jono)
            }
        }
    }

    pub fn voi_siirtaa(&self, lahde: Lahde, kohde: Kohde) -> bool {
        let Some(kortit) = self.siirrettavat(lahde) else {
            return false;
        };
        let alin = kortit[0];
        match kohde {
            Kohde::Perusta(perusta) => {
                let Some(pino) = self.tila.perustat.get(perusta) else {
                    return false;
                };
                kortit.len() == 1
                    && Maa::KAIKKI[perusta] == alin.maa
                    && pino.len() + 1 == alin.arvo as usize
            }
            Kohde::Sarake(kohde) => {
                if matches!(lahde, Lahde::Sarake { sarake, .. } if sarake == kohde) {
                    return false;
                }
                let Some(sarake) = self.tila.sarakkeet.get(kohde) else {
                    return false;
                };
                match sarake.kortit.last() {
                    Some(&paalla) => alin.sopii_paalle(paalla),
                    None => alin.arvo == KUNINGAS,
                }
            }
        }
    }

    /// Tekee laillisen siirron. Laittomasta palauttaa epätoden eikä muuta mitään.
    pub fn siirra(&mut self, lahde: Lahde, kohde: Kohde) -> bool {
        if !self.voi_siirtaa(lahde, kohde) {
            return false;
        }
        self.muista();
        let tila = &mut self.tila;
        let kortit: Vec<Kortti> = match lahde {
            Lahde::Jate => tila.jate.pop().into_iter().collect(),
            Lahde::Perusta(perusta) => tila.perustat[perusta].pop().into_iter().collect(),
            Lahde::Sarake { sarake, kortti } => {
                let sarake = &mut tila.sarakkeet[sarake];
                let jono = sarake.kortit.split_off(kortti);
                if sarake.piilossa > 0 && sarake.piilossa == sarake.kortit.len() {
                    sarake.piilossa -= 1;
                }
                jono
            }
        };
        match kohde {
            Kohde::Perusta(perusta) => tila.perustat[perusta].extend(kortit),
            Kohde::Sarake(sarake) => tila.sarakkeet[sarake].kortit.extend(kortit),
        }
        tila.siirrot += 1;
        true
    }

    /// Siirtää yksittäisen kortin oman maansa perustalle.
    pub fn perustalle(&mut self, lahde: Lahde) -> bool {
        let Some(&[kortti]) = self.siirrettavat(lahde) else {
            return false;
        };
        let Some(perusta) = Maa::KAIKKI.iter().position(|&maa| maa == kortti.maa) else {
            return false;
        };
        self.siirra(lahde, Kohde::Perusta(perusta))
    }

    /// Siirtää perustoille kaiken, mikä sinne juuri nyt pääsee. Palauttaa korttien määrän.
    pub fn keraa(&mut self) -> usize {
        let mut siirretty = 0;
        loop {
            let jatteesta = self.perustalle(Lahde::Jate);
            let sarakkeista = (0..SARAKKEITA).any(|sarake| {
                let kortti = self.tila.sarakkeet[sarake].kortit.len().wrapping_sub(1);
                self.perustalle(Lahde::Sarake { sarake, kortti })
            });
            if !jatteesta && !sarakkeista {
                return siirretty;
            }
            siirretty += jatteesta as usize + sarakkeista as usize;
        }
    }

    pub fn on_voitettu(&self) -> bool {
        self.tila
            .perustat
            .iter()
            .all(|pino| pino.len() == KUNINGAS as usize)
    }

    /// Peruu viimeisimmän siirron. Kello ei peruudu, joten peruutus maksaa aikaa.
    pub fn peruuta(&mut self) -> bool {
        if !self.vaikeus.peruutus_sallittu() {
            return false;
        }
        let Some(edellinen) = self.historia.pop() else {
            return false;
        };
        self.tila = edellinen;
        true
    }

    /// Voiko kortin laskea jollekin perustalle tai sarakkeeseen.
    fn kortille_on_paikka(&self, kortti: Kortti) -> bool {
        let perustalle = (0..MAITA).any(|perusta| {
            Maa::KAIKKI[perusta] == kortti.maa
                && self.tila.perustat[perusta].len() + 1 == kortti.arvo as usize
        });
        perustalle
            || self
                .tila
                .sarakkeet
                .iter()
                .any(|sarake| match sarake.kortit.last() {
                    Some(&paalla) => kortti.sopii_paalle(paalla),
                    None => kortti.arvo == KUNINGAS,
                })
    }

    /// Saako jätepinon päällimmäiseksi nostamalla kortin, jolle on paikka.
    fn pakasta_saa_kortin(&self) -> bool {
        let nosto = self.vaikeus.nosto();
        let kelpaa = |kortit: &[Kortti]| {
            kortit
                .chunks(nosto)
                .filter_map(<[Kortti]>::last)
                .any(|&kortti| self.kortille_on_paikka(kortti))
        };
        let tila = &self.tila;
        if tila
            .jate
            .last()
            .is_some_and(|&k| self.kortille_on_paikka(k))
        {
            return true;
        }
        let pakasta: Vec<Kortti> = tila.pakka.iter().rev().copied().collect();
        if kelpaa(&pakasta) {
            return true;
        }
        // Yksi kokonainen kierto riittää: seuraavat toistavat samat kortit.
        let saa_kaantaa = self
            .vaikeus
            .kierroksia()
            .is_none_or(|kierroksia| self.kierros() < kierroksia);
        let kierto: Vec<Kortti> = tila.jate.iter().chain(&pakasta).copied().collect();
        saa_kaantaa && kelpaa(&kierto)
    }

    /// Onko laudalla enää yhtään siirtoa, joka muuttaa tilannetta. Pelkkä pakan selaaminen
    /// ei ole sellainen, ei myöskään kokonaisen sarakkeen siirto tyhjästä paikasta toiseen.
    pub fn on_siirtoja(&self) -> bool {
        if self.pakasta_saa_kortin() {
            return true;
        }
        let perustalta = (0..MAITA).any(|perusta| {
            (0..SARAKKEITA)
                .any(|kohde| self.voi_siirtaa(Lahde::Perusta(perusta), Kohde::Sarake(kohde)))
        });
        perustalta
            || (0..SARAKKEITA).any(|sarake| {
                let tiedot = &self.tila.sarakkeet[sarake];
                (tiedot.piilossa..tiedot.kortit.len()).any(|kortti| {
                    let lahde = Lahde::Sarake { sarake, kortti };
                    let perustalle =
                        (0..MAITA).any(|perusta| self.voi_siirtaa(lahde, Kohde::Perusta(perusta)));
                    perustalle
                        || (0..SARAKKEITA).any(|kohde| {
                            let turha = kortti == 0 && self.tila.sarakkeet[kohde].kortit.is_empty();
                            !turha && self.voi_siirtaa(lahde, Kohde::Sarake(kohde))
                        })
                })
            })
    }

    /// Onko näkyvissä vielä hyödyllistä siirtoa. Vain vihje pelaajalle: jonon osan siirto
    /// saattaa yhä auttaa, joten peli loppuu vasta, kun `on_siirtoja` on epätosi.
    pub fn on_hyodyllisia_siirtoja(&self) -> bool {
        // Pakan kortit tulevat vielä vastaan, jos pakkaa on jäljellä tai sen saa kääntää.
        let pakka_kiertaa = !self.tila.pakka.is_empty() || self.voi_kierrattaa();
        let pakasta = if pakka_kiertaa {
            self.tila
                .pakka
                .iter()
                .chain(&self.tila.jate)
                .any(|&k| self.kortille_on_paikka(k))
        } else {
            self.tila
                .jate
                .last()
                .is_some_and(|&k| self.kortille_on_paikka(k))
        };
        if pakasta {
            return true;
        }
        (0..SARAKKEITA).any(|sarake| {
            let tiedot = &self.tila.sarakkeet[sarake];
            let paallimmainen = tiedot.kortit.len().wrapping_sub(1);
            let perustalle = (0..MAITA).any(|perusta| {
                self.voi_siirtaa(
                    Lahde::Sarake {
                        sarake,
                        kortti: paallimmainen,
                    },
                    Kohde::Perusta(perusta),
                )
            });
            // Koko jonon siirto hyödyttää vain, jos sen alta paljastuu uusi kortti.
            let jono = Lahde::Sarake {
                sarake,
                kortti: tiedot.piilossa,
            };
            let paljastaa = tiedot.piilossa > 0
                && (0..SARAKKEITA).any(|kohde| self.voi_siirtaa(jono, Kohde::Sarake(kohde)));
            perustalle || paljastaa
        })
    }

    fn muista(&mut self) {
        self.historia.push(self.tila.clone());
    }
}
