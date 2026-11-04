use klassikot8000::Arpa;
use klassikot8000::silmukka::{ASKEL_MS, Niitty, Ruutu, Saalis, Suunta, Tapahtuma};

use super::piirto;
use crate::yhteinen::aanet::{Aani, PUHE_VIIVE_MS};
use crate::yhteinen::kieli::{kaanna, muotoile};
use crate::yhteinen::pelinakyma::{Lista, Pelinakyma};
use crate::yhteinen::piirto::{Kehys, aanikytkimella};
use crate::yhteinen::syote::{Syote, Yleinen, yleinen};
use crate::yhteinen::tulokset::{
    Nettipeli, Tulos, lisaa_tulos, mahtuu_parhaisiin, muokkaa_nimea, ota_nimi,
};

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
    pub paras: u32,
    pub parhaat: Vec<Tulos>,
    /// Nettilista. `None`, kunnes palvelin on vastannut.
    pub netti_parhaat: Option<Vec<Tulos>>,
    /// Kirjattu tulos, joka odottaa lähetystä nettilistalle.
    pub lahetys: Option<Tulos>,
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
    pub aanet_paalla: bool,
    pub aanijono: Vec<(Aani, u32)>,
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
            paras: 0,
            parhaat: Vec::new(),
            netti_parhaat: None,
            lahetys: None,
            nimi: String::new(),
            uusi_tulos: false,
            viesti: String::new(),
            valahdykset: Vec::new(),
            pamaukset: Vec::new(),
            leijuvat: Vec::new(),
            kehu: None,
            tarina: None,
            lopeta: false,
            aanet_paalla: true,
            aanijono: Vec::new(),
            kello_ms: 0,
            poistuu: false,
            kertyma_ms: 0,
            arpa,
        }
    }

    /// Esillä oleva lista: nettilista, tai oma lista, jos verkkoon ei ole saatu yhteyttä.
    pub fn nakyva_lista(&self) -> &[Tulos] {
        self.netti_parhaat.as_deref().unwrap_or(&self.parhaat)
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
        self.paras = self.paras.max(self.lauta.pisteet);
        match tapahtuma {
            Tapahtuma::Omena { paikka, pisteet } => {
                self.aanijono.push((Aani::Omena, 0));
                self.leiju(paikka, pisteet, piirto::OMENAN_VARI);
            }
            Tapahtuma::Kulta { paikka, pisteet } => {
                self.aanijono.push((Aani::Yhdistys, 0));
                self.leiju(paikka, pisteet, piirto::KULLAN_VARI);
                self.pamahda(paikka, 40.0);
                self.viesti = kaanna("silmukka.kultaomena").to_string();
            }
            Tapahtuma::Muna { paikka, pisteet } => {
                self.aanijono.push((Aani::Liuku, 0));
                self.leiju(paikka, pisteet, piirto::MUNAN_VARI);
                self.viesti = kaanna("silmukka.muna_litistyi").to_string();
            }
            Tapahtuma::Silmukka(saalis) => self.pyydysta(saalis),
            Tapahtuma::TyhjaSilmukka { reitti } => {
                self.aanijono.push((Aani::Puhkus, 0));
                self.valahdykset.push(Valahdys {
                    reitti,
                    sisus: Vec::new(),
                    saalis: false,
                    alku: self.kello_ms,
                });
                self.viesti = kaanna("silmukka.tyhja").to_string();
            }
            Tapahtuma::Kuoriutui { .. } => self.aanijono.push((Aani::Surina, 0)),
            Tapahtuma::KultaIlmestyi { .. } => {
                self.aanijono.push((Aani::Helina5, 0));
                self.aanijono.push((Aani::Helina6, HELINAN_VALI_MS));
                self.viesti = kaanna("silmukka.kulta_ilmestyi").to_string();
            }
            Tapahtuma::KultaKatosi { paikka } => self.pamahda(paikka, 20.0),
            Tapahtuma::UusiTaso => {
                self.aanijono.push((Aani::Alku, 0));
                self.viesti = muotoile("silmukka.uusi_taso", &[&self.lauta.taso]);
            }
            Tapahtuma::Seina => self.aanijono.push((Aani::Pomppu, 0)),
            Tapahtuma::Kuolema { paikka } => {
                self.aanijono.push((Aani::Rajahdys, 0));
                self.aanijono.push((Aani::Tappio, LOPPU_VIIVE_MS));
                self.aanijono
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
                self.aanijono.push((aani, i as u32 * HELINAN_VALI_MS));
            }
        }
        self.aanijono.push((Aani::Kivi, 0));
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
        self.paras = self.paras.max(self.lauta.pisteet);
        self.vaihe = if mahtuu_parhaisiin(self.nakyva_lista(), self.lauta.pisteet) {
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
        let nimi = ota_nimi(&mut self.nimi);
        let pisteet = self.lauta.pisteet;
        let tulos = Tulos { nimi, pisteet };
        lisaa_tulos(&mut self.parhaat, tulos.clone());
        if let Some(lista) = &mut self.netti_parhaat {
            lisaa_tulos(lista, tulos.clone());
        }
        self.lahetys = Some(tulos);
        self.uusi_tulos = true;
        self.vaihe = Vaihe::Tappio;
        self.lopeta = self.poistuu;
    }

    /// Esc tai ikkunan sulkeminen kesken pelin: listalle yltävä tulos ei saa kadota.
    pub fn poistu(&mut self) {
        let kesken = matches!(self.vaihe, Vaihe::Pelissa | Vaihe::Tauko);
        if kesken && mahtuu_parhaisiin(self.nakyva_lista(), self.lauta.pisteet) {
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
        self.aanijono.push((Aani::Alku, 0));
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
            &mut self.aanet_paalla,
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

    fn aanijono(&mut self) -> &mut Vec<(Aani, u32)> {
        &mut self.aanijono
    }

    fn aanet_paalla(&self) -> bool {
        self.aanet_paalla
    }

    fn aseta_aanet(&mut self, paalla: bool) {
        self.aanet_paalla = paalla;
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

    fn listat(&mut self) -> Vec<Lista<'_>> {
        vec![Lista {
            peli: Nettipeli::Silmukka,
            paras: Some(&mut self.paras),
            parhaat: &mut self.parhaat,
            netti: &mut self.netti_parhaat,
            lahetys: Some(&mut self.lahetys),
        }]
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
mod tests {
    use super::*;
    use klassikot8000::silmukka::{Herkku, Laji, Otus};
    use std::collections::VecDeque;

    fn pelissa() -> Silmukka {
        let mut peli = Silmukka::new(1);
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.vaihe, Vaihe::Pelissa);
        peli.aanijono.clear();
        peli
    }

    /// Mato on juuri törmäämässä paikallaan olevaan itikkaan.
    fn paattyva(pisteet: u32) -> Silmukka {
        let mut peli = pelissa();
        peli.lauta.pisteet = pisteet;
        let paa = peli.lauta.paa();
        let mut itikka = Otus::uusi(Laji::Itikka, (paa.0 + 1, paa.1), (1, 1), 0);
        itikka.odotus_ms = 100_000;
        itikka.jakso_ms = 100_000;
        peli.lauta.otukset = vec![itikka];
        peli.lauta.odotus_ms = ASKEL_MS;
        peli
    }

    /// Mato sulkee seuraavalla askeleella renkaan, jonka sisällä on `maara` itikkaa.
    fn sulkeutuva(maara: i32) -> Silmukka {
        let mut peli = pelissa();
        let mut ruudut: VecDeque<_> = (5..=12).map(|x| (x, 5)).collect();
        ruudut.extend((6..=12).map(|y| (12, y)));
        ruudut.extend((5..=11).rev().map(|x| (x, 12)));
        ruudut.extend((6..=11).rev().map(|y| (5, y)));
        ruudut.push_front((4, 5));
        peli.lauta.mato = ruudut.into_iter().rev().collect();
        peli.lauta.suunta = Suunta::Ylos;
        peli.lauta.odotus_ms = ASKEL_MS;
        peli.lauta.herkut.clear();
        peli.lauta.otukset = (0..maara)
            .map(|i| {
                let mut otus = Otus::uusi(Laji::Itikka, (6 + i, 8), (1, 1), 0);
                otus.odotus_ms = 100_000;
                otus.jakso_ms = 100_000;
                otus
            })
            .collect();
        peli
    }

    #[test]
    fn valilyonti_aloittaa_ja_nuolet_ja_kirjaimet_kaantavat() {
        let mut peli = Silmukka::new(1);
        peli.kasittele(Syote::Nuoli(-1, 0));
        assert_eq!(peli.vaihe, Vaihe::Alku);
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.vaihe, Vaihe::Pelissa);
        assert_eq!(peli.aanijono, [(Aani::Alku, 0)]);

        peli.kasittele(Syote::Nuoli(-1, 0));
        peli.kasittele(Syote::Kirjain('A'));
        assert_eq!(peli.lauta.jono, [Suunta::Ylos, Suunta::Vasen]);
        peli.lauta.jono.clear();
        peli.kasittele(Syote::Kirjain('S'));
        peli.kasittele(Syote::Nuoli(0, 1));
        assert_eq!(peli.lauta.jono, [Suunta::Alas, Suunta::Oikea]);
    }

    #[test]
    fn hiiri_asettaa_tahtaimen_ruudun_ja_nuoli_poistaa_sen() {
        let mut peli = pelissa();
        let (x, y) = piirto::ruudun_keskipiste((3, 4));
        peli.kasittele(Syote::Hiiri(x, y));
        assert_eq!(peli.lauta.tahtain, Some((3, 4)));
        peli.kasittele(Syote::Nuoli(-1, 0));
        assert_eq!(peli.lauta.tahtain, None);
    }

    #[test]
    fn saalis_antaa_pisteet_tehosteet_ja_nousevan_helinan() {
        let mut peli = sulkeutuva(3);
        peli.edista(ASKEL_MS);
        assert_eq!(peli.lauta.pisteet, 25 * 3 * 3);
        assert_eq!(peli.valahdykset.len(), 1);
        assert!(peli.valahdykset[0].saalis);
        assert_eq!(peli.pamaukset.len(), 3);
        assert_eq!(peli.leijuvat[0].teksti, "+225");
        assert_eq!(peli.kehu.map(|k| k.0), Some("KOLMOSET!"));
        assert!(peli.tarina.is_some());
        assert!(peli.viesti.contains("225"));
        assert_eq!(
            &peli.aanijono[..3],
            [
                (Aani::Helina1, 0),
                (Aani::Helina2, HELINAN_VALI_MS),
                (Aani::Helina3, 2 * HELINAN_VALI_MS)
            ]
        );
        assert_eq!(peli.paras, 25 * 3 * 3);

        peli.edista(KEHU_MS + VALAHDYS_MS);
        assert!(peli.valahdykset.is_empty());
        assert!(peli.pamaukset.is_empty());
        assert!(peli.leijuvat.is_empty());
        assert_eq!(peli.kehu, None);
        assert_eq!(peli.tarina, None);
    }

    #[test]
    fn suuri_saalis_saa_suuren_kehun_ja_yksittainen_ei_mitaan() {
        assert_eq!(kehu(1), None);
        assert_eq!(kehu(2), Some("TUPLA!"));
        assert_eq!(kehu(16), Some("JUHANNUSTAIKAA!"));
        let mut peli = sulkeutuva(1);
        peli.edista(ASKEL_MS);
        assert_eq!(peli.kehu, None);
        assert_eq!(peli.lauta.pisteet, 25);
    }

    #[test]
    fn tyhja_silmukka_soi_eri_aanen_eika_anna_pisteita() {
        let mut peli = sulkeutuva(0);
        peli.edista(ASKEL_MS);
        assert_eq!(peli.lauta.pisteet, 0);
        assert_eq!(peli.aanijono, [(Aani::Puhkus, 0)]);
        assert!(!peli.valahdykset[0].saalis);
        assert_eq!(peli.vaihe, Vaihe::Pelissa);
    }

    #[test]
    fn tauko_pysayttaa_madon_ja_jatkuu_samasta() {
        let mut peli = pelissa();
        peli.kasittele(Syote::Kirjain('P'));
        let paa = peli.lauta.paa();
        peli.edista(100);
        peli.edista(100);
        assert_eq!(peli.lauta.paa(), paa);
        peli.kasittele(Syote::Kirjain('P'));
        peli.edista(100);
        peli.edista(100);
        assert_ne!(peli.lauta.paa(), paa);

        peli.keskeyta();
        assert_eq!(peli.vaihe, Vaihe::Tauko);
    }

    #[test]
    fn iso_aikahyppy_ei_siirra_matoa_kerralla_pitkalle() {
        let mut peli = pelissa();
        let (x, _) = peli.lauta.paa();
        peli.edista(60_000);
        assert!(peli.lauta.paa().0 - x <= 1);
    }

    #[test]
    fn kuolema_kysyy_nimen_ja_palaa_loppuruutuun() {
        let mut peli = paattyva(40);
        peli.edista(ASKEL_MS);
        assert_eq!(peli.vaihe, Vaihe::Nimi);
        assert_eq!(peli.paras, 40);
        assert!(peli.uusi_tulos);
        assert_eq!(peli.aanijono[0], (Aani::Rajahdys, 0));

        // Viimeisenä painettu välilyönti ei kuittaa tyhjää nimeä, ja K on nimessä kirjain.
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.vaihe, Vaihe::Nimi);
        peli.kasittele(Syote::Kirjain('K'));
        peli.kasittele(Syote::VaihdaAanet);
        assert!(peli.aanet_paalla);
        peli.uusi_tulos = false;
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.vaihe, Vaihe::Tappio);
        assert!(peli.uusi_tulos);
        assert_eq!(peli.parhaat[0].nimi, "KM");
        assert_eq!(peli.lahetys.as_ref().unwrap().pisteet, 40);
    }

    #[test]
    fn listalle_yltamaton_tulos_ei_kysy_nimea_ja_uusi_peli_alkaa_alusta() {
        let taysi = (0..3)
            .map(|_| Tulos {
                nimi: "A".to_string(),
                pisteet: 1000,
            })
            .collect::<Vec<_>>();
        let mut peli = paattyva(40);
        peli.netti_parhaat = Some(taysi);
        peli.edista(ASKEL_MS);
        assert_eq!(peli.vaihe, Vaihe::Tappio);
        assert_eq!(peli.lahetys, None);

        peli.kasittele(Syote::Kirjain('K'));
        assert_eq!(peli.vaihe, Vaihe::Pelissa);
        assert_eq!(peli.lauta.pisteet, 0);
        assert_eq!(peli.paras, 40);
        assert!(peli.pamaukset.is_empty());
    }

    #[test]
    fn esc_kirjaa_nimen_oletuksella_ja_vahvistus_ei_toistu() {
        let mut peli = paattyva(40);
        peli.edista(ASKEL_MS);
        peli.kasittele(Syote::Peru);
        assert!(!peli.lopeta);
        assert_eq!(peli.vaihe, Vaihe::Tappio);
        assert_eq!(peli.parhaat[0].nimi, "NIMETÖN");
        peli.vahvista_nimi();
        assert_eq!(peli.parhaat.len(), 1);
    }

    #[test]
    fn esc_kesken_pelin_pysayttaa_ja_toinen_kysyy_nimen() {
        let mut peli = paattyva(500);
        peli.kasittele(Syote::Peru);
        assert_eq!(peli.vaihe, Vaihe::Tauko);
        assert!(!peli.lopeta);
        peli.kasittele(Syote::Peru);
        assert_eq!(peli.vaihe, Vaihe::Nimi);
        assert!(!peli.lopeta);
        // Itikka madon edessä ei saa päättää peliä nimen kirjoituksen aikana.
        peli.edista(1000);
        assert!(!peli.lauta.ohi);

        peli.kasittele(Syote::Kirjain('B'));
        peli.kasittele(Syote::Ammu);
        assert!(peli.lopeta);
        assert_eq!(peli.parhaat[0].nimi, "B");
        assert_eq!(peli.lahetys.as_ref().unwrap().pisteet, 500);
    }

    #[test]
    fn listalle_yltamaton_tulos_poistuu_heti_ja_sulkeminen_kirjaa_oletusnimella() {
        let mut peli = paattyva(500);
        peli.netti_parhaat = Some(vec![
            Tulos {
                nimi: "A".to_string(),
                pisteet: 1000,
            };
            3
        ]);
        peli.kasittele(Syote::Peru);
        assert!(!peli.lopeta);
        peli.kasittele(Syote::Peru);
        assert!(peli.lopeta);
        assert_eq!(peli.lahetys, None);

        let mut peli = paattyva(500);
        peli.poistu();
        peli.vahvista_nimi();
        assert_eq!(peli.parhaat[0].nimi, "NIMETÖN");
    }

    #[test]
    fn lopun_napit_ja_esc_toimivat() {
        let mut peli = Silmukka::new(1);
        peli.vaihe = Vaihe::Tappio;
        let (x, y) = piirto::napin_keskipiste(Nappi::Uudestaan);
        peli.kasittele(Syote::Klikkaus(x, y));
        assert_eq!(peli.vaihe, Vaihe::Pelissa);

        peli.vaihe = Vaihe::Tappio;
        let (x, y) = piirto::napin_keskipiste(Nappi::Valikko);
        peli.kasittele(Syote::Klikkaus(x, y));
        assert!(peli.lopeta);

        // Esc pysäyttää, välilyönti jatkaa ja tauon aikana Esc palaa valikkoon.
        let mut peli = pelissa();
        peli.kasittele(Syote::Peru);
        assert_eq!(peli.vaihe, Vaihe::Tauko);
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.vaihe, Vaihe::Pelissa);
        peli.kasittele(Syote::Peru);
        assert!(!peli.lopeta);
        peli.kasittele(Syote::Peru);
        assert!(peli.lopeta);
    }

    #[test]
    fn kultaomena_soi_ja_antaa_lisaa_pisteita() {
        let mut peli = pelissa();
        let paa = peli.lauta.paa();
        peli.lauta.herkut = vec![Herkku {
            paikka: (paa.0 + 1, paa.1),
            kulta_ms: Some(1000),
        }];
        peli.lauta.odotus_ms = ASKEL_MS;
        peli.edista(ASKEL_MS);
        assert_eq!(peli.lauta.pisteet, 100);
        assert_eq!(peli.aanijono[0], (Aani::Yhdistys, 0));
    }
}
