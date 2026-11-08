use klassikot8000::Arpa;
use klassikot8000::luvut::{Lukulauta, Ruudut, Siirto, Suunta, TAVOITE};

use super::piirto;
use crate::kaynnistin::tallennus::Tallennus;
use crate::yhteinen::aanet::{Aanet, Aani, PUHE_VIIVE_MS};
use crate::yhteinen::kieli::{kaanna, muotoile};
use crate::yhteinen::pelinakyma::Pelinakyma;
use crate::yhteinen::piirto::{Kehys, VILKKU_MS, aanikytkimella};
use crate::yhteinen::syote::{Syote, Yleinen, yleinen};
use crate::yhteinen::tulokset::{Nettipeli, Tulostaulu, muokkaa_nimea};

pub const LIUKU_MS: u32 = 110;
pub const KIPINA_MS: u32 = 450;
const YHDISTYS_VIIVE_MS: u32 = 60;
const LOPPU_VIIVE_MS: u32 = 250;

/// Kesken jäänyt peli, joka jatkuu seuraavalla kerralla.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Kesken {
    pub ruudut: Ruudut,
    pub pisteet: u32,
    pub siirrot: u32,
    pub jatkettu: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vaihe {
    Alku,
    Pelissa,
    Voitto,
    Tappio,
    /// Tulos pääsi kolmen parhaan joukkoon ja odottaa nimimerkkiä.
    Nimi,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nappi {
    Uudestaan,
    Valikko,
    Jatka,
}

pub struct Luvut {
    pub vaihe: Vaihe,
    pub lauta: Lukulauta,
    pub taulu: Tulostaulu,
    /// Kirjoitettavana oleva nimimerkki.
    pub nimi: String,
    /// Tosi, kun uusi tulos odottaa tiedostoon kirjoitusta.
    pub uusi_tulos: bool,
    pub siirtoja: u32,
    pub viesti: String,
    pub ei_liiku: bool,
    /// Suunta, johon hiiren klikkaus siirtäisi. Piirretään nuolena laudan reunaan.
    pub hiiren_suunta: Option<Suunta>,
    /// Viimeisin siirto ja sen alkuhetki animaatiota varten.
    pub viime_siirto: Option<(Siirto, u32)>,
    /// Pyyntö palata käynnistimeen.
    pub lopeta: bool,
    pub aanet: Aanet,
    pub kello_ms: u32,
    /// Alkuruudulla odottaa tallennettu peli, jota voi jatkaa.
    pub jatkuu: bool,
    jatkettu: bool,
    /// Nimimerkin jälkeen alkaa uusi peli eikä näytetä tappiota.
    uusi_alkaa: bool,
    arpa: Arpa,
}

impl Luvut {
    pub fn new(siemen: u64) -> Self {
        let mut arpa = Arpa::new(siemen);
        Self {
            vaihe: Vaihe::Alku,
            lauta: Lukulauta::new(&mut arpa),
            taulu: Tulostaulu::new(Nettipeli::Luvut),
            nimi: String::new(),
            uusi_tulos: false,
            siirtoja: 0,
            viesti: String::new(),
            ei_liiku: false,
            hiiren_suunta: None,
            viime_siirto: None,
            lopeta: false,
            aanet: Aanet::default(),
            kello_ms: 0,
            jatkuu: false,
            jatkettu: false,
            uusi_alkaa: false,
            arpa,
        }
    }

    pub fn palauta(&mut self, kesken: Kesken) {
        self.lauta = Lukulauta::ruuduista(kesken.ruudut);
        self.lauta.pisteet = kesken.pisteet;
        self.siirtoja = kesken.siirrot;
        self.jatkettu = kesken.jatkettu;
        self.jatkuu = true;
        self.taulu.paras = self.taulu.paras.max(kesken.pisteet);
    }

    /// Tallennettava tilanne. Päättynyttä tai aloittamatonta peliä ei tallenneta.
    pub fn kesken(&self) -> Option<Kesken> {
        let tallennetaan = match self.vaihe {
            Vaihe::Alku => self.jatkuu,
            Vaihe::Pelissa | Vaihe::Voitto => true,
            Vaihe::Tappio => false,
            Vaihe::Nimi => self.uusi_alkaa,
        };
        tallennetaan.then(|| Kesken {
            ruudut: self.lauta.ruudut(),
            pisteet: self.lauta.pisteet,
            siirrot: self.siirtoja,
            // Voitto on jo juhlittu, joten sitä ei näytetä jatkettaessa uudelleen.
            jatkettu: self.jatkettu || self.vaihe == Vaihe::Voitto,
        })
    }

    pub fn edista(&mut self, ms: u32) {
        self.kello_ms = self.kello_ms.saturating_add(ms);
        let kello = self.kello_ms;
        if let Some((_, alku)) = self.viime_siirto
            && kello >= alku + LIUKU_MS + KIPINA_MS
        {
            self.viime_siirto = None;
        }
    }

    fn aloita_alusta(&mut self) {
        self.vaihe = Vaihe::Alku;
        self.lauta = Lukulauta::new(&mut self.arpa);
        self.siirtoja = 0;
        self.viesti.clear();
        self.ei_liiku = false;
        self.hiiren_suunta = None;
        self.viime_siirto = None;
        self.jatkuu = false;
        self.jatkettu = false;
    }

    /// Uusi peli hylkää nykyisen, joten hyvä tulos kirjataan ensin listalle.
    fn hylkaa_ja_aloita(&mut self) {
        if self.taulu.mahtuu(self.lauta.pisteet) {
            self.uusi_alkaa = true;
            self.kysy_nimi();
        } else {
            self.aloita_alusta();
        }
    }

    fn kysy_nimi(&mut self) {
        self.nimi.clear();
        self.vaihe = Vaihe::Nimi;
    }

    fn paata_peli(&mut self) {
        if self.taulu.mahtuu(self.lauta.pisteet) {
            self.kysy_nimi();
        } else {
            self.vaihe = Vaihe::Tappio;
        }
    }

    /// Kirjaa odottavan tuloksen.
    pub fn vahvista_nimi(&mut self) {
        if self.vaihe != Vaihe::Nimi {
            return;
        }
        let pisteet = self.lauta.pisteet;
        self.taulu.kirjaa(&mut self.nimi, pisteet);
        self.uusi_tulos = true;
        if std::mem::take(&mut self.uusi_alkaa) {
            self.aloita_alusta();
        } else {
            self.vaihe = Vaihe::Tappio;
        }
    }

    fn kasittele_nimi(&mut self, syote: Syote) {
        match syote {
            Syote::Ammu | Syote::Peru => self.vahvista_nimi(),
            _ => muokkaa_nimea(&mut self.nimi, syote),
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
            Yleinen::Peru => {
                self.lopeta = true;
                return;
            }
            Yleinen::Aanet | Yleinen::Ohita => return,
        }
        match self.vaihe {
            Vaihe::Alku => {
                let uusi = syote == Syote::Kirjain('K')
                    || matches!(syote, Syote::Klikkaus(x, y) if piirto::uusi_peli_pisteessa(x, y));
                if self.jatkuu && uusi {
                    self.hylkaa_ja_aloita();
                } else if matches!(syote, Syote::Ammu | Syote::Klikkaus(..)) {
                    self.vaihe = Vaihe::Pelissa;
                    self.aanet.soita(Aani::Alku);
                    if self.jatkuu {
                        self.jatkuu = false;
                        self.viesti = kaanna("luvut.jatketaan_tallennettua").to_string();
                    }
                }
            }
            Vaihe::Pelissa => self.kasittele_pelissa(syote),
            Vaihe::Voitto | Vaihe::Tappio => self.kasittele_loppu(syote),
            Vaihe::Nimi => {}
        }
    }

    fn kasittele_pelissa(&mut self, syote: Syote) {
        match syote {
            Syote::Nuoli(-1, _) => self.siirra(Suunta::Ylos),
            Syote::Nuoli(1, _) => self.siirra(Suunta::Alas),
            Syote::Nuoli(_, -1) => self.siirra(Suunta::Vasen),
            Syote::Nuoli(_, 1) => self.siirra(Suunta::Oikea),
            Syote::Hiiri(x, y) => self.hiiren_suunta = piirto::suunta_pisteesta(x, y),
            Syote::Klikkaus(x, y) => {
                if let Some(suunta) = piirto::suunta_pisteesta(x, y) {
                    self.siirra(suunta);
                }
            }
            _ => {}
        }
    }

    fn kasittele_loppu(&mut self, syote: Syote) {
        let nappi = match syote {
            Syote::Kirjain('K') => Nappi::Uudestaan,
            Syote::Kirjain('E') => Nappi::Valikko,
            Syote::Kirjain('J') => Nappi::Jatka,
            Syote::Klikkaus(x, y) => match piirto::nappi_pisteesta(x, y) {
                Some(nappi) => nappi,
                None => return,
            },
            _ => return,
        };
        match nappi {
            Nappi::Uudestaan if self.vaihe == Vaihe::Voitto => {
                // Voitto on jo juhlittu, jos nimen kirjoitus keskeytyy tallennukseen.
                self.jatkettu = true;
                self.hylkaa_ja_aloita();
            }
            Nappi::Uudestaan => self.aloita_alusta(),
            Nappi::Valikko => self.lopeta = true,
            Nappi::Jatka if self.vaihe == Vaihe::Voitto => {
                self.jatkettu = true;
                self.viesti = kaanna("luvut.jatketaan").to_string();
                if self.lauta.on_siirtoja() {
                    self.vaihe = Vaihe::Pelissa;
                } else {
                    self.paata_peli();
                }
            }
            Nappi::Jatka => {}
        }
    }

    pub fn siirra(&mut self, suunta: Suunta) {
        let Some(siirto) = self.lauta.siirra(suunta, &mut self.arpa) else {
            self.ei_liiku = true;
            self.aanet.soita(Aani::Virhe);
            return;
        };
        self.ei_liiku = false;
        self.siirtoja += 1;
        self.taulu.paras = self.taulu.paras.max(self.lauta.pisteet);
        self.viesti = if siirto.pisteet > 0 {
            muotoile("luvut.siirto_pisteet", &[&self.siirtoja, &siirto.pisteet])
        } else {
            muotoile("luvut.siirto", &[&self.siirtoja])
        };
        self.aanet.soita(Aani::Liuku);
        if siirto.pisteet > 0 {
            self.aanet
                .soita_viiveella(Aani::Yhdistys, YHDISTYS_VIIVE_MS);
        }
        self.viime_siirto = Some((siirto, self.kello_ms));

        if !self.jatkettu && self.lauta.suurin() >= TAVOITE {
            self.vaihe = Vaihe::Voitto;
            self.aanet.soita_viiveella(Aani::Voitto, LOPPU_VIIVE_MS);
            self.aanet
                .soita_viiveella(Aani::Voitit, LOPPU_VIIVE_MS + PUHE_VIIVE_MS);
        } else if !self.lauta.on_siirtoja() {
            self.paata_peli();
            self.aanet.soita_viiveella(Aani::Tappio, LOPPU_VIIVE_MS);
            self.aanet
                .soita_viiveella(Aani::EiSiirtoja, LOPPU_VIIVE_MS + PUHE_VIIVE_MS);
        }
    }
}

impl Pelinakyma for Luvut {
    fn lue_tallennuksesta(&mut self, tallennus: &Tallennus) {
        if let Some(kesken) = tallennus.kesken {
            self.palauta(kesken);
        }
    }

    fn kirjaa_tallennukseen(&self, tallennus: &mut Tallennus) {
        tallennus.kesken = self.kesken();
    }

    fn kasittele(&mut self, syote: Syote) {
        Luvut::kasittele(self, syote);
    }

    fn edista(&mut self, ms: u32) {
        Luvut::edista(self, ms);
    }

    fn piirra(&self, kehys: &mut Kehys) {
        piirto::piirra(self, &mut kehys.kuva);
        kehys.venyta();
    }

    /// Levossa lauta piirretään vain syötteestä ja vilkkumisen tahdissa. Alkuruudun malli
    /// pyörii ja siirto liukuu, joten niiden aikana kuva elää joka ruudussa.
    fn kuvan_avain(&self) -> Option<(u32, u32)> {
        if self.vaihe == Vaihe::Alku || self.viime_siirto.is_some() {
            return None;
        }
        let suuntanuoli = self.vaihe == Vaihe::Pelissa && self.hiiren_suunta.is_some();
        let vilkkuu = suuntanuoli || self.vaihe == Vaihe::Nimi;
        Some((
            0,
            if vilkkuu {
                self.kello_ms / VILKKU_MS
            } else {
                0
            },
        ))
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

    fn nuolet_toistuvat(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests;
