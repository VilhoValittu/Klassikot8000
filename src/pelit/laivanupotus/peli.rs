use klassikot8000::Arpa;
use klassikot8000::laivanupotus::{
    KOKO, LAIVAT, Laivasto, Laukaustulos, Ruudut, Ruutu, SARAKKEET, Vaikeus, pisteet, vastustaja,
};

use super::piirto;
use crate::kaynnistin::tallennus::{Asetukset, Tallennus};
use crate::yhteinen::aanet::{Aani, PUHE_VIIVE_MS};
use crate::yhteinen::kieli::{kaanna, muotoile};
use crate::yhteinen::pelinakyma::{Lista, Pelinakyma};
use crate::yhteinen::piirto::{Kehys, aanikytkimella};
use crate::yhteinen::syote::{Syote, Yleinen, yleinen};
use crate::yhteinen::tulokset::{
    Nettipeli, Tulos, lisaa_tulos, mahtuu_parhaisiin, muokkaa_nimea, ota_nimi,
};

// Kierroksen äänet tulevat peräkkäin, jotta laukaus ja sen seuraus erottuvat.
pub const TULOS_VIIVE_MS: u32 = 220;
const VASTAUS_VIIVE_MS: u32 = 950;
const LOPPU_VIIVE_MS: u32 = 900;
pub const TEHOSTE_KESTO_MS: u32 = 800;

/// Laukauksen lento ja osuman animaatio. Ajastus on sama kuin äänillä.
#[derive(Clone, Copy, Debug)]
pub struct Tehoste {
    /// Tosi, kun laukaus osuu pelaajan omalle laudalle.
    pub omalle: bool,
    pub ruutu: Ruutu,
    pub tulos: Laukaustulos,
    pub laukaus_ms: u32,
}

impl Tehoste {
    fn osuma_ms(&self) -> u32 {
        self.laukaus_ms + TULOS_VIIVE_MS
    }
}

/// Voitot, tappiot ja ennätys yhdellä vaikeustasolla.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Tilasto {
    pub voitot: u32,
    pub tappiot: u32,
    /// Vähiten laukauksia voittoon. Nolla tarkoittaa, ettei voittoa ole vielä.
    pub vahiten_laukauksia: u32,
}

impl Tilasto {
    /// Tason paras voitto top-listan pisteinä. Nolla, jos voittoa ei ole.
    pub fn parhaat_pisteet(&self, vaikeus: Vaikeus) -> u32 {
        if self.vahiten_laukauksia == 0 {
            0
        } else {
            pisteet(vaikeus, self.vahiten_laukauksia)
        }
    }

    fn kirjaa(&mut self, voitto: bool, laukauksia: u32) {
        if voitto {
            self.voitot += 1;
            if self.vahiten_laukauksia == 0 || laukauksia < self.vahiten_laukauksia {
                self.vahiten_laukauksia = laukauksia;
            }
        } else {
            self.tappiot += 1;
        }
    }
}

/// Tason paikka tilastotaulukossa.
pub fn tason_numero(vaikeus: Vaikeus) -> usize {
    Vaikeus::KAIKKI
        .iter()
        .position(|&t| t == vaikeus)
        .unwrap_or(0)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vaihe {
    Alku,
    /// Pelaaja voi siirtää omia laivojaan ennen ensimmäistä laukausta.
    Sijoitus,
    Ammunta,
    Voitto,
    Tappio,
    /// Voitto pääsi kolmen parhaan joukkoon ja odottaa nimimerkkiä.
    Nimi,
}

/// Siirrettäväksi nostettu oma laiva. Se seuraa kursoria siitä kohdasta, josta siihen tartuttiin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Nostettu {
    pub laiva: usize,
    pub vaaka: bool,
    /// Monesko laivan ruutu on kursorin kohdalla.
    pub tartunta: usize,
}

pub struct Peli {
    pub vaihe: Vaihe,
    pub vaikeus: Vaikeus,
    pub oma: Laivasto,
    pub vastustaja: Laivasto,
    /// Sijoituksessa oman laudan ruutu, ammunnassa vastustajan.
    pub kursori: Ruutu,
    pub nostettu: Option<Nostettu>,
    /// Edellisen kierroksen tulokset: ensin pelaajan, sitten vastustajan laukaus.
    pub viestit: [String; 2],
    pub ammuttu_jo: bool,
    /// Pyyntö palata käynnistimeen.
    pub lopeta: bool,
    pub aanet_paalla: bool,
    /// Soitettavat äänet ja niiden viiveet millisekunteina. `main.rs` tyhjentää jonon.
    pub aanijono: Vec<(Aani, u32)>,
    pub tehosteet: Vec<Tehoste>,
    /// Pelin alusta kulunut aika, josta kaikki animaatiot lasketaan.
    pub kello_ms: u32,
    /// Tasoittain, järjestys kuten `Vaikeus::KAIKKI`.
    pub tilastot: [Tilasto; 3],
    /// Tosi, kun päättynyt peli on kirjattu tilastoon: käynnistin tallentaa sen.
    pub uusi_tulos: bool,
    pub parhaat: Vec<Tulos>,
    /// Nettilista. `None`, kunnes palvelin on vastannut.
    pub netti_parhaat: Option<Vec<Tulos>>,
    /// Kirjattu tulos, joka odottaa lähetystä nettilistalle.
    pub lahetys: Option<Tulos>,
    /// Kirjoitettavana oleva nimimerkki.
    pub nimi: String,
    arpa: Arpa,
}

impl Peli {
    pub fn new(siemen: u64) -> Self {
        let mut arpa = Arpa::new(siemen);
        Self {
            vaihe: Vaihe::Alku,
            vaikeus: Vaikeus::Normaali,
            oma: Laivasto::satunnainen(&mut arpa),
            vastustaja: Laivasto::satunnainen(&mut arpa),
            kursori: Ruutu { rivi: 0, sarake: 0 },
            nostettu: None,
            viestit: Default::default(),
            ammuttu_jo: false,
            lopeta: false,
            aanet_paalla: true,
            aanijono: Vec::new(),
            tehosteet: Vec::new(),
            kello_ms: 0,
            tilastot: Default::default(),
            uusi_tulos: false,
            parhaat: Vec::new(),
            netti_parhaat: None,
            lahetys: None,
            nimi: String::new(),
            arpa,
        }
    }

    /// Esillä oleva lista: nettilista, tai oma lista, jos verkkoon ei ole saatu yhteyttä.
    pub fn nakyva_lista(&self) -> &[Tulos] {
        self.netti_parhaat.as_deref().unwrap_or(&self.parhaat)
    }

    /// Voiton tulos top-listalle. Kesken olevassa tai hävityssä pelissä nolla.
    pub fn pisteet(&self) -> u32 {
        if matches!(self.vaihe, Vaihe::Voitto | Vaihe::Nimi) {
            pisteet(self.vaikeus, self.laukauksia())
        } else {
            0
        }
    }

    /// Pelaajan ammuttujen laukausten määrä.
    pub fn laukauksia(&self) -> u32 {
        self.vastustaja.ammutut().maara() as u32
    }

    fn kirjaa_tulos(&mut self, voitto: bool) {
        let laukauksia = self.laukauksia();
        self.tilastot[tason_numero(self.vaikeus)].kirjaa(voitto, laukauksia);
        self.uusi_tulos = true;
        if !voitto {
            return;
        }
        let pisteet = self.pisteet();
        self.viestit[1] = muotoile("laivanupotus.voiton_pisteet", &[&laukauksia, &pisteet]);
        if mahtuu_parhaisiin(self.nakyva_lista(), pisteet) {
            self.nimi.clear();
            self.vaihe = Vaihe::Nimi;
        }
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

    /// Kertoo, muuttuiko näkymä.
    fn kasittele_nimi(&mut self, syote: Syote) -> bool {
        match syote {
            // Viimeiseen laukaukseen painettu välilyönti ei saa kuitata nimeä ennen kuin sitä on kirjoitettu.
            Syote::Ammu if self.nimi.is_empty() => return false,
            Syote::Ammu | Syote::Peru => self.vahvista_nimi(),
            _ => muokkaa_nimea(&mut self.nimi, syote),
        }
        true
    }

    pub fn edista(&mut self, ms: u32) {
        self.kello_ms = self.kello_ms.saturating_add(ms);
        let kello = self.kello_ms;
        self.tehosteet
            .retain(|t| kello < t.osuma_ms() + TEHOSTE_KESTO_MS);
    }

    /// Onko ruutuun ammuttu laukaus vielä ilmassa? Tulos ei saa näkyä ennen osumaa.
    pub fn odottaa_osumaa(&self, omalle: bool, ruutu: Ruutu) -> bool {
        self.tehosteet
            .iter()
            .any(|t| t.omalle == omalle && t.ruutu == ruutu && self.kello_ms < t.osuma_ms())
    }

    /// Onko vastustajan laukaus vielä matkalla? Silloin sen tulosta ei kerrota.
    pub fn vastaus_kesken(&self) -> bool {
        self.tehosteet
            .iter()
            .any(|t| t.omalle && self.kello_ms < t.osuma_ms())
    }

    /// Vastustajan laudan ruudut uponneiden laivojen ympärillä: laivat eivät kosketa toisiaan,
    /// joten niissä ei voi olla mitään. Ympäristö paljastuu vasta, kun laukaus on osunut.
    pub fn varmasti_tyhjat(&self) -> Ruudut {
        let mut tyhjat = Ruudut::default();
        for laiva in self.vastustaja.laivat() {
            let ilmassa = self.tehosteet.iter().any(|t| {
                !t.omalle && laiva.ruudut.sisaltaa(t.ruutu) && self.kello_ms < t.osuma_ms()
            });
            if self.vastustaja.on_upotettu(laiva) && !ilmassa {
                tyhjat = tyhjat.yhdiste(laiva.ruudut.ymparys());
            }
        }
        tyhjat.erotus(self.vastustaja.ammutut())
    }

    fn aloita_alusta(&mut self) {
        self.vaihe = Vaihe::Alku;
        self.oma = Laivasto::satunnainen(&mut self.arpa);
        self.vastustaja = Laivasto::satunnainen(&mut self.arpa);
        self.kursori = Ruutu { rivi: 0, sarake: 0 };
        self.nostettu = None;
        self.viestit = Default::default();
        self.ammuttu_jo = false;
        self.tehosteet.clear();
    }

    /// Nostetun laivan ruudut kursorin kohdalla ja kelpaako paikka. Laudan reunalla laiva
    /// pysähtyy, vaikka kursori jatkaisi.
    pub fn haamu(&self) -> Option<(usize, Ruudut, bool)> {
        let nostettu = self.nostettu?;
        let pituus = self.oma.laivat().get(nostettu.laiva)?.ruudut.maara();
        let alkuun = |kohta: u8| {
            (kohta as usize)
                .saturating_sub(nostettu.tartunta)
                .min(KOKO - pituus)
        };
        let (rivi, sarake) = (self.kursori.rivi as usize, self.kursori.sarake as usize);
        let alku = if nostettu.vaaka {
            Ruutu::new(rivi, alkuun(self.kursori.sarake))
        } else {
            Ruutu::new(alkuun(self.kursori.rivi), sarake)
        }?;
        let ruudut = Ruudut::suora(alku, pituus, nostettu.vaaka)?;
        Some((
            nostettu.laiva,
            ruudut,
            self.oma.mahtuu(nostettu.laiva, ruudut),
        ))
    }

    /// Nostaa laivan `i`. Kursori siirtyy ruutuun, josta siihen tartutaan.
    fn nosta(&mut self, i: usize, tartuntaruutu: Ruutu) -> bool {
        let Some(laiva) = self.oma.laivat().get(i).copied() else {
            return false;
        };
        let Some(alku) = laiva.alku() else {
            return false;
        };
        let tartunta =
            (tartuntaruutu.rivi - alku.rivi + tartuntaruutu.sarake - alku.sarake) as usize;
        self.kursori = tartuntaruutu;
        self.nostettu = Some(Nostettu {
            laiva: i,
            vaaka: laiva.vaaka(),
            tartunta,
        });
        true
    }

    /// Laskee nostetun laivan kursorin kohdalle. Muihin koskeva paikka ei kelpaa.
    fn laske(&mut self) -> bool {
        let Some((laiva, ruudut, kelpaa)) = self.haamu() else {
            return false;
        };
        if kelpaa && self.oma.siirra(laiva, ruudut) {
            self.nostettu = None;
        } else {
            self.aanijono.push((Aani::Virhe, 0));
        }
        true
    }

    /// Nostettu laiva kääntyy kädessä, laudalla oleva paikallaan tartuntakohtansa ympäri.
    fn kaanna(&mut self) -> bool {
        if let Some(nostettu) = &mut self.nostettu {
            nostettu.vaaka = !nostettu.vaaka;
            return true;
        }
        let Some(i) = self.oma.laiva_ruudussa(self.kursori) else {
            return false;
        };
        self.nosta(i, self.kursori);
        if let Some(nostettu) = &mut self.nostettu {
            nostettu.vaaka = !nostettu.vaaka;
        }
        match self.haamu() {
            Some((i, ruudut, true)) => {
                self.oma.siirra(i, ruudut);
            }
            _ => self.aanijono.push((Aani::Virhe, 0)),
        }
        self.nostettu = None;
        true
    }

    fn arvo_laivat(&mut self) -> bool {
        self.oma = Laivasto::satunnainen(&mut self.arpa);
        self.nostettu = None;
        true
    }

    /// Kesken jäänyt nosto perutaan: laiva jää entiselle paikalleen.
    fn aloita_ammunta(&mut self) -> bool {
        self.nostettu = None;
        self.kursori = Ruutu { rivi: 0, sarake: 0 };
        self.vaihe = Vaihe::Ammunta;
        self.aanijono.push((Aani::Alku, 0));
        true
    }

    /// Käsittelee syötteen ja kertoo, muuttuiko näkymä.
    pub fn kasittele(&mut self, syote: Syote) -> bool {
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
                return false;
            }
            Yleinen::Aanet => return true,
            Yleinen::Ohita => return false,
        }
        match self.vaihe {
            Vaihe::Alku => self.kasittele_alku(syote),
            Vaihe::Sijoitus => self.kasittele_sijoitus(syote),
            Vaihe::Ammunta => self.kasittele_ammunta(syote),
            Vaihe::Voitto | Vaihe::Tappio => self.kasittele_loppu(syote),
            Vaihe::Nimi => false,
        }
    }

    fn kasittele_alku(&mut self, syote: Syote) -> bool {
        let tasot = Vaikeus::KAIKKI;
        let valittu = match syote {
            Syote::Numero(numero) => (numero as usize)
                .checked_sub(1)
                .and_then(|i| tasot.get(i).copied()),
            Syote::Nuoli(_, muutos) => tasot.iter().position(|&t| t == self.vaikeus).map(|i| {
                tasot[i
                    .saturating_add_signed(muutos as isize)
                    .min(tasot.len() - 1)]
            }),
            Syote::Klikkaus(x, y) => piirto::vaikeus_pisteesta(x, y),
            _ => None,
        };
        if let Some(vaikeus) = valittu {
            let muuttui = vaikeus != self.vaikeus;
            self.vaikeus = vaikeus;
            return muuttui;
        }
        if matches!(syote, Syote::Ammu | Syote::Klikkaus(..)) {
            self.vaihe = Vaihe::Sijoitus;
            return true;
        }
        false
    }

    fn kasittele_sijoitus(&mut self, syote: Syote) -> bool {
        match syote {
            Syote::Hiiri(x, y) => match piirto::oma_ruutu_pisteesta(x, y) {
                Some(ruutu) if ruutu != self.kursori => {
                    self.kursori = ruutu;
                    true
                }
                _ => false,
            },
            Syote::Klikkaus(x, y) => {
                match piirto::nappi_pisteesta(Vaihe::Sijoitus, x, y) {
                    Some(piirto::Nappi::Arvo) => return self.arvo_laivat(),
                    Some(piirto::Nappi::Taisteluun) => return self.aloita_ammunta(),
                    _ => {}
                }
                let Some(ruutu) = piirto::oma_ruutu_pisteesta(x, y) else {
                    return false;
                };
                self.kursori = ruutu;
                if self.nostettu.is_some() {
                    return self.laske();
                }
                match self.oma.laiva_ruudussa(ruutu) {
                    Some(i) => self.nosta(i, ruutu),
                    None => false,
                }
            }
            Syote::OikeaKlikkaus(x, y) => {
                if let Some(ruutu) = piirto::oma_ruutu_pisteesta(x, y) {
                    self.kursori = ruutu;
                }
                self.kaanna()
            }
            Syote::Kirjain('R') => self.kaanna(),
            Syote::Kirjain('A') => self.arvo_laivat(),
            // Numero nostaa laivan sen alusta, samassa järjestyksessä kuin laudan alla.
            Syote::Numero(numero @ 1..) if (numero as usize) <= LAIVAT.len() => {
                let i = numero as usize - 1;
                match self.oma.laivat()[i].alku() {
                    Some(alku) => self.nosta(i, alku),
                    None => false,
                }
            }
            Syote::Nuoli(drivi, dsarake) => {
                self.kursori = Ruutu {
                    rivi: siirra(self.kursori.rivi, drivi),
                    sarake: siirra(self.kursori.sarake, dsarake),
                };
                true
            }
            Syote::Ammu if self.nostettu.is_some() => self.laske(),
            Syote::Ammu => self.aloita_ammunta(),
            _ => false,
        }
    }

    fn kasittele_ammunta(&mut self, syote: Syote) -> bool {
        let uusi = match syote {
            Syote::Hiiri(x, y) => match piirto::ruutu_pisteesta(x, y) {
                Some(ruutu) => ruutu,
                None => return false,
            },
            Syote::Klikkaus(x, y) => {
                let Some(ruutu) = piirto::ruutu_pisteesta(x, y) else {
                    return false;
                };
                self.kursori = ruutu;
                if !self.vastaus_kesken() {
                    self.ammu(ruutu);
                }
                return true;
            }
            Syote::Nuoli(drivi, dsarake) => Ruutu {
                rivi: siirra(self.kursori.rivi, drivi),
                sarake: siirra(self.kursori.sarake, dsarake),
            },
            Syote::Kirjain(kirjain) => match SARAKKEET.find(kirjain) {
                Some(sarake) => Ruutu {
                    sarake: sarake as u8,
                    ..self.kursori
                },
                None => return false,
            },
            Syote::Numero(numero) => Ruutu {
                rivi: (numero + 9) % 10,
                ..self.kursori
            },
            // Uusi laukaus odottaa vastustajan vuoroa, muuten sen tulos jäisi näkemättä.
            Syote::Ammu if self.vastaus_kesken() => return false,
            Syote::Ammu => {
                self.ammu(self.kursori);
                return true;
            }
            Syote::Peru
            | Syote::VaihdaAanet
            | Syote::Pyyhi
            | Syote::Vapautus(..)
            | Syote::OikeaKlikkaus(..)
            | Syote::Rulla(_) => return false,
        };
        let muuttui = uusi != self.kursori || self.ammuttu_jo;
        self.kursori = uusi;
        self.ammuttu_jo = false;
        muuttui
    }

    fn kasittele_loppu(&mut self, syote: Syote) -> bool {
        let uudestaan = match syote {
            Syote::Kirjain('K') => true,
            Syote::Kirjain('E') => false,
            Syote::Klikkaus(x, y) => match piirto::nappi_pisteesta(self.vaihe, x, y) {
                Some(piirto::Nappi::Uudestaan) => true,
                Some(piirto::Nappi::Lopeta) => false,
                _ => return false,
            },
            _ => return false,
        };
        if uudestaan {
            self.aloita_alusta();
        } else {
            self.lopeta = true;
        }
        uudestaan
    }

    /// Yksi kierros: pelaaja ampuu, voittotarkistus, vastustaja ampuu, häviötarkistus.
    pub fn ammu(&mut self, ruutu: Ruutu) {
        let Some(tulos) = self.vastustaja.kasittele_laukaus(ruutu) else {
            self.ammuttu_jo = true;
            self.aanijono.push((Aani::Virhe, 0));
            return;
        };
        self.ammuttu_jo = false;
        self.viestit = [
            tulos_teksti("laivanupotus.sina_ammuit", ruutu, tulos),
            String::new(),
        ];
        self.aanijono.push((Aani::Laukaus, 0));
        self.aanijono.push((tuloksen_aani(tulos), TULOS_VIIVE_MS));
        self.tehosteet.push(Tehoste {
            omalle: false,
            ruutu,
            tulos,
            laukaus_ms: self.kello_ms,
        });
        if self.vastustaja.kaikki_upotettu() {
            self.vaihe = Vaihe::Voitto;
            // Kirjaus voi vielä vaihtaa vaiheen nimen kysymiseen.
            self.kirjaa_tulos(true);
            self.aanijono.push((Aani::Voitto, LOPPU_VIIVE_MS));
            self.aanijono
                .push((Aani::Voitit, LOPPU_VIIVE_MS + PUHE_VIIVE_MS));
            return;
        }

        let laukaus = vastustaja::valitse(self.vaikeus, &self.oma.tiedot(), &mut self.arpa);
        let tulos = laukaus.and_then(|r| Some((r, self.oma.kasittele_laukaus(r)?)));
        if let Some((ruutu, tulos)) = tulos {
            self.viestit[1] = tulos_teksti("laivanupotus.vastustaja_ampui", ruutu, tulos);
            self.aanijono.push((Aani::Vastalaukaus, VASTAUS_VIIVE_MS));
            self.aanijono
                .push((tuloksen_aani(tulos), VASTAUS_VIIVE_MS + TULOS_VIIVE_MS));
            self.tehosteet.push(Tehoste {
                omalle: true,
                ruutu,
                tulos,
                laukaus_ms: self.kello_ms + VASTAUS_VIIVE_MS,
            });
        }
        if self.oma.kaikki_upotettu() {
            self.vaihe = Vaihe::Tappio;
            self.kirjaa_tulos(false);
            self.aanijono
                .push((Aani::Tappio, VASTAUS_VIIVE_MS + LOPPU_VIIVE_MS));
            self.aanijono.push((
                Aani::PeliOhi,
                VASTAUS_VIIVE_MS + LOPPU_VIIVE_MS + PUHE_VIIVE_MS,
            ));
        }
    }
}

fn tuloksen_aani(tulos: Laukaustulos) -> Aani {
    match tulos {
        Laukaustulos::Ohi => Aani::Roiske,
        Laukaustulos::Osuma => Aani::Rajahdys,
        Laukaustulos::Upposi(_) => Aani::Upotus,
    }
}

fn siirra(arvo: u8, muutos: i8) -> u8 {
    arvo.saturating_add_signed(muutos).min(KOKO as u8 - 1)
}

/// Logiikka tuntee laivat suomenkielisillä nimillä, näkyvä nimi tulee kielitiedostosta.
pub fn laivan_nimi(nimi: &'static str) -> &'static str {
    match nimi {
        "Lentotukialus" => kaanna("laivanupotus.lentotukialus"),
        "Taistelulaiva" => kaanna("laivanupotus.taistelulaiva"),
        "Hävittäjä" => kaanna("laivanupotus.havittaja"),
        "Sukellusvene" => kaanna("laivanupotus.sukellusvene"),
        "Partioalus" => kaanna("laivanupotus.partioalus"),
        _ => nimi,
    }
}

pub fn tulos_teksti(ampuja: &str, ruutu: Ruutu, tulos: Laukaustulos) -> String {
    let ampuja = kaanna(ampuja);
    match tulos {
        Laukaustulos::Ohi => muotoile("laivanupotus.ohi", &[&ampuja, &ruutu]),
        Laukaustulos::Osuma => muotoile("laivanupotus.osuma", &[&ampuja, &ruutu]),
        Laukaustulos::Upposi(laiva) => muotoile(
            "laivanupotus.upposi",
            &[&ampuja, &ruutu, &laivan_nimi(laiva)],
        ),
    }
}

impl Pelinakyma for Peli {
    fn lue_tallennuksesta(&mut self, tallennus: &Tallennus) {
        self.vaikeus = tallennus.asetukset.laivanupotus;
        self.tilastot = tallennus.laivatilasto;
    }

    fn kirjaa_asetukset(&self, asetukset: &mut Asetukset) {
        asetukset.laivanupotus = self.vaikeus;
    }

    fn kirjaa_tallennukseen(&self, tallennus: &mut Tallennus) {
        tallennus.laivatilasto = self.tilastot;
    }

    fn kasittele(&mut self, syote: Syote) {
        Peli::kasittele(self, syote);
    }

    fn edista(&mut self, ms: u32) {
        Peli::edista(self, ms);
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
            peli: Nettipeli::Laivanupotus,
            paras: None,
            parhaat: &mut self.parhaat,
            netti: &mut self.netti_parhaat,
            lahetys: Some(&mut self.lahetys),
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yhteinen::piirto::aanikytkimen_keskipiste;

    fn sijoituksessa(siemen: u64) -> Peli {
        let mut peli = Peli::new(siemen);
        assert!(peli.kasittele(Syote::Ammu));
        assert_eq!(peli.vaihe, Vaihe::Sijoitus);
        peli
    }

    fn pelissa(siemen: u64) -> Peli {
        let mut peli = sijoituksessa(siemen);
        assert!(peli.kasittele(Syote::Ammu));
        assert_eq!(peli.vaihe, Vaihe::Ammunta);
        peli
    }

    /// Ruutu, jossa ei ole laivaa eikä laivan vieressä.
    fn vapaa_ruutu(peli: &Peli) -> Ruutu {
        let varatut = peli
            .oma
            .laivat()
            .iter()
            .fold(Ruudut::default(), |kaikki, l| {
                kaikki.yhdiste(l.ruudut.ymparys())
            });
        (0..KOKO * KOKO)
            .filter_map(|i| Ruutu::new(i / KOKO, i % KOKO))
            .find(|&r| !varatut.sisaltaa(r))
            .unwrap()
    }

    #[test]
    fn laiva_nostetaan_ja_lasketaan_hiirella_vain_vapaaseen_paikkaan() {
        let mut peli = sijoituksessa(1);
        let laiva = peli.oma.laivat()[4];
        let alku = laiva.alku().unwrap();
        let (x, y) = piirto::oman_ruudun_keskipiste(alku);
        assert!(peli.kasittele(Syote::Klikkaus(x, y)));
        assert_eq!(peli.nostettu.map(|n| n.laiva), Some(4));

        // toisen laivan päälle ei voi laskea
        let toinen = peli.oma.laivat()[0].alku().unwrap();
        let (x, y) = piirto::oman_ruudun_keskipiste(toinen);
        peli.kasittele(Syote::Hiiri(x, y));
        assert!(!peli.haamu().unwrap().2);
        peli.aanijono.clear();
        peli.kasittele(Syote::Klikkaus(x, y));
        assert_eq!(peli.aanijono, [(Aani::Virhe, 0)]);
        assert!(peli.nostettu.is_some());
        assert_eq!(peli.oma.laivat()[4].ruudut, laiva.ruudut);

        // Vapaa paikka löytyy aina kahden ruudun laivalle, kun se on nostettu pois.
        let ruudut = (0..KOKO * KOKO)
            .filter_map(|i| Ruutu::new(i / KOKO, i % KOKO))
            .find(|&r| {
                peli.kursori = r;
                peli.haamu()
                    .is_some_and(|(_, ruudut, kelpaa)| kelpaa && ruudut != laiva.ruudut)
            })
            .unwrap();
        let (x, y) = piirto::oman_ruudun_keskipiste(ruudut);
        peli.kasittele(Syote::Klikkaus(x, y));
        assert!(peli.nostettu.is_none());
        assert_ne!(peli.oma.laivat()[4].ruudut, laiva.ruudut);
        assert!(peli.oma.mahtuu(4, peli.oma.laivat()[4].ruudut));
    }

    #[test]
    fn nappaimisto_nostaa_kaantaa_ja_laskee_laivan() {
        let mut peli = sijoituksessa(2);
        assert!(peli.kasittele(Syote::Numero(5)));
        let ennen = peli.oma.laivat()[4];
        assert_eq!(peli.kursori, ennen.alku().unwrap());
        peli.kasittele(Syote::Kirjain('R'));
        assert_eq!(peli.nostettu.map(|n| n.vaaka), Some(!ennen.vaaka()));
        peli.kasittele(Syote::Kirjain('R'));
        // Nuolet vievät haamun vapaaseen kohtaan, ja välilyönti laskee sen.
        let kohde = vapaa_ruutu(&peli);
        while peli.kursori != kohde {
            let d = |a: u8, b: u8| (b as i8 - a as i8).signum();
            peli.kasittele(Syote::Nuoli(
                d(peli.kursori.rivi, kohde.rivi),
                d(peli.kursori.sarake, kohde.sarake),
            ));
        }
        if peli.haamu().unwrap().2 {
            peli.kasittele(Syote::Ammu);
            assert!(peli.nostettu.is_none());
            assert_eq!(peli.vaihe, Vaihe::Sijoitus);
        }
        // Välilyönti ilman nostettua laivaa aloittaa taistelun.
        peli.nostettu = None;
        peli.aanijono.clear();
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.vaihe, Vaihe::Ammunta);
        assert_eq!(peli.aanijono, [(Aani::Alku, 0)]);
    }

    #[test]
    fn laudalla_oleva_laiva_kaantyy_paikallaan_jos_mahtuu() {
        let mut kaantyi = 0;
        for siemen in 0..50 {
            let mut peli = sijoituksessa(siemen);
            let ennen = peli.oma.laivat()[4];
            peli.kursori = ennen.alku().unwrap();
            peli.kasittele(Syote::Kirjain('R'));
            assert!(peli.nostettu.is_none());
            let jalkeen = peli.oma.laivat()[4];
            if jalkeen.ruudut != ennen.ruudut {
                kaantyi += 1;
                assert_eq!(jalkeen.vaaka(), !ennen.vaaka());
                assert!(jalkeen.ruudut.sisaltaa(ennen.alku().unwrap()));
            } else {
                assert_eq!(peli.aanijono.last(), Some(&(Aani::Virhe, 0)));
            }
        }
        assert!(kaantyi > 0);
    }

    #[test]
    fn arvonta_ja_taistelunappi_toimivat_ja_nosto_peruuntuu() {
        let mut peli = sijoituksessa(1);
        let ennen: Vec<_> = peli.oma.laivat().iter().map(|l| l.ruudut).collect();
        let (x, y) = piirto::napin_keskipiste(piirto::Nappi::Arvo);
        assert!(peli.kasittele(Syote::Klikkaus(x, y)));
        let jalkeen: Vec<_> = peli.oma.laivat().iter().map(|l| l.ruudut).collect();
        assert_ne!(ennen, jalkeen);

        peli.kasittele(Syote::Numero(1));
        let (x, y) = piirto::napin_keskipiste(piirto::Nappi::Taisteluun);
        peli.kasittele(Syote::Klikkaus(x, y));
        assert_eq!(peli.vaihe, Vaihe::Ammunta);
        assert!(peli.nostettu.is_none());
        let lopuksi: Vec<_> = peli.oma.laivat().iter().map(|l| l.ruudut).collect();
        assert_eq!(lopuksi, jalkeen);
    }

    #[test]
    fn tilasto_kirjaa_voitot_tappiot_ja_vahimmat_laukaukset() {
        let mut voitot = 0;
        let mut tappiot = 0;
        for siemen in 0..60 {
            let mut peli = pelissa(siemen);
            peli.vaikeus = Vaikeus::Helppo;
            for i in 0..KOKO * KOKO {
                peli.ammu(Ruutu::new(i / KOKO, i % KOKO).unwrap());
                if peli.vaihe != Vaihe::Ammunta {
                    break;
                }
            }
            assert!(peli.uusi_tulos);
            let tilasto = peli.tilastot[0];
            assert_eq!(peli.tilastot[2], Tilasto::default());
            // Tyhjälle listalle jokainen voitto yltää, joten se odottaa nimeä.
            if peli.vaihe == Vaihe::Nimi {
                voitot += 1;
                assert_eq!(tilasto.voitot, 1);
                assert_eq!(tilasto.vahiten_laukauksia, peli.laukauksia());
            } else {
                tappiot += 1;
                assert_eq!(tilasto.tappiot, 1);
                assert_eq!(tilasto.vahiten_laukauksia, 0);
            }
        }
        assert!(voitot > 0 && tappiot > 0);

        let mut tilasto = Tilasto::default();
        tilasto.kirjaa(true, 50);
        tilasto.kirjaa(true, 40);
        tilasto.kirjaa(true, 60);
        tilasto.kirjaa(false, 30);
        assert_eq!(
            tilasto,
            Tilasto {
                voitot: 3,
                tappiot: 1,
                vahiten_laukauksia: 40
            }
        );
    }

    #[test]
    fn nappaimisto_siirtaa_kursoria_rajojen_sisalla() {
        let mut peli = pelissa(1);
        peli.kasittele(Syote::Nuoli(-1, -1));
        assert_eq!(peli.kursori, Ruutu { rivi: 0, sarake: 0 });
        peli.kasittele(Syote::Kirjain('J'));
        peli.kasittele(Syote::Numero(0));
        assert_eq!(peli.kursori.to_string(), "J10");
        peli.kasittele(Syote::Nuoli(1, 1));
        assert_eq!(peli.kursori.to_string(), "J10");
        peli.kasittele(Syote::Numero(3));
        peli.kasittele(Syote::Nuoli(0, -1));
        assert_eq!(peli.kursori.to_string(), "I3");
    }

    #[test]
    fn klikkaus_ampuu_ja_vastustaja_vastaa() {
        let mut peli = pelissa(1);
        let (x, y) = piirto::ruudun_keskipiste(Ruutu { rivi: 2, sarake: 3 });
        assert!(peli.kasittele(Syote::Klikkaus(x, y)));
        assert_eq!(peli.vastustaja.ammutut().maara(), 1);
        assert_eq!(peli.oma.ammutut().maara(), 1);
        assert!(peli.viestit[0].starts_with("Sinä ammuit D3: "));
        assert!(peli.viestit[1].starts_with("Vastustaja ampui "));
    }

    #[test]
    fn klikkaus_laudan_ulkopuolelle_ei_tee_mitaan() {
        let mut peli = pelissa(1);
        assert!(!peli.kasittele(Syote::Klikkaus(0, 0)));
        assert_eq!(peli.vastustaja.ammutut().maara(), 0);
    }

    #[test]
    fn sama_ruutu_antaa_huomautuksen_eika_kierrosta() {
        let mut peli = pelissa(1);
        peli.kasittele(Syote::Ammu);
        peli.edista(VASTAUS_VIIVE_MS + TULOS_VIIVE_MS);
        peli.kasittele(Syote::Ammu);
        assert!(peli.ammuttu_jo);
        assert_eq!(peli.oma.ammutut().maara(), 1);
    }

    #[test]
    fn uusi_laukaus_odottaa_vastustajan_vuoroa() {
        let mut peli = pelissa(1);
        peli.kasittele(Syote::Ammu);
        peli.kasittele(Syote::Nuoli(0, 1));
        assert!(!peli.kasittele(Syote::Ammu));
        let (x, y) = piirto::ruudun_keskipiste(Ruutu { rivi: 5, sarake: 5 });
        peli.kasittele(Syote::Klikkaus(x, y));
        assert_eq!(peli.vastustaja.ammutut().maara(), 1);
        assert_eq!(peli.oma.ammutut().maara(), 1);

        peli.edista(VASTAUS_VIIVE_MS + TULOS_VIIVE_MS);
        assert!(peli.kasittele(Syote::Ammu));
        assert_eq!(peli.vastustaja.ammutut().maara(), 2);
    }

    #[test]
    fn uponneen_laivan_ymparys_merkitaan_vasta_osuman_jalkeen() {
        let mut peli = pelissa(1);
        assert_eq!(peli.varmasti_tyhjat().maara(), 0);
        let laiva = peli.vastustaja.laivat()[4];
        let ruudut: Vec<_> = (0..KOKO * KOKO)
            .filter_map(|i| Ruutu::new(i / KOKO, i % KOKO))
            .filter(|&r| laiva.ruudut.sisaltaa(r))
            .collect();
        peli.ammu(ruudut[0]);
        peli.ammu(ruudut[1]);
        assert_eq!(peli.varmasti_tyhjat().maara(), 0);

        peli.edista(TULOS_VIIVE_MS);
        let tyhjat = peli.varmasti_tyhjat();
        assert!(tyhjat.maara() > 0);
        assert!(!tyhjat.leikkaako(laiva.ruudut));
        assert!(tyhjat.on_osajoukko(laiva.ruudut.ymparys()));
        for muu in peli.vastustaja.laivat() {
            assert!(!tyhjat.leikkaako(muu.ruudut));
        }
    }

    #[test]
    fn voiton_jalkeen_vastustaja_ei_ammu_ja_tappio_tunnistetaan() {
        let mut voittoja = 0;
        let mut tappioita = 0;
        for siemen in 0..200 {
            let mut peli = pelissa(siemen);
            for i in 0..KOKO * KOKO {
                peli.ammu(Ruutu::new(i / KOKO, i % KOKO).unwrap());
                if peli.vaihe != Vaihe::Ammunta {
                    break;
                }
            }
            let omat = peli.oma.ammutut().maara();
            let vastustajan = peli.vastustaja.ammutut().maara();
            match peli.vaihe {
                Vaihe::Nimi => {
                    voittoja += 1;
                    assert!(peli.viestit[1].contains(&peli.pisteet().to_string()));
                    assert_eq!(omat + 1, vastustajan);
                }
                Vaihe::Tappio => {
                    tappioita += 1;
                    assert_eq!(omat, vastustajan);
                }
                _ => panic!("peli ei päättynyt"),
            }
        }
        assert!(voittoja > 0 && tappioita > 0);
    }

    #[test]
    fn lopussa_k_aloittaa_alusta_ja_e_lopettaa() {
        let mut peli = pelissa(1);
        peli.ammu(Ruutu { rivi: 0, sarake: 0 });
        peli.vaihe = Vaihe::Tappio;
        assert!(peli.kasittele(Syote::Kirjain('K')));
        assert_eq!(peli.vaihe, Vaihe::Alku);
        assert_eq!(peli.oma.ammutut().maara(), 0);
        assert!(peli.viestit[0].is_empty());

        let mut peli = pelissa(1);
        peli.vaihe = Vaihe::Voitto;
        peli.kasittele(Syote::Kirjain('E'));
        assert!(peli.lopeta);
    }

    #[test]
    fn lopussa_napit_toimivat_hiirella() {
        let mut peli = pelissa(1);
        peli.vaihe = Vaihe::Voitto;
        let (x, y) = piirto::napin_keskipiste(piirto::Nappi::Uudestaan);
        assert!(peli.kasittele(Syote::Klikkaus(x, y)));
        assert_eq!(peli.vaihe, Vaihe::Alku);

        let mut peli = pelissa(1);
        peli.vaihe = Vaihe::Voitto;
        let (x, y) = piirto::napin_keskipiste(piirto::Nappi::Lopeta);
        peli.kasittele(Syote::Klikkaus(x, y));
        assert!(peli.lopeta);
    }

    #[test]
    fn aanet_kytketaan_napilla_ja_kytkimella_kesken_pelin() {
        let mut peli = pelissa(1);
        assert!(peli.aanet_paalla);
        assert!(peli.kasittele(Syote::VaihdaAanet));
        assert!(!peli.aanet_paalla);
        let (x, y) = aanikytkimen_keskipiste();
        assert!(peli.kasittele(Syote::Klikkaus(x, y)));
        assert!(peli.aanet_paalla);
        assert_eq!(peli.vastustaja.ammutut().maara(), 0);
    }

    #[test]
    fn kytkimen_klikkaus_ei_aloita_pelia_ja_valinta_sailyy_uusinnassa() {
        let mut peli = Peli::new(1);
        let (x, y) = aanikytkimen_keskipiste();
        peli.kasittele(Syote::Klikkaus(x, y));
        assert_eq!(peli.vaihe, Vaihe::Alku);
        assert!(!peli.aanet_paalla);

        peli.kasittele(Syote::Ammu);
        peli.vaihe = Vaihe::Voitto;
        peli.kasittele(Syote::Kirjain('K'));
        assert!(!peli.aanet_paalla);
    }

    #[test]
    fn kierros_soittaa_laukauksen_ja_vastustajan_tulipallon() {
        let mut peli = pelissa(1);
        peli.aanijono.clear();
        peli.ammu(Ruutu { rivi: 0, sarake: 0 });
        assert_eq!(peli.aanijono[0], (Aani::Laukaus, 0));
        assert!(
            peli.aanijono
                .contains(&(Aani::Vastalaukaus, VASTAUS_VIIVE_MS))
        );

        peli.aanijono.clear();
        peli.ammu(Ruutu { rivi: 0, sarake: 0 });
        assert_eq!(peli.aanijono, [(Aani::Virhe, 0)]);
    }

    #[test]
    fn tulos_paljastuu_vasta_osumahetkella_ja_tehoste_poistuu() {
        let mut peli = pelissa(1);
        let ruutu = Ruutu { rivi: 0, sarake: 0 };
        peli.ammu(ruutu);
        assert!(peli.odottaa_osumaa(false, ruutu));
        assert!(peli.vastaus_kesken());

        peli.edista(TULOS_VIIVE_MS);
        assert!(!peli.odottaa_osumaa(false, ruutu));
        assert!(peli.vastaus_kesken());

        peli.edista(VASTAUS_VIIVE_MS);
        assert!(!peli.vastaus_kesken());
        assert!(!peli.tehosteet.is_empty());

        peli.edista(TEHOSTE_KESTO_MS);
        assert!(peli.tehosteet.is_empty());
    }

    #[test]
    fn taso_valitaan_alkuruudulla_ja_sailyy_uusinnassa() {
        let mut peli = Peli::new(1);
        assert_eq!(peli.vaikeus, Vaikeus::Normaali);
        peli.kasittele(Syote::Numero(3));
        assert_eq!(peli.vaikeus, Vaikeus::Vaikea);
        peli.kasittele(Syote::Nuoli(0, 1));
        assert_eq!(peli.vaikeus, Vaikeus::Vaikea);
        peli.kasittele(Syote::Nuoli(0, -1));
        assert_eq!(peli.vaikeus, Vaikeus::Normaali);
        peli.kasittele(Syote::Numero(0));
        peli.kasittele(Syote::Numero(4));
        assert_eq!(peli.vaikeus, Vaikeus::Normaali);

        let (x, y) = piirto::vaikeuden_keskipiste(Vaikeus::Helppo);
        peli.kasittele(Syote::Klikkaus(x, y));
        assert_eq!(peli.vaikeus, Vaikeus::Helppo);
        assert_eq!(peli.vaihe, Vaihe::Alku);

        peli.kasittele(Syote::Klikkaus(0, 0));
        assert_eq!(peli.vaihe, Vaihe::Sijoitus);
        peli.vaihe = Vaihe::Voitto;
        peli.kasittele(Syote::Kirjain('K'));
        assert_eq!(peli.vaikeus, Vaikeus::Helppo);
    }

    #[test]
    fn numerot_eivat_vaihda_tasoa_kesken_pelin() {
        let mut peli = pelissa(1);
        peli.kasittele(Syote::Numero(1));
        assert_eq!(peli.vaikeus, Vaikeus::Normaali);
        assert_eq!(peli.kursori.rivi, 0);
    }

    /// Ampuu ruudut järjestyksessä, kunnes peli päättyy. `None`, jos se päättyi tappioon.
    fn voitettu(siemen: u64) -> Option<Peli> {
        let mut peli = pelissa(siemen);
        for i in 0..KOKO * KOKO {
            peli.ammu(Ruutu::new(i / KOKO, i % KOKO).unwrap());
            if peli.vaihe != Vaihe::Ammunta {
                break;
            }
        }
        (peli.vaihe != Vaihe::Tappio).then_some(peli)
    }

    #[test]
    fn listalle_yltava_voitto_kysyy_nimen_ja_kirjaa_pisteet() {
        let mut peli = (0..).find_map(voitettu).unwrap();
        assert_eq!(peli.vaihe, Vaihe::Nimi);
        let pisteet = pisteet(peli.vaikeus, peli.laukauksia());
        assert_eq!(peli.pisteet(), pisteet);
        // Välilyönti ei kuittaa tyhjää nimeä, eivätkä kirjaimet ammu tai lopeta peliä.
        assert!(!peli.kasittele(Syote::Ammu));
        peli.kasittele(Syote::Kirjain('A'));
        peli.kasittele(Syote::VaihdaAanet);
        peli.kasittele(Syote::Kirjain('E'));
        assert!(peli.aanet_paalla);
        assert!(!peli.lopeta);
        peli.kasittele(Syote::Ammu);
        assert_eq!(peli.vaihe, Vaihe::Voitto);
        let tulos = Tulos {
            nimi: "AME".to_string(),
            pisteet,
        };
        assert_eq!(peli.parhaat, std::slice::from_ref(&tulos));
        assert_eq!(peli.lahetys, Some(tulos));
        assert!(peli.uusi_tulos);
        assert_eq!(peli.pisteet(), pisteet);
    }

    #[test]
    fn listalle_yltamaton_voitto_ja_tappio_eivat_kysy_nimea() {
        let taysi = vec![
            Tulos {
                nimi: "X".to_string(),
                pisteet: 999,
            };
            3
        ];
        let mut peli = pelissa(0);
        peli.netti_parhaat = Some(taysi);
        for i in 0..KOKO * KOKO {
            peli.ammu(Ruutu::new(i / KOKO, i % KOKO).unwrap());
            if peli.vaihe != Vaihe::Ammunta {
                break;
            }
        }
        assert!(matches!(peli.vaihe, Vaihe::Voitto | Vaihe::Tappio));
        assert_eq!(peli.lahetys, None);
        if peli.vaihe == Vaihe::Tappio {
            assert_eq!(peli.pisteet(), 0);
        }
    }

    #[test]
    fn ikkunan_sulkeminen_kirjaa_nimea_odottavan_voiton() {
        let mut peli = (0..).find_map(voitettu).unwrap();
        Pelinakyma::viimeistele(&mut peli);
        assert_eq!(peli.parhaat[0].nimi, "NIMETÖN");
        let listat = peli.listat();
        assert_eq!(listat[0].peli, Nettipeli::Laivanupotus);
        assert!(listat[0].lahetys.as_ref().unwrap().is_some());
    }

    #[test]
    fn tilaston_paras_voitto_muuttuu_pisteiksi() {
        let tilasto = Tilasto {
            voitot: 2,
            tappiot: 1,
            vahiten_laukauksia: 60,
        };
        assert_eq!(tilasto.parhaat_pisteet(Vaikeus::Vaikea), 123);
        assert_eq!(Tilasto::default().parhaat_pisteet(Vaikeus::Vaikea), 0);
    }

    #[test]
    fn esc_lopettaa_missa_tahansa_vaiheessa() {
        let mut peli = Peli::new(1);
        peli.kasittele(Syote::Peru);
        assert!(peli.lopeta);
    }
}
