//! Nettilistan pyynnöt omassa säikeessään, jottei hidas yhteys nyki peliä. Epäonnistunut
//! pyyntö ei näy pelaajalle: lista pysyy ennallaan ja tulos jää jonoon seuraavaa käynnistystä varten.
//! Sama säie katsoo käynnistyksessä, onko GitHubissa uudempi julkaisu.

use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use std::time::Duration;

use ureq::Agent;
use ureq::tls::{TlsConfig, TlsProvider};

use crate::yhteinen::tulokset::{Tulos, lista_tekstista};

#[cfg(not(debug_assertions))]
pub const OSOITE: &str = "https://klassikot8000-tulokset.palvelin.workers.dev";
#[cfg(not(debug_assertions))]
pub const JULKAISUT: &str = "https://github.com/VilhoValittu/Klassikot8000";
/// Julkaisun tagi on tämä luku v-etuliitteellä: `julkaisu.yml` tarkistaa, että ne täsmäävät.
pub const VERSIO: &str = env!("CARGO_PKG_VERSION");
const AIKARAJA: Duration = Duration::from_secs(8);
const KELVOTON_PYYNTO: u16 = 400;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nettipeli {
    Luvut,
    Tiilirikko,
    Asteroidit,
    Silmukka,
    Palikka,
    Pasianssi,
    Blackjack,
    Holdem,
}

impl Nettipeli {
    /// Pelin tunnus palvelimella.
    fn tunnus(self) -> &'static str {
        match self {
            Nettipeli::Luvut => "luvut",
            Nettipeli::Tiilirikko => "tiilirikko",
            Nettipeli::Asteroidit => "asteroidit",
            Nettipeli::Silmukka => "silmukka",
            Nettipeli::Palikka => "palikka",
            Nettipeli::Pasianssi => "pasianssi",
            Nettipeli::Blackjack => "blackjack",
            Nettipeli::Holdem => "holdem",
        }
    }
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
    pub fn kaynnista(osoite: &str) -> Option<Self> {
        let (pyynnot, saapuvat) = channel();
        let (lahtevat, vastaukset) = channel();
        let osoite = osoite.to_string();
        thread::Builder::new()
            .spawn(move || aja(&osoite, saapuvat, lahtevat))
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

fn aja(osoite: &str, saapuvat: Receiver<Pyynto>, lahtevat: Sender<Vastaus>) {
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
        let vastaukset = kasittele(&agentti, osoite, pyynto);
        for vastaus in vastaukset {
            if lahtevat.send(vastaus).is_err() {
                return;
            }
        }
    }
}

fn kasittele(agentti: &Agent, osoite: &str, pyynto: Pyynto) -> Vec<Vastaus> {
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
            let tunnus = peli.tunnus();
            let vastaus = agentti
                .post(format!("{osoite}/tulos?peli={tunnus}"))
                .content_type("text/plain; charset=utf-8")
                .send(format!("{} {}", tulos.pisteet, tulos.nimi));
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
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader, Read, Write};
    use std::net::TcpListener;

    /// Paikallinen palvelin, joka antaa vastaukset järjestyksessä ja palauttaa saamansa pyynnöt.
    fn palvelin(vastaukset: Vec<String>) -> (String, thread::JoinHandle<Vec<String>>) {
        let kuuntelija = TcpListener::bind("127.0.0.1:0").unwrap();
        let osoite = format!("http://{}", kuuntelija.local_addr().unwrap());
        let saie = thread::spawn(move || {
            let mut pyynnot = Vec::new();
            for vastaus in vastaukset {
                let (mut yhteys, _) = kuuntelija.accept().unwrap();
                let mut lukija = BufReader::new(yhteys.try_clone().unwrap());
                let mut alku = String::new();
                lukija.read_line(&mut alku).unwrap();
                let mut pituus = 0;
                loop {
                    let mut rivi = String::new();
                    lukija.read_line(&mut rivi).unwrap();
                    if rivi.trim().is_empty() {
                        break;
                    }
                    if let Some(arvo) = rivi.to_lowercase().strip_prefix("content-length:") {
                        pituus = arvo.trim().parse().unwrap();
                    }
                }
                let mut runko = vec![0; pituus];
                lukija.read_exact(&mut runko).unwrap();
                pyynnot.push(format!(
                    "{} {}",
                    alku.trim(),
                    String::from_utf8(runko).unwrap()
                ));
                write!(yhteys, "{vastaus}").unwrap();
            }
            pyynnot
        });
        (osoite, saie)
    }

    fn http(status: &str, runko: &str) -> String {
        format!(
            "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{runko}",
            runko.len()
        )
    }

    fn tulos(nimi: &str, pisteet: u32) -> Tulos {
        Tulos {
            nimi: nimi.to_string(),
            pisteet,
        }
    }

    #[test]
    fn haku_ja_lahetys_kulkevat_ja_hylatty_tulos_kuitataan() {
        let lista = http("200 OK", "120 VÄINÖ\n50 B\n");
        let hylatty = http("400 Bad Request", "kelvoton tulos\n");
        let (osoite, saie) = palvelin(vec![lista.clone(), lista, hylatty]);
        let netti = Netti::kaynnista(&osoite).unwrap();
        let odotettu = vec![tulos("VÄINÖ", 120), tulos("B", 50)];

        netti.laheta(Pyynto::Hae(Nettipeli::Tiilirikko));
        assert_eq!(
            netti.vastaukset.recv().unwrap(),
            Vastaus::Lista(Nettipeli::Tiilirikko, odotettu.clone())
        );

        netti.laheta(Pyynto::Laheta(Nettipeli::Luvut, tulos("VÄINÖ", 120)));
        assert_eq!(
            netti.vastaukset.recv().unwrap(),
            Vastaus::Lista(Nettipeli::Luvut, odotettu)
        );
        assert_eq!(
            netti.vastaukset.recv().unwrap(),
            Vastaus::Lahetetty(Nettipeli::Luvut, tulos("VÄINÖ", 120))
        );

        netti.laheta(Pyynto::Laheta(Nettipeli::Tiilirikko, tulos("X", 1)));
        assert_eq!(
            netti.vastaukset.recv().unwrap(),
            Vastaus::Lahetetty(Nettipeli::Tiilirikko, tulos("X", 1))
        );

        let pyynnot = saie.join().unwrap();
        assert!(pyynnot[0].starts_with("GET /top3?peli=tiilirikko "));
        assert!(pyynnot[1].starts_with("POST /tulos?peli=luvut "));
        assert!(pyynnot[1].ends_with(" 120 VÄINÖ"));
    }

    #[test]
    fn vain_uudempi_kelvollinen_versio_kelpaa() {
        let polku = "https://github.com/x/y/releases/tag/";
        let uudempi = |tagi: &str| uudempi_versio(&format!("{polku}{tagi}"), "0.3.6");
        assert_eq!(uudempi("v0.3.7").as_deref(), Some("0.3.7"));
        assert_eq!(uudempi("v0.4.0").as_deref(), Some("0.4.0"));
        assert_eq!(uudempi("v1.0.0").as_deref(), Some("1.0.0"));
        // luvut vertaillaan lukuina eikä kirjaimina
        assert_eq!(uudempi("v0.3.10").as_deref(), Some("0.3.10"));
        assert_eq!(uudempi("v0.3.6"), None);
        assert_eq!(uudempi("v0.3.5"), None);
        assert_eq!(uudempi("v0.2.99"), None);
        for kelvoton in ["0.3.7", "v0.3", "v0.3.7.1", "v0.3.x", "v0.4.0-beta", "v"] {
            assert_eq!(uudempi(kelvoton), None, "{kelvoton}");
        }
        // ilman julkaisuja GitHub ohjaa julkaisujen listaan
        assert_eq!(
            uudempi_versio("https://github.com/x/y/releases", "0.3.6"),
            None
        );
        assert_eq!(uudempi_versio(&format!("{polku}v9.0.0"), "kehitys"), None);
    }

    #[test]
    fn versio_luetaan_uudelleenohjauksesta_jota_ei_seurata() {
        let ohjaus = |tagi: &str| {
            format!(
                "HTTP/1.1 302 Found\r\nLocation: http://127.0.0.1:9/releases/tag/{tagi}\r\n\
                 Content-Length: 0\r\nConnection: close\r\n\r\n"
            )
        };
        let (osoite, saie) = palvelin(vec![
            ohjaus("v999.0.0"),
            ohjaus("v0.0.1"),
            http("200 OK", ""),
            http("404 Not Found", ""),
        ]);
        let agentti = Agent::new_with_defaults();
        let hae = || kasittele(&agentti, &osoite, Pyynto::HaeVersio(osoite.clone()));
        assert_eq!(hae(), [Vastaus::UusiVersio("999.0.0".to_string())]);
        assert!(hae().is_empty());
        assert!(hae().is_empty());
        assert!(hae().is_empty());
        let pyynnot = saie.join().unwrap();
        assert!(pyynnot[0].starts_with("GET /releases/latest "));
    }

    #[test]
    fn palvelinvirhe_ei_kuittaa_tulosta_lahetetyksi() {
        let (osoite, saie) = palvelin(vec![http("500 Oops", "")]);
        let agentti = Agent::new_with_defaults();
        let pyynto = Pyynto::Laheta(Nettipeli::Luvut, tulos("A", 1));
        assert!(kasittele(&agentti, &osoite, pyynto).is_empty());
        saie.join().unwrap();
    }

    #[test]
    fn yhteysvirhe_ei_tuota_vastausta() {
        let kuuntelija = TcpListener::bind("127.0.0.1:0").unwrap();
        let osoite = format!("http://{}", kuuntelija.local_addr().unwrap());
        drop(kuuntelija);
        let agentti = Agent::new_with_defaults();
        assert!(kasittele(&agentti, &osoite, Pyynto::Hae(Nettipeli::Luvut)).is_empty());
    }
}
