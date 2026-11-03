use klassikot8000::Arpa;
use klassikot8000::tiilirikko::{ASKEL_MS, Tapahtuma, Tiilipeli, Tila};

use super::piirto;
use crate::yhteinen::aanet::Aani;
use crate::yhteinen::kieli::{kaanna, muotoile};
use crate::yhteinen::pelinakyma::{Lista, Pelinakyma};
use crate::yhteinen::piirto::{Kehys, aanikytkimella};
use crate::yhteinen::syote::Syote;
use crate::yhteinen::tulokset::{
    Nettipeli, Tulos, lisaa_tulos, mahtuu_parhaisiin, muokkaa_nimea, ota_nimi,
};

pub const KIPINA_MS: u32 = 350;
// Ikkunan raahaus voi pysäyttää silmukan: pallo ei saa sen jälkeen hypätä kerralla pitkälle.
const SUURIN_KURO_MS: u32 = 100;
const LOPPU_VIIVE_MS: u32 = 250;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vaihe {
    Alku,
    Pelissa,
    Tauko,
    Voitto,
    Tappio,
    /// Tulos pääsi kolmen parhaan joukkoon ja odottaa nimimerkkiä.
    Nimi,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nappi {
    Uudestaan,
    Valikko,
}

pub struct Tiilirikko {
    pub vaihe: Vaihe,
    pub lauta: Tiilipeli,
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
    /// Rikkoutuneet tiilet (rivi, sarake, alkuhetki) tehostetta varten.
    pub kipinat: Vec<(usize, usize, u32)>,
    /// Pyyntö palata käynnistimeen.
    pub lopeta: bool,
    pub aanet_paalla: bool,
    pub aanijono: Vec<(Aani, u32)>,
    pub kello_ms: u32,
    /// Pohjassa olevien nuolten suunta: -1, 0 tai 1.
    pub ohjaus: i8,
    /// Pelaaja poistuu kesken pelin: nimen jälkeen palataan valikkoon.
    poistuu: bool,
    kertyma_ms: u32,
    /// Vaihe, johon nimimerkin jälkeen palataan: voitto tai tappio.
    loppu: Vaihe,
    arpa: Arpa,
}

impl Tiilirikko {
    pub fn new(siemen: u64) -> Self {
        let mut arpa = Arpa::new(siemen);
        Self {
            vaihe: Vaihe::Alku,
            lauta: Tiilipeli::new(0, &mut arpa),
            paras: 0,
            parhaat: Vec::new(),
            netti_parhaat: None,
            lahetys: None,
            nimi: String::new(),
            uusi_tulos: false,
            viesti: String::new(),
            kipinat: Vec::new(),
            lopeta: false,
            aanet_paalla: true,
            aanijono: Vec::new(),
            kello_ms: 0,
            ohjaus: 0,
            poistuu: false,
            kertyma_ms: 0,
            loppu: Vaihe::Tappio,
            arpa,
        }
    }

    /// Esillä oleva lista: nettilista, tai oma lista, jos verkkoon ei ole saatu yhteyttä.
    pub fn nakyva_lista(&self) -> &[Tulos] {
        self.netti_parhaat.as_deref().unwrap_or(&self.parhaat)
    }

    pub fn edista(&mut self, ms: u32) {
        self.kello_ms = self.kello_ms.saturating_add(ms);
        let kello = self.kello_ms;
        self.kipinat
            .retain(|&(_, _, alku)| kello < alku.saturating_add(KIPINA_MS));
        if self.vaihe != Vaihe::Pelissa {
            return;
        }
        self.kertyma_ms = (self.kertyma_ms + ms).min(SUURIN_KURO_MS);
        while self.kertyma_ms >= ASKEL_MS && self.vaihe == Vaihe::Pelissa {
            self.kertyma_ms -= ASKEL_MS;
            for tapahtuma in self.lauta.askel(self.ohjaus, &mut self.arpa) {
                self.kasittele_tapahtuma(tapahtuma);
            }
        }
    }

    fn kasittele_tapahtuma(&mut self, tapahtuma: Tapahtuma) {
        self.paras = self.paras.max(self.lauta.pisteet);
        match tapahtuma {
            Tapahtuma::Seina | Tapahtuma::Maila => self.aanijono.push((Aani::Pomppu, 0)),
            Tapahtuma::Tiili {
                rivi,
                sarake,
                rikki,
            } => {
                self.aanijono.push((Aani::Tiili, 0));
                if rikki {
                    self.kipinat.push((rivi, sarake, self.kello_ms));
                }
            }
            Tapahtuma::UusiPallo => {
                self.aanijono.push((Aani::Yhdistys, 0));
                self.viesti = kaanna("tiilirikko.uusi_pallo").to_string();
            }
            Tapahtuma::Superpallo => {
                self.aanijono.push((Aani::Yhdistys, 0));
                self.viesti = kaanna("tiilirikko.superpallo").to_string();
            }
            Tapahtuma::ToinenMeni => {
                self.aanijono.push((Aani::Puhkus, 0));
                self.viesti = kaanna("tiilirikko.toinen_meni").to_string();
            }
            Tapahtuma::Menetys => {
                self.aanijono.push((Aani::Rajahdys, 0));
                self.viesti = muotoile("tiilirikko.pallo_meni", &[&self.lauta.elamat]);
            }
            Tapahtuma::KenttaLapi { bonus } => {
                self.aanijono.push((Aani::Yhdistys, 0));
                self.viesti = muotoile("tiilirikko.kentta_lapi", &[&bonus]);
            }
            Tapahtuma::Voitto { bonus } => {
                self.aanijono.push((Aani::Voitto, LOPPU_VIIVE_MS));
                self.lopeta_peli(Vaihe::Voitto);
                self.viesti = muotoile("tiilirikko.loppubonus", &[&bonus, &self.lauta.pisteet]);
            }
            Tapahtuma::Ohi => {
                self.aanijono.push((Aani::Tappio, LOPPU_VIIVE_MS));
                self.lopeta_peli(Vaihe::Tappio);
            }
        }
    }

    fn lopeta_peli(&mut self, vaihe: Vaihe) {
        self.loppu = vaihe;
        self.vaihe = if mahtuu_parhaisiin(self.nakyva_lista(), self.lauta.pisteet) {
            self.nimi.clear();
            Vaihe::Nimi
        } else {
            vaihe
        };
        self.ohjaus = 0;
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
        self.vaihe = self.loppu;
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
            // Laukaisuun painettu välilyönti ei saa kuitata nimeä ennen kuin sitä on kirjoitettu.
            Syote::Ammu if self.nimi.is_empty() => {}
            Syote::Ammu | Syote::Peru => self.vahvista_nimi(),
            _ => muokkaa_nimea(&mut self.nimi, syote),
        }
    }

    fn aloita_alusta(&mut self) {
        self.lauta = Tiilipeli::new(0, &mut self.arpa);
        self.vaihe = Vaihe::Pelissa;
        self.viesti.clear();
        self.kipinat.clear();
        self.kertyma_ms = 0;
        self.aanijono.push((Aani::Ovi, 0));
    }

    /// Ikkuna menetti kohdistuksen: lennossa oleva pallo ei saa kadota pelaajan poissa ollessa.
    pub fn keskeyta(&mut self) {
        if self.vaihe == Vaihe::Pelissa && self.lauta.tila == Tila::Lennossa {
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
            self.poistu();
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
            Vaihe::Pelissa => match syote {
                Syote::Hiiri(x, _) => self.lauta.aseta_maila((x - piirto::KENTTA_X) as f32),
                Syote::Ammu | Syote::Klikkaus(..) if self.lauta.tila == Tila::Kiinni => {
                    self.viesti.clear();
                    self.lauta.laukaise(&mut self.arpa);
                }
                Syote::Kirjain('P') => self.vaihe = Vaihe::Tauko,
                _ => {}
            },
            Vaihe::Tauko => {
                if matches!(
                    syote,
                    Syote::Ammu | Syote::Klikkaus(..) | Syote::Kirjain('P')
                ) {
                    self.vaihe = Vaihe::Pelissa;
                }
            }
            Vaihe::Voitto | Vaihe::Tappio => self.kasittele_loppu(syote),
            Vaihe::Nimi => {}
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

impl Pelinakyma for Tiilirikko {
    fn kasittele(&mut self, syote: Syote) {
        Tiilirikko::kasittele(self, syote);
    }

    fn edista(&mut self, ms: u32) {
        Tiilirikko::edista(self, ms);
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
            peli: Nettipeli::Tiilirikko,
            paras: Some(&mut self.paras),
            parhaat: &mut self.parhaat,
            netti: &mut self.netti_parhaat,
            lahetys: Some(&mut self.lahetys),
        }]
    }

    fn toimintapeli(&self) -> bool {
        self.vaihe != Vaihe::Alku
    }

    fn aseta_ohjaus(&mut self, suunta: i8, _ylos: bool) {
        self.ohjaus = suunta;
    }

    fn keskeyta(&mut self) {
        Self::keskeyta(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use klassikot8000::tiilirikko::{KENTTIA, KORKEUS, LEVEYS, MAILA_L, RIVEJA, SARAKKEITA};

    fn lennossa() -> Tiilirikko {
        let mut peli = Tiilirikko::new(1);
        peli.kasittele(Syote::Ammu);
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.lauta.tila, Tila::Lennossa);
        peli
    }

    /// Viimeinen pallo putoaa seuraavalla askeleella.
    fn paattyva(pisteet: u32) -> Tiilirikko {
        let mut peli = lennossa();
        peli.lauta.pisteet = pisteet;
        peli.paras = pisteet;
        peli.lauta.elamat = 1;
        peli.lauta.pallot[0].paikka = (LEVEYS / 2.0, KORKEUS + 8.0);
        peli.lauta.pallot[0].nopeus = (0.0, 300.0);
        peli.aanijono.clear();
        peli
    }

    #[test]
    fn valilyonti_aloittaa_ja_toinen_laukaisee() {
        let mut peli = Tiilirikko::new(1);
        peli.kasittele(Syote::Nuoli(0, 1));
        assert_eq!(peli.vaihe, Vaihe::Alku);
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.vaihe, Vaihe::Pelissa);
        assert_eq!(peli.lauta.tila, Tila::Kiinni);
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.lauta.tila, Tila::Lennossa);
    }

    #[test]
    fn hiiri_ja_pohjassa_oleva_nuoli_liikuttavat_mailaa() {
        let mut peli = Tiilirikko::new(1);
        peli.kasittele(Syote::Ammu);
        peli.kasittele(Syote::Hiiri(piirto::KENTTA_X + 200, 300));
        assert_eq!(peli.lauta.maila_x, 200.0);
        peli.kasittele(Syote::Hiiri(-500, 300));
        assert_eq!(peli.lauta.maila_x, MAILA_L / 2.0);

        peli.ohjaus = 1;
        peli.edista(50);
        assert!(peli.lauta.maila_x > MAILA_L / 2.0);
        assert_eq!(peli.lauta.pallot[0].paikka.0, peli.lauta.maila_x);
    }

    #[test]
    fn tauko_pysayttaa_pallon_ja_jatkuu_samasta() {
        let mut peli = lennossa();
        peli.kasittele(Syote::Kirjain('P'));
        let paikka = peli.lauta.pallot[0].paikka;
        peli.edista(100);
        assert_eq!(peli.lauta.pallot[0].paikka, paikka);
        peli.kasittele(Syote::Kirjain('P'));
        peli.edista(100);
        assert_ne!(peli.lauta.pallot[0].paikka, paikka);

        peli.keskeyta();
        assert_eq!(peli.vaihe, Vaihe::Tauko);
    }

    #[test]
    fn kohdistuksen_menetys_ei_keskeyta_kiinni_olevaa_palloa() {
        let mut peli = Tiilirikko::new(1);
        peli.kasittele(Syote::Ammu);
        peli.keskeyta();
        assert_eq!(peli.vaihe, Vaihe::Pelissa);
    }

    #[test]
    fn iso_aikahyppy_ei_siirra_palloa_kerralla_pitkalle() {
        let mut peli = lennossa();
        let alku = peli.lauta.pallot[0].paikka;
        peli.edista(60_000);
        let matka = (peli.lauta.pallot[0].paikka.1 - alku.1).abs();
        assert!(matka < 100.0, "{matka}");
    }

    #[test]
    fn viimeisen_pallon_menetys_paattaa_pelin_ja_paras_sailyy_uudessa() {
        let mut peli = paattyva(40);
        peli.parhaat = (0..3)
            .map(|_| Tulos {
                nimi: "A".to_string(),
                pisteet: 40,
            })
            .collect();
        peli.edista(ASKEL_MS);
        assert_eq!(peli.vaihe, Vaihe::Tappio);
        assert!(peli.uusi_tulos);
        assert_eq!(peli.aanijono, [(Aani::Tappio, LOPPU_VIIVE_MS)]);

        peli.kasittele(Syote::Kirjain('K'));
        assert_eq!(peli.vaihe, Vaihe::Pelissa);
        assert_eq!(peli.lauta.pisteet, 0);
        assert_eq!(peli.paras, 40);
    }

    #[test]
    fn listalle_yltava_tulos_kysyy_nimen_ja_palaa_loppuruutuun() {
        let mut peli = paattyva(40);
        peli.edista(ASKEL_MS);
        assert_eq!(peli.vaihe, Vaihe::Nimi);

        // Laukaisuun painettu välilyönti ei kuittaa tyhjää nimeä, ja K on nimessä kirjain.
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
        assert_eq!(peli.parhaat[0].pisteet, 40);
    }

    #[test]
    fn esc_ja_sulkeminen_kirjaavat_tuloksen_oletusnimella() {
        let mut peli = paattyva(40);
        peli.edista(ASKEL_MS);
        peli.kasittele(Syote::Peru);
        assert!(!peli.lopeta);
        assert_eq!(peli.vaihe, Vaihe::Tappio);
        assert_eq!(peli.parhaat[0].nimi, "NIMETÖN");

        let mut peli = paattyva(40);
        peli.edista(ASKEL_MS);
        peli.vahvista_nimi();
        assert_eq!(peli.parhaat.len(), 1);
        peli.vahvista_nimi();
        assert_eq!(peli.parhaat.len(), 1);
    }

    #[test]
    fn nettilista_ratkaisee_nimen_kysymisen_kun_se_on_tiedossa() {
        let taysi = |pisteet| {
            (0..3)
                .map(|_| Tulos {
                    nimi: "A".to_string(),
                    pisteet,
                })
                .collect::<Vec<_>>()
        };

        let mut peli = paattyva(40);
        peli.netti_parhaat = Some(taysi(1000));
        peli.edista(ASKEL_MS);
        assert_eq!(peli.vaihe, Vaihe::Tappio);
        assert_eq!(peli.lahetys, None);

        let mut peli = paattyva(40);
        peli.parhaat = taysi(1000);
        peli.netti_parhaat = Some(taysi(10));
        peli.edista(ASKEL_MS);
        assert_eq!(peli.vaihe, Vaihe::Nimi);
        peli.kasittele(Syote::Kirjain('B'));
        peli.kasittele(Syote::Ammu);
        let lista = peli.netti_parhaat.as_ref().unwrap();
        assert_eq!(lista[0].nimi, "B");
        assert_eq!(peli.lahetys.as_ref().unwrap().pisteet, 40);
    }

    #[test]
    fn ilman_verkkoa_tulos_kirjataan_omalle_listalle_ja_lahetettavaksi() {
        let mut peli = paattyva(40);
        peli.edista(ASKEL_MS);
        peli.kasittele(Syote::Kirjain('B'));
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.nakyva_lista()[0].nimi, "B");
        assert_eq!(peli.netti_parhaat, None);
        assert_eq!(peli.lahetys.as_ref().unwrap().nimi, "B");
    }

    #[test]
    fn nollatulos_ei_paase_listalle() {
        let mut peli = paattyva(0);
        peli.edista(ASKEL_MS);
        assert_eq!(peli.vaihe, Vaihe::Tappio);
    }

    #[test]
    fn viimeisen_kentan_voitto_palaa_nimen_jalkeen_voittoon_ja_bonus_nostaa_parasta() {
        let mut peli = Tiilirikko::new(1);
        peli.kasittele(Syote::Ammu);
        peli.lauta = Tiilipeli::new(KENTTIA - 1, &mut Arpa::new(1));
        peli.kasittele(Syote::Ammu);
        peli.lauta.tiilet = [[0; SARAKKEITA]; RIVEJA];
        peli.lauta.tiilet[0][0] = 1;
        peli.lauta.pallot[0].paikka = (30.0, 80.0);
        peli.lauta.pallot[0].nopeus = (0.0, -300.0);
        peli.edista(ASKEL_MS);
        assert_eq!(peli.vaihe, Vaihe::Nimi);
        assert!(peli.lauta.pisteet > 1000);
        assert_eq!(peli.paras, peli.lauta.pisteet);
        assert!(peli.viesti.contains("Aikabonus"));

        peli.kasittele(Syote::Kirjain('A'));
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.vaihe, Vaihe::Voitto);
    }

    #[test]
    fn rikkoutuva_tiili_jattaa_kipinan_ja_soittaa_aanen() {
        let mut peli = lennossa();
        peli.lauta.tiilet = [[0; SARAKKEITA]; RIVEJA];
        peli.lauta.tiilet[0][0] = 1;
        peli.lauta.tiilet[7][10] = 1;
        peli.lauta.pallotiilet = [[false; SARAKKEITA]; RIVEJA];
        peli.lauta.supertiilet = [[false; SARAKKEITA]; RIVEJA];
        peli.lauta.pallot[0].paikka = (30.0, 80.0);
        peli.lauta.pallot[0].nopeus = (0.0, -300.0);
        peli.aanijono.clear();
        peli.edista(ASKEL_MS);
        assert_eq!(peli.kipinat.len(), 1);
        assert_eq!(peli.aanijono, [(Aani::Tiili, 0)]);
        assert_eq!(peli.paras, 10);
        peli.edista(KIPINA_MS);
        assert!(peli.kipinat.is_empty());
    }

    #[test]
    fn toinen_pallo_kertoo_tulostaan_ja_menetys_ei_vie_elamaa() {
        let mut peli = lennossa();
        peli.lauta.tiilet = [[0; SARAKKEITA]; RIVEJA];
        peli.lauta.tiilet[0][0] = 1;
        peli.lauta.tiilet[7][10] = 1;
        peli.lauta.supertiilet = [[false; SARAKKEITA]; RIVEJA];
        peli.lauta.pallotiilet[0][0] = true;
        peli.lauta.pallot[0].paikka = (30.0, 80.0);
        peli.lauta.pallot[0].nopeus = (0.0, -300.0);
        peli.aanijono.clear();
        peli.edista(ASKEL_MS);
        assert_eq!(peli.lauta.pallot.len(), 2);
        assert_eq!(peli.aanijono, [(Aani::Tiili, 0), (Aani::Yhdistys, 0)]);
        assert_eq!(peli.viesti, kaanna("tiilirikko.uusi_pallo"));

        peli.lauta.pallot[1].paikka = (LEVEYS / 2.0, KORKEUS + 8.0);
        peli.lauta.pallot[1].nopeus = (0.0, 300.0);
        peli.aanijono.clear();
        peli.edista(ASKEL_MS);
        assert_eq!(peli.lauta.pallot.len(), 1);
        assert_eq!(peli.lauta.elamat, klassikot8000::tiilirikko::ELAMIA);
        assert_eq!(peli.aanijono, [(Aani::Puhkus, 0)]);
        assert_eq!(peli.viesti, kaanna("tiilirikko.toinen_meni"));
    }

    #[test]
    fn esc_kesken_pelin_kysyy_nimen_ja_pysayttaa_pelin() {
        let mut peli = paattyva(500);
        peli.kasittele(Syote::Peru);
        assert_eq!(peli.vaihe, Vaihe::Nimi);
        assert!(!peli.lopeta);
        peli.edista(1000);
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
        assert!(peli.lopeta);
        assert_eq!(peli.lahetys, None);

        let mut peli = paattyva(500);
        peli.poistu();
        peli.vahvista_nimi();
        assert_eq!(peli.parhaat[0].nimi, "NIMETÖN");
    }

    #[test]
    fn lopun_napit_ja_esc_toimivat() {
        let mut peli = Tiilirikko::new(1);
        peli.vaihe = Vaihe::Voitto;
        let (x, y) = piirto::napin_keskipiste(Nappi::Uudestaan);
        peli.kasittele(Syote::Klikkaus(x, y));
        assert_eq!(peli.vaihe, Vaihe::Pelissa);

        peli.vaihe = Vaihe::Tappio;
        let (x, y) = piirto::napin_keskipiste(Nappi::Valikko);
        peli.kasittele(Syote::Klikkaus(x, y));
        assert!(peli.lopeta);

        let mut peli = lennossa();
        peli.kasittele(Syote::Peru);
        assert!(peli.lopeta);
    }
}
