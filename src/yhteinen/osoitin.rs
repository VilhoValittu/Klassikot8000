//! Hiiren paikka näkyvän näkymän koordinaateissa, jotta napit voivat korostua hiiren alla
//! ja painua painettaessa. Käynnistin kirjaa jokaisen hiiren syötteen ennen näkymää.
//!
//! Pelinäkymien kuvan avaimeen kuuluu jo syötteiden määrä, joten hiiren liike piirtää
//! näkymän uudelleen ilman näkymäkohtaista koodia.

use std::cell::Cell;

use super::syote::Syote;

thread_local! {
    // Vain ikkunan säie piirtää, ja näin testit eivät näe toistensa hiirtä.
    static PAIKKA: Cell<Option<(i32, i32)>> = const { Cell::new(None) };
    static ALHAALLA: Cell<bool> = const { Cell::new(false) };
}

/// Napin tila hiiren suhteen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tila {
    Lepo,
    Alla,
    Painettu,
}

pub fn kirjaa(syote: Syote) {
    match syote {
        Syote::Hiiri(x, y) => PAIKKA.set(Some((x, y))),
        Syote::Klikkaus(x, y) => {
            PAIKKA.set(Some((x, y)));
            ALHAALLA.set(true);
        }
        Syote::Vapautus(x, y) => {
            PAIKKA.set(Some((x, y)));
            ALHAALLA.set(false);
        }
        _ => {}
    }
}

/// Näkymä vaihtui: vanha paikka on eri koordinaateissa, joten se unohdetaan.
pub fn unohda() {
    PAIKKA.set(None);
    ALHAALLA.set(false);
}

/// Alueen `(x, y, l, k)` tila: hiiri on sen päällä, ja nappi voi olla painettuna.
pub fn tila(x: i32, y: i32, l: i32, k: i32) -> Tila {
    match PAIKKA.get() {
        Some((hx, hy)) if (x..x + l).contains(&hx) && (y..y + k).contains(&hy) => {
            if ALHAALLA.get() {
                Tila::Painettu
            } else {
                Tila::Alla
            }
        }
        _ => Tila::Lepo,
    }
}

#[cfg(test)]
mod tests;
