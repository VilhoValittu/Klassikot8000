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
mod tests {
    use super::*;
    use crate::pelit::pasianssi::peli::{Nappi, Vaihe};
    use crate::yhteinen::kieli::kaanna;
    use crate::yhteinen::syote::Syote;
    use crate::yhteinen::tulokset::Tulos;
    use klassikot8000::kortit::{Kortti, Maa};
    use klassikot8000::pasianssi::{Sarake, Variantti};
    use klassikot8000::spider::SARJOJA;

    fn k(maa: Maa, arvo: u8) -> Kortti {
        Kortti { maa, arvo }
    }

    fn pelissa() -> Pasianssi {
        let mut peli = Pasianssi::new(1);
        peli.variantti = Variantti::Spider;
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.vaihe, Vaihe::Pelissa);
        peli.spider.tila.pakka.clear();
        peli.spider.tila.sarakkeet = Default::default();
        peli
    }

    fn aseta(peli: &mut Pasianssi, sarake: usize, kortit: &[Kortti]) {
        peli.spider.tila.sarakkeet[sarake] = Sarake {
            kortit: kortit.to_vec(),
            piilossa: 0,
        };
    }

    fn kortin_keskipiste(peli: &Pasianssi, sarake: usize, kortti: usize) -> (i32, i32) {
        let (x, y) = piirto::kortin_paikka(&peli.spider, Jono { sarake, kortti });
        (x + 20, y + 10)
    }

    #[test]
    fn veto_ja_klikkaus_siirtavat_jonon() {
        let mut peli = pelissa();
        aseta(&mut peli, 0, &[k(Maa::Pata, 6), k(Maa::Pata, 5)]);
        aseta(&mut peli, 1, &[k(Maa::Hertta, 7)]);
        let (x, y) = kortin_keskipiste(&peli, 0, 0);
        peli.kasittele(Syote::Klikkaus(x, y));
        assert!(peli.spider_pito.is_some());
        let (kx, ky) = kortin_keskipiste(&peli, 1, 0);
        peli.kasittele(Syote::Hiiri(kx, ky));
        peli.kasittele(Syote::Vapautus(kx, ky));
        assert_eq!(peli.spider.tila.sarakkeet[1].kortit.len(), 3);
        assert!(peli.spider_pito.is_none());

        let mut peli = pelissa();
        aseta(&mut peli, 0, &[k(Maa::Pata, 6)]);
        aseta(&mut peli, 1, &[k(Maa::Hertta, 7)]);
        let (x, y) = kortin_keskipiste(&peli, 0, 0);
        peli.kasittele(Syote::Klikkaus(x, y));
        peli.kasittele(Syote::Vapautus(x, y));
        assert!(peli.spider_valittu.is_some());
        let (kx, ky) = kortin_keskipiste(&peli, 1, 0);
        peli.kasittele(Syote::Klikkaus(kx, ky));
        assert_eq!(peli.spider.tila.sarakkeet[1].kortit.len(), 2);
    }

    #[test]
    fn laiton_pudotus_ahkaisee_ja_rikkinainen_jono_ei_lahde() {
        let mut peli = pelissa();
        aseta(&mut peli, 0, &[k(Maa::Pata, 6)]);
        aseta(&mut peli, 1, &[k(Maa::Hertta, 9)]);
        let (x, y) = kortin_keskipiste(&peli, 0, 0);
        peli.kasittele(Syote::Klikkaus(x, y));
        let (kx, ky) = kortin_keskipiste(&peli, 1, 0);
        peli.kasittele(Syote::Hiiri(kx, ky));
        peli.kasittele(Syote::Vapautus(kx, ky));
        assert_eq!(peli.spider.tila.sarakkeet[0].kortit.len(), 1);
        assert_eq!(peli.viesti, kaanna(EI_KAY));

        let mut peli = pelissa();
        aseta(&mut peli, 0, &[k(Maa::Hertta, 6), k(Maa::Pata, 5)]);
        let (x, y) = kortin_keskipiste(&peli, 0, 0);
        peli.kasittele(Syote::Klikkaus(x, y));
        assert!(peli.spider_pito.is_none());
    }

    #[test]
    fn jako_toimii_pakasta_napista_ja_n_nappaimesta() {
        let mut peli = Pasianssi::new(1);
        peli.variantti = Variantti::Spider;
        peli.kasittele(Syote::Ammu);
        let (x, y) = piirto::pakan_keskipiste();
        peli.kasittele(Syote::Klikkaus(x, y));
        assert_eq!(peli.spider.jakoja(), 4);
        peli.kasittele(Syote::Kirjain('N'));
        assert_eq!(peli.spider.jakoja(), 3);
        let (x, y) = piirto::napin_keskipiste(Nappi::Nosta);
        peli.kasittele(Syote::Klikkaus(x, y));
        assert_eq!(peli.spider.jakoja(), 2);

        peli.spider.tila.sarakkeet[3].kortit.clear();
        peli.spider.tila.sarakkeet[3].piilossa = 0;
        peli.kasittele(Syote::Kirjain('N'));
        assert_eq!(peli.spider.jakoja(), 2);
        assert_eq!(peli.viesti, kaanna(EI_JAKOA));
    }

    #[test]
    fn nappaimilla_poimitaan_ja_siirretaan_jono() {
        let mut peli = pelissa();
        aseta(&mut peli, 0, &[k(Maa::Pata, 6), k(Maa::Pata, 5)]);
        aseta(&mut peli, 1, &[k(Maa::Hertta, 7)]);
        peli.kasittele(Syote::Nuoli(-1, 0));
        peli.kasittele(Syote::Ammu);
        assert_eq!(
            peli.spider_valittu,
            Some(Jono {
                sarake: 0,
                kortti: 0
            })
        );
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.spider_valittu, None);
        peli.kasittele(Syote::Ammu);
        peli.kasittele(Syote::Nuoli(0, 1));
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.spider.tila.sarakkeet[1].kortit.len(), 3);
        assert_eq!(peli.kursori.syvyys, 0);
    }

    #[test]
    fn peruutus_toimii_vaikeallakin_ja_viimeinen_sarja_voittaa() {
        let mut peli = Pasianssi::new(1);
        peli.variantti = Variantti::Spider;
        peli.vaikeus = klassikot8000::pasianssi::Vaikeus::Vaikea;
        peli.kasittele(Syote::Ammu);
        peli.kasittele(Syote::Kirjain('N'));
        peli.kasittele(Syote::Kirjain('U'));
        assert_eq!(peli.spider.jakoja(), 5);

        let mut peli = pelissa();
        peli.spider.tila.valmiit = vec![Maa::Pata; SARJOJA - 1];
        let sarja: Vec<Kortti> = (2..=13).rev().map(|arvo| k(Maa::Pata, arvo)).collect();
        aseta(&mut peli, 0, &sarja);
        aseta(&mut peli, 1, &[k(Maa::Pata, 1)]);
        peli.edista(2000);
        let (x, y) = kortin_keskipiste(&peli, 1, 0);
        peli.kasittele(Syote::Klikkaus(x, y));
        peli.kasittele(Syote::Vapautus(x, y));
        let (kx, ky) = kortin_keskipiste(&peli, 0, 11);
        peli.kasittele(Syote::Klikkaus(kx, ky));
        assert_eq!(peli.vaihe, Vaihe::Nimi);

        // Tulos menee Spiderin omalle listalle.
        peli.kasittele(Syote::Kirjain('S'));
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.vaihe, Vaihe::Voitto);
        let oma = &peli.parhaat[Variantti::Spider as usize];
        assert_eq!(
            oma.iter().map(|t: &Tulos| &t.nimi[..]).collect::<Vec<_>>(),
            ["S"]
        );
        assert!(peli.parhaat[Variantti::Klondike as usize].is_empty());
    }

    #[test]
    fn peli_loppuu_kun_siirrot_loppuvat_ja_kello_pysahtyy() {
        let mut peli = pelissa();
        // Vain viitosen siirto kuutosen päälle on mahdollinen, ja sen jälkeen mikään ei käy.
        for i in 2..SARAKKEITA {
            aseta(&mut peli, i, &[k(Maa::Pata, 1), k(Maa::Hertta, 13)]);
        }
        aseta(
            &mut peli,
            0,
            &[k(Maa::Pata, 1), k(Maa::Hertta, 9), k(Maa::Pata, 5)],
        );
        aseta(&mut peli, 1, &[k(Maa::Pata, 1), k(Maa::Risti, 6)]);
        peli.edista(1000);
        let (x, y) = kortin_keskipiste(&peli, 0, 2);
        peli.kasittele(Syote::Klikkaus(x, y));
        let (kx, ky) = kortin_keskipiste(&peli, 1, 1);
        peli.kasittele(Syote::Hiiri(kx, ky));
        peli.kasittele(Syote::Vapautus(kx, ky));
        assert_eq!(peli.spider.tila.sarakkeet[1].kortit.len(), 3);
        assert_eq!(peli.vaihe, Vaihe::Tappio);
        assert_eq!(peli.pisteet(), 0);
        peli.edista(1000);
        assert_eq!(peli.peliaika_ms, 1000);
        peli.kasittele(Syote::Kirjain('K'));
        assert_eq!(peli.vaihe, Vaihe::Alku);
    }
}
