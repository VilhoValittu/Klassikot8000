use std::fmt;
use std::str::FromStr;

use super::{KOKO, SARAKKEET};

/// Laudan ruutu, rivi ja sarake alkavat nollasta.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Ruutu {
    pub rivi: u8,
    pub sarake: u8,
}

impl Ruutu {
    pub fn new(rivi: usize, sarake: usize) -> Option<Self> {
        (rivi < KOKO && sarake < KOKO).then_some(Self {
            rivi: rivi as u8,
            sarake: sarake as u8,
        })
    }

    fn bitti(self) -> u128 {
        1 << (self.rivi as usize * KOKO + self.sarake as usize)
    }
}

impl fmt::Display for Ruutu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kirjain = SARAKKEET.as_bytes()[self.sarake as usize] as char;
        write!(f, "{}{}", kirjain, self.rivi + 1)
    }
}

/// Jäsentää muodon "B10". Rivinumeron etunollat hylätään kuten alkuperäisessä pelissä.
impl FromStr for Ruutu {
    type Err = ();

    fn from_str(teksti: &str) -> Result<Self, ()> {
        let mut merkit = teksti.chars();
        let kirjain = merkit.next().ok_or(())?;
        let sarake = SARAKKEET.find(kirjain).ok_or(())?;
        let numero = merkit.as_str();
        if numero.starts_with('0') {
            return Err(());
        }
        let rivi: usize = numero.parse().map_err(|_| ())?;
        Ruutu::new(rivi.checked_sub(1).ok_or(())?, sarake).ok_or(())
    }
}

/// Ruutujoukko bittikenttänä: 100 ruutua mahtuu 16 tavuun ilman varauksia.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Ruudut(u128);

impl Ruudut {
    pub fn sisaltaa(self, ruutu: Ruutu) -> bool {
        self.0 & ruutu.bitti() != 0
    }

    pub fn lisaa(&mut self, ruutu: Ruutu) {
        self.0 |= ruutu.bitti();
    }

    pub fn yhdiste(self, muut: Ruudut) -> Ruudut {
        Ruudut(self.0 | muut.0)
    }

    pub fn leikkaus(self, muut: Ruudut) -> Ruudut {
        Ruudut(self.0 & muut.0)
    }

    pub fn erotus(self, muut: Ruudut) -> Ruudut {
        Ruudut(self.0 & !muut.0)
    }

    /// Joukon ruudut ja kaikki niiden naapurit, myös vinottaiset.
    pub fn ymparys(self) -> Ruudut {
        let mut tulos = self;
        for rivi in 0..KOKO {
            for sarake in 0..KOKO {
                let Some(ruutu) = Ruutu::new(rivi, sarake) else {
                    continue;
                };
                if !self.sisaltaa(ruutu) {
                    continue;
                }
                for r in rivi.saturating_sub(1)..=rivi + 1 {
                    for s in sarake.saturating_sub(1)..=sarake + 1 {
                        if let Some(naapuri) = Ruutu::new(r, s) {
                            tulos.lisaa(naapuri);
                        }
                    }
                }
            }
        }
        tulos
    }

    pub fn leikkaako(self, muut: Ruudut) -> bool {
        self.0 & muut.0 != 0
    }

    pub fn on_osajoukko(self, muut: Ruudut) -> bool {
        self.0 & !muut.0 == 0
    }

    pub fn maara(self) -> usize {
        self.0.count_ones() as usize
    }

    /// Ylimmän rivin vasemmanpuoleisin ruutu: suoran laivan alku.
    pub fn ensimmainen(self) -> Option<Ruutu> {
        let i = self.0.trailing_zeros() as usize;
        Ruutu::new(i / KOKO, i % KOKO)
    }

    /// Suora jono ruutuja alkaen `alku`sta oikealle tai alas. `None`, jos jono ei mahdu laudalle.
    pub fn suora(alku: Ruutu, pituus: usize, vaaka: bool) -> Option<Ruudut> {
        let (rivi, sarake) = (alku.rivi as usize, alku.sarake as usize);
        let mut ruudut = Ruudut::default();
        for i in 0..pituus {
            let ruutu = if vaaka {
                Ruutu::new(rivi, sarake + i)
            } else {
                Ruutu::new(rivi + i, sarake)
            };
            ruudut.lisaa(ruutu?);
        }
        Some(ruudut)
    }
}

#[cfg(test)]
mod tests;
