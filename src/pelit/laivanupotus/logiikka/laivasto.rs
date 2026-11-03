use super::vastustaja::Tiedot;
use super::{KOKO, LAIVAT, Ruudut, Ruutu};
use crate::Arpa;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Laukaustulos {
    Ohi,
    Osuma,
    Upposi(&'static str),
}

#[derive(Clone, Copy, Debug)]
pub struct Laiva {
    pub nimi: &'static str,
    pub ruudut: Ruudut,
}

impl Laiva {
    /// Ylin tai vasemmanpuoleisin ruutu.
    pub fn alku(&self) -> Option<Ruutu> {
        self.ruudut.ensimmainen()
    }

    pub fn vaaka(&self) -> bool {
        self.alku().is_some_and(|alku| {
            let seuraava = Ruutu::new(alku.rivi as usize, alku.sarake as usize + 1);
            seuraava.is_some_and(|ruutu| self.ruudut.sisaltaa(ruutu))
        })
    }
}

/// Yhden pelaajan laivat ja niihin ammutut laukaukset.
#[derive(Clone, Debug)]
pub struct Laivasto {
    laivat: [Laiva; LAIVAT.len()],
    ammutut: Ruudut,
}

impl Laivasto {
    /// Sijoittaa laivat satunnaisesti. Laivat eivät saa koskettaa toisiaan edes kulmittain,
    /// koska vierekkäiset laivat näyttäisivät laudalla yhdeltä kasalta.
    pub fn satunnainen(arpa: &mut Arpa) -> Self {
        let mut varatut = Ruudut::default();
        let laivat = LAIVAT.map(|(nimi, pituus)| {
            let ruudut = loop {
                let ruudut = arvo_paikka(pituus, arpa);
                if !ruudut.leikkaako(varatut) {
                    break ruudut;
                }
            };
            varatut = varatut.yhdiste(ruudut.ymparys());
            Laiva { nimi, ruudut }
        });
        Self {
            laivat,
            ammutut: Ruudut::default(),
        }
    }

    pub fn laivat(&self) -> &[Laiva] {
        &self.laivat
    }

    pub fn ammutut(&self) -> Ruudut {
        self.ammutut
    }

    /// Laudasta se osa, jonka ampuja on saanut laukauksillaan selville.
    pub fn tiedot(&self) -> Tiedot {
        let mut tiedot = Tiedot {
            ammutut: self.ammutut,
            ..Tiedot::default()
        };
        for laiva in &self.laivat {
            if self.on_upotettu(laiva) {
                tiedot.upotetut = tiedot.upotetut.yhdiste(laiva.ruudut);
            } else {
                let osumat = laiva.ruudut.leikkaus(self.ammutut);
                tiedot.osumat = tiedot.osumat.yhdiste(osumat);
                tiedot.jaljella.push(laiva.ruudut.maara());
            }
        }
        tiedot
    }

    pub fn on_laiva(&self, ruutu: Ruutu) -> bool {
        self.laivat.iter().any(|l| l.ruudut.sisaltaa(ruutu))
    }

    /// Laiva, johon ruutu kuuluu, järjestysnumerona.
    pub fn laiva_ruudussa(&self, ruutu: Ruutu) -> Option<usize> {
        self.laivat.iter().position(|l| l.ruudut.sisaltaa(ruutu))
    }

    /// Mahtuuko laiva `i` ruutuihin koskematta muihin laivoihin, ei edes kulmittain.
    pub fn mahtuu(&self, i: usize, ruudut: Ruudut) -> bool {
        self.laivat
            .iter()
            .enumerate()
            .all(|(j, muu)| j == i || !muu.ruudut.ymparys().leikkaako(ruudut))
    }

    /// Siirtää laivan `i` uuteen paikkaan ennen ensimmäistä laukausta. Epäkelpo paikka
    /// palauttaa epätoden, eikä mikään muutu.
    pub fn siirra(&mut self, i: usize, ruudut: Ruudut) -> bool {
        let kelpaa = self.ammutut == Ruudut::default()
            && self
                .laivat
                .get(i)
                .is_some_and(|laiva| laiva.ruudut.maara() == ruudut.maara())
            && self.mahtuu(i, ruudut);
        if kelpaa {
            self.laivat[i].ruudut = ruudut;
        }
        kelpaa
    }

    /// Palauttaa `None`, jos ruutuun on jo ammuttu.
    pub fn kasittele_laukaus(&mut self, ruutu: Ruutu) -> Option<Laukaustulos> {
        if self.ammutut.sisaltaa(ruutu) {
            return None;
        }
        self.ammutut.lisaa(ruutu);
        let Some(laiva) = self.laivat.iter().find(|l| l.ruudut.sisaltaa(ruutu)) else {
            return Some(Laukaustulos::Ohi);
        };
        // laiva uppoaa kun kaikkiin sen ruutuihin on osunut
        Some(if laiva.ruudut.on_osajoukko(self.ammutut) {
            Laukaustulos::Upposi(laiva.nimi)
        } else {
            Laukaustulos::Osuma
        })
    }

    pub fn on_upotettu(&self, laiva: &Laiva) -> bool {
        laiva.ruudut.on_osajoukko(self.ammutut)
    }

    pub fn upotettuja(&self) -> usize {
        self.laivat.iter().filter(|l| self.on_upotettu(l)).count()
    }

    pub fn kaikki_upotettu(&self) -> bool {
        self.upotettuja() == self.laivat.len()
    }
}

fn arvo_paikka(pituus: usize, arpa: &mut Arpa) -> Ruudut {
    let (drivi, dsarake) = if arpa.alle(2) == 0 { (0, 1) } else { (1, 0) };
    let rivi = arpa.alle(KOKO - drivi * (pituus - 1));
    let sarake = arpa.alle(KOKO - dsarake * (pituus - 1));
    let mut ruudut = Ruudut::default();
    for i in 0..pituus {
        if let Some(ruutu) = Ruutu::new(rivi + drivi * i, sarake + dsarake * i) {
            ruudut.lisaa(ruutu);
        }
    }
    ruudut
}
