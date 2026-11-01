use klassikot8000::Arpa;
use klassikot8000::pasianssi::{Kohde, Lahde, Lauta, SARAKKEITA, Vaikeus, aikapisteet};

use super::piirto::{self, Kohta};
use crate::yhteinen::aanet::Aani;
use crate::yhteinen::kieli::{kaanna, muotoile};
use crate::yhteinen::piirto::VILKKU_MS;
use crate::yhteinen::syote::Syote;
use crate::yhteinen::tulokset::{Tulos, lisaa_tulos, mahtuu_parhaisiin, muokkaa_nimea, ota_nimi};

const TUPLAKLIKKAUS_MS: u32 = 400;
// Pienempi liike on klikkauksen tärinää eikä vetämistä.
const VETO_RAJA: i32 = 5;
const VOITTO_VIIVE_MS: u32 = 250;
// Ylärivin kolmas paikka on tyhjä: pakka ja jäte ovat vasemmalla, perustat oikealla.
const TYHJA_YLASARAKE: usize = 2;
const PERUSTA_ALKAA: usize = 3;

const EI_KAY: &str = "pasianssi.ei_kay";
const PAKKA_LOPPU: &str = "pasianssi.pakka_loppu";
const EI_PERUUTUSTA: &str = "pasianssi.ei_peruutusta";
const EI_PERUTTAVAA: &str = "pasianssi.ei_peruttavaa";
const EI_KERATTAVAA: &str = "pasianssi.ei_kerattavaa";
const JUMISSA: &str = "pasianssi.jumissa";
const VOITTOVIESTI: &str = "pasianssi.voittoviesti";
const SIIRROT_LOPPU: &str = "pasianssi.siirrot_loppu";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vaihe {
    Alku,
    Pelissa,
    Voitto,
    Tappio,
    /// Voitto pääsi kolmen parhaan joukkoon ja odottaa nimimerkkiä.
    Nimi,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nappi {
    Uusi,
    Peruuta,
    Keraa,
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

/// Hiirellä kiinni pidetty kortti tai jono.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pito {
    pub lahde: Lahde,
    /// Hiiren paikka suhteessa jonon alimman kortin kulmaan.
    pub tartunta: (i32, i32),
    alku: (i32, i32),
    pub liikkui: bool,
}

pub struct Pasianssi {
    pub vaihe: Vaihe,
    pub vaikeus: Vaikeus,
    pub lauta: Lauta,
    pub parhaat: Vec<Tulos>,
    /// Nettilista. `None`, kunnes palvelin on vastannut.
    pub netti_parhaat: Option<Vec<Tulos>>,
    /// Kirjattu tulos, joka odottaa lähetystä nettilistalle.
    pub lahetys: Option<Tulos>,
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
    pub aanet_paalla: bool,
    pub aanijono: Vec<(Aani, u32)>,
    pub kello_ms: u32,
    pub peliaika_ms: u32,
    pub voitto_alku: u32,
    viime_klikkaus: Option<(Lahde, u32)>,
    arpa: Arpa,
}

impl Pasianssi {
    pub fn new(siemen: u64) -> Self {
        let mut arpa = Arpa::new(siemen);
        Self {
            vaihe: Vaihe::Alku,
            vaikeus: Vaikeus::Normaali,
            lauta: Lauta::new(Vaikeus::Normaali, &mut arpa),
            parhaat: Vec::new(),
            netti_parhaat: None,
            lahetys: None,
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
            aanet_paalla: true,
            aanijono: Vec::new(),
            kello_ms: 0,
            peliaika_ms: 0,
            voitto_alku: 0,
            viime_klikkaus: None,
            arpa,
        }
    }

    /// Esillä oleva lista: nettilista, tai oma lista, jos verkkoon ei ole saatu yhteyttä.
    pub fn nakyva_lista(&self) -> &[Tulos] {
        self.netti_parhaat.as_deref().unwrap_or(&self.parhaat)
    }

    /// Mitä voitto antaisi juuri nyt. Hävitty peli ei anna mitään.
    pub fn pisteet(&self) -> u32 {
        if self.vaihe == Vaihe::Tappio {
            return 0;
        }
        aikapisteet(self.lauta.vaikeus, self.peliaika_ms / 1000)
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
        self.lauta = Lauta::new(self.vaikeus, &mut self.arpa);
        self.vaihe = Vaihe::Pelissa;
        self.peliaika_ms = 0;
        self.viesti.clear();
        self.unohda_poiminta();
        self.kursori = Kursori {
            ylarivi: true,
            sarake: 0,
            syvyys: 0,
        };
        self.aanijono.push((Aani::Ovi, 0));
        if !self.lauta.on_siirtoja() {
            self.siirrot_loppuivat();
        }
    }

    fn siirrot_loppuivat(&mut self) {
        self.vaihe = Vaihe::Tappio;
        self.viesti = kaanna(SIIRROT_LOPPU).to_string();
        self.aanijono.push((Aani::Tappio, VOITTO_VIIVE_MS));
    }

    fn unohda_poiminta(&mut self) {
        self.valittu = None;
        self.pito = None;
        self.viime_klikkaus = None;
    }

    /// Kirjaa odottavan tuloksen.
    pub fn vahvista_nimi(&mut self) {
        if self.vaihe != Vaihe::Nimi {
            return;
        }
        let nimi = ota_nimi(&mut self.nimi);
        let pisteet = self.pisteet();
        let tulos = Tulos { nimi, pisteet };
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
            // Viimeisenä painettu välilyönti ei saa kuitata nimeä ennen kuin sitä on kirjoitettu.
            Syote::Ammu if self.nimi.is_empty() => {}
            Syote::Ammu | Syote::Peru => self.vahvista_nimi(),
            _ => muokkaa_nimea(&mut self.nimi, syote),
        }
    }

    pub fn kasittele(&mut self, syote: Syote) {
        // Klikkaus jatkaa eteenpäin, jotta äänikytkin toimii nimeä kirjoittaessakin.
        if self.vaihe == Vaihe::Nimi && !matches!(syote, Syote::Klikkaus(..)) {
            self.kasittele_nimi(syote);
            return;
        }
        if syote == Syote::Peru {
            if self.valittu.is_some() || self.pito.is_some() {
                self.unohda_poiminta();
            } else {
                self.lopeta = true;
            }
            return;
        }
        let kytkimella =
            matches!(syote, Syote::Klikkaus(x, y) if piirto::aanikytkimella(self.vaihe, x, y));
        if syote == Syote::VaihdaAanet || kytkimella {
            self.aanet_paalla = !self.aanet_paalla;
            return;
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
            Syote::Kirjain('E') => Nappi::Valikko,
            Syote::Klikkaus(x, y) => match piirto::nappi_pisteesta(x, y) {
                Some(nappi) => nappi,
                None => return,
            },
            _ => return,
        };
        match nappi {
            Nappi::Uusi => self.vaihe = Vaihe::Alku,
            Nappi::Valikko => self.lopeta = true,
            Nappi::Peruuta | Nappi::Keraa => {}
        }
    }

    fn kasittele_pelissa(&mut self, syote: Syote) {
        match syote {
            Syote::Hiiri(x, y) => self.hiiri_liikkuu(x, y),
            Syote::Klikkaus(x, y) => self.klikkaus(x, y),
            Syote::Vapautus(x, y) => self.vapautus(x, y),
            Syote::Nuoli(drivi, dsarake) => self.liikuta_kursoria(drivi, dsarake),
            Syote::Ammu => self.kursorin_toiminto(),
            Syote::Kirjain('N') => self.nosta(),
            Syote::Kirjain('U') => self.paina(Nappi::Peruuta),
            Syote::Kirjain('A') => self.paina(Nappi::Keraa),
            Syote::Kirjain('K') => self.paina(Nappi::Uusi),
            Syote::Kirjain('E') => self.paina(Nappi::Valikko),
            _ => {}
        }
    }

    fn paina(&mut self, nappi: Nappi) {
        self.unohda_poiminta();
        match nappi {
            Nappi::Uusi => self.vaihe = Vaihe::Alku,
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
        if !self.vaikeus.peruutus_sallittu() {
            self.virhe(EI_PERUUTUSTA);
        } else if self.lauta.peruuta() {
            self.viesti.clear();
            self.kursori.syvyys = 0;
            self.aanijono.push((Aani::Liuku, 0));
        } else {
            self.virhe(EI_PERUTTAVAA);
        }
    }

    fn nosta(&mut self) {
        self.unohda_poiminta();
        if self.lauta.nosta() {
            self.siirto_tehty(Aani::Liuku);
        } else {
            self.virhe(PAKKA_LOPPU);
        }
    }

    fn virhe(&mut self, viesti: &str) {
        self.viesti = kaanna(viesti).to_string();
        self.aanijono.push((Aani::Ahkaisy, 0));
    }

    /// Yhteinen loppu kaikille laudan muuttaneille siirroille.
    fn siirto_tehty(&mut self, aani: Aani) {
        self.valittu = None;
        self.pito = None;
        self.kursori.syvyys = 0;
        self.aanijono.push((aani, 0));
        if self.lauta.on_voitettu() {
            self.vaihe = Vaihe::Voitto;
            self.voitto_alku = self.kello_ms;
            let pisteet = self.pisteet();
            self.viesti = muotoile(VOITTOVIESTI, &[&pisteet]);
            self.aanijono.push((Aani::Voitto, VOITTO_VIIVE_MS));
            if mahtuu_parhaisiin(self.nakyva_lista(), pisteet) {
                self.nimi.clear();
                self.vaihe = Vaihe::Nimi;
            }
        } else if !self.lauta.on_siirtoja() {
            self.siirrot_loppuivat();
        } else if self.lauta.on_hyodyllisia_siirtoja() {
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
        if let Some(pito) = &mut self.pito
            && ((x - pito.alku.0).abs() > VETO_RAJA || (y - pito.alku.1).abs() > VETO_RAJA)
        {
            pito.liikkui = true;
        }
    }

    fn klikkaus(&mut self, x: i32, y: i32) {
        self.hiiri = (x, y);
        self.kursori_nakyy = false;
        if let Some(nappi) = piirto::nappi_pisteesta(x, y) {
            self.paina(nappi);
            return;
        }
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
                self.pito = Some(Pito {
                    lahde,
                    tartunta: (x - kulma.0, y - kulma.1),
                    alku: (x, y),
                    liikkui: false,
                });
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
        self.kursori_nakyy = true;
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
        self.kursori_nakyy = true;
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

#[cfg(test)]
mod tests {
    use super::*;
    use klassikot8000::pasianssi::{Kortti, Maa, Sarake};

    fn pelissa() -> Pasianssi {
        let mut peli = Pasianssi::new(1);
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.vaihe, Vaihe::Pelissa);
        peli.lauta.tila.pakka.clear();
        peli.lauta.tila.sarakkeet = Default::default();
        peli
    }

    fn kortti(maa: Maa, arvo: u8) -> Kortti {
        Kortti { maa, arvo }
    }

    fn aseta(peli: &mut Pasianssi, sarake: usize, kortit: &[Kortti]) {
        peli.lauta.tila.sarakkeet[sarake] = Sarake {
            kortit: kortit.to_vec(),
            piilossa: 0,
        };
    }

    /// Kaksi saraketta, joiden välillä musta 6 siirtyy punaisen 7:n päälle.
    fn kaksi_saraketta() -> Pasianssi {
        let mut peli = pelissa();
        aseta(&mut peli, 0, &[kortti(Maa::Pata, 6)]);
        aseta(&mut peli, 1, &[kortti(Maa::Hertta, 7)]);
        peli
    }

    fn kortin_keskipiste(peli: &Pasianssi, sarake: usize, kortti: usize) -> (i32, i32) {
        let (x, y) = piirto::kortin_paikka(&peli.lauta, Lahde::Sarake { sarake, kortti });
        (x + 20, y + 20)
    }

    #[test]
    fn alussa_valitaan_taso_ja_peli_alkaa_sillä() {
        let mut peli = Pasianssi::new(1);
        peli.kasittele(Syote::Numero(3));
        assert_eq!(peli.vaikeus, Vaikeus::Vaikea);
        peli.kasittele(Syote::Nuoli(0, -1));
        assert_eq!(peli.vaikeus, Vaikeus::Normaali);
        let (x, y) = piirto::vaikeuden_keskipiste(Vaikeus::Helppo);
        peli.kasittele(Syote::Klikkaus(x, y));
        assert_eq!(peli.vaikeus, Vaikeus::Helppo);
        assert_eq!(peli.vaihe, Vaihe::Alku);
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.vaihe, Vaihe::Pelissa);
        assert_eq!(peli.lauta.vaikeus, Vaikeus::Helppo);
        assert_eq!(peli.aanijono, [(Aani::Ovi, 0)]);
    }

    #[test]
    fn veto_siirtaa_kortin_ja_laiton_pudotus_ahkaisee() {
        let mut peli = kaksi_saraketta();
        let (x, y) = kortin_keskipiste(&peli, 0, 0);
        peli.kasittele(Syote::Klikkaus(x, y));
        assert!(peli.pito.is_some());
        let (kx, ky) = kortin_keskipiste(&peli, 1, 0);
        peli.kasittele(Syote::Hiiri(kx, ky));
        assert!(peli.pito.is_some_and(|pito| pito.liikkui));
        peli.kasittele(Syote::Vapautus(kx, ky));
        assert_eq!(peli.lauta.tila.sarakkeet[1].kortit.len(), 2);
        assert!(peli.pito.is_none());

        let mut peli = pelissa();
        aseta(&mut peli, 0, &[kortti(Maa::Pata, 6)]);
        aseta(&mut peli, 1, &[kortti(Maa::Pata, 7)]);
        let (x, y) = kortin_keskipiste(&peli, 0, 0);
        peli.kasittele(Syote::Klikkaus(x, y));
        let (kx, ky) = kortin_keskipiste(&peli, 1, 0);
        peli.kasittele(Syote::Hiiri(kx, ky));
        peli.kasittele(Syote::Vapautus(kx, ky));
        assert_eq!(peli.lauta.tila.sarakkeet[0].kortit.len(), 1);
        assert_eq!(peli.viesti, kaanna(EI_KAY));
        assert!(peli.aanijono.contains(&(Aani::Ahkaisy, 0)));
    }

    #[test]
    fn klikkaus_valitsee_ja_toinen_klikkaus_siirtaa() {
        let mut peli = kaksi_saraketta();
        let (x, y) = kortin_keskipiste(&peli, 0, 0);
        peli.kasittele(Syote::Klikkaus(x, y));
        peli.kasittele(Syote::Vapautus(x, y));
        assert!(peli.valittu.is_some());

        let (kx, ky) = kortin_keskipiste(&peli, 1, 0);
        peli.kasittele(Syote::Klikkaus(kx, ky));
        assert_eq!(peli.lauta.tila.sarakkeet[1].kortit.len(), 2);
        assert!(peli.valittu.is_none());
    }

    #[test]
    fn tuplaklikkaus_vie_perustalle_ja_hidas_ei() {
        let mut peli = pelissa();
        aseta(&mut peli, 0, &[kortti(Maa::Risti, 1)]);
        let (x, y) = kortin_keskipiste(&peli, 0, 0);
        peli.kasittele(Syote::Klikkaus(x, y));
        peli.kasittele(Syote::Vapautus(x, y));
        peli.edista(TUPLAKLIKKAUS_MS + 1);
        peli.kasittele(Syote::Klikkaus(x, y));
        peli.kasittele(Syote::Vapautus(x, y));
        assert_eq!(peli.lauta.tila.perustat[Maa::Risti as usize].len(), 0);

        peli.edista(100);
        peli.kasittele(Syote::Klikkaus(x, y));
        assert_eq!(peli.lauta.tila.perustat[Maa::Risti as usize].len(), 1);
    }

    #[test]
    fn pakan_klikkaus_ja_n_nostavat_ja_lopussa_tulee_viesti() {
        let mut peli = Pasianssi::new(1);
        peli.vaikeus = Vaikeus::Vaikea;
        peli.kasittele(Syote::Ammu);
        let (x, y) = piirto::kortin_kulma_pakka();
        peli.kasittele(Syote::Klikkaus(x + 5, y + 5));
        assert_eq!(peli.lauta.tila.jate.len(), 3);
        peli.kasittele(Syote::Kirjain('N'));
        assert_eq!(peli.lauta.tila.jate.len(), 6);
        for _ in 0..100 {
            peli.kasittele(Syote::Kirjain('N'));
        }
        assert_eq!(peli.viesti, kaanna(PAKKA_LOPPU));
    }

    #[test]
    fn kursori_liikkuu_jonon_ylos_ja_ylarivi_ohittaa_tyhjan_paikan() {
        let mut peli = pelissa();
        peli.kasittele(Syote::Nuoli(0, 1));
        assert_eq!(peli.kursorin_kohta(), Some(Kohta::Jate));
        peli.kasittele(Syote::Nuoli(0, 1));
        assert_eq!(peli.kursorin_kohta(), Some(Kohta::Perusta(0)));
        peli.kasittele(Syote::Nuoli(0, -1));
        assert_eq!(peli.kursorin_kohta(), Some(Kohta::Jate));

        aseta(
            &mut peli,
            1,
            &[kortti(Maa::Hertta, 9), kortti(Maa::Pata, 8)],
        );
        peli.kasittele(Syote::Nuoli(1, 0));
        let kohta = |kortti| Kohta::Sarake { sarake: 1, kortti };
        assert_eq!(peli.kursorin_kohta(), Some(kohta(Some(1))));
        peli.kasittele(Syote::Nuoli(-1, 0));
        assert_eq!(peli.kursorin_kohta(), Some(kohta(Some(0))));
        peli.kasittele(Syote::Nuoli(-1, 0));
        assert_eq!(peli.kursorin_kohta(), Some(Kohta::Jate));
    }

    #[test]
    fn nappaimilla_siirto_ja_toinen_painallus_peruu_valinnan() {
        let mut peli = kaksi_saraketta();
        peli.kasittele(Syote::Nuoli(1, 0));
        peli.kasittele(Syote::Ammu);
        assert!(peli.valittu.is_some());
        peli.kasittele(Syote::Ammu);
        assert!(peli.valittu.is_none());

        peli.kasittele(Syote::Ammu);
        peli.kasittele(Syote::Nuoli(0, 1));
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.lauta.tila.sarakkeet[1].kortit.len(), 2);
    }

    #[test]
    fn esc_peruu_ensin_valinnan_ja_sitten_palaa_valikkoon() {
        let mut peli = kaksi_saraketta();
        peli.kasittele(Syote::Nuoli(1, 0));
        peli.kasittele(Syote::Ammu);
        peli.kasittele(Syote::Peru);
        assert!(peli.valittu.is_none());
        assert!(!peli.lopeta);
        peli.kasittele(Syote::Peru);
        assert!(peli.lopeta);
    }

    #[test]
    fn peruutus_ja_keraa_toimivat_napeista_ja_vaikealla_ei_voi_perua() {
        let mut peli = kaksi_saraketta();
        aseta(&mut peli, 2, &[kortti(Maa::Risti, 1)]);
        let (x, y) = piirto::napin_keskipiste(Nappi::Keraa);
        peli.kasittele(Syote::Klikkaus(x, y));
        assert_eq!(peli.lauta.tila.perustat[Maa::Risti as usize].len(), 1);
        let (x, y) = piirto::napin_keskipiste(Nappi::Peruuta);
        peli.kasittele(Syote::Klikkaus(x, y));
        assert_eq!(peli.lauta.tila.perustat[Maa::Risti as usize].len(), 0);

        let mut peli = Pasianssi::new(1);
        peli.vaikeus = Vaikeus::Vaikea;
        peli.kasittele(Syote::Ammu);
        peli.kasittele(Syote::Kirjain('N'));
        peli.kasittele(Syote::Kirjain('U'));
        assert_eq!(peli.viesti, kaanna(EI_PERUUTUSTA));
        assert_eq!(peli.lauta.tila.jate.len(), 3);
    }

    #[test]
    fn viimeinen_kortti_voittaa_ja_kello_pysahtyy() {
        let mut peli = pelissa();
        for (i, &maa) in Maa::KAIKKI.iter().enumerate() {
            peli.lauta.tila.perustat[i] = (1..=13).map(|arvo| kortti(maa, arvo)).collect();
        }
        let viimeinen = peli.lauta.tila.perustat[0].pop().unwrap();
        aseta(&mut peli, 0, &[viimeinen]);
        peli.edista(1000);
        peli.kasittele(Syote::Kirjain('A'));
        assert_eq!(peli.vaihe, Vaihe::Nimi);
        assert!(peli.aanijono.contains(&(Aani::Voitto, VOITTO_VIIVE_MS)));
        peli.edista(1000);
        assert_eq!(peli.peliaika_ms, 1000);

        // Tulos riippuu vain ajasta, ja nimi kirjoitetaan ennen kuin loppu kuittaa mitään.
        let pisteet = aikapisteet(Vaikeus::Normaali, 1);
        assert_eq!(peli.viesti, muotoile(VOITTOVIESTI, &[&pisteet]));
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.vaihe, Vaihe::Nimi);
        peli.kasittele(Syote::Kirjain('K'));
        assert_eq!(peli.nimi, "K");
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.vaihe, Vaihe::Voitto);
        assert!(peli.uusi_tulos);
        assert_eq!(peli.parhaat[0].pisteet, pisteet);
        assert_eq!(peli.lahetys.as_ref().unwrap().nimi, "K");

        peli.kasittele(Syote::Kirjain('K'));
        assert_eq!(peli.vaihe, Vaihe::Alku);
        peli.vaihe = Vaihe::Voitto;
        peli.kasittele(Syote::Kirjain('E'));
        assert!(peli.lopeta);
    }

    #[test]
    fn listalle_yltamaton_voitto_ja_keskeytys_eivat_kysy_nimea() {
        let mut peli = pelissa();
        peli.netti_parhaat = Some(vec![
            Tulos {
                nimi: "A".to_string(),
                pisteet: 100_000,
            };
            3
        ]);
        for (i, &maa) in Maa::KAIKKI.iter().enumerate() {
            peli.lauta.tila.perustat[i] = (1..=13).map(|arvo| kortti(maa, arvo)).collect();
        }
        let viimeinen = peli.lauta.tila.perustat[0].pop().unwrap();
        aseta(&mut peli, 0, &[viimeinen]);
        peli.kasittele(Syote::Kirjain('A'));
        assert_eq!(peli.vaihe, Vaihe::Voitto);
        assert_eq!(peli.lahetys, None);

        let mut peli = pelissa();
        peli.kasittele(Syote::Peru);
        assert!(peli.lopeta);
        assert_eq!(peli.lahetys, None);
    }

    #[test]
    fn pisteet_laskevat_ajan_kuluessa_ja_havio_ei_anna_mitaan() {
        let mut peli = pelissa();
        let alussa = peli.pisteet();
        peli.edista(1000);
        assert!(peli.pisteet() < alussa);
        peli.vaihe = Vaihe::Tappio;
        assert_eq!(peli.pisteet(), 0);
    }

    #[test]
    fn peli_loppuu_kun_siirrot_loppuvat_ja_kello_pysahtyy() {
        let mut peli = pelissa();
        peli.lauta.vaikeus = Vaikeus::Vaikea;
        peli.lauta.tila.kierratykset = 2;
        peli.lauta.tila.jate = vec![kortti(Maa::Hertta, 5)];
        aseta(&mut peli, 0, &[kortti(Maa::Pata, 9)]);
        aseta(
            &mut peli,
            1,
            &[kortti(Maa::Risti, 5), kortti(Maa::Hertta, 1)],
        );
        peli.edista(1000);
        peli.kasittele(Syote::Kirjain('A'));
        assert_eq!(peli.vaihe, Vaihe::Tappio);
        assert_eq!(peli.viesti, kaanna(SIIRROT_LOPPU));
        assert!(peli.aanijono.contains(&(Aani::Tappio, VOITTO_VIIVE_MS)));
        peli.edista(1000);
        assert_eq!(peli.peliaika_ms, 1000);

        // Lopussa kortteja ei enää liikuteta, vain uusi peli tai valikko.
        peli.kasittele(Syote::Kirjain('N'));
        assert_eq!(peli.lauta.tila.jate.len(), 1);
        peli.kasittele(Syote::Kirjain('K'));
        assert_eq!(peli.vaihe, Vaihe::Alku);
    }

    #[test]
    fn jonon_osan_siirto_pitaa_pelin_kaynnissa_ja_vihje_nakyy() {
        let mut peli = pelissa();
        aseta(
            &mut peli,
            0,
            &[
                kortti(Maa::Hertta, 9),
                kortti(Maa::Pata, 8),
                kortti(Maa::Hertta, 7),
            ],
        );
        aseta(&mut peli, 1, &[kortti(Maa::Risti, 8)]);
        aseta(&mut peli, 2, &[kortti(Maa::Hertta, 1)]);
        peli.kasittele(Syote::Kirjain('A'));
        assert_eq!(peli.vaihe, Vaihe::Pelissa);
        assert_eq!(peli.viesti, kaanna(JUMISSA));
    }
}
