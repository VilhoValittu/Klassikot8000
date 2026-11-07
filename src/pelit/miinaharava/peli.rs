use klassikot8000::Arpa;
use klassikot8000::miinaharava::{Avaus, Miinakentta, Paikka, SUURIN_AIKA, Vaikeus, aikapisteet};

use super::piirto;
use crate::kaynnistin::tallennus::{Asetukset, Tallennus};
use crate::yhteinen::aanet::{Aanet, Aani, PUHE_VIIVE_MS};
use crate::yhteinen::kieli::{kaanna, muotoile};
use crate::yhteinen::pelinakyma::Pelinakyma;
use crate::yhteinen::piirto::{Kehys, VILKKU_MS, aanikytkimella};
use crate::yhteinen::syote::{Syote, Yleinen, yleinen};
use crate::yhteinen::tulokset::{Nettipeli, Tulostaulu, muokkaa_nimea};

pub const RAJAHDYS_MS: u32 = 500;
const LOPPU_VIIVE_MS: u32 = 250;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vaihe {
    Alku,
    Pelissa,
    /// Voitto pääsi kolmen parhaan joukkoon ja odottaa nimimerkkiä.
    Nimi,
    Voitto,
    Tappio,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nappi {
    Uudestaan,
    Valikko,
}

pub struct Miinaharava {
    pub vaihe: Vaihe,
    pub vaikeus: Vaikeus,
    pub lauta: Miinakentta,
    /// Parhaat ajat sekunteina tasoittain. Nolla tarkoittaa, ettei tasoa ole läpäisty.
    pub ajat: [u32; 3],
    /// Tosi, kun uusi ennätys odottaa tiedostoon kirjoitusta.
    pub uusi_tulos: bool,
    pub taulu: Tulostaulu,
    /// Kirjoitettavana oleva nimimerkki.
    pub nimi: String,
    pub viesti: String,
    pub kursori: Paikka,
    /// Kursori näkyy vasta, kun näppäimistöä on käytetty.
    pub kursori_nakyy: bool,
    pub hiiren_ruutu: Option<Paikka>,
    /// Miinan räjähdyksen alkuhetki.
    pub rajahdys: Option<u32>,
    /// Pyyntö palata käynnistimeen.
    pub lopeta: bool,
    pub aanet: Aanet,
    pub kello_ms: u32,
    /// Ensimmäisen avauksen hetki: kello käy siitä asti.
    aika_alku: Option<u32>,
    /// Päättyneen pelin aika sekunteina.
    loppuaika: Option<u32>,
    arpa: Arpa,
}

fn tason_indeksi(vaikeus: Vaikeus) -> usize {
    Vaikeus::KAIKKI
        .iter()
        .position(|&taso| taso == vaikeus)
        .unwrap_or(0)
}

impl Miinaharava {
    pub fn new(siemen: u64) -> Self {
        Self {
            vaihe: Vaihe::Alku,
            vaikeus: Vaikeus::Helppo,
            lauta: Miinakentta::new(Vaikeus::Helppo),
            ajat: [0; 3],
            uusi_tulos: false,
            taulu: Tulostaulu::new(Nettipeli::Miinaharava),
            nimi: String::new(),
            viesti: String::new(),
            kursori: (0, 0),
            kursori_nakyy: false,
            hiiren_ruutu: None,
            rajahdys: None,
            lopeta: false,
            aanet: Aanet::default(),
            kello_ms: 0,
            aika_alku: None,
            loppuaika: None,
            arpa: Arpa::new(siemen),
        }
    }

    /// Kuva riippuu tästä ja syötteestä. `None`, kun alkuruudun malli pyörii.
    /// Paikallaan oleva kenttä muuttuu vain kellon ja vilkun tahdissa.
    pub fn kuvan_askel(&self) -> Option<u32> {
        if self.vaihe == Vaihe::Alku {
            None
        } else if self.rajahdys.is_some() {
            Some(self.kello_ms)
        } else {
            Some(self.kello_ms / VILKKU_MS)
        }
    }

    pub fn aika(&self) -> u32 {
        let kulunut = self
            .aika_alku
            .map_or(0, |alku| self.kello_ms.saturating_sub(alku) / 1000);
        self.loppuaika.unwrap_or(kulunut).min(SUURIN_AIKA)
    }

    /// Päättyneen pelin pisteet top-listalle.
    pub fn pisteet(&self) -> u32 {
        aikapisteet(self.vaikeus, self.aika())
    }

    /// Kirjaa nimeä odottavan voiton.
    pub fn vahvista_nimi(&mut self) {
        if self.vaihe != Vaihe::Nimi {
            return;
        }
        let pisteet = self.pisteet();
        self.taulu.kirjaa(&mut self.nimi, pisteet);
        self.uusi_tulos = true;
        self.vaihe = Vaihe::Voitto;
    }

    fn kasittele_nimi(&mut self, syote: Syote) {
        match syote {
            // Avaukseen painettu välilyönti ei saa kuitata nimeä ennen kuin sitä on kirjoitettu.
            Syote::Ammu if self.nimi.is_empty() => {}
            Syote::Ammu | Syote::Peru => self.vahvista_nimi(),
            _ => muokkaa_nimea(&mut self.nimi, syote),
        }
    }

    /// Nykyisen tason paras aika, jos taso on läpäisty.
    pub fn paras_aika(&self) -> Option<u32> {
        Some(self.ajat[tason_indeksi(self.vaikeus)]).filter(|&aika| aika > 0)
    }

    pub fn edista(&mut self, ms: u32) {
        self.kello_ms = self.kello_ms.saturating_add(ms);
        if self
            .rajahdys
            .is_some_and(|alku| self.kello_ms >= alku.saturating_add(RAJAHDYS_MS))
        {
            self.rajahdys = None;
        }
    }

    pub fn aloita(&mut self) {
        self.lauta = Miinakentta::new(self.vaikeus);
        self.vaihe = Vaihe::Pelissa;
        self.kursori = (self.vaikeus.korkeus() / 2, self.vaikeus.leveys() / 2);
        self.kursori_nakyy = false;
        self.hiiren_ruutu = None;
        self.rajahdys = None;
        self.aika_alku = None;
        self.loppuaika = None;
        self.viesti = kaanna("miinaharava.aloita").to_string();
        self.aanet.jono.push((Aani::Alku, 0));
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
            Yleinen::Peru => {
                self.lopeta = true;
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

    fn kasittele_pelissa(&mut self, syote: Syote) {
        let koko = (self.vaikeus.korkeus(), self.vaikeus.leveys());
        match syote {
            Syote::Hiiri(x, y) => {
                self.hiiren_ruutu = piirto::ruutu_pisteesta(self.vaikeus, x, y);
                self.kursori_nakyy = false;
            }
            Syote::Klikkaus(x, y) => {
                if let Some(ruutu) = piirto::ruutu_pisteesta(self.vaikeus, x, y) {
                    self.kursori = ruutu;
                    self.kursori_nakyy = false;
                    self.avaa(ruutu);
                }
            }
            Syote::OikeaKlikkaus(x, y) => {
                if let Some(ruutu) = piirto::ruutu_pisteesta(self.vaikeus, x, y) {
                    self.kursori = ruutu;
                    self.kursori_nakyy = false;
                    self.liputa(ruutu);
                }
            }
            Syote::Nuoli(drivi, dsarake) => {
                // Ensimmäinen painallus vain tuo kursorin näkyviin.
                if self.kursori_nakyy {
                    let rivi = self.kursori.0.saturating_add_signed(drivi as isize);
                    let sarake = self.kursori.1.saturating_add_signed(dsarake as isize);
                    self.kursori = (rivi.min(koko.0 - 1), sarake.min(koko.1 - 1));
                }
                self.kursori_nakyy = true;
            }
            Syote::Ammu => {
                if self.kursori_nakyy {
                    self.avaa(self.kursori);
                }
                self.kursori_nakyy = true;
            }
            Syote::Kirjain('F') => {
                if self.kursori_nakyy {
                    self.liputa(self.kursori);
                }
                self.kursori_nakyy = true;
            }
            _ => {}
        }
    }

    /// Suljettu ruutu aukeaa. Avattu numero avaa ympäriltään, kun liput täsmäävät.
    pub fn avaa(&mut self, ruutu: Paikka) {
        let tulos = if self.lauta.ruutu(ruutu).auki {
            self.lauta.avaa_ymparilta(ruutu, &mut self.arpa)
        } else {
            self.lauta.avaa(ruutu, &mut self.arpa)
        };
        match tulos {
            Avaus::Ei => {}
            Avaus::Auki(maara) => {
                if self.aika_alku.is_none() {
                    self.aika_alku = Some(self.kello_ms);
                }
                self.aanet
                    .jono
                    .push((if maara > 1 { Aani::Liuku } else { Aani::Pomppu }, 0));
                self.viesti.clear();
            }
            Avaus::Miina => {
                self.loppuaika = Some(self.aika());
                self.vaihe = Vaihe::Tappio;
                self.rajahdys = Some(self.kello_ms);
                self.aanet.jono.push((Aani::Rajahdys, 0));
                self.aanet.jono.push((Aani::Tappio, LOPPU_VIIVE_MS));
                self.aanet
                    .jono
                    .push((Aani::PeliOhi, LOPPU_VIIVE_MS + PUHE_VIIVE_MS));
                self.viesti = kaanna("miinaharava.pamaus").to_string();
            }
            Avaus::Voitto => self.voita(),
        }
    }

    fn voita(&mut self) {
        // Kenttä voi aueta jo ensimmäisellä avauksella, jolloin kello ei ehdi käynnistyä.
        let aika = self.aika().max(1);
        self.loppuaika = Some(aika);
        self.vaihe = Vaihe::Voitto;
        self.aanet.jono.push((Aani::Voitto, LOPPU_VIIVE_MS));
        self.aanet
            .jono
            .push((Aani::Voitit, LOPPU_VIIVE_MS + PUHE_VIIVE_MS));
        let pisteet = self.pisteet();
        let paras = &mut self.ajat[tason_indeksi(self.vaikeus)];
        if *paras == 0 || aika < *paras {
            *paras = aika;
            self.uusi_tulos = true;
            self.viesti = muotoile("miinaharava.ennatys", &[&aika, &pisteet]);
        } else {
            self.viesti = muotoile("miinaharava.aika_viesti", &[&aika, &pisteet]);
        }
        if self.taulu.mahtuu(pisteet) {
            self.nimi.clear();
            self.vaihe = Vaihe::Nimi;
        }
    }

    pub fn liputa(&mut self, ruutu: Paikka) {
        if self.lauta.liputa(ruutu) {
            self.aanet.jono.push((Aani::Tiili, 0));
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
            Some(Nappi::Uudestaan) => self.aloita(),
            Some(Nappi::Valikko) => self.lopeta = true,
            None => {}
        }
    }
}

impl Pelinakyma for Miinaharava {
    fn lue_tallennuksesta(&mut self, tallennus: &Tallennus) {
        self.vaikeus = tallennus.asetukset.miinaharava;
        self.ajat = tallennus.miinaajat;
    }

    fn kirjaa_asetukset(&self, asetukset: &mut Asetukset) {
        asetukset.miinaharava = self.vaikeus;
    }

    fn kirjaa_tallennukseen(&self, tallennus: &mut Tallennus) {
        tallennus.miinaajat = self.ajat;
    }

    fn kasittele(&mut self, syote: Syote) {
        Miinaharava::kasittele(self, syote);
    }

    fn edista(&mut self, ms: u32) {
        Miinaharava::edista(self, ms);
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
        self.vahvista_nimi();
    }

    fn listat(&mut self) -> Vec<&mut Tulostaulu> {
        vec![&mut self.taulu]
    }

    fn kuvan_avain(&self) -> Option<(u32, u32)> {
        self.kuvan_askel().map(|askel| (askel, 0))
    }
}

#[cfg(test)]
mod tests;
