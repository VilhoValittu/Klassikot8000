use klassikot8000::Arpa;
use klassikot8000::blackjack::{ALKURAHA, PIENIN_PANOS, Poyta, Tulos, Vuoro, YKSI};

use super::piirto::{self, raha};
use crate::yhteinen::aanet::Aani;
use crate::yhteinen::kieli::{kaanna, muotoile};
use crate::yhteinen::piirto::VILKKU_MS;
use crate::yhteinen::syote::Syote;
use crate::yhteinen::tulokset::{self, lisaa_tulos, mahtuu_parhaisiin, muokkaa_nimea, ota_nimi};

const ALKUPANOS: u32 = 10 * YKSI;
const LISAYKSET: [u32; 4] = [YKSI, 5 * YKSI, 25 * YKSI, 100 * YKSI];
const NUOLEN_ASKEL: i64 = YKSI as i64;
const NUOLEN_ISO_ASKEL: i64 = 10 * YKSI as i64;
const JAKAJAN_SIIRTO_MS: u32 = 550;
const LOPPU_VIIVE_MS: u32 = 250;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vaihe {
    Alku,
    Pelissa,
    /// Rahat loppuivat.
    Loppu,
    /// Pöydästä lähdetään voitolla, joka pääsi kolmen parhaan joukkoon.
    Nimi,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nappi {
    /// Lisää panokseen `LISAYKSET`-taulukon summan.
    Lisaa(usize),
    Pienin,
    Kaikki,
    Jako,
    Vakuuta,
    EiVakuutusta,
    Ota,
    Jaa,
    Tuplaa,
    Halkaise,
    Uudestaan,
    Valikko,
}

pub struct Blackjack {
    pub vaihe: Vaihe,
    pub poyta: Poyta,
    /// Seuraavan jaon panos. Säilyy kierroksesta toiseen, jos rahat riittävät.
    pub panos: u32,
    pub parhaat: Vec<tulokset::Tulos>,
    /// Nettilista. `None`, kunnes palvelin on vastannut.
    pub netti_parhaat: Option<Vec<tulokset::Tulos>>,
    /// Kirjattu tulos, joka odottaa lähetystä nettilistalle.
    pub lahetys: Option<tulokset::Tulos>,
    /// Kirjoitettavana oleva nimimerkki.
    pub nimi: String,
    /// Tosi, kun tulos odottaa tiedostoon kirjoitusta.
    pub uusi_tulos: bool,
    pub viesti: String,
    /// Pyyntö palata käynnistimeen.
    pub lopeta: bool,
    pub aanet_paalla: bool,
    pub aanijono: Vec<(Aani, u32)>,
    pub kello_ms: u32,
    /// Jakajan siirtojen määrä: kuva muuttuu niistä ilman syötettä.
    siirtoja: u32,
    seuraava_siirto_ms: u32,
    arpa: Arpa,
}

impl Blackjack {
    pub fn new(siemen: u64) -> Self {
        let mut arpa = Arpa::new(siemen);
        Self {
            vaihe: Vaihe::Alku,
            poyta: Poyta::new(&mut arpa),
            panos: ALKUPANOS,
            parhaat: Vec::new(),
            netti_parhaat: None,
            lahetys: None,
            nimi: String::new(),
            uusi_tulos: false,
            viesti: String::new(),
            lopeta: false,
            aanet_paalla: true,
            aanijono: Vec::new(),
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
            Vaihe::Pelissa | Vaihe::Loppu => Some(self.siirtoja),
        }
    }

    /// Esillä oleva lista: nettilista, tai oma lista, jos verkkoon ei ole saatu yhteyttä.
    pub fn nakyva_lista(&self) -> &[tulokset::Tulos] {
        self.netti_parhaat.as_deref().unwrap_or(&self.parhaat)
    }

    /// Pöydästä lähtö on ainoa hetki, jolloin voitto lunastetaan. Kesken jäänyt panos jää pöytään.
    pub fn poistu(&mut self) {
        if self.vaihe == Vaihe::Pelissa
            && mahtuu_parhaisiin(self.nakyva_lista(), self.poyta.pisteet())
        {
            self.nimi.clear();
            self.vaihe = Vaihe::Nimi;
        } else {
            self.lopeta = true;
        }
    }

    /// Kirjaa odottavan tuloksen ja palaa valikkoon.
    pub fn vahvista_nimi(&mut self) {
        if self.vaihe != Vaihe::Nimi {
            return;
        }
        let tulos = tulokset::Tulos {
            nimi: ota_nimi(&mut self.nimi),
            pisteet: self.poyta.pisteet(),
        };
        lisaa_tulos(&mut self.parhaat, tulos.clone());
        if let Some(lista) = &mut self.netti_parhaat {
            lisaa_tulos(lista, tulos.clone());
        }
        self.lahetys = Some(tulos);
        self.uusi_tulos = true;
        self.lopeta = true;
    }

    fn kasittele_nimi(&mut self, syote: Syote) {
        match syote {
            // Viimeisenä painettu välilyönti ei saa kuitata nimeä ennen kuin sitä on kirjoitettu.
            Syote::Ammu if self.nimi.is_empty() => {}
            Syote::Ammu | Syote::Peru => self.vahvista_nimi(),
            _ => muokkaa_nimea(&mut self.nimi, syote),
        }
    }

    pub fn edista(&mut self, ms: u32) {
        self.kello_ms = self.kello_ms.saturating_add(ms);
        while self.vaihe == Vaihe::Pelissa
            && self.poyta.vuoro == Vuoro::Jakaja
            && self.kello_ms >= self.seuraava_siirto_ms
        {
            self.poyta.jakajan_siirto();
            self.siirtoja = self.siirtoja.wrapping_add(1);
            self.seuraava_siirto_ms = self.seuraava_siirto_ms.saturating_add(JAKAJAN_SIIRTO_MS);
            self.aanijono.push((Aani::Liuku, 0));
            if self.poyta.vuoro == Vuoro::Panostus {
                self.kierros_paattyi();
            }
        }
    }

    fn aloita(&mut self) {
        self.poyta = Poyta::new(&mut self.arpa);
        self.panos = ALKUPANOS;
        self.vaihe = Vaihe::Pelissa;
        self.viesti = kaanna("blackjack.aseta_panos").to_string();
        self.aanijono.push((Aani::Ovi, 0));
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
        let kytkimella =
            matches!(syote, Syote::Klikkaus(x, y) if piirto::aanikytkimella(self.vaihe, x, y));
        if syote == Syote::VaihdaAanet || kytkimella {
            self.aanet_paalla = !self.aanet_paalla;
            return;
        }
        if self.vaihe == Vaihe::Nimi {
            return;
        }
        if self.vaihe == Vaihe::Alku {
            if matches!(syote, Syote::Ammu | Syote::Klikkaus(..)) {
                self.aloita();
            }
            return;
        }
        if let (Syote::Nuoli(drivi, dsarake), Vuoro::Panostus, Vaihe::Pelissa) =
            (syote, self.poyta.vuoro, self.vaihe)
        {
            self.muuta_panosta(dsarake as i64 * NUOLEN_ASKEL - drivi as i64 * NUOLEN_ISO_ASKEL);
        } else if let Some(nappi) = self.syotteen_nappi(syote) {
            self.paina(nappi);
        }
    }

    /// Sama kirjain voi tarkoittaa eri asiaa panostaessa ja pelatessa, kuten napitkin.
    fn syotteen_nappi(&self, syote: Syote) -> Option<Nappi> {
        if let Syote::Klikkaus(x, y) = syote {
            return piirto::nappi_pisteesta(self.vaihe, self.poyta.vuoro, x, y);
        }
        if syote == Syote::Kirjain('E') {
            return Some(Nappi::Valikko);
        }
        match (self.vaihe, self.poyta.vuoro, syote) {
            (Vaihe::Loppu, _, Syote::Kirjain('K')) => Some(Nappi::Uudestaan),
            (Vaihe::Pelissa, Vuoro::Panostus, Syote::Numero(numero)) => (numero as usize)
                .checked_sub(1)
                .filter(|&i| i < LISAYKSET.len())
                .map(Nappi::Lisaa),
            (Vaihe::Pelissa, Vuoro::Panostus, Syote::Kirjain('T')) => Some(Nappi::Pienin),
            (Vaihe::Pelissa, Vuoro::Panostus, Syote::Kirjain('K')) => Some(Nappi::Kaikki),
            (Vaihe::Pelissa, Vuoro::Panostus, Syote::Ammu) => Some(Nappi::Jako),
            (Vaihe::Pelissa, Vuoro::Vakuutus, Syote::Kirjain('V')) => Some(Nappi::Vakuuta),
            (Vaihe::Pelissa, Vuoro::Vakuutus, Syote::Kirjain('N')) => Some(Nappi::EiVakuutusta),
            (Vaihe::Pelissa, Vuoro::Pelaaja, Syote::Kirjain('O')) => Some(Nappi::Ota),
            (Vaihe::Pelissa, Vuoro::Pelaaja, Syote::Kirjain('J')) => Some(Nappi::Jaa),
            (Vaihe::Pelissa, Vuoro::Pelaaja, Syote::Kirjain('T')) => Some(Nappi::Tuplaa),
            (Vaihe::Pelissa, Vuoro::Pelaaja, Syote::Kirjain('H')) => Some(Nappi::Halkaise),
            _ => None,
        }
    }

    fn paina(&mut self, nappi: Nappi) {
        match nappi {
            Nappi::Lisaa(i) => self.muuta_panosta(LISAYKSET[i] as i64),
            Nappi::Pienin => self.panos = PIENIN_PANOS,
            Nappi::Kaikki => self.panos = self.suurin_panos(),
            Nappi::Jako => self.jaa_kortit(),
            Nappi::Vakuuta => self.vakuutuspaatos(true),
            Nappi::EiVakuutusta => self.vakuutuspaatos(false),
            Nappi::Ota => {
                self.poyta.ota();
                self.pelaaja_siirsi();
            }
            Nappi::Jaa => {
                self.poyta.jaa_tahan();
                self.pelaaja_siirsi();
            }
            Nappi::Tuplaa => {
                if self.poyta.tuplaa() {
                    self.pelaaja_siirsi();
                } else {
                    self.virhe("blackjack.ei_tuplausta");
                }
            }
            Nappi::Halkaise => {
                if self.poyta.halkaise() {
                    self.pelaaja_siirsi();
                } else {
                    self.virhe("blackjack.ei_halkaisua");
                }
            }
            Nappi::Uudestaan => self.aloita(),
            Nappi::Valikko => self.poistu(),
        }
    }

    /// Panos on aina kokonaisia rahoja, vaikka rahoissa olisi puolikas.
    fn suurin_panos(&self) -> u32 {
        self.poyta.rahat - self.poyta.rahat % YKSI
    }

    fn muuta_panosta(&mut self, muutos: i64) {
        let uusi = self.panos as i64 + muutos;
        self.panos = uusi.clamp(PIENIN_PANOS as i64, self.suurin_panos() as i64) as u32;
    }

    fn jaa_kortit(&mut self) {
        if !self.poyta.jaa(self.panos, &mut self.arpa) {
            return;
        }
        self.viesti.clear();
        self.aanijono.push((Aani::Liuku, 0));
        if self.poyta.vuoro == Vuoro::Panostus {
            self.kierros_paattyi();
        } else if self.poyta.vuoro == Vuoro::Vakuutus {
            self.viesti = if self.poyta.kadet[0].on_blackjack() {
                muotoile("blackjack.tasaraha_tarjous", &[&raha(self.panos)])
            } else {
                let hinta = raha(self.poyta.vakuutuksen_hinta());
                muotoile("blackjack.vakuutus_tarjous", &[&hinta])
            };
        }
    }

    fn vakuutuspaatos(&mut self, ottaa: bool) {
        self.poyta.vakuuta(ottaa);
        self.viesti.clear();
        self.aanijono.push((Aani::Liuku, 0));
        if self.poyta.vuoro == Vuoro::Panostus {
            self.kierros_paattyi();
        } else if ottaa {
            self.viesti = kaanna("blackjack.vakuutus_havisi").to_string();
        }
    }

    fn virhe(&mut self, avain: &str) {
        self.viesti = kaanna(avain).to_string();
        self.aanijono.push((Aani::Ahkaisy, 0));
    }

    fn pelaaja_siirsi(&mut self) {
        self.viesti.clear();
        self.aanijono.push((Aani::Liuku, 0));
        if self.poyta.vuoro == Vuoro::Jakaja {
            self.seuraava_siirto_ms = self.kello_ms.saturating_add(JAKAJAN_SIIRTO_MS);
        }
    }

    fn kierros_paattyi(&mut self) {
        self.panos = self.panos.min(self.suurin_panos()).max(PIENIN_PANOS);
        let voitto = self.poyta.voitto();
        let maara = raha(voitto.unsigned_abs() as u32);
        let blackjack = (self.poyta.kadet.iter()).any(|k| k.tulos == Some(Tulos::Blackjack));
        // Pelkkä puolikas ei riitä pienimpään panokseen.
        if self.poyta.rahat < PIENIN_PANOS {
            self.vaihe = Vaihe::Loppu;
            self.viesti = muotoile("blackjack.konkurssi", &[&raha(ALKURAHA)]);
            self.aanijono.push((Aani::Tappio, LOPPU_VIIVE_MS));
        } else if self.poyta.vakuutus_voitti() {
            let maksu = raha(2 * self.poyta.vakuutus);
            self.viesti = muotoile("blackjack.viesti_vakuutus", &[&maksu, &maara]);
            self.aanijono.push((Aani::Yhdistys, LOPPU_VIIVE_MS));
        } else if blackjack {
            self.viesti = muotoile("blackjack.viesti_blackjack", &[&maara]);
            self.aanijono.push((Aani::Voitto, LOPPU_VIIVE_MS));
        } else if voitto > 0 {
            self.viesti = muotoile("blackjack.viesti_voitto", &[&maara]);
            self.aanijono.push((Aani::Yhdistys, LOPPU_VIIVE_MS));
        } else if voitto < 0 {
            self.viesti = muotoile("blackjack.viesti_tappio", &[&maara]);
            self.aanijono.push((Aani::Ahkaisy, LOPPU_VIIVE_MS));
        } else {
            self.viesti = kaanna("blackjack.viesti_tasan").to_string();
            self.aanijono.push((Aani::Pomppu, LOPPU_VIIVE_MS));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use klassikot8000::blackjack::{Kortti, Maa};

    /// Peli panostusvaiheessa. Kortit nousevat annetussa järjestyksessä.
    fn pelissa(arvot: &[u8]) -> Blackjack {
        let mut peli = Blackjack::new(1);
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.vaihe, Vaihe::Pelissa);
        let mut kenka = vec![
            Kortti {
                maa: Maa::Pata,
                arvo: 2,
            };
            100
        ];
        kenka.extend(arvot.iter().rev().map(|&arvo| Kortti {
            maa: Maa::Hertta,
            arvo,
        }));
        peli.poyta.kenka = kenka;
        peli
    }

    #[test]
    fn panos_muuttuu_nuolilla_numeroilla_ja_napeilla_mutta_pysyy_rajoissa() {
        let mut peli = pelissa(&[]);
        assert_eq!(peli.panos, ALKUPANOS);
        peli.kasittele(Syote::Nuoli(0, 1));
        assert_eq!(peli.panos, 11 * YKSI);
        peli.kasittele(Syote::Nuoli(-1, 0));
        assert_eq!(peli.panos, 21 * YKSI);
        peli.kasittele(Syote::Numero(4));
        assert_eq!(peli.panos, 121 * YKSI);
        peli.kasittele(Syote::Numero(9));
        assert_eq!(peli.panos, 121 * YKSI);
        peli.kasittele(Syote::Kirjain('T'));
        assert_eq!(peli.panos, PIENIN_PANOS);
        peli.kasittele(Syote::Nuoli(1, 0));
        assert_eq!(peli.panos, PIENIN_PANOS);
        peli.kasittele(Syote::Kirjain('K'));
        assert_eq!(peli.panos, ALKURAHA);
        peli.kasittele(Syote::Numero(4));
        assert_eq!(peli.panos, ALKURAHA);

        let (x, y) = piirto::napin_keskipiste(Vaihe::Pelissa, Vuoro::Panostus, Nappi::Pienin);
        peli.kasittele(Syote::Klikkaus(x, y));
        assert_eq!(peli.panos, PIENIN_PANOS);
        let (x, y) = piirto::napin_keskipiste(Vaihe::Pelissa, Vuoro::Panostus, Nappi::Lisaa(2));
        peli.kasittele(Syote::Klikkaus(x, y));
        assert_eq!(peli.panos, 26 * YKSI);
    }

    #[test]
    fn jakaja_pelaa_ajan_kuluessa_ja_tulos_nakyy_vasta_lopussa() {
        let mut peli = pelissa(&[10, 5, 10, 6, 4, 2]);
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.poyta.vuoro, Vuoro::Pelaaja);
        assert_eq!(peli.poyta.rahat, ALKURAHA - ALKUPANOS);
        let lepo = peli.kuvan_askel();
        peli.edista(10_000);
        assert_eq!(peli.kuvan_askel(), lepo);

        peli.kasittele(Syote::Kirjain('J'));
        assert_eq!(peli.poyta.vuoro, Vuoro::Jakaja);
        peli.edista(JAKAJAN_SIIRTO_MS - 1);
        assert!(peli.poyta.piilossa);
        // syöte ei hoputa jakajaa eikä muuta panosta kesken kierroksen
        peli.kasittele(Syote::Ammu);
        peli.kasittele(Syote::Nuoli(0, 1));
        assert_eq!(peli.panos, ALKUPANOS);
        assert_eq!(peli.poyta.vuoro, Vuoro::Jakaja);

        let ennen = peli.kuvan_askel();
        peli.edista(1);
        assert!(!peli.poyta.piilossa);
        assert_ne!(peli.kuvan_askel(), ennen);
        assert_eq!(peli.poyta.rahat, ALKURAHA - ALKUPANOS);
        assert!(peli.viesti.is_empty());

        peli.edista(2 * JAKAJAN_SIIRTO_MS);
        assert_eq!(peli.poyta.vuoro, Vuoro::Panostus);
        assert_eq!(peli.poyta.rahat, ALKURAHA + ALKUPANOS);
        assert_eq!(
            peli.viesti,
            muotoile("blackjack.viesti_voitto", &[&raha(ALKUPANOS)])
        );
        assert!(peli.aanijono.contains(&(Aani::Yhdistys, LOPPU_VIIVE_MS)));
    }

    #[test]
    fn blackjack_paattyy_heti_ja_soi_voitolle() {
        let mut peli = pelissa(&[1, 9, 13, 8]);
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.poyta.vuoro, Vuoro::Panostus);
        assert_eq!(peli.poyta.rahat, ALKURAHA + 15 * YKSI);
        assert!(peli.aanijono.contains(&(Aani::Voitto, LOPPU_VIIVE_MS)));
    }

    #[test]
    fn pienimman_panoksen_blackjack_maksaa_puolikkaan_ja_nakyy_viestissa() {
        let mut peli = pelissa(&[1, 9, 13, 8]);
        peli.kasittele(Syote::Kirjain('T'));
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.poyta.rahat, ALKURAHA + 3);
        assert_eq!(
            peli.viesti,
            muotoile("blackjack.viesti_blackjack", &[&"1,5"])
        );
    }

    #[test]
    fn kaikki_rahat_havittyaan_peli_loppuu_ja_alkaa_alusta() {
        let mut peli = pelissa(&[10, 10, 7, 10]);
        peli.kasittele(Syote::Kirjain('K'));
        peli.kasittele(Syote::Ammu);
        peli.kasittele(Syote::Kirjain('J'));
        peli.edista(10_000);
        assert_eq!(peli.vaihe, Vaihe::Loppu);
        assert_eq!(peli.poyta.rahat, 0);
        assert!(peli.aanijono.contains(&(Aani::Tappio, LOPPU_VIIVE_MS)));

        // lopussa panos tai jako eivät enää toimi
        peli.kasittele(Syote::Ammu);
        peli.kasittele(Syote::Nuoli(0, 1));
        assert_eq!(peli.vaihe, Vaihe::Loppu);
        peli.kasittele(Syote::Kirjain('K'));
        assert_eq!(peli.vaihe, Vaihe::Pelissa);
        assert_eq!(peli.poyta.rahat, ALKURAHA);
        assert_eq!(peli.panos, ALKUPANOS);
        assert!(peli.poyta.kadet.is_empty());
    }

    #[test]
    fn pelkka_puolikas_ei_riita_panokseen() {
        let mut peli = pelissa(&[10, 10, 7, 10]);
        peli.poyta.rahat = PIENIN_PANOS + 1;
        peli.panos = PIENIN_PANOS;
        peli.kasittele(Syote::Ammu);
        peli.kasittele(Syote::Kirjain('J'));
        peli.edista(10_000);
        assert_eq!(peli.poyta.rahat, 1);
        assert_eq!(peli.vaihe, Vaihe::Loppu);
    }

    #[test]
    fn panos_pienenee_jaljella_oleviin_kokonaisiin_rahoihin() {
        let mut peli = pelissa(&[10, 10, 7, 10]);
        peli.panos = 295 * YKSI;
        peli.poyta.rahat = ALKURAHA + 1;
        peli.kasittele(Syote::Ammu);
        peli.kasittele(Syote::Kirjain('J'));
        peli.edista(10_000);
        assert_eq!(peli.poyta.rahat, 5 * YKSI + 1);
        assert_eq!(peli.panos, 5 * YKSI);
        assert_eq!(peli.vaihe, Vaihe::Pelissa);
    }

    #[test]
    fn vakuutus_kysytaan_jakajan_assasta_ja_kierros_jatkuu_ilman_blackjackia() {
        let mut peli = pelissa(&[10, 1, 9, 6]);
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.poyta.vuoro, Vuoro::Vakuutus);
        assert_eq!(
            peli.viesti,
            muotoile("blackjack.vakuutus_tarjous", &[&raha(ALKUPANOS / 2)])
        );
        // muut näppäimet eivät tee mitään ennen päätöstä
        peli.kasittele(Syote::Kirjain('O'));
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.poyta.kadet[0].kortit.len(), 2);
        assert_eq!(peli.poyta.vuoro, Vuoro::Vakuutus);

        peli.kasittele(Syote::Kirjain('V'));
        assert_eq!(peli.poyta.vuoro, Vuoro::Pelaaja);
        assert!(peli.poyta.piilossa);
        assert_eq!(peli.poyta.rahat, ALKURAHA - ALKUPANOS - ALKUPANOS / 2);
        assert_eq!(peli.viesti, kaanna("blackjack.vakuutus_havisi"));
    }

    #[test]
    fn vakuutus_maksaa_jakajan_blackjackista_ja_kieltaytyminen_haviaa() {
        let mut peli = pelissa(&[10, 1, 9, 13]);
        peli.kasittele(Syote::Ammu);
        let (x, y) = piirto::napin_keskipiste(Vaihe::Pelissa, Vuoro::Vakuutus, Nappi::Vakuuta);
        peli.kasittele(Syote::Klikkaus(x, y));
        assert_eq!(peli.poyta.vuoro, Vuoro::Panostus);
        assert_eq!(peli.poyta.rahat, ALKURAHA);
        assert_eq!(
            peli.viesti,
            muotoile("blackjack.viesti_vakuutus", &[&raha(ALKUPANOS), &raha(0)])
        );

        let mut peli = pelissa(&[10, 1, 9, 13]);
        peli.kasittele(Syote::Ammu);
        peli.kasittele(Syote::Kirjain('N'));
        assert_eq!(peli.poyta.rahat, ALKURAHA - ALKUPANOS);
        assert_eq!(
            peli.viesti,
            muotoile("blackjack.viesti_tappio", &[&raha(ALKUPANOS)])
        );
    }

    #[test]
    fn blackjack_kadella_vakuutus_on_tasaraha() {
        let mut peli = pelissa(&[1, 1, 13, 6]);
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.poyta.vuoro, Vuoro::Vakuutus);
        assert_eq!(
            peli.viesti,
            muotoile("blackjack.tasaraha_tarjous", &[&raha(ALKUPANOS)])
        );
        peli.kasittele(Syote::Kirjain('V'));
        assert_eq!(peli.poyta.rahat, ALKURAHA + ALKUPANOS);
    }

    #[test]
    fn mahdoton_tuplaus_ja_halkaisu_antavat_viestin_eivatka_muuta_tilaa() {
        let mut peli = pelissa(&[5, 10, 6, 9]);
        peli.kasittele(Syote::Kirjain('K'));
        peli.kasittele(Syote::Ammu);
        peli.kasittele(Syote::Kirjain('T'));
        assert_eq!(peli.viesti, kaanna("blackjack.ei_tuplausta"));
        peli.kasittele(Syote::Kirjain('H'));
        assert_eq!(peli.viesti, kaanna("blackjack.ei_halkaisua"));
        assert_eq!(peli.poyta.kadet[0].kortit.len(), 2);
        assert_eq!(peli.poyta.vuoro, Vuoro::Pelaaja);
    }

    #[test]
    fn voitolla_poistuva_kirjoittaa_nimen_ja_tulos_on_voitto() {
        let mut peli = pelissa(&[]);
        peli.poyta.rahat = ALKURAHA + 120 * YKSI;
        peli.kasittele(Syote::Kirjain('E'));
        assert_eq!(peli.vaihe, Vaihe::Nimi);
        assert!(!peli.lopeta);
        // Nappikirjain menee nimeen, eikä klikkaus paina piilossa olevia nappeja.
        peli.kasittele(Syote::Kirjain('E'));
        let (x, y) = piirto::napin_keskipiste(Vaihe::Nimi, Vuoro::Panostus, Nappi::Valikko);
        peli.kasittele(Syote::Klikkaus(x, y));
        assert!(!peli.lopeta);
        peli.kasittele(Syote::Ammu);
        assert!(peli.lopeta);
        assert_eq!(peli.parhaat[0].nimi, "E");
        assert_eq!(peli.parhaat[0].pisteet, 120);
        assert!(peli.uusi_tulos);
        assert_eq!(peli.lahetys.as_ref().unwrap().pisteet, 120);
    }

    #[test]
    fn ilman_voittoa_tai_listapaikkaa_poistutaan_heti() {
        let mut peli = pelissa(&[]);
        peli.kasittele(Syote::Peru);
        assert!(peli.lopeta);
        assert_eq!(peli.lahetys, None);

        let mut peli = pelissa(&[]);
        peli.poyta.rahat = ALKURAHA + 5 * YKSI;
        peli.netti_parhaat = Some(vec![
            tulokset::Tulos {
                nimi: "A".to_string(),
                pisteet: 100,
            };
            3
        ]);
        peli.kasittele(Syote::Peru);
        assert!(peli.lopeta);
        assert_eq!(peli.lahetys, None);
    }

    #[test]
    fn kesken_kierroksen_poistuva_jattaa_panoksen_eika_jakaja_jatka() {
        let mut peli = pelissa(&[10, 5, 10, 6, 4, 2]);
        peli.poyta.rahat = ALKURAHA + 50 * YKSI;
        peli.kasittele(Syote::Ammu);
        peli.kasittele(Syote::Kirjain('J'));
        assert_eq!(peli.poyta.vuoro, Vuoro::Jakaja);
        peli.kasittele(Syote::Peru);
        assert_eq!(peli.vaihe, Vaihe::Nimi);
        let kuva = peli.kuvan_askel();
        peli.edista(10_000);
        assert_eq!(peli.poyta.vuoro, Vuoro::Jakaja);
        assert_ne!(peli.kuvan_askel(), kuva);
        peli.kasittele(Syote::Kirjain('A'));
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.parhaat[0].pisteet, 40);
    }

    #[test]
    fn napit_toimivat_pelatessa_ja_esc_palaa_valikkoon() {
        let mut peli = pelissa(&[8, 10, 8, 9, 3, 2]);
        peli.kasittele(Syote::Ammu);
        let (x, y) = piirto::napin_keskipiste(Vaihe::Pelissa, Vuoro::Pelaaja, Nappi::Halkaise);
        peli.kasittele(Syote::Klikkaus(x, y));
        assert_eq!(peli.poyta.kadet.len(), 2);
        let (x, y) = piirto::napin_keskipiste(Vaihe::Pelissa, Vuoro::Pelaaja, Nappi::Ota);
        peli.kasittele(Syote::Klikkaus(x, y));
        assert_eq!(peli.poyta.kadet[0].kortit.len(), 3);
        peli.kasittele(Syote::Peru);
        assert!(peli.lopeta);

        let mut peli = pelissa(&[]);
        peli.kasittele(Syote::Kirjain('E'));
        assert!(peli.lopeta);
    }
}
