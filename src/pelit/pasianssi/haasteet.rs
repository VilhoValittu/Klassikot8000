//! Pasianssien haasteet: valmiiksi pelatut tilanteet, joihin pelaaja liittyy kesken pelin.
//! `haasteet.txt` tulee ohjelmasta `examples/tee_haasteet.rs`, ja `tests/haasteet.rs` pelaa
//! jokaisen läpi tallennetulla ratkaisulla. Siksi jokainen haaste on varmasti voitettavissa.

use crate::kortit::{KUNINGAS, Kortti, MAITA, Maa};
use crate::pasianssi::{self, SUURIN_TULOS, Sarake, Vaikeus, Variantti, aikapisteet};
use crate::{pyramidi, spider};

pub const HAASTEITA: usize = 20;
/// Kaikkien lajien haasteet parhaalla mahdollisella tuloksella. Palvelimen raja on sama.
pub const SUURIN_SUMMA: u32 = SUURIN_TULOS * (HAASTEITA * Variantti::KAIKKI.len()) as u32;
const HELPPOJA: usize = 7;
const NORMAALEJA: usize = 7;
const DATA: &str = include_str!("haasteet.txt");
const ARVOT: &str = "A23456789TJQK";
const MAAT: &str = "HRIP";

/// Haasteet vaikeutuvat järjestyksessä, ja taso määrää säännöt ja pistekertoimen kuten tavallisessa pelissä.
pub fn taso(haaste: usize) -> Vaikeus {
    match haaste {
        i if i < HELPPOJA => Vaikeus::Helppo,
        i if i < HELPPOJA + NORMAALEJA => Vaikeus::Normaali,
        _ => Vaikeus::Vaikea,
    }
}

pub fn pisteet(haaste: usize, sekunnit: u32) -> u32 {
    aikapisteet(taso(haaste), sekunnit)
}

fn lajin_tunnus(variantti: Variantti) -> &'static str {
    match variantti {
        Variantti::Klondike => "klondike",
        Variantti::Spider => "spider",
        Variantti::Pyramidi => "pyramidi",
    }
}

fn rivi(variantti: Variantti, haaste: usize) -> Option<std::str::SplitWhitespace<'static>> {
    let tunnus = lajin_tunnus(variantti);
    DATA.lines()
        .filter_map(|rivi| rivi.strip_prefix(tunnus)?.strip_prefix(' '))
        .nth(haaste)
        .map(str::split_whitespace)
}

fn maa(merkki: char) -> Option<Maa> {
    MAAT.find(merkki).map(|i| Maa::KAIKKI[i])
}

fn kortit(teksti: &str) -> Option<Vec<Kortti>> {
    if teksti == "-" {
        return Some(Vec::new());
    }
    let merkit: Vec<char> = teksti.chars().collect();
    merkit
        .chunks(2)
        .map(|pari| {
            let arvo = ARVOT.find(*pari.first()?)? as u8 + 1;
            Some(Kortti {
                maa: maa(*pari.get(1)?)?,
                arvo,
            })
        })
        .collect()
}

fn sarake(teksti: &str) -> Option<Sarake> {
    let (piilossa, korttilista) = teksti.split_once(':')?;
    Some(Sarake {
        kortit: kortit(korttilista)?,
        piilossa: piilossa.parse().ok()?,
    })
}

fn sarakkeet<const N: usize>(osat: &mut impl Iterator<Item = &'static str>) -> Option<[Sarake; N]> {
    let mut sarakkeet: [Sarake; N] = std::array::from_fn(|_| Sarake::default());
    for paikka in &mut sarakkeet {
        *paikka = sarake(osat.next()?)?;
    }
    Some(sarakkeet)
}

pub fn klondike(haaste: usize) -> Option<pasianssi::Lauta> {
    let mut osat = rivi(Variantti::Klondike, haaste)?;
    let kierratykset = osat.next()?.parse().ok()?;
    let pakka = kortit(osat.next()?)?;
    let jate = kortit(osat.next()?)?;
    let mut perustat: [Vec<Kortti>; MAITA] = Default::default();
    let mut maarat = osat.next()?.split(',');
    for (pino, &maa) in perustat.iter_mut().zip(&Maa::KAIKKI) {
        let maara: u8 = maarat.next()?.parse().ok()?;
        *pino = (1..=maara.min(KUNINGAS))
            .map(|arvo| Kortti { maa, arvo })
            .collect();
    }
    let tila = pasianssi::Tila {
        pakka,
        jate,
        perustat,
        sarakkeet: sarakkeet(&mut osat)?,
        kierratykset,
        siirrot: 0,
    };
    Some(pasianssi::Lauta::kesken(taso(haaste), tila))
}

pub fn spider(haaste: usize) -> Option<spider::Lauta> {
    let mut osat = rivi(Variantti::Spider, haaste)?;
    let pakka = kortit(osat.next()?)?;
    let valmiit = osat.next()?;
    let valmiit = if valmiit == "-" {
        Vec::new()
    } else {
        valmiit.chars().map(maa).collect::<Option<Vec<Maa>>>()?
    };
    let tila = spider::Tila {
        pakka,
        sarakkeet: sarakkeet(&mut osat)?,
        valmiit,
        siirrot: 0,
    };
    Some(spider::Lauta::kesken(taso(haaste), tila))
}

pub fn pyramidi(haaste: usize) -> Option<pyramidi::Lauta> {
    let mut osat = rivi(Variantti::Pyramidi, haaste)?;
    let kierratykset = osat.next()?.parse().ok()?;
    let pakka = kortit(osat.next()?)?;
    let jate = kortit(osat.next()?)?;
    let mut rivit = [[None; pyramidi::RIVEJA]; pyramidi::RIVEJA];
    let mut paikat = osat.next()?.split(',');
    for (r, kortit_rivilla) in rivit.iter_mut().enumerate() {
        for paikka in &mut kortit_rivilla[..=r] {
            *paikka = kortit(paikat.next()?)?.first().copied();
        }
    }
    let tila = pyramidi::Tila {
        rivit,
        pakka,
        jate,
        kierratykset,
        siirrot: 0,
    };
    Some(pyramidi::Lauta::kesken(taso(haaste), tila))
}
