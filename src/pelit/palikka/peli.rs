use klassikot8000::Arpa;
use klassikot8000::palikka::{Paikka, Palikkalauta, SIVU, Vaihto, vierekkain};

use super::piirto;
use crate::yhteinen::aanet::{Aanet, Aani, PUHE_VIIVE_MS};
use crate::yhteinen::kieli::{kaanna, muotoile};
use crate::yhteinen::pelinakyma::Pelinakyma;
use crate::yhteinen::piirto::{Kehys, VILKKU_MS, aanikytkimella};
use crate::yhteinen::syote::{Syote, Yleinen, yleinen};
use crate::yhteinen::tulokset::{Nettipeli, Tulostaulu, muokkaa_nimea};

pub const VAIHTO_MS: u32 = 130;
pub const POKSAHDUS_MS: u32 = 170;
pub const PUTOAMINEN_MS: u32 = 190;
pub const HUTI_MS: u32 = 220;
const VIHJEEN_ODOTUS_MS: u32 = 8000;
const LOPPU_VIIVE_MS: u32 = 250;
const HELINAT: [Aani; 6] = [
    Aani::Helina1,
    Aani::Helina2,
    Aani::Helina3,
    Aani::Helina4,
    Aani::Helina5,
    Aani::Helina6,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vaihe {
    Alku,
    Pelissa,
    /// Esc pysäytti pelin: toinen Esc palaa valikkoon, muu jatkaa.
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

/// Toistettava liike. Logiikka on jo valmis, joten syöte odottaa toiston loppua.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Animaatio {
    Vaihto {
        vaihto: Vaihto,
        alku: u32,
    },
    /// Vaihto ei tuottanut suoraa: palikat käyvät puolimatkassa ja palaavat.
    Huti {
        a: Paikka,
        b: Paikka,
        alku: u32,
    },
}

impl Animaatio {
    fn loppu(&self) -> u32 {
        match self {
            Animaatio::Vaihto { vaihto, alku } => {
                alku + vaihdon_kesto(vaihto.kierrokset.len() as u32)
            }
            Animaatio::Huti { alku, .. } => alku + HUTI_MS,
        }
    }
}

/// Kohta vaihdon toistossa: osuus on 0..1 kyseisen vaiheen sisällä.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kohta {
    Vaihdetaan(f32),
    Poksuu(usize, f32),
    Putoaa(usize, f32),
}

fn vaihdon_kesto(kierroksia: u32) -> u32 {
    VAIHTO_MS + kierroksia * (POKSAHDUS_MS + PUTOAMINEN_MS)
}

/// Mikä vaihe toistosta on menossa, kun alusta on kulunut `kulunut` millisekuntia.
pub fn kohta(kulunut: u32, kierroksia: usize) -> Kohta {
    if kulunut < VAIHTO_MS {
        return Kohta::Vaihdetaan(kulunut as f32 / VAIHTO_MS as f32);
    }
    let kierros = POKSAHDUS_MS + PUTOAMINEN_MS;
    let (i, sisalla) = (
        (((kulunut - VAIHTO_MS) / kierros) as usize).min(kierroksia.saturating_sub(1)),
        kulunut - VAIHTO_MS,
    );
    let sisalla = sisalla.saturating_sub(i as u32 * kierros);
    if sisalla < POKSAHDUS_MS {
        Kohta::Poksuu(i, sisalla as f32 / POKSAHDUS_MS as f32)
    } else {
        let osuus = (sisalla - POKSAHDUS_MS) as f32 / PUTOAMINEN_MS as f32;
        Kohta::Putoaa(i, osuus.min(1.0))
    }
}

pub struct Palikkapeli {
    pub vaihe: Vaihe,
    pub lauta: Palikkalauta,
    pub taulu: Tulostaulu,
    /// Kirjoitettavana oleva nimimerkki.
    pub nimi: String,
    /// Tosi, kun tulos odottaa tiedostoon kirjoitusta.
    pub uusi_tulos: bool,
    pub viesti: String,
    pub valittu: Option<Paikka>,
    pub kursori: Paikka,
    /// Kursori näkyy vasta, kun näppäimistöä on käytetty.
    pub kursori_nakyy: bool,
    pub hiiren_ruutu: Option<Paikka>,
    pub animaatio: Option<Animaatio>,
    /// Pyyntö palata käynnistimeen.
    pub lopeta: bool,
    pub aanet: Aanet,
    pub kello_ms: u32,
    /// Palikka, josta hiirellä raahataan.
    raahaus: Option<Paikka>,
    /// Viimeisin pelaajan siirto tai toiston loppu: vihje tulee, kun tästä on kulunut aikaa.
    viime_toimi_ms: u32,
    /// Pelaaja poistuu kesken pelin: nimen jälkeen palataan valikkoon.
    poistuu: bool,
    arpa: Arpa,
}

impl Palikkapeli {
    pub fn new(siemen: u64) -> Self {
        let mut arpa = Arpa::new(siemen);
        Self {
            vaihe: Vaihe::Alku,
            lauta: Palikkalauta::new(&mut arpa),
            taulu: Tulostaulu::new(Nettipeli::Palikka),
            nimi: String::new(),
            uusi_tulos: false,
            viesti: String::new(),
            valittu: None,
            kursori: (SIVU / 2, SIVU / 2),
            kursori_nakyy: false,
            hiiren_ruutu: None,
            animaatio: None,
            lopeta: false,
            aanet: Aanet::default(),
            kello_ms: 0,
            raahaus: None,
            viime_toimi_ms: 0,
            poistuu: false,
            arpa,
        }
    }

    /// Kuva riippuu tästä ja syötteestä. `None`, kun alkuruudun malli pyörii.
    /// Paikallaan oleva lauta muuttuu vain vilkun tahdissa, joten sitä ei piirretä turhaan.
    pub fn kuvan_askel(&self) -> Option<u32> {
        match self.vaihe {
            Vaihe::Alku => None,
            _ if self.animaatio.is_some() => Some(self.kello_ms),
            _ => Some(self.kello_ms / VILKKU_MS),
        }
    }

    pub fn animoi(&self) -> bool {
        self.animaatio.is_some()
    }

    /// Siirto, joka näytetään, kun pelaaja on miettinyt pitkään.
    pub fn vihje(&self) -> Option<(Paikka, Paikka)> {
        let odotti = self.kello_ms.saturating_sub(self.viime_toimi_ms) >= VIHJEEN_ODOTUS_MS;
        if self.vaihe == Vaihe::Pelissa && self.animaatio.is_none() && odotti {
            self.lauta.vihje()
        } else {
            None
        }
    }

    pub fn edista(&mut self, ms: u32) {
        self.kello_ms = self.kello_ms.saturating_add(ms);
        if self
            .animaatio
            .as_ref()
            .is_some_and(|a| self.kello_ms >= a.loppu())
        {
            self.animaatio = None;
            self.viime_toimi_ms = self.kello_ms;
        }
        // Loppu odottaa toiston loppua, jotta viimeinen ketju näkyy kokonaan.
        if self.vaihe == Vaihe::Pelissa && self.animaatio.is_none() && self.lauta.ohi {
            self.aanet.jono.push((Aani::Tappio, LOPPU_VIIVE_MS));
            self.aanet
                .jono
                .push((Aani::EiSiirtoja, LOPPU_VIIVE_MS + PUHE_VIIVE_MS));
            self.lopeta_peli();
        }
    }

    fn lopeta_peli(&mut self) {
        self.valittu = None;
        self.vaihe = if self.taulu.mahtuu(self.lauta.pisteet) {
            self.nimi.clear();
            Vaihe::Nimi
        } else {
            Vaihe::Tappio
        };
        self.uusi_tulos = true;
        self.viesti = muotoile("palikka.ei_siirtoja", &[&self.lauta.pisteet]);
    }

    /// Kirjaa odottavan tuloksen.
    pub fn vahvista_nimi(&mut self) {
        if self.vaihe != Vaihe::Nimi {
            return;
        }
        let pisteet = self.lauta.pisteet;
        self.taulu.kirjaa(&mut self.nimi, pisteet);
        self.uusi_tulos = true;
        self.vaihe = Vaihe::Tappio;
        self.lopeta = self.poistuu;
    }

    /// Esc tai ikkunan sulkeminen kesken pelin: listalle yltävä tulos ei saa kadota.
    pub fn poistu(&mut self) {
        if matches!(self.vaihe, Vaihe::Pelissa | Vaihe::Tauko)
            && self.taulu.mahtuu(self.lauta.pisteet)
        {
            self.nimi.clear();
            self.poistuu = true;
            self.vaihe = Vaihe::Nimi;
        } else {
            self.lopeta = true;
        }
    }

    fn kasittele_nimi(&mut self, syote: Syote) {
        match syote {
            // Viimeisenä painettu välilyönti ei saa kuitata nimeä ennen kuin sitä on kirjoitettu.
            Syote::Ammu if self.nimi.is_empty() => {}
            Syote::Ammu | Syote::Peru => self.vahvista_nimi(),
            _ => muokkaa_nimea(&mut self.nimi, syote),
        }
    }

    fn aloita_alusta(&mut self) {
        self.lauta = Palikkalauta::new(&mut self.arpa);
        self.vaihe = Vaihe::Pelissa;
        self.viesti.clear();
        self.valittu = None;
        self.kursori_nakyy = false;
        self.animaatio = None;
        self.raahaus = None;
        self.viime_toimi_ms = self.kello_ms;
        self.aanet.jono.push((Aani::Alku, 0));
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
        if let Syote::Hiiri(x, y) = syote {
            self.hiiren_ruutu = piirto::ruutu_pisteesta(x, y);
            self.kursori_nakyy = false;
        }
        if matches!(syote, Syote::Vapautus(..)) {
            self.raahaus = None;
        }
        if self.animoi() {
            return;
        }
        self.viime_toimi_ms = self.kello_ms;
        match syote {
            Syote::Klikkaus(x, y) => match piirto::ruutu_pisteesta(x, y) {
                Some(ruutu) => {
                    self.kursori_nakyy = false;
                    self.valitse_tai_vaihda(ruutu);
                    self.raahaus = self.valittu;
                }
                None => self.valittu = None,
            },
            Syote::Hiiri(..) => {
                if let (Some(alku), Some(ruutu)) = (self.raahaus, self.hiiren_ruutu)
                    && vierekkain(alku, ruutu)
                {
                    self.raahaus = None;
                    self.vaihda(alku, ruutu);
                }
            }
            Syote::Nuoli(drivi, dsarake) => self.nuoli(drivi, dsarake),
            Syote::Ammu => {
                if self.kursori_nakyy {
                    self.valitse_tai_vaihda(self.kursori);
                } else {
                    self.kursori_nakyy = true;
                }
            }
            _ => {}
        }
    }

    fn nuoli(&mut self, drivi: i8, dsarake: i8) {
        let alku = self.valittu.unwrap_or(self.kursori);
        let rivi = alku.0.saturating_add_signed(drivi as isize).min(SIVU - 1);
        let sarake = alku.1.saturating_add_signed(dsarake as isize).min(SIVU - 1);
        let kohde = (rivi, sarake);
        if self.kursori_nakyy && self.valittu.is_some() && kohde != alku {
            self.kursori = kohde;
            self.vaihda(alku, kohde);
        } else if self.kursori_nakyy {
            self.kursori = kohde;
        } else if let Some(valittu) = self.valittu {
            self.kursori = valittu;
        }
        self.kursori_nakyy = true;
    }

    fn valitse_tai_vaihda(&mut self, ruutu: Paikka) {
        match self.valittu {
            Some(alku) if alku == ruutu => self.valittu = None,
            Some(alku) if vierekkain(alku, ruutu) => self.vaihda(alku, ruutu),
            _ => self.valittu = Some(ruutu),
        }
    }

    /// Yrittää vaihtaa kaksi naapuripalikkaa ja aloittaa toiston.
    pub fn vaihda(&mut self, a: Paikka, b: Paikka) {
        self.valittu = None;
        let alku = self.kello_ms;
        let Some(vaihto) = self.lauta.vaihda(a, b, &mut self.arpa) else {
            self.aanet.jono.push((Aani::Virhe, 0));
            self.viesti = kaanna("palikka.huono").to_string();
            self.animaatio = Some(Animaatio::Huti { a, b, alku });
            return;
        };
        self.taulu.paras = self.taulu.paras.max(self.lauta.pisteet);
        self.aanet.jono.push((Aani::Liuku, 0));
        for (i, kierros) in vaihto.kierrokset.iter().enumerate() {
            let viive = VAIHTO_MS + i as u32 * (POKSAHDUS_MS + PUTOAMINEN_MS);
            self.aanet
                .jono
                .push((HELINAT[i.min(HELINAT.len() - 1)], viive));
            if !kierros.rajahdykset.is_empty() {
                self.aanet.jono.push((Aani::Rajahdys, viive));
            }
            if !kierros.luodut.is_empty() {
                self.aanet.jono.push((Aani::Yhdistys, viive));
            }
        }
        let ketju = vaihto.kierrokset.len();
        self.viesti = if vaihto.uusi_taso {
            let viive = vaihdon_kesto(ketju as u32);
            self.aanet.jono.push((Aani::Alku, viive));
            muotoile("palikka.uusi_taso", &[&self.lauta.taso])
        } else if ketju > 1 {
            muotoile("palikka.ketju", &[&ketju, &vaihto.pisteet])
        } else {
            muotoile("palikka.siirto", &[&vaihto.pisteet])
        };
        self.animaatio = Some(Animaatio::Vaihto { vaihto, alku });
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

impl Pelinakyma for Palikkapeli {
    fn kasittele(&mut self, syote: Syote) {
        Palikkapeli::kasittele(self, syote);
    }

    fn edista(&mut self, ms: u32) {
        Palikkapeli::edista(self, ms);
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

    fn kuvan_avain(&self) -> Option<(u32, u32)> {
        self.kuvan_askel().map(|askel| (askel, 0))
    }

    /// Palikat liikkuvat vain vaihdon jälkeen, muuten lauta seisoo.
    fn toimintapeli(&self) -> bool {
        self.animoi()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yhteinen::tulokset::Tulos;
    use klassikot8000::palikka::{Palikka, Ruudut};

    /// Lauta, jossa (2,2) ja (3,2) vaihtamalla syntyy kolmen suora.
    fn yksi_siirto() -> Ruudut {
        let mut ruudut: Ruudut = [[None; SIVU]; SIVU];
        for (r, rivi) in ruudut.iter_mut().enumerate() {
            for (s, ruutu) in rivi.iter_mut().enumerate() {
                let vari = (2 + (r % 3) * 2 + s % 2 + (s / 2) % 2 * 3) % 6;
                *ruutu = Some(Palikka::Tavallinen(vari as u8));
            }
        }
        for (r, s) in [(2, 0), (2, 1), (3, 2)] {
            ruudut[r][s] = Some(Palikka::Tavallinen(0));
        }
        ruudut
    }

    fn pelissa() -> Palikkapeli {
        let mut peli = Palikkapeli::new(1);
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.vaihe, Vaihe::Pelissa);
        peli.aanet.jono.clear();
        peli.lauta = Palikkalauta::ruuduista(yksi_siirto());
        peli
    }

    fn klikkaa(peli: &mut Palikkapeli, ruutu: Paikka) {
        let (x, y) = piirto::ruudun_keskipiste(ruutu);
        peli.kasittele(Syote::Klikkaus(x, y));
        peli.kasittele(Syote::Vapautus(x, y));
    }

    /// Lauta, jossa ei ole siirtoja. Peli päättyy, kun toisto on ohi.
    fn jumissa(peli: &mut Palikkapeli, pisteet: u32) {
        peli.lauta.pisteet = pisteet;
        peli.lauta.ohi = true;
    }

    #[test]
    fn klikkaus_valitsee_ja_naapurin_klikkaus_vaihtaa() {
        let mut peli = pelissa();
        klikkaa(&mut peli, (2, 2));
        assert_eq!(peli.valittu, Some((2, 2)));
        klikkaa(&mut peli, (2, 2));
        assert_eq!(peli.valittu, None);

        klikkaa(&mut peli, (2, 2));
        klikkaa(&mut peli, (5, 5));
        assert_eq!(peli.valittu, Some((5, 5)));
        klikkaa(&mut peli, (2, 2));
        klikkaa(&mut peli, (3, 2));
        assert!(peli.lauta.pisteet >= 30);
        assert_eq!(peli.valittu, None);
        assert!(peli.animoi());
        assert_eq!(peli.taulu.paras, peli.lauta.pisteet);
        assert_eq!(peli.aanet.jono[0], (Aani::Liuku, 0));
        assert_eq!(peli.aanet.jono[1], (Aani::Helina1, VAIHTO_MS));
        assert!(peli.viesti.contains('+'));
    }

    #[test]
    fn raahaus_naapuriin_vaihtaa_ja_vapautus_perii_raahauksen() {
        let mut peli = pelissa();
        let (x, y) = piirto::ruudun_keskipiste((2, 2));
        peli.kasittele(Syote::Klikkaus(x, y));
        let (x2, y2) = piirto::ruudun_keskipiste((3, 2));
        peli.kasittele(Syote::Hiiri(x2, y2));
        assert!(peli.lauta.pisteet >= 30);

        let mut peli = pelissa();
        peli.kasittele(Syote::Klikkaus(x, y));
        peli.kasittele(Syote::Vapautus(x, y));
        peli.kasittele(Syote::Hiiri(x2, y2));
        assert_eq!(peli.lauta.pisteet, 0);
        assert_eq!(peli.valittu, Some((2, 2)));
    }

    #[test]
    fn hyodyton_vaihto_ahkaisee_ja_palaa() {
        let mut peli = pelissa();
        let lauta = peli.lauta.clone();
        klikkaa(&mut peli, (6, 6));
        klikkaa(&mut peli, (6, 7));
        assert_eq!(peli.lauta, lauta);
        assert_eq!(peli.aanet.jono, [(Aani::Virhe, 0)]);
        assert!(matches!(peli.animaatio, Some(Animaatio::Huti { .. })));
        peli.edista(HUTI_MS);
        assert!(!peli.animoi());
    }

    #[test]
    fn syote_odottaa_toiston_loppua() {
        let mut peli = pelissa();
        klikkaa(&mut peli, (2, 2));
        klikkaa(&mut peli, (3, 2));
        assert!(peli.animoi());
        klikkaa(&mut peli, (5, 5));
        assert_eq!(peli.valittu, None);
        // Pudonneet palikat voivat sattua lukitsemaan laudan, mutta tässä peli jatkuu.
        peli.lauta.ohi = false;
        peli.edista(100_000);
        assert!(!peli.animoi());
        klikkaa(&mut peli, (5, 5));
        assert_eq!(peli.valittu, Some((5, 5)));
    }

    #[test]
    fn nappaimisto_nayttaa_kursorin_valitsee_ja_vaihtaa() {
        let mut peli = pelissa();
        // Ensimmäinen painallus vain tuo kursorin näkyviin.
        peli.kasittele(Syote::Nuoli(0, 1));
        assert!(peli.kursori_nakyy);
        assert_eq!(peli.kursori, (4, 4));
        peli.kasittele(Syote::Nuoli(-1, 0));
        peli.kasittele(Syote::Nuoli(-1, 0));
        peli.kasittele(Syote::Nuoli(0, -1));
        peli.kasittele(Syote::Nuoli(0, -1));
        assert_eq!(peli.kursori, (2, 2));
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.valittu, Some((2, 2)));
        peli.kasittele(Syote::Nuoli(1, 0));
        assert_eq!(peli.kursori, (3, 2));
        assert!(peli.lauta.pisteet >= 30);
        assert_eq!(peli.valittu, None);

        // Laudan reuna pysäyttää kursorin.
        let mut peli = pelissa();
        peli.kursori_nakyy = true;
        peli.kursori = (0, 0);
        peli.kasittele(Syote::Nuoli(-1, 0));
        peli.kasittele(Syote::Nuoli(0, -1));
        assert_eq!(peli.kursori, (0, 0));
    }

    #[test]
    fn valilyonti_ilman_kursoria_ei_valitse_vaan_nayttaa_sen() {
        let mut peli = pelissa();
        peli.kasittele(Syote::Ammu);
        assert!(peli.kursori_nakyy);
        assert_eq!(peli.valittu, None);
        peli.kasittele(Syote::Hiiri(0, 0));
        assert!(!peli.kursori_nakyy);
    }

    #[test]
    fn vihje_tulee_odotuksen_jalkeen_ja_syote_nollaa_sen() {
        let mut peli = pelissa();
        assert_eq!(peli.vihje(), None);
        peli.edista(VIHJEEN_ODOTUS_MS);
        assert_eq!(peli.vihje(), peli.lauta.vihje());
        assert!(peli.vihje().is_some());
        peli.kasittele(Syote::Hiiri(1, 1));
        assert_eq!(peli.vihje(), None);
    }

    #[test]
    fn toiston_kohdat_seuraavat_toisiaan() {
        assert!(matches!(kohta(0, 2), Kohta::Vaihdetaan(_)));
        assert!(matches!(kohta(VAIHTO_MS, 2), Kohta::Poksuu(0, _)));
        assert!(matches!(
            kohta(VAIHTO_MS + POKSAHDUS_MS, 2),
            Kohta::Putoaa(0, _)
        ));
        let toinen = VAIHTO_MS + POKSAHDUS_MS + PUTOAMINEN_MS;
        assert_eq!(kohta(toinen, 2), Kohta::Poksuu(1, 0.0));
        // Viimeisen kuvan jälkeen pysytään viimeisessä kierroksessa.
        assert_eq!(kohta(100_000, 2), Kohta::Putoaa(1, 1.0));
    }

    #[test]
    fn kuva_paivittyy_toistossa_mutta_ei_levossa() {
        let mut peli = Palikkapeli::new(1);
        assert_eq!(peli.kuvan_askel(), None);
        peli.kasittele(Syote::Ammu);
        peli.lauta = Palikkalauta::ruuduista(yksi_siirto());
        let lepo = peli.kuvan_askel();
        peli.edista(1);
        assert_eq!(peli.kuvan_askel(), lepo);
        peli.vaihda((2, 2), (3, 2));
        let toisto = peli.kuvan_askel();
        peli.edista(1);
        assert_ne!(peli.kuvan_askel(), toisto);
    }

    #[test]
    fn jumi_paattaa_pelin_vasta_toiston_jalkeen_ja_kysyy_nimen() {
        let mut peli = pelissa();
        klikkaa(&mut peli, (2, 2));
        klikkaa(&mut peli, (3, 2));
        jumissa(&mut peli, 40);
        peli.edista(1);
        assert_eq!(peli.vaihe, Vaihe::Pelissa);
        peli.edista(100_000);
        assert_eq!(peli.vaihe, Vaihe::Nimi);
        assert!(peli.uusi_tulos);

        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.vaihe, Vaihe::Nimi);
        peli.kasittele(Syote::Kirjain('K'));
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.vaihe, Vaihe::Tappio);
        assert_eq!(peli.taulu.parhaat[0].nimi, "K");
        assert_eq!(peli.taulu.lahetys.as_ref().unwrap().pisteet, 40);
    }

    #[test]
    fn listalle_yltamaton_loppu_ja_uusi_peli() {
        let mut peli = pelissa();
        peli.taulu.netti = Some(vec![
            Tulos {
                nimi: "A".to_string(),
                pisteet: 1000,
            };
            3
        ]);
        jumissa(&mut peli, 40);
        peli.taulu.paras = 40;
        peli.edista(1);
        assert_eq!(peli.vaihe, Vaihe::Tappio);
        assert_eq!(peli.taulu.lahetys, None);

        peli.kasittele(Syote::Kirjain('K'));
        assert_eq!(peli.vaihe, Vaihe::Pelissa);
        assert_eq!(peli.lauta.pisteet, 0);
        assert_eq!(peli.taulu.paras, 40);
    }

    #[test]
    fn esc_kesken_pelin_pysayttaa_ja_toinen_kysyy_nimen() {
        let mut peli = pelissa();
        peli.lauta.pisteet = 500;
        peli.kasittele(Syote::Peru);
        assert_eq!(peli.vaihe, Vaihe::Tauko);
        // Tauolla lauta ei ota siirtoja, ja klikkaus jatkaa peliä.
        peli.kasittele(Syote::Klikkaus(0, 0));
        assert_eq!(peli.vaihe, Vaihe::Pelissa);
        peli.kasittele(Syote::Peru);
        peli.kasittele(Syote::Nuoli(0, 1));
        assert_eq!(peli.vaihe, Vaihe::Tauko);
        assert!(!peli.kursori_nakyy);
        peli.kasittele(Syote::Peru);
        assert_eq!(peli.vaihe, Vaihe::Nimi);
        assert!(!peli.lopeta);
        peli.kasittele(Syote::Kirjain('B'));
        peli.kasittele(Syote::Ammu);
        assert!(peli.lopeta);
        assert_eq!(peli.taulu.parhaat[0].nimi, "B");

        // Ilman pisteitä toinen Esc poistuu heti.
        let mut peli = pelissa();
        peli.kasittele(Syote::Peru);
        assert!(!peli.lopeta);
        peli.kasittele(Syote::Peru);
        assert!(peli.lopeta);
        assert_eq!(peli.taulu.lahetys, None);
    }

    #[test]
    fn lopun_napit_toimivat() {
        let mut peli = Palikkapeli::new(1);
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
