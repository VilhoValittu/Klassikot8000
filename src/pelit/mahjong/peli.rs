use klassikot8000::Arpa;
use klassikot8000::mahjong::{
    Laatta, Lauta, SEKOITUKSEN_HINTA, SUURIN_KETJU, Tila, VIHJEEN_HINTA, Vaikeus,
};

use super::piirto;
use crate::kaynnistin::tallennus::{Asetukset, Tallennus};
use crate::yhteinen::aanet::{Aanet, Aani, PUHE_VIIVE_MS};
use crate::yhteinen::kieli::{kaanna, muotoile};
use crate::yhteinen::pelinakyma::Pelinakyma;
use crate::yhteinen::piirto::{Kehys, VILKKU_MS};
use crate::yhteinen::syote::{Syote, Yleinen, yleinen};
use crate::yhteinen::tulokset::{Nettipeli, Tulostaulu, muokkaa_nimea};

/// Otetun parin pisteet nousevat laudalta näin kauan.
pub const PONNAHDUS_MS: u32 = 900;
/// Voiton ilotulitus kestää näin kauan, sitten kuva pysähtyy.
pub const JUHLA_MS: u32 = 4000;
const LOPPU_VIIVE_MS: u32 = 250;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vaihe {
    Alku,
    Pelissa,
    /// Tulos pääsi kolmen parhaan joukkoon ja odottaa nimimerkkiä.
    Nimi,
    Voitto,
    Tappio,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nappi {
    Vihje,
    Sekoita,
    Peruuta,
    Uusi,
    Valikko,
}

/// Juuri otettu pari: sen paikalta nousevat pisteet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ponnahdus {
    pub paikat: (usize, usize),
    pub laatat: (Laatta, Laatta),
    pub pisteet: u32,
    pub ketju: u32,
    pub alku: u32,
}

pub struct Mahjong {
    pub vaihe: Vaihe,
    pub vaikeus: Vaikeus,
    pub lauta: Lauta,
    pub valittu: Option<usize>,
    /// Vihjeen pari vilkkuu, kunnes pelaaja tekee jotain.
    pub vihje: Option<(usize, usize)>,
    pub hiiren_laatta: Option<usize>,
    pub kursori: usize,
    /// Kursori näkyy vasta, kun näppäimistöä on käytetty.
    pub kursori_nakyy: bool,
    /// Tosi, kun uusi tulos odottaa tiedostoon kirjoitusta.
    pub uusi_tulos: bool,
    pub taulu: Tulostaulu,
    pub nimi: String,
    pub viesti: String,
    pub ponnahdus: Option<Ponnahdus>,
    pub voitto_alku: u32,
    pub lopeta: bool,
    pub aanet: Aanet,
    pub kello_ms: u32,
    /// Pelattu aika: kello käy vain kesken olevassa pelissä.
    pub peliaika_ms: u32,
    arpa: Arpa,
}

fn tason_indeksi(vaikeus: Vaikeus) -> usize {
    Vaikeus::KAIKKI
        .iter()
        .position(|&taso| taso == vaikeus)
        .unwrap_or(0)
}

impl Mahjong {
    pub fn new(siemen: u64) -> Self {
        let mut arpa = Arpa::new(siemen);
        Self {
            vaihe: Vaihe::Alku,
            vaikeus: Vaikeus::Helppo,
            lauta: Lauta::new(Vaikeus::Helppo, &mut arpa),
            valittu: None,
            vihje: None,
            hiiren_laatta: None,
            kursori: 0,
            kursori_nakyy: false,
            uusi_tulos: false,
            taulu: Tulostaulu::new(Nettipeli::Mahjong),
            nimi: String::new(),
            viesti: String::new(),
            ponnahdus: None,
            voitto_alku: 0,
            lopeta: false,
            aanet: Aanet::default(),
            kello_ms: 0,
            peliaika_ms: 0,
            arpa,
        }
    }

    fn juhlii(&self) -> bool {
        matches!(self.vaihe, Vaihe::Voitto | Vaihe::Nimi)
            && self.lauta.tila == Tila::Voitto
            && self.kello_ms.saturating_sub(self.voitto_alku) < JUHLA_MS
    }

    /// Kuva riippuu tästä ja syötteestä. `None`, kun kuva elää joka ruudussa: alkuruudun
    /// malli pyörii, pisteet nousevat tai ilotulitus paukkuu.
    pub fn kuvan_askel(&self) -> Option<(u32, u32)> {
        if self.vaihe == Vaihe::Alku || self.ponnahdus.is_some() || self.juhlii() {
            return None;
        }
        let vilkkuu = self.vaihe == Vaihe::Pelissa && (self.kursori_nakyy || self.vihje.is_some());
        let vilkku = if vilkkuu || self.vaihe == Vaihe::Nimi {
            self.kello_ms / VILKKU_MS
        } else {
            0
        };
        Some((self.peliaika_ms / 1000, vilkku))
    }

    pub fn edista(&mut self, ms: u32) {
        self.kello_ms = self.kello_ms.saturating_add(ms);
        if self.vaihe == Vaihe::Pelissa {
            self.peliaika_ms = self.peliaika_ms.saturating_add(ms);
        }
        if self
            .ponnahdus
            .is_some_and(|p| self.kello_ms >= p.alku.saturating_add(PONNAHDUS_MS))
        {
            self.ponnahdus = None;
        }
    }

    pub fn aloita(&mut self) {
        self.lauta = Lauta::new(self.vaikeus, &mut self.arpa);
        self.vaihe = Vaihe::Pelissa;
        self.valittu = None;
        self.vihje = None;
        self.hiiren_laatta = None;
        self.kursori_nakyy = false;
        self.kursori = self.ensimmainen_vapaa().unwrap_or(0);
        self.ponnahdus = None;
        self.peliaika_ms = 0;
        self.viesti = kaanna("mahjong.aloita").to_string();
        self.aanet.jono.push((Aani::Alku, 0));
    }

    fn ensimmainen_vapaa(&self) -> Option<usize> {
        let vapaat = self.lauta.vapaat();
        piirto::piirtojarjestys(&self.lauta)
            .into_iter()
            .rev()
            .find(|&i| vapaat[i])
    }

    pub fn kasittele(&mut self, syote: Syote) {
        let vaihe = self.vaihe;
        let kytkimella = |x, y| piirto::aanikytkimella(vaihe, x, y);
        match yleinen(
            syote,
            vaihe == Vaihe::Nimi,
            &mut self.aanet.paalla,
            kytkimella,
        ) {
            Yleinen::Peli => {}
            Yleinen::Nimi => return self.kasittele_nimi(syote),
            Yleinen::Peru => {
                if self.vaihe == Vaihe::Pelissa && (self.valittu.is_some() || self.vihje.is_some())
                {
                    self.valittu = None;
                    self.vihje = None;
                } else {
                    self.lopeta = true;
                }
                return;
            }
            Yleinen::Aanet | Yleinen::Ohita => return,
        }
        match self.vaihe {
            Vaihe::Alku => self.kasittele_alku(syote),
            Vaihe::Pelissa => self.kasittele_pelissa(syote),
            Vaihe::Voitto | Vaihe::Tappio => self.kasittele_loppu(syote),
            Vaihe::Nimi => {}
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

    /// Kirjaa nimeä odottavan tuloksen.
    pub fn vahvista_nimi(&mut self) {
        if self.vaihe != Vaihe::Nimi {
            return;
        }
        let pisteet = self.lauta.pisteet;
        self.taulu.kirjaa(&mut self.nimi, pisteet);
        self.uusi_tulos = true;
        self.vaihe = self.loppuvaihe();
    }

    fn loppuvaihe(&self) -> Vaihe {
        if self.lauta.tila == Tila::Voitto {
            Vaihe::Voitto
        } else {
            Vaihe::Tappio
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
            self.aloita();
        }
    }

    fn nappi_kirjaimesta(syote: Syote) -> Option<Nappi> {
        match syote {
            Syote::Kirjain('H') => Some(Nappi::Vihje),
            Syote::Kirjain('S') => Some(Nappi::Sekoita),
            Syote::Kirjain('U') => Some(Nappi::Peruuta),
            Syote::Kirjain('K') => Some(Nappi::Uusi),
            Syote::Kirjain('E') => Some(Nappi::Valikko),
            _ => None,
        }
    }

    fn kasittele_pelissa(&mut self, syote: Syote) {
        let nappi = match syote {
            Syote::Klikkaus(x, y) => piirto::nappi_pisteesta(x, y),
            _ => Self::nappi_kirjaimesta(syote),
        };
        if let Some(nappi) = nappi {
            self.paina(nappi);
            return;
        }
        match syote {
            Syote::Hiiri(x, y) => {
                self.hiiren_laatta = piirto::laatta_pisteesta(&self.lauta, x, y);
                self.kursori_nakyy = false;
            }
            Syote::Klikkaus(x, y) => {
                if let Some(i) = piirto::laatta_pisteesta(&self.lauta, x, y) {
                    self.kursori = i;
                    self.kursori_nakyy = false;
                    self.valitse(i);
                }
            }
            Syote::OikeaKlikkaus(..) => {
                self.valittu = None;
                self.vihje = None;
            }
            Syote::Nuoli(drivi, dsarake) => {
                // Ensimmäinen painallus vain tuo kursorin näkyviin.
                if self.kursori_nakyy {
                    if let Some(i) = piirto::naapuri(&self.lauta, self.kursori, drivi, dsarake) {
                        self.kursori = i;
                    }
                } else if !self.lauta.vapaa(self.kursori) {
                    self.kursori = self.ensimmainen_vapaa().unwrap_or(0);
                }
                self.kursori_nakyy = true;
            }
            Syote::Ammu => {
                if self.kursori_nakyy {
                    self.valitse(self.kursori);
                } else if !self.lauta.vapaa(self.kursori) {
                    self.kursori = self.ensimmainen_vapaa().unwrap_or(0);
                }
                self.kursori_nakyy = true;
            }
            _ => {}
        }
    }

    pub fn paina(&mut self, nappi: Nappi) {
        match nappi {
            Nappi::Vihje => self.nayta_vihje(),
            Nappi::Sekoita => self.sekoita(),
            Nappi::Peruuta => self.peru(),
            Nappi::Uusi => self.aloita(),
            Nappi::Valikko => self.lopeta = true,
        }
    }

    /// Ensimmäinen klikkaus valitsee, toinen ottaa parin tai vaihtaa valinnan.
    pub fn valitse(&mut self, i: usize) {
        if self.lauta.laatat.get(i).is_none_or(Option::is_none) {
            return;
        }
        if !self.lauta.vapaa(i) {
            self.viesti = kaanna("mahjong.ei_vapaa").to_string();
            self.aanet.jono.push((Aani::Virhe, 0));
            return;
        }
        match self.valittu {
            None => {
                self.valittu = Some(i);
                self.aanet.jono.push((Aani::Tiili, 0));
            }
            Some(j) if j == i => self.valittu = None,
            Some(j) if self.lauta.kelpaa(j, i) => self.ota_pari(j, i),
            Some(_) => {
                self.valittu = Some(i);
                self.viesti = kaanna("mahjong.ei_pari").to_string();
                self.aanet.jono.push((Aani::Tiili, 0));
            }
        }
    }

    fn ota_pari(&mut self, a: usize, b: usize) {
        let laatat = match (self.lauta.laatat[a], self.lauta.laatat[b]) {
            (Some(x), Some(y)) => (x, y),
            _ => return,
        };
        let Some(pisteet) = self.lauta.poista(a, b, self.peliaika_ms) else {
            return;
        };
        self.valittu = None;
        self.vihje = None;
        let ketju = self.lauta.ketju;
        self.ponnahdus = Some(Ponnahdus {
            paikat: (a, b),
            laatat,
            pisteet,
            ketju,
            alku: self.kello_ms,
        });
        self.viesti = if ketju > 1 {
            muotoile("mahjong.ketju_viesti", &[&ketju, &pisteet])
        } else {
            muotoile("mahjong.pari", &[&pisteet])
        };
        self.aanet.jono.push((Aani::Yhdistys, 0));
        if ketju > 1 {
            let helina = [
                Aani::Helina1,
                Aani::Helina2,
                Aani::Helina3,
                Aani::Helina4,
                Aani::Helina5,
                Aani::Helina6,
            ];
            let i = (ketju.min(SUURIN_KETJU) as usize).saturating_sub(2);
            self.aanet.jono.push((helina[i.min(helina.len() - 1)], 60));
        }
        if self.hiiren_laatta.is_some_and(|i| i == a || i == b) {
            self.hiiren_laatta = None;
        }
        match self.lauta.tila {
            Tila::Kesken => {}
            Tila::Jumissa => {
                self.viesti = muotoile("mahjong.jumissa", &[&SEKOITUKSEN_HINTA]);
                self.aanet.jono.push((Aani::EiSiirtoja, LOPPU_VIIVE_MS));
            }
            Tila::Voitto => {
                self.voitto_alku = self.kello_ms;
                self.viesti = muotoile("mahjong.voitto_viesti", &[&self.lauta.pisteet]);
                self.aanet.jono.push((Aani::Voitto, LOPPU_VIIVE_MS));
                self.aanet
                    .jono
                    .push((Aani::Voitit, LOPPU_VIIVE_MS + PUHE_VIIVE_MS));
                self.paata();
            }
            Tila::Tappio => {
                self.viesti = muotoile("mahjong.tappio_viesti", &[&self.lauta.pisteet]);
                self.aanet.jono.push((Aani::Tappio, LOPPU_VIIVE_MS));
                self.aanet
                    .jono
                    .push((Aani::EiSiirtoja, LOPPU_VIIVE_MS + PUHE_VIIVE_MS));
                self.paata();
            }
        }
        if !self.lauta.vapaa(self.kursori) {
            self.kursori = self.ensimmainen_vapaa().unwrap_or(self.kursori);
        }
    }

    /// Päättynyt peli kirjaa ennätyksen ja kysyy nimen, jos tulos yltää listalle.
    fn paata(&mut self) {
        let pisteet = self.lauta.pisteet;
        if pisteet > self.taulu.paras {
            self.taulu.paras = pisteet;
            self.uusi_tulos = true;
        }
        self.vaihe = self.loppuvaihe();
        if self.taulu.mahtuu(pisteet) {
            self.nimi.clear();
            self.vaihe = Vaihe::Nimi;
        }
    }

    fn nayta_vihje(&mut self) {
        match self.lauta.vihje() {
            Some(pari) => {
                self.vihje = Some(pari);
                self.valittu = None;
                self.viesti = muotoile("mahjong.vihje", &[&VIHJEEN_HINTA]);
                self.aanet.jono.push((Aani::Pomppu, 0));
            }
            None => {
                self.viesti = muotoile("mahjong.jumissa", &[&SEKOITUKSEN_HINTA]);
                self.aanet.jono.push((Aani::Virhe, 0));
            }
        }
    }

    fn sekoita(&mut self) {
        if self.lauta.sekoita(&mut self.arpa) {
            self.valittu = None;
            self.vihje = None;
            self.ponnahdus = None;
            self.aanet.jono.push((Aani::Liuku, 0));
            if self.lauta.tila == Tila::Tappio {
                self.viesti = muotoile("mahjong.tappio_viesti", &[&self.lauta.pisteet]);
                self.aanet.jono.push((Aani::Tappio, LOPPU_VIIVE_MS));
                self.aanet
                    .jono
                    .push((Aani::EiSiirtoja, LOPPU_VIIVE_MS + PUHE_VIIVE_MS));
                self.paata();
            } else {
                self.viesti = muotoile("mahjong.sekoitettu", &[&SEKOITUKSEN_HINTA]);
            }
        } else {
            self.viesti = kaanna("mahjong.ei_sekoituksia").to_string();
            self.aanet.jono.push((Aani::Virhe, 0));
        }
    }

    fn peru(&mut self) {
        if self.lauta.peru() {
            self.valittu = None;
            self.vihje = None;
            self.ponnahdus = None;
            self.viesti = kaanna("mahjong.peruttu").to_string();
            self.aanet.jono.push((Aani::Liuku, 0));
        } else {
            let avain = if self.lauta.vaikeus.voi_perua() {
                "mahjong.ei_peruttavaa"
            } else {
                "mahjong.ei_perua_vaikealla"
            };
            self.viesti = kaanna(avain).to_string();
            self.aanet.jono.push((Aani::Virhe, 0));
        }
    }

    fn kasittele_loppu(&mut self, syote: Syote) {
        let nappi = match syote {
            Syote::Klikkaus(x, y) => piirto::nappi_pisteesta(x, y),
            _ => Self::nappi_kirjaimesta(syote),
        };
        match nappi {
            Some(Nappi::Uusi) => self.aloita(),
            Some(Nappi::Valikko) => self.lopeta = true,
            _ => {}
        }
    }
}

impl Pelinakyma for Mahjong {
    fn lue_tallennuksesta(&mut self, tallennus: &Tallennus) {
        self.vaikeus = tallennus.asetukset.mahjong;
    }

    fn kirjaa_asetukset(&self, asetukset: &mut Asetukset) {
        asetukset.mahjong = self.vaikeus;
    }

    fn kasittele(&mut self, syote: Syote) {
        Mahjong::kasittele(self, syote);
    }

    fn edista(&mut self, ms: u32) {
        Mahjong::edista(self, ms);
    }

    fn piirra(&self, kehys: &mut Kehys) {
        if self.suoraan_ruudulle() {
            piirto::piirra(self, &mut kehys.ruutu);
            kehys.piirretty_suoraan();
        } else {
            piirto::piirra_alku(self, &mut kehys.kuva);
            kehys.venyta();
        }
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
        self.vahvista_nimi();
    }

    fn listat(&mut self) -> Vec<&mut Tulostaulu> {
        vec![&mut self.taulu]
    }

    fn kuvan_avain(&self) -> Option<(u32, u32)> {
        self.kuvan_askel()
    }

    fn nuolet_toistuvat(&self) -> bool {
        false
    }

    fn suoraan_ruudulle(&self) -> bool {
        self.vaihe != Vaihe::Alku
    }
}

#[cfg(test)]
mod tests;
