use klassikot8000::spider::{Jono, SARAKKEITA};

use super::piirto::{self, Kohta};
use crate::pelit::pasianssi::peli::{EI_KAY, PAKKA_LOPPU, Pasianssi, Pito};
use crate::yhteinen::aanet::Aani;

const EI_JAKOA: &str = "spider.ei_jakoa";

impl Pasianssi {
    pub(in crate::pelit::pasianssi) fn spider_jaa(&mut self) {
        if self.spider.jaa() {
            self.siirto_tehty(Aani::Liuku);
        } else if self.spider.tila.pakka.is_empty() {
            self.virhe(PAKKA_LOPPU);
        } else {
            self.virhe(EI_JAKOA);
        }
    }

    fn spider_yrita_siirtoa(&mut self, jono: Jono, kohde: usize) -> bool {
        let valmiita = self.spider.tila.valmiit.len();
        if !self.spider.siirra(jono, kohde) {
            return false;
        }
        let aani = if self.spider.tila.valmiit.len() > valmiita {
            Aani::Yhdistys
        } else {
            Aani::Liuku
        };
        self.siirto_tehty(aani);
        true
    }

    /// Ehjä jono kohdasta, muuten ei mitään poimittavaa.
    fn spider_jono(&self, kohta: Option<Kohta>) -> Option<Jono> {
        let Some(Kohta::Sarake { sarake, kortti }) = kohta else {
            return None;
        };
        let jono = Jono {
            sarake,
            kortti: kortti?,
        };
        self.spider.siirrettavat(jono).map(|_| jono)
    }

    pub(in crate::pelit::pasianssi) fn spider_klikkaus(&mut self, x: i32, y: i32) {
        let kohta = piirto::kohta_pisteesta(&self.spider, x, y);
        if kohta == Some(Kohta::Pakka) {
            self.nosta();
            return;
        }
        let jono = self.spider_jono(kohta);
        if let Some(valittu) = self.spider_valittu.take()
            && let Some(kohde) = piirto::kohde_pisteesta(x, y)
        {
            if self.spider_yrita_siirtoa(valittu, kohde) {
                return;
            }
            if jono.is_none() || jono == Some(valittu) {
                if jono.is_none() {
                    self.virhe(EI_KAY);
                }
                return;
            }
        }
        if let Some(jono) = jono {
            let kulma = piirto::kortin_paikka(&self.spider, jono);
            self.spider_pito = Some(Pito::new(jono, kulma, (x, y)));
        }
    }

    pub(in crate::pelit::pasianssi) fn spider_vapautus(&mut self, x: i32, y: i32) {
        let Some(pito) = self.spider_pito.take() else {
            return;
        };
        if !pito.liikkui {
            self.spider_valittu = Some(pito.lahde);
            return;
        }
        let keski = piirto::vedetyn_keskipiste(x - pito.tartunta.0);
        let kohde = piirto::kohde_pisteesta(x, y).or_else(|| piirto::kohde_pisteesta(keski, y));
        if let Some(kohde) = kohde
            && !self.spider_yrita_siirtoa(pito.lahde, kohde)
        {
            self.virhe(EI_KAY);
        }
    }

    pub(in crate::pelit::pasianssi) fn spider_liikuta_kursoria(&mut self, drivi: i8, dsarake: i8) {
        let k = &mut self.kursori;
        if dsarake != 0 {
            k.sarake = k
                .sarake
                .saturating_add_signed(dsarake as isize)
                .min(SARAKKEITA - 1);
            k.syvyys = 0;
        }
        let sarake = &self.spider.tila.sarakkeet[k.sarake];
        let nakyvia = sarake.kortit.len() - sarake.piilossa;
        if drivi > 0 {
            k.syvyys = k.syvyys.saturating_sub(1);
        } else if drivi < 0 && k.syvyys + 1 < nakyvia {
            k.syvyys += 1;
        }
    }

    /// Osoittimen kohta. Sarake on voinut lyhentyä siirrossa, joten kortti voi puuttua.
    pub fn spider_kursorin_kohta(&self) -> Kohta {
        let k = self.kursori;
        let sarake = &self.spider.tila.sarakkeet[k.sarake];
        Kohta::Sarake {
            sarake: k.sarake,
            kortti: sarake.kortit.len().checked_sub(1 + k.syvyys),
        }
    }

    pub(in crate::pelit::pasianssi) fn spider_kursorin_toiminto(&mut self) {
        let kohta = self.spider_kursorin_kohta();
        let jono = self.spider_jono(Some(kohta));
        if let Some(valittu) = self.spider_valittu.take() {
            // Toinen painallus samaan kohtaan vain peruu valinnan.
            if jono != Some(valittu) && !self.spider_yrita_siirtoa(valittu, self.kursori.sarake) {
                self.virhe(EI_KAY);
            }
            return;
        }
        self.spider_valittu = jono;
    }
}

#[cfg(test)]
mod tests;
