//! Mahjonggin näkymät. Alkuruutu käyttää muiden pelien kanssa samaa klassista kuvaa,
//! pelilauta piirretään suoraan täysikokoiselle ruudulle, jotta laattojen kuvat erottuvat.

use klassikot8000::mahjong::{Laatta, Lauta, Sarja, Tila, Vaikeus};

use super::peli::{JUHLA_MS, Mahjong, Nappi, PONNAHDUS_MS, Vaihe};
use crate::yhteinen::kieli::{kaanna, muotoile};
use crate::yhteinen::piirto::*;
use crate::yhteinen::vektori::{Kangas, Tahko, kuutio, tummenna, vaalenna};

// Laatta konepikseleinä: kasvo 24x30 ja kolmen pikselin paksuus vasemmalle alas.
const U: i32 = 4;
const LAATTA_L: i32 = 24 * U;
const LAATTA_K: i32 = 30 * U;
const PAKSUUS: i32 = 3 * U;
const PUOLI_L: i32 = LAATTA_L / 2;
const PUOLI_K: i32 = LAATTA_K / 2;

const SIVU_L: i32 = 468;
const SIVU_X: i32 = RUUDUN_LEVEYS as i32 - 24 - SIVU_L;
const SIVU_TEKSTI_X: i32 = SIVU_X + 18;
const SIVU_MERKKEJA: usize = 18;
const TILASTO_Y: i32 = 60;
const TILASTO_ASKEL: i32 = 36;
const TILASTOJA: usize = 6;
const VIESTI_Y: i32 = TILASTO_Y + TILASTOJA as i32 * TILASTO_ASKEL + 12;
const VIESTI_K: i32 = 4 * RIVIN_KORKEUS + 18;
const SIVU_OHJE_Y: i32 = VIESTI_Y + VIESTI_K + 18;
const NAPIT_Y: i32 = SIVU_OHJE_Y + SIVU_OHJEET.len() as i32 * RIVIN_KORKEUS + 18;
const NAPPI_ASKEL: i32 = NAPPI_K + 12;
const SIVU_PARHAAT_Y: i32 = NAPIT_Y + NAPIT.len() as i32 * NAPPI_ASKEL + 18;

// Lauta keskitetään otsikkopalkin ja sivupaneelin väliin.
const ALUE_X: i32 = 24;
const ALUE_Y: i32 = 48;
const ALUE_L: i32 = SIVU_X - 24 - ALUE_X;
const ALUE_K: i32 = RUUDUN_KORKEUS as i32 - 24 - ALUE_Y;

const LOPPU_L: i32 = 744;
const LOPPU_K: i32 = 240;

const SIVU_OHJEET: [&str; 5] = [
    "mahjong.sivu_hiiri",
    "mahjong.sivu_oikea",
    "mahjong.sivu_nuolet",
    "mahjong.sivu_aanet",
    "mahjong.sivu_valikko",
];

const NAPIT: [(Nappi, &str); 5] = [
    (Nappi::Vihje, "mahjong.nappi_vihje"),
    (Nappi::Sekoita, "mahjong.nappi_sekoita"),
    (Nappi::Peruuta, "mahjong.nappi_peruuta"),
    (Nappi::Uusi, "mahjong.nappi_uusi"),
    (Nappi::Valikko, "yhteinen.valikkoon"),
];

// Alkuruudun tasovalinta samoilla mitoilla kuin miinaharavassa.
const TASO_Y: i32 = ALOITUS_Y - NAPPI_K - 6;
const TASO_X: i32 = 48 + 6 * MERKKI;
const TASO_VALI: i32 = 12;
const TASOT: [(Vaikeus, &str, i32); 3] = [
    (Vaikeus::Helppo, "yhteinen.taso_1", TASO_X),
    (
        Vaikeus::Normaali,
        "yhteinen.taso_2",
        TASO_X + 8 * MERKKI + 24 + TASO_VALI,
    ),
    (
        Vaikeus::Vaikea,
        "yhteinen.taso_3",
        TASO_X + 18 * MERKKI + 48 + 2 * TASO_VALI,
    ),
];

const ALKU_OSAT: [&str; 4] = [
    "mahjong.tervetuloa",
    "yhteinen.ohjeet",
    "mahjong.ohje_1",
    "mahjong.ohje_2",
];
const ALKU_RIVIT: [&str; 4] = [
    "mahjong.nappain_hiiri",
    "mahjong.nappain_nuolet",
    "yhteinen.aanet_kytkin",
    "yhteinen.takaisin",
];
/// Ilmansuunnat kirjaimina: itä, etelä, länsi ja pohjoinen.
const TUULET: [&str; 4] = [
    "mahjong.tuuli_1",
    "mahjong.tuuli_2",
    "mahjong.tuuli_3",
    "mahjong.tuuli_4",
];

// Laatan värit: norsunluinen kasvo ja hunajainen puutausta, joka erottuu vihreästä verasta.
const KASVO_YLA: u32 = 0xFBF6E8;
const KASVO_ALA: u32 = 0xE4D8BC;
const PEITETTY_YLA: u32 = 0xCFC4AA;
const PEITETTY_ALA: u32 = 0xB2A68C;
const HIIRI_YLA: u32 = 0xFFFFFF;
const HIIRI_ALA: u32 = 0xF4ECD4;
const VALITTU_YLA: u32 = 0xFFF2A0;
const VALITTU_ALA: u32 = 0xF0C040;
const VIHJE_YLA: u32 = 0xD0F4FF;
const VIHJE_ALA: u32 = 0x80C8F0;
/// Kerrokset syvimmästä kasvoon päin.
const KYLKI: [u32; 3] = [0x8A4A10, 0xC8802C, 0xD8CCAE];
const AARIVIIVA: u32 = 0x1A1008;

const PUNAINEN: u32 = 0xC8242A;
const VIHREA: u32 = 0x1E8040;
const SININEN: u32 = 0x2450C0;
const LAIVASTO: u32 = 0x1A2470;
const KULTA: u32 = 0xF0B020;
const ORANSSI: u32 = 0xE86C18;
const RUSKEA: u32 = 0x7A4A1E;
const ROSA: u32 = 0xF06898;
const JAA: u32 = 0x40A0E0;
const VIOLETTI: u32 = 0x8840C0;

#[rustfmt::skip]
const NUMEROT: [[&str; 5]; 9] = [
    [".#.", "##.", ".#.", ".#.", "###"],
    ["##.", "..#", ".#.", "#..", "###"],
    ["##.", "..#", ".#.", "..#", "##."],
    ["#.#", "#.#", "###", "..#", "..#"],
    ["###", "#..", "##.", "..#", "##."],
    [".##", "#..", "###", "#.#", "###"],
    ["###", "..#", ".#.", ".#.", ".#."],
    ["###", "#.#", "###", "#.#", "###"],
    ["###", "#.#", "###", "..#", "##."],
];

const PALLO: [&str; 5] = [".###.", "#...#", "#.#.#", "#...#", ".###."];
const ISO_PALLO: [&str; 11] = [
    "...ggggg...",
    "..g.....g..",
    ".g..rrr..g.",
    "g..r...r..g",
    "g.r..b..r.g",
    "g.r.bbb.r.g",
    "g.r..b..r.g",
    "g..r...r..g",
    ".g..rrr..g.",
    "..g.....g..",
    "...ggggg...",
];
/// Kuvion keskipisteet konepikseleinä laatan keskeltä.
type Paikat = &'static [(i32, i32)];

/// Pallojen keskipisteet laatan keskeltä ja värit (g vihreä, b sininen, r punainen).
const PALLOJEN_PAIKAT: [(Paikat, &str); 8] = [
    (&[(0, -4), (0, 4)], "gb"),
    (&[(-6, -6), (0, 0), (6, 6)], "brg"),
    (&[(-3, -3), (3, -3), (-3, 3), (3, 3)], "bggb"),
    (&[(-6, -6), (6, -6), (0, 0), (-6, 6), (6, 6)], "bgrgb"),
    (
        &[(-3, -6), (3, -6), (-3, 0), (3, 0), (-3, 6), (3, 6)],
        "ggrrrr",
    ),
    (
        &[(-5, -9), (0, -7), (5, -5), (-3, 2), (3, 2), (-3, 8), (3, 8)],
        "gggrrrr",
    ),
    (
        &[
            (-3, -9),
            (3, -9),
            (-3, -3),
            (3, -3),
            (-3, 3),
            (3, 3),
            (-3, 9),
            (3, 9),
        ],
        "bbbbbbbb",
    ),
    (
        &[
            (-6, -6),
            (0, -6),
            (6, -6),
            (-6, 0),
            (0, 0),
            (6, 0),
            (-6, 6),
            (0, 6),
            (6, 6),
        ],
        "bbbrrrggg",
    ),
];

const BAMBU: [&str; 7] = [".#.", "###", ".#.", ".#.", ".#.", "###", ".#."];
const LINTU: [&str; 15] = [
    "....rr.......",
    "...rrrr......",
    "....gg.......",
    "...gggg......",
    "..ggwkgg.....",
    "..gggggyy....",
    "...ggggg.....",
    "...gggggg....",
    "....ggggggg..",
    "....gggbggbg.",
    ".....ggbgbbgg",
    "......g.b..bg",
    ".....yy.y...b",
    "....yy..yy...",
    ".............",
];
/// Bambujen keskipisteet; punainen bambu merkitty toisena arvona.
const BAMBUJEN_PAIKAT: [(Paikat, &[usize]); 8] = [
    (&[(0, -4), (0, 4)], &[]),
    (&[(0, -4), (-3, 4), (3, 4)], &[]),
    (&[(-3, -4), (3, -4), (-3, 4), (3, 4)], &[]),
    (&[(-5, -4), (5, -4), (0, 0), (-5, 4), (5, 4)], &[2]),
    (&[(-5, -4), (0, -4), (5, -4), (-5, 4), (0, 4), (5, 4)], &[]),
    (
        &[(0, -8), (-5, 0), (0, 0), (5, 0), (-5, 8), (0, 8), (5, 8)],
        &[0],
    ),
    (
        &[
            (-6, -4),
            (-2, -4),
            (2, -4),
            (6, -4),
            (-6, 4),
            (-2, 4),
            (2, 4),
            (6, 4),
        ],
        &[],
    ),
    (
        &[
            (-5, -8),
            (0, -8),
            (5, -8),
            (-5, 0),
            (0, 0),
            (5, 0),
            (-5, 8),
            (0, 8),
            (5, 8),
        ],
        &[1, 4, 7],
    ),
];

// Kiinalaiset numerot yhdestä yhdeksään ja niiden alla merkki "kymmenentuhatta".
#[rustfmt::skip]
const NUMERAALIT: [[&str; 8]; 9] = [
    ["...........", "...........", "...........", "...........", "###########", "...........", "...........", "..........."],
    ["...........", "..#######..", "...........", "...........", "...........", "...........", "###########", "..........."],
    ["..#######..", "...........", "...........", "...#####...", "...........", "...........", "###########", "..........."],
    ["###########", "#...#.#...#", "#...#.#...#", "#..#..#...#", "#.#....##.#", "#.........#", "###########", "..........."],
    [".#########.", ".....#.....", ".....#.....", "..#######..", "..#.....#..", "..#.....#..", "###########", "..........."],
    [".....#.....", "......#....", "###########", "...........", "...#...#...", "..#.....#..", ".#.......#.", "#.........#"],
    ["...#.......", "...#.......", "...#....##.", "#########..", "...#.......", "...#.......", "...#......#", "....######."],
    ["....#.#....", "....#.#....", "....#..#...", "...#...#...", "...#....#..", "..#.....#..", ".#.......#.", "#.........#"],
    ["...#.......", "...#.......", "#######....", "...#..#....", "...#..#....", "..#...#....", ".#....#...#", "#.....####."],
];
const WAN: [&str; 11] = [
    "..#.....#..",
    "###########",
    "..#.....#..",
    ".#########.",
    ".#...#...#.",
    ".#########.",
    ".#...#...#.",
    ".#########.",
    "...#.#.#...",
    "..#..#..#.#",
    ".#..##...##",
];
const ZHONG: [&str; 11] = [
    "....#....",
    "....#....",
    "#########",
    "#...#...#",
    "#...#...#",
    "#########",
    "....#....",
    "....#....",
    "....#....",
    "....#....",
    "....#....",
];
const FA: [&str; 11] = [
    "#.#...#.#",
    ".#.#.#.#.",
    "#..#.#..#",
    ".........",
    "###.##.##",
    "..#..#.#.",
    "###..##..",
    "#...#..#.",
    "###.#..#.",
    "..#.#.#.#",
    "##.#...#.",
];
const KUKKA: [&str; 15] = [
    "....ppp....",
    "...ppppp...",
    "...ppppp...",
    ".ppp.p.ppp.",
    "pppppyppppp",
    "ppppyyypppp",
    "pppppyppppp",
    ".ppp.p.ppp.",
    "...ppppp...",
    "...ppgpp...",
    "....pgp....",
    "..gg.g.....",
    "...ggg.gg..",
    ".....ggg...",
    ".....g.....",
];
const KEVAT: [&str; 12] = [
    "........pp.",
    ".......pyp.",
    "...pp...pp.",
    "..pyp..n...",
    "...ppnn....",
    ".....n..pp.",
    "....n..pyp.",
    "...n...pp..",
    "..n.nn.....",
    ".n....n.pp.",
    "n......pyp.",
    "........pp.",
];
const KESA: [&str; 11] = [
    ".....y.....",
    ".y...y...y.",
    "..y.....y..",
    "....ooo....",
    "...ooooo...",
    "yy.ooooo.yy",
    "...ooooo...",
    "....ooo....",
    "..y.....y..",
    ".y...y...y.",
    ".....y.....",
];
const SYKSY: [&str; 12] = [
    ".....o.....",
    "....ooo....",
    ".o..ooo..o.",
    ".oo.ooo.oo.",
    "..ooooooo..",
    "ooooooooooo",
    ".ooooroooo.",
    "..oooroo...",
    "...oorooo..",
    ".....n.....",
    ".....n.....",
    "....n......",
];
const TALVI: [&str; 11] = [
    ".....c.....",
    "...c.c.c...",
    "....ccc....",
    "c.c..c..c.c",
    ".cc..c..cc.",
    "ccccccccccc",
    ".cc..c..cc.",
    "c.c..c..c.c",
    "....ccc....",
    "...c.c.c...",
    ".....c.....",
];

fn kuvan_vari(merkki: u8) -> Option<u32> {
    Some(match merkki {
        b'g' => VIHREA,
        b'b' => SININEN,
        b'r' => PUNAINEN,
        b'k' => LAIVASTO,
        b'w' => VALKOINEN,
        b'y' => KULTA,
        b'o' => ORANSSI,
        b'n' => RUSKEA,
        b'c' => JAA,
        _ => return None,
    })
}

fn kukan_vari(arvo: u8) -> u32 {
    [PUNAINEN, VIOLETTI, SININEN, KULTA][(arvo as usize - 1) % 4]
}

fn vuodenajan_vari(arvo: u8) -> u32 {
    [ROSA, ORANSSI, PUNAINEN, JAA][(arvo as usize - 1) % 4]
}

/// Laattojen piirtojärjestys: alemmat kerrokset ensin, ja kerroksen sisällä kaukaisimmat
/// (oikea yläkulma) ensin, jotta lähempien kyljet peittävät ne.
pub fn piirtojarjestys(lauta: &Lauta) -> Vec<usize> {
    let mut jarjestys: Vec<usize> = (0..lauta.paikat.len()).collect();
    jarjestys.sort_by_key(|&i| {
        let p = lauta.paikat[i];
        (p.z, p.y as i32 - p.x as i32, p.y)
    });
    jarjestys
}

/// Laudan siirto ruudulle: koko pohja keskitetään, joten lauta ei liiku laattojen lähtiessä.
fn origo(lauta: &Lauta) -> (i32, i32) {
    let (mut vasen, mut oikea, mut yla, mut ala) = (i32::MAX, i32::MIN, i32::MAX, i32::MIN);
    for p in &lauta.paikat {
        let (x, y) = suhteellinen(p.x, p.y, p.z);
        vasen = vasen.min(x - PAKSUUS);
        oikea = oikea.max(x + LAATTA_L);
        yla = yla.min(y);
        ala = ala.max(y + LAATTA_K + PAKSUUS);
    }
    if vasen > oikea {
        return (ALUE_X, ALUE_Y);
    }
    let x = ALUE_X + (ALUE_L - (oikea - vasen)) / 2 - vasen;
    let y = ALUE_Y + (ALUE_K - (ala - yla)) / 2 - yla;
    (x / U * U, y / U * U)
}

fn suhteellinen(x: i8, y: i8, z: u8) -> (i32, i32) {
    (
        x as i32 * PUOLI_L + z as i32 * PAKSUUS,
        y as i32 * PUOLI_K - z as i32 * PAKSUUS,
    )
}

/// Laatan kasvon vasen yläkulma ruudulla.
fn kasvo(lauta: &Lauta, origo: (i32, i32), i: usize) -> (i32, i32) {
    let p = lauta.paikat[i];
    let (x, y) = suhteellinen(p.x, p.y, p.z);
    (origo.0 + x, origo.1 + y)
}

fn kasvon_keski(lauta: &Lauta, origo: (i32, i32), i: usize) -> (i32, i32) {
    let (x, y) = kasvo(lauta, origo, i);
    (x + LAATTA_L / 2, y + LAATTA_K / 2)
}

/// Päällimmäinen jäljellä oleva laatta pisteessä: kasvo tai sen alla näkyvä kylki.
pub fn laatta_pisteesta(lauta: &Lauta, x: i32, y: i32) -> Option<usize> {
    let o = origo(lauta);
    piirtojarjestys(lauta).into_iter().rev().find(|&i| {
        if lauta.laatat[i].is_none() {
            return false;
        }
        let (kx, ky) = kasvo(lauta, o, i);
        let kasvolla = (kx..kx + LAATTA_L).contains(&x) && (ky..ky + LAATTA_K).contains(&y);
        let (sx, sy) = (kx - PAKSUUS, ky + PAKSUUS);
        let kyljella = (sx..sx + LAATTA_L).contains(&x) && (sy..sy + LAATTA_K).contains(&y);
        kasvolla || kyljella
    })
}

#[cfg(test)]
pub fn laatan_keskipiste(lauta: &Lauta, i: usize) -> (i32, i32) {
    kasvon_keski(lauta, origo(lauta), i)
}

/// Lähin vapaa laatta nuolen suunnassa. Sivuttainen poikkeama painaa enemmän kuin matka.
pub fn naapuri(lauta: &Lauta, nykyinen: usize, drivi: i8, dsarake: i8) -> Option<usize> {
    let o = origo(lauta);
    let vapaat = lauta.vapaat();
    let (x0, y0) = kasvon_keski(lauta, o, nykyinen.min(lauta.paikat.len().saturating_sub(1)));
    let (sx, sy) = (dsarake as i32, drivi as i32);
    (0..lauta.paikat.len())
        .filter(|&i| i != nykyinen && vapaat[i])
        .filter_map(|i| {
            let (x, y) = kasvon_keski(lauta, o, i);
            let (dx, dy) = (x - x0, y - y0);
            let matka = dx * sx + dy * sy;
            let sivu = (dx * sy - dy * sx).abs();
            (matka > 0).then_some((matka + 2 * sivu, i))
        })
        .min()
        .map(|(_, i)| i)
}

pub fn nappi_pisteesta(x: i32, y: i32) -> Option<Nappi> {
    let i = (y - NAPIT_Y).div_euclid(NAPPI_ASKEL);
    let sisalla = (y - NAPIT_Y).rem_euclid(NAPPI_ASKEL) < NAPPI_K;
    if !sisalla || !(0..NAPIT.len() as i32).contains(&i) {
        return None;
    }
    let (nappi, teksti) = NAPIT[i as usize];
    (SIVU_TEKSTI_X..SIVU_TEKSTI_X + napin_leveys(teksti))
        .contains(&x)
        .then_some(nappi)
}

#[cfg(test)]
pub fn napin_keskipiste(nappi: Nappi) -> (i32, i32) {
    let i = NAPIT.iter().position(|n| n.0 == nappi).unwrap() as i32;
    (SIVU_TEKSTI_X + 12, NAPIT_Y + i * NAPPI_ASKEL + NAPPI_K / 2)
}

pub fn vaikeus_pisteesta(x: i32, y: i32) -> Option<Vaikeus> {
    TASOT
        .iter()
        .find(|&&(_, teksti, nx)| {
            (nx..nx + napin_leveys(teksti)).contains(&x) && (TASO_Y..TASO_Y + NAPPI_K).contains(&y)
        })
        .map(|&(taso, ..)| taso)
}

#[cfg(test)]
pub fn vaikeuden_keskipiste(vaikeus: Vaikeus) -> (i32, i32) {
    let (_, teksti, x) = TASOT.iter().find(|t| t.0 == vaikeus).copied().unwrap();
    (x + napin_leveys(teksti) / 2, TASO_Y + NAPPI_K / 2)
}

/// Pelilaudan näkymässä äänikytkin on täyden ruudun oikeassa laidassa.
pub fn aanikytkimella(vaihe: Vaihe, x: i32, y: i32) -> bool {
    let siirto = if vaihe == Vaihe::Alku {
        0
    } else {
        (RUUDUN_LEVEYS - LEVEYS) as i32
    };
    crate::yhteinen::piirto::aanikytkimella(x - siirto, y)
}

fn pohjan_nimi(vaikeus: Vaikeus) -> &'static str {
    kaanna(match vaikeus {
        Vaikeus::Helppo => "mahjong.pohja_1",
        Vaikeus::Normaali => "mahjong.pohja_2",
        Vaikeus::Vaikea => "mahjong.pohja_3",
    })
}

fn tason_nimi(vaikeus: Vaikeus) -> &'static str {
    kaanna(match vaikeus {
        Vaikeus::Helppo => "yhteinen.helppo",
        Vaikeus::Normaali => "yhteinen.normaali",
        Vaikeus::Vaikea => "yhteinen.vaikea",
    })
}

/// Alkuruudun malli: pieni laattapyramidi, jonka päällimmäisissä laatoissa on värimerkit.
pub fn laattamalli() -> Vec<Tahko> {
    const L: f32 = 1.5;
    const S: f32 = 2.0;
    const K: f32 = 0.75;
    let mut tahkot = Vec::new();
    // (kerros, sarakkeita, rivejä)
    for (kerros, sarakkeita, riveja) in [(0, 3, 2), (1, 2, 1), (2, 1, 1)] {
        let y = kerros as f32 * K;
        for sarake in 0..sarakkeita {
            for rivi in 0..riveja {
                let x = (sarake as f32 - (sarakkeita - 1) as f32 / 2.0) * L - L / 2.0;
                let z = (rivi as f32 - (riveja - 1) as f32 / 2.0) * S - S / 2.0;
                let (xs, zs) = ([x + 0.04, x + L - 0.04], [z + 0.04, z + S - 0.04]);
                kuutio(xs, [y, y + 0.3], zs, 0xC8802C, &mut tahkot);
                kuutio(xs, [y + 0.3, y + K], zs, 0xF4ECD8, &mut tahkot);
                let vari = [VIHREA, SININEN, PUNAINEN][kerros];
                let (mx, mz) = (x + L / 2.0, z + S / 2.0);
                kuutio(
                    [mx - 0.35, mx + 0.35],
                    [y + K, y + K + 0.06],
                    [mz - 0.5, mz + 0.5],
                    vari,
                    &mut tahkot,
                );
            }
        }
    }
    tahkot
}

/// Alkuruutu piirretään klassiseen kuvaan, jonka käynnistin skaalaa ruudulle.
pub fn piirra_alku(peli: &Mahjong, kuva: &mut [u32]) {
    let mut p = Piirtaja {
        k: Kangas::new(kuva),
        kello: peli.kello_ms,
    };
    p.mahjong_alkuruutu(peli);
    p.aanikytkin(peli.aanet.paalla);
}

pub fn piirra(peli: &Mahjong, ruutu: &mut [u32]) {
    let mut p = Piirtaja {
        k: Kangas::kokoinen(ruutu, RUUDUN_LEVEYS, RUUDUN_KORKEUS),
        kello: peli.kello_ms,
    };
    p.mahjong_nakyma(peli);
    p.aanikytkin(peli.aanet.paalla);
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Korostus {
    Ei,
    Peitetty,
    Hiiri,
    Valittu,
    Vihje,
}

fn sekunnit(ms: u32) -> String {
    let s = ms / 1000;
    format!("{}:{:02}", s / 60, s % 60)
}

impl Piirtaja<'_> {
    fn mahjong_alkuruutu(&mut self, peli: &Mahjong) {
        self.maisema(kaanna("mahjong.banneri"), &laattamalli());
        let otsikko = listan_otsikko(peli.taulu.netti.is_some());
        self.parhaat_alkuruudulla(otsikko, peli.taulu.nakyva());
        let y = self.alkuohje(&ALKU_OSAT, &ALKU_RIVIT) + RIVIN_KORKEUS / 2;
        let taso = peli.vaikeus;
        let kuvaus = muotoile(
            "mahjong.kuvaus",
            &[&pohjan_nimi(taso), &taso.pohja().len(), &taso.kerroin()],
        );
        self.teksti(48, y, &kuvaus, KELTAINEN);

        self.teksti(48, TASO_Y + 3, kaanna("yhteinen.taso"), KELTAINEN);
        for (taso, teksti, x) in TASOT {
            if taso == peli.vaikeus {
                self.nappi(x, TASO_Y, teksti, WB_ORANSSI, MUSTA);
            } else {
                self.nappi(x, TASO_Y, teksti, VALKOINEN, WB_SININEN);
            }
        }
        self.aloituskehote(ALOITUS);
    }

    fn mahjong_nakyma(&mut self, peli: &Mahjong) {
        self.pelitausta();
        let nimi = format!(
            "{} - {}",
            kaanna("peli.mahjong"),
            pohjan_nimi(peli.lauta.vaikeus)
        );
        self.otsikkopalkki(&nimi.to_uppercase(), tason_nimi(peli.lauta.vaikeus));
        self.mahjong_lauta(peli);
        self.mahjong_ponnahdus(peli);
        self.mahjong_sivu(peli);
        match peli.vaihe {
            Vaihe::Voitto => {
                self.mahjong_juhla(peli);
                self.mahjong_loppu(peli);
            }
            Vaihe::Tappio => self.mahjong_loppu(peli),
            Vaihe::Nimi => {
                self.mahjong_juhla(peli);
                self.nimi_ikkuna(peli.lauta.pisteet, &peli.nimi);
            }
            Vaihe::Alku | Vaihe::Pelissa => {}
        }
    }

    fn mahjong_lauta(&mut self, peli: &Mahjong) {
        let lauta = &peli.lauta;
        let o = origo(lauta);
        let vapaat = lauta.vapaat();
        let pelissa = peli.vaihe == Vaihe::Pelissa;
        let vihje_nakyy = pelissa && self.vilkku();
        let hiiri = if pelissa && !peli.kursori_nakyy {
            peli.hiiren_laatta.filter(|&i| vapaat[i])
        } else {
            None
        };
        for i in piirtojarjestys(lauta) {
            let Some(laatta) = lauta.laatat[i] else {
                continue;
            };
            let vihjeessa = peli.vihje.is_some_and(|(a, b)| a == i || b == i);
            let korostus = if peli.valittu == Some(i) {
                Korostus::Valittu
            } else if vihjeessa && vihje_nakyy {
                Korostus::Vihje
            } else if hiiri == Some(i) {
                Korostus::Hiiri
            } else if lauta.vaikeus.nayta_vapaat() && !vapaat[i] {
                Korostus::Peitetty
            } else {
                Korostus::Ei
            };
            let (x, y) = kasvo(lauta, o, i);
            self.mahjong_laatta(x, y, laatta, korostus);
        }
        if pelissa && peli.kursori_nakyy && self.vilkku() && lauta.laatat[peli.kursori].is_some() {
            let (x, y) = kasvo(lauta, o, peli.kursori);
            for i in 0..2 {
                let d = i * U;
                self.k
                    .kehys(x - d, y - d, LAATTA_L + 2 * d, LAATTA_K + 2 * d, WB_ORANSSI);
            }
        }
    }

    /// Laatta kerroksittain: ääriviiva, kyljen kerrokset ja kasvo. Kustakin kerroksesta
    /// piirretään vain näkyvä vasen sarake ja alarivi, joten iso lauta piirtyy kevyesti.
    fn mahjong_laatta(&mut self, x: i32, y: i32, laatta: Laatta, korostus: Korostus) {
        let kerrokset = |k: i32| (x - PAKSUUS + k * U, y + PAKSUUS - k * U);
        for k in 0..=3 {
            let (lx, ly) = kerrokset(k);
            let (ux, uy, ul, uk) = (lx - U, ly - U, LAATTA_L + 2 * U, LAATTA_K + 2 * U);
            self.k.laatikko(ux, uy, ul, U, AARIVIIVA);
            self.k.laatikko(ux, uy + uk - U, ul, U, AARIVIIVA);
            self.k.laatikko(ux, uy, U, uk, AARIVIIVA);
            self.k.laatikko(ux + ul - U, uy, U, uk, AARIVIIVA);
        }
        for (k, &vari) in KYLKI.iter().enumerate() {
            let (lx, ly) = kerrokset(k as i32);
            self.k.laatikko(lx, ly, U, LAATTA_K, vari);
            self.k
                .laatikko(lx, ly + LAATTA_K - U, LAATTA_L, U, tummenna(vari, 0.25));
        }
        let (yla, ala) = match korostus {
            Korostus::Ei => (KASVO_YLA, KASVO_ALA),
            Korostus::Peitetty => (PEITETTY_YLA, PEITETTY_ALA),
            Korostus::Hiiri => (HIIRI_YLA, HIIRI_ALA),
            Korostus::Valittu => (VALITTU_YLA, VALITTU_ALA),
            Korostus::Vihje => (VIHJE_YLA, VIHJE_ALA),
        };
        self.k.liukulaatikko(x, y, LAATTA_L, LAATTA_K, yla, ala);
        self.k.laatikko(x, y, LAATTA_L, U, vaalenna(yla, 0.6));
        self.k.laatikko(x, y, U, LAATTA_K, vaalenna(yla, 0.4));
        self.k.laatikko(
            x + LAATTA_L - U,
            y + U,
            U,
            LAATTA_K - U,
            tummenna(ala, 0.08),
        );
        self.mahjong_kuva(x, y, laatta);
    }

    /// Kuva konepikselin ruudukossa: (mx, my) on kuvan kulma kasvon kulmasta.
    fn mahjong_kuvio(
        &mut self,
        (x, y): (i32, i32),
        (mx, my): (i32, i32),
        kuva: &[&str],
        vari: u32,
    ) {
        self.k.sprite((x + mx * U, y + my * U), kuva, U, |merkki| {
            (merkki != b'.').then_some(vari)
        });
    }

    fn mahjong_varikuvio(
        &mut self,
        (x, y): (i32, i32),
        (mx, my): (i32, i32),
        kuva: &[&str],
        kerroin: i32,
    ) {
        self.k
            .sprite((x + mx * U, y + my * U), kuva, kerroin, kuvan_vari);
    }

    fn mahjong_kuva(&mut self, x: i32, y: i32, laatta: Laatta) {
        let kulma = (x, y);
        let arvo = laatta.arvo();
        let numero = |p: &mut Self, vari: u32| {
            p.mahjong_kuvio(kulma, (2, 2), &NUMEROT[arvo as usize - 1], vari);
        };
        // Kuvan keskikohta konepikseleinä kasvon kulmasta.
        let (kx, ky) = (12, 16);
        match laatta.sarja() {
            Sarja::Pallot => {
                numero(self, SININEN);
                if arvo == 1 {
                    self.mahjong_varikuvio(kulma, (kx - 5, ky - 5), &ISO_PALLO, U);
                    return;
                }
                let (paikat, varit) = PALLOJEN_PAIKAT[arvo as usize - 2];
                for (&(dx, dy), vari) in paikat.iter().zip(varit.bytes()) {
                    let vari = kuvan_vari(vari).unwrap_or(SININEN);
                    self.mahjong_kuvio(kulma, (kx + dx - 2, ky + dy - 2), &PALLO, vari);
                }
            }
            Sarja::Bambut => {
                numero(self, VIHREA);
                if arvo == 1 {
                    self.mahjong_varikuvio(kulma, (kx - 6, ky - 7), &LINTU, U);
                    return;
                }
                let (paikat, punaiset) = BAMBUJEN_PAIKAT[arvo as usize - 2];
                for (i, &(dx, dy)) in paikat.iter().enumerate() {
                    let vari = if punaiset.contains(&i) {
                        PUNAINEN
                    } else {
                        VIHREA
                    };
                    self.mahjong_kuvio(kulma, (kx + dx - 1, ky + dy - 3), &BAMBU, vari);
                }
            }
            Sarja::Merkit => {
                numero(self, PUNAINEN);
                self.mahjong_kuvio(kulma, (7, 4), &NUMERAALIT[arvo as usize - 1], LAIVASTO);
                self.mahjong_kuvio(kulma, (7, 14), &WAN, PUNAINEN);
            }
            Sarja::Tuulet => {
                let avain = TUULET[(arvo as usize - 1) % TUULET.len()];
                let kirjain = kaanna(avain).chars().next().unwrap_or('?').to_string();
                self.teksti_koossa(x + 4 * U, y + 7 * U, &kirjain, 2 * U, LAIVASTO);
            }
            Sarja::Lohikaarmeet => match arvo {
                1 => self.mahjong_merkki(kulma, (3, 4), &ZHONG, PUNAINEN),
                2 => self.mahjong_merkki(kulma, (3, 4), &FA, VIHREA),
                _ => {
                    for (d, paksuus) in [(4, 2), (7, 1)] {
                        for i in 0..paksuus {
                            let reuna = (d + i) * U;
                            self.k.kehys(
                                x + reuna,
                                y + reuna,
                                LAATTA_L - 2 * reuna,
                                LAATTA_K - 2 * reuna,
                                SININEN,
                            );
                        }
                    }
                }
            },
            Sarja::Kukat => {
                let vari = kukan_vari(arvo);
                numero(self, vari);
                self.k
                    .sprite((x + 7 * U, y + 10 * U), &KUKKA, U, |merkki| match merkki {
                        b'p' => Some(vari),
                        b'y' => Some(KULTA),
                        b'g' => Some(VIHREA),
                        _ => None,
                    });
            }
            Sarja::Vuodenajat => {
                numero(self, vuodenajan_vari(arvo));
                let kuva: &[&str] = match arvo {
                    1 => &KEVAT,
                    2 => &KESA,
                    3 => &SYKSY,
                    _ => &TALVI,
                };
                self.k
                    .sprite((x + 7 * U, y + 11 * U), kuva, U, |merkki| match merkki {
                        b'p' => Some(ROSA),
                        _ => kuvan_vari(merkki),
                    });
            }
        }
    }

    /// Lohikäärmeiden merkit kaksinkertaisina, jotta ne erottuvat kauempaakin.
    fn mahjong_merkki(
        &mut self,
        (x, y): (i32, i32),
        (mx, my): (i32, i32),
        kuva: &[&str],
        vari: u32,
    ) {
        self.k
            .sprite((x + mx * U, y + my * U), kuva, 2 * U, |merkki| {
                (merkki != b'.').then_some(vari)
            });
    }

    /// Otetun parin pisteet nousevat ja kipinät sinkoavat laattojen paikalta.
    fn mahjong_ponnahdus(&mut self, peli: &Mahjong) {
        let Some(ponnahdus) = peli.ponnahdus else {
            return;
        };
        let lauta = &peli.lauta;
        let o = origo(lauta);
        let ika = self.kello.saturating_sub(ponnahdus.alku) as f32 / PONNAHDUS_MS as f32;
        let (a, b) = ponnahdus.paikat;
        let (ax, ay) = kasvon_keski(lauta, o, a);
        let (bx, by) = kasvon_keski(lauta, o, b);
        if ika < 0.6 {
            for keski in [(ax, ay), (bx, by)] {
                self.rajahdys(keski, ika / 0.6, 10, 90.0);
            }
        }
        let teksti = if ponnahdus.ketju > 1 {
            format!("+{} x{}", ponnahdus.pisteet, ponnahdus.ketju)
        } else {
            format!("+{}", ponnahdus.pisteet)
        };
        let leveys = teksti.chars().count() as i32 * OTSIKKO.2;
        let nousu = (ika * 80.0) as i32 / U * U;
        let (kx, ky) = ((ax + bx) / 2, (ay + by) / 2);
        let x = (kx - leveys / 2).clamp(ALUE_X, ALUE_X + ALUE_L - leveys);
        let vari = if ponnahdus.ketju > 1 {
            WB_ORANSSI
        } else {
            KELTAINEN
        };
        self.suurteksti(x, ky - 12 - nousu, &teksti, OTSIKKO, vari, true);
    }

    fn mahjong_sivu(&mut self, peli: &Mahjong) {
        let lauta = &peli.lauta;
        self.ikkuna(SIVU_X - 12, 42, SIVU_L + 24, RUUDUN_KORKEUS as i32 - 66);
        let ketju = if lauta.ketju > 1 {
            format!("x{}", lauta.ketju)
        } else {
            "-".to_string()
        };
        let tilastot: [(&str, String); TILASTOJA] = [
            (kaanna("yhteinen.pisteet"), lauta.pisteet.to_string()),
            (kaanna("mahjong.ketju"), ketju),
            (kaanna("mahjong.aika"), sekunnit(peli.peliaika_ms)),
            (
                kaanna("mahjong.pareja"),
                lauta.vapaat_parit().len().to_string(),
            ),
            (kaanna("mahjong.laattoja"), lauta.jaljella().to_string()),
            (kaanna("mahjong.sekoituksia"), lauta.sekoituksia.to_string()),
        ];
        let oikea = SIVU_X + SIVU_L - 18;
        for (i, (otsikko, arvo)) in tilastot.iter().enumerate() {
            let y = TILASTO_Y + i as i32 * TILASTO_ASKEL;
            self.suurteksti(SIVU_TEKSTI_X, y, otsikko, OTSIKKO, KELTAINEN, false);
            let x = oikea - arvo.chars().count() as i32 * OTSIKKO.2;
            self.suurteksti(x, y, arvo, OTSIKKO, VALKOINEN, true);
        }

        self.ikkuna(SIVU_X, VIESTI_Y, SIVU_L, VIESTI_K);
        let mut y = VIESTI_Y + 9;
        for rivi in rivita(&peli.viesti, SIVU_MERKKEJA).into_iter().take(4) {
            self.teksti(SIVU_TEKSTI_X, y, rivi, VALKOINEN);
            y += RIVIN_KORKEUS;
        }

        let mut y = SIVU_OHJE_Y;
        for rivi in SIVU_OHJEET {
            self.teksti(SIVU_TEKSTI_X, y, kaanna(rivi), HUTI);
            y += RIVIN_KORKEUS;
        }

        let pelissa = peli.vaihe == Vaihe::Pelissa;
        for (i, &(nappi, teksti)) in NAPIT.iter().enumerate() {
            let y = NAPIT_Y + i as i32 * NAPPI_ASKEL;
            let kaytossa = match nappi {
                Nappi::Vihje => pelissa && lauta.tila == Tila::Kesken,
                Nappi::Sekoita => pelissa && lauta.sekoituksia > 0,
                Nappi::Peruuta => pelissa && lauta.voi_perua(),
                Nappi::Uusi | Nappi::Valikko => peli.vaihe != Vaihe::Nimi,
            };
            let vari = if kaytossa { WB_SININEN } else { KANSI };
            self.nappi(SIVU_TEKSTI_X, y, teksti, VALKOINEN, vari);
        }

        let otsikko = listan_otsikko(peli.taulu.netti.is_some());
        self.parhaat_lista(
            SIVU_TEKSTI_X,
            SIVU_PARHAAT_Y,
            SIVU_L - 36,
            TULOS,
            otsikko,
            peli.taulu.nakyva(),
        );
    }

    /// Voiton ilotulitus: raketit räjähtävät vuorotellen laudan yllä.
    fn mahjong_juhla(&mut self, peli: &Mahjong) {
        if peli.lauta.tila != Tila::Voitto {
            return;
        }
        const VALI_MS: u32 = 250;
        const KESTO_MS: u32 = 900;
        let kulunut = self.kello.saturating_sub(peli.voitto_alku);
        for i in 0..JUHLA_MS / VALI_MS {
            let alku = i * VALI_MS;
            if kulunut < alku || kulunut >= alku + KESTO_MS {
                continue;
            }
            let siemen = i.wrapping_mul(2_654_435_761) >> 7;
            let x = ALUE_X + 60 + (siemen % (ALUE_L as u32 - 120)) as i32;
            let y = ALUE_Y + 60 + ((siemen / 7) % (ALUE_K as u32 - 360)) as i32;
            let ika = (kulunut - alku) as f32 / KESTO_MS as f32;
            self.rajahdys((x, y), ika, 18, 220.0);
        }
    }

    fn mahjong_loppu(&mut self, peli: &Mahjong) {
        let lauta = &peli.lauta;
        let x = ALUE_X + (ALUE_L - LOPPU_L) / 2;
        let y = ALUE_Y + (ALUE_K - LOPPU_K) / 2;
        self.ikkuna(x, y, LOPPU_L, LOPPU_K);
        let tx = x + 24;
        let voitto = lauta.tila == Tila::Voitto;
        let otsikko = kaanna(if voitto {
            "yhteinen.voitit"
        } else {
            "mahjong.tappio"
        });
        self.suurteksti(tx, y + 18, otsikko, OTSIKKO, KELTAINEN, true);
        let mut rivit = Vec::new();
        if voitto {
            let bonus = lauta.bonus;
            let pareista = lauta.pisteet - bonus.tyhjennys - bonus.aika;
            rivit.push(muotoile("mahjong.erittely_parit", &[&pareista]));
            rivit.push(muotoile("mahjong.erittely_tyhjennys", &[&bonus.tyhjennys]));
            rivit.push(muotoile("mahjong.erittely_aika", &[&bonus.aika]));
        } else {
            rivit.push(muotoile("mahjong.erittely_jaljella", &[&lauta.jaljella()]));
        }
        let mut ry = y + 66;
        for rivi in &rivit {
            self.teksti(tx, ry, rivi, VALKOINEN);
            ry += RIVIN_KORKEUS;
        }
        let yhteensa = muotoile("mahjong.yhteensa", &[&lauta.pisteet]);
        self.suurteksti(tx, y + LOPPU_K - 84, &yhteensa, OTSIKKO, KELTAINEN, true);
        self.teksti(tx, y + LOPPU_K - 42, kaanna("mahjong.loppu_ohje"), HUTI);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yhteinen::kieli::kaikilla_kielilla;
    use crate::yhteinen::syote::Syote;
    use klassikot8000::Arpa;

    #[test]
    fn kaikki_pohjat_mahtuvat_alueelle() {
        let mut arpa = Arpa::new(1);
        for taso in Vaikeus::KAIKKI {
            let lauta = Lauta::new(taso, &mut arpa);
            let o = origo(&lauta);
            for i in 0..lauta.paikat.len() {
                let (x, y) = kasvo(&lauta, o, i);
                assert!(x - PAKSUUS - U >= ALUE_X, "{taso:?}");
                assert!(x + LAATTA_L + U <= ALUE_X + ALUE_L, "{taso:?}");
                assert!(y - U >= ALUE_Y, "{taso:?}");
                assert!(y + LAATTA_K + PAKSUUS + U <= ALUE_Y + ALUE_K, "{taso:?}");
                assert_eq!(x % U, 0);
                assert_eq!(y % U, 0);
            }
        }
        assert!(SIVU_PARHAAT_Y + PARHAAT_OTSIKKO + 3 * PARHAAT_RIVI < RUUDUN_KORKEUS as i32 - 30);
        assert!(SIVU_MERKKEJA as i32 * MERKKI <= SIVU_L - 36);
    }

    #[test]
    fn jokainen_jaljella_oleva_laatta_loytyy_jostain_pisteesta() {
        let mut arpa = Arpa::new(3);
        for taso in Vaikeus::KAIKKI {
            let lauta = Lauta::new(taso, &mut arpa);
            let o = origo(&lauta);
            let mut loytyi = vec![false; lauta.paikat.len()];
            for y in (ALUE_Y..ALUE_Y + ALUE_K).step_by(U as usize) {
                for x in (ALUE_X..ALUE_X + ALUE_L).step_by(U as usize) {
                    if let Some(i) = laatta_pisteesta(&lauta, x, y) {
                        loytyi[i] = true;
                    }
                }
            }
            // Päällimmäinen laatta peittää alemman, joten vain näkyvät löytyvät; vapaat ainakin.
            for (i, vapaa) in lauta.vapaat().into_iter().enumerate() {
                if vapaa {
                    assert!(loytyi[i], "{taso:?}: {i}");
                    let (x, y) = kasvon_keski(&lauta, o, i);
                    assert_eq!(laatta_pisteesta(&lauta, x, y), Some(i), "{taso:?}: {i}");
                }
            }
            assert_eq!(laatta_pisteesta(&lauta, 5, 5), None);
        }
    }

    #[test]
    fn napit_ja_tasot_loytyvat() {
        for (nappi, _) in NAPIT {
            let (x, y) = napin_keskipiste(nappi);
            assert_eq!(nappi_pisteesta(x, y), Some(nappi));
        }
        assert_eq!(nappi_pisteesta(SIVU_TEKSTI_X - 1, NAPIT_Y + 3), None);
        assert_eq!(nappi_pisteesta(SIVU_TEKSTI_X + 3, NAPIT_Y - 1), None);
        for taso in Vaikeus::KAIKKI {
            let (x, y) = vaikeuden_keskipiste(taso);
            assert_eq!(vaikeus_pisteesta(x, y), Some(taso));
        }
    }

    #[test]
    fn tekstit_mahtuvat_paikoilleen() {
        kaikilla_kielilla(|| {
            for (taso, teksti, x) in TASOT {
                assert!(x + napin_leveys(teksti) < LEVEYS as i32, "{taso:?}");
            }
            for rivi in SIVU_OHJEET {
                assert!(kaanna(rivi).chars().count() <= SIVU_MERKKEJA, "{rivi}");
            }
            for (_, teksti) in NAPIT {
                assert!(napin_leveys(teksti) <= SIVU_L - 36, "{teksti}");
            }
            for avain in [
                "yhteinen.pisteet",
                "mahjong.ketju",
                "mahjong.aika",
                "mahjong.pareja",
                "mahjong.laattoja",
                "mahjong.sekoituksia",
            ] {
                // Otsikko ja viisinumeroinen arvo samalla rivillä.
                let merkkeja = kaanna(avain).chars().count() as i32 + 1 + 5;
                assert!(merkkeja * OTSIKKO.2 <= SIVU_L - 36, "{avain}");
            }
            for taso in Vaikeus::KAIKKI {
                let kuvaus = muotoile(
                    "mahjong.kuvaus",
                    &[&pohjan_nimi(taso), &taso.pohja().len(), &taso.kerroin()],
                );
                assert!(kuvaus.chars().count() <= RIVILLE_MERKKEJA, "{kuvaus}");
                let otsikko = format!("{} - {}", kaanna("peli.mahjong"), pohjan_nimi(taso));
                assert!(otsikko.chars().count() <= 30, "{otsikko}");
            }
            for avain in TUULET {
                assert_eq!(kaanna(avain).chars().count(), 1, "{avain}");
            }
            // Viestit mahtuvat neljälle riville.
            for viesti in [
                kaanna("mahjong.aloita").to_string(),
                kaanna("mahjong.ei_vapaa").to_string(),
                kaanna("mahjong.ei_pari").to_string(),
                kaanna("mahjong.ei_sekoituksia").to_string(),
                kaanna("mahjong.peruttu").to_string(),
                kaanna("mahjong.ei_peruttavaa").to_string(),
                kaanna("mahjong.ei_perua_vaikealla").to_string(),
                muotoile("mahjong.pari", &[&999]),
                muotoile("mahjong.ketju_viesti", &[&5, &999]),
                muotoile("mahjong.vihje", &[&20]),
                muotoile("mahjong.jumissa", &[&50]),
                muotoile("mahjong.sekoitettu", &[&50]),
                muotoile("mahjong.voitto_viesti", &[&99999]),
                muotoile("mahjong.tappio_viesti", &[&99999]),
            ] {
                assert!(rivita(&viesti, SIVU_MERKKEJA).len() <= 4, "{viesti}");
            }
            let tila = LOPPU_L - 48;
            for otsikko in ["yhteinen.voitit", "mahjong.tappio"] {
                assert!(
                    kaanna(otsikko).chars().count() as i32 * OTSIKKO.2 <= tila,
                    "{otsikko}"
                );
            }
            let yhteensa = muotoile("mahjong.yhteensa", &[&99999]);
            assert!(yhteensa.chars().count() as i32 * OTSIKKO.2 <= tila);
            for rivi in [
                muotoile("mahjong.erittely_parit", &[&99999]),
                muotoile("mahjong.erittely_tyhjennys", &[&9999]),
                muotoile("mahjong.erittely_aika", &[&9999]),
                muotoile("mahjong.erittely_jaljella", &[&144]),
                kaanna("mahjong.loppu_ohje").to_string(),
            ] {
                assert!(rivi.chars().count() as i32 * MERKKI <= tila, "{rivi}");
            }
        });
        let rivit: Vec<&str> = ALKU_RIVIT
            .iter()
            .copied()
            .chain(["mahjong.kuvaus"])
            .collect();
        alkuohje_mahtuu(&ALKU_OSAT, &rivit, TASO_Y);
    }

    #[test]
    fn jokainen_laatta_ja_vaihe_piirtyy_kaatumatta() {
        let mut ruutu = vec![0; RUUDUN_LEVEYS * RUUDUN_KORKEUS];
        let mut kuva = vec![0; LEVEYS * KORKEUS];
        for taso in Vaikeus::KAIKKI {
            let mut peli = Mahjong::new(1);
            peli.vaikeus = taso;
            piirra_alku(&peli, &mut kuva);
            peli.kasittele(Syote::Ammu);
            // Kaikki lajit näkyviin samalle laudalle.
            for (i, laatta) in peli.lauta.laatat.iter_mut().enumerate() {
                *laatta = Some(Laatta((i % 42) as u8));
            }
            let (a, b) = (0, 1);
            peli.valittu = Some(a);
            peli.vihje = Some((a, b));
            peli.hiiren_laatta = Some(b);
            piirra(&peli, &mut ruutu);
            peli.kursori_nakyy = true;
            piirra(&peli, &mut ruutu);
            let mut arpa = Arpa::new(9);
            peli.lauta = Lauta::new(taso, &mut arpa);
            let (a, b) = peli.lauta.ratkaisu[0];
            peli.valitse(a);
            peli.valitse(b);
            piirra(&peli, &mut ruutu);
            peli.edista(PONNAHDUS_MS / 2);
            piirra(&peli, &mut ruutu);
            for &(a, b) in &peli.lauta.ratkaisu.clone()[1..] {
                peli.valitse(a);
                peli.valitse(b);
            }
            assert_eq!(peli.vaihe, Vaihe::Nimi);
            piirra(&peli, &mut ruutu);
            peli.vahvista_nimi();
            for _ in 0..5 {
                peli.edista(JUHLA_MS / 4);
                piirra(&peli, &mut ruutu);
            }
            peli.lauta.tila = Tila::Tappio;
            peli.vaihe = Vaihe::Tappio;
            piirra(&peli, &mut ruutu);
        }
    }
}
