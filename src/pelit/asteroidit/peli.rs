use klassikot8000::Arpa;
use klassikot8000::asteroidit::{ASKEL_MS, Avaruus, Ohjaus, Piste, Tapahtuma, Tila};

use super::piirto;
use crate::yhteinen::aanet::{Aani, PUHE_VIIVE_MS};
use crate::yhteinen::kieli::{kaanna, muotoile};
use crate::yhteinen::pelinakyma::{Lista, Pelinakyma};
use crate::yhteinen::piirto::{Kehys, aanikytkimella};
use crate::yhteinen::syote::Syote;
use crate::yhteinen::tulokset::{
    Nettipeli, Tulos, lisaa_tulos, mahtuu_parhaisiin, muokkaa_nimea, ota_nimi,
};

pub const RAJAHDYS_MS: u32 = 450;
// Ikkunan raahaus voi pysäyttää silmukan: kivet eivät saa sen jälkeen hypätä kerralla pitkälle.
const SUURIN_KURO_MS: u32 = 100;
const LOPPU_VIIVE_MS: u32 = 250;
// Lyhyet äänet soivat uudelleen näin usein, joten ne kuulostavat jatkuvilta.
const MOOTTORIN_VALI_MS: u32 = 120;
const LAUTASEN_VALI_MS: u32 = 300;

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

/// Räjähdystehoste: paikka kentällä, koko ja alkuhetki.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rajahdys {
    pub paikka: Piste,
    pub koko: f32,
    pub alku: u32,
}

pub struct Asteroidit {
    pub vaihe: Vaihe,
    pub lauta: Avaruus,
    pub paras: u32,
    pub parhaat: Vec<Tulos>,
    /// Nettilista. `None`, kunnes palvelin on vastannut.
    pub netti_parhaat: Option<Vec<Tulos>>,
    /// Kirjattu tulos, joka odottaa lähetystä nettilistalle.
    pub lahetys: Option<Tulos>,
    /// Kirjoitettavana oleva nimimerkki.
    pub nimi: String,
    /// Tosi, kun päättyneen pelin ennätys odottaa tiedostoon kirjoitusta.
    pub uusi_tulos: bool,
    pub viesti: String,
    pub rajahdykset: Vec<Rajahdys>,
    /// Pyyntö palata käynnistimeen.
    pub lopeta: bool,
    pub aanet_paalla: bool,
    pub aanijono: Vec<(Aani, u32)>,
    pub kello_ms: u32,
    pub ohjaus: Ohjaus,
    /// Pelaaja poistuu kesken pelin: peli seisoo, ja nimen jälkeen palataan valikkoon.
    poistuu: bool,
    kertyma_ms: u32,
    moottori_ms: u32,
    lautanen_ms: u32,
    arpa: Arpa,
}

impl Asteroidit {
    pub fn new(siemen: u64) -> Self {
        let mut arpa = Arpa::new(siemen);
        Self {
            vaihe: Vaihe::Alku,
            lauta: Avaruus::new(0, &mut arpa),
            paras: 0,
            parhaat: Vec::new(),
            netti_parhaat: None,
            lahetys: None,
            nimi: String::new(),
            uusi_tulos: false,
            viesti: String::new(),
            rajahdykset: Vec::new(),
            lopeta: false,
            aanet_paalla: true,
            aanijono: Vec::new(),
            kello_ms: 0,
            ohjaus: Ohjaus::default(),
            poistuu: false,
            kertyma_ms: 0,
            moottori_ms: 0,
            lautanen_ms: 0,
            arpa,
        }
    }

    /// Esillä oleva lista: nettilista, tai oma lista, jos verkkoon ei ole saatu yhteyttä.
    pub fn nakyva_lista(&self) -> &[Tulos] {
        self.netti_parhaat.as_deref().unwrap_or(&self.parhaat)
    }

    /// Pohjassa olevat nuolet. Nuolella kääntäminen ohittaa hiiren tähtäimen.
    pub fn aseta_nuolet(&mut self, kaanto: i8, tyonto: bool) {
        self.ohjaus.kaanto = kaanto;
        self.ohjaus.tyonto = tyonto;
        if kaanto != 0 {
            self.ohjaus.tahtain = None;
        }
    }

    pub fn edista(&mut self, ms: u32) {
        self.kello_ms = self.kello_ms.saturating_add(ms);
        let kello = self.kello_ms;
        self.rajahdykset
            .retain(|r| kello < r.alku.saturating_add(RAJAHDYS_MS));
        // Kivet ajelehtivat vielä pelin päätyttyä.
        if self.poistuu || matches!(self.vaihe, Vaihe::Alku | Vaihe::Tauko) {
            return;
        }
        self.jatkuvat_aanet(ms);
        self.kertyma_ms = (self.kertyma_ms + ms).min(SUURIN_KURO_MS);
        while self.kertyma_ms >= ASKEL_MS {
            self.kertyma_ms -= ASKEL_MS;
            for tapahtuma in self.lauta.askel(self.ohjaus, &mut self.arpa) {
                self.kasittele_tapahtuma(tapahtuma);
            }
        }
    }

    fn jatkuvat_aanet(&mut self, ms: u32) {
        self.moottori_ms = self.moottori_ms.saturating_sub(ms);
        self.lautanen_ms = self.lautanen_ms.saturating_sub(ms);
        if self.lauta.tyontaa && self.moottori_ms == 0 {
            self.moottori_ms = MOOTTORIN_VALI_MS;
            self.aanijono.push((Aani::Moottori, 0));
        }
        if self.lauta.lautanen.is_some() && self.lautanen_ms == 0 {
            self.lautanen_ms = LAUTASEN_VALI_MS;
            self.aanijono.push((Aani::Lautanen, 0));
        }
    }

    fn rajahda(&mut self, paikka: Piste, koko: f32) {
        self.rajahdykset.push(Rajahdys {
            paikka,
            koko,
            alku: self.kello_ms,
        });
    }

    fn kasittele_tapahtuma(&mut self, tapahtuma: Tapahtuma) {
        self.paras = self.paras.max(self.lauta.pisteet);
        match tapahtuma {
            Tapahtuma::Kivi { paikka, koko } => {
                self.aanijono.push((Aani::Kivi, 0));
                self.rajahda(paikka, koko.sade() * 1.5);
            }
            Tapahtuma::LautanenTuhoutui { paikka } => {
                self.aanijono.push((Aani::Rajahdys, 0));
                self.rajahda(paikka, 45.0);
            }
            Tapahtuma::LautanenAmpui => self.aanijono.push((Aani::Ammus, 0)),
            Tapahtuma::Menetys { paikka } => {
                self.aanijono.push((Aani::Rajahdys, 0));
                self.rajahda(paikka, 70.0);
                self.viesti = muotoile("asteroidit.alus_tuhoutui", &[&self.lauta.elamat]);
            }
            Tapahtuma::Palasi => self.viesti.clear(),
            Tapahtuma::Lisaelama => {
                self.aanijono.push((Aani::Yhdistys, 0));
                self.viesti = kaanna("asteroidit.lisaalus").to_string();
            }
            Tapahtuma::UusiAalto => {
                self.aanijono.push((Aani::Alku, 0));
                self.viesti = muotoile("asteroidit.uusi_aalto", &[&(self.lauta.aalto + 1)]);
            }
            Tapahtuma::Ohi { paikka } => {
                self.aanijono.push((Aani::Rajahdys, 0));
                self.aanijono.push((Aani::Tappio, LOPPU_VIIVE_MS));
                self.aanijono
                    .push((Aani::PeliOhi, LOPPU_VIIVE_MS + PUHE_VIIVE_MS));
                self.rajahda(paikka, 70.0);
                self.lopeta_peli();
            }
        }
    }

    fn lopeta_peli(&mut self) {
        self.vaihe = if mahtuu_parhaisiin(self.nakyva_lista(), self.lauta.pisteet) {
            self.nimi.clear();
            Vaihe::Nimi
        } else {
            Vaihe::Tappio
        };
        self.uusi_tulos = true;
        self.viesti = muotoile("yhteinen.tulos", &[&self.lauta.pisteet]);
    }

    /// Kirjaa odottavan tuloksen.
    pub fn vahvista_nimi(&mut self) {
        if self.vaihe != Vaihe::Nimi {
            return;
        }
        let nimi = ota_nimi(&mut self.nimi);
        let pisteet = self.lauta.pisteet;
        let tulos = Tulos { nimi, pisteet };
        lisaa_tulos(&mut self.parhaat, tulos.clone());
        if let Some(lista) = &mut self.netti_parhaat {
            lisaa_tulos(lista, tulos.clone());
        }
        self.lahetys = Some(tulos);
        self.uusi_tulos = true;
        self.vaihe = Vaihe::Tappio;
        self.lopeta = self.poistuu;
    }

    /// Esc tai ikkunan sulkeminen kesken pelin: listalle yltävä tulos ei saa kadota.
    pub fn poistu(&mut self) {
        let kesken = matches!(self.vaihe, Vaihe::Pelissa | Vaihe::Tauko);
        if kesken && mahtuu_parhaisiin(self.nakyva_lista(), self.lauta.pisteet) {
            self.nimi.clear();
            self.poistuu = true;
            self.vaihe = Vaihe::Nimi;
        } else {
            self.lopeta = true;
        }
    }

    fn kasittele_nimi(&mut self, syote: Syote) {
        match syote {
            // Ampumiseen painettu välilyönti ei saa kuitata nimeä ennen kuin sitä on kirjoitettu.
            Syote::Ammu if self.nimi.is_empty() => {}
            Syote::Ammu | Syote::Peru => self.vahvista_nimi(),
            _ => muokkaa_nimea(&mut self.nimi, syote),
        }
    }

    fn aloita_alusta(&mut self) {
        self.lauta = Avaruus::new(0, &mut self.arpa);
        self.vaihe = Vaihe::Pelissa;
        self.viesti.clear();
        self.rajahdykset.clear();
        self.kertyma_ms = 0;
        self.aanijono.push((Aani::Alku, 0));
    }

    /// Ikkuna menetti kohdistuksen: alus ei saa tuhoutua pelaajan poissa ollessa.
    pub fn keskeyta(&mut self) {
        if self.vaihe == Vaihe::Pelissa {
            self.vaihe = Vaihe::Tauko;
        }
    }

    pub fn kasittele(&mut self, syote: Syote) {
        // Klikkaus jatkaa eteenpäin, jotta äänikytkin toimii nimeä kirjoittaessakin.
        if self.vaihe == Vaihe::Nimi && !matches!(syote, Syote::Klikkaus(..)) {
            self.kasittele_nimi(syote);
            return;
        }
        if syote == Syote::Peru {
            // Kesken pelin Esc pysäyttää ensin, jottei vahinkopainallus hävitä peliä.
            if self.vaihe == Vaihe::Pelissa {
                self.vaihe = Vaihe::Tauko;
            } else {
                self.poistu();
            }
            return;
        }
        let kytkimella = matches!(syote, Syote::Klikkaus(x, y) if aanikytkimella(x, y));
        if syote == Syote::VaihdaAanet || kytkimella {
            self.aanet_paalla = !self.aanet_paalla;
            return;
        }
        match self.vaihe {
            Vaihe::Alku => {
                if matches!(syote, Syote::Ammu | Syote::Klikkaus(..)) {
                    self.aloita_alusta();
                }
            }
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

    fn kasittele_pelissa(&mut self, syote: Syote) {
        match syote {
            Syote::Hiiri(x, y) => {
                let paikka = ((x - piirto::KENTTA_X) as f32, (y - piirto::KENTTA_Y) as f32);
                self.ohjaus.tahtain = Some(paikka);
            }
            Syote::Ammu | Syote::Klikkaus(..) => {
                let lahti = self.lauta.ammu();
                if lahti {
                    self.aanijono.push((Aani::Ammus, 0));
                }
            }
            Syote::Kirjain('H') => {
                let hyppasi = self.lauta.hyppaa(&mut self.arpa);
                if hyppasi {
                    self.aanijono.push((Aani::Liuku, 0));
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

    /// Tuhoutunutta alusta ei piirretä.
    pub fn alus_nakyy(&self) -> bool {
        self.lauta.tila == Tila::Lennossa
    }
}

impl Pelinakyma for Asteroidit {
    fn kasittele(&mut self, syote: Syote) {
        Asteroidit::kasittele(self, syote);
    }

    fn edista(&mut self, ms: u32) {
        Asteroidit::edista(self, ms);
    }

    fn piirra(&self, kehys: &mut Kehys) {
        piirto::piirra(self, &mut kehys.kuva);
        kehys.venyta();
    }

    fn aanijono(&mut self) -> &mut Vec<(Aani, u32)> {
        &mut self.aanijono
    }

    fn aanet_paalla(&self) -> bool {
        self.aanet_paalla
    }

    fn aseta_aanet(&mut self, paalla: bool) {
        self.aanet_paalla = paalla;
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

    fn listat(&mut self) -> Vec<Lista<'_>> {
        vec![Lista {
            peli: Nettipeli::Asteroidit,
            paras: Some(&mut self.paras),
            parhaat: &mut self.parhaat,
            netti: &mut self.netti_parhaat,
            lahetys: Some(&mut self.lahetys),
        }]
    }

    fn toimintapeli(&self) -> bool {
        self.vaihe != Vaihe::Alku
    }

    fn aseta_ohjaus(&mut self, suunta: i8, ylos: bool) {
        self.aseta_nuolet(suunta, ylos);
    }

    fn keskeyta(&mut self) {
        Self::keskeyta(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use klassikot8000::asteroidit::{Kivi, Koko};

    fn pelissa() -> Asteroidit {
        let mut peli = Asteroidit::new(1);
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.vaihe, Vaihe::Pelissa);
        peli.aanijono.clear();
        peli
    }

    /// Paikallaan oleva kivi aluksen kohdalla: viimeinen alus tuhoutuu seuraavalla askeleella.
    fn paattyva(pisteet: u32) -> Asteroidit {
        let mut peli = pelissa();
        peli.lauta.pisteet = pisteet;
        peli.lauta.elamat = 1;
        peli.lauta.kivet = vec![Kivi {
            paikka: peli.lauta.alus.paikka,
            nopeus: (0.0, 0.0),
            koko: Koko::Pieni,
            kulma: 0.0,
            pyorinta: 0.0,
            muoto: 0,
        }];
        peli.aanijono.clear();
        peli
    }

    #[test]
    fn valilyonti_aloittaa_ja_toinen_ampuu() {
        let mut peli = Asteroidit::new(1);
        peli.kasittele(Syote::Nuoli(0, 1));
        assert_eq!(peli.vaihe, Vaihe::Alku);
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.vaihe, Vaihe::Pelissa);
        assert_eq!(peli.aanijono, [(Aani::Alku, 0)]);
        peli.aanijono.clear();
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.lauta.ammukset.len(), 1);
        assert_eq!(peli.aanijono, [(Aani::Ammus, 0)]);
    }

    #[test]
    fn klikkaus_ampuu_ja_hiiri_kaantaa_alusta_kentan_koordinaateissa() {
        let mut peli = pelissa();
        let (x, y) = peli.lauta.alus.paikka;
        peli.kasittele(Syote::Hiiri(
            piirto::KENTTA_X + x as i32 + 100,
            piirto::KENTTA_Y + y as i32,
        ));
        assert_eq!(peli.ohjaus.tahtain, Some((x + 100.0, y)));
        peli.lauta.kivet.clear();
        for _ in 0..10 {
            peli.edista(50);
        }
        assert!(peli.lauta.alus.kulma.sin().abs() < 0.01);
        peli.kasittele(Syote::Klikkaus(300, 300));
        assert_eq!(peli.lauta.ammukset.len(), 1);

        peli.aseta_nuolet(1, false);
        assert_eq!(peli.ohjaus.tahtain, None);
    }

    #[test]
    fn tauko_pysayttaa_kivet_ja_jatkuu_samasta() {
        let mut peli = pelissa();
        peli.kasittele(Syote::Kirjain('P'));
        let paikka = peli.lauta.kivet[0].paikka;
        peli.edista(100);
        assert_eq!(peli.lauta.kivet[0].paikka, paikka);
        peli.kasittele(Syote::Kirjain('P'));
        peli.edista(100);
        assert_ne!(peli.lauta.kivet[0].paikka, paikka);

        peli.keskeyta();
        assert_eq!(peli.vaihe, Vaihe::Tauko);
    }

    #[test]
    fn iso_aikahyppy_ei_siirra_kivia_kerralla_pitkalle() {
        let mut peli = pelissa();
        peli.lauta.kivet.truncate(1);
        peli.lauta.kivet[0].nopeus = (100.0, 0.0);
        peli.lauta.kivet[0].paikka = (0.0, 0.0);
        peli.edista(60_000);
        assert!(peli.lauta.kivet[0].paikka.0 < 100.0);
    }

    #[test]
    fn moottori_ja_lautanen_soivat_uudelleen_valein() {
        let mut peli = pelissa();
        peli.lauta.kivet.clear();
        peli.aseta_nuolet(0, true);
        peli.edista(16);
        peli.edista(16);
        assert_eq!(peli.aanijono, [(Aani::Moottori, 0)]);
        peli.edista(MOOTTORIN_VALI_MS);
        assert_eq!(peli.aanijono.len(), 2);
    }

    #[test]
    fn viimeisen_aluksen_tuho_kysyy_nimen_ja_palaa_loppuruutuun() {
        let mut peli = paattyva(40);
        peli.edista(ASKEL_MS);
        assert_eq!(peli.vaihe, Vaihe::Nimi);
        assert_eq!(peli.lauta.pisteet, 140);
        assert_eq!(peli.paras, 140);
        assert!(peli.uusi_tulos);
        assert_eq!(peli.rajahdykset.len(), 2);

        // Ampumiseen painettu välilyönti ei kuittaa tyhjää nimeä, ja K on nimessä kirjain.
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.vaihe, Vaihe::Nimi);
        peli.kasittele(Syote::Kirjain('K'));
        peli.kasittele(Syote::VaihdaAanet);
        assert!(peli.aanet_paalla);
        peli.uusi_tulos = false;
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.vaihe, Vaihe::Tappio);
        assert!(peli.uusi_tulos);
        assert_eq!(peli.parhaat[0].nimi, "KM");
        assert_eq!(peli.lahetys.as_ref().unwrap().pisteet, 140);
    }

    #[test]
    fn listalle_yltamaton_tulos_ei_kysy_nimea_ja_uusi_peli_alkaa_alusta() {
        let taysi = (0..3)
            .map(|_| Tulos {
                nimi: "A".to_string(),
                pisteet: 1000,
            })
            .collect::<Vec<_>>();
        let mut peli = paattyva(40);
        peli.netti_parhaat = Some(taysi);
        peli.edista(ASKEL_MS);
        assert_eq!(peli.vaihe, Vaihe::Tappio);
        assert_eq!(peli.lahetys, None);

        peli.kasittele(Syote::Kirjain('K'));
        assert_eq!(peli.vaihe, Vaihe::Pelissa);
        assert_eq!(peli.lauta.pisteet, 0);
        assert_eq!(peli.paras, 140);
    }

    #[test]
    fn kivet_ajelehtivat_pelin_paatyttya_mutta_tulos_ei_muutu() {
        let mut peli = paattyva(40);
        peli.edista(ASKEL_MS);
        peli.vahvista_nimi();
        // Osunut kivi hajosi, joten kentälle tuodaan uusi.
        peli.lauta.kivet = vec![Kivi {
            paikka: (100.0, 100.0),
            nopeus: (100.0, 0.0),
            koko: Koko::Iso,
            kulma: 0.0,
            pyorinta: 0.0,
            muoto: 0,
        }];
        let paikka = peli.lauta.kivet[0].paikka;
        let pisteet = peli.lauta.pisteet;
        peli.edista(50);
        assert_ne!(peli.lauta.kivet[0].paikka, paikka);
        assert_eq!(peli.lauta.pisteet, pisteet);
    }

    #[test]
    fn esc_kirjaa_nimen_oletuksella_ja_vahvistus_ei_toistu() {
        let mut peli = paattyva(40);
        peli.edista(ASKEL_MS);
        peli.kasittele(Syote::Peru);
        assert!(!peli.lopeta);
        assert_eq!(peli.vaihe, Vaihe::Tappio);
        assert_eq!(peli.parhaat[0].nimi, "NIMETÖN");
        peli.vahvista_nimi();
        assert_eq!(peli.parhaat.len(), 1);
    }

    #[test]
    fn esc_kesken_pelin_pysayttaa_ja_toinen_kysyy_nimen() {
        let mut peli = paattyva(500);
        peli.kasittele(Syote::Peru);
        assert_eq!(peli.vaihe, Vaihe::Tauko);
        assert!(!peli.lopeta);
        peli.kasittele(Syote::Peru);
        assert_eq!(peli.vaihe, Vaihe::Nimi);
        assert!(!peli.lopeta);
        // Kivi aluksen päällä ei saa päättää peliä nimen kirjoituksen aikana.
        peli.edista(1000);
        assert_eq!(peli.lauta.pisteet, 500);
        assert_eq!(peli.lauta.elamat, 1);

        peli.kasittele(Syote::Kirjain('B'));
        peli.kasittele(Syote::Ammu);
        assert!(peli.lopeta);
        assert_eq!(peli.parhaat[0].nimi, "B");
        assert_eq!(peli.lahetys.as_ref().unwrap().pisteet, 500);
    }

    #[test]
    fn listalle_yltamaton_tulos_poistuu_heti_ja_sulkeminen_kirjaa_oletusnimella() {
        let mut peli = paattyva(500);
        peli.netti_parhaat = Some(vec![
            Tulos {
                nimi: "A".to_string(),
                pisteet: 1000,
            };
            3
        ]);
        peli.kasittele(Syote::Peru);
        assert!(!peli.lopeta);
        peli.kasittele(Syote::Peru);
        assert!(peli.lopeta);
        assert_eq!(peli.lahetys, None);

        let mut peli = paattyva(500);
        peli.poistu();
        peli.vahvista_nimi();
        assert_eq!(peli.parhaat[0].nimi, "NIMETÖN");
    }

    #[test]
    fn lopun_napit_ja_esc_toimivat() {
        let mut peli = Asteroidit::new(1);
        peli.vaihe = Vaihe::Tappio;
        let (x, y) = piirto::napin_keskipiste(Nappi::Uudestaan);
        peli.kasittele(Syote::Klikkaus(x, y));
        assert_eq!(peli.vaihe, Vaihe::Pelissa);

        peli.vaihe = Vaihe::Tappio;
        let (x, y) = piirto::napin_keskipiste(Nappi::Valikko);
        peli.kasittele(Syote::Klikkaus(x, y));
        assert!(peli.lopeta);

        // Esc pysäyttää, välilyönti jatkaa ja tauon aikana Esc palaa valikkoon.
        let mut peli = pelissa();
        peli.kasittele(Syote::Peru);
        assert_eq!(peli.vaihe, Vaihe::Tauko);
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.vaihe, Vaihe::Pelissa);
        peli.kasittele(Syote::Peru);
        assert!(!peli.lopeta);
        peli.kasittele(Syote::Peru);
        assert!(peli.lopeta);
    }
}
