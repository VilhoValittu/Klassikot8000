use klassikot8000::Arpa;
use klassikot8000::kaivo::{
    ASKEL_MS, Kaivos, Loyto, MAA_ALKAA, Ruutu, Solu, Tapahtuma, Toimi, Vaikeus,
};

use super::piirto::{self, Kaivoskuva};
use crate::kaynnistin::tallennus::{Asetukset, Tallennus};
use crate::yhteinen::aanet::{Aanet, Aani, PUHE_VIIVE_MS};
use crate::yhteinen::kieli::{kaanna, muotoile};
use crate::yhteinen::pelinakyma::Pelinakyma;
use crate::yhteinen::piirto::{Kehys, aanikytkimella};
use crate::yhteinen::syote::{Syote, Yleinen, yleinen};
use crate::yhteinen::tulokset::{Nettipeli, Tulostaulu, muokkaa_nimea};

pub const TEHOSTE_MS: u32 = 1000;
// Ikkunan raahaus voi pysäyttää silmukan: kaivos ei saa sen jälkeen sortua kerralla.
const SUURIN_KURO_MS: u32 = 100;
const LOPPU_VIIVE_MS: u32 = 250;
/// Paneeliin mahtuu kaksi viestiä pelin ohjeen yläpuolelle.
pub const VIESTEJA: usize = 2;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tehostelaji {
    /// Löydön tai nostettujen pisteiden luku nousee.
    Pisteet(u32),
    /// Pöly pöllähtää putoavan ruudun osuessa pohjaan.
    Poly,
    /// Kaivaja jäi alle tai putosi.
    Isku,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tehoste {
    pub laji: Tehostelaji,
    pub paikka: Ruutu,
    pub alku: u32,
}

pub struct Kaivo {
    pub vaihe: Vaihe,
    pub vaikeus: Vaikeus,
    pub kaivos: Kaivos,
    /// Kaivoksen kuva valmiiksi laskettuna. Lasketaan uudelleen, kun jokin ruutu muuttuu.
    pub kuva: Option<Kaivoskuva>,
    pub taulu: Tulostaulu,
    pub nimi: String,
    pub uusi_tulos: bool,
    /// Uusin viesti viimeisenä.
    pub viestit: Vec<String>,
    pub tehosteet: Vec<Tehoste>,
    pub lopeta: bool,
    pub aanet: Aanet,
    pub kello_ms: u32,
    /// Pelaaja poistuu kesken pelin: peli seisoo, ja nimen jälkeen palataan valikkoon.
    poistuu: bool,
    kertyma_ms: u32,
    arpa: Arpa,
}

fn tason_indeksi(vaikeus: Vaikeus) -> usize {
    Vaikeus::KAIKKI
        .iter()
        .position(|&taso| taso == vaikeus)
        .unwrap_or(0)
}

impl Kaivo {
    pub fn new(siemen: u64) -> Self {
        let mut arpa = Arpa::new(siemen);
        Self {
            vaihe: Vaihe::Alku,
            vaikeus: Vaikeus::Normaali,
            kaivos: Kaivos::new(Vaikeus::Normaali, 0, &mut arpa),
            kuva: None,
            taulu: Tulostaulu::new(Nettipeli::Kaivo),
            nimi: String::new(),
            uusi_tulos: false,
            viestit: Vec::new(),
            tehosteet: Vec::new(),
            lopeta: false,
            aanet: Aanet::default(),
            kello_ms: 0,
            poistuu: false,
            kertyma_ms: 0,
            arpa,
        }
    }

    /// Laskee kaivoksen kuvan uudelleen, jos jokin ruutu on muuttunut.
    pub fn paivita_kuva(&mut self) {
        if self
            .kuva
            .as_ref()
            .is_none_or(|kuva| kuva.versio != self.kaivos.versio)
        {
            self.kuva = Some(Kaivoskuva::new(&self.kaivos));
        }
    }

    pub fn edista(&mut self, ms: u32) {
        self.kello_ms = self.kello_ms.saturating_add(ms);
        let kello = self.kello_ms;
        self.tehosteet
            .retain(|t| kello < t.alku.saturating_add(TEHOSTE_MS));
        if self.poistuu || self.vaihe != Vaihe::Pelissa {
            return;
        }
        self.kertyma_ms = (self.kertyma_ms + ms).min(SUURIN_KURO_MS);
        while self.kertyma_ms >= ASKEL_MS {
            self.kertyma_ms -= ASKEL_MS;
            for tapahtuma in self.kaivos.askel(&mut self.arpa) {
                self.kasittele_tapahtuma(tapahtuma);
            }
            if self.vaihe != Vaihe::Pelissa {
                break;
            }
        }
        self.paivita_kuva();
    }

    fn viesti(&mut self, teksti: String) {
        self.viestit.push(teksti);
        if self.viestit.len() > VIESTEJA {
            self.viestit.remove(0);
        }
    }

    fn tehoste(&mut self, laji: Tehostelaji, paikka: Ruutu) {
        self.tehosteet.push(Tehoste {
            laji,
            paikka,
            alku: self.kello_ms,
        });
    }

    fn aani(&mut self, aani: Aani) {
        self.aanet.jono.push((aani, 0));
    }

    fn kasittele_tapahtuma(&mut self, tapahtuma: Tapahtuma) {
        match tapahtuma {
            Tapahtuma::Kaivoi(Solu::Hiekka) => self.aani(Aani::Liuku),
            Tapahtuma::Kaivoi(_) => self.aani(Aani::Kuokka),
            Tapahtuma::Loysi { loyto, paikka } => match loyto {
                Loyto::Kulta(arvo) => {
                    self.aani(Aani::Helina2);
                    self.tehoste(Tehostelaji::Pisteet(arvo), paikka);
                    self.viesti(muotoile("kaivo.kulta", &[&arvo]));
                }
                Loyto::Timantti(arvo) => {
                    self.aani(Aani::Helina6);
                    self.tehoste(Tehostelaji::Pisteet(arvo), paikka);
                    self.viesti(muotoile("kaivo.timantti", &[&arvo]));
                }
                Loyto::Oljy => {
                    self.aani(Aani::Pomppu);
                    self.viesti(kaanna("kaivo.oljy").to_string());
                }
                Loyto::Puut(maara) => {
                    self.aani(Aani::Liuku);
                    self.viesti(muotoile("kaivo.puut", &[&maara]));
                }
            },
            Tapahtuma::KiintioTaynna => {
                self.aani(Aani::Laivakello);
                self.viesti(kaanna("kaivo.kiintio_taynna").to_string());
            }
            Tapahtuma::Ritisee(_) => self.aani(Aani::Ritina),
            Tapahtuma::Sortui { solu, paikka } => {
                self.aani(if solu == Solu::Hiekka {
                    Aani::Liuku
                } else {
                    Aani::Sortuma
                });
                self.tehoste(Tehostelaji::Poly, paikka);
            }
            Tapahtuma::Murskautui { paikka, menetetty } => {
                self.aani(Aani::Karille);
                self.tehoste(Tehostelaji::Isku, paikka);
                let kyparat = self.kaivos.kyparat;
                self.viesti(muotoile("kaivo.murskautui", &[&menetetty, &kyparat]));
            }
            Tapahtuma::Putosi { paikka, .. } => {
                self.aani(Aani::Karille);
                self.tehoste(Tehostelaji::Isku, paikka);
                self.viesti(muotoile("kaivo.putosi", &[&self.kaivos.kyparat]));
            }
            Tapahtuma::Koysi { .. } => {
                self.aani(Aani::Virhe);
                self.viesti(muotoile("kaivo.koysi", &[&self.kaivos.kyparat]));
            }
            Tapahtuma::Nosti { pisteet } => {
                self.aani(Aani::Yhdistys);
                self.tehoste(Tehostelaji::Pisteet(pisteet), klassikot8000::kaivo::HISSI);
                let puuttuu = self.kaivos.kiintio.saturating_sub(self.kaivos.nostettu);
                self.viesti(muotoile("kaivo.nosto", &[&pisteet, &puuttuu]));
            }
            Tapahtuma::TasoValmis { bonus } => {
                self.aani(Aani::Voitto);
                self.viesti(muotoile("kaivo.taso_valmis", &[&bonus]));
            }
            Tapahtuma::UusiTaso => {
                self.aani(Aani::Alku);
                self.tehosteet.clear();
                self.viesti(muotoile("kaivo.uusi_taso", &[&(self.kaivos.taso + 1)]));
            }
            Tapahtuma::OljyLoppui => {
                self.aani(Aani::Virhe);
                self.viesti(kaanna("kaivo.oljy_loppui").to_string());
            }
            Tapahtuma::Ohi => {
                self.aanet.jono.push((Aani::Tappio, LOPPU_VIIVE_MS));
                self.aanet
                    .jono
                    .push((Aani::PeliOhi, LOPPU_VIIVE_MS + PUHE_VIIVE_MS));
                self.lopeta_peli();
            }
        }
        self.taulu.paras = self.taulu.paras.max(self.kaivos.pisteet);
    }

    fn lopeta_peli(&mut self) {
        self.vaihe = if self.taulu.mahtuu(self.kaivos.pisteet) {
            self.nimi.clear();
            Vaihe::Nimi
        } else {
            Vaihe::Tappio
        };
        self.uusi_tulos = true;
        self.viesti(muotoile("yhteinen.tulos", &[&self.kaivos.pisteet]));
    }

    /// Kirjaa odottavan tuloksen.
    pub fn vahvista_nimi(&mut self) {
        if self.vaihe != Vaihe::Nimi {
            return;
        }
        let pisteet = self.kaivos.pisteet;
        self.taulu.kirjaa(&mut self.nimi, pisteet);
        self.uusi_tulos = true;
        self.vaihe = Vaihe::Tappio;
        self.lopeta = self.poistuu;
    }

    /// Esc tai ikkunan sulkeminen kesken pelin: listalle yltävä tulos ei saa kadota.
    pub fn poistu(&mut self) {
        let kesken = matches!(self.vaihe, Vaihe::Pelissa | Vaihe::Tauko);
        if kesken && self.taulu.mahtuu(self.kaivos.pisteet) {
            self.nimi.clear();
            self.poistuu = true;
            self.vaihe = Vaihe::Nimi;
        } else {
            self.lopeta = true;
        }
    }

    fn kasittele_nimi(&mut self, syote: Syote) {
        match syote {
            // Tukeen painettu välilyönti ei saa kuitata nimeä ennen kuin sitä on kirjoitettu.
            Syote::Ammu if self.nimi.is_empty() => {}
            Syote::Ammu | Syote::Peru => self.vahvista_nimi(),
            _ => muokkaa_nimea(&mut self.nimi, syote),
        }
    }

    pub fn aloita_alusta(&mut self) {
        self.kaivos = Kaivos::new(self.vaikeus, 0, &mut self.arpa);
        self.paivita_kuva();
        self.vaihe = Vaihe::Pelissa;
        self.viestit.clear();
        self.viesti(muotoile("kaivo.uusi_taso", &[&1]));
        self.tehosteet.clear();
        self.kertyma_ms = 0;
        self.aani(Aani::Alku);
    }

    /// Ikkuna menetti kohdistuksen: katto ei saa sortua pelaajan poissa ollessa.
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
            Vaihe::Alku => self.kasittele_alku(syote),
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
            self.aloita_alusta();
        }
    }

    fn vaihda_tuki(&mut self) {
        if self.kaivos.vaihda_tuki() {
            self.aani(Aani::Tiili);
            self.paivita_kuva();
            return;
        }
        let k = self.kaivos.kaivaja;
        let tyhja = self.kaivos.solu(k.paikka) == Solu::Tyhja;
        if self.kaivos.tuet == 0 && tyhja && k.paikka.1 >= MAA_ALKAA && k.toimi == Toimi::Seisoo {
            self.aani(Aani::Virhe);
            self.viesti(kaanna("kaivo.ei_tukia").to_string());
        }
    }

    fn kasittele_pelissa(&mut self, syote: Syote) {
        let suunta = match syote {
            Syote::Nuoli(rivi, sarake) => Some((sarake, rivi)),
            Syote::Kirjain('W') => Some((0, -1)),
            Syote::Kirjain('A') => Some((-1, 0)),
            Syote::Kirjain('S') => Some((0, 1)),
            Syote::Kirjain('D') => Some((1, 0)),
            Syote::Klikkaus(x, y) => match piirto::ruutu_pisteesta(x, y) {
                Some(ruutu) => {
                    let oma = self.kaivos.kaivaja.paikka;
                    let (dx, dy) = (ruutu.0 - oma.0, ruutu.1 - oma.1);
                    if (dx, dy) == (0, 0) {
                        self.vaihda_tuki();
                        None
                    } else if dx.abs() >= dy.abs() {
                        Some((dx.signum() as i8, 0))
                    } else {
                        Some((0, dy.signum() as i8))
                    }
                }
                None => None,
            },
            _ => None,
        };
        if let Some(suunta) = suunta {
            self.kaivos.paina(suunta);
            return;
        }
        match syote {
            Syote::Ammu | Syote::OikeaKlikkaus(..) => self.vaihda_tuki(),
            Syote::Kirjain('R') => {
                let mut tapahtumat = Vec::new();
                if self.kaivos.heita_koysi(&mut tapahtumat) {
                    for tapahtuma in tapahtumat {
                        self.kasittele_tapahtuma(tapahtuma);
                    }
                }
            }
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

impl Pelinakyma for Kaivo {
    fn kasittele(&mut self, syote: Syote) {
        Kaivo::kasittele(self, syote);
    }

    fn edista(&mut self, ms: u32) {
        Kaivo::edista(self, ms);
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

    fn lue_tallennuksesta(&mut self, tallennus: &Tallennus) {
        self.vaikeus = tallennus.asetukset.kaivo;
    }

    fn kirjaa_asetukset(&self, asetukset: &mut Asetukset) {
        asetukset.kaivo = self.vaikeus;
    }

    fn toimintapeli(&self) -> bool {
        self.vaihe != Vaihe::Alku
    }

    fn aseta_nuolet(&mut self, vaaka: i8, pysty: i8) {
        let suunta = if vaaka != 0 {
            Some((vaaka.signum(), 0))
        } else if pysty != 0 {
            Some((0, pysty.signum()))
        } else {
            None
        };
        self.kaivos.aseta_pito(suunta);
    }

    fn keskeyta(&mut self) {
        Self::keskeyta(self);
    }

    // Pito luetaan `aseta_nuolet`-kutsusta omalla viiveellään. Näppäintoisto jättäisi irti
    // päästetyn nuolen jonoon ylimääräiseksi askeleeksi.
    fn nuolet_toistuvat(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests;
