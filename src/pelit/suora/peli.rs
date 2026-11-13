use klassikot8000::Arpa;
use klassikot8000::suora::{KOKO, Lauta, Merkki, Paikka, Tila, Vaikeus, koneen_siirto, pisteet};

use super::piirto;
use crate::kaynnistin::tallennus::{Asetukset, Tallennus};
use crate::yhteinen::aanet::{Aanet, Aani, PUHE_VIIVE_MS};
use crate::yhteinen::kieli::{kaanna, muotoile};
use crate::yhteinen::loppunapit::{self, Loppunappi};
use crate::yhteinen::pelinakyma::Pelinakyma;
use crate::yhteinen::piirto::{Kehys, VILKKU_MS, aanikytkimella};
use crate::yhteinen::syote::{Syote, Yleinen, yleinen};
use crate::yhteinen::tulokset::{Nettipeli, Tulostaulu, nimi_valmis};

/// Kone "miettii" näin kauan, jotta oma merkki ehtii näkyä ennen vastausta.
pub const KONE_VIIVE_MS: u32 = 450;
const LOPPU_VIIVE_MS: u32 = 300;

/// Konetta vastaan pelaaja on aina risti ja aloittaa, kone on nolla.
pub const PELAAJA: Merkki = Merkki::Risti;

/// Konetta vastaan vai kaksi ihmistä samalla koneella vuorotellen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vastus {
    Kone,
    Kaksin,
}

impl Vastus {
    pub const KAIKKI: [Vastus; 2] = [Vastus::Kone, Vastus::Kaksin];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vaihe {
    Alku,
    Pelissa,
    /// Voitto pääsi kolmen parhaan joukkoon ja odottaa nimimerkkiä.
    Nimi,
    Voitto,
    Tappio,
    Tasapeli,
}

pub struct Suora {
    pub vaihe: Vaihe,
    pub vastus: Vastus,
    pub vaikeus: Vaikeus,
    pub lauta: Lauta,
    pub taulu: Tulostaulu,
    /// Tosi, kun uusi ennätys odottaa tiedostoon kirjoitusta.
    pub uusi_tulos: bool,
    /// Kirjoitettavana oleva nimimerkki.
    pub nimi: String,
    pub viesti: String,
    pub kursori: Paikka,
    /// Kursori näkyy vasta, kun näppäimistöä on käytetty.
    pub kursori_nakyy: bool,
    pub hiiren_ruutu: Option<Paikka>,
    /// Koneen siirron hetki, kun kone on vuorossa.
    pub kone_ms: Option<u32>,
    /// Tämän istunnon voitot: risti (pelaaja), nolla (kone). Nollautuu, kun vastus vaihtuu.
    pub ottelu: (u32, u32),
    /// Kaksinpelin aloitetut pelit: aloittaja vaihtuu joka pelissä.
    kaksinpeleja: u32,
    /// Pyyntö palata käynnistimeen.
    pub lopeta: bool,
    pub aanet: Aanet,
    pub kello_ms: u32,
    arpa: Arpa,
}

fn tason_indeksi(vaikeus: Vaikeus) -> usize {
    Vaikeus::KAIKKI
        .iter()
        .position(|&taso| taso == vaikeus)
        .unwrap_or(0)
}

impl Suora {
    pub fn new(siemen: u64) -> Self {
        Self {
            vaihe: Vaihe::Alku,
            vastus: Vastus::Kone,
            vaikeus: Vaikeus::Normaali,
            lauta: Lauta::new(),
            taulu: Tulostaulu::new(Nettipeli::Suora),
            uusi_tulos: false,
            nimi: String::new(),
            viesti: String::new(),
            kursori: (KOKO / 2, KOKO / 2),
            kursori_nakyy: false,
            hiiren_ruutu: None,
            kone_ms: None,
            ottelu: (0, 0),
            kaksinpeleja: 0,
            lopeta: false,
            aanet: Aanet::default(),
            kello_ms: 0,
            arpa: Arpa::new(siemen),
        }
    }

    /// Kuva riippuu tästä ja syötteestä. `None`, kun alkuruudun malli pyörii. Laudalla
    /// kuva vaihtuu vain vilkun tahdissa ja siirron tullessa.
    pub fn kuvan_askel(&self) -> Option<(u32, u32)> {
        (self.vaihe != Vaihe::Alku)
            .then_some((self.kello_ms / VILKKU_MS, self.lauta.siirrot.len() as u32))
    }

    /// Konetta vastaan pelaajan siirrot, kaksinpelissä molempien yhteensä.
    pub fn omat_siirrot(&self) -> u32 {
        match self.vastus {
            Vastus::Kone => self.lauta.siirtoja(PELAAJA),
            Vastus::Kaksin => self.lauta.siirrot.len() as u32,
        }
    }

    /// Voiton pisteet top-listalle.
    pub fn pisteet(&self) -> u32 {
        pisteet(self.vaikeus, self.omat_siirrot())
    }

    pub fn kone_vuorossa(&self) -> bool {
        self.vastus == Vastus::Kone && self.vaihe == Vaihe::Pelissa && self.lauta.vuoro != PELAAJA
    }

    /// Kaksinpelin vuoroviesti.
    fn vuoroviesti(&self) -> &'static str {
        match self.lauta.vuoro {
            Merkki::Risti => "suora.vuoro_risti",
            Merkki::Nolla => "suora.vuoro_nolla",
        }
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

    pub fn edista(&mut self, ms: u32) {
        self.kello_ms = self.kello_ms.saturating_add(ms);
        if self.kone_ms.is_some_and(|hetki| self.kello_ms >= hetki) {
            self.kone_ms = None;
            self.koneen_vuoro();
        }
    }

    pub fn aloita(&mut self) {
        self.lauta = match self.vastus {
            Vastus::Kone => Lauta::new(),
            Vastus::Kaksin => {
                let aloittaja = if self.kaksinpeleja.is_multiple_of(2) {
                    Merkki::Risti
                } else {
                    Merkki::Nolla
                };
                self.kaksinpeleja += 1;
                Lauta::aloittaen(aloittaja)
            }
        };
        self.vaihe = Vaihe::Pelissa;
        self.kursori = (KOKO / 2, KOKO / 2);
        self.kursori_nakyy = false;
        self.hiiren_ruutu = None;
        self.kone_ms = None;
        self.viesti = match self.vastus {
            Vastus::Kone => kaanna("suora.aloita"),
            Vastus::Kaksin => kaanna(self.vuoroviesti()),
        }
        .to_string();
        self.aanet.soita(Aani::Alku);
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
            Yleinen::Peru => {
                self.lopeta = true;
                return;
            }
            Yleinen::Aanet | Yleinen::Ohita => return,
        }
        match self.vaihe {
            Vaihe::Alku => self.kasittele_alku(syote),
            Vaihe::Pelissa => self.kasittele_pelissa(syote),
            Vaihe::Voitto | Vaihe::Tappio | Vaihe::Tasapeli => self.kasittele_loppu(syote),
            Vaihe::Nimi => {}
        }
    }

    fn kasittele_alku(&mut self, syote: Syote) {
        let vastus = match syote {
            Syote::Nuoli(pysty, 0) if pysty != 0 => Some(match self.vastus {
                Vastus::Kone => Vastus::Kaksin,
                Vastus::Kaksin => Vastus::Kone,
            }),
            Syote::Klikkaus(x, y) => piirto::vastus_pisteesta(x, y),
            _ => None,
        };
        if let Some(vastus) = vastus {
            self.valitse_vastus(vastus);
            return;
        }
        let tasot = Vaikeus::KAIKKI;
        let valittu = match syote {
            Syote::Numero(numero) => (numero as usize)
                .checked_sub(1)
                .and_then(|i| tasot.get(i).copied()),
            Syote::Nuoli(_, muutos) => {
                let uusi = tason_indeksi(self.vaikeus).saturating_add_signed(muutos as isize);
                Some(tasot[uusi.min(tasot.len() - 1)])
            }
            // Kaksinpelissä tasorivi on piilossa, joten sen paikka ei valitse tasoa.
            Syote::Klikkaus(x, y) if self.vastus == Vastus::Kone => piirto::vaikeus_pisteesta(x, y),
            _ => None,
        };
        // Tason valinta tarkoittaa konetta vastaan pelaamista.
        if let Some(vaikeus) = valittu {
            self.vaikeus = vaikeus;
            self.valitse_vastus(Vastus::Kone);
        } else if matches!(syote, Syote::Ammu | Syote::Klikkaus(..)) {
            self.aloita();
        }
    }

    /// Voittolaskuri koskee yhtä vastusta, joten se nollautuu vastuksen vaihtuessa.
    pub fn valitse_vastus(&mut self, vastus: Vastus) {
        if vastus != self.vastus {
            self.vastus = vastus;
            self.ottelu = (0, 0);
            self.kaksinpeleja = 0;
        }
    }

    fn kasittele_pelissa(&mut self, syote: Syote) {
        match syote {
            Syote::Hiiri(x, y) => {
                self.hiiren_ruutu = piirto::ruutu_pisteesta(x, y);
                self.kursori_nakyy = false;
            }
            Syote::Klikkaus(x, y) => {
                if let Some(ruutu) = piirto::ruutu_pisteesta(x, y) {
                    self.kursori = ruutu;
                    self.kursori_nakyy = false;
                    self.merkitse(ruutu);
                }
            }
            Syote::Nuoli(drivi, dsarake) => {
                // Ensimmäinen painallus vain tuo kursorin näkyviin.
                if self.kursori_nakyy {
                    let rivi = self.kursori.0.saturating_add_signed(drivi as isize);
                    let sarake = self.kursori.1.saturating_add_signed(dsarake as isize);
                    self.kursori = (rivi.min(KOKO - 1), sarake.min(KOKO - 1));
                }
                self.kursori_nakyy = true;
            }
            Syote::Ammu => {
                if self.kursori_nakyy {
                    self.merkitse(self.kursori);
                }
                self.kursori_nakyy = true;
            }
            _ => {}
        }
    }

    /// Vuorossa olevan ihmisen merkki ruutuun. Koneen vuorolla tai varattuun ruutuun ei voi
    /// merkitä.
    pub fn merkitse(&mut self, ruutu: Paikka) {
        if self.kone_vuorossa() || self.vaihe != Vaihe::Pelissa {
            return;
        }
        let merkki = self.lauta.vuoro;
        match self.lauta.aseta(ruutu) {
            None => self.aanet.soita(Aani::Virhe),
            Some(tila) => {
                // Risti ja nolla kuulostavat eri ääniltä, myös kaksinpelissä.
                self.aanet.soita(match merkki {
                    Merkki::Risti => Aani::Pomppu,
                    Merkki::Nolla => Aani::Tiili,
                });
                self.viesti = match self.vastus {
                    Vastus::Kone => kaanna("suora.kone_miettii"),
                    Vastus::Kaksin => kaanna(self.vuoroviesti()),
                }
                .to_string();
                self.siirron_jalkeen(tila);
            }
        }
    }

    fn koneen_vuoro(&mut self) {
        if !self.kone_vuorossa() {
            return;
        }
        let Some(ruutu) = koneen_siirto(self.vaikeus, &self.lauta, &mut self.arpa) else {
            return;
        };
        if let Some(tila) = self.lauta.aseta(ruutu) {
            self.aanet.soita(Aani::Tiili);
            self.viesti = kaanna("suora.vuorosi").to_string();
            self.siirron_jalkeen(tila);
        }
    }

    fn siirron_jalkeen(&mut self, tila: Tila) {
        match tila {
            Tila::Kesken => {
                if self.kone_vuorossa() {
                    self.kone_ms = Some(self.kello_ms + KONE_VIIVE_MS);
                }
            }
            Tila::Voitto(merkki) if self.vastus == Vastus::Kaksin => self.kaksinvoitto(merkki),
            Tila::Voitto(merkki) if merkki == PELAAJA => self.voita(),
            Tila::Voitto(_) => {
                self.ottelu.1 += 1;
                self.vaihe = Vaihe::Tappio;
                self.viesti = kaanna("suora.kone_voitti").to_string();
                self.aanet.soita_viiveella(Aani::Tappio, LOPPU_VIIVE_MS);
                self.aanet
                    .soita_viiveella(Aani::PeliOhi, LOPPU_VIIVE_MS + PUHE_VIIVE_MS);
            }
            Tila::Tasapeli => {
                self.vaihe = Vaihe::Tasapeli;
                self.viesti = kaanna("suora.lauta_taynna").to_string();
                self.aanet.soita_viiveella(Aani::EiSiirtoja, LOPPU_VIIVE_MS);
            }
        }
    }

    /// Kaksinpelin voitto: ei pisteitä listalle, koska voittoa ei voi kirjata kenenkään nimiin.
    fn kaksinvoitto(&mut self, merkki: Merkki) {
        let (voitot, viesti) = match merkki {
            Merkki::Risti => (&mut self.ottelu.0, "suora.risti_sai"),
            Merkki::Nolla => (&mut self.ottelu.1, "suora.nolla_sai"),
        };
        *voitot += 1;
        self.vaihe = Vaihe::Voitto;
        self.viesti = kaanna(viesti).to_string();
        self.aanet.soita_viiveella(Aani::Voitto, LOPPU_VIIVE_MS);
        self.aanet
            .soita_viiveella(Aani::Voitit, LOPPU_VIIVE_MS + PUHE_VIIVE_MS);
    }

    fn voita(&mut self) {
        self.ottelu.0 += 1;
        self.vaihe = Vaihe::Voitto;
        self.aanet.soita_viiveella(Aani::Voitto, LOPPU_VIIVE_MS);
        self.aanet
            .soita_viiveella(Aani::Voitit, LOPPU_VIIVE_MS + PUHE_VIIVE_MS);
        let pisteet = self.pisteet();
        let siirrot = self.omat_siirrot();
        if pisteet > self.taulu.paras {
            self.taulu.paras = pisteet;
            self.uusi_tulos = true;
            self.viesti = muotoile("suora.ennatys", &[&siirrot, &pisteet]);
        } else {
            self.viesti = muotoile("suora.voitto_viesti", &[&siirrot, &pisteet]);
        }
        if self.taulu.mahtuu(pisteet) {
            self.nimi.clear();
            self.vaihe = Vaihe::Nimi;
        }
    }

    fn kasittele_loppu(&mut self, syote: Syote) {
        match loppunapit::valinta(syote) {
            Some(Loppunappi::Uudestaan) => self.aloita(),
            Some(Loppunappi::Valikko) => self.lopeta = true,
            None => {}
        }
    }
}

impl Pelinakyma for Suora {
    fn lue_tallennuksesta(&mut self, tallennus: &Tallennus) {
        self.vaikeus = tallennus.asetukset.suora;
        self.vastus = tallennus.asetukset.suoravastus;
    }

    fn kirjaa_asetukset(&self, asetukset: &mut Asetukset) {
        asetukset.suora = self.vaikeus;
        asetukset.suoravastus = self.vastus;
    }

    fn kasittele(&mut self, syote: Syote) {
        Suora::kasittele(self, syote);
    }

    fn edista(&mut self, ms: u32) {
        Suora::edista(self, ms);
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
        self.kuvan_askel()
    }
}

#[cfg(test)]
mod tests;
