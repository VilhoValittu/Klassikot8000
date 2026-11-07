use klassikot8000::Arpa;
use klassikot8000::blackjack::{ALKURAHA, PIENIN_PANOS, Poyta, Tulos, Vuoro, YKSI};

use super::piirto::{self, raha};
use crate::yhteinen::aanet::{Aanet, Aani, PUHE_VIIVE_MS};
use crate::yhteinen::kieli::{kaanna, muotoile};
use crate::yhteinen::pelinakyma::Pelinakyma;
use crate::yhteinen::piirto::{Kehys, VILKKU_MS};
use crate::yhteinen::syote::{Syote, Yleinen, yleinen};
use crate::yhteinen::tulokset::{Nettipeli, Tulostaulu, muokkaa_nimea};

const ALKUPANOS: u32 = 10 * YKSI;
const LISAYKSET: [u32; 4] = [YKSI, 5 * YKSI, 25 * YKSI, 100 * YKSI];
const NUOLEN_ASKEL: i64 = YKSI as i64;
const NUOLEN_ISO_ASKEL: i64 = 10 * YKSI as i64;
const JAKAJAN_SIIRTO_MS: u32 = 550;
const LOPPU_VIIVE_MS: u32 = 250;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vaihe {
    Alku,
    Pelissa,
    /// Rahat loppuivat.
    Loppu,
    /// Pöydästä lähdetään voitolla, joka pääsi kolmen parhaan joukkoon.
    Nimi,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nappi {
    /// Lisää panokseen `LISAYKSET`-taulukon summan.
    Lisaa(usize),
    Pienin,
    Kaikki,
    Jako,
    Vakuuta,
    EiVakuutusta,
    Ota,
    Jaa,
    Tuplaa,
    Halkaise,
    Uudestaan,
    Valikko,
}

pub struct Blackjack {
    pub vaihe: Vaihe,
    pub poyta: Poyta,
    /// Seuraavan jaon panos. Säilyy kierroksesta toiseen, jos rahat riittävät.
    pub panos: u32,
    pub taulu: Tulostaulu,
    /// Kirjoitettavana oleva nimimerkki.
    pub nimi: String,
    /// Tosi, kun tulos odottaa tiedostoon kirjoitusta.
    pub uusi_tulos: bool,
    pub viesti: String,
    /// Pyyntö palata käynnistimeen.
    pub lopeta: bool,
    pub aanet: Aanet,
    pub kello_ms: u32,
    /// Jakajan siirtojen määrä: kuva muuttuu niistä ilman syötettä.
    siirtoja: u32,
    seuraava_siirto_ms: u32,
    arpa: Arpa,
}

impl Blackjack {
    pub fn new(siemen: u64) -> Self {
        let mut arpa = Arpa::new(siemen);
        Self {
            vaihe: Vaihe::Alku,
            poyta: Poyta::new(&mut arpa),
            panos: ALKUPANOS,
            taulu: Tulostaulu::new(Nettipeli::Blackjack),
            nimi: String::new(),
            uusi_tulos: false,
            viesti: String::new(),
            lopeta: false,
            aanet: Aanet::default(),
            kello_ms: 0,
            siirtoja: 0,
            seuraava_siirto_ms: 0,
            arpa,
        }
    }

    /// Kuva riippuu tästä ja syötteestä. `None`, kun alkuruudun malli pyörii.
    pub fn kuvan_askel(&self) -> Option<u32> {
        match self.vaihe {
            Vaihe::Alku => None,
            Vaihe::Nimi => Some(self.kello_ms / VILKKU_MS),
            Vaihe::Pelissa | Vaihe::Loppu => Some(self.siirtoja),
        }
    }

    /// Pöydästä viety voitto on oma ennätys, vaikkei se yltäisi listalle.
    fn kirjaa_paras(&mut self) {
        let pisteet = self.poyta.pisteet();
        if pisteet > self.taulu.paras {
            self.taulu.paras = pisteet;
            self.uusi_tulos = true;
        }
    }

    /// Pöydästä lähtö on ainoa hetki, jolloin voitto lunastetaan. Kesken jäänyt panos jää pöytään.
    pub fn poistu(&mut self) {
        if self.vaihe == Vaihe::Pelissa {
            self.kirjaa_paras();
        }
        if self.vaihe == Vaihe::Pelissa && self.taulu.mahtuu(self.poyta.pisteet()) {
            self.nimi.clear();
            self.vaihe = Vaihe::Nimi;
        } else {
            self.lopeta = true;
        }
    }

    /// Kirjaa odottavan tuloksen ja palaa valikkoon.
    pub fn vahvista_nimi(&mut self) {
        if self.vaihe != Vaihe::Nimi {
            return;
        }
        let pisteet = self.poyta.pisteet();
        self.taulu.kirjaa(&mut self.nimi, pisteet);
        self.uusi_tulos = true;
        self.lopeta = true;
    }

    fn kasittele_nimi(&mut self, syote: Syote) {
        match syote {
            // Viimeisenä painettu välilyönti ei saa kuitata nimeä ennen kuin sitä on kirjoitettu.
            Syote::Ammu if self.nimi.is_empty() => {}
            Syote::Ammu | Syote::Peru => self.vahvista_nimi(),
            _ => muokkaa_nimea(&mut self.nimi, syote),
        }
    }

    pub fn edista(&mut self, ms: u32) {
        self.kello_ms = self.kello_ms.saturating_add(ms);
        while self.vaihe == Vaihe::Pelissa
            && self.poyta.vuoro == Vuoro::Jakaja
            && self.kello_ms >= self.seuraava_siirto_ms
        {
            self.poyta.jakajan_siirto();
            self.siirtoja = self.siirtoja.wrapping_add(1);
            self.seuraava_siirto_ms = self.seuraava_siirto_ms.saturating_add(JAKAJAN_SIIRTO_MS);
            self.aanet.jono.push((Aani::Liuku, 0));
            if self.poyta.vuoro == Vuoro::Panostus {
                self.kierros_paattyi();
            }
        }
    }

    fn aloita(&mut self) {
        self.poyta = Poyta::new(&mut self.arpa);
        self.panos = ALKUPANOS;
        self.vaihe = Vaihe::Pelissa;
        self.viesti = kaanna("blackjack.aseta_panos").to_string();
        self.aanet.jono.push((Aani::Alku, 0));
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
            Yleinen::Peru => return self.poistu(),
            Yleinen::Aanet | Yleinen::Ohita => return,
        }
        if self.vaihe == Vaihe::Alku {
            if matches!(syote, Syote::Ammu | Syote::Klikkaus(..)) {
                self.aloita();
            }
            return;
        }
        if let (Syote::Nuoli(drivi, dsarake), Vuoro::Panostus, Vaihe::Pelissa) =
            (syote, self.poyta.vuoro, self.vaihe)
        {
            self.muuta_panosta(dsarake as i64 * NUOLEN_ASKEL - drivi as i64 * NUOLEN_ISO_ASKEL);
        } else if let Some(nappi) = self.syotteen_nappi(syote) {
            self.paina(nappi);
        }
    }

    /// Sama kirjain voi tarkoittaa eri asiaa panostaessa ja pelatessa, kuten napitkin.
    fn syotteen_nappi(&self, syote: Syote) -> Option<Nappi> {
        if let Syote::Klikkaus(x, y) = syote {
            return piirto::nappi_pisteesta(self.vaihe, self.poyta.vuoro, x, y);
        }
        if syote == Syote::Kirjain('E') {
            return Some(Nappi::Valikko);
        }
        match (self.vaihe, self.poyta.vuoro, syote) {
            (Vaihe::Loppu, _, Syote::Kirjain('K')) => Some(Nappi::Uudestaan),
            (Vaihe::Pelissa, Vuoro::Panostus, Syote::Numero(numero)) => (numero as usize)
                .checked_sub(1)
                .filter(|&i| i < LISAYKSET.len())
                .map(Nappi::Lisaa),
            (Vaihe::Pelissa, Vuoro::Panostus, Syote::Kirjain('T')) => Some(Nappi::Pienin),
            (Vaihe::Pelissa, Vuoro::Panostus, Syote::Kirjain('K')) => Some(Nappi::Kaikki),
            (Vaihe::Pelissa, Vuoro::Panostus, Syote::Ammu) => Some(Nappi::Jako),
            (Vaihe::Pelissa, Vuoro::Vakuutus, Syote::Kirjain('V')) => Some(Nappi::Vakuuta),
            (Vaihe::Pelissa, Vuoro::Vakuutus, Syote::Kirjain('N')) => Some(Nappi::EiVakuutusta),
            (Vaihe::Pelissa, Vuoro::Pelaaja, Syote::Kirjain('O')) => Some(Nappi::Ota),
            (Vaihe::Pelissa, Vuoro::Pelaaja, Syote::Kirjain('J')) => Some(Nappi::Jaa),
            (Vaihe::Pelissa, Vuoro::Pelaaja, Syote::Kirjain('T')) => Some(Nappi::Tuplaa),
            (Vaihe::Pelissa, Vuoro::Pelaaja, Syote::Kirjain('H')) => Some(Nappi::Halkaise),
            _ => None,
        }
    }

    fn paina(&mut self, nappi: Nappi) {
        match nappi {
            Nappi::Lisaa(i) => self.muuta_panosta(LISAYKSET[i] as i64),
            Nappi::Pienin => self.panos = PIENIN_PANOS,
            Nappi::Kaikki => self.panos = self.suurin_panos(),
            Nappi::Jako => self.jaa_kortit(),
            Nappi::Vakuuta => self.vakuutuspaatos(true),
            Nappi::EiVakuutusta => self.vakuutuspaatos(false),
            Nappi::Ota => {
                self.poyta.ota();
                self.pelaaja_siirsi();
            }
            Nappi::Jaa => {
                self.poyta.jaa_tahan();
                self.pelaaja_siirsi();
            }
            Nappi::Tuplaa => {
                if self.poyta.tuplaa() {
                    self.pelaaja_siirsi();
                } else {
                    self.virhe("blackjack.ei_tuplausta");
                }
            }
            Nappi::Halkaise => {
                if self.poyta.halkaise() {
                    self.pelaaja_siirsi();
                } else {
                    self.virhe("blackjack.ei_halkaisua");
                }
            }
            Nappi::Uudestaan => self.aloita(),
            Nappi::Valikko => self.poistu(),
        }
    }

    /// Panos on aina kokonaisia rahoja, vaikka rahoissa olisi puolikas.
    fn suurin_panos(&self) -> u32 {
        self.poyta.rahat - self.poyta.rahat % YKSI
    }

    fn muuta_panosta(&mut self, muutos: i64) {
        let uusi = self.panos as i64 + muutos;
        self.panos = uusi.clamp(PIENIN_PANOS as i64, self.suurin_panos() as i64) as u32;
    }

    fn jaa_kortit(&mut self) {
        if !self.poyta.jaa(self.panos, &mut self.arpa) {
            return;
        }
        self.viesti.clear();
        self.aanet.jono.push((Aani::Liuku, 0));
        if self.poyta.vuoro == Vuoro::Panostus {
            self.kierros_paattyi();
        } else if self.poyta.vuoro == Vuoro::Vakuutus {
            self.viesti = if self.poyta.kadet[0].on_blackjack() {
                muotoile("blackjack.tasaraha_tarjous", &[&raha(self.panos)])
            } else {
                let hinta = raha(self.poyta.vakuutuksen_hinta());
                muotoile("blackjack.vakuutus_tarjous", &[&hinta])
            };
        }
    }

    fn vakuutuspaatos(&mut self, ottaa: bool) {
        self.poyta.vakuuta(ottaa);
        self.viesti.clear();
        self.aanet.jono.push((Aani::Liuku, 0));
        if self.poyta.vuoro == Vuoro::Panostus {
            self.kierros_paattyi();
        } else if ottaa {
            self.viesti = kaanna("blackjack.vakuutus_havisi").to_string();
        }
    }

    fn virhe(&mut self, avain: &str) {
        self.viesti = kaanna(avain).to_string();
        self.aanet.jono.push((Aani::Virhe, 0));
    }

    fn pelaaja_siirsi(&mut self) {
        self.viesti.clear();
        self.aanet.jono.push((Aani::Liuku, 0));
        if self.poyta.vuoro == Vuoro::Jakaja {
            self.seuraava_siirto_ms = self.kello_ms.saturating_add(JAKAJAN_SIIRTO_MS);
        }
    }

    fn kierros_paattyi(&mut self) {
        self.panos = self.panos.min(self.suurin_panos()).max(PIENIN_PANOS);
        let voitto = self.poyta.voitto();
        let maara = raha(voitto.unsigned_abs() as u32);
        let blackjack = (self.poyta.kadet.iter()).any(|k| k.tulos == Some(Tulos::Blackjack));
        // Pelkkä puolikas ei riitä pienimpään panokseen.
        if self.poyta.rahat < PIENIN_PANOS {
            self.vaihe = Vaihe::Loppu;
            self.viesti = muotoile("blackjack.konkurssi", &[&raha(ALKURAHA)]);
            self.aanet.jono.push((Aani::Tappio, LOPPU_VIIVE_MS));
            self.aanet
                .jono
                .push((Aani::PeliOhi, LOPPU_VIIVE_MS + PUHE_VIIVE_MS));
        } else if self.poyta.vakuutus_voitti() {
            let maksu = raha(2 * self.poyta.vakuutus);
            self.viesti = muotoile("blackjack.viesti_vakuutus", &[&maksu, &maara]);
            self.aanet.jono.push((Aani::Yhdistys, LOPPU_VIIVE_MS));
        } else if blackjack {
            self.viesti = muotoile("blackjack.viesti_blackjack", &[&maara]);
            self.aanet.jono.push((Aani::Voitto, LOPPU_VIIVE_MS));
        } else if voitto > 0 {
            self.viesti = muotoile("blackjack.viesti_voitto", &[&maara]);
            self.aanet.jono.push((Aani::Yhdistys, LOPPU_VIIVE_MS));
        } else if voitto < 0 {
            self.viesti = muotoile("blackjack.viesti_tappio", &[&maara]);
            self.aanet.jono.push((Aani::Virhe, LOPPU_VIIVE_MS));
        } else {
            self.viesti = kaanna("blackjack.viesti_tasan").to_string();
            self.aanet.jono.push((Aani::Pomppu, LOPPU_VIIVE_MS));
        }
    }
}

impl Pelinakyma for Blackjack {
    fn kasittele(&mut self, syote: Syote) {
        Blackjack::kasittele(self, syote);
    }

    fn edista(&mut self, ms: u32) {
        Blackjack::edista(self, ms);
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
        self.poistu();
        self.vahvista_nimi();
    }

    fn listat(&mut self) -> Vec<&mut Tulostaulu> {
        vec![&mut self.taulu]
    }

    fn kuvan_avain(&self) -> Option<(u32, u32)> {
        self.kuvan_askel().map(|askel| (askel, 0))
    }

    fn suoraan_ruudulle(&self) -> bool {
        self.vaihe != Vaihe::Alku
    }
}

#[cfg(test)]
mod tests;
