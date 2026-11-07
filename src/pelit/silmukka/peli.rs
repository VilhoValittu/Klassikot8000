use klassikot8000::Arpa;
use klassikot8000::silmukka::{ASKEL_MS, Niitty, Ruutu, Saalis, Suunta, Tapahtuma};

use super::piirto;
use crate::yhteinen::aanet::{Aanet, Aani, PUHE_VIIVE_MS};
use crate::yhteinen::kieli::{kaanna, muotoile};
use crate::yhteinen::pelinakyma::Pelinakyma;
use crate::yhteinen::piirto::{Kehys, aanikytkimella};
use crate::yhteinen::syote::{Syote, Yleinen, yleinen};
use crate::yhteinen::tulokset::{Nettipeli, Tulostaulu, muokkaa_nimea};

pub const VALAHDYS_MS: u32 = 900;
pub const PAMAUS_MS: u32 = 450;
pub const LEIJUNTA_MS: u32 = 1000;
pub const KEHU_MS: u32 = 1200;
pub const TARINA_MS: u32 = 350;
// Ikkunan raahaus voi pysäyttää silmukan: mato ei saa sen jälkeen hypätä kerralla pitkälle.
const SUURIN_KURO_MS: u32 = 100;
const LOPPU_VIIVE_MS: u32 = 250;
const HELINAN_VALI_MS: u32 = 70;
const HELINAT: [Aani; 6] = [
    Aani::Helina1,
    Aani::Helina2,
    Aani::Helina3,
    Aani::Helina4,
    Aani::Helina5,
    Aani::Helina6,
];

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

/// Suljetun silmukan tehoste: reitti purkautuu ruutu kerrallaan ja sisus leimahtaa.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Valahdys {
    pub reitti: Vec<Ruutu>,
    pub sisus: Vec<Ruutu>,
    /// Tyhjä silmukka vain haihtuu.
    pub saalis: bool,
    pub alku: u32,
}

/// Saaliin tai kuolleen madon räjähdys.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pamaus {
    pub paikka: Ruutu,
    pub koko: f32,
    pub alku: u32,
}

/// Kentän yllä nouseva pisteteksti.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Leijuva {
    pub paikka: Ruutu,
    pub teksti: String,
    pub vari: u32,
    pub alku: u32,
}

pub struct Silmukka {
    pub vaihe: Vaihe,
    pub lauta: Niitty,
    pub taulu: Tulostaulu,
    /// Kirjoitettavana oleva nimimerkki.
    pub nimi: String,
    /// Tosi, kun päättyneen pelin ennätys odottaa tiedostoon kirjoitusta.
    pub uusi_tulos: bool,
    pub viesti: String,
    pub valahdykset: Vec<Valahdys>,
    pub pamaukset: Vec<Pamaus>,
    pub leijuvat: Vec<Leijuva>,
    /// Kentän keskellä hetken näkyvä kehu ja sen alkuhetki.
    pub kehu: Option<(&'static str, u32)>,
    /// Ruudun tärinän voimakkuus ja alkuhetki.
    pub tarina: Option<(f32, u32)>,
    /// Pyyntö palata käynnistimeen.
    pub lopeta: bool,
    pub aanet: Aanet,
    pub kello_ms: u32,
    /// Pelaaja poistuu kesken pelin: peli seisoo, ja nimen jälkeen palataan valikkoon.
    poistuu: bool,
    kertyma_ms: u32,
    arpa: Arpa,
}

/// Kehu kertoo, kuinka suuri saalis oli. Pienestä saaliista ei tarvitse kehua.
fn kehu(maara: usize) -> Option<&'static str> {
    match maara {
        0 | 1 => None,
        2 => Some(kaanna("silmukka.kehu_2")),
        3 => Some(kaanna("silmukka.kehu_3")),
        4 => Some(kaanna("silmukka.kehu_4")),
        5 | 6 => Some(kaanna("silmukka.kehu_5")),
        7 | 8 => Some(kaanna("silmukka.kehu_7")),
        _ => Some(kaanna("silmukka.kehu_9")),
    }
}

impl Silmukka {
    pub fn new(siemen: u64) -> Self {
        let mut arpa = Arpa::new(siemen);
        Self {
            vaihe: Vaihe::Alku,
            lauta: Niitty::new(&mut arpa),
            taulu: Tulostaulu::new(Nettipeli::Silmukka),
            nimi: String::new(),
            uusi_tulos: false,
            viesti: String::new(),
            valahdykset: Vec::new(),
            pamaukset: Vec::new(),
            leijuvat: Vec::new(),
            kehu: None,
            tarina: None,
            lopeta: false,
            aanet: Aanet::default(),
            kello_ms: 0,
            poistuu: false,
            kertyma_ms: 0,
            arpa,
        }
    }

    pub fn edista(&mut self, ms: u32) {
        self.kello_ms = self.kello_ms.saturating_add(ms);
        self.siivoa_tehosteet();
        // Otukset lentävät vielä pelin päätyttyä.
        if self.poistuu || matches!(self.vaihe, Vaihe::Alku | Vaihe::Tauko) {
            return;
        }
        self.kertyma_ms = (self.kertyma_ms + ms).min(SUURIN_KURO_MS);
        while self.kertyma_ms >= ASKEL_MS {
            self.kertyma_ms -= ASKEL_MS;
            for tapahtuma in self.lauta.askel(&mut self.arpa) {
                self.kasittele_tapahtuma(tapahtuma);
            }
        }
    }

    fn siivoa_tehosteet(&mut self) {
        let kello = self.kello_ms;
        let kesken = |alku: u32, kesto: u32| kello < alku.saturating_add(kesto);
        self.valahdykset.retain(|v| kesken(v.alku, VALAHDYS_MS));
        self.pamaukset.retain(|p| kesken(p.alku, PAMAUS_MS));
        self.leijuvat.retain(|l| kesken(l.alku, LEIJUNTA_MS));
        self.kehu = self.kehu.filter(|k| kesken(k.1, KEHU_MS));
        self.tarina = self.tarina.filter(|t| kesken(t.1, TARINA_MS));
    }

    fn leiju(&mut self, paikka: Ruutu, pisteet: u32, vari: u32) {
        self.leijuvat.push(Leijuva {
            paikka,
            teksti: format!("+{pisteet}"),
            vari,
            alku: self.kello_ms,
        });
    }

    fn pamahda(&mut self, paikka: Ruutu, koko: f32) {
        self.pamaukset.push(Pamaus {
            paikka,
            koko,
            alku: self.kello_ms,
        });
    }

    fn kasittele_tapahtuma(&mut self, tapahtuma: Tapahtuma) {
        self.taulu.paras = self.taulu.paras.max(self.lauta.pisteet);
        match tapahtuma {
            Tapahtuma::Omena { paikka, pisteet } => {
                self.aanet.jono.push((Aani::Omena, 0));
                self.leiju(paikka, pisteet, piirto::OMENAN_VARI);
            }
            Tapahtuma::Kulta { paikka, pisteet } => {
                self.aanet.jono.push((Aani::Yhdistys, 0));
                self.leiju(paikka, pisteet, piirto::KULLAN_VARI);
                self.pamahda(paikka, 40.0);
                self.viesti = kaanna("silmukka.kultaomena").to_string();
            }
            Tapahtuma::Muna { paikka, pisteet } => {
                self.aanet.jono.push((Aani::Liuku, 0));
                self.leiju(paikka, pisteet, piirto::MUNAN_VARI);
                self.viesti = kaanna("silmukka.muna_litistyi").to_string();
            }
            Tapahtuma::Silmukka(saalis) => self.pyydysta(saalis),
            Tapahtuma::TyhjaSilmukka { reitti } => {
                self.aanet.jono.push((Aani::Puhkus, 0));
                self.valahdykset.push(Valahdys {
                    reitti,
                    sisus: Vec::new(),
                    saalis: false,
                    alku: self.kello_ms,
                });
                self.viesti = kaanna("silmukka.tyhja").to_string();
            }
            Tapahtuma::Kuoriutui { .. } => self.aanet.jono.push((Aani::Surina, 0)),
            Tapahtuma::KultaIlmestyi { .. } => {
                self.aanet.jono.push((Aani::Helina5, 0));
                self.aanet.jono.push((Aani::Helina6, HELINAN_VALI_MS));
                self.viesti = kaanna("silmukka.kulta_ilmestyi").to_string();
            }
            Tapahtuma::KultaKatosi { paikka } => self.pamahda(paikka, 20.0),
            Tapahtuma::UusiTaso => {
                self.aanet.jono.push((Aani::Alku, 0));
                self.viesti = muotoile("silmukka.uusi_taso", &[&self.lauta.taso]);
            }
            Tapahtuma::Seina => self.aanet.jono.push((Aani::Pomppu, 0)),
            Tapahtuma::Kuolema { paikka } => {
                self.aanet.jono.push((Aani::Rajahdys, 0));
                self.aanet.jono.push((Aani::Tappio, LOPPU_VIIVE_MS));
                self.aanet
                    .jono
                    .push((Aani::PeliOhi, LOPPU_VIIVE_MS + PUHE_VIIVE_MS));
                self.pamahda(paikka, 70.0);
                self.tarina = Some((8.0, self.kello_ms));
                self.lopeta_peli();
            }
        }
    }

    fn pyydysta(&mut self, saalis: Saalis) {
        let maara = saalis.kohteet.len();
        for (i, &(paikka, _)) in saalis.kohteet.iter().enumerate() {
            self.pamahda(paikka, 45.0);
            if let Some(&aani) = HELINAT.get(i) {
                self.aanet.jono.push((aani, i as u32 * HELINAN_VALI_MS));
            }
        }
        self.aanet.jono.push((Aani::Kivi, 0));
        // Pisteet nousevat saaliin keskeltä, jotta ne näkyvät siellä, minne katse on suunnattu.
        let keski = saalis.kohteet[maara / 2].0;
        self.leiju(keski, saalis.pisteet, piirto::KULLAN_VARI);
        if let Some(teksti) = kehu(maara) {
            self.kehu = Some((teksti, self.kello_ms));
        }
        self.tarina = Some(((maara as f32 * 1.5).min(7.0), self.kello_ms));
        self.viesti = muotoile("silmukka.saalis", &[&maara, &saalis.pisteet]);
        self.valahdykset.push(Valahdys {
            reitti: saalis.reitti,
            sisus: saalis.sisus,
            saalis: true,
            alku: self.kello_ms,
        });
    }

    fn lopeta_peli(&mut self) {
        self.taulu.paras = self.taulu.paras.max(self.lauta.pisteet);
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

    fn kasittele_nimi(&mut self, syote: Syote) {
        match syote {
            // Viimeisenä painettu välilyönti ei saa kuitata nimeä ennen kuin sitä on kirjoitettu.
            Syote::Ammu if self.nimi.is_empty() => {}
            Syote::Ammu | Syote::Peru => self.vahvista_nimi(),
            _ => muokkaa_nimea(&mut self.nimi, syote),
        }
    }

    fn aloita_alusta(&mut self) {
        self.lauta = Niitty::new(&mut self.arpa);
        self.vaihe = Vaihe::Pelissa;
        self.viesti.clear();
        self.valahdykset.clear();
        self.pamaukset.clear();
        self.leijuvat.clear();
        self.kehu = None;
        self.tarina = None;
        self.kertyma_ms = 0;
        self.aanet.jono.push((Aani::Alku, 0));
    }

    /// Ikkuna menetti kohdistuksen: mato ei saa ajaa seinään pelaajan poissa ollessa.
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
            Syote::Nuoli(-1, 0) | Syote::Kirjain('W') => self.lauta.kaanny(Suunta::Ylos),
            Syote::Nuoli(1, 0) | Syote::Kirjain('S') => self.lauta.kaanny(Suunta::Alas),
            Syote::Nuoli(0, -1) | Syote::Kirjain('A') => self.lauta.kaanny(Suunta::Vasen),
            Syote::Nuoli(0, 1) | Syote::Kirjain('D') => self.lauta.kaanny(Suunta::Oikea),
            Syote::Hiiri(x, y) => self.lauta.aseta_tahtain(piirto::ruutu_pisteesta(x, y)),
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

impl Pelinakyma for Silmukka {
    fn kasittele(&mut self, syote: Syote) {
        Silmukka::kasittele(self, syote);
    }

    fn edista(&mut self, ms: u32) {
        Silmukka::edista(self, ms);
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

    fn keskeyta(&mut self) {
        Self::keskeyta(self);
    }

    fn nuolet_toistuvat(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests;
