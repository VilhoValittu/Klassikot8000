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

    fn keskeyta(&mut self) {
        Self::keskeyta(self);
    }

    // Yksi painallus on yksi ruutu: näppäintoisto veisi pohjassa pidetyllä nuolella monta.
    fn nuolet_toistuvat(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yhteinen::tulokset::Tulos;
    use klassikot8000::kaivo::{ALKU, HISSI, KYPARIA, PINTA, RIVEJA, SARAKKEITA};

    /// Aikaa pienissä paloissa: yksi `edista` ajaa enintään 100 ms.
    fn aja(peli: &mut Kaivo, ms: u32) {
        for _ in 0..ms.div_ceil(20) {
            peli.edista(20);
        }
    }

    fn pelissa() -> Kaivo {
        let mut peli = Kaivo::new(1);
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.vaihe, Vaihe::Pelissa);
        // Tasainen maa, jottei arvottu kenttä sotke testejä.
        for y in MAA_ALKAA..RIVEJA as i32 - 1 {
            for x in 0..SARAKKEITA as i32 {
                peli.kaivos.aseta((x, y), Solu::Maa);
            }
        }
        peli.aanet.jono.clear();
        peli
    }

    /// Viimeinen kypärä ja kivi kaivajan yläpuolella: peli päättyy kiven pudotessa.
    fn paattyva(pisteet: u32) -> Kaivo {
        let mut peli = pelissa();
        peli.kaivos.pisteet = pisteet;
        peli.kaivos.kyparat = 1;
        for x in 8..12 {
            peli.kaivos.aseta((x, 6), Solu::Tyhja);
        }
        peli.kaivos.aseta((10, 5), Solu::Kivi);
        peli.kaivos.kaivaja.paikka = (10, 6);
        peli
    }

    #[test]
    fn taso_valitaan_alussa_ja_peli_alkaa_silla() {
        let mut peli = Kaivo::new(1);
        peli.kasittele(Syote::Numero(3));
        assert_eq!(peli.vaikeus, Vaikeus::Vaikea);
        peli.kasittele(Syote::Nuoli(0, -1));
        assert_eq!(peli.vaikeus, Vaikeus::Normaali);
        let (x, y) = piirto::vaikeuden_keskipiste(Vaikeus::Helppo);
        peli.kasittele(Syote::Klikkaus(x, y));
        assert_eq!(peli.vaikeus, Vaikeus::Helppo);
        assert_eq!(peli.vaihe, Vaihe::Alku);
        peli.kasittele(Syote::Klikkaus(0, 300));
        assert_eq!(peli.vaihe, Vaihe::Pelissa);
        assert_eq!(peli.kaivos.vaikeus, Vaikeus::Helppo);
        assert!(peli.kuva.is_some());
        assert_eq!(peli.aanet.jono, [(Aani::Alku, 0)]);

        let mut asetukset = Asetukset::default();
        peli.kirjaa_asetukset(&mut asetukset);
        assert_eq!(asetukset.kaivo, Vaikeus::Helppo);
    }

    #[test]
    fn nuoli_kirjain_ja_klikkaus_kaivavat() {
        let mut peli = pelissa();
        peli.kasittele(Syote::Nuoli(1, 0));
        aja(&mut peli, 400);
        assert_eq!(peli.kaivos.kaivaja.paikka, (ALKU.0, MAA_ALKAA));
        assert!(peli.aanet.jono.contains(&(Aani::Kuokka, 0)));

        peli.kasittele(Syote::Kirjain('D'));
        aja(&mut peli, 400);
        assert_eq!(peli.kaivos.kaivaja.paikka, (ALKU.0 + 1, MAA_ALKAA));

        // Klikkaus kauas vasemmalle ja vähän alas: vaakasuunta voittaa.
        let (x, y) = piirto::ruudun_keskipiste((0, MAA_ALKAA + 1));
        peli.kasittele(Syote::Klikkaus(x, y));
        aja(&mut peli, 400);
        assert_eq!(peli.kaivos.kaivaja.paikka, (ALKU.0, MAA_ALKAA));
    }

    #[test]
    fn yksi_painallus_vie_yhden_ruudun_maan_allakin() {
        let mut peli = pelissa();
        peli.kasittele(Syote::Nuoli(0, 1));
        aja(&mut peli, 1000);
        assert_eq!(peli.kaivos.kaivaja.paikka, (ALKU.0 + 1, PINTA));

        // Maan alla valmiissa tunnelissa.
        for x in 5..9 {
            peli.kaivos.aseta((x, 6), Solu::Tyhja);
        }
        peli.kaivos.kaivaja.paikka = (6, 6);
        peli.kasittele(Syote::Kirjain('D'));
        aja(&mut peli, 1000);
        assert_eq!(peli.kaivos.kaivaja.paikka, (7, 6));
        // Kävelyn aikana annettu painallus odottaa vuoroaan: kaksi painallusta, kaksi ruutua.
        peli.kasittele(Syote::Nuoli(0, 1));
        aja(&mut peli, 20);
        peli.kasittele(Syote::Nuoli(1, 0));
        aja(&mut peli, 1000);
        assert_eq!(peli.kaivos.kaivaja.paikka, (8, 7));
    }

    #[test]
    fn valilyonti_pystyttaa_ja_purkaa_tuen_ja_kertoo_kun_tuet_loppuvat() {
        let mut peli = pelissa();
        peli.kasittele(Syote::Nuoli(1, 0));
        aja(&mut peli, 400);
        peli.kasittele(Syote::Ammu);
        let paikka = peli.kaivos.kaivaja.paikka;
        assert_eq!(peli.kaivos.solu(paikka), Solu::Tuki);
        assert!(peli.aanet.jono.contains(&(Aani::Tiili, 0)));
        let versio = peli.kuva.as_ref().unwrap().versio;
        assert_eq!(versio, peli.kaivos.versio);
        peli.kasittele(Syote::OikeaKlikkaus(0, 0));
        assert_eq!(peli.kaivos.solu(paikka), Solu::Tyhja);

        peli.kaivos.tuet = 0;
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.viestit.last().unwrap(), "Tuet loppuivat.");
        assert!(peli.aanet.jono.contains(&(Aani::Virhe, 0)));
    }

    #[test]
    fn r_heittaa_koyden_vain_maan_alla() {
        let mut peli = pelissa();
        peli.kasittele(Syote::Kirjain('R'));
        assert_eq!(peli.kaivos.kyparat, KYPARIA);
        peli.kasittele(Syote::Nuoli(1, 0));
        aja(&mut peli, 400);
        peli.kasittele(Syote::Kirjain('R'));
        assert_eq!(peli.kaivos.kyparat, KYPARIA - 1);
        assert_eq!(peli.kaivos.kaivaja.paikka, ALKU);
        assert!(peli.viestit.last().unwrap().contains('2'));
    }

    #[test]
    fn kulta_hissille_antaa_pisteet_ja_kuvan_muutokset_seuraavat() {
        let mut peli = pelissa();
        peli.kaivos.kiintio = 1000;
        peli.kaivos.saalis = 40;
        peli.kaivos.kaivaja.paikka = (HISSI.0 + 1, PINTA);
        peli.kasittele(Syote::Nuoli(0, -1));
        aja(&mut peli, 300);
        assert_eq!(peli.kaivos.pisteet, 40);
        assert_eq!(peli.taulu.paras, 40);
        assert!(peli.aanet.jono.contains(&(Aani::Yhdistys, 0)));
        assert_eq!(peli.tehosteet[0].laji, Tehostelaji::Pisteet(40));
        assert_eq!(peli.viestit.last().unwrap(), "Nostettu 40, puuttuu 960.");
    }

    #[test]
    fn sortuma_paattaa_viimeisen_kyparan_ja_kysyy_nimen() {
        let mut peli = paattyva(40);
        aja(&mut peli, 100);
        aja(&mut peli, 100);
        for _ in 0..20 {
            aja(&mut peli, 100);
        }
        assert_eq!(peli.vaihe, Vaihe::Nimi);
        assert!(peli.uusi_tulos);
        assert!(peli.aanet.jono.contains(&(Aani::Karille, 0)));
        assert!(peli.aanet.jono.contains(&(Aani::Ritina, 0)));
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.vaihe, Vaihe::Nimi);
        peli.kasittele(Syote::Kirjain('K'));
        peli.uusi_tulos = false;
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.vaihe, Vaihe::Tappio);
        assert!(peli.uusi_tulos);
        assert_eq!(peli.taulu.parhaat[0].nimi, "K");
        assert_eq!(peli.taulu.lahetys.as_ref().unwrap().pisteet, 40);
    }

    #[test]
    fn listalle_yltamaton_tulos_ei_kysy_nimea_ja_uusi_peli_alkaa_alusta() {
        let mut peli = paattyva(40);
        peli.taulu.netti = Some(vec![
            Tulos {
                nimi: "A".to_string(),
                pisteet: 1000,
            };
            3
        ]);
        for _ in 0..20 {
            aja(&mut peli, 100);
        }
        assert_eq!(peli.vaihe, Vaihe::Tappio);
        assert_eq!(peli.taulu.lahetys, None);
        peli.kasittele(Syote::Kirjain('K'));
        assert_eq!(peli.vaihe, Vaihe::Pelissa);
        assert_eq!(peli.kaivos.pisteet, 0);
        assert_eq!(peli.kaivos.kyparat, KYPARIA);
    }

    #[test]
    fn esc_kesken_pelin_pysayttaa_ja_toinen_kysyy_nimen() {
        let mut peli = paattyva(500);
        peli.kasittele(Syote::Peru);
        assert_eq!(peli.vaihe, Vaihe::Tauko);
        peli.kasittele(Syote::Peru);
        assert_eq!(peli.vaihe, Vaihe::Nimi);
        assert!(!peli.lopeta);
        // Kivi kaivajan yllä ei saa pudota nimen kirjoituksen aikana.
        aja(&mut peli, 3000);
        assert_eq!(peli.kaivos.kyparat, 1);
        peli.kasittele(Syote::Kirjain('B'));
        peli.kasittele(Syote::Ammu);
        assert!(peli.lopeta);
        assert_eq!(peli.taulu.parhaat[0].nimi, "B");
    }

    #[test]
    fn tauko_pysayttaa_ja_ikkunan_vaihto_pysayttaa() {
        let mut peli = pelissa();
        peli.kasittele(Syote::Kirjain('P'));
        assert_eq!(peli.vaihe, Vaihe::Tauko);
        peli.kasittele(Syote::Nuoli(1, 0));
        aja(&mut peli, 400);
        assert_eq!(peli.kaivos.kaivaja.paikka, ALKU);
        peli.kasittele(Syote::Kirjain('P'));
        assert_eq!(peli.vaihe, Vaihe::Pelissa);
        peli.keskeyta();
        assert_eq!(peli.vaihe, Vaihe::Tauko);
    }

    #[test]
    fn lopun_napit_toimivat() {
        let mut peli = Kaivo::new(1);
        peli.vaihe = Vaihe::Tappio;
        let (x, y) = piirto::napin_keskipiste(Nappi::Uudestaan);
        peli.kasittele(Syote::Klikkaus(x, y));
        assert_eq!(peli.vaihe, Vaihe::Pelissa);
        peli.vaihe = Vaihe::Tappio;
        let (x, y) = piirto::napin_keskipiste(Nappi::Valikko);
        peli.kasittele(Syote::Klikkaus(x, y));
        assert!(peli.lopeta);
    }
}
