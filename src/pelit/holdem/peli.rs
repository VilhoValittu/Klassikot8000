use klassikot8000::Arpa;
use klassikot8000::holdem::{ALKURAHA, Kulku, PELAAJA, PELAAJIA, Poyta, Toimi};

use super::piirto::{self, kaden_nimi, pelaajan_nimi};
use crate::yhteinen::aanet::{Aanet, Aani, PUHE_VIIVE_MS};
use crate::yhteinen::kieli::{kaanna, muotoile};
use crate::yhteinen::pelinakyma::Pelinakyma;
use crate::yhteinen::piirto::{Kehys, VILKKU_MS};
use crate::yhteinen::syote::{Syote, Yleinen, yleinen};
use crate::yhteinen::tulokset::{Nettipeli, Tulostaulu, nimi_valmis};

/// Koneen teko, kortin käännös ja käden näyttö etenevät tässä tahdissa.
const ASKEL_MS: u32 = 900;
const LOPPU_VIIVE_MS: u32 = 250;
const NUOLEN_ISOT_ASKELEET: i64 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vaihe {
    Alku,
    Pelissa,
    /// Pelaajan rahat loppuivat.
    Havio,
    /// Muiden rahat loppuivat.
    Voitto,
    /// Voitto pääsi kolmen parhaan joukkoon ja odottaa nimimerkkiä.
    Nimi,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nappi {
    Maksa,
    Kippaa,
    Korota,
    Pienin,
    PuoliPottia,
    KokoPotti,
    Kaikki,
    /// Hoputtaa koneita tai aloittaa seuraavan jaon.
    Jatka,
    Uudestaan,
    Valikko,
}

/// Mitkä napit ovat esillä.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Napisto {
    Vuoro,
    Odotus,
    JakoOhi,
    Loppu,
}

pub struct Holdem {
    pub vaihe: Vaihe,
    pub poyta: Poyta,
    /// Oma kokonaispanos kadulla, johon korotus nostaisi.
    pub korotus: u32,
    pub taulu: Tulostaulu,
    /// Kirjoitettavana oleva nimimerkki.
    pub nimi: String,
    /// Tosi, kun tulos odottaa tiedostoon kirjoitusta.
    pub uusi_tulos: bool,
    /// Pelaaja poistuu kesken pelin: nimen jälkeen palataan valikkoon.
    poistuu: bool,
    pub viesti: String,
    pub lopeta: bool,
    pub aanet: Aanet,
    pub kello_ms: u32,
    /// Siirtojen määrä: kuva muuttuu niistä ilman syötettä.
    siirtoja: u32,
    seuraava_siirto_ms: u32,
    arpa: Arpa,
}

impl Holdem {
    pub fn new(siemen: u64) -> Self {
        let mut arpa = Arpa::new(siemen);
        Self {
            vaihe: Vaihe::Alku,
            poyta: Poyta::new(&mut arpa),
            korotus: 0,
            taulu: Tulostaulu::new(Nettipeli::Holdem),
            nimi: String::new(),
            uusi_tulos: false,
            poistuu: false,
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
            Vaihe::Pelissa | Vaihe::Havio | Vaihe::Voitto => Some(self.siirtoja),
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

    /// Pöydästä lähtö lunastaa voiton. Pöydässä oleva panos jää sinne.
    pub fn poistu(&mut self) {
        if self.vaihe == Vaihe::Pelissa {
            self.kirjaa_paras();
        }
        if self.vaihe == Vaihe::Pelissa && self.taulu.mahtuu(self.poyta.pisteet()) {
            self.nimi.clear();
            self.poistuu = true;
            self.vaihe = Vaihe::Nimi;
        } else {
            self.lopeta = true;
        }
    }

    /// Kirjaa odottavan tuloksen.
    pub fn vahvista_nimi(&mut self) {
        if self.vaihe != Vaihe::Nimi {
            return;
        }
        let pisteet = self.poyta.pisteet();
        self.taulu.kirjaa(&mut self.nimi, pisteet);
        self.uusi_tulos = true;
        self.vaihe = Vaihe::Voitto;
        self.lopeta = self.poistuu;
    }

    pub fn napisto(&self) -> Napisto {
        if self.vaihe != Vaihe::Pelissa {
            Napisto::Loppu
        } else if self.poyta.kulku == Kulku::Ohi {
            Napisto::JakoOhi
        } else if self.pelaajan_vuoro() {
            Napisto::Vuoro
        } else {
            Napisto::Odotus
        }
    }

    fn pelaajan_vuoro(&self) -> bool {
        self.poyta.kulku == Kulku::Panostus && self.poyta.vuorossa == Some(PELAAJA)
    }

    pub fn edista(&mut self, ms: u32) {
        self.kello_ms = self.kello_ms.saturating_add(ms);
        while self.napisto() == Napisto::Odotus && self.kello_ms >= self.seuraava_siirto_ms {
            self.seuraava_siirto_ms = self.seuraava_siirto_ms.saturating_add(ASKEL_MS);
            self.siirto();
        }
    }

    fn siirto(&mut self) {
        self.poyta.etene(&mut self.arpa);
        self.siirtoja = self.siirtoja.wrapping_add(1);
        self.aanet.soita(Aani::Liuku);
        self.tila_muuttui();
    }

    fn aloita(&mut self) {
        self.poyta = Poyta::new(&mut self.arpa);
        self.vaihe = Vaihe::Pelissa;
        self.aanet.soita(Aani::Alku);
        self.uusi_jako();
    }

    fn uusi_jako(&mut self) {
        self.poyta.jaa(&mut self.arpa);
        self.seuraava_siirto_ms = self.kello_ms.saturating_add(ASKEL_MS);
        self.aanet.soita(Aani::Liuku);
        self.tila_muuttui();
    }

    /// Päivittää viestin siihen, mitä nyt tapahtuu. Oman vuoron alkaessa korotus palaa pienimpään.
    fn tila_muuttui(&mut self) {
        self.viesti = match (self.poyta.kulku, self.poyta.vuorossa) {
            (Kulku::Ohi, _) => return self.jako_paattyi(),
            (Kulku::Panostus, Some(PELAAJA)) => {
                self.korotus = self.poyta.pienin_taso(PELAAJA);
                match self.poyta.maksu(PELAAJA) {
                    0 => kaanna("holdem.sinun_vuorosi").to_string(),
                    maksu => muotoile("holdem.maksettavaa", &[&maksu]),
                }
            }
            (Kulku::Panostus, Some(i)) => muotoile("holdem.miettii", &[&pelaajan_nimi(i)]),
            (Kulku::Naytto, _) => kaanna("holdem.kortit_esiin").to_string(),
            _ => kaanna("holdem.seuraava_kortti").to_string(),
        };
    }

    fn jako_paattyi(&mut self) {
        self.viesti = self.tulosviesti();
        let voitti = self.poyta.pelaajat[PELAAJA].voitto > 0;
        if self.poyta.pelaajat[PELAAJA].rahat == 0 {
            self.vaihe = Vaihe::Havio;
            let sija = self.poyta.rahoissa() + 1;
            self.viesti = muotoile("holdem.havio", &[&sija]);
            self.aanet.soita_viiveella(Aani::Tappio, LOPPU_VIIVE_MS);
            self.aanet
                .soita_viiveella(Aani::PeliOhi, LOPPU_VIIVE_MS + PUHE_VIIVE_MS);
        } else if self.poyta.rahoissa() == 1 {
            self.vaihe = Vaihe::Voitto;
            self.kirjaa_paras();
            let kaikki = ALKURAHA * PELAAJIA as u32;
            self.viesti = muotoile("holdem.voitto", &[&kaikki]);
            self.aanet.soita_viiveella(Aani::Voitto, LOPPU_VIIVE_MS);
            self.aanet
                .soita_viiveella(Aani::Voitit, LOPPU_VIIVE_MS + PUHE_VIIVE_MS);
            if self.taulu.mahtuu(self.poyta.pisteet()) {
                self.nimi.clear();
                self.vaihe = Vaihe::Nimi;
            }
        } else if voitti {
            self.aanet.soita_viiveella(Aani::Yhdistys, LOPPU_VIIVE_MS);
        } else {
            self.aanet.soita_viiveella(Aani::Pomppu, LOPPU_VIIVE_MS);
        }
    }

    /// Tulos kertoo, miksi voittajia on useampi: täsmälleen sama käsi jakaa potin, ja sivupotin
    /// voi voittaa vain sen maksaneista.
    fn tulosviesti(&self) -> String {
        let pelaajat = &self.poyta.pelaajat;
        let summat = |osuus: &dyn Fn(usize) -> u32| {
            let rivit: Vec<String> = (0..PELAAJIA)
                .filter(|&i| osuus(i) > 0)
                .map(|i| format!("{} +{}", pelaajan_nimi(i), osuus(i)))
                .collect();
            rivit.join(", ")
        };
        let voitto = |i: usize| pelaajat[i].voitto;
        let sivupotista = |i: usize| pelaajat[i].voitto - pelaajat[i].paapotista;
        let voittajia = (0..PELAAJIA).filter(|&i| voitto(i) > 0).count();
        let suurin = (0..PELAAJIA).max_by_key(|&i| voitto(i));
        let Some((arvo, _)) = suurin.and_then(|i| pelaajat[i].kasi) else {
            return muotoile("holdem.muut_kippasivat", &[&summat(&voitto)]);
        };
        if voittajia == 1 {
            muotoile(
                "holdem.voitot_kasi",
                &[&summat(&voitto), &kaden_nimi(&arvo)],
            )
        } else if (0..PELAAJIA).any(|i| sivupotista(i) > 0) {
            let paapotista = summat(&|i| pelaajat[i].paapotista);
            muotoile(
                "holdem.sivupotti_tulos",
                &[&paapotista, &summat(&sivupotista)],
            )
        } else {
            // Summat näkyvät paikoilla, joten viesti mahtuu viidellekin voittajalle.
            let nimet: Vec<&str> = (0..PELAAJIA)
                .filter(|&i| voitto(i) > 0)
                .map(pelaajan_nimi)
                .collect();
            muotoile("holdem.tasan", &[&nimet.join(", "), &kaden_nimi(&arvo)])
        }
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
            Yleinen::Nimi => {
                if nimi_valmis(&mut self.nimi, syote) {
                    self.vahvista_nimi();
                }
                return;
            }
            Yleinen::Peru => return self.poistu(),
            Yleinen::Aanet | Yleinen::Ohita => return,
        }
        if self.vaihe == Vaihe::Alku {
            if matches!(syote, Syote::Ammu | Syote::Klikkaus(..)) {
                self.aloita();
            }
            return;
        }
        if let (Syote::Nuoli(drivi, dsarake), Napisto::Vuoro) = (syote, self.napisto()) {
            let isoja = dsarake as i64 - drivi as i64 * NUOLEN_ISOT_ASKELEET;
            self.muuta_korotusta(isoja * self.poyta.blindit().1 as i64);
        } else if let Some(nappi) = self.syotteen_nappi(syote) {
            self.paina(nappi);
        }
    }

    fn syotteen_nappi(&self, syote: Syote) -> Option<Nappi> {
        if let Syote::Klikkaus(x, y) = syote {
            return piirto::nappi_pisteesta(self.napisto(), x, y);
        }
        if syote == Syote::Kirjain('E') {
            return Some(Nappi::Valikko);
        }
        // Väli ei koskaan maksa: hoputuksen toistuminen ei saa viedä rahoja vuoron alkaessa.
        match (self.napisto(), syote) {
            (Napisto::Loppu, Syote::Kirjain('K')) => Some(Nappi::Uudestaan),
            (Napisto::Odotus | Napisto::JakoOhi, Syote::Ammu) => Some(Nappi::Jatka),
            (Napisto::Vuoro, Syote::Kirjain('V')) => Some(Nappi::Maksa),
            (Napisto::Vuoro, Syote::Kirjain('K')) => Some(Nappi::Kippaa),
            (Napisto::Vuoro, Syote::Kirjain('R')) => Some(Nappi::Korota),
            (Napisto::Vuoro, Syote::Numero(1)) => Some(Nappi::Pienin),
            (Napisto::Vuoro, Syote::Numero(2)) => Some(Nappi::PuoliPottia),
            (Napisto::Vuoro, Syote::Numero(3)) => Some(Nappi::KokoPotti),
            (Napisto::Vuoro, Syote::Numero(4)) => Some(Nappi::Kaikki),
            _ => None,
        }
    }

    fn paina(&mut self, nappi: Nappi) {
        // Napit ovat näkyvissä vain oikeassa tilassa, mutta hiiri voi ehtiä ennen uutta kuvaa.
        let vuoro = self.napisto() == Napisto::Vuoro;
        match nappi {
            Nappi::Maksa if vuoro => self.tee(Toimi::Maksa),
            Nappi::Kippaa if vuoro => self.tee(Toimi::Kippaa),
            Nappi::Korota if vuoro => self.korota(),
            Nappi::Pienin if vuoro => self.aseta_korotus(self.poyta.pienin_taso(PELAAJA)),
            Nappi::PuoliPottia if vuoro => self.aseta_korotus(self.potin_mukaan(2)),
            Nappi::KokoPotti if vuoro => self.aseta_korotus(self.potin_mukaan(1)),
            Nappi::Kaikki if vuoro => self.aseta_korotus(self.poyta.suurin_taso(PELAAJA)),
            Nappi::Jatka => match self.napisto() {
                Napisto::Odotus => {
                    self.seuraava_siirto_ms = self.kello_ms.saturating_add(ASKEL_MS);
                    self.siirto();
                }
                Napisto::JakoOhi => self.uusi_jako(),
                _ => {}
            },
            Nappi::Uudestaan if self.napisto() == Napisto::Loppu => self.aloita(),
            Nappi::Valikko => self.poistu(),
            _ => {}
        }
    }

    fn korota(&mut self) {
        if self.poyta.voi_korottaa(PELAAJA) {
            self.tee(Toimi::Korota(self.korotus));
        } else {
            self.viesti = kaanna("holdem.ei_korotusta").to_string();
            self.aanet.soita(Aani::Virhe);
        }
    }

    fn tee(&mut self, toimi: Toimi) {
        if self.poyta.toimi(toimi) {
            self.siirtoja = self.siirtoja.wrapping_add(1);
            self.seuraava_siirto_ms = self.kello_ms.saturating_add(ASKEL_MS);
            self.aanet.soita(Aani::Liuku);
            self.tila_muuttui();
        }
    }

    /// Korotus potin osuudella: pottiin lasketaan myös oma maksu, kuten pottirajoitteisessa.
    fn potin_mukaan(&self, jakaja: u32) -> u32 {
        let maksu = self.poyta.maksu(PELAAJA);
        self.poyta.korkein + (self.poyta.potti() + maksu) / jakaja
    }

    fn aseta_korotus(&mut self, taso: u32) {
        self.korotus = taso.clamp(
            self.poyta.pienin_taso(PELAAJA),
            self.poyta.suurin_taso(PELAAJA),
        );
    }

    fn muuta_korotusta(&mut self, muutos: i64) {
        let uusi = (self.korotus as i64 + muutos).max(0) as u32;
        self.aseta_korotus(uusi);
    }
}

impl Pelinakyma for Holdem {
    fn kasittele(&mut self, syote: Syote) {
        Holdem::kasittele(self, syote);
    }

    fn edista(&mut self, ms: u32) {
        Holdem::edista(self, ms);
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
