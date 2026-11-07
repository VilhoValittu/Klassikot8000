use klassikot8000::kortit::KUNINGAS;
use klassikot8000::pyramidi::{JATTEEN_NAKYVAT, Paikka, RIVEJA};

use super::piirto::{self, Kohta};
use crate::pelit::pasianssi::peli::{Pasianssi, Pito};
use crate::yhteinen::aanet::Aani;

const EI_PARIA: &str = "pyramidi.ei_paria";
// Ylärivillä osoitin on pakan ja jätepinon kahden näkyvän kortin kohdalla.
const YLARIVIN_PAIKKOJA: usize = 1 + JATTEEN_NAKYVAT;

impl Pasianssi {
    /// Poistaa kuninkaan heti, muut kortit jäävät odottamaan paria.
    fn pyramidi_poimi(&mut self, paikka: Paikka) -> bool {
        let kuningas = self
            .pyramidi
            .pelattava(paikka)
            .is_some_and(|kortti| kortti.arvo == KUNINGAS);
        if kuningas && self.pyramidi.poista_kuningas(paikka) {
            self.siirto_tehty(Aani::Yhdistys);
        }
        kuningas
    }

    fn pyramidi_yrita_paria(&mut self, a: Paikka, b: Paikka) -> bool {
        let poistui = self.pyramidi.poista_pari(a, b);
        if poistui {
            self.siirto_tehty(Aani::Yhdistys);
        } else {
            self.virhe(EI_PARIA);
        }
        poistui
    }

    pub(in crate::pelit::pasianssi) fn pyramidi_klikkaus(&mut self, x: i32, y: i32) {
        let paikka = match piirto::kohta_pisteesta(&self.pyramidi, x, y) {
            Some(Kohta::Pakka) => {
                self.nosta();
                return;
            }
            Some(Kohta::Paikka(paikka)) => Some(paikka),
            None => None,
        };
        let valittu = self.pyramidi_valittu.take();
        let Some(paikka) = paikka else {
            return;
        };
        if self.pyramidi_poimi(paikka) {
            return;
        }
        if let Some(valittu) = valittu {
            // Toinen klikkaus samaan korttiin vain peruu valinnan.
            if valittu == paikka || self.pyramidi_yrita_paria(valittu, paikka) {
                return;
            }
        }
        let kulma = piirto::kortin_paikka(&self.pyramidi, paikka);
        self.pyramidi_pito = Some(Pito::new(paikka, kulma, (x, y)));
    }

    pub(in crate::pelit::pasianssi) fn pyramidi_vapautus(&mut self, x: i32, y: i32) {
        let Some(pito) = self.pyramidi_pito.take() else {
            return;
        };
        if !pito.liikkui {
            self.pyramidi_valittu = Some(pito.lahde);
            return;
        }
        if let Some(Kohta::Paikka(kohde)) = piirto::kohta_pisteesta(&self.pyramidi, x, y)
            && kohde != pito.lahde
        {
            self.pyramidi_yrita_paria(pito.lahde, kohde);
        }
    }

    pub(in crate::pelit::pasianssi) fn pyramidi_liikuta_kursoria(
        &mut self,
        drivi: i8,
        dsarake: i8,
    ) {
        // `syvyys` on tässä pelissä pyramidin rivi.
        let k = &mut self.kursori;
        let vasen = k.sarake.saturating_add_signed(dsarake as isize);
        k.sarake = if k.ylarivi {
            vasen.min(YLARIVIN_PAIKKOJA - 1)
        } else {
            vasen.min(k.syvyys)
        };
        if drivi > 0 {
            if k.ylarivi {
                k.ylarivi = false;
                k.syvyys = 0;
                k.sarake = 0;
            } else if k.syvyys + 1 < RIVEJA {
                k.syvyys += 1;
            }
        } else if drivi < 0 && !k.ylarivi {
            if k.syvyys == 0 {
                k.ylarivi = true;
                k.sarake = 0;
            } else {
                k.syvyys -= 1;
                k.sarake = k.sarake.min(k.syvyys);
            }
        }
    }

    pub fn pyramidi_kursorin_kohta(&self) -> Kohta {
        let k = self.kursori;
        if !k.ylarivi {
            return Kohta::Paikka(Paikka::Pyramidi {
                rivi: k.syvyys,
                sarake: k.sarake,
            });
        }
        match k.sarake {
            0 => Kohta::Pakka,
            // Päällimmäinen kortti on oikealla, kuten ruudulla.
            s => Kohta::Paikka(Paikka::Jate(YLARIVIN_PAIKKOJA - 1 - s)),
        }
    }

    pub(in crate::pelit::pasianssi) fn pyramidi_kursorin_toiminto(&mut self) {
        let paikka = match self.pyramidi_kursorin_kohta() {
            Kohta::Pakka => {
                self.nosta();
                return;
            }
            Kohta::Paikka(paikka) => paikka,
        };
        if self.pyramidi.pelattava(paikka).is_none() {
            return;
        }
        let valittu = self.pyramidi_valittu.take();
        if self.pyramidi_poimi(paikka) {
            return;
        }
        match valittu {
            Some(valittu) if valittu == paikka => {}
            Some(valittu) => {
                self.pyramidi_yrita_paria(valittu, paikka);
            }
            None => self.pyramidi_valittu = Some(paikka),
        }
    }
}

#[cfg(test)]
mod tests;
