use klassikot8000::Arpa;
use klassikot8000::holdem::{ALKURAHA, Kulku, PELAAJA, PELAAJIA, Poyta, Toimi};

use super::piirto::{self, kaden_nimi, pelaajan_nimi};
use crate::yhteinen::aanet::Aani;
use crate::yhteinen::kieli::{kaanna, muotoile};
use crate::yhteinen::piirto::VILKKU_MS;
use crate::yhteinen::syote::Syote;
use crate::yhteinen::tulokset::{Tulos, lisaa_tulos, mahtuu_parhaisiin, muokkaa_nimea, ota_nimi};

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
    pub parhaat: Vec<Tulos>,
    /// Nettilista. `None`, kunnes palvelin on vastannut.
    pub netti_parhaat: Option<Vec<Tulos>>,
    /// Kirjattu tulos, joka odottaa lähetystä nettilistalle.
    pub lahetys: Option<Tulos>,
    /// Kirjoitettavana oleva nimimerkki.
    pub nimi: String,
    /// Tosi, kun tulos odottaa tiedostoon kirjoitusta.
    pub uusi_tulos: bool,
    /// Pelaaja poistuu kesken pelin: nimen jälkeen palataan valikkoon.
    poistuu: bool,
    pub viesti: String,
    pub lopeta: bool,
    pub aanet_paalla: bool,
    pub aanijono: Vec<(Aani, u32)>,
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
            parhaat: Vec::new(),
            netti_parhaat: None,
            lahetys: None,
            nimi: String::new(),
            uusi_tulos: false,
            poistuu: false,
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
            Vaihe::Pelissa | Vaihe::Havio | Vaihe::Voitto => Some(self.siirtoja),
        }
    }

    /// Esillä oleva lista: nettilista, tai oma lista, jos verkkoon ei ole saatu yhteyttä.
    pub fn nakyva_lista(&self) -> &[Tulos] {
        self.netti_parhaat.as_deref().unwrap_or(&self.parhaat)
    }

    /// Pöydästä lähtö lunastaa voiton. Pöydässä oleva panos jää sinne.
    pub fn poistu(&mut self) {
        if self.vaihe == Vaihe::Pelissa
            && mahtuu_parhaisiin(self.nakyva_lista(), self.poyta.pisteet())
        {
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
        let nimi = ota_nimi(&mut self.nimi);
        let pisteet = self.poyta.pisteet();
        let tulos = Tulos { nimi, pisteet };
        lisaa_tulos(&mut self.parhaat, tulos.clone());
        if let Some(lista) = &mut self.netti_parhaat {
            lisaa_tulos(lista, tulos.clone());
        }
        self.lahetys = Some(tulos);
        self.uusi_tulos = true;
        self.vaihe = Vaihe::Voitto;
        self.lopeta = self.poistuu;
    }

    fn kasittele_nimi(&mut self, syote: Syote) {
        match syote {
            // Viimeisenä painettu välilyönti ei saa kuitata nimeä ennen kuin sitä on kirjoitettu.
            Syote::Ammu if self.nimi.is_empty() => {}
            Syote::Ammu | Syote::Peru => self.vahvista_nimi(),
            _ => muokkaa_nimea(&mut self.nimi, syote),
        }
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
        self.aanijono.push((Aani::Liuku, 0));
        self.tila_muuttui();
    }

    fn aloita(&mut self) {
        self.poyta = Poyta::new(&mut self.arpa);
        self.vaihe = Vaihe::Pelissa;
        self.aanijono.push((Aani::Ovi, 0));
        self.uusi_jako();
    }

    fn uusi_jako(&mut self) {
        self.poyta.jaa(&mut self.arpa);
        self.seuraava_siirto_ms = self.kello_ms.saturating_add(ASKEL_MS);
        self.aanijono.push((Aani::Liuku, 0));
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
            self.aanijono.push((Aani::Tappio, LOPPU_VIIVE_MS));
        } else if self.poyta.rahoissa() == 1 {
            self.vaihe = Vaihe::Voitto;
            let kaikki = ALKURAHA * PELAAJIA as u32;
            self.viesti = muotoile("holdem.voitto", &[&kaikki]);
            self.aanijono.push((Aani::Voitto, LOPPU_VIIVE_MS));
            if mahtuu_parhaisiin(self.nakyva_lista(), self.poyta.pisteet()) {
                self.nimi.clear();
                self.vaihe = Vaihe::Nimi;
            }
        } else if voitti {
            self.aanijono.push((Aani::Yhdistys, LOPPU_VIIVE_MS));
        } else {
            self.aanijono.push((Aani::Pomppu, LOPPU_VIIVE_MS));
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
            self.aanijono.push((Aani::Ahkaisy, 0));
        }
    }

    fn tee(&mut self, toimi: Toimi) {
        if self.poyta.toimi(toimi) {
            self.siirtoja = self.siirtoja.wrapping_add(1);
            self.seuraava_siirto_ms = self.kello_ms.saturating_add(ASKEL_MS);
            self.aanijono.push((Aani::Liuku, 0));
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

#[cfg(test)]
mod tests {
    use super::*;
    use klassikot8000::holdem::{Kortti, Maa, Tila, paras_kasi};

    /// Peli, jossa on pelaajan vuoro. Pelaajalla on oikea pino.
    fn omalla_vuorolla(siemen: u64) -> Holdem {
        let mut peli = Holdem::new(siemen);
        peli.kasittele(Syote::Ammu);
        peli.edista(1_000_000);
        // Kierrokset, joilla pelaaja ei pääse toimimaan, jatkuvat uudella jaolla.
        while peli.napisto() != Napisto::Vuoro {
            assert_eq!(peli.vaihe, Vaihe::Pelissa, "siemen {siemen}");
            peli.kasittele(Syote::Ammu);
            peli.edista(1_000_000);
        }
        peli
    }

    #[test]
    fn alkuruutu_aloittaa_pelin_ja_kaikilla_on_tuhat() {
        let mut peli = Holdem::new(1);
        assert_eq!(peli.vaihe, Vaihe::Alku);
        assert_eq!(peli.kuvan_askel(), None);
        assert!(peli.poyta.pelaajat.iter().all(|p| p.rahat == ALKURAHA));
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.vaihe, Vaihe::Pelissa);
        assert_eq!(peli.poyta.jako, 1);
        assert_eq!(
            peli.poyta
                .pelaajat
                .iter()
                .filter(|p| p.kortit.len() == 2)
                .count(),
            5
        );
    }

    #[test]
    fn koneet_pelaavat_ajan_kuluessa_ja_valilyonti_hoputtaa() {
        let mut peli = Holdem::new(5);
        peli.kasittele(Syote::Ammu);
        // Etsitään jako, jossa kone on ensin vuorossa.
        while peli.napisto() != Napisto::Odotus {
            peli.kasittele(Syote::Kirjain('K'));
            peli.edista(1_000_000);
            peli.kasittele(Syote::Ammu);
        }
        let ennen = peli.kuvan_askel();
        peli.edista(ASKEL_MS - 1);
        assert_eq!(peli.kuvan_askel(), ennen);
        peli.edista(1);
        assert_ne!(peli.kuvan_askel(), ennen);

        if peli.napisto() == Napisto::Odotus {
            let ennen = peli.kuvan_askel();
            peli.kasittele(Syote::Ammu);
            assert_ne!(peli.kuvan_askel(), ennen);
        }
    }

    #[test]
    fn levossa_kuva_ei_muutu_omalla_vuorolla() {
        let mut peli = omalla_vuorolla(1);
        let lepo = peli.kuvan_askel();
        peli.edista(600_000);
        assert_eq!(peli.kuvan_askel(), lepo);
        assert_eq!(peli.napisto(), Napisto::Vuoro);
    }

    #[test]
    fn valilyonti_ei_koskaan_maksa_omalla_vuorolla() {
        let mut peli = omalla_vuorolla(1);
        let rahat = peli.poyta.pelaajat[PELAAJA].rahat;
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.poyta.pelaajat[PELAAJA].rahat, rahat);
        assert_eq!(peli.napisto(), Napisto::Vuoro);
    }

    #[test]
    fn korotus_pysyy_rajoissa_ja_napit_asettavat_sen() {
        let mut peli = omalla_vuorolla(1);
        let (pienin, suurin) = (
            peli.poyta.pienin_taso(PELAAJA),
            peli.poyta.suurin_taso(PELAAJA),
        );
        let iso = peli.poyta.blindit().1;
        assert_eq!(peli.korotus, pienin);
        peli.kasittele(Syote::Nuoli(0, -1));
        assert_eq!(peli.korotus, pienin);
        peli.kasittele(Syote::Nuoli(0, 1));
        assert_eq!(peli.korotus, pienin + iso);
        peli.kasittele(Syote::Nuoli(-1, 0));
        assert_eq!(peli.korotus, pienin + 6 * iso);
        peli.kasittele(Syote::Nuoli(1, 0));
        assert_eq!(peli.korotus, pienin + iso);
        peli.kasittele(Syote::Numero(4));
        assert_eq!(peli.korotus, suurin);
        peli.kasittele(Syote::Nuoli(0, 1));
        assert_eq!(peli.korotus, suurin);
        peli.kasittele(Syote::Numero(1));
        assert_eq!(peli.korotus, pienin);
        peli.kasittele(Syote::Numero(3));
        let potti = peli.poyta.korkein + peli.poyta.potti() + peli.poyta.maksu(PELAAJA);
        assert_eq!(peli.korotus, potti.clamp(pienin, suurin));
        let (x, y) = piirto::napin_keskipiste(Napisto::Vuoro, Nappi::Kaikki);
        peli.kasittele(Syote::Klikkaus(x, y));
        assert_eq!(peli.korotus, suurin);
        // asettaminen ei vielä korota
        assert_eq!(peli.napisto(), Napisto::Vuoro);
    }

    #[test]
    fn kippaus_maksu_ja_korotus_toimivat_nappaimilla() {
        let mut peli = omalla_vuorolla(1);
        peli.kasittele(Syote::Kirjain('K'));
        assert_eq!(peli.poyta.pelaajat[PELAAJA].tila, Tila::Kippasi);

        let mut peli = omalla_vuorolla(1);
        let rahat = peli.poyta.pelaajat[PELAAJA].rahat;
        let maksu = peli.poyta.maksu(PELAAJA);
        peli.kasittele(Syote::Kirjain('V'));
        assert_eq!(peli.poyta.pelaajat[PELAAJA].rahat, rahat - maksu);

        let mut peli = omalla_vuorolla(1);
        let taso = peli.korotus;
        peli.kasittele(Syote::Kirjain('R'));
        assert_eq!(peli.poyta.korkein, taso);
        assert_eq!(peli.poyta.viime_hyokkaaja, Some(PELAAJA));
    }

    #[test]
    fn jakojen_valissa_vali_jakaa_ja_tulos_nakyy_viestissa() {
        let mut peli = omalla_vuorolla(2);
        peli.kasittele(Syote::Kirjain('K'));
        peli.edista(1_000_000);
        assert_eq!(peli.napisto(), Napisto::JakoOhi);
        assert!(peli.viesti.contains('+'), "{}", peli.viesti);
        let jako = peli.poyta.jako;
        peli.edista(1_000_000);
        assert_eq!(peli.poyta.jako, jako);
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.poyta.jako, jako + 1);
    }

    /// Asettaa voitot (voitto, josta pääpotista) ja kaikille saman näytetyn käden.
    fn tulos(voitot: [(u32, u32); PELAAJIA]) -> String {
        let mut peli = Holdem::new(1);
        let kortit = [2, 3, 4, 5, 6].map(|arvo| Kortti {
            maa: Maa::Pata,
            arvo,
        });
        for (pelaaja, (voitto, paapotista)) in peli.poyta.pelaajat.iter_mut().zip(voitot) {
            pelaaja.voitto = voitto;
            pelaaja.paapotista = paapotista;
            pelaaja.kasi = paras_kasi(&kortit);
        }
        peli.tulosviesti()
    }

    #[test]
    fn tulos_kertoo_yhden_voittajan_tasakaden_ja_sivupotin_eri_tavoin() {
        let kasi = kaanna("holdem.kasi_varisuora");
        let nimi = pelaajan_nimi;
        assert_eq!(
            tulos([(50, 50), (0, 0), (0, 0), (0, 0), (0, 0)]),
            muotoile("holdem.voitot_kasi", &[&format!("{} +50", nimi(0)), &kasi])
        );
        // Sama voittaja kummassakin potissa on yhä yksi voittaja.
        assert_eq!(
            tulos([(70, 50), (0, 0), (0, 0), (0, 0), (0, 0)]),
            muotoile("holdem.voitot_kasi", &[&format!("{} +70", nimi(0)), &kasi])
        );
        assert_eq!(
            tulos([(25, 25), (0, 0), (25, 25), (0, 0), (0, 0)]),
            muotoile(
                "holdem.tasan",
                &[&format!("{}, {}", nimi(0), nimi(2)), &kasi]
            )
        );
        assert_eq!(
            tulos([(330, 330), (0, 0), (400, 0), (0, 0), (0, 0)]),
            muotoile(
                "holdem.sivupotti_tulos",
                &[&format!("{} +330", nimi(0)), &format!("{} +400", nimi(2))]
            )
        );
    }

    #[test]
    fn rahojen_loppuessa_peli_paattyy_ja_alkaa_alusta() {
        let mut peli = omalla_vuorolla(3);
        peli.poyta.pelaajat[PELAAJA].rahat = 0;
        peli.poyta.pelaajat[1].rahat = 0;
        peli.jako_paattyi();
        assert_eq!(peli.vaihe, Vaihe::Havio);
        assert!(peli.aanijono.contains(&(Aani::Tappio, LOPPU_VIIVE_MS)));
        // Kolme on yhä rahoissa, joten pelaajan sija on neljäs.
        assert_eq!(peli.viesti, muotoile("holdem.havio", &[&4]));
        peli.kasittele(Syote::Ammu);
        peli.kasittele(Syote::Kirjain('V'));
        assert_eq!(peli.vaihe, Vaihe::Havio);
        peli.kasittele(Syote::Kirjain('K'));
        assert_eq!(peli.vaihe, Vaihe::Pelissa);
        assert_eq!(peli.poyta.jako, 1);
        assert!(peli.poyta.pelaajat[PELAAJA].rahat > 0);
    }

    #[test]
    fn viimeisena_rahoissa_voittaa_pelin() {
        let mut peli = omalla_vuorolla(4);
        for i in 1..PELAAJIA {
            peli.poyta.pelaajat[i].rahat = 0;
        }
        peli.poyta.pelaajat[PELAAJA].rahat = ALKURAHA * PELAAJIA as u32;
        peli.jako_paattyi();
        assert_eq!(peli.vaihe, Vaihe::Nimi);
        assert!(peli.aanijono.contains(&(Aani::Voitto, LOPPU_VIIVE_MS)));
        assert_eq!(peli.viesti, muotoile("holdem.voitto", &[&5000]));
        peli.kasittele(Syote::Kirjain('K'));
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.vaihe, Vaihe::Voitto);
        assert!(!peli.lopeta);
        assert_eq!(peli.parhaat[0].pisteet, 4000);
        // Kertaalleen kirjattu voitto ei kirjaudu toista kertaa poistuttaessa.
        peli.lahetys = None;
        peli.kasittele(Syote::Peru);
        assert!(peli.lopeta);
        assert_eq!(peli.lahetys, None);

        peli.lopeta = false;
        peli.kasittele(Syote::Kirjain('K'));
        assert_eq!(peli.vaihe, Vaihe::Pelissa);
    }

    #[test]
    fn voitolla_poistuva_kirjoittaa_nimen_ja_palaa_valikkoon() {
        let mut peli = omalla_vuorolla(1);
        peli.poyta.pelaajat[PELAAJA].rahat = ALKURAHA + 300;
        peli.kasittele(Syote::Peru);
        assert_eq!(peli.vaihe, Vaihe::Nimi);
        // Koneet eivät pelaa nimen aikana, eikä K kippaa.
        let siirrot = peli.siirtoja;
        peli.edista(100_000);
        assert_eq!(peli.siirtoja, siirrot);
        peli.kasittele(Syote::Kirjain('K'));
        assert_eq!(peli.poyta.pelaajat[PELAAJA].tila, Tila::Mukana);
        peli.kasittele(Syote::Ammu);
        assert!(peli.lopeta);
        assert_eq!(peli.parhaat[0].nimi, "K");
        assert_eq!(peli.parhaat[0].pisteet, 300);
    }

    #[test]
    fn napit_ja_esc_palaavat_valikkoon() {
        let mut peli = omalla_vuorolla(1);
        let (x, y) = piirto::napin_keskipiste(Napisto::Vuoro, Nappi::Kippaa);
        peli.kasittele(Syote::Klikkaus(x, y));
        assert_eq!(peli.poyta.pelaajat[PELAAJA].tila, Tila::Kippasi);
        peli.kasittele(Syote::Peru);
        assert!(peli.lopeta);

        let mut peli = omalla_vuorolla(1);
        peli.kasittele(Syote::Kirjain('E'));
        assert!(peli.lopeta);
    }

    #[test]
    fn kokonainen_peli_paattyy_hoputtamalla_ja_kippaamalla() {
        let mut peli = Holdem::new(7);
        peli.kasittele(Syote::Ammu);
        for _ in 0..3000 {
            if peli.vaihe != Vaihe::Pelissa {
                return;
            }
            peli.kasittele(Syote::Kirjain('V'));
            peli.edista(10_000);
            peli.kasittele(Syote::Ammu);
        }
        panic!("peli ei päättynyt");
    }
}
