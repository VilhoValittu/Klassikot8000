use klassikot8000::Arpa;
use klassikot8000::tiilirikko::{ASKEL_MS, Tapahtuma, Tiilipeli, Tila};

use super::piirto;
use crate::yhteinen::aanet::{Aanet, Aani, PUHE_VIIVE_MS};
use crate::yhteinen::kieli::{kaanna, muotoile};
use crate::yhteinen::loppunapit::{self, Loppunappi};
use crate::yhteinen::pelinakyma::Pelinakyma;
use crate::yhteinen::piirto::{Kehys, aanikytkimella};
use crate::yhteinen::syote::{Syote, Yleinen, yleinen};
use crate::yhteinen::tulokset::{Nettipeli, Tulostaulu, nimi_valmis};

pub const KIPINA_MS: u32 = 350;
// Ikkunan raahaus voi pysäyttää silmukan: pallo ei saa sen jälkeen hypätä kerralla pitkälle.
const SUURIN_KURO_MS: u32 = 100;
const LOPPU_VIIVE_MS: u32 = 250;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vaihe {
    Alku,
    Pelissa,
    Tauko,
    Voitto,
    Tappio,
    /// Tulos pääsi kolmen parhaan joukkoon ja odottaa nimimerkkiä.
    Nimi,
}

pub struct Tiilirikko {
    pub vaihe: Vaihe,
    pub lauta: Tiilipeli,
    pub taulu: Tulostaulu,
    /// Kirjoitettavana oleva nimimerkki.
    pub nimi: String,
    /// Tosi, kun päättyneen pelin ennätys odottaa tiedostoon kirjoitusta.
    pub uusi_tulos: bool,
    pub viesti: String,
    /// Rikkoutuneet tiilet (rivi, sarake, alkuhetki) tehostetta varten.
    pub kipinat: Vec<(usize, usize, u32)>,
    /// Pyyntö palata käynnistimeen.
    pub lopeta: bool,
    pub aanet: Aanet,
    pub kello_ms: u32,
    /// Pohjassa olevien nuolten suunta: -1, 0 tai 1.
    pub ohjaus: i8,
    /// Pelaaja poistuu kesken pelin: nimen jälkeen palataan valikkoon.
    poistuu: bool,
    kertyma_ms: u32,
    /// Vaihe, johon nimimerkin jälkeen palataan: voitto tai tappio.
    loppu: Vaihe,
    arpa: Arpa,
}

impl Tiilirikko {
    pub fn new(siemen: u64) -> Self {
        let mut arpa = Arpa::new(siemen);
        Self {
            vaihe: Vaihe::Alku,
            lauta: Tiilipeli::new(0, &mut arpa),
            taulu: Tulostaulu::new(Nettipeli::Tiilirikko),
            nimi: String::new(),
            uusi_tulos: false,
            viesti: String::new(),
            kipinat: Vec::new(),
            lopeta: false,
            aanet: Aanet::default(),
            kello_ms: 0,
            ohjaus: 0,
            poistuu: false,
            kertyma_ms: 0,
            loppu: Vaihe::Tappio,
            arpa,
        }
    }

    pub fn edista(&mut self, ms: u32) {
        self.kello_ms = self.kello_ms.saturating_add(ms);
        let kello = self.kello_ms;
        self.kipinat
            .retain(|&(_, _, alku)| kello < alku.saturating_add(KIPINA_MS));
        if self.vaihe != Vaihe::Pelissa {
            return;
        }
        self.kertyma_ms = (self.kertyma_ms + ms).min(SUURIN_KURO_MS);
        while self.kertyma_ms >= ASKEL_MS && self.vaihe == Vaihe::Pelissa {
            self.kertyma_ms -= ASKEL_MS;
            for tapahtuma in self.lauta.askel(self.ohjaus, &mut self.arpa) {
                self.kasittele_tapahtuma(tapahtuma);
            }
        }
    }

    fn kasittele_tapahtuma(&mut self, tapahtuma: Tapahtuma) {
        self.taulu.paras = self.taulu.paras.max(self.lauta.pisteet);
        match tapahtuma {
            Tapahtuma::Seina | Tapahtuma::Maila => self.aanet.soita(Aani::Pomppu),
            Tapahtuma::Tiili {
                rivi,
                sarake,
                rikki,
            } => {
                self.aanet.soita(Aani::Tiili);
                if rikki {
                    self.kipinat.push((rivi, sarake, self.kello_ms));
                }
            }
            Tapahtuma::UusiPallo => {
                self.aanet.soita(Aani::Yhdistys);
                self.viesti = kaanna("tiilirikko.uusi_pallo").to_string();
            }
            Tapahtuma::Superpallo => {
                self.aanet.soita(Aani::Yhdistys);
                self.viesti = kaanna("tiilirikko.superpallo").to_string();
            }
            Tapahtuma::ToinenMeni => {
                self.aanet.soita(Aani::Puhkus);
                self.viesti = kaanna("tiilirikko.toinen_meni").to_string();
            }
            Tapahtuma::Menetys => {
                self.aanet.soita(Aani::Rajahdys);
                self.viesti = muotoile("tiilirikko.pallo_meni", &[&self.lauta.elamat]);
            }
            Tapahtuma::KenttaLapi { bonus } => {
                self.aanet.soita(Aani::Yhdistys);
                self.viesti = muotoile("tiilirikko.kentta_lapi", &[&bonus]);
            }
            Tapahtuma::Voitto { bonus } => {
                self.aanet.soita_viiveella(Aani::Voitto, LOPPU_VIIVE_MS);
                self.aanet
                    .soita_viiveella(Aani::Voitit, LOPPU_VIIVE_MS + PUHE_VIIVE_MS);
                self.lopeta_peli(Vaihe::Voitto);
                self.viesti = muotoile("tiilirikko.loppubonus", &[&bonus, &self.lauta.pisteet]);
            }
            Tapahtuma::Ohi => {
                self.aanet.soita_viiveella(Aani::Tappio, LOPPU_VIIVE_MS);
                self.aanet
                    .soita_viiveella(Aani::PeliOhi, LOPPU_VIIVE_MS + PUHE_VIIVE_MS);
                self.lopeta_peli(Vaihe::Tappio);
            }
        }
    }

    fn lopeta_peli(&mut self, vaihe: Vaihe) {
        self.loppu = vaihe;
        self.vaihe = if self.taulu.mahtuu(self.lauta.pisteet) {
            self.nimi.clear();
            Vaihe::Nimi
        } else {
            vaihe
        };
        self.ohjaus = 0;
        self.uusi_tulos = true;
        self.viesti = muotoile("yhteinen.tulos", &[&self.lauta.pisteet]);
    }

    /// Kirjaa odottavan tuloksen.
    pub fn vahvista_nimi(&mut self) {
        if self.vaihe != Vaihe::Nimi {
            return;
        }
        let pisteet = self.lauta.pisteet;
        self.taulu.kirjaa(&mut self.nimi, pisteet);
        self.uusi_tulos = true;
        self.vaihe = self.loppu;
        self.lopeta = self.poistuu;
    }

    /// Esc tai ikkunan sulkeminen kesken pelin: listalle yltävä tulos ei saa kadota.
    pub fn poistu(&mut self) {
        let kesken = matches!(self.vaihe, Vaihe::Pelissa | Vaihe::Tauko);
        if kesken && self.taulu.mahtuu(self.lauta.pisteet) {
            self.nimi.clear();
            self.poistuu = true;
            self.vaihe = Vaihe::Nimi;
        } else {
            self.lopeta = true;
        }
    }

    fn aloita_alusta(&mut self) {
        self.lauta = Tiilipeli::new(0, &mut self.arpa);
        self.vaihe = Vaihe::Pelissa;
        self.viesti.clear();
        self.kipinat.clear();
        self.kertyma_ms = 0;
        self.aanet.soita(Aani::Alku);
    }

    /// Ikkuna menetti kohdistuksen: lennossa oleva pallo ei saa kadota pelaajan poissa ollessa.
    pub fn keskeyta(&mut self) {
        if self.vaihe == Vaihe::Pelissa && self.lauta.tila == Tila::Lennossa {
            self.vaihe = Vaihe::Tauko;
        }
    }

    pub fn kasittele(&mut self, syote: Syote) {
        match yleinen(
            syote,
            self.vaihe == Vaihe::Nimi,
            &mut self.aanet.paalla,
            aanikytkimella,
        ) {
            Yleinen::Peli => {}
            Yleinen::Nimi => {
                if nimi_valmis(&mut self.nimi, syote) {
                    self.vahvista_nimi();
                }
                return;
            }
            // Kesken pelin Esc pysäyttää ensin, jottei vahinkopainallus hävitä peliä.
            Yleinen::Peru if self.vaihe == Vaihe::Pelissa => {
                self.vaihe = Vaihe::Tauko;
                return;
            }
            Yleinen::Peru => return self.poistu(),
            Yleinen::Aanet | Yleinen::Ohita => return,
        }
        match self.vaihe {
            Vaihe::Alku => {
                if matches!(syote, Syote::Ammu | Syote::Klikkaus(..)) {
                    self.aloita_alusta();
                }
            }
            Vaihe::Pelissa => match syote {
                Syote::Hiiri(x, _) => self.lauta.aseta_maila((x - piirto::KENTTA_X) as f32),
                Syote::Ammu | Syote::Klikkaus(..) if self.lauta.tila == Tila::Kiinni => {
                    self.viesti.clear();
                    self.lauta.laukaise(&mut self.arpa);
                }
                Syote::Kirjain('P') => self.vaihe = Vaihe::Tauko,
                _ => {}
            },
            Vaihe::Tauko => {
                if matches!(
                    syote,
                    Syote::Ammu | Syote::Klikkaus(..) | Syote::Kirjain('P')
                ) {
                    self.vaihe = Vaihe::Pelissa;
                }
            }
            Vaihe::Voitto | Vaihe::Tappio => self.kasittele_loppu(syote),
            Vaihe::Nimi => {}
        }
    }

    fn kasittele_loppu(&mut self, syote: Syote) {
        match loppunapit::valinta(syote) {
            Some(Loppunappi::Uudestaan) => self.aloita_alusta(),
            Some(Loppunappi::Valikko) => self.lopeta = true,
            None => {}
        }
    }
}

impl Pelinakyma for Tiilirikko {
    fn kasittele(&mut self, syote: Syote) {
        Tiilirikko::kasittele(self, syote);
    }

    fn edista(&mut self, ms: u32) {
        Tiilirikko::edista(self, ms);
    }

    fn piirra(&self, kehys: &mut Kehys) {
        piirto::piirra(self, &mut kehys.kuva);
        kehys.venyta();
    }

    fn aanet(&mut self) -> &mut Aanet {
        &mut self.aanet
    }

    fn lopeta(&self) -> bool {
        self.lopeta
    }

    fn ota_uusi_tulos(&mut self) -> bool {
        std::mem::take(&mut self.uusi_tulos)
    }

    fn viimeistele(&mut self) {
        self.poistu();
        self.vahvista_nimi();
    }

    fn listat(&mut self) -> Vec<&mut Tulostaulu> {
        vec![&mut self.taulu]
    }

    fn toimintapeli(&self) -> bool {
        self.vaihe != Vaihe::Alku
    }

    fn aseta_ohjaus(&mut self, suunta: i8, _ylos: bool) {
        self.ohjaus = suunta;
    }

    fn keskeyta(&mut self) {
        Self::keskeyta(self);
    }

    fn hiiri_ulkonakin(&self) -> bool {
        self.vaihe == Vaihe::Pelissa
    }
}

#[cfg(test)]
mod tests;
