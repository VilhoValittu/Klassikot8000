//! Parhaiden tulosten lista ja nimimerkin kirjoitus.

use super::kieli::kaanna;
use super::paivays::{SUURIN_AIKA, nyt};
use super::syote::Syote;

pub const PARHAITA: usize = 3;
pub const NIMEN_PITUUS: usize = 8;
const OLETUSNIMI: &str = "yhteinen.oletusnimi";
// Raja pitää muokatun tiedoston tai vieraan vastauksen luvut sellaisina, ettei mikään voi ylivuotaa.
const SUURIN_PISTEMAARA: u32 = 1_000_000_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tulos {
    pub nimi: String,
    pub pisteet: u32,
    /// Tuloksen hetki Unix-sekunteina. Vanhoissa tallennuksissa ja vanhalta palvelimelta `None`.
    pub aika: Option<u64>,
}

/// Top-lista, jolla on oma rivinsä tallennuksessa ja oma tunnuksensa nettipalvelimella.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nettipeli {
    Luvut,
    Tiilirikko,
    Asteroidit,
    Silmukka,
    Palikka,
    Pasianssi,
    Spider,
    Pyramidi,
    Haasteet,
    Blackjack,
    Holdem,
    Miinaharava,
    Laivanupotus,
    Mahjong,
    Majakka,
    Kaivo,
    Jatsi,
}

pub const NETTIPELEJA: usize = Nettipeli::KAIKKI.len();

impl Nettipeli {
    /// Järjestys on sama kuin luvuilla, joten `peli as usize` osoittaa listaan.
    pub const KAIKKI: [Nettipeli; 17] = [
        Nettipeli::Luvut,
        Nettipeli::Tiilirikko,
        Nettipeli::Asteroidit,
        Nettipeli::Silmukka,
        Nettipeli::Palikka,
        Nettipeli::Pasianssi,
        Nettipeli::Spider,
        Nettipeli::Pyramidi,
        Nettipeli::Haasteet,
        Nettipeli::Blackjack,
        Nettipeli::Holdem,
        Nettipeli::Miinaharava,
        Nettipeli::Laivanupotus,
        Nettipeli::Mahjong,
        Nettipeli::Majakka,
        Nettipeli::Kaivo,
        Nettipeli::Jatsi,
    ];

    /// Pelin tunnus palvelimella.
    pub fn tunnus(self) -> &'static str {
        match self {
            Nettipeli::Luvut => "luvut",
            Nettipeli::Tiilirikko => "tiilirikko",
            Nettipeli::Asteroidit => "asteroidit",
            Nettipeli::Silmukka => "silmukka",
            Nettipeli::Palikka => "palikka",
            Nettipeli::Pasianssi => "pasianssi",
            Nettipeli::Spider => "spider",
            Nettipeli::Pyramidi => "pyramidi",
            Nettipeli::Haasteet => "haasteet",
            Nettipeli::Blackjack => "blackjack",
            Nettipeli::Holdem => "holdem",
            Nettipeli::Miinaharava => "miinaharava",
            Nettipeli::Laivanupotus => "laivanupotus",
            Nettipeli::Mahjong => "mahjong",
            Nettipeli::Majakka => "majakka",
            Nettipeli::Kaivo => "kaivo",
            Nettipeli::Jatsi => "jatsi",
        }
    }

    /// Listalla on kullakin nimellä yksi rivi, jonka uusi tulos korvaa.
    pub fn yksi_per_nimi(self) -> bool {
        self == Nettipeli::Haasteet
    }
}

/// Lukee rivin muodossa "pisteet NIMI" tai "pisteet NIMI aika". Tallennustiedosto ja nettilista
/// käyttävät samaa muotoa.
pub fn tulos_tekstista(teksti: &str) -> Option<Tulos> {
    let mut osat = teksti.split(' ');
    let pisteet = osat
        .next()?
        .parse()
        .ok()
        .filter(|&p| p > 0 && p <= SUURIN_PISTEMAARA)?;
    let nimi = osat.next().filter(|nimi| nimi_kelpaa(nimi))?;
    let aika = match osat.next() {
        Some(aika) => Some(aika.parse().ok().filter(|&a| a <= SUURIN_AIKA)?),
        None => None,
    };
    osat.next().is_none().then(|| Tulos {
        nimi: nimi.to_string(),
        pisteet,
        aika,
    })
}

/// Tulos samassa muodossa, jonka `tulos_tekstista` lukee.
pub fn tulos_tekstiksi(tulos: &Tulos) -> String {
    match tulos.aika {
        Some(aika) => format!("{} {} {aika}", tulos.pisteet, tulos.nimi),
        None => format!("{} {}", tulos.pisteet, tulos.nimi),
    }
}

/// Nimimerkissä on 1-`NIMEN_PITUUS` isoa kirjainta tai numeroa.
pub fn nimi_kelpaa(nimi: &str) -> bool {
    (1..=NIMEN_PITUUS).contains(&nimi.chars().count())
        && nimi.chars().all(|m| m.is_uppercase() || m.is_ascii_digit())
}

pub fn lista_tekstista(teksti: &str) -> Vec<Tulos> {
    let mut lista = Vec::new();
    for tulos in teksti.lines().filter_map(tulos_tekstista) {
        lisaa_tulos(&mut lista, tulos);
    }
    lista
}

/// Tasapisteissä vanhempi tulos säilyttää paikkansa, joten sen ohi ei pääse.
pub fn mahtuu_parhaisiin(parhaat: &[Tulos], pisteet: u32) -> bool {
    pisteet > 0 && (parhaat.len() < PARHAITA || parhaat.iter().any(|t| t.pisteet < pisteet))
}

pub fn lisaa_tulos(parhaat: &mut Vec<Tulos>, tulos: Tulos) {
    let paikka = parhaat.partition_point(|t| t.pisteet >= tulos.pisteet);
    parhaat.insert(paikka, tulos);
    parhaat.truncate(PARHAITA);
}

/// Listalle, jossa kullakin nimellä on yksi rivi: uusi tulos korvaa saman nimen vanhan.
pub fn paivita_nimen_tulos(parhaat: &mut Vec<Tulos>, tulos: Tulos) {
    parhaat.retain(|t| t.nimi != tulos.nimi);
    lisaa_tulos(parhaat, tulos);
}

/// Pelin yksi top-lista: oma ennätys, omat parhaat, nettilista ja lähtevä tulos. Käynnistin
/// täyttää sen tallennuksesta ja nettilistasta ja kirjaa siitä tallennukseen ja jonoon.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tulostaulu {
    pub peli: Nettipeli,
    /// Oma ennätys, myös listalle yltämätön. Pysyy nollana, jos peli ei pidä sitä.
    pub paras: u32,
    pub parhaat: Vec<Tulos>,
    /// Nettilista. `None`, kunnes palvelin on vastannut.
    pub netti: Option<Vec<Tulos>>,
    /// Kirjatut tulokset, jotka odottavat lähetystä nettilistalle. Moninpelistä niitä voi
    /// tulla kerralla useampi.
    pub lahetys: Vec<Tulos>,
}

impl Tulostaulu {
    pub fn new(peli: Nettipeli) -> Self {
        Self {
            peli,
            paras: 0,
            parhaat: Vec::new(),
            netti: None,
            lahetys: Vec::new(),
        }
    }

    /// Esillä oleva lista: nettilista, tai oma lista, jos verkkoon ei ole saatu yhteyttä.
    pub fn nakyva(&self) -> &[Tulos] {
        self.netti.as_deref().unwrap_or(&self.parhaat)
    }

    pub fn mahtuu(&self, pisteet: u32) -> bool {
        mahtuu_parhaisiin(self.nakyva(), pisteet)
    }

    /// Kirjaa tuloksen kirjoitetulla nimellä omalle listalle, nettilistalle ja lähtemään.
    pub fn kirjaa(&mut self, nimi: &mut String, pisteet: u32) {
        self.lisaa(Tulos {
            nimi: ota_nimi(nimi),
            pisteet,
            aika: None,
        });
    }

    /// Lisää tuloksen omalle listalle, nettilistalle ja lähtemään. Tulos ilman aikaa saa
    /// tämän hetken.
    pub fn lisaa(&mut self, mut tulos: Tulos) {
        tulos.aika.get_or_insert_with(nyt);
        let lisaa = if self.peli.yksi_per_nimi() {
            paivita_nimen_tulos
        } else {
            lisaa_tulos
        };
        lisaa(&mut self.parhaat, tulos.clone());
        if let Some(lista) = &mut self.netti {
            lisaa(lista, tulos.clone());
        }
        self.lahetys.push(tulos);
    }
}

/// Ottaa kirjoitetun nimen. Ikkunan sulkeminen ei saa hävittää tulosta, joten tyhjä
/// nimi korvataan oletuksella.
pub fn ota_nimi(nimi: &mut String) -> String {
    if nimi.is_empty() {
        kaanna(OLETUSNIMI).to_string()
    } else {
        std::mem::take(nimi)
    }
}

pub fn muokkaa_nimea(nimi: &mut String, syote: Syote) {
    let merkki = match syote {
        Syote::Kirjain(kirjain) => kirjain,
        // M on äänikytkimen pikanäppäin, mutta nimessä se on kirjain.
        Syote::VaihdaAanet => 'M',
        Syote::Numero(numero) => (b'0' + numero) as char,
        Syote::Pyyhi => {
            nimi.pop();
            return;
        }
        _ => return,
    };
    if nimi.chars().count() < NIMEN_PITUUS {
        nimi.push(merkki);
    }
}

/// Muokkaa nimeä syötteen mukaan ja kertoo, kuitattiinko nimi välilyönnillä, Enterillä tai
/// Escillä. Pelin viimeisenä painettu välilyönti ei saa kuitata nimeä ennen kuin sitä on
/// kirjoitettu.
pub fn nimi_valmis(nimi: &mut String, syote: Syote) -> bool {
    match syote {
        Syote::Ammu if nimi.is_empty() => false,
        Syote::Ammu | Syote::Peru => true,
        _ => {
            muokkaa_nimea(nimi, syote);
            false
        }
    }
}

#[cfg(test)]
mod tests;
