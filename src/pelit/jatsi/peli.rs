use klassikot8000::Arpa;
use klassikot8000::jatsi::{
    NOPPIA, PELAAJIA, Poyta, Poytakirja, RIVEJA, Rivi, Toimi, Vaikeus, koneen_toimi, pisteet,
};

use super::piirto;
use crate::kaynnistin::tallennus::{Asetukset, Tallennus};
use crate::yhteinen::aanet::{Aanet, Aani, PUHE_VIIVE_MS};
use crate::yhteinen::kieli::{kaanna, muotoile};
use crate::yhteinen::loppunapit::{self, Loppunappi};
use crate::yhteinen::pelinakyma::Pelinakyma;
use crate::yhteinen::piirto::{Kehys, VILKKU_MS};
use crate::yhteinen::syote::{Syote, Yleinen, yleinen};
use crate::yhteinen::tulokset::{Nettipeli, Tulos, Tulostaulu, nimi_kelpaa, nimi_valmis};

/// Noppien pyörimisen kesto heitossa.
pub const HEITTO_MS: u32 = 450;
/// Koneen askelten väli, jotta pelaaja ehtii nähdä heiton, lukituksen ja kirjauksen.
pub const KONE_ASKEL_MS: u32 = 700;
/// Jatsin juhlinnan kesto.
pub const JATSI_MS: u32 = 1500;
const LOPPU_VIIVE_MS: u32 = 250;
const HELINAT: [Aani; 6] = [
    Aani::Helina1,
    Aani::Helina2,
    Aani::Helina3,
    Aani::Helina4,
    Aani::Helina5,
    Aani::Helina6,
];

/// Kuka istuu pöydän paikalla.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Istuja {
    Ihminen,
    Kone,
    Tyhja,
}

/// Alkuruudulla valittu paikka. Nimi on ihmisen nimimerkki, ja tyhjänä käytetään oletusta.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Istuin {
    pub istuja: Istuja,
    pub nimi: String,
}

/// Oletuspöytä: pelaaja ja yksi kone.
pub fn oletuspaikat() -> Vec<Istuin> {
    [Istuja::Ihminen, Istuja::Kone, Istuja::Tyhja, Istuja::Tyhja]
        .map(|istuja| Istuin {
            istuja,
            nimi: String::new(),
        })
        .to_vec()
}

/// Nimi kelpaa samoin ehdoin kuin top-listan rivillä.
fn kelpaava_nimi(nimi: &str) -> Option<String> {
    nimi_kelpaa(nimi).then(|| nimi.to_string())
}

/// Paikat tallennuksen riveistä "paikka laji [nimi]". Ensimmäinen paikka on aina ihmisen.
pub fn paikat_riveista(rivit: &[&str]) -> Option<Vec<Istuin>> {
    let mut paikat = oletuspaikat();
    for rivi in rivit {
        let mut osat = rivi.split(' ');
        let paikka: usize = osat.next()?.parse().ok().filter(|&i| i < PELAAJIA)?;
        let istuja = match osat.next()? {
            "I" => Istuja::Ihminen,
            "K" if paikka > 0 => Istuja::Kone,
            "-" if paikka > 0 => Istuja::Tyhja,
            _ => return None,
        };
        let nimi = match osat.next() {
            Some(nimi) => kelpaava_nimi(nimi)?,
            None => String::new(),
        };
        paikat[paikka] = Istuin { istuja, nimi };
    }
    Some(paikat)
}

pub fn paikat_riveiksi(paikat: &[Istuin]) -> Vec<String> {
    paikat
        .iter()
        .enumerate()
        .map(|(i, paikka)| {
            let laji = match paikka.istuja {
                Istuja::Ihminen => "I",
                Istuja::Kone => "K",
                Istuja::Tyhja => "-",
            };
            if paikka.nimi.is_empty() {
                format!("{i} {laji}")
            } else {
                format!("{i} {laji} {}", paikka.nimi)
            }
        })
        .collect()
}

/// Kesken jäänyt peli, joka jatkuu seuraavalla kerralla.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Kesken {
    pub poyta: Poyta,
    /// Ihmisten nimet pöydän järjestyksessä. Koneen kohdalla tyhjä.
    pub nimet: Vec<String>,
}

impl Kesken {
    /// Vuoron tila rivinä "vuorossa,heitetty,nopat,lukitut".
    pub fn vuororivi(&self) -> String {
        let p = &self.poyta;
        let nopat: String = p.nopat.iter().map(|n| n.to_string()).collect();
        let lukitut: String = p
            .lukitut
            .iter()
            .map(|&l| if l { '1' } else { '0' })
            .collect();
        format!("{},{},{nopat},{lukitut}", p.vuorossa, p.heitetty)
    }

    /// Pelaajien kirjat riveinä "laji rivit [nimi]", rivit pilkuin ja kirjaamaton viivana.
    pub fn kirjarivit(&self) -> Vec<String> {
        let p = &self.poyta;
        (0..p.pelaajia())
            .map(|i| {
                let laji = if p.koneet[i] { "K" } else { "I" };
                let rivit: Vec<String> = p.kirjat[i]
                    .rivit()
                    .iter()
                    .map(|arvo| arvo.map_or("-".to_string(), |a| a.to_string()))
                    .collect();
                let nimi = &self.nimet[i];
                if nimi.is_empty() {
                    format!("{laji} {}", rivit.join(","))
                } else {
                    format!("{laji} {} {nimi}", rivit.join(","))
                }
            })
            .collect()
    }

    /// Kelvoton tai päättynyt peli antaa `None`.
    pub fn riveista(vuoro: &str, kirjat: &[&str]) -> Option<Self> {
        if !(1..=PELAAJIA).contains(&kirjat.len()) {
            return None;
        }
        let mut koneet = Vec::new();
        let mut nimet = Vec::new();
        let mut poytakirjat = Vec::new();
        for rivi in kirjat {
            let mut osat = rivi.split(' ');
            let kone = match osat.next()? {
                "I" => false,
                "K" => true,
                _ => return None,
            };
            let mut arvot = [None; RIVEJA];
            let mut luvut = osat.next()?.split(',');
            for arvo in &mut arvot {
                *arvo = match luvut.next()? {
                    "-" => None,
                    luku => Some(luku.parse().ok()?),
                };
            }
            if luvut.next().is_some() {
                return None;
            }
            let nimi = match osat.next() {
                Some(nimi) if !kone => kelpaava_nimi(nimi)?,
                Some(_) => return None,
                None => String::new(),
            };
            koneet.push(kone);
            nimet.push(nimi);
            poytakirjat.push(Poytakirja::riveista(arvot)?);
        }
        if koneet.iter().all(|&kone| kone) {
            return None;
        }
        let mut osat = vuoro.split(',');
        let vuorossa: usize = osat.next()?.parse().ok()?;
        let heitetty: u8 = osat.next()?.parse().ok()?;
        let nopat = osat.next()?.as_bytes();
        let lukitut = osat.next()?.as_bytes();
        if osat.next().is_some() || nopat.len() != NOPPIA || lukitut.len() != NOPPIA {
            return None;
        }
        let mut poyta = Poyta::new(koneet, 0);
        poyta.kirjat = poytakirjat;
        poyta.vuorossa = vuorossa;
        poyta.heitetty = heitetty;
        for i in 0..NOPPIA {
            poyta.nopat[i] = match nopat[i] {
                b'1'..=b'6' => nopat[i] - b'0',
                _ => return None,
            };
            poyta.lukitut[i] = lukitut[i] == b'1';
        }
        let kelpaa = vuorossa < poyta.pelaajia()
            && heitetty <= klassikot8000::jatsi::HEITTOJA
            && !poyta.kirjat[vuorossa].taynna();
        kelpaa.then_some(Self { poyta, nimet })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vaihe {
    Alku,
    Pelissa,
    Loppu,
}

pub struct Jatsi {
    pub vaihe: Vaihe,
    pub vaikeus: Vaikeus,
    /// Alkuruudun paikat. Seuraava peli alkaa näillä.
    pub paikat: Vec<Istuin>,
    /// Alkuruudulla valittu paikka.
    pub valittu_paikka: usize,
    /// Valitun paikan nimeä kirjoitetaan.
    pub nimetaan: bool,
    pub poyta: Poyta,
    /// Pelaajien nimet pöydän järjestyksessä. Koneella tyhjä.
    pub nimet: Vec<String>,
    /// Pöytäkirjan valittu rivi, johon K tai viimeisen heiton jälkeinen välilyönti kirjaa.
    pub kursori: Rivi,
    pub hiiren_rivi: Option<Rivi>,
    /// Nollan kirjaus odottaa vahvistusta: toinen painallus viivaa rivin.
    pub viivaus: Option<Rivi>,
    pub viesti: String,
    /// Heiton alkuhetki, kun nopat pyörivät.
    pub heitto_ms: Option<u32>,
    /// Jatsin juhlinnan alkuhetki.
    pub jatsi_ms: Option<u32>,
    /// Hetki, jona kone tekee seuraavan askeleensa.
    kone_ms: u32,
    /// Kasvaa, kun pöytä muuttuu ilman syötettä. Kuvan avain seuraa sitä.
    pub muutoksia: u32,
    /// Päättyneen pelin ihmisten tulokset: (pelaaja, pisteet listalle).
    pub tulokset: Vec<(usize, u32)>,
    /// Alkuruudulla odottaa tallennettu peli, jota voi jatkaa.
    pub jatkuu: bool,
    pub taulu: Tulostaulu,
    /// Tosi, kun uusi tulos odottaa tiedostoon kirjoitusta.
    pub uusi_tulos: bool,
    /// Pyyntö palata käynnistimeen.
    pub lopeta: bool,
    pub aanet: Aanet,
    pub kello_ms: u32,
    arpa: Arpa,
}

impl Jatsi {
    pub fn new(siemen: u64) -> Self {
        Self {
            vaihe: Vaihe::Alku,
            vaikeus: Vaikeus::Normaali,
            paikat: oletuspaikat(),
            valittu_paikka: 0,
            nimetaan: false,
            poyta: Poyta::new(vec![false, true], 0),
            nimet: vec![String::new(); 2],
            kursori: Rivi::Ykkoset,
            hiiren_rivi: None,
            viivaus: None,
            viesti: String::new(),
            heitto_ms: None,
            jatsi_ms: None,
            kone_ms: 0,
            muutoksia: 0,
            tulokset: Vec::new(),
            jatkuu: false,
            taulu: Tulostaulu::new(Nettipeli::Jatsi),
            uusi_tulos: false,
            lopeta: false,
            aanet: Aanet::default(),
            kello_ms: 0,
            arpa: Arpa::new(siemen),
        }
    }

    /// Pelaajan nimi pöydässä: ihmisen nimimerkki tai koneen järjestysnumero.
    pub fn nimi(&self, pelaaja: usize) -> String {
        if !self.poyta.koneet[pelaaja] {
            return self.nimet[pelaaja].clone();
        }
        let numero = self.poyta.koneet[..=pelaaja].iter().filter(|&&k| k).count();
        muotoile("jatsi.kone", &[&numero])
    }

    /// Ihmisen oletusnimi alkuruudun paikan mukaan.
    pub fn oletusnimi(paikka: usize) -> String {
        muotoile("jatsi.pelaaja", &[&(paikka + 1)])
    }

    pub fn palauta(&mut self, kesken: Kesken) {
        self.poyta = kesken.poyta;
        self.nimet = kesken.nimet;
        self.jatkuu = true;
    }

    /// Tallennettava tilanne. Päättynyttä tai aloittamatonta peliä ei tallenneta.
    pub fn kesken(&self) -> Option<Kesken> {
        let tallennetaan = match self.vaihe {
            Vaihe::Alku => self.jatkuu,
            Vaihe::Pelissa => true,
            Vaihe::Loppu => false,
        };
        tallennetaan.then(|| Kesken {
            poyta: self.poyta.clone(),
            nimet: self.nimet.clone(),
        })
    }

    /// Heitto tai juhlinta on käynnissä, joten kuva elää joka ruudussa.
    pub fn animoi(&self) -> bool {
        let kesken = |alku: Option<u32>, kesto: u32| {
            alku.is_some_and(|alku| self.kello_ms < alku.saturating_add(kesto))
        };
        kesken(self.heitto_ms, HEITTO_MS) || kesken(self.jatsi_ms, JATSI_MS)
    }

    fn heittaa(&self) -> bool {
        self.heitto_ms
            .is_some_and(|alku| self.kello_ms < alku.saturating_add(HEITTO_MS))
    }

    pub fn edista(&mut self, ms: u32) {
        self.kello_ms = self.kello_ms.saturating_add(ms);
        if self.vaihe == Vaihe::Pelissa
            && self.poyta.kone_vuorossa()
            && !self.heittaa()
            && self.kello_ms >= self.kone_ms
        {
            self.koneen_askel(true);
        }
    }

    pub fn aloita(&mut self) {
        let mut koneet = Vec::new();
        let mut nimet = Vec::new();
        for (i, paikka) in self.paikat.iter().enumerate() {
            match paikka.istuja {
                Istuja::Tyhja => continue,
                Istuja::Kone => {
                    koneet.push(true);
                    nimet.push(String::new());
                }
                Istuja::Ihminen => {
                    koneet.push(false);
                    let nimi = if paikka.nimi.is_empty() {
                        Self::oletusnimi(i)
                    } else {
                        paikka.nimi.clone()
                    };
                    nimet.push(nimi);
                }
            }
        }
        let aloittaja = self.arpa.alle(koneet.len());
        self.poyta = Poyta::new(koneet, aloittaja);
        self.nimet = nimet;
        self.vaihe = Vaihe::Pelissa;
        self.jatkuu = false;
        self.tulokset.clear();
        self.kursori = Rivi::Ykkoset;
        self.viivaus = None;
        self.heitto_ms = None;
        self.jatsi_ms = None;
        self.kone_ms = self.kello_ms + KONE_ASKEL_MS;
        self.viesti = muotoile("jatsi.aloittaa", &[&self.nimi(aloittaja)]);
        self.aanet.soita(Aani::Alku);
    }

    pub fn kasittele(&mut self, syote: Syote) {
        let vaihe = self.vaihe;
        let kytkimella = |x, y| piirto::aanikytkimella(vaihe, x, y);
        match yleinen(syote, self.nimetaan, &mut self.aanet.paalla, kytkimella) {
            Yleinen::Peli => {}
            Yleinen::Nimi => {
                let nimi = &mut self.paikat[self.valittu_paikka].nimi;
                if nimi_valmis(nimi, syote) {
                    self.nimetaan = false;
                }
                return;
            }
            Yleinen::Ohita => {
                // Klikkaus muualle lopettaa nimen kirjoituksen.
                self.nimetaan = false;
                return;
            }
            Yleinen::Peru => {
                self.lopeta = true;
                return;
            }
            Yleinen::Aanet => return,
        }
        match self.vaihe {
            Vaihe::Alku => self.kasittele_alku(syote),
            Vaihe::Pelissa => self.kasittele_pelissa(syote),
            Vaihe::Loppu => {
                // Pöytä on täysi ruutu, joten napit ovat sivupaneelissa eivätkä klassisessa kuvassa.
                let nappi = match syote {
                    Syote::Klikkaus(x, y) => piirto::loppunappi_pisteesta(x, y),
                    _ => loppunapit::valinta(syote),
                };
                match nappi {
                    Some(Loppunappi::Uudestaan) => self.aloita(),
                    Some(Loppunappi::Valikko) => self.lopeta = true,
                    None => {}
                }
            }
        }
    }

    fn kasittele_alku(&mut self, syote: Syote) {
        if self.jatkuu {
            let uusi = syote == Syote::Kirjain('K')
                || matches!(syote, Syote::Klikkaus(x, y) if piirto::uusi_peli_pisteessa(x, y));
            if uusi {
                self.jatkuu = false;
            } else if matches!(syote, Syote::Ammu | Syote::Klikkaus(..)) {
                self.jatka();
            }
            return;
        }
        match syote {
            Syote::Numero(numero @ 1..=3) => {
                self.vaikeus = Vaikeus::KAIKKI[numero as usize - 1];
            }
            Syote::Nuoli(rivit, 0) => {
                let uusi = self.valittu_paikka as i32 + rivit as i32;
                self.valittu_paikka = uusi.clamp(0, PELAAJIA as i32 - 1) as usize;
            }
            Syote::Nuoli(0, suunta) => self.vaihda_istuja(self.valittu_paikka, suunta),
            Syote::Kirjain('N') => self.nimea(self.valittu_paikka),
            Syote::Ammu => self.aloita(),
            Syote::Klikkaus(x, y) => {
                if let Some(vaikeus) = piirto::vaikeus_pisteesta(x, y) {
                    self.vaikeus = vaikeus;
                } else if let Some(paikka) = piirto::istuja_pisteesta(x, y) {
                    self.valittu_paikka = paikka;
                    self.vaihda_istuja(paikka, 1);
                } else if let Some(paikka) = piirto::nimi_pisteesta(x, y) {
                    self.valittu_paikka = paikka;
                    self.nimea(paikka);
                } else {
                    self.aloita();
                }
            }
            _ => {}
        }
    }

    /// Ensimmäinen paikka on aina ihmisen, jotta pöydässä on joku pelaamassa.
    fn vaihda_istuja(&mut self, paikka: usize, suunta: i8) {
        if paikka == 0 {
            return;
        }
        let jarjestys = [Istuja::Kone, Istuja::Ihminen, Istuja::Tyhja];
        let nyt = jarjestys
            .iter()
            .position(|&i| i == self.paikat[paikka].istuja)
            .unwrap_or(0);
        let uusi = (nyt as i32 + suunta.signum() as i32).rem_euclid(jarjestys.len() as i32);
        self.paikat[paikka].istuja = jarjestys[uusi as usize];
        self.aanet.soita(Aani::Liuku);
    }

    fn nimea(&mut self, paikka: usize) {
        if self.paikat[paikka].istuja == Istuja::Ihminen {
            self.nimetaan = true;
        }
    }

    fn jatka(&mut self) {
        self.jatkuu = false;
        self.vaihe = Vaihe::Pelissa;
        self.kone_ms = self.kello_ms + KONE_ASKEL_MS;
        self.viesti = kaanna("jatsi.jatketaan").to_string();
        self.aanet.soita(Aani::Alku);
    }

    fn kasittele_pelissa(&mut self, syote: Syote) {
        if self.poyta.kone_vuorossa() {
            if matches!(syote, Syote::Ammu | Syote::Klikkaus(..)) {
                self.hoputa();
            }
            return;
        }
        if let Syote::Hiiri(x, y) = syote {
            self.hiiren_rivi = piirto::rivi_pisteesta(self.poyta.pelaajia(), x, y);
            return;
        }
        // Pyörivää noppaa ei lukita eikä sen silmälukua kirjata.
        if self.heittaa() {
            return;
        }
        match syote {
            Syote::Ammu if self.poyta.voi_heittaa() => self.heita(),
            Syote::Ammu | Syote::Kirjain('K') => self.kirjaa(self.kursori),
            Syote::Numero(numero @ 1..=5) => self.lukitse(numero as usize - 1),
            Syote::Nuoli(rivit, 0) => self.siirra_kursoria(rivit),
            Syote::Klikkaus(x, y) => {
                if piirto::heitto_pisteessa(x, y) {
                    self.heita();
                } else if let Some(noppa) = piirto::noppa_pisteesta(x, y) {
                    self.lukitse(noppa);
                } else if let Some(rivi) = piirto::rivi_pisteesta(self.poyta.pelaajia(), x, y) {
                    self.kursori = rivi;
                    self.kirjaa(rivi);
                }
            }
            _ => {}
        }
    }

    /// Kursori hyppää kirjattujen rivien yli.
    fn siirra_kursoria(&mut self, suunta: i8) {
        let kirja = *self.poyta.kirja();
        let alku = self.kursori.indeksi() as i32;
        for askel in 1..=RIVEJA as i32 {
            let i = (alku + askel * suunta.signum() as i32).rem_euclid(RIVEJA as i32);
            let rivi = Rivi::KAIKKI[i as usize];
            if kirja.vapaa(rivi) {
                self.kursori = rivi;
                self.viivaus = None;
                return;
            }
        }
    }

    /// Paras vapaa rivi näillä nopilla: kursorin paikka heiton jälkeen.
    fn paras_rivi(&self) -> Rivi {
        let kirja = self.poyta.kirja();
        let mut paras: Option<(Rivi, u32)> = None;
        for rivi in kirja.vapaat() {
            let p = pisteet(rivi, &self.poyta.nopat);
            if paras.is_none_or(|(_, b)| p > b) {
                paras = Some((rivi, p));
            }
        }
        paras.map_or(self.kursori, |(rivi, _)| rivi)
    }

    pub fn heita(&mut self) {
        self.heita_aanella(true);
    }

    fn heita_aanella(&mut self, nakyva: bool) {
        if !self.poyta.heita(&mut self.arpa) {
            self.aanet.soita(Aani::Virhe);
            return;
        }
        self.viivaus = None;
        self.muutoksia = self.muutoksia.wrapping_add(1);
        if nakyva {
            self.heitto_ms = Some(self.kello_ms);
            self.aanet.soita(Aani::Pomppu);
            self.aanet.soita_viiveella(Aani::Pomppu, 120);
            self.aanet.soita_viiveella(Aani::Liuku, HEITTO_MS - 60);
        }
        if !self.poyta.kone_vuorossa() {
            self.kursori = self.paras_rivi();
            let jaljella = klassikot8000::jatsi::HEITTOJA - self.poyta.heitetty;
            self.viesti = if jaljella > 0 {
                muotoile("jatsi.heittoja_jaljella", &[&jaljella])
            } else {
                kaanna("jatsi.valitse_rivi").to_string()
            };
        }
    }

    pub fn lukitse(&mut self, noppa: usize) {
        if self.poyta.lukitse(noppa) {
            self.viivaus = None;
            self.aanet.soita(Aani::Tiili);
        } else {
            self.aanet.soita(Aani::Virhe);
        }
    }

    /// Ihmisen kirjaus. Nolla viivaa rivin vasta toisella painalluksella.
    pub fn kirjaa(&mut self, rivi: Rivi) {
        if !self.poyta.voi_kirjata(rivi) {
            self.aanet.soita(Aani::Virhe);
            return;
        }
        if pisteet(rivi, &self.poyta.nopat) == 0 && self.viivaus != Some(rivi) {
            self.viivaus = Some(rivi);
            self.viesti = muotoile("jatsi.viivataanko", &[&rivin_nimi(rivi)]);
            self.aanet.soita(Aani::Virhe);
            return;
        }
        self.kirjaa_suoraan(rivi, true);
    }

    fn kirjaa_suoraan(&mut self, rivi: Rivi, aanella: bool) {
        let Some(kirjaus) = self.poyta.kirjaa(rivi) else {
            return;
        };
        self.viivaus = None;
        self.hiiren_rivi = None;
        self.muutoksia = self.muutoksia.wrapping_add(1);
        let nimi = self.nimi(kirjaus.pelaaja);
        self.viesti = if kirjaus.pisteet > 0 {
            muotoile(
                "jatsi.kirjasi",
                &[&nimi, &rivin_nimi(rivi), &kirjaus.pisteet],
            )
        } else {
            muotoile("jatsi.viivasi", &[&nimi, &rivin_nimi(rivi)])
        };
        if kirjaus.bonus {
            let bonus = muotoile("jatsi.bonus_viesti", &[&klassikot8000::jatsi::BONUS]);
            self.viesti = format!("{} {bonus}", self.viesti);
        }
        if aanella {
            let aani = if kirjaus.pisteet > 0 {
                Aani::Yhdistys
            } else {
                Aani::Liuku
            };
            self.aanet.soita(aani);
        }
        if rivi == Rivi::Jatsi && kirjaus.pisteet > 0 {
            self.jatsi_ms = Some(self.kello_ms);
            for (i, &aani) in HELINAT.iter().enumerate() {
                self.aanet.soita_viiveella(aani, 70 * i as u32);
            }
        } else if kirjaus.bonus {
            for (i, &aani) in HELINAT[..3].iter().enumerate() {
                self.aanet.soita_viiveella(aani, 90 * i as u32);
            }
        }
        self.kone_ms = self.kello_ms + KONE_ASKEL_MS;
        self.kursori = Rivi::Ykkoset;
        if self.poyta.ohi() {
            self.paata();
        } else if !self.poyta.kone_vuorossa() {
            let seuraava = muotoile("jatsi.vuorossa", &[&self.nimi(self.poyta.vuorossa)]);
            self.viesti = format!("{} {seuraava}", self.viesti);
        }
    }

    /// Koneen yksi askel. Näkyvä askel pyörittää nopat ja odottaa ennen seuraavaa.
    fn koneen_askel(&mut self, nakyva: bool) {
        match koneen_toimi(self.vaikeus, &self.poyta) {
            Toimi::Heita => {
                self.heita_aanella(nakyva);
                self.kone_ms = self.kello_ms + HEITTO_MS + KONE_ASKEL_MS / 2;
            }
            Toimi::Lukitse(lukitut) => {
                self.poyta.lukitut = lukitut;
                self.muutoksia = self.muutoksia.wrapping_add(1);
                if nakyva {
                    self.aanet.soita(Aani::Tiili);
                }
                self.kone_ms = self.kello_ms + KONE_ASKEL_MS;
            }
            Toimi::Kirjaa(rivi) => self.kirjaa_suoraan(rivi, true),
        }
    }

    /// Välilyönti pelaa koneen vuoron loppuun kerralla.
    fn hoputa(&mut self) {
        let pelaaja = self.poyta.vuorossa;
        self.heitto_ms = None;
        while self.vaihe == Vaihe::Pelissa && self.poyta.vuorossa == pelaaja {
            let toimi = koneen_toimi(self.vaikeus, &self.poyta);
            if let Toimi::Kirjaa(rivi) = toimi {
                self.kirjaa_suoraan(rivi, true);
                // Yksi pelaaja voi jäädä vuoroon, kun muiden kirjat ovat täynnä.
                break;
            }
            self.koneen_askel(false);
        }
        if self.vaihe == Vaihe::Pelissa && self.poyta.kone_vuorossa() {
            self.kone_ms = self.kello_ms + KONE_ASKEL_MS;
        }
    }

    /// Peli päättyy: ihmisten tulokset listalle. Koneen voittanut saa tason kertoimen.
    fn paata(&mut self) {
        self.vaihe = Vaihe::Loppu;
        let voittajat = self.poyta.voittajat();
        let koneita = self.poyta.koneet.iter().any(|&kone| kone);
        let ihmisia = self.poyta.koneet.iter().filter(|&&kone| !kone).count();
        let summat = self.poyta.summat();
        self.tulokset.clear();
        for (pelaaja, &summa) in summat.iter().enumerate() {
            if self.poyta.koneet[pelaaja] {
                continue;
            }
            let voitti = voittajat.contains(&pelaaja);
            let kerroin = if voitti && koneita {
                self.vaikeus.kerroin()
            } else {
                1
            };
            let tulos = summa * kerroin;
            self.tulokset.push((pelaaja, tulos));
            self.taulu.paras = self.taulu.paras.max(tulos);
            if tulos > 0 && self.taulu.mahtuu(tulos) {
                self.taulu.lisaa(Tulos {
                    nimi: self.nimet[pelaaja].clone(),
                    pisteet: tulos,
                    aika: None,
                });
            }
        }
        self.uusi_tulos = true;
        let ihminen_voitti = voittajat.iter().any(|&p| !self.poyta.koneet[p]);
        let (soitto, puhe) = if ihminen_voitti && (koneita || ihmisia > 1) {
            (Aani::Voitto, Aani::Voitit)
        } else if ihminen_voitti {
            (Aani::Voitto, Aani::PeliOhi)
        } else {
            (Aani::Tappio, Aani::PeliOhi)
        };
        self.aanet.soita_viiveella(soitto, LOPPU_VIIVE_MS);
        self.aanet
            .soita_viiveella(puhe, LOPPU_VIIVE_MS + PUHE_VIIVE_MS);
    }

    /// Lopputuloksen otsikko: yksi ihminen voittaa tai häviää, moninpelissä voittaja nimellä.
    pub fn loppuotsikko(&self) -> String {
        let voittajat = self.poyta.voittajat();
        let ihmisia = self.poyta.koneet.iter().filter(|&&kone| !kone).count();
        if self.poyta.pelaajia() == 1 {
            return kaanna("yhteinen.peli_paattyi").to_string();
        }
        if voittajat.len() > 1 {
            return kaanna("jatsi.tasapeli").to_string();
        }
        let voittaja = voittajat[0];
        if ihmisia == 1 {
            let avain = if self.poyta.koneet[voittaja] {
                "jatsi.havisit"
            } else {
                "jatsi.voitit"
            };
            return kaanna(avain).to_string();
        }
        muotoile("jatsi.voitti", &[&self.nimi(voittaja)])
    }
}

/// Rivin nimi pöytäkirjassa ja viesteissä.
pub fn rivin_nimi(rivi: Rivi) -> &'static str {
    kaanna(match rivi {
        Rivi::Ykkoset => "jatsi.ykkoset",
        Rivi::Kakkoset => "jatsi.kakkoset",
        Rivi::Kolmoset => "jatsi.kolmoset",
        Rivi::Neloset => "jatsi.neloset",
        Rivi::Vitoset => "jatsi.vitoset",
        Rivi::Kuutoset => "jatsi.kuutoset",
        Rivi::Pari => "jatsi.pari",
        Rivi::KaksiParia => "jatsi.kaksi_paria",
        Rivi::KolmeSamaa => "jatsi.kolme_samaa",
        Rivi::NeljaSamaa => "jatsi.nelja_samaa",
        Rivi::PieniSuora => "jatsi.pieni_suora",
        Rivi::SuuriSuora => "jatsi.suuri_suora",
        Rivi::Tayskasi => "jatsi.tayskasi",
        Rivi::Sattuma => "jatsi.sattuma",
        Rivi::Jatsi => "jatsi.jatsi",
    })
}

impl Pelinakyma for Jatsi {
    fn lue_tallennuksesta(&mut self, tallennus: &Tallennus) {
        self.vaikeus = tallennus.asetukset.jatsi;
        if tallennus.jatsipaikat.len() == PELAAJIA {
            self.paikat = tallennus.jatsipaikat.clone();
        }
        if let Some(kesken) = &tallennus.jatsikesken {
            self.palauta(kesken.clone());
        }
    }

    fn kirjaa_asetukset(&self, asetukset: &mut Asetukset) {
        asetukset.jatsi = self.vaikeus;
    }

    fn kirjaa_tallennukseen(&self, tallennus: &mut Tallennus) {
        tallennus.jatsipaikat = self.paikat.clone();
        tallennus.jatsikesken = self.kesken();
    }

    fn kasittele(&mut self, syote: Syote) {
        Jatsi::kasittele(self, syote);
    }

    fn edista(&mut self, ms: u32) {
        Jatsi::edista(self, ms);
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

    fn suoraan_ruudulle(&self) -> bool {
        self.vaihe != Vaihe::Alku
    }

    /// Alkuruudun malli pyörii ja nopat pyörivät heitossa. Muuten kuva muuttuu syötteestä,
    /// koneen askelista ja valintakehyksen vilkkeestä.
    fn kuvan_avain(&self) -> Option<(u32, u32)> {
        if self.vaihe == Vaihe::Alku || self.animoi() {
            return None;
        }
        Some((self.muutoksia, self.kello_ms / VILKKU_MS))
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

    fn listat(&mut self) -> Vec<&mut Tulostaulu> {
        vec![&mut self.taulu]
    }

    fn nuolet_toistuvat(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests;
