use klassikot8000::Arpa;
use klassikot8000::haasteet::{self, HAASTEITA};
use klassikot8000::pasianssi::{Kohde, Lahde, Lauta, SARAKKEITA, Vaikeus, Variantti, aikapisteet};
use klassikot8000::{pyramidi as pyramidilogiikka, spider as spiderlogiikka};

use super::piirto::{self, Kohta};
use super::{haastenakyma, pyramidi, spider};
use crate::kaynnistin::tallennus::{Asetukset, Tallennus};
use crate::yhteinen::aanet::{Aanet, Aani, PUHE_VIIVE_MS};
use crate::yhteinen::kieli::{kaanna, muotoile};
use crate::yhteinen::pelinakyma::Pelinakyma;
use crate::yhteinen::piirto::{Kehys, VILKKU_MS};
use crate::yhteinen::syote::{Syote, Yleinen, yleinen};
use crate::yhteinen::tulokset::{Nettipeli, Tulos, Tulostaulu, nimi_valmis, ota_nimi};

const TUPLAKLIKKAUS_MS: u32 = 400;
// Pienempi liike on klikkauksen tärinää eikä vetämistä.
const VETO_RAJA: i32 = 5;
const VOITTO_VIIVE_MS: u32 = 250;
pub const VARIANTTEJA: usize = Variantti::KAIKKI.len();

/// Kunkin lajin haasteiden parhaat tulokset. Nolla tarkoittaa, ettei haastetta ole läpäisty.
pub type Haastepisteet = [[u32; HAASTEITA]; VARIANTTEJA];

/// Lajin oma top-lista.
pub fn nettipeli(variantti: Variantti) -> Nettipeli {
    match variantti {
        Variantti::Klondike => Nettipeli::Pasianssi,
        Variantti::Spider => Nettipeli::Spider,
        Variantti::Pyramidi => Nettipeli::Pyramidi,
    }
}
// Ylärivin kolmas paikka on tyhjä: pakka ja jäte ovat vasemmalla, perustat oikealla.
const TYHJA_YLASARAKE: usize = 2;
const PERUSTA_ALKAA: usize = 3;

pub(super) const EI_KAY: &str = "pasianssi.ei_kay";
pub(super) const PAKKA_LOPPU: &str = "pasianssi.pakka_loppu";
const EI_PERUUTUSTA: &str = "pasianssi.ei_peruutusta";
const EI_PERUTTAVAA: &str = "pasianssi.ei_peruttavaa";
const EI_KERATTAVAA: &str = "pasianssi.ei_kerattavaa";
const JUMISSA: &str = "pasianssi.jumissa";
const VOITTOVIESTI: &str = "pasianssi.voittoviesti";
const PYRAMIDI_VOITTOVIESTI: &str = "pyramidi.voittoviesti";
const SIIRROT_LOPPU: &str = "pasianssi.siirrot_loppu";
const HAASTE_VIESTI: &str = "pasianssi.haaste_viesti";
const HAASTE_VOITTO: &str = "pasianssi.haaste_voitto";
const HAASTE_JUMISSA: &str = "pasianssi.haaste_jumissa";
// Listan ruudukon leveys: nuolet ylös ja alas hyppäävät näin monta haastetta.
pub const HAASTERIVILLA: usize = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vaihe {
    Alku,
    /// Haastelista. Haaste itse pelataan `Pelissa`-vaiheessa.
    Haasteet,
    Pelissa,
    Voitto,
    Tappio,
    /// Voitto pääsi kolmen parhaan joukkoon ja odottaa nimimerkkiä.
    Nimi,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nappi {
    Uusi,
    /// Sama jako tai haaste uudestaan alusta.
    Alusta,
    Peruuta,
    Keraa,
    /// Spider jakaa ja Pyramidi nostaa. Klondikessa pakkaa klikataan suoraan.
    Nosta,
    Valikko,
}

/// Näppäimistön osoitin. `syvyys` kertoo, montako korttia sarakkeen päällimmäisestä
/// alaspäin osoitetaan, jotta jonon voi poimia.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Kursori {
    pub ylarivi: bool,
    pub sarake: usize,
    pub syvyys: usize,
}

/// Hiirellä kiinni pidetty kortti tai jono. `L` kertoo, mistä se otettiin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pito<L = Lahde> {
    pub lahde: L,
    /// Hiiren paikka suhteessa jonon alimman kortin kulmaan.
    pub tartunta: (i32, i32),
    alku: (i32, i32),
    pub liikkui: bool,
}

impl<L> Pito<L> {
    /// `kulma` on poimitun kortin vasen yläkulma ja `hiiri` paikka, josta se poimittiin.
    pub(super) fn new(lahde: L, kulma: (i32, i32), hiiri: (i32, i32)) -> Self {
        Self {
            lahde,
            tartunta: (hiiri.0 - kulma.0, hiiri.1 - kulma.1),
            alku: hiiri,
            liikkui: false,
        }
    }

    fn seuraa(&mut self, x: i32, y: i32) {
        if (x - self.alku.0).abs() > VETO_RAJA || (y - self.alku.1).abs() > VETO_RAJA {
            self.liikkui = true;
        }
    }
}

pub struct Pasianssi {
    pub vaihe: Vaihe,
    pub vaikeus: Vaikeus,
    /// Valittu laji. Vaihtuu vain alkuruudulla, joten peli ja sen tulos kuuluvat aina tälle.
    pub variantti: Variantti,
    pub lauta: Lauta,
    pub spider: spiderlogiikka::Lauta,
    pub spider_valittu: Option<spiderlogiikka::Jono>,
    pub spider_pito: Option<Pito<spiderlogiikka::Jono>>,
    pub pyramidi: pyramidilogiikka::Lauta,
    pub pyramidi_valittu: Option<pyramidilogiikka::Paikka>,
    pub pyramidi_pito: Option<Pito<pyramidilogiikka::Paikka>>,
    /// Kunkin lajin oma lista, järjestys kuten `Variantti::KAIKKI`.
    pub taulut: [Tulostaulu; VARIANTTEJA],
    /// Käynnissä oleva haaste. `None` on tavallinen peli.
    pub haaste: Option<usize>,
    /// Listan osoitin.
    pub haaste_kursori: usize,
    pub haastepisteet: Haastepisteet,
    /// Haasteiden summien lista.
    pub haastetaulu: Tulostaulu,
    /// Kysytään kerran. Summa kasvaa haaste kerrallaan, joten rivi päivittyy saman nimen alle.
    pub haastenimi: String,
    /// Kirjoitettavana oleva nimimerkki.
    pub nimi: String,
    /// Tosi, kun tulos odottaa tiedostoon kirjoitusta.
    pub uusi_tulos: bool,
    pub kursori: Kursori,
    pub kursori_nakyy: bool,
    pub valittu: Option<Lahde>,
    pub pito: Option<Pito>,
    pub hiiri: (i32, i32),
    pub viesti: String,
    /// Pyyntö palata käynnistimeen.
    pub lopeta: bool,
    pub aanet: Aanet,
    pub kello_ms: u32,
    pub peliaika_ms: u32,
    pub voitto_alku: u32,
    viime_klikkaus: Option<(Lahde, u32)>,
    arpa: Arpa,
    /// Arpa ennen viimeisintä jakoa, jotta saman jaon voi aloittaa alusta.
    jaon_arpa: Arpa,
}

impl Pasianssi {
    pub fn new(siemen: u64) -> Self {
        let mut arpa = Arpa::new(siemen);
        Self {
            vaihe: Vaihe::Alku,
            vaikeus: Vaikeus::Normaali,
            variantti: Variantti::Klondike,
            lauta: Lauta::new(Vaikeus::Normaali, &mut arpa),
            // Korvataan aloitettaessa, joten nämä eivät saa kuluttaa pelin arpaa.
            spider: spiderlogiikka::Lauta::new(Vaikeus::Normaali, &mut Arpa::new(siemen)),
            spider_valittu: None,
            spider_pito: None,
            pyramidi: pyramidilogiikka::Lauta::new(Vaikeus::Normaali, &mut Arpa::new(siemen)),
            pyramidi_valittu: None,
            pyramidi_pito: None,
            taulut: Variantti::KAIKKI.map(|variantti| Tulostaulu::new(nettipeli(variantti))),
            haaste: None,
            haaste_kursori: 0,
            haastepisteet: Haastepisteet::default(),
            haastetaulu: Tulostaulu::new(Nettipeli::Haasteet),
            haastenimi: String::new(),
            nimi: String::new(),
            uusi_tulos: false,
            kursori: Kursori {
                ylarivi: true,
                sarake: 0,
                syvyys: 0,
            },
            kursori_nakyy: false,
            valittu: None,
            pito: None,
            hiiri: (0, 0),
            viesti: String::new(),
            lopeta: false,
            aanet: Aanet::default(),
            kello_ms: 0,
            peliaika_ms: 0,
            voitto_alku: 0,
            viime_klikkaus: None,
            jaon_arpa: arpa.clone(),
            arpa,
        }
    }

    /// Valitun lajin paikka tuloslistoissa.
    pub fn lista_indeksi(&self) -> usize {
        self.variantti as usize
    }

    /// Valitun lajin lista.
    pub fn taulu(&self) -> &Tulostaulu {
        &self.taulut[self.lista_indeksi()]
    }

    /// Esillä oleva lista: nettilista, tai oma lista, jos verkkoon ei ole saatu yhteyttä.
    pub fn nakyva_lista(&self) -> &[Tulos] {
        self.taulu().nakyva()
    }

    /// Top-listan tulos: kaikkien haasteiden parhaiden pisteiden summa.
    pub fn haasteiden_summa(&self) -> u32 {
        self.haastepisteet.iter().flatten().sum()
    }

    pub fn lapaistyja_haasteita(&self) -> usize {
        self.haastepisteet
            .iter()
            .flatten()
            .filter(|&&pisteet| pisteet > 0)
            .count()
    }

    /// Nimikyselyn näyttämä tulos.
    pub fn nimen_pisteet(&self) -> u32 {
        if self.haaste.is_some() {
            self.haasteiden_summa()
        } else {
            self.pisteet()
        }
    }

    pub fn netti_vastasi(&self) -> bool {
        self.taulu().netti.is_some()
    }

    /// Käynnissä olevan laudan taso.
    pub fn laudan_vaikeus(&self) -> Vaikeus {
        match self.variantti {
            Variantti::Klondike => self.lauta.vaikeus,
            Variantti::Spider => self.spider.vaikeus,
            Variantti::Pyramidi => self.pyramidi.vaikeus,
        }
    }

    /// Mitä voitto antaisi juuri nyt. Hävitty peli ei anna mitään.
    pub fn pisteet(&self) -> u32 {
        if self.vaihe == Vaihe::Tappio {
            return 0;
        }
        aikapisteet(self.laudan_vaikeus(), self.peliaika_ms / 1000)
    }

    pub fn edista(&mut self, ms: u32) {
        self.kello_ms = self.kello_ms.saturating_add(ms);
        if self.vaihe == Vaihe::Pelissa {
            self.peliaika_ms = self.peliaika_ms.saturating_add(ms);
        }
    }

    /// Kuvassa näkyvät ajat: peliaika sekunteina ja kursorin vilkku. `None` tarkoittaa, että
    /// kuva elää joka ruudussa. Muuten pelinäkymä muuttuu vain syötteestä.
    pub fn ajan_avain(&self) -> Option<(u32, u32)> {
        match self.vaihe {
            Vaihe::Alku | Vaihe::Voitto | Vaihe::Nimi => None,
            // Lista muuttuu vain syötteestä, joten se ei kuormita konetta levossa.
            Vaihe::Haasteet => Some((0, 0)),
            Vaihe::Pelissa | Vaihe::Tappio => {
                let vilkkuu = self.kursori_nakyy && self.vaihe == Vaihe::Pelissa;
                let vilkku = if vilkkuu {
                    self.kello_ms / VILKKU_MS
                } else {
                    0
                };
                Some((self.peliaika_ms / 1000, vilkku))
            }
        }
    }

    pub fn aloita(&mut self) {
        self.jaon_arpa = self.arpa.clone();
        self.arpa = self.jaa();
    }

    /// Jakaa `jaon_arpa`-tilasta ja palauttaa arvan jaon jälkeen.
    fn jaa(&mut self) -> Arpa {
        let mut arpa = self.jaon_arpa.clone();
        match self.variantti {
            Variantti::Klondike => self.lauta = Lauta::new(self.vaikeus, &mut arpa),
            Variantti::Spider => {
                self.spider = spiderlogiikka::Lauta::new(self.vaikeus, &mut arpa);
            }
            Variantti::Pyramidi => {
                self.pyramidi = pyramidilogiikka::Lauta::new(self.vaikeus, &mut arpa);
            }
        }
        self.haaste = None;
        self.alusta_peli();
        if !self.laudalla_siirtoja() {
            self.siirrot_loppuivat();
        }
        arpa
    }

    /// Sama jako tai haaste uudestaan. Taso ja laji vaihtuvat vain alkuruudulla, joten jako toistuu samana.
    fn aloita_alusta(&mut self) {
        match self.haaste {
            Some(haaste) => self.aloita_haaste(haaste),
            None => {
                self.jaa();
            }
        }
    }

    /// Liittyy kesken olevaan peliin. Sen säännöt tulevat haasteesta eivätkä alkuruudun tasosta.
    pub fn aloita_haaste(&mut self, haaste: usize) {
        let ladattu = match self.variantti {
            Variantti::Klondike => haasteet::klondike(haaste).map(|l| self.lauta = l).is_some(),
            Variantti::Spider => haasteet::spider(haaste).map(|l| self.spider = l).is_some(),
            Variantti::Pyramidi => haasteet::pyramidi(haaste)
                .map(|l| self.pyramidi = l)
                .is_some(),
        };
        if ladattu {
            self.haaste = Some(haaste);
            self.alusta_peli();
            self.viesti = kaanna(HAASTE_VIESTI).to_string();
        }
    }

    fn alusta_peli(&mut self) {
        self.vaihe = Vaihe::Pelissa;
        self.peliaika_ms = 0;
        self.viesti.clear();
        self.unohda_poiminta();
        // Spiderissa ylärivillä ei ole mitään osoitettavaa.
        self.kursori = Kursori {
            ylarivi: self.variantti != Variantti::Spider,
            sarake: 0,
            syvyys: 0,
        };
        self.aanet.soita(Aani::Alku);
    }

    /// Uusi peli vie sinne, mistä tähän peliin tultiin.
    fn palaa_valintaan(&mut self) {
        self.vaihe = if self.haaste.take().is_some() {
            Vaihe::Haasteet
        } else {
            Vaihe::Alku
        };
    }

    fn siirrot_loppuivat(&mut self) {
        self.vaihe = Vaihe::Tappio;
        self.viesti = kaanna(SIIRROT_LOPPU).to_string();
        self.aanet.soita_viiveella(Aani::Tappio, VOITTO_VIIVE_MS);
        self.aanet
            .soita_viiveella(Aani::EiSiirtoja, VOITTO_VIIVE_MS + PUHE_VIIVE_MS);
    }

    fn unohda_poiminta(&mut self) {
        self.valittu = None;
        self.pito = None;
        self.spider_valittu = None;
        self.spider_pito = None;
        self.pyramidi_valittu = None;
        self.pyramidi_pito = None;
        self.viime_klikkaus = None;
    }

    fn on_poiminta(&self) -> bool {
        self.valittu.is_some()
            || self.pito.is_some()
            || self.spider_valittu.is_some()
            || self.spider_pito.is_some()
            || self.pyramidi_valittu.is_some()
            || self.pyramidi_pito.is_some()
    }

    fn laudalla_siirtoja(&self) -> bool {
        match self.variantti {
            Variantti::Klondike => self.lauta.on_siirtoja(),
            Variantti::Spider => self.spider.on_siirtoja(),
            Variantti::Pyramidi => self.pyramidi.on_siirtoja(),
        }
    }

    fn lauta_voitettu(&self) -> bool {
        match self.variantti {
            Variantti::Klondike => self.lauta.on_voitettu(),
            Variantti::Spider => self.spider.on_voitettu(),
            Variantti::Pyramidi => self.pyramidi.on_voitettu(),
        }
    }

    fn laudalla_hyodyllisia_siirtoja(&self) -> bool {
        match self.variantti {
            Variantti::Klondike => self.lauta.on_hyodyllisia_siirtoja(),
            Variantti::Spider => self.spider.on_hyodyllisia_siirtoja(),
            // Pyramidissa jokainen jäljellä oleva siirto poistaa kortteja.
            Variantti::Pyramidi => true,
        }
    }

    fn nappi_kohdassa(&self, x: i32, y: i32) -> Option<Nappi> {
        match self.variantti {
            Variantti::Klondike => piirto::nappi_pisteesta(x, y),
            Variantti::Spider => spider::piirto::nappi_pisteesta(x, y),
            Variantti::Pyramidi => pyramidi::piirto::nappi_pisteesta(x, y),
        }
    }

    /// Kirjaa odottavan tuloksen.
    pub fn vahvista_nimi(&mut self) {
        if self.vaihe != Vaihe::Nimi {
            return;
        }
        let nimi = ota_nimi(&mut self.nimi);
        if self.haaste.is_some() {
            self.haastenimi = nimi;
            self.kirjaa_haasteiden_summa();
            self.vaihe = Vaihe::Voitto;
            return;
        }
        let pisteet = self.pisteet();
        let i = self.lista_indeksi();
        self.taulut[i].lisaa(Tulos { nimi, pisteet });
        self.uusi_tulos = true;
        self.vaihe = Vaihe::Voitto;
    }

    fn kirjaa_haasteiden_summa(&mut self) {
        let tulos = Tulos {
            nimi: self.haastenimi.clone(),
            pisteet: self.haasteiden_summa(),
        };
        self.haastetaulu.lisaa(tulos);
        self.uusi_tulos = true;
    }

    /// Paras tulos jää haasteelle, ja parannus nostaa top-listan summaa.
    fn haaste_voitettu(&mut self, haaste: usize) {
        let pisteet = self.pisteet();
        let paras = &mut self.haastepisteet[self.variantti as usize][haaste];
        let parani = pisteet > *paras;
        if parani {
            *paras = pisteet;
            self.uusi_tulos = true;
        }
        let summa = self.haasteiden_summa();
        self.viesti = muotoile(HAASTE_VOITTO, &[&pisteet, &summa]);
        if parani && self.haastetaulu.mahtuu(summa) {
            if self.haastenimi.is_empty() {
                self.nimi.clear();
                self.vaihe = Vaihe::Nimi;
            } else {
                self.kirjaa_haasteiden_summa();
            }
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
            Yleinen::Peru if self.vaihe == Vaihe::Haasteet => {
                self.vaihe = Vaihe::Alku;
                return;
            }
            Yleinen::Peru if self.on_poiminta() => return self.unohda_poiminta(),
            Yleinen::Peru => {
                self.lopeta = true;
                return;
            }
            Yleinen::Aanet | Yleinen::Ohita => return,
        }
        match self.vaihe {
            Vaihe::Alku => self.kasittele_alku(syote),
            Vaihe::Haasteet => self.kasittele_haasteet(syote),
            Vaihe::Pelissa => self.kasittele_pelissa(syote),
            Vaihe::Voitto | Vaihe::Tappio => self.kasittele_loppu(syote),
            Vaihe::Nimi => {}
        }
    }

    fn kasittele_haasteet(&mut self, syote: Syote) {
        match syote {
            Syote::Numero(numero) => {
                let lajit = Variantti::KAIKKI;
                if let Some(&laji) = (numero as usize).checked_sub(1).and_then(|i| lajit.get(i)) {
                    self.variantti = laji;
                }
            }
            Syote::Nuoli(drivi, dsarake) => {
                let muutos = drivi as isize * HAASTERIVILLA as isize + dsarake as isize;
                if let Some(uusi) = self.haaste_kursori.checked_add_signed(muutos)
                    && uusi < HAASTEITA
                {
                    self.haaste_kursori = uusi;
                }
            }
            Syote::Ammu => self.aloita_haaste(self.haaste_kursori),
            Syote::Kirjain('E') => self.vaihe = Vaihe::Alku,
            Syote::Klikkaus(x, y) => {
                if let Some(laji) = haastenakyma::laji_pisteesta(x, y) {
                    self.variantti = laji;
                } else if let Some(haaste) = haastenakyma::haaste_pisteesta(x, y) {
                    self.haaste_kursori = haaste;
                    self.aloita_haaste(haaste);
                } else if haastenakyma::takaisin_pisteessa(x, y) {
                    self.vaihe = Vaihe::Alku;
                }
            }
            _ => {}
        }
    }

    fn kasittele_alku(&mut self, syote: Syote) {
        let haasteisiin = match syote {
            Syote::Kirjain('H') => true,
            Syote::Klikkaus(x, y) => piirto::haastenappi_pisteessa(x, y),
            _ => false,
        };
        if haasteisiin {
            self.vaihe = Vaihe::Haasteet;
            return;
        }
        // Ylös ja alas vaihtavat lajia, vasen ja oikea tasoa.
        let laji = match syote {
            Syote::Nuoli(muutos, 0) => {
                let lajit = Variantti::KAIKKI;
                let i = self.lista_indeksi().saturating_add_signed(muutos as isize);
                Some(lajit[i.min(lajit.len() - 1)])
            }
            Syote::Klikkaus(x, y) => piirto::variantti_pisteesta(x, y),
            _ => None,
        };
        if let Some(laji) = laji {
            self.variantti = laji;
            return;
        }
        let tasot = Vaikeus::KAIKKI;
        let valittu = match syote {
            Syote::Numero(numero) => (numero as usize)
                .checked_sub(1)
                .and_then(|i| tasot.get(i).copied()),
            Syote::Nuoli(_, muutos) => tasot.iter().position(|&t| t == self.vaikeus).map(|i| {
                let uusi = i.saturating_add_signed(muutos as isize);
                tasot[uusi.min(tasot.len() - 1)]
            }),
            Syote::Klikkaus(x, y) => piirto::vaikeus_pisteesta(x, y),
            _ => None,
        };
        if let Some(vaikeus) = valittu {
            self.vaikeus = vaikeus;
        } else if matches!(syote, Syote::Ammu | Syote::Klikkaus(..)) {
            self.aloita();
        }
    }

    fn kasittele_loppu(&mut self, syote: Syote) {
        let nappi = match syote {
            Syote::Kirjain('K') => Nappi::Uusi,
            Syote::Kirjain('R') => Nappi::Alusta,
            Syote::Kirjain('E') => Nappi::Valikko,
            Syote::Klikkaus(x, y) => match self.nappi_kohdassa(x, y) {
                Some(nappi) => nappi,
                None => return,
            },
            _ => return,
        };
        match nappi {
            Nappi::Uusi => self.palaa_valintaan(),
            Nappi::Alusta => self.aloita_alusta(),
            Nappi::Valikko => self.lopeta = true,
            Nappi::Peruuta | Nappi::Keraa | Nappi::Nosta => {}
        }
    }

    fn kasittele_pelissa(&mut self, syote: Syote) {
        let laji = self.variantti;
        match syote {
            Syote::Hiiri(x, y) => self.hiiri_liikkuu(x, y),
            Syote::Klikkaus(x, y) => {
                self.hiiri = (x, y);
                self.kursori_nakyy = false;
                if let Some(nappi) = self.nappi_kohdassa(x, y) {
                    self.paina(nappi);
                    return;
                }
                match laji {
                    Variantti::Klondike => self.klikkaus(x, y),
                    Variantti::Spider => self.spider_klikkaus(x, y),
                    Variantti::Pyramidi => self.pyramidi_klikkaus(x, y),
                }
            }
            Syote::Vapautus(x, y) => match laji {
                Variantti::Klondike => self.vapautus(x, y),
                Variantti::Spider => self.spider_vapautus(x, y),
                Variantti::Pyramidi => self.pyramidi_vapautus(x, y),
            },
            Syote::Nuoli(drivi, dsarake) => {
                self.kursori_nakyy = true;
                match laji {
                    Variantti::Klondike => self.liikuta_kursoria(drivi, dsarake),
                    Variantti::Spider => self.spider_liikuta_kursoria(drivi, dsarake),
                    Variantti::Pyramidi => self.pyramidi_liikuta_kursoria(drivi, dsarake),
                }
            }
            Syote::Ammu => {
                self.kursori_nakyy = true;
                match laji {
                    Variantti::Klondike => self.kursorin_toiminto(),
                    Variantti::Spider => self.spider_kursorin_toiminto(),
                    Variantti::Pyramidi => self.pyramidi_kursorin_toiminto(),
                }
            }
            Syote::Kirjain('N') => self.nosta(),
            Syote::Kirjain('U') => self.paina(Nappi::Peruuta),
            Syote::Kirjain('A') if laji == Variantti::Klondike => self.paina(Nappi::Keraa),
            Syote::Kirjain('K') => self.paina(Nappi::Uusi),
            Syote::Kirjain('R') => self.paina(Nappi::Alusta),
            Syote::Kirjain('E') => self.paina(Nappi::Valikko),
            _ => {}
        }
    }

    fn paina(&mut self, nappi: Nappi) {
        self.unohda_poiminta();
        match nappi {
            Nappi::Nosta => self.nosta(),
            Nappi::Uusi => self.palaa_valintaan(),
            Nappi::Alusta => self.aloita_alusta(),
            Nappi::Valikko => self.lopeta = true,
            Nappi::Peruuta => self.peruuta(),
            Nappi::Keraa => {
                if self.lauta.keraa() > 0 {
                    self.siirto_tehty(Aani::Yhdistys);
                } else {
                    self.virhe(EI_KERATTAVAA);
                }
            }
        }
    }

    fn peruuta(&mut self) {
        // Spiderissa perua saa aina: neljää maata ei muuten juuri voita.
        let sallittu = match self.variantti {
            Variantti::Klondike => self.lauta.voi_perua(),
            Variantti::Spider => true,
            Variantti::Pyramidi => self.pyramidi.voi_perua(),
        };
        let peruttu = sallittu
            && match self.variantti {
                Variantti::Klondike => self.lauta.peruuta(),
                Variantti::Spider => self.spider.peruuta(),
                Variantti::Pyramidi => self.pyramidi.peruuta(),
            };
        if !sallittu {
            self.virhe(EI_PERUUTUSTA);
        } else if peruttu {
            self.viesti.clear();
            self.kursori.syvyys = 0;
            self.aanet.soita(Aani::Liuku);
        } else {
            self.virhe(EI_PERUTTAVAA);
        }
    }

    pub(super) fn nosta(&mut self) {
        self.unohda_poiminta();
        let nostettu = match self.variantti {
            Variantti::Klondike => self.lauta.nosta(),
            Variantti::Pyramidi => self.pyramidi.nosta(),
            Variantti::Spider => return self.spider_jaa(),
        };
        if nostettu {
            self.siirto_tehty(Aani::Liuku);
        } else {
            self.virhe(PAKKA_LOPPU);
        }
    }

    pub(super) fn virhe(&mut self, viesti: &str) {
        self.viesti = kaanna(viesti).to_string();
        self.aanet.soita(Aani::Virhe);
    }

    /// Yhteinen loppu kaikille laudan muuttaneille siirroille.
    pub(super) fn siirto_tehty(&mut self, aani: Aani) {
        self.valittu = None;
        self.pito = None;
        self.spider_valittu = None;
        self.spider_pito = None;
        self.pyramidi_valittu = None;
        self.pyramidi_pito = None;
        // Pyramidissa `syvyys` on osoittimen rivi, joten se saa jäädä paikalleen.
        if self.variantti != Variantti::Pyramidi {
            self.kursori.syvyys = 0;
        }
        self.aanet.soita(aani);
        if self.lauta_voitettu() {
            self.vaihe = Vaihe::Voitto;
            self.voitto_alku = self.kello_ms;
            self.aanet.soita_viiveella(Aani::Voitto, VOITTO_VIIVE_MS);
            self.aanet
                .soita_viiveella(Aani::Voitit, VOITTO_VIIVE_MS + PUHE_VIIVE_MS);
            if let Some(haaste) = self.haaste {
                self.haaste_voitettu(haaste);
                return;
            }
            let pisteet = self.pisteet();
            let oma = &mut self.taulut[self.variantti as usize].paras;
            if pisteet > *oma {
                *oma = pisteet;
                self.uusi_tulos = true;
            }
            let viesti = if self.variantti == Variantti::Pyramidi {
                PYRAMIDI_VOITTOVIESTI
            } else {
                VOITTOVIESTI
            };
            self.viesti = muotoile(viesti, &[&pisteet]);
            if self.taulu().mahtuu(pisteet) {
                self.nimi.clear();
                self.vaihe = Vaihe::Nimi;
            }
        } else if !self.laudalla_siirtoja() {
            // Haaste ei pääty tappioon: voitto on aina jäljellä peruuttamalla.
            if self.haaste.is_some() {
                self.viesti = kaanna(HAASTE_JUMISSA).to_string();
            } else {
                self.siirrot_loppuivat();
            }
        } else if self.laudalla_hyodyllisia_siirtoja() {
            self.viesti.clear();
        } else {
            self.viesti = kaanna(JUMISSA).to_string();
        }
    }

    fn yrita_siirtoa(&mut self, lahde: Lahde, kohde: Kohde) -> bool {
        if !self.lauta.siirra(lahde, kohde) {
            return false;
        }
        let aani = if matches!(kohde, Kohde::Perusta(_)) {
            Aani::Yhdistys
        } else {
            Aani::Liuku
        };
        self.siirto_tehty(aani);
        true
    }

    fn hiiri_liikkuu(&mut self, x: i32, y: i32) {
        self.hiiri = (x, y);
        self.kursori_nakyy = false;
        if let Some(pito) = &mut self.pito {
            pito.seuraa(x, y);
        }
        if let Some(pito) = &mut self.spider_pito {
            pito.seuraa(x, y);
        }
        if let Some(pito) = &mut self.pyramidi_pito {
            pito.seuraa(x, y);
        }
    }

    fn klikkaus(&mut self, x: i32, y: i32) {
        let kohta = piirto::kohta_pisteesta(&self.lauta, x, y);
        let lahde = kohta.and_then(|kohta| self.kohdan_lahde(kohta));

        // Tuplaklikkaus vie kortin perustalle.
        if let (Some(lahde), Some((viime, aika))) = (lahde, self.viime_klikkaus)
            && viime == lahde
            && self.kello_ms.saturating_sub(aika) <= TUPLAKLIKKAUS_MS
            && self.lauta.perustalle(lahde)
        {
            self.siirto_tehty(Aani::Yhdistys);
            self.viime_klikkaus = None;
            return;
        }
        self.viime_klikkaus = lahde.map(|lahde| (lahde, self.kello_ms));

        if let Some(valittu) = self.valittu.take()
            && let Some(kohde) = piirto::kohde_pisteesta(x, y)
        {
            if self.yrita_siirtoa(valittu, kohde) {
                return;
            }
            if lahde.is_none() || lahde == Some(valittu) {
                if lahde.is_none() {
                    self.virhe(EI_KAY);
                }
                return;
            }
        }

        match (kohta, lahde) {
            (Some(Kohta::Pakka), _) => self.nosta(),
            (_, Some(lahde)) => {
                let kulma = piirto::kortin_paikka(&self.lauta, lahde);
                self.pito = Some(Pito::new(lahde, kulma, (x, y)));
            }
            _ => {}
        }
    }

    fn vapautus(&mut self, x: i32, y: i32) {
        let Some(pito) = self.pito.take() else {
            return;
        };
        if !pito.liikkui {
            self.valittu = Some(pito.lahde);
            return;
        }
        // Kohde luetaan osoittimesta ja sitten kortin keskeltä, jos osoitin jäi rakoon.
        let keski = piirto::vedetyn_keskipiste(x - pito.tartunta.0, y - pito.tartunta.1);
        let kohde =
            piirto::kohde_pisteesta(x, y).or_else(|| piirto::kohde_pisteesta(keski.0, keski.1));
        if let Some(kohde) = kohde
            && !self.yrita_siirtoa(pito.lahde, kohde)
        {
            self.virhe(EI_KAY);
        }
    }

    /// Mitä kortteja kohdasta voi poimia.
    fn kohdan_lahde(&self, kohta: Kohta) -> Option<Lahde> {
        let lahde = match kohta {
            Kohta::Pakka => return None,
            Kohta::Jate => Lahde::Jate,
            Kohta::Perusta(perusta) => Lahde::Perusta(perusta),
            Kohta::Sarake { sarake, kortti } => Lahde::Sarake {
                sarake,
                kortti: kortti?,
            },
        };
        self.lauta.siirrettavat(lahde).map(|_| lahde)
    }

    fn liikuta_kursoria(&mut self, drivi: i8, dsarake: i8) {
        let k = &mut self.kursori;
        if dsarake != 0 {
            k.sarake = k
                .sarake
                .saturating_add_signed(dsarake as isize)
                .min(SARAKKEITA - 1);
            if k.ylarivi && k.sarake == TYHJA_YLASARAKE {
                k.sarake = k.sarake.saturating_add_signed(dsarake as isize);
            }
            k.syvyys = 0;
        }
        if drivi > 0 {
            if k.ylarivi {
                k.ylarivi = false;
                k.syvyys = 0;
            } else {
                k.syvyys = k.syvyys.saturating_sub(1);
            }
        } else if drivi < 0 && !k.ylarivi {
            let sarake = &self.lauta.tila.sarakkeet[k.sarake];
            let nakyvia = sarake.kortit.len() - sarake.piilossa;
            if k.syvyys + 1 < nakyvia {
                k.syvyys += 1;
            } else {
                k.ylarivi = true;
                k.syvyys = 0;
                if k.sarake == TYHJA_YLASARAKE {
                    k.sarake -= 1;
                }
            }
        }
    }

    /// Osoittimen kohta. Sarake on voinut lyhentyä siirrossa, joten syvyys rajataan.
    pub fn kursorin_kohta(&self) -> Option<Kohta> {
        let k = self.kursori;
        if k.ylarivi {
            return match k.sarake {
                0 => Some(Kohta::Pakka),
                1 => Some(Kohta::Jate),
                TYHJA_YLASARAKE => None,
                s => Some(Kohta::Perusta(s - PERUSTA_ALKAA)),
            };
        }
        let sarake = &self.lauta.tila.sarakkeet[k.sarake];
        Some(Kohta::Sarake {
            sarake: k.sarake,
            kortti: sarake.kortit.len().checked_sub(1 + k.syvyys),
        })
    }

    fn kursorin_toiminto(&mut self) {
        let Some(kohta) = self.kursorin_kohta() else {
            return;
        };
        if let Some(valittu) = self.valittu.take() {
            let kohde = match kohta {
                Kohta::Perusta(perusta) => Some(Kohde::Perusta(perusta)),
                Kohta::Sarake { sarake, .. } => Some(Kohde::Sarake(sarake)),
                Kohta::Pakka | Kohta::Jate => None,
            };
            // Toinen painallus samaan kohtaan vain peruu valinnan.
            let sama = self.kohdan_lahde(kohta) == Some(valittu);
            if let Some(kohde) = kohde
                && !sama
                && !self.yrita_siirtoa(valittu, kohde)
            {
                self.virhe(EI_KAY);
            }
            return;
        }
        if kohta == Kohta::Pakka {
            self.nosta();
        } else {
            self.valittu = self.kohdan_lahde(kohta);
        }
    }
}

impl Pelinakyma for Pasianssi {
    fn lue_tallennuksesta(&mut self, tallennus: &Tallennus) {
        self.vaikeus = tallennus.asetukset.pasianssi;
        self.variantti = tallennus.asetukset.variantti;
        self.haastepisteet = tallennus.haastepisteet;
        self.haastenimi.clone_from(&tallennus.haastenimi);
    }

    fn kirjaa_asetukset(&self, asetukset: &mut Asetukset) {
        asetukset.pasianssi = self.vaikeus;
        asetukset.variantti = self.variantti;
    }

    fn kirjaa_tallennukseen(&self, tallennus: &mut Tallennus) {
        tallennus.haastepisteet = self.haastepisteet;
        tallennus.haastenimi.clone_from(&self.haastenimi);
    }

    fn kasittele(&mut self, syote: Syote) {
        Pasianssi::kasittele(self, syote);
    }

    fn edista(&mut self, ms: u32) {
        Pasianssi::edista(self, ms);
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
        let mut listat: Vec<_> = self.taulut.iter_mut().collect();
        listat.push(&mut self.haastetaulu);
        listat
    }

    fn kuvan_avain(&self) -> Option<(u32, u32)> {
        self.ajan_avain()
    }

    fn suoraan_ruudulle(&self) -> bool {
        self.vaihe != Vaihe::Alku
    }
}

#[cfg(test)]
mod tests;
