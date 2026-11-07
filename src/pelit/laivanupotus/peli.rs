use klassikot8000::Arpa;
use klassikot8000::laivanupotus::{
    KOKO, LAIVAT, Laivasto, Laukaustulos, Ruudut, Ruutu, SARAKKEET, Vaikeus, pisteet, vastustaja,
};

use super::piirto;
use crate::kaynnistin::tallennus::{Asetukset, Tallennus};
use crate::yhteinen::aanet::{Aanet, Aani, PUHE_VIIVE_MS};
use crate::yhteinen::kieli::{kaanna, muotoile};
use crate::yhteinen::pelinakyma::Pelinakyma;
use crate::yhteinen::piirto::{Kehys, aanikytkimella};
use crate::yhteinen::syote::{Syote, Yleinen, yleinen};
use crate::yhteinen::tulokset::{Nettipeli, Tulostaulu, muokkaa_nimea};

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
    /// Soitettavat äänet ja niiden viiveet millisekunteina. `main.rs` tyhjentää jonon.
    pub aanet: Aanet,
    pub tehosteet: Vec<Tehoste>,
    /// Pelin alusta kulunut aika, josta kaikki animaatiot lasketaan.
    pub kello_ms: u32,
    /// Tasoittain, järjestys kuten `Vaikeus::KAIKKI`.
    pub tilastot: [Tilasto; 3],
    /// Tosi, kun päättynyt peli on kirjattu tilastoon: käynnistin tallentaa sen.
    pub uusi_tulos: bool,
    pub taulu: Tulostaulu,
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
            aanet: Aanet::default(),
            tehosteet: Vec::new(),
            kello_ms: 0,
            tilastot: Default::default(),
            uusi_tulos: false,
            taulu: Tulostaulu::new(Nettipeli::Laivanupotus),
            nimi: String::new(),
            arpa,
        }
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
        if self.taulu.mahtuu(pisteet) {
            self.nimi.clear();
            self.vaihe = Vaihe::Nimi;
        }
    }

    /// Kirjaa nimeä odottavan voiton.
    pub fn vahvista_nimi(&mut self) {
        if self.vaihe != Vaihe::Nimi {
            return;
        }
        let pisteet = self.pisteet();
        self.taulu.kirjaa(&mut self.nimi, pisteet);
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
            self.aanet.jono.push((Aani::Virhe, 0));
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
            _ => self.aanet.jono.push((Aani::Virhe, 0)),
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
        self.aanet.jono.push((Aani::Alku, 0));
        true
    }

    /// Käsittelee syötteen ja kertoo, muuttuiko näkymä.
    pub fn kasittele(&mut self, syote: Syote) -> bool {
        match yleinen(
            syote,
            self.vaihe == Vaihe::Nimi,
            &mut self.aanet.paalla,
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
            self.aanet.jono.push((Aani::Virhe, 0));
            return;
        };
        self.ammuttu_jo = false;
        self.viestit = [
            tulos_teksti("laivanupotus.sina_ammuit", ruutu, tulos),
            String::new(),
        ];
        self.aanet.jono.push((Aani::Laukaus, 0));
        self.aanet.jono.push((tuloksen_aani(tulos), TULOS_VIIVE_MS));
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
            self.aanet.jono.push((Aani::Voitto, LOPPU_VIIVE_MS));
            self.aanet
                .jono
                .push((Aani::Voitit, LOPPU_VIIVE_MS + PUHE_VIIVE_MS));
            return;
        }

        let laukaus = vastustaja::valitse(self.vaikeus, &self.oma.tiedot(), &mut self.arpa);
        let tulos = laukaus.and_then(|r| Some((r, self.oma.kasittele_laukaus(r)?)));
        if let Some((ruutu, tulos)) = tulos {
            self.viestit[1] = tulos_teksti("laivanupotus.vastustaja_ampui", ruutu, tulos);
            self.aanet.jono.push((Aani::Vastalaukaus, VASTAUS_VIIVE_MS));
            self.aanet
                .jono
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
            self.aanet
                .jono
                .push((Aani::Tappio, VASTAUS_VIIVE_MS + LOPPU_VIIVE_MS));
            self.aanet.jono.push((
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
        self.vahvista_nimi();
    }

    fn listat(&mut self) -> Vec<&mut Tulostaulu> {
        vec![&mut self.taulu]
    }
}

#[cfg(test)]
mod tests;
