//! Laivanupotuksen säännöt.

pub mod laivasto;
pub mod ruutu;
pub mod vastustaja;

pub use laivasto::{Laiva, Laivasto, Laukaustulos};
pub use ruutu::{Ruudut, Ruutu};
pub use vastustaja::Vaikeus;

pub const KOKO: usize = 10;
pub const SARAKKEET: &str = "ABCDEFGHIJ";
pub const LAIVAT: [(&str, usize); 5] = [
    ("Lentotukialus", 5),
    ("Taistelulaiva", 4),
    ("Hävittäjä", 3),
    ("Sukellusvene", 3),
    ("Partioalus", 2),
];

const RUUTUJA: u32 = (KOKO * KOKO) as u32;
/// Laivojen ruutujen määrä: nopein mahdollinen voitto.
pub const LAIVARUUTUJA: u32 = 17;
/// Vaikealla tasolla nopeimmasta voitosta saatava tulos, top-listan yläraja.
pub const SUURIN_TULOS: u32 = pisteet(Vaikeus::Vaikea, LAIVARUUTUJA);

/// Voiton tulos top-listalle: jokainen säästetty laukaus tuo pisteen, ja taso kertoo sen
/// (normaali kahdella, vaikea kolmella). Voitto viimeisellä mahdollisella laukauksella antaa pisteen.
pub const fn pisteet(vaikeus: Vaikeus, laukauksia: u32) -> u32 {
    let ammutut = if laukauksia < RUUTUJA {
        laukauksia
    } else {
        RUUTUJA
    };
    (RUUTUJA + 1 - ammutut) * vaikeus.kerroin()
}
