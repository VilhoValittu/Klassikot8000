//! 8x8-bittikarttafontti. Merkit ovat ASCII-kuvina tiedostossa `fontti.txt`, josta ne
//! luetaan kerran. Jokainen tavu on yksi pikselirivi, vähiten merkitsevä bitti on vasemmalla.

use std::sync::LazyLock;

pub const LEVEYS: i32 = 8;

const TYHJA: [u8; 8] = [0; 8];

struct Kirjasin {
    ascii: [Option<[u8; 8]>; 128],
    /// Ääkköset ja muut ASCII:n ulkopuoliset merkit.
    muut: Vec<(char, [u8; 8])>,
}

static KIRJASIN: LazyLock<Kirjasin> = LazyLock::new(|| lue(include_str!("fontti.txt")));

/// Jokainen merkki on rivi `= <merkki>` ja sen alla kahdeksan kuvioriviä. Muut rivit,
/// kuten tyhjät ja kommentit, ohitetaan merkkien välissä.
fn lue(teksti: &str) -> Kirjasin {
    let mut kirjasin = Kirjasin {
        ascii: [None; 128],
        muut: Vec::new(),
    };
    let mut rivit = teksti.lines();
    while let Some(rivi) = rivit.next() {
        let Some(otsikko) = rivi.strip_prefix("= ") else {
            continue;
        };
        let mut merkit = otsikko.chars();
        let (Some(merkki), None) = (merkit.next(), merkit.next()) else {
            panic!("fontin otsikossa on oltava yksi merkki: {rivi:?}");
        };
        let mut kuvio = TYHJA;
        for tavu in &mut kuvio {
            let rivi = rivit.next().expect("fontin merkistä puuttuu rivejä");
            assert_eq!(rivi.len(), 8, "fontin rivi ei ole 8 merkkiä: {merkki}");
            for (sarake, kohta) in rivi.bytes().enumerate() {
                match kohta {
                    b'#' => *tavu |= 1 << sarake,
                    b'.' => {}
                    _ => panic!("fontin rivillä on vieras merkki: {merkki}"),
                }
            }
        }
        match kirjasin.ascii.get_mut(merkki as usize) {
            Some(paikka) => *paikka = Some(kuvio),
            None => kirjasin.muut.push((merkki, kuvio)),
        }
    }
    kirjasin
}

/// Tuntematon merkki piirtyy kysymysmerkkinä.
pub fn kuvio(merkki: char) -> &'static [u8; 8] {
    if merkki == ' ' {
        return &TYHJA;
    }
    let kirjasin = &*KIRJASIN;
    let loytyi = match kirjasin.ascii.get(merkki as usize) {
        Some(kuvio) => kuvio.as_ref(),
        None => kirjasin
            .muut
            .iter()
            .find(|(m, _)| *m == merkki)
            .map(|(_, kuvio)| kuvio),
    };
    loytyi.unwrap_or_else(|| {
        kirjasin.ascii['?' as usize]
            .as_ref()
            .expect("fontissa on kysymysmerkki")
    })
}

#[cfg(test)]
mod tests;
