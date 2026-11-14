//! 8x8-bittikarttafontti. Merkit ovat ASCII-kuvina tiedostossa `fontti.txt`, josta ne
//! luetaan kerran. Jokainen tavu on yksi pikselirivi, vähiten merkitsevä bitti on vasemmalla.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::LazyLock;

use super::pehmennys::{Peitto, pehmenna};

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

thread_local! {
    /// Pehmennetyt merkit koon mukaan: kukin lasketaan ensimmäisellä piirtokerralla.
    static PEITOT: RefCell<HashMap<(char, i32, i32), Rc<Peitto>>> = RefCell::new(HashMap::new());
}

/// Merkin pehmennetty peittokuva, kun fontin pikseli on `pl` x `pk` pikseliä.
pub fn peitto(merkki: char, pl: i32, pk: i32) -> Rc<Peitto> {
    PEITOT.with_borrow_mut(|peitot| {
        peitot
            .entry((merkki, pl, pk))
            .or_insert_with(|| {
                let kuvio = kuvio(merkki);
                Rc::new(pehmenna(LEVEYS, 8, (pl, pk), |rivi, sarake| {
                    (kuvio[rivi as usize] >> sarake & 1 != 0).then_some(0xFFFFFF)
                }))
            })
            .clone()
    })
}

/// Merkkien koostamisen tunniste: merkki, fontin pikseli, rivien värit, varjon väri ja siirrot.
type Koosteavain = (char, i32, i32, [u32; 8], u32, &'static [(i32, i32)]);

thread_local! {
    static KOOSTEET: RefCell<HashMap<Koosteavain, Rc<Peitto>>> = RefCell::new(HashMap::new());
}

/// Välimuistin yläraja: vaihtuvat värit tekevät joka värille omat merkkinsä.
const KOOSTEITA_ENINTAAN: usize = 4096;

/// Merkki valmiiksi koostettuna: tumma `varjo` kussakin siirrossa `varjot` (fontin
/// pikseleinä) ja sen päällä merkki, jonka rivi `r` on väriä `rivivarit[r]`. Kuvassa on
/// yhden fontin pikselin reunus, joten se piirretään kohtaan (x - pl, y - pk). Teksti
/// piirtyy näin yhdellä kertaa, vaikka ääriviivassa on viisi kerrosta.
pub fn koostettu(
    merkki: char,
    (pl, pk): (i32, i32),
    rivivarit: [u32; 8],
    varjo: u32,
    varjot: &'static [(i32, i32)],
) -> Rc<Peitto> {
    let avain = (merkki, pl, pk, rivivarit, varjo, varjot);
    if let Some(valmis) = KOOSTEET.with_borrow(|k| k.get(&avain).cloned()) {
        return valmis;
    }
    let pohja = peitto(merkki, pl, pk);
    let (ul, uk) = (pl as usize, pk as usize);
    let (leveys, korkeus) = (pohja.leveys + 2 * ul, pohja.korkeus + 2 * uk);
    let mut kooste = Peitto::uusi(leveys, korkeus, vec![(0, 0); leveys * korkeus]);
    for &(dx, dy) in varjot {
        let kohta = ((1 + dx) as usize * ul, (1 + dy) as usize * uk);
        kooste.maalaa(kohta, &pohja, |_| varjo);
    }
    kooste.maalaa((ul, uk), &pohja, |rivi| rivivarit[rivi / uk]);
    let kooste = Rc::new(kooste.viimeistele());
    KOOSTEET.with_borrow_mut(|k| {
        if k.len() >= KOOSTEITA_ENINTAAN {
            k.clear();
        }
        k.insert(avain, kooste.clone());
    });
    kooste
}

#[cfg(test)]
mod tests;
