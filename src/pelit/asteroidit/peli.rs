use klassikot8000::Arpa;
use klassikot8000::asteroidit::{ASKEL_MS, Avaruus, Ohjaus, Piste, Tapahtuma, Tila};

use super::piirto;
use crate::yhteinen::aanet::{Aanet, Aani, PUHE_VIIVE_MS};
use crate::yhteinen::kieli::{kaanna, muotoile};
use crate::yhteinen::loppunapit::{self, Loppunappi};
use crate::yhteinen::pelinakyma::Pelinakyma;
use crate::yhteinen::piirto::{Kehys, aanikytkimella};
use crate::yhteinen::syote::{Syote, Yleinen, yleinen};
use crate::yhteinen::tulokset::{Nettipeli, Tulostaulu, nimi_valmis};

pub const RAJAHDYS_MS: u32 = 450;
// Ikkunan raahaus voi pysäyttää silmukan: kivet eivät saa sen jälkeen hypätä kerralla pitkälle.
const SUURIN_KURO_MS: u32 = 100;
const LOPPU_VIIVE_MS: u32 = 250;
// Lyhyet äänet soivat uudelleen näin usein, joten ne kuulostavat jatkuvilta.
const MOOTTORIN_VALI_MS: u32 = 120;
const LAUTASEN_VALI_MS: u32 = 300;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vaihe {
    Alku,
    Pelissa,
    Tauko,
    Tappio,
    /// Tulos pääsi kolmen parhaan joukkoon ja odottaa nimimerkkiä.
    Nimi,
}

/// Räjähdystehoste: paikka kentällä, koko ja alkuhetki.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rajahdys {
    pub paikka: Piste,
    pub koko: f32,
    pub alku: u32,
}

pub struct Asteroidit {
    pub vaihe: Vaihe,
    pub lauta: Avaruus,
    pub taulu: Tulostaulu,
    /// Kirjoitettavana oleva nimimerkki.
    pub nimi: String,
    /// Tosi, kun päättyneen pelin ennätys odottaa tiedostoon kirjoitusta.
    pub uusi_tulos: bool,
    pub viesti: String,
    pub rajahdykset: Vec<Rajahdys>,
    /// Pyyntö palata käynnistimeen.
    pub lopeta: bool,
    pub aanet: Aanet,
    pub kello_ms: u32,
    pub ohjaus: Ohjaus,
    /// Pelaaja poistuu kesken pelin: peli seisoo, ja nimen jälkeen palataan valikkoon.
    poistuu: bool,
    kertyma_ms: u32,
    moottori_ms: u32,
    lautanen_ms: u32,
    arpa: Arpa,
}

impl Asteroidit {
    pub fn new(siemen: u64) -> Self {
        let mut arpa = Arpa::new(siemen);
        Self {
            vaihe: Vaihe::Alku,
            lauta: Avaruus::new(0, &mut arpa),
            taulu: Tulostaulu::new(Nettipeli::Asteroidit),
            nimi: String::new(),
            uusi_tulos: false,
            viesti: String::new(),
            rajahdykset: Vec::new(),
            lopeta: false,
            aanet: Aanet::default(),
            kello_ms: 0,
            ohjaus: Ohjaus::default(),
            poistuu: false,
            kertyma_ms: 0,
            moottori_ms: 0,
            lautanen_ms: 0,
            arpa,
        }
    }

    /// Pohjassa olevat nuolet. Nuolella kääntäminen ohittaa hiiren tähtäimen.
    pub fn aseta_nuolet(&mut self, kaanto: i8, tyonto: bool) {
        self.ohjaus.kaanto = kaanto;
        self.ohjaus.tyonto = tyonto;
        if kaanto != 0 {
            self.ohjaus.tahtain = None;
        }
    }

    pub fn edista(&mut self, ms: u32) {
        self.kello_ms = self.kello_ms.saturating_add(ms);
        let kello = self.kello_ms;
        self.rajahdykset
            .retain(|r| kello < r.alku.saturating_add(RAJAHDYS_MS));
        // Kivet ajelehtivat vielä pelin päätyttyä.
        if self.poistuu || matches!(self.vaihe, Vaihe::Alku | Vaihe::Tauko) {
            return;
        }
        self.jatkuvat_aanet(ms);
        self.kertyma_ms = (self.kertyma_ms + ms).min(SUURIN_KURO_MS);
        while self.kertyma_ms >= ASKEL_MS {
            self.kertyma_ms -= ASKEL_MS;
            for tapahtuma in self.lauta.askel(self.ohjaus, &mut self.arpa) {
                self.kasittele_tapahtuma(tapahtuma);
            }
        }
    }

    fn jatkuvat_aanet(&mut self, ms: u32) {
        self.moottori_ms = self.moottori_ms.saturating_sub(ms);
        self.lautanen_ms = self.lautanen_ms.saturating_sub(ms);
        if self.lauta.tyontaa && self.moottori_ms == 0 {
            self.moottori_ms = MOOTTORIN_VALI_MS;
            self.aanet.soita(Aani::Moottori);
        }
        if self.lauta.lautanen.is_some() && self.lautanen_ms == 0 {
            self.lautanen_ms = LAUTASEN_VALI_MS;
            self.aanet.soita(Aani::Lautanen);
        }
    }

    fn rajahda(&mut self, paikka: Piste, koko: f32) {
        self.rajahdykset.push(Rajahdys {
            paikka,
            koko,
            alku: self.kello_ms,
        });
    }

    fn kasittele_tapahtuma(&mut self, tapahtuma: Tapahtuma) {
        self.taulu.paras = self.taulu.paras.max(self.lauta.pisteet);
        match tapahtuma {
            Tapahtuma::Kivi { paikka, koko } => {
                self.aanet.soita(Aani::Kivi);
                self.rajahda(paikka, koko.sade() * 1.5);
            }
            Tapahtuma::LautanenTuhoutui { paikka } => {
                self.aanet.soita(Aani::Rajahdys);
                self.rajahda(paikka, 45.0);
            }
            Tapahtuma::LautanenAmpui => self.aanet.soita(Aani::Ammus),
            Tapahtuma::Menetys { paikka } => {
                self.aanet.soita(Aani::Rajahdys);
                self.rajahda(paikka, 70.0);
                self.viesti = muotoile("asteroidit.alus_tuhoutui", &[&self.lauta.elamat]);
            }
            Tapahtuma::Palasi => self.viesti.clear(),
            Tapahtuma::Lisaelama => {
                self.aanet.soita(Aani::Yhdistys);
                self.viesti = kaanna("asteroidit.lisaalus").to_string();
            }
            Tapahtuma::UusiAalto => {
                self.aanet.soita(Aani::Alku);
                self.viesti = muotoile("asteroidit.uusi_aalto", &[&(self.lauta.aalto + 1)]);
            }
            Tapahtuma::Ohi { paikka } => {
                self.aanet.soita(Aani::Rajahdys);
                self.aanet.soita_viiveella(Aani::Tappio, LOPPU_VIIVE_MS);
                self.aanet
                    .soita_viiveella(Aani::PeliOhi, LOPPU_VIIVE_MS + PUHE_VIIVE_MS);
                self.rajahda(paikka, 70.0);
                self.lopeta_peli();
            }
        }
    }

    fn lopeta_peli(&mut self) {
        self.vaihe = if self.taulu.mahtuu(self.lauta.pisteet) {
            self.nimi.clear();
            Vaihe::Nimi
        } else {
            Vaihe::Tappio
        };
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
        self.vaihe = Vaihe::Tappio;
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
        self.lauta = Avaruus::new(0, &mut self.arpa);
        self.vaihe = Vaihe::Pelissa;
        self.viesti.clear();
        self.rajahdykset.clear();
        self.kertyma_ms = 0;
        self.aanet.soita(Aani::Alku);
    }

    /// Ikkuna menetti kohdistuksen: alus ei saa tuhoutua pelaajan poissa ollessa.
    pub fn keskeyta(&mut self) {
        if self.vaihe == Vaihe::Pelissa {
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
            Vaihe::Pelissa => self.kasittele_pelissa(syote),
            Vaihe::Tauko => {
                if matches!(
                    syote,
                    Syote::Ammu | Syote::Klikkaus(..) | Syote::Kirjain('P')
                ) {
                    self.vaihe = Vaihe::Pelissa;
                }
            }
            Vaihe::Tappio => self.kasittele_loppu(syote),
            Vaihe::Nimi => {}
        }
    }

    fn kasittele_pelissa(&mut self, syote: Syote) {
        match syote {
            Syote::Hiiri(x, y) => {
                let paikka = ((x - piirto::KENTTA_X) as f32, (y - piirto::KENTTA_Y) as f32);
                self.ohjaus.tahtain = Some(paikka);
            }
            Syote::Ammu | Syote::Klikkaus(..) => {
                let lahti = self.lauta.ammu();
                if lahti {
                    self.aanet.soita(Aani::Ammus);
                }
            }
            Syote::Kirjain('H') => {
                let hyppasi = self.lauta.hyppaa(&mut self.arpa);
                if hyppasi {
                    self.aanet.soita(Aani::Liuku);
                }
            }
            Syote::Kirjain('P') => self.vaihe = Vaihe::Tauko,
            _ => {}
        }
    }

    fn kasittele_loppu(&mut self, syote: Syote) {
        match loppunapit::valinta(syote) {
            Some(Loppunappi::Uudestaan) => self.aloita_alusta(),
            Some(Loppunappi::Valikko) => self.lopeta = true,
            None => {}
        }
    }

    /// Tuhoutunutta alusta ei piirretä.
    pub fn alus_nakyy(&self) -> bool {
        self.lauta.tila == Tila::Lennossa
    }
}

impl Pelinakyma for Asteroidit {
    fn kasittele(&mut self, syote: Syote) {
        Asteroidit::kasittele(self, syote);
    }

    fn edista(&mut self, ms: u32) {
        Asteroidit::edista(self, ms);
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

    fn aseta_ohjaus(&mut self, suunta: i8, ylos: bool) {
        self.aseta_nuolet(suunta, ylos);
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
