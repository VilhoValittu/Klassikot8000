//! Nettilistan pyynnöt omassa säikeessään, jottei hidas yhteys nyki peliä. Epäonnistunut
//! pyyntö ei näy pelaajalle: lista pysyy ennallaan ja tulos jää jonoon seuraavaa käynnistystä varten.
//! Sama säie katsoo käynnistyksessä, onko GitHubissa uudempi julkaisu.

use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use std::time::Duration;

use ureq::Agent;
use ureq::tls::{TlsConfig, TlsProvider};

use super::allekirjoitus::allekirjoita;
use crate::yhteinen::tulokset::{Nettipeli, Tulos, lista_tekstista};

// `build.rs` kirjoittaa tähän julkaisijan avaimen peitettynä tai `None`, jos avainta ei ole.
include!(concat!(env!("OUT_DIR"), "/tulosavain.rs"));

#[cfg(not(debug_assertions))]
pub const OSOITE: &str = "https://klassikot8000-tulokset.palvelin.workers.dev";
macro_rules! julkaisut {
    () => {
        "https://github.com/VilhoValittu/Klassikot8000"
    };
}
#[cfg(not(debug_assertions))]
pub const JULKAISUT: &str = julkaisut!();
/// Sivu, jonka valikon ilmoitus uudesta versiosta avaa selaimeen.
pub const JULKAISUSIVU: &str = concat!(julkaisut!(), "/releases/latest");
/// Julkaisun tagi on tämä luku v-etuliitteellä: `julkaisu.yml` tarkistaa, että ne täsmäävät.
pub const VERSIO: &str = env!("CARGO_PKG_VERSION");
const AIKARAJA: Duration = Duration::from_secs(8);
const KELVOTON_PYYNTO: u16 = 400;
const ALLEKIRJOITUS: &str = "x-allekirjoitus";

/// Nettilistan allekirjoitusavain. Sama avain on palvelimella salaisuutena.
pub type Avain = [u8; 32];

/// Ohjelmaan käännetty avain. Ilman sitä tuloksia ei lähetetä, vaan ne jäävät jonoon.
pub fn tulosavain() -> Option<Avain> {
    TULOSAVAIN.map(|(peitetty, peite)| std::array::from_fn(|i| peitetty[i] ^ peite[i]))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Pyynto {
    Hae(Nettipeli),
    Laheta(Nettipeli, Tulos),
    /// Julkaisujen osoite, jonka alta `releases/latest` löytyy.
    HaeVersio(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Vastaus {
    Lista(Nettipeli, Vec<Tulos>),
    /// Tulos on käsitelty, eikä sitä tarvitse lähettää uudelleen.
    Lahetetty(Nettipeli, Tulos),
    /// Julkaistu versio on uudempi kuin tämä ohjelma.
    UusiVersio(String),
}

pub struct Netti {
    pyynnot: Sender<Pyynto>,
    pub vastaukset: Receiver<Vastaus>,
}

impl Netti {
    /// Säikeen luonnin epäonnistuminen tarkoittaa vain, että peliä pelataan ilman nettilistaa.
    pub fn kaynnista(osoite: &str, avain: Option<Avain>) -> Option<Self> {
        let (pyynnot, saapuvat) = channel();
        let (lahtevat, vastaukset) = channel();
        let osoite = osoite.to_string();
        thread::Builder::new()
            .spawn(move || aja(&osoite, avain, saapuvat, lahtevat))
            .ok()?;
        Some(Self {
            pyynnot,
            vastaukset,
        })
    }

    pub fn laheta(&self, pyynto: Pyynto) {
        // Säie elää yhtä kauan kuin peli, joten virhe voi tulla vain lopetuksessa.
        let _ = self.pyynnot.send(pyynto);
    }
}

fn aja(osoite: &str, avain: Option<Avain>, saapuvat: Receiver<Pyynto>, lahtevat: Sender<Vastaus>) {
    let agentti: Agent = Agent::config_builder()
        .timeout_global(Some(AIKARAJA))
        .tls_config(
            TlsConfig::builder()
                .provider(TlsProvider::NativeTls)
                .build(),
        )
        .build()
        .into();
    for pyynto in saapuvat {
        let vastaukset = kasittele(&agentti, osoite, avain.as_ref(), pyynto);
        for vastaus in vastaukset {
            if lahtevat.send(vastaus).is_err() {
                return;
            }
        }
    }
}

fn kasittele(agentti: &Agent, osoite: &str, avain: Option<&Avain>, pyynto: Pyynto) -> Vec<Vastaus> {
    match pyynto {
        Pyynto::Hae(peli) => {
            let tunnus = peli.tunnus();
            let vastaus = agentti.get(format!("{osoite}/top3?peli={tunnus}")).call();
            lue_lista(vastaus)
                .map(|lista| Vastaus::Lista(peli, lista))
                .into_iter()
                .collect()
        }
        Pyynto::Laheta(peli, tulos) => {
            // Kehitysversio ilman avainta ei lähetä: palvelin hylkäisi tuloksen.
            let Some(avain) = avain else {
                return Vec::new();
            };
            let tunnus = peli.tunnus();
            let runko = format!("{} {}", tulos.pisteet, tulos.nimi);
            let vastaus = agentti
                .post(format!("{osoite}/tulos?peli={tunnus}"))
                .content_type("text/plain; charset=utf-8")
                .header(ALLEKIRJOITUS, allekirjoita(avain, tunnus, &runko))
                .send(runko);
            match vastaus {
                // Palvelin hylkää tämän joka kerta, joten uudelleenyritys olisi turha.
                Err(ureq::Error::StatusCode(KELVOTON_PYYNTO)) => {
                    vec![Vastaus::Lahetetty(peli, tulos)]
                }
                vastaus => match lue_lista(vastaus) {
                    Some(lista) => {
                        vec![Vastaus::Lista(peli, lista), Vastaus::Lahetetty(peli, tulos)]
                    }
                    None => Vec::new(),
                },
            }
        }
        Pyynto::HaeVersio(julkaisut) => {
            // GitHub ohjaa uusimman julkaisun sivulle. Osoite kertoo tagin, joten sivua ei tarvitse
            // ladata, eikä API:n käyttörajakaan koske tätä.
            let vastaus = agentti
                .get(format!("{julkaisut}/releases/latest"))
                .config()
                .max_redirects(0)
                .build()
                .call();
            vastaus
                .ok()
                .and_then(|vastaus| {
                    let sijainti = vastaus.headers().get("location")?.to_str().ok()?;
                    uudempi_versio(sijainti, VERSIO)
                })
                .map(Vastaus::UusiVersio)
                .into_iter()
                .collect()
        }
    }
}

fn versionumero(teksti: &str) -> Option<(u32, u32, u32)> {
    let mut osat = teksti.split('.').map(|osa| osa.parse().ok());
    let numero = (osat.next()??, osat.next()??, osat.next()??);
    osat.next().is_none().then_some(numero)
}

/// Lukee version julkaisun osoitteen lopusta. Kaikki muu kuin selvästi uudempi versio ohitetaan.
fn uudempi_versio(sijainti: &str, nykyinen: &str) -> Option<String> {
    let tagi = sijainti.rsplit('/').next()?.strip_prefix('v')?;
    let uusi = versionumero(tagi)?;
    (uusi > versionumero(nykyinen)?).then(|| format!("{}.{}.{}", uusi.0, uusi.1, uusi.2))
}

fn lue_lista(vastaus: Result<ureq::http::Response<ureq::Body>, ureq::Error>) -> Option<Vec<Tulos>> {
    let teksti = vastaus.ok()?.body_mut().read_to_string().ok()?;
    Some(lista_tekstista(&teksti))
}

#[cfg(test)]
mod tests;
