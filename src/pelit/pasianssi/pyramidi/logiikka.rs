//! Pyramidipasianssin säännöt ilman ikkunaa, piirtoa tai syötettä.

use crate::Arpa;
use crate::kortit::{KUNINGAS, Kortti, sekoitettu};
use crate::pasianssi::Vaikeus;

pub const RIVEJA: usize = 7;
pub const PYRAMIDISSA: usize = RIVEJA * (RIVEJA + 1) / 2;
/// Montako jätepinon päällimmäistä korttia on pelattavissa.
pub const JATTEEN_NAKYVAT: usize = 2;
const SUMMA: u8 = 13;

/// Montako kertaa pakan saa käydä läpi. `None` tarkoittaa rajatonta.
pub fn kierroksia(vaikeus: Vaikeus) -> Option<u32> {
    match vaikeus {
        Vaikeus::Helppo => None,
        Vaikeus::Normaali => Some(3),
        Vaikeus::Vaikea => Some(2),
    }
}

/// Kortin paikka. `Jate(0)` on jätepinon päällimmäinen ja `Jate(1)` sen alla oleva.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Paikka {
    Pyramidi { rivi: usize, sarake: usize },
    Jate(usize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tila {
    /// Rivillä `r` on `r + 1` korttia. Poistettu kortti on `None`.
    pub rivit: [[Option<Kortti>; RIVEJA]; RIVEJA],
    /// Seuraavaksi nostettava on viimeisenä.
    pub pakka: Vec<Kortti>,
    pub jate: Vec<Kortti>,
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
        let mut rivit = [[None; RIVEJA]; RIVEJA];
        for (r, rivi) in rivit.iter_mut().enumerate() {
            for paikka in &mut rivi[..=r] {
                *paikka = pakka.pop();
            }
        }
        Self {
            vaikeus,
            tila: Tila {
                rivit,
                pakka,
                jate: Vec::new(),
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
            && kierroksia(self.vaikeus).is_none_or(|kierroksia| self.kierros() < kierroksia)
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
            if let Some(kortti) = self.tila.pakka.pop() {
                self.tila.jate.push(kortti);
            }
        }
        self.tila.siirrot += 1;
        true
    }

    /// Pyramidin kortti on vapaa, kun sen päällä olevat kaksi korttia on poistettu.
    pub fn on_vapaa(&self, rivi: usize, sarake: usize) -> bool {
        let rivit = &self.tila.rivit;
        let olemassa = rivi < RIVEJA && sarake <= rivi && rivit[rivi][sarake].is_some();
        olemassa
            && (rivi + 1 == RIVEJA
                || (rivit[rivi + 1][sarake].is_none() && rivit[rivi + 1][sarake + 1].is_none()))
    }

    /// Paikan kortti, jos sen voi juuri nyt poistaa.
    pub fn pelattava(&self, paikka: Paikka) -> Option<Kortti> {
        match paikka {
            Paikka::Pyramidi { rivi, sarake } => {
                if self.on_vapaa(rivi, sarake) {
                    self.tila.rivit[rivi][sarake]
                } else {
                    None
                }
            }
            Paikka::Jate(syvyys) if syvyys < JATTEEN_NAKYVAT => {
                let i = self.tila.jate.len().checked_sub(1 + syvyys)?;
                Some(self.tila.jate[i])
            }
            Paikka::Jate(_) => None,
        }
    }

    fn ota_pois(&mut self, paikka: Paikka) {
        match paikka {
            Paikka::Pyramidi { rivi, sarake } => self.tila.rivit[rivi][sarake] = None,
            Paikka::Jate(syvyys) => {
                let i = self.tila.jate.len() - 1 - syvyys;
                self.tila.jate.remove(i);
            }
        }
    }

    /// Poistaa kuninkaan, joka lähtee yksin.
    pub fn poista_kuningas(&mut self, paikka: Paikka) -> bool {
        if self.pelattava(paikka).is_none_or(|k| k.arvo != KUNINGAS) {
            return false;
        }
        self.muista();
        self.ota_pois(paikka);
        self.tila.siirrot += 1;
        true
    }

    /// Poistaa kaksi korttia, joiden summa on 13.
    pub fn poista_pari(&mut self, a: Paikka, b: Paikka) -> bool {
        let (Some(eka), Some(toka)) = (self.pelattava(a), self.pelattava(b)) else {
            return false;
        };
        if a == b || eka.arvo + toka.arvo != SUMMA {
            return false;
        }
        self.muista();
        // Syvempi jätepinon kortti ensin, jotta päällimmäisen paikka säilyy.
        let syvempi_ensin = matches!((a, b), (Paikka::Jate(x), Paikka::Jate(y)) if x < y);
        let (ensin, sitten) = if syvempi_ensin { (b, a) } else { (a, b) };
        self.ota_pois(ensin);
        self.ota_pois(sitten);
        self.tila.siirrot += 1;
        true
    }

    pub fn on_voitettu(&self) -> bool {
        self.tila.rivit.iter().flatten().all(Option::is_none)
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

    /// Voiko vielä poistaa jotain. Peitetyt kortit eivät auta, koska ne vapautuvat vasta
    /// poistosta, ja pakan kortit käyvät vuorollaan jätepinon kahden päällimmäisen joukossa.
    pub fn on_siirtoja(&self) -> bool {
        let vapaat: Vec<Kortti> = (0..RIVEJA)
            .flat_map(|rivi| (0..=rivi).map(move |sarake| (rivi, sarake)))
            .filter(|&(rivi, sarake)| self.on_vapaa(rivi, sarake))
            .filter_map(|(rivi, sarake)| self.tila.rivit[rivi][sarake])
            .collect();
        let sopii = |kortti: &Kortti| {
            kortti.arvo == KUNINGAS || vapaat.iter().any(|v| v.arvo + kortti.arvo == SUMMA)
        };
        if vapaat.iter().any(sopii) {
            return true;
        }
        let tila = &self.tila;
        // Nostojärjestys: kierrätys kääntää jätepinon niin, että sama järjestys toistuu.
        let jarjestys: Vec<Kortti> = tila
            .jate
            .iter()
            .chain(tila.pakka.iter().rev())
            .copied()
            .collect();
        let alku = if self.voi_kierrattaa() {
            0
        } else {
            tila.jate.len().saturating_sub(JATTEEN_NAKYVAT)
        };
        let tulevat = &jarjestys[alku..];
        tulevat.iter().any(sopii)
            || tulevat
                .windows(JATTEEN_NAKYVAT)
                .any(|pari| pari[0].arvo + pari[1].arvo == SUMMA)
    }

    fn muista(&mut self) {
        self.historia.push(self.tila.clone());
    }
}
