use klassikot8000::Arpa;
use klassikot8000::miinaharava::{Avaus, Miinakentta, Paikka, SUURIN_AIKA, Vaikeus, aikapisteet};

use super::piirto;
use crate::kaynnistin::tallennus::{Asetukset, Tallennus};
use crate::yhteinen::aanet::{Aani, PUHE_VIIVE_MS};
use crate::yhteinen::kieli::{kaanna, muotoile};
use crate::yhteinen::pelinakyma::{Lista, Pelinakyma};
use crate::yhteinen::piirto::{Kehys, VILKKU_MS, aanikytkimella};
use crate::yhteinen::syote::{Syote, Yleinen, yleinen};
use crate::yhteinen::tulokset::{
    Nettipeli, Tulos, lisaa_tulos, mahtuu_parhaisiin, muokkaa_nimea, ota_nimi,
};

pub const RAJAHDYS_MS: u32 = 500;
const LOPPU_VIIVE_MS: u32 = 250;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vaihe {
    Alku,
    Pelissa,
    /// Voitto pääsi kolmen parhaan joukkoon ja odottaa nimimerkkiä.
    Nimi,
    Voitto,
    Tappio,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nappi {
    Uudestaan,
    Valikko,
}

pub struct Miinaharava {
    pub vaihe: Vaihe,
    pub vaikeus: Vaikeus,
    pub lauta: Miinakentta,
    /// Parhaat ajat sekunteina tasoittain. Nolla tarkoittaa, ettei tasoa ole läpäisty.
    pub ajat: [u32; 3],
    /// Tosi, kun uusi ennätys odottaa tiedostoon kirjoitusta.
    pub uusi_tulos: bool,
    /// Top 3 pisteinä, suurin ensin. Pisteet tulevat ajasta ja tasosta.
    pub parhaat: Vec<Tulos>,
    /// Nettilista, kun palvelin on vastannut.
    pub netti_parhaat: Option<Vec<Tulos>>,
    /// Nimetty tulos, jonka käynnistin lähettää nettilistalle.
    pub lahetys: Option<Tulos>,
    /// Kirjoitettavana oleva nimimerkki.
    pub nimi: String,
    pub viesti: String,
    pub kursori: Paikka,
    /// Kursori näkyy vasta, kun näppäimistöä on käytetty.
    pub kursori_nakyy: bool,
    pub hiiren_ruutu: Option<Paikka>,
    /// Miinan räjähdyksen alkuhetki.
    pub rajahdys: Option<u32>,
    /// Pyyntö palata käynnistimeen.
    pub lopeta: bool,
    pub aanet_paalla: bool,
    pub aanijono: Vec<(Aani, u32)>,
    pub kello_ms: u32,
    /// Ensimmäisen avauksen hetki: kello käy siitä asti.
    aika_alku: Option<u32>,
    /// Päättyneen pelin aika sekunteina.
    loppuaika: Option<u32>,
    arpa: Arpa,
}

fn tason_indeksi(vaikeus: Vaikeus) -> usize {
    Vaikeus::KAIKKI
        .iter()
        .position(|&taso| taso == vaikeus)
        .unwrap_or(0)
}

impl Miinaharava {
    pub fn new(siemen: u64) -> Self {
        Self {
            vaihe: Vaihe::Alku,
            vaikeus: Vaikeus::Helppo,
            lauta: Miinakentta::new(Vaikeus::Helppo),
            ajat: [0; 3],
            uusi_tulos: false,
            parhaat: Vec::new(),
            netti_parhaat: None,
            lahetys: None,
            nimi: String::new(),
            viesti: String::new(),
            kursori: (0, 0),
            kursori_nakyy: false,
            hiiren_ruutu: None,
            rajahdys: None,
            lopeta: false,
            aanet_paalla: true,
            aanijono: Vec::new(),
            kello_ms: 0,
            aika_alku: None,
            loppuaika: None,
            arpa: Arpa::new(siemen),
        }
    }

    /// Kuva riippuu tästä ja syötteestä. `None`, kun alkuruudun malli pyörii.
    /// Paikallaan oleva kenttä muuttuu vain kellon ja vilkun tahdissa.
    pub fn kuvan_askel(&self) -> Option<u32> {
        if self.vaihe == Vaihe::Alku {
            None
        } else if self.rajahdys.is_some() {
            Some(self.kello_ms)
        } else {
            Some(self.kello_ms / VILKKU_MS)
        }
    }

    pub fn aika(&self) -> u32 {
        let kulunut = self
            .aika_alku
            .map_or(0, |alku| self.kello_ms.saturating_sub(alku) / 1000);
        self.loppuaika.unwrap_or(kulunut).min(SUURIN_AIKA)
    }

    /// Nettilista, kun se on saatu, muuten omat tulokset.
    pub fn nakyva_lista(&self) -> &[Tulos] {
        self.netti_parhaat.as_deref().unwrap_or(&self.parhaat)
    }

    /// Päättyneen pelin pisteet top-listalle.
    pub fn pisteet(&self) -> u32 {
        aikapisteet(self.vaikeus, self.aika())
    }

    /// Kirjaa nimeä odottavan voiton.
    pub fn vahvista_nimi(&mut self) {
        if self.vaihe != Vaihe::Nimi {
            return;
        }
        let tulos = Tulos {
            nimi: ota_nimi(&mut self.nimi),
            pisteet: self.pisteet(),
        };
        lisaa_tulos(&mut self.parhaat, tulos.clone());
        if let Some(lista) = &mut self.netti_parhaat {
            lisaa_tulos(lista, tulos.clone());
        }
        self.lahetys = Some(tulos);
        self.uusi_tulos = true;
        self.vaihe = Vaihe::Voitto;
    }

    fn kasittele_nimi(&mut self, syote: Syote) {
        match syote {
            // Avaukseen painettu välilyönti ei saa kuitata nimeä ennen kuin sitä on kirjoitettu.
            Syote::Ammu if self.nimi.is_empty() => {}
            Syote::Ammu | Syote::Peru => self.vahvista_nimi(),
            _ => muokkaa_nimea(&mut self.nimi, syote),
        }
    }

    /// Nykyisen tason paras aika, jos taso on läpäisty.
    pub fn paras_aika(&self) -> Option<u32> {
        Some(self.ajat[tason_indeksi(self.vaikeus)]).filter(|&aika| aika > 0)
    }

    pub fn edista(&mut self, ms: u32) {
        self.kello_ms = self.kello_ms.saturating_add(ms);
        if self
            .rajahdys
            .is_some_and(|alku| self.kello_ms >= alku.saturating_add(RAJAHDYS_MS))
        {
            self.rajahdys = None;
        }
    }

    pub fn aloita(&mut self) {
        self.lauta = Miinakentta::new(self.vaikeus);
        self.vaihe = Vaihe::Pelissa;
        self.kursori = (self.vaikeus.korkeus() / 2, self.vaikeus.leveys() / 2);
        self.kursori_nakyy = false;
        self.hiiren_ruutu = None;
        self.rajahdys = None;
        self.aika_alku = None;
        self.loppuaika = None;
        self.viesti = kaanna("miinaharava.aloita").to_string();
        self.aanijono.push((Aani::Alku, 0));
    }

    pub fn kasittele(&mut self, syote: Syote) {
        match yleinen(
            syote,
            self.vaihe == Vaihe::Nimi,
            &mut self.aanet_paalla,
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
            Vaihe::Alku => self.kasittele_alku(syote),
            Vaihe::Pelissa => self.kasittele_pelissa(syote),
            Vaihe::Voitto | Vaihe::Tappio => self.kasittele_loppu(syote),
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
            self.aloita();
        }
    }

    fn kasittele_pelissa(&mut self, syote: Syote) {
        let koko = (self.vaikeus.korkeus(), self.vaikeus.leveys());
        match syote {
            Syote::Hiiri(x, y) => {
                self.hiiren_ruutu = piirto::ruutu_pisteesta(self.vaikeus, x, y);
                self.kursori_nakyy = false;
            }
            Syote::Klikkaus(x, y) => {
                if let Some(ruutu) = piirto::ruutu_pisteesta(self.vaikeus, x, y) {
                    self.kursori = ruutu;
                    self.kursori_nakyy = false;
                    self.avaa(ruutu);
                }
            }
            Syote::OikeaKlikkaus(x, y) => {
                if let Some(ruutu) = piirto::ruutu_pisteesta(self.vaikeus, x, y) {
                    self.kursori = ruutu;
                    self.kursori_nakyy = false;
                    self.liputa(ruutu);
                }
            }
            Syote::Nuoli(drivi, dsarake) => {
                // Ensimmäinen painallus vain tuo kursorin näkyviin.
                if self.kursori_nakyy {
                    let rivi = self.kursori.0.saturating_add_signed(drivi as isize);
                    let sarake = self.kursori.1.saturating_add_signed(dsarake as isize);
                    self.kursori = (rivi.min(koko.0 - 1), sarake.min(koko.1 - 1));
                }
                self.kursori_nakyy = true;
            }
            Syote::Ammu => {
                if self.kursori_nakyy {
                    self.avaa(self.kursori);
                }
                self.kursori_nakyy = true;
            }
            Syote::Kirjain('F') => {
                if self.kursori_nakyy {
                    self.liputa(self.kursori);
                }
                self.kursori_nakyy = true;
            }
            _ => {}
        }
    }

    /// Suljettu ruutu aukeaa. Avattu numero avaa ympäriltään, kun liput täsmäävät.
    pub fn avaa(&mut self, ruutu: Paikka) {
        let tulos = if self.lauta.ruutu(ruutu).auki {
            self.lauta.avaa_ymparilta(ruutu, &mut self.arpa)
        } else {
            self.lauta.avaa(ruutu, &mut self.arpa)
        };
        match tulos {
            Avaus::Ei => {}
            Avaus::Auki(maara) => {
                if self.aika_alku.is_none() {
                    self.aika_alku = Some(self.kello_ms);
                }
                self.aanijono
                    .push((if maara > 1 { Aani::Liuku } else { Aani::Pomppu }, 0));
                self.viesti.clear();
            }
            Avaus::Miina => {
                self.loppuaika = Some(self.aika());
                self.vaihe = Vaihe::Tappio;
                self.rajahdys = Some(self.kello_ms);
                self.aanijono.push((Aani::Rajahdys, 0));
                self.aanijono.push((Aani::Tappio, LOPPU_VIIVE_MS));
                self.aanijono
                    .push((Aani::PeliOhi, LOPPU_VIIVE_MS + PUHE_VIIVE_MS));
                self.viesti = kaanna("miinaharava.pamaus").to_string();
            }
            Avaus::Voitto => self.voita(),
        }
    }

    fn voita(&mut self) {
        // Kenttä voi aueta jo ensimmäisellä avauksella, jolloin kello ei ehdi käynnistyä.
        let aika = self.aika().max(1);
        self.loppuaika = Some(aika);
        self.vaihe = Vaihe::Voitto;
        self.aanijono.push((Aani::Voitto, LOPPU_VIIVE_MS));
        self.aanijono
            .push((Aani::Voitit, LOPPU_VIIVE_MS + PUHE_VIIVE_MS));
        let pisteet = self.pisteet();
        let paras = &mut self.ajat[tason_indeksi(self.vaikeus)];
        if *paras == 0 || aika < *paras {
            *paras = aika;
            self.uusi_tulos = true;
            self.viesti = muotoile("miinaharava.ennatys", &[&aika, &pisteet]);
        } else {
            self.viesti = muotoile("miinaharava.aika_viesti", &[&aika, &pisteet]);
        }
        if mahtuu_parhaisiin(self.nakyva_lista(), pisteet) {
            self.nimi.clear();
            self.vaihe = Vaihe::Nimi;
        }
    }

    pub fn liputa(&mut self, ruutu: Paikka) {
        if self.lauta.liputa(ruutu) {
            self.aanijono.push((Aani::Tiili, 0));
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
            Some(Nappi::Uudestaan) => self.aloita(),
            Some(Nappi::Valikko) => self.lopeta = true,
            None => {}
        }
    }
}

impl Pelinakyma for Miinaharava {
    fn lue_tallennuksesta(&mut self, tallennus: &Tallennus) {
        self.vaikeus = tallennus.asetukset.miinaharava;
        self.ajat = tallennus.miinaajat;
    }

    fn kirjaa_asetukset(&self, asetukset: &mut Asetukset) {
        asetukset.miinaharava = self.vaikeus;
    }

    fn kirjaa_tallennukseen(&self, tallennus: &mut Tallennus) {
        tallennus.miinaajat = self.ajat;
    }

    fn kasittele(&mut self, syote: Syote) {
        Miinaharava::kasittele(self, syote);
    }

    fn edista(&mut self, ms: u32) {
        Miinaharava::edista(self, ms);
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
        self.vahvista_nimi();
    }

    fn listat(&mut self) -> Vec<Lista<'_>> {
        vec![Lista {
            peli: Nettipeli::Miinaharava,
            paras: None,
            parhaat: &mut self.parhaat,
            netti: &mut self.netti_parhaat,
            lahetys: Some(&mut self.lahetys),
        }]
    }

    fn kuvan_avain(&self) -> Option<(u32, u32)> {
        self.kuvan_askel().map(|askel| (askel, 0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use klassikot8000::miinaharava::Tila;

    fn pelissa(vaikeus: Vaikeus, miinat: &[Paikka]) -> Miinaharava {
        let mut peli = Miinaharava::new(1);
        peli.vaikeus = vaikeus;
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.vaihe, Vaihe::Pelissa);
        peli.aanijono.clear();
        peli.lauta = Miinakentta::miinoista(vaikeus, miinat);
        peli
    }

    #[test]
    fn listalle_yltava_voitto_kysyy_nimen_ja_lahtee_nettiin() {
        let mut peli = pelissa(Vaikeus::Normaali, &[(0, 0)]);
        peli.avaa((1, 1));
        peli.edista(40_000);
        peli.avaa((15, 15));
        assert_eq!(peli.vaihe, Vaihe::Nimi);
        let pisteet = aikapisteet(Vaikeus::Normaali, 40);
        assert_eq!(peli.pisteet(), pisteet);
        // tyhjää nimeä ei kuitata välilyönnillä
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.vaihe, Vaihe::Nimi);
        peli.kasittele(Syote::Kirjain('V'));
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.vaihe, Vaihe::Voitto);
        let tulos = Tulos {
            nimi: "V".to_string(),
            pisteet,
        };
        assert_eq!(peli.parhaat, std::slice::from_ref(&tulos));
        assert_eq!(peli.lahetys, Some(tulos));
        assert!(peli.uusi_tulos);

        // täydelle listalle heikompi tulos ei pääse, eikä nimeä kysytä
        let mut peli = pelissa(Vaikeus::Helppo, &[(0, 0)]);
        peli.parhaat = vec![
            Tulos {
                nimi: "A".to_string(),
                pisteet: 5000,
            };
            3
        ];
        peli.avaa((1, 1));
        peli.avaa((8, 8));
        assert_eq!(peli.vaihe, Vaihe::Voitto);
        assert!(peli.lahetys.is_none());
    }

    #[test]
    fn suljettu_ikkuna_kirjaa_nimea_odottavan_voiton() {
        let mut peli = pelissa(Vaikeus::Helppo, &[(0, 0)]);
        peli.avaa((1, 1));
        peli.avaa((8, 8));
        assert_eq!(peli.vaihe, Vaihe::Nimi);
        Pelinakyma::viimeistele(&mut peli);
        assert_eq!(peli.parhaat[0].nimi, kaanna("yhteinen.oletusnimi"));
    }

    fn klikkaa(peli: &mut Miinaharava, ruutu: Paikka) {
        let (x, y) = piirto::ruudun_keskipiste(peli.vaikeus, ruutu);
        peli.kasittele(Syote::Klikkaus(x, y));
    }

    #[test]
    fn taso_valitaan_alussa_ja_peli_alkaa_silla() {
        let mut peli = Miinaharava::new(1);
        peli.kasittele(Syote::Numero(3));
        assert_eq!(peli.vaikeus, Vaikeus::Vaikea);
        peli.kasittele(Syote::Nuoli(0, -1));
        assert_eq!(peli.vaikeus, Vaikeus::Normaali);
        peli.kasittele(Syote::Numero(9));
        assert_eq!(peli.vaikeus, Vaikeus::Normaali);
        let (x, y) = piirto::vaikeuden_keskipiste(Vaikeus::Helppo);
        peli.kasittele(Syote::Klikkaus(x, y));
        assert_eq!(peli.vaikeus, Vaikeus::Helppo);
        assert_eq!(peli.vaihe, Vaihe::Alku);
        peli.kasittele(Syote::Klikkaus(0, 300));
        assert_eq!(peli.vaihe, Vaihe::Pelissa);
        assert_eq!(peli.lauta.vaikeus, Vaikeus::Helppo);
        assert_eq!(peli.aanijono, [(Aani::Alku, 0)]);
    }

    #[test]
    fn vasen_avaa_ja_oikea_liputtaa() {
        let mut peli = pelissa(Vaikeus::Helppo, &[(0, 0), (8, 8)]);
        klikkaa(&mut peli, (1, 1));
        assert!(peli.lauta.ruutu((1, 1)).auki);
        assert_eq!(peli.aanijono, [(Aani::Pomppu, 0)]);

        let (x, y) = piirto::ruudun_keskipiste(Vaikeus::Helppo, (0, 0));
        peli.kasittele(Syote::OikeaKlikkaus(x, y));
        assert!(peli.lauta.ruutu((0, 0)).lippu);
        // liputettu ei aukea, joten miina ei räjähdä
        peli.kasittele(Syote::Klikkaus(x, y));
        assert_eq!(peli.vaihe, Vaihe::Pelissa);

        // numeron klikkaus avaa ympäriltä, kun liput täsmäävät
        klikkaa(&mut peli, (1, 1));
        assert!(peli.lauta.ruutu((2, 2)).auki);
    }

    #[test]
    fn nappaimisto_liikuttaa_avaa_ja_liputtaa() {
        let mut peli = pelissa(Vaikeus::Helppo, &[(0, 0), (8, 8)]);
        peli.kasittele(Syote::Ammu);
        assert!(peli.kursori_nakyy);
        assert!(!peli.lauta.ruutu(peli.kursori).auki);
        peli.kursori = (0, 1);
        peli.kasittele(Syote::Ammu);
        assert!(peli.lauta.ruutu((0, 1)).auki);
        peli.kasittele(Syote::Nuoli(0, -1));
        assert_eq!(peli.kursori, (0, 0));
        peli.kasittele(Syote::Nuoli(0, -1));
        peli.kasittele(Syote::Nuoli(-1, 0));
        assert_eq!(peli.kursori, (0, 0));
        peli.kasittele(Syote::Kirjain('F'));
        assert!(peli.lauta.ruutu((0, 0)).lippu);
        assert_eq!(peli.aanijono.last(), Some(&(Aani::Tiili, 0)));
        peli.kasittele(Syote::Hiiri(0, 0));
        assert!(!peli.kursori_nakyy);
    }

    #[test]
    fn kello_kay_ensimmaisesta_avauksesta_ja_pysahtyy_loppuun() {
        let mut peli = pelissa(Vaikeus::Helppo, &[(0, 0), (8, 8)]);
        peli.edista(5000);
        assert_eq!(peli.aika(), 0);
        klikkaa(&mut peli, (1, 1));
        peli.edista(3000);
        assert_eq!(peli.aika(), 3);
        klikkaa(&mut peli, (0, 0));
        assert_eq!(peli.vaihe, Vaihe::Tappio);
        peli.edista(5000);
        assert_eq!(peli.aika(), 3);
        assert!(peli.rajahdys.is_none());
    }

    #[test]
    fn miina_soi_ja_paattaa_pelin_ilman_ennatysta() {
        let mut peli = pelissa(Vaikeus::Helppo, &[(0, 0), (8, 8)]);
        klikkaa(&mut peli, (0, 0));
        assert_eq!(peli.vaihe, Vaihe::Tappio);
        assert_eq!(peli.aanijono[0], (Aani::Rajahdys, 0));
        assert!(peli.rajahdys.is_some());
        assert!(!peli.uusi_tulos);
        assert_eq!(peli.ajat, [0; 3]);
    }

    /// Numeroruutu käynnistää kellon, ja kaukainen tyhjä ruutu avaa kerralla kaiken muun.
    fn pelaa_voittoon(peli: &mut Miinaharava, aika_ms: u32) {
        peli.lauta = Miinakentta::miinoista(peli.vaikeus, &[(0, 0)]);
        klikkaa(peli, (1, 1));
        peli.edista(aika_ms);
        peli.aanijono.clear();
        klikkaa(peli, (8, 8));
        assert_eq!(peli.vaihe, Vaihe::Voitto);
    }

    #[test]
    fn voitto_kirjaa_parhaan_ajan_vain_jos_se_paranee() {
        let mut peli = pelissa(Vaikeus::Helppo, &[(0, 0)]);
        // Täysi top 3, jottei nimikysely tule väliin.
        peli.parhaat = vec![
            Tulos {
                nimi: "A".to_string(),
                pisteet: 100_000,
            };
            3
        ];
        pelaa_voittoon(&mut peli, 40_000);
        assert_eq!(peli.ajat, [40, 0, 0]);
        assert!(peli.uusi_tulos);
        assert_eq!(peli.paras_aika(), Some(40));
        assert!(peli.viesti.contains("40"));
        assert!(peli.aanijono.contains(&(Aani::Voitto, LOPPU_VIIVE_MS)));
        assert_eq!(
            peli.aanijono.last(),
            Some(&(Aani::Voitit, LOPPU_VIIVE_MS + PUHE_VIIVE_MS))
        );

        // hitaampi ei kirjaudu
        peli.uusi_tulos = false;
        peli.aloita();
        pelaa_voittoon(&mut peli, 90_000);
        assert_eq!(peli.ajat, [40, 0, 0]);
        assert!(!peli.uusi_tulos);

        // nopeampi kirjautuu
        peli.aloita();
        pelaa_voittoon(&mut peli, 9_000);
        assert_eq!(peli.ajat, [9, 0, 0]);
        assert!(peli.uusi_tulos);

        // heti aukeava kenttä ei kirjaa nollaa, joka tarkoittaisi läpäisemätöntä tasoa
        peli.ajat = [0; 3];
        peli.aloita();
        peli.lauta = Miinakentta::miinoista(Vaikeus::Helppo, &[(0, 0)]);
        klikkaa(&mut peli, (8, 8));
        assert_eq!(peli.ajat, [1, 0, 0]);
    }

    #[test]
    fn taso_pitaa_oman_aikansa_ja_kellolla_on_kattonsa() {
        let mut peli = pelissa(Vaikeus::Normaali, &[(0, 0)]);
        peli.ajat = [50, 0, 7];
        assert_eq!(peli.paras_aika(), None);
        klikkaa(&mut peli, (1, 1));
        peli.edista(5_000_000);
        assert_eq!(peli.aika(), SUURIN_AIKA);
    }

    #[test]
    fn lopun_napit_ja_kirjaimet_toimivat() {
        let mut peli = pelissa(Vaikeus::Normaali, &[(0, 0)]);
        klikkaa(&mut peli, (0, 0));
        let (x, y) = piirto::napin_keskipiste(Nappi::Uudestaan);
        peli.kasittele(Syote::Klikkaus(x, y));
        assert_eq!(peli.vaihe, Vaihe::Pelissa);
        assert_eq!(peli.lauta.tila, Tila::Alussa);
        assert_eq!(peli.lauta.vaikeus, Vaikeus::Normaali);
        assert_eq!(peli.aika(), 0);

        peli.vaihe = Vaihe::Tappio;
        let (x, y) = piirto::napin_keskipiste(Nappi::Valikko);
        peli.kasittele(Syote::Klikkaus(x, y));
        assert!(peli.lopeta);

        let mut peli = pelissa(Vaikeus::Helppo, &[(0, 0)]);
        peli.kasittele(Syote::Peru);
        assert!(peli.lopeta);
    }

    #[test]
    fn kuva_paivittyy_vain_vilkun_ja_rajahdyksen_tahdissa() {
        let mut peli = Miinaharava::new(1);
        assert_eq!(peli.kuvan_askel(), None);
        peli.kasittele(Syote::Ammu);
        let lepo = peli.kuvan_askel();
        peli.edista(1);
        assert_eq!(peli.kuvan_askel(), lepo);
        peli.edista(VILKKU_MS);
        assert_ne!(peli.kuvan_askel(), lepo);

        peli.rajahdys = Some(peli.kello_ms);
        let rajahdys = peli.kuvan_askel();
        peli.edista(1);
        assert_ne!(peli.kuvan_askel(), rajahdys);
    }
}
