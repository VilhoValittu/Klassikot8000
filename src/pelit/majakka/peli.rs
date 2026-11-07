use klassikot8000::Arpa;
use klassikot8000::majakka::{
    ASKEL_MS, Laji, Meri, Ohjaus, Piste, Tapahtuma, Vaikeus, suunta_majakalta,
};

use super::piirto::{self, Merikuva};
use crate::kaynnistin::tallennus::{Asetukset, Tallennus};
use crate::yhteinen::aanet::{Aanet, Aani, PUHE_VIIVE_MS};
use crate::yhteinen::kieli::{kaanna, muotoile};
use crate::yhteinen::pelinakyma::Pelinakyma;
use crate::yhteinen::piirto::{Kehys, aanikytkimella};
use crate::yhteinen::syote::{Syote, Yleinen, yleinen};
use crate::yhteinen::tulokset::{Nettipeli, Tulostaulu, muokkaa_nimea};

pub const TEHOSTE_MS: u32 = 1000;
// Ikkunan raahaus voi pysäyttää silmukan: laivat eivät saa sen jälkeen hypätä kerralla pitkälle.
const SUURIN_KURO_MS: u32 = 100;
const LOPPU_VIIVE_MS: u32 = 250;
/// Paneeliin mahtuu kaksi viestiä pelin ohjeen yläpuolelle.
pub const VIESTEJA: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vaihe {
    Alku,
    Pelissa,
    Tauko,
    Tappio,
    /// Tulos pääsi kolmen parhaan joukkoon ja odottaa nimimerkkiä.
    Nimi,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nappi {
    Uudestaan,
    Valikko,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Tehostelaji {
    /// Laiva hajosi karikkoon tai rantaan.
    Haaksirikko,
    /// Satamaan päässeen laivan tai kiinni jääneen salakuljettajan pisteet nousevat.
    Pisteet(u32),
    /// Salakuljettaja jäi kiinni.
    Kiinni,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tehoste {
    pub laji: Tehostelaji,
    pub paikka: Piste,
    pub alku: u32,
}

pub struct Majakka {
    pub vaihe: Vaihe,
    pub vaikeus: Vaikeus,
    pub meri: Meri,
    /// Yön meri valmiiksi laskettuna. Lasketaan uudelleen, kun karikot vaihtuvat.
    pub kuva: Option<Merikuva>,
    pub taulu: Tulostaulu,
    pub nimi: String,
    pub uusi_tulos: bool,
    /// Uusin viesti viimeisenä.
    pub viestit: Vec<String>,
    pub tehosteet: Vec<Tehoste>,
    pub lopeta: bool,
    pub aanet: Aanet,
    pub kello_ms: u32,
    pub ohjaus: Ohjaus,
    /// Pelaaja poistuu kesken pelin: peli seisoo, ja nimen jälkeen palataan valikkoon.
    poistuu: bool,
    kertyma_ms: u32,
    arpa: Arpa,
}

fn tason_indeksi(vaikeus: Vaikeus) -> usize {
    Vaikeus::KAIKKI
        .iter()
        .position(|&taso| taso == vaikeus)
        .unwrap_or(0)
}

pub fn lajin_nimi(laji: Laji) -> &'static str {
    match laji {
        Laji::Vene => "majakka.vene",
        Laji::Kalastaja => "majakka.kalastaja",
        Laji::Rahtilaiva => "majakka.rahtilaiva",
        Laji::Salakuljettaja => "majakka.salakuljettaja",
    }
}

impl Majakka {
    pub fn new(siemen: u64) -> Self {
        let mut arpa = Arpa::new(siemen);
        Self {
            vaihe: Vaihe::Alku,
            vaikeus: Vaikeus::Normaali,
            meri: Meri::new(Vaikeus::Normaali, 0, &mut arpa),
            kuva: None,
            taulu: Tulostaulu::new(Nettipeli::Majakka),
            nimi: String::new(),
            uusi_tulos: false,
            viestit: Vec::new(),
            tehosteet: Vec::new(),
            lopeta: false,
            aanet: Aanet::default(),
            kello_ms: 0,
            ohjaus: Ohjaus::default(),
            poistuu: false,
            kertyma_ms: 0,
            arpa,
        }
    }

    /// Pohjassa olevat nuolet. Nuolella kääntäminen ohittaa hiiren tähtäimen.
    pub fn aseta_nuolet(&mut self, kaanto: i8) {
        self.ohjaus.kaanto = kaanto;
        if kaanto != 0 {
            self.ohjaus.tahtain = None;
        }
    }

    pub fn edista(&mut self, ms: u32) {
        self.kello_ms = self.kello_ms.saturating_add(ms);
        let kello = self.kello_ms;
        self.tehosteet
            .retain(|t| kello < t.alku.saturating_add(TEHOSTE_MS));
        if self.poistuu || self.vaihe != Vaihe::Pelissa {
            return;
        }
        self.kertyma_ms = (self.kertyma_ms + ms).min(SUURIN_KURO_MS);
        while self.kertyma_ms >= ASKEL_MS {
            self.kertyma_ms -= ASKEL_MS;
            for tapahtuma in self.meri.askel(self.ohjaus, &mut self.arpa) {
                self.kasittele_tapahtuma(tapahtuma);
            }
            if self.vaihe != Vaihe::Pelissa {
                return;
            }
        }
    }

    fn viesti(&mut self, teksti: String) {
        self.viestit.push(teksti);
        if self.viestit.len() > VIESTEJA {
            self.viestit.remove(0);
        }
    }

    fn tehoste(&mut self, laji: Tehostelaji, paikka: Piste) {
        self.tehosteet.push(Tehoste {
            laji,
            paikka,
            alku: self.kello_ms,
        });
    }

    fn kasittele_tapahtuma(&mut self, tapahtuma: Tapahtuma) {
        self.taulu.paras = self.taulu.paras.max(self.meri.pisteet);
        match tapahtuma {
            Tapahtuma::Satamassa { laji, pisteet } => {
                self.aanet.jono.push((Aani::Laivakello, 0));
                let satama = klassikot8000::majakka::SATAMA;
                self.tehoste(Tehostelaji::Pisteet(pisteet), satama);
                let nimi = kaanna(lajin_nimi(laji));
                self.viesti(muotoile("majakka.satamassa", &[&nimi, &pisteet]));
            }
            Tapahtuma::Karille { laji, paikka } => {
                self.aanet.jono.push((Aani::Karille, 0));
                self.tehoste(Tehostelaji::Haaksirikko, paikka);
                let nimi = kaanna(lajin_nimi(laji));
                self.viesti(muotoile("majakka.karille", &[&nimi, &self.meri.renkaat]));
            }
            Tapahtuma::Eksyi { laji } => {
                self.aanet.jono.push((Aani::Virhe, 0));
                self.viesti(muotoile("majakka.eksyi", &[&kaanna(lajin_nimi(laji))]));
            }
            Tapahtuma::Kiinni { pisteet, paikka } => {
                self.aanet.jono.push((Aani::Yhdistys, 0));
                self.tehoste(Tehostelaji::Kiinni, paikka);
                self.tehoste(Tehostelaji::Pisteet(pisteet), paikka);
                self.viesti(muotoile("majakka.kiinni", &[&pisteet]));
            }
            Tapahtuma::Karkasi => self.viesti(kaanna("majakka.karkasi").to_string()),
            Tapahtuma::Puuska => {
                self.aanet.jono.push((Aani::Tuuli, 0));
                self.viesti(kaanna("majakka.puuska").to_string());
            }
            Tapahtuma::TorviLatautui => self.aanet.jono.push((Aani::Pomppu, 0)),
            Tapahtuma::Aamu { satamassa, bonus } => {
                self.viesti(muotoile("majakka.aamu", &[&satamassa]));
                if bonus > 0 {
                    self.aanet.jono.push((Aani::Voitto, 0));
                    self.viesti(muotoile("majakka.bonus", &[&bonus]));
                } else {
                    self.aanet.jono.push((Aani::Yhdistys, 0));
                }
            }
            Tapahtuma::UusiYo => {
                self.aanet.jono.push((Aani::Alku, 0));
                self.kuva = Some(Merikuva::new(&self.meri));
                self.viesti(muotoile("majakka.uusi_yo", &[&(self.meri.yo + 1)]));
            }
            Tapahtuma::Ohi => {
                self.aanet.jono.push((Aani::Tappio, LOPPU_VIIVE_MS));
                self.aanet
                    .jono
                    .push((Aani::PeliOhi, LOPPU_VIIVE_MS + PUHE_VIIVE_MS));
                self.lopeta_peli();
            }
        }
        self.taulu.paras = self.taulu.paras.max(self.meri.pisteet);
    }

    fn lopeta_peli(&mut self) {
        self.vaihe = if self.taulu.mahtuu(self.meri.pisteet) {
            self.nimi.clear();
            Vaihe::Nimi
        } else {
            Vaihe::Tappio
        };
        self.uusi_tulos = true;
        self.viesti(muotoile("yhteinen.tulos", &[&self.meri.pisteet]));
    }

    /// Kirjaa odottavan tuloksen.
    pub fn vahvista_nimi(&mut self) {
        if self.vaihe != Vaihe::Nimi {
            return;
        }
        let pisteet = self.meri.pisteet;
        self.taulu.kirjaa(&mut self.nimi, pisteet);
        self.uusi_tulos = true;
        self.vaihe = Vaihe::Tappio;
        self.lopeta = self.poistuu;
    }

    /// Esc tai ikkunan sulkeminen kesken pelin: listalle yltävä tulos ei saa kadota.
    pub fn poistu(&mut self) {
        let kesken = matches!(self.vaihe, Vaihe::Pelissa | Vaihe::Tauko);
        if kesken && self.taulu.mahtuu(self.meri.pisteet) {
            self.nimi.clear();
            self.poistuu = true;
            self.vaihe = Vaihe::Nimi;
        } else {
            self.lopeta = true;
        }
    }

    fn kasittele_nimi(&mut self, syote: Syote) {
        match syote {
            // Torveen painettu välilyönti ei saa kuitata nimeä ennen kuin sitä on kirjoitettu.
            Syote::Ammu if self.nimi.is_empty() => {}
            Syote::Ammu | Syote::Peru => self.vahvista_nimi(),
            _ => muokkaa_nimea(&mut self.nimi, syote),
        }
    }

    pub fn aloita_alusta(&mut self) {
        self.meri = Meri::new(self.vaikeus, 0, &mut self.arpa);
        self.kuva = Some(Merikuva::new(&self.meri));
        self.vaihe = Vaihe::Pelissa;
        self.viestit.clear();
        self.viesti(muotoile("majakka.uusi_yo", &[&1]));
        self.tehosteet.clear();
        self.kertyma_ms = 0;
        self.aanet.jono.push((Aani::Alku, 0));
    }

    /// Ikkuna menetti kohdistuksen: laivat eivät saa ajaa karille pelaajan poissa ollessa.
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
            Yleinen::Nimi => return self.kasittele_nimi(syote),
            // Kesken pelin Esc pysäyttää ensin, jottei vahinkopainallus hävitä peliä.
            Yleinen::Peru if self.vaihe == Vaihe::Pelissa => {
                self.vaihe = Vaihe::Tauko;
                return;
            }
            Yleinen::Peru => return self.poistu(),
            Yleinen::Aanet | Yleinen::Ohita => return,
        }
        match self.vaihe {
            Vaihe::Alku => self.kasittele_alku(syote),
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

    fn kasittele_alku(&mut self, syote: Syote) {
        let tasot = Vaikeus::KAIKKI;
        let valittu = match syote {
            Syote::Numero(numero) => (numero as usize)
                .checked_sub(1)
                .and_then(|i| tasot.get(i).copied()),
            Syote::Nuoli(_, muutos) => {
                let uusi = tason_indeksi(self.vaikeus).saturating_add_signed(muutos as isize);
                Some(tasot[uusi.min(tasot.len() - 1)])
            }
            Syote::Klikkaus(x, y) => piirto::vaikeus_pisteesta(x, y),
            _ => None,
        };
        if let Some(vaikeus) = valittu {
            self.vaikeus = vaikeus;
        } else if matches!(syote, Syote::Ammu | Syote::Klikkaus(..)) {
            self.aloita_alusta();
        }
    }

    fn kasittele_pelissa(&mut self, syote: Syote) {
        match syote {
            Syote::Hiiri(x, y) => {
                let paikka = ((x - piirto::KENTTA_X) as f32, (y - piirto::KENTTA_Y) as f32);
                self.ohjaus.tahtain = Some(suunta_majakalta(paikka));
            }
            Syote::Ammu | Syote::Klikkaus(..) | Syote::OikeaKlikkaus(..)
                if self.meri.soita_torvi() =>
            {
                self.aanet.jono.push((Aani::Sumutorvi, 0));
            }
            Syote::Kirjain('P') => self.vaihe = Vaihe::Tauko,
            _ => {}
        }
    }

    fn kasittele_loppu(&mut self, syote: Syote) {
        let nappi = match syote {
            Syote::Kirjain('K') => Some(Nappi::Uudestaan),
            Syote::Kirjain('E') => Some(Nappi::Valikko),
            Syote::Klikkaus(x, y) => piirto::nappi_pisteesta(x, y),
            _ => None,
        };
        match nappi {
            Some(Nappi::Uudestaan) => self.aloita_alusta(),
            Some(Nappi::Valikko) => self.lopeta = true,
            None => {}
        }
    }
}

impl Pelinakyma for Majakka {
    fn kasittele(&mut self, syote: Syote) {
        Majakka::kasittele(self, syote);
    }

    fn edista(&mut self, ms: u32) {
        Majakka::edista(self, ms);
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

    fn lue_tallennuksesta(&mut self, tallennus: &Tallennus) {
        self.vaikeus = tallennus.asetukset.majakka;
    }

    fn kirjaa_asetukset(&self, asetukset: &mut Asetukset) {
        asetukset.majakka = self.vaikeus;
    }

    fn toimintapeli(&self) -> bool {
        self.vaihe != Vaihe::Alku
    }

    fn aseta_ohjaus(&mut self, suunta: i8, _ylos: bool) {
        self.aseta_nuolet(suunta);
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
