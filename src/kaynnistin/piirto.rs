//! Käynnistimen näkymä: pelilista, valitun pelin esittely, tulokset ja näppäinohjeet.

use std::sync::LazyLock;

use klassikot8000::laivanupotus::Vaikeus;

use super::netti::VERSIO;
use super::{Kaynnistin, Laji, PELIT};
use crate::pelit::laivanupotus::peli::Tilasto;
use crate::yhteinen::kieli::{kaanna, muotoile};
use crate::yhteinen::piirto::*;
use crate::yhteinen::rasteri::Rasteri;
use crate::yhteinen::tulokset::{PARHAITA, Tulos};
use crate::yhteinen::vektori::{Kangas, Tahko, sekoita_liuku};

// Käynnistin piirtää suoraan ruudulle, joten mitat ovat ruudun pikseleitä. Reunat, varjot ja
// laatikot ovat konepikselin (4) monikertoja, jotta lasipinnat osuvat taustan ruudukkoon.
const RUUTU_L: i32 = RUUDUN_LEVEYS as i32;
const P: i32 = 4;
const VASEN_REUNA: i32 = 48;
const OIKEA_REUNA: i32 = RUUTU_L - 48;
/// Kovat varjot korostettujen osien alla.
const VARJO: i32 = 8;
const KOROSTUS: u32 = WB_ORANSSI;
const HIMMEA_REUNA: u32 = 0x3E3290;

/// Pieni teksti: kaksinkertainen bittikarttafontti.
const PIENI: i32 = 2;
const PIENI_MERKKI: i32 = PIENI * 8;
/// Näppäimen kuva ohjeissa ja kytkimissä.
const NAPPAIN: i32 = 32;

const LOGO_Y: i32 = 36;
const LOGO_KOKO: (i32, i32, i32) = (35, 35, 44);
const SIRU_Y: i32 = 32;
const SIRU_K: i32 = 48;
const SIRU_VALI: i32 = 16;
const SIRUN_NAPPAIN_X: i32 = 8;
const SIRU_TEKSTI_X: i32 = SIRUN_NAPPAIN_X + NAPPAIN + 12;
const SIRU_LOPPU: i32 = 16;
const KAIUTIN_TILA: i32 = 36;
const KIELIKYTKIN: &str = "valikko.kielikytkin";
const AANI_PAALLA: &str = "yhteinen.aani_paalla";
const AANI_POIS: &str = "yhteinen.aani_pois";
const ILMOITUS_TEKSTI_X: i32 = 36;

const KORTTI_X: i32 = VASEN_REUNA;
const KORTTI_L: i32 = 600;
const KORTTI_Y: i32 = 136;
const KORTTI_K: i32 = 72;
const KORTTI_ASKEL: i32 = KORTTI_K + 8;
/// Valittu kortti liukuu kohti esittelyä.
const VALITUN_SIIRTO: i32 = 16;
const NUMERO_X: i32 = 16;
const KUVAKE_X: i32 = NUMERO_X + NAPPAIN + 16;
const KUVAKE_L: i32 = 56;
const NIMI_X: i32 = KUVAKE_X + KUVAKE_L + 16;
const NIMI_KOKO: (i32, i32, i32) = (21, 21, 26);
/// `NIMI_KOKO`n merkki on kolmen pikselin riveistä: 3 * 8.
const NIMEN_KORKEUS: i32 = 24;
/// Valitun kortin nuoli oikeassa laidassa.
const KORTIN_NUOLI: i32 = 40;
/// Valikkoon mahtuvat kortit. Pidempi lista vierii, ja nuolet kertovat piiloon jääneistä.
pub const NAKYVIA: usize = 10;
const NAKYVIA_KORTTEJA: i32 = if PELIT.len() < NAKYVIA {
    PELIT.len()
} else {
    NAKYVIA
} as i32;
const LISTAN_ALA: i32 = KORTTI_Y + (NAKYVIA_KORTTEJA - 1) * KORTTI_ASKEL + KORTTI_K;
const NUOLI_X: i32 = (KORTTI_X + KORTTI_L + VALITUN_SIIRTO + VARJO + ESITTELY_X) / 2;
const NUOLI_L: i32 = 5 * P;
const NUOLI_K: i32 = 3 * P;

const ESITTELY_X: i32 = 720;
const ESITTELY_L: i32 = 560;
const JARJESTYS_Y: i32 = 144;
const NIMI_Y: i32 = 176;
const SUURI_NIMI: (i32, i32, i32) = (35, 35, 42);
const PIENEMPI_NIMI: (i32, i32, i32) = (28, 28, 34);
const KUVAUS_Y: i32 = 248;
const KUVAUS_KERROIN: i32 = 3;
const KUVAUS_RIVI: i32 = 36;
const KUVAUS_MERKKEJA: usize = (ESITTELY_L / (KUVAUS_KERROIN * 8)) as usize;
const PELAA: &str = "valikko.pelaa";
/// Painike on kuvauksen alla tämän verran viimeisen rivin alareunasta.
const PELAA_YLA: i32 = 40;
const KUVAUS_VALI: i32 = KUVAUS_RIVI - KUVAUS_KERROIN * 8;
const PELAA_K: i32 = 64;
const PELAA_SISENNYS: i32 = 24;
const PELAA_KOLMIO: i32 = 16;
const PELAA_VALI: i32 = 16;

// Tausta lasketaan kerran, ja se pysyy paikallaan katsojan seuratessa mallia.
const HORISONTTI_RIVI: i32 = 130;
static MAISEMA: LazyLock<Rasteri> = LazyLock::new(|| {
    maisemakuva(
        RUUDUN_LEVEYS / 4,
        ALAPALKKI_Y as usize / 4,
        HORISONTTI_RIVI,
        MALLI_X / 4,
    )
});
/// Maisema tummennettuna: listan ja laatikoiden läpikuultava pohja kopioidaan tästä, joten
/// läpinäkyvyys ei maksa piirrossa mitään.
static LASI: LazyLock<Rasteri> = LazyLock::new(|| {
    Rasteri::luo(RUUDUN_LEVEYS / 4, ALAPALKKI_Y as usize / 4, |x, y| {
        sekoita_liuku(MAISEMA.rivi(y as usize)[x as usize], LASIN_VARI, 0.82)
    })
});
const LASIN_VARI: u32 = 0x0E0830;
const MALLI_X: i32 = 1512;
const MALLI_Y: i32 = 460;
const MALLIN_KOKO: f32 = 76.0;
const MALLIN_KEINUNTA: f32 = 6.0;
pub const MALLIN_KASVU_MS: u32 = 200;
const MALLIN_ALKUKOKO: f32 = 0.6;

// Tilastot ja top 3 omissa laatikoissaan meren päällä, oikeaan laitaan tasattuina.
const LAATIKKO_K: i32 = 248;
const LAATIKKO_Y: i32 = LISTAN_ALA - LAATIKKO_K;
const LAATIKKO_SISENNYS: i32 = 24;
const LAATIKKO_VALI: i32 = 24;
const LISTA_L: i32 = 440;
const LISTA_X: i32 = OIKEA_REUNA - LISTA_L;
const TILASTO_X: i32 = ESITTELY_X;
const TILASTO_L: i32 = LISTA_X - LAATIKKO_VALI - TILASTO_X;
const LISTA_KOKO: (i32, i32, i32) = (14, 14, 16);
const LAATIKON_OTSIKKO: i32 = 32;
const OMA_VALI: i32 = 8;
const OMA_PARAS: &str = "valikko.oma_paras";
const TILASTOT: &str = "valikko.tilastot";

// Laivanupotuksen tilasto: tasot riveinä, sarakkeiden oikeat reunat laatikon sisällä.
const TAULUKON_RIVI: i32 = 44;
const TAULUKON_SARAKKEET: [(&str, i32); 3] = [
    ("valikko.voitot", 400),
    ("valikko.tappiot", 528),
    ("yhteinen.paras", 640),
];
const TASOJEN_NIMET: [&str; 3] = ["yhteinen.helppo", "yhteinen.normaali", "yhteinen.vaikea"];

const ALAPALKKI_Y: i32 = 1016;
const OHJE_NAPPAIN_Y: i32 = ALAPALKKI_Y + (RUUDUN_KORKEUS as i32 - ALAPALKKI_Y - NAPPAIN) / 2;
const OHJE_TEKSTI_Y: i32 = OHJE_NAPPAIN_Y + (NAPPAIN - PIENI_MERKKI) / 2;
const NAPPAINTEN_VALI: i32 = 8;
const SELITTEEN_VALI: i32 = 12;
const RYHMIEN_VALI: i32 = 40;
/// Nuolinäppäimet piirtyvät kolmioina, välilyönti korostettuna ja käännettynä.
const YLOS: &str = "^";
const ALAS: &str = "v";
const VALI: &str = "valikko.vali";
const LOPETA: &str = "valikko.lopeta";
const LOPETA_UUDELLEEN: &str = "valikko.lopeta_uudelleen";
const OHJEET: [(&[&str], &str); 5] = [
    (&[YLOS, ALAS, "1-0"], "valikko.valitse"),
    (&[VALI], PELAA),
    (&["M"], "valikko.aanet"),
    (&["L"], "valikko.kieli"),
    (&["ESC"], LOPETA),
];

// Pelien pikselikuvakkeet 12x12. Kirjain on väri, piste läpinäkyvä.
#[rustfmt::skip]
const LAIVA: [&str; 12] = [
    "............", ".....W......", ".....W......", "....GGG.....",
    "...GGGGG....", ".DDDDDDDDDD.", "..DDDDDDDD..", "...DDDDDD...",
    "CCCCCCCCCCCC", ".C..C...C..C", "............", "............",
];
#[rustfmt::skip]
const LAATAT: [&str; 12] = [
    "............", ".YYYY..OOOO.", ".YYYY..OOOO.", ".YYYY..OOOO.",
    ".YYYY..OOOO.", "............", "............", ".PPPP..LLLL.",
    ".PPPP..LLLL.", ".PPPP..LLLL.", ".PPPP..LLLL.", "............",
];
#[rustfmt::skip]
const KORTTI: [&str; 12] = [
    "............", "..WWWWWWWW..", "..WWWWWWWW..", "..WRRWWRRW..",
    "..WRRRRRRW..", "..WRRRRRRW..", "..WWRRRRWW..", "..WWWRRWWW..",
    "..WWWWWWWW..", "..WWWWWWWW..", "..WWWWWWWW..", "............",
];
#[rustfmt::skip]
const TIILET: [&str; 12] = [
    "............", "PPP.OOO.YYY.", "PPP.OOO.YYY.", "............",
    "CCC.LLL.PPP.", "CCC.LLL.PPP.", "............", "........W...",
    "............", "............", "...LLLLL....", "............",
];
#[rustfmt::skip]
const ALUS: [&str; 12] = [
    "............", ".......GGG..", "......GGDGG.", "......GDGGG.",
    ".......GGG..", "............", "..W.........", ".WWW.....Y..",
    ".WWW........", "WWWWW.......", "W...W.......", "............",
];
#[rustfmt::skip]
const MATO: [&str; 12] = [
    "............", ".NNNNNNNN...", ".N......N...", ".N.W.W..N...",
    ".N..W...N...", ".N......N...", ".N......N...", ".NNNNNY.N...",
    "........N...", "........NN..", "............", "............",
];
#[rustfmt::skip]
const KIVET: [&str; 12] = [
    "............", ".RRR.BBB.NNN", ".RRR.BBB.NNN", ".RRR.BBB.NNN",
    "............", ".YYY.YYY.YYY", ".YYY.YYY.YYY", ".YYY.YYY.YYY",
    "............", ".BBB.RRR.PPP", ".BBB.RRR.PPP", ".BBB.RRR.PPP",
];
#[rustfmt::skip]
const MIINA: [&str; 12] = [
    "............", ".....K......", "..K.KKK.K...", "...KKKKK....",
    "..KKWWKKK...", ".KKKWKKKKK..", "..KKKKKKK...", "...KKKKK....",
    "..K.KKK.K...", ".....K......", "............", "............",
];
#[rustfmt::skip]
const KAKSIYKSI: [&str; 12] = [
    "............", ".WWWWWWWWWW.", ".WWWWWWWWWW.", ".WKKKWWKWWW.",
    ".WWWKWKKWWW.", ".WKKKWWKWWW.", ".WKWWWWKWWW.", ".WKKKWKKKWW.",
    ".WWWWWWWWWW.", ".WWWWWWWWWW.", "............", "............",
];
#[rustfmt::skip]
const MERKKI_KUVA: [&str; 12] = [
    "............", "....RRRR....", "..RWRRRRWR..", ".RRRWWWWRRR.",
    ".RRWRRRRWRR.", "RWRWRRRRWRWR", "RWRWRRRRWRWR", ".RRWRRRRWRR.",
    ".RRRWWWWRRR.", "..RWRRRRWR..", "....RRRR....", "............",
];
#[rustfmt::skip]
const LAATTA: [&str; 12] = [
    "............", "..WWWWWWWW..", "..WWWWRWWW..", "..WRRRRRRW..",
    "..WRWWRWRW..", "..WRRRRRRW..", "..WWWWRWWW..", "..WWWWRWWW..",
    "..WWWWWWWW..", "..OOOOOOOO..", "..OOOOOOOO..", "............",
];
#[rustfmt::skip]
const MAJAKKA_KUVA: [&str; 12] = [
    "...........Y", ".....K...YYY", "....KYKYYYY.", "....KKK.YY..",
    ".....W......", ".....R......", ".....W......", "....RRR.....",
    "....WWW.....", "...DDDDD....", "BBDDDDDDDBBB", "BBBBBBBBBBBB",
];
#[rustfmt::skip]
const KAIVO_KUVA: [&str; 12] = [
    "............", "..GGGGGG....", ".GG....GO...", "G......O....",
    "......O.....", ".....O......", "....O.......", "...O........",
    "..O....Y....", ".O....YYY...", "O....YYWYY..", "....YYYYYYY.",
];
#[rustfmt::skip]
const KAIUTIN: [&str; 5] = ["..W...", "WWW.C.", "WWW..C", "WWW.C.", "..W..."];
// Kolmiot pikselikuvina: monikulmiona näin pieni kolmio pyöristyy ruudukkoon vinoksi.
const YLOS_KUVA: [&str; 3] = ["..W..", ".WWW.", "WWWWW"];
const ALAS_KUVA: [&str; 3] = ["WWWWW", ".WWW.", "..W.."];
const NUOLI_KUVA: [&str; 5] = ["W..", "WW.", "WWW", "WW.", "W.."];
const PELAA_KUVA: [&str; 7] = ["W...", "WW..", "WWW.", "WWWW", "WWW.", "WW..", "W..."];

/// Kuvakkeen pohjaväri ja kuva.
fn kuvake(laji: Laji) -> (u32, &'static [&'static str; 12]) {
    match laji {
        Laji::Laivanupotus => (0x1A3A8A, &LAIVA),
        Laji::Luvut => (0x3A2A6A, &LAATAT),
        Laji::Pasianssi => (0x1E6A3A, &KORTTI),
        Laji::Mahjong => (0x6A2418, &LAATTA),
        Laji::Tiilirikko => (TAUSTA, &TIILET),
        Laji::Asteroidit => (0x14102E, &ALUS),
        Laji::Silmukka => (0x183A28, &MATO),
        Laji::Palikka => (0x3A1D6E, &KIVET),
        Laji::Miinaharava => (LAVENTELI, &MIINA),
        Laji::Blackjack => (0x1E6A3A, &KAKSIYKSI),
        Laji::Holdem => (0x1E5A3A, &MERKKI_KUVA),
        Laji::Majakka => (0x0C1450, &MAJAKKA_KUVA),
        Laji::Kaivo => (0x3A2418, &KAIVO_KUVA),
    }
}

fn kuvakkeen_vari(merkki: u8) -> Option<u32> {
    Some(match merkki {
        b'Y' => KELTAINEN,
        b'O' => WB_ORANSSI,
        b'W' => VALKOINEN,
        b'L' => LAVENTELI,
        b'G' => KANSI,
        b'D' => RUNKO,
        b'R' => 0xE04838,
        b'C' => HUTI,
        b'P' => TAIVAS_ALA,
        b'K' => PALKKI,
        b'B' => 0x3878E8,
        b'N' => 0x58C850,
        _ => return None,
    })
}

fn pieni_leveys(teksti: &str) -> i32 {
    teksti.chars().count() as i32 * PIENI_MERKKI
}

/// Ohjeen selite ja sen väri. Ensimmäisen Esc-painalluksen jälkeen lopetuksen selite kertoo
/// keltaisella, että toinen painallus sulkee ohjelman.
fn ohjeen_selite(selite: &str, kysyy_lopetusta: bool) -> (&'static str, u32) {
    if kysyy_lopetusta && selite == LOPETA {
        (kaanna(LOPETA_UUDELLEEN), KELTAINEN)
    } else {
        (kaanna(selite), LAVENTELI)
    }
}

fn nappaimen_leveys(nimi: &str) -> i32 {
    match nimi {
        YLOS | ALAS => NAPPAIN,
        VALI => nappaimen_leveys(kaanna(VALI)),
        _ => (pieni_leveys(nimi) + 16).max(NAPPAIN),
    }
}

fn sirun_leveys(teksti: &str, kaiutin: bool) -> i32 {
    SIRU_TEKSTI_X + kaiutin as i32 * KAIUTIN_TILA + pieni_leveys(teksti) + SIRU_LOPPU
}

/// Äänikytkin on oikeassa yläkulmassa, ja sen leveys ei muutu tilan mukana.
fn aanikytkimen_alue() -> (i32, i32) {
    let l = sirun_leveys(kaanna(AANI_PAALLA), true).max(sirun_leveys(kaanna(AANI_POIS), true));
    (OIKEA_REUNA - l, l)
}

fn kielikytkimen_alue() -> (i32, i32) {
    let l = sirun_leveys(kaanna(KIELIKYTKIN), false);
    (aanikytkimen_alue().0 - SIRU_VALI - l, l)
}

/// Uuden version ilmoitus kytkinten vasemmalla puolella: (x, leveys).
fn ilmoituksen_alue(versio: &str) -> (i32, i32) {
    let teksti = muotoile("valikko.uusi_versio", &[&versio]);
    let l = ILMOITUS_TEKSTI_X + pieni_leveys(&teksti) + SIRU_LOPPU;
    (kielikytkimen_alue().0 - SIRU_VALI - l, l)
}

fn sirulla((vasen, leveys): (i32, i32), x: i32, y: i32) -> bool {
    (vasen..vasen + leveys).contains(&x) && (SIRU_Y..SIRU_Y + SIRU_K).contains(&y)
}

/// Ilmoitusta klikkaamalla pääsee julkaisusivulle.
pub fn ilmoituksella(x: i32, y: i32, versio: &str) -> bool {
    sirulla(ilmoituksen_alue(versio), x, y)
}

pub fn aanikytkimella(x: i32, y: i32) -> bool {
    sirulla(aanikytkimen_alue(), x, y)
}

pub fn kielikytkimella(x: i32, y: i32) -> bool {
    sirulla(kielikytkimen_alue(), x, y)
}

fn pelaa_napin_leveys() -> i32 {
    let teksti = kaanna(PELAA).chars().count() as i32 * KUVAUS_KERROIN * 8;
    2 * PELAA_SISENNYS + PELAA_KOLMIO + 2 * PELAA_VALI + teksti + nappaimen_leveys(VALI)
}

/// Painike seuraa kuvausta, jonka pituus vaihtelee pelistä ja kielestä toiseen.
fn pelaa_y(valittu: usize) -> i32 {
    let rivit = rivita(kaanna(PELIT[valittu].kuvaus), KUVAUS_MERKKEJA).len() as i32;
    KUVAUS_Y + rivit * KUVAUS_RIVI - KUVAUS_VALI + PELAA_YLA
}

/// Pelaa-painike esittelyn alla käynnistää valitun pelin.
pub fn pelaa_napilla(x: i32, y: i32, valittu: usize) -> bool {
    let yla = pelaa_y(valittu);
    (ESITTELY_X..ESITTELY_X + pelaa_napin_leveys()).contains(&x)
        && (yla..yla + PELAA_K).contains(&y)
}

/// Pitkä nimi pienemmällä kirjasimella, jotta se mahtuu mallin vasemmalle puolelle.
fn nimen_koko(nimi: &str) -> (i32, i32, i32) {
    if nimi.chars().count() as i32 * SUURI_NIMI.2 <= ESITTELY_L {
        SUURI_NIMI
    } else {
        PIENEMPI_NIMI
    }
}

/// Näkyvä paikka, jonka kortin kohdalla piste on. Käynnistin tietää, mikä peli paikalla on.
pub fn kortti_pisteesta(x: i32, y: i32) -> Option<usize> {
    if !(KORTTI_X..KORTTI_X + KORTTI_L).contains(&x) {
        return None;
    }
    let i = (y - KORTTI_Y).div_euclid(KORTTI_ASKEL);
    let sisalla = (y - KORTTI_Y).rem_euclid(KORTTI_ASKEL) < KORTTI_K;
    (sisalla && (0..NAKYVIA_KORTTEJA).contains(&i)).then_some(i as usize)
}

/// Piirtää esillä olevan näkymän ruudulle. Pelien klassiset näkymät kulkevat välikuvan kautta.
pub fn piirra(kaynnistin: &Kaynnistin, kehys: &mut Kehys) {
    if let Some(peli) = &kaynnistin.peli {
        peli.nakyma().piirra(kehys);
        return;
    }
    let mut p = Piirtaja {
        k: Kangas::kokoinen(&mut kehys.ruutu, RUUDUN_LEVEYS, RUUDUN_KORKEUS),
        kello: kaynnistin.kello_ms,
    };
    p.valikko(kaynnistin);
    kehys.piirretty_suoraan();
}

impl Piirtaja<'_> {
    fn valikko(&mut self, kaynnistin: &Kaynnistin) {
        MAISEMA.piirra(&mut self.k, 0, 0);
        let ika = kaynnistin.kello_ms.saturating_sub(kaynnistin.valittu_ms);
        self.valikon_malli(&kaynnistin.malli, ika);
        self.ylapalkki(kaynnistin);
        for paikka in 0..NAKYVIA_KORTTEJA as usize {
            let i = kaynnistin.alku + paikka;
            self.valikkokortti(paikka, i, i == kaynnistin.valittu);
        }
        let alempana = kaynnistin.alku + NAKYVIA < PELIT.len();
        self.vieritysnuolet(kaynnistin.alku > 0, alempana);
        self.esittely(kaynnistin.valittu);
        if let Some(tilastot) = kaynnistin.valitun_tilastot() {
            self.tilastolaatikko(tilastot);
        }
        let (netista, parhaat) = kaynnistin.valitun_parhaat();
        let laji = kaynnistin.valikon_listan_laji();
        self.listalaatikko(netista, laji, parhaat, kaynnistin.valitun_oma_paras());
        self.ohjepalkki(kaynnistin.kysyy_lopetusta());
    }

    /// `ika` on aika valinnan vaihtumisesta: uusi malli kasvaa täyteen kokoonsa.
    fn valikon_malli(&mut self, malli: &[Tahko], ika: u32) {
        let keinunta = (self.kello as f32 * 0.002).sin() * MALLIN_KEINUNTA;
        let kasvu = (ika as f32 / MALLIN_KASVU_MS as f32).min(1.0);
        self.k.malli(
            malli,
            self.kello as f32 * LAIVAN_KIERTO,
            (MALLI_X, MALLI_Y + keinunta as i32),
            MALLIN_KOKO * (MALLIN_ALKUKOKO + (1.0 - MALLIN_ALKUKOKO) * kasvu),
        );
    }

    /// Maiseman tummennettu kopio: näyttää läpikuultavalta lasilta.
    fn lasi(&mut self, x: i32, y: i32, l: i32, k: i32) {
        let alue = (x / P, y / P, l / P, k / P);
        let alue = (
            alue.0 as usize,
            alue.1 as usize,
            alue.2 as usize,
            alue.3 as usize,
        );
        LASI.piirra_alue(&mut self.k, x, y, alue);
    }

    /// Täysi värillinen pinta keltaisella kehyksellä ja kovalla varjolla.
    fn korostettu(&mut self, x: i32, y: i32, l: i32, k: i32) {
        self.k.laatikko(x + VARJO, y + VARJO, l, k, PALKKI);
        self.k.laatikko(x, y, l, k, KOROSTUS);
        self.k.kehys(x, y, l, k, KELTAINEN);
    }

    fn pieni(&mut self, x: i32, y: i32, teksti: &str, vari: u32) {
        self.teksti_koossa(x, y, teksti, PIENI, vari);
    }

    fn ylapalkki(&mut self, kaynnistin: &Kaynnistin) {
        let logo = kaanna("valikko.banneri");
        self.suurteksti(VASEN_REUNA, LOGO_Y, logo, LOGO_KOKO, KELTAINEN, true);
        if let Some(uusi) = kaynnistin.uusi_versio() {
            self.versioilmoitus(uusi, kaynnistin.ilmoitus_alla());
        }
        let kieli = kaanna(KIELIKYTKIN);
        self.siru(kielikytkimen_alue(), "L", kieli, VALKOINEN, None);
        let paalla = kaynnistin.aanet_paalla;
        let (aani, vari) = if paalla {
            (AANI_PAALLA, VALKOINEN)
        } else {
            (AANI_POIS, KOROSTUS)
        };
        self.siru(aanikytkimen_alue(), "M", kaanna(aani), vari, Some(paalla));
    }

    /// Kytkin yläpalkissa: näppäin, mahdollinen kaiutin ja tila.
    fn siru(
        &mut self,
        (x, l): (i32, i32),
        nappain: &str,
        teksti: &str,
        vari: u32,
        kaiutin: Option<bool>,
    ) {
        self.lasi(x, SIRU_Y, l, SIRU_K);
        self.k.kehys(x, SIRU_Y, l, SIRU_K, REUNA);
        let y = SIRU_Y + (SIRU_K - NAPPAIN) / 2;
        let nx = x + SIRUN_NAPPAIN_X;
        self.k.laatikko(nx, y, NAPPAIN, NAPPAIN, LAVENTELI);
        let reuna = (NAPPAIN - PIENI_MERKKI) / 2;
        self.pieni(nx + reuna, y + reuna, nappain, PALKKI);
        let mut tx = x + SIRU_TEKSTI_X;
        if let Some(paalla) = kaiutin {
            let aallot = if paalla { HUTI } else { LASIN_VARI };
            self.k.sprite((tx, y + 6), &KAIUTIN, P, |m| match m {
                b'W' => Some(VALKOINEN),
                b'C' => Some(aallot),
                _ => None,
            });
            tx += KAIUTIN_TILA;
        }
        self.pieni(tx, y + reuna, teksti, vari);
    }

    /// Hiiren alla ilmoitus vaalenee kuin painike, koska klikkaus avaa julkaisusivun.
    fn versioilmoitus(&mut self, versio: &str, hiiren_alla: bool) {
        let teksti = muotoile("valikko.uusi_versio", &[&versio]);
        let (x, l) = ilmoituksen_alue(versio);
        let tausta = if hiiren_alla { VALKOINEN } else { KELTAINEN };
        self.k.laatikko(x + P, SIRU_Y + P, l, SIRU_K, PALKKI);
        self.k.laatikko(x, SIRU_Y, l, SIRU_K, tausta);
        let keski = SIRU_Y + SIRU_K / 2;
        self.k.laatikko(x + 16, keski - P, 2 * P, 2 * P, PALKKI);
        let y = keski - PIENI_MERKKI / 2;
        self.pieni(x + ILMOITUS_TEKSTI_X, y, &teksti, PALKKI);
    }

    /// Kortti näkyvällä `paikka`lla. `i` on pelin järjestysnumero `PELIT`-listassa.
    fn valikkokortti(&mut self, paikka: usize, i: usize, valittu: bool) {
        let y = KORTTI_Y + paikka as i32 * KORTTI_ASKEL;
        let keski_y = y + KORTTI_K / 2;
        let x = KORTTI_X + if valittu { VALITUN_SIIRTO } else { 0 };
        let numero_x = x + NUMERO_X;
        let numero_y = keski_y - NAPPAIN / 2;
        let (nimi_vari, numero_vari) = if valittu {
            self.korostettu(x, y, KORTTI_L, KORTTI_K);
            let nx = x + KORTTI_L - KORTIN_NUOLI + 12;
            self.kolmio((nx, keski_y - 10), &NUOLI_KUVA, PALKKI);
            self.k
                .laatikko(numero_x, numero_y, NAPPAIN, NAPPAIN, PALKKI);
            (PALKKI, KOROSTUS)
        } else {
            self.lasi(x, y, KORTTI_L, KORTTI_K);
            self.k.kehys(x, y, KORTTI_L, KORTTI_K, HIMMEA_REUNA);
            self.k.kehys(numero_x, numero_y, NAPPAIN, NAPPAIN, REUNA);
            (VALKOINEN, KELTAINEN)
        };
        // Kymmenes kortti on nolla, kuten numerorivillä.
        let numero = ((paikka + 1) % 10).to_string();
        let reuna = (NAPPAIN - PIENI_MERKKI) / 2;
        self.pieni(numero_x + reuna, numero_y + reuna, &numero, numero_vari);

        let (kx, ky) = (x + KUVAKE_X, keski_y - KUVAKE_L / 2);
        let (pohja, kuva) = kuvake(PELIT[i].laji);
        self.k.laatikko(kx, ky, KUVAKE_L, KUVAKE_L, PALKKI);
        self.k
            .laatikko(kx + P, ky + P, KUVAKE_L - 2 * P, KUVAKE_L - 2 * P, pohja);
        self.k.sprite((kx + P, ky + P), kuva, P, kuvakkeen_vari);

        let nimi = kaanna(PELIT[i].nimi).to_uppercase();
        let teksti_y = keski_y - NIMEN_KORKEUS / 2;
        self.suurteksti(x + NIMI_X, teksti_y, &nimi, NIMI_KOKO, nimi_vari, false);
    }

    /// Nuolet listan vieressä, kun sen ylä- tai alapuolelle jää pelejä piiloon.
    fn vieritysnuolet(&mut self, ylempana: bool, alempana: bool) {
        let vasen = NUOLI_X - NUOLI_L / 2;
        if ylempana {
            self.kolmio((vasen, KORTTI_Y), &YLOS_KUVA, KELTAINEN);
        }
        if alempana {
            self.kolmio((vasen, LISTAN_ALA - NUOLI_K), &ALAS_KUVA, KELTAINEN);
        }
    }

    /// Pikselikuva yhdellä värillä konepikselin kokoisena.
    fn kolmio(&mut self, kohta: (i32, i32), kuva: &[&str], vari: u32) {
        self.k
            .sprite(kohta, kuva, P, |m| (m == b'W').then_some(vari));
    }

    /// Valitun pelin järjestysnumero, nimi, kuvaus ja pelaa-painike mallin vasemmalla puolella.
    fn esittely(&mut self, valittu: usize) {
        let kortti = &PELIT[valittu];
        let jarjestys = muotoile("valikko.peli_nro", &[&(valittu + 1), &PELIT.len()]);
        self.pieni(ESITTELY_X, JARJESTYS_Y, &jarjestys, LAVENTELI);
        let nimi = kaanna(kortti.nimi).to_uppercase();
        self.suurteksti(
            ESITTELY_X,
            NIMI_Y,
            &nimi,
            nimen_koko(&nimi),
            KELTAINEN,
            true,
        );
        let mut y = KUVAUS_Y;
        for rivi in rivita(kaanna(kortti.kuvaus), KUVAUS_MERKKEJA) {
            self.teksti_koossa(ESITTELY_X, y, rivi, KUVAUS_KERROIN, VALKOINEN);
            y += KUVAUS_RIVI;
        }
        self.pelaa_nappi(pelaa_y(valittu));
    }

    fn pelaa_nappi(&mut self, y: i32) {
        let l = pelaa_napin_leveys();
        self.korostettu(ESITTELY_X, y, l, PELAA_K);
        let keski = y + PELAA_K / 2;
        let mut x = ESITTELY_X + PELAA_SISENNYS;
        self.kolmio((x, keski - 14), &PELAA_KUVA, PALKKI);
        x += PELAA_KOLMIO + PELAA_VALI;
        let teksti = kaanna(PELAA).to_uppercase();
        let korkeus = KUVAUS_KERROIN * 8;
        self.teksti_koossa(x, keski - korkeus / 2, &teksti, KUVAUS_KERROIN, PALKKI);
        x += teksti.chars().count() as i32 * korkeus + PELAA_VALI;
        let vali = kaanna(VALI);
        let nl = nappaimen_leveys(VALI);
        self.k.laatikko(x, keski - NAPPAIN / 2, nl, NAPPAIN, PALKKI);
        let tx = x + (nl - pieni_leveys(vali)) / 2;
        self.pieni(tx, keski - PIENI_MERKKI / 2, vali, KOROSTUS);
    }

    /// Lasinen laatikko kovalla varjolla. Palauttaa sisällön vasemman yläkulman.
    fn laatikko(&mut self, x: i32, l: i32) -> (i32, i32) {
        self.k
            .laatikko(x + VARJO, LAATIKKO_Y + VARJO, l, LAATIKKO_K, PALKKI);
        self.lasi(x, LAATIKKO_Y, l, LAATIKKO_K);
        self.k.kehys(x, LAATIKKO_Y, l, LAATIKKO_K, REUNA);
        (x + LAATIKKO_SISENNYS, LAATIKKO_Y + LAATIKKO_SISENNYS)
    }

    /// Voitot, tappiot ja paras voitto pisteinä kultakin tasolta.
    fn tilastolaatikko(&mut self, tilastot: &[Tilasto; 3]) {
        let (x, y) = self.laatikko(TILASTO_X, TILASTO_L);
        self.pieni(x, y, kaanna(TILASTOT), KELTAINEN);
        let y = y + TAULUKON_RIVI;
        for (avain, oikea) in TAULUKON_SARAKKEET {
            self.oikealle(x + oikea, y, kaanna(avain), LAVENTELI);
        }
        for (i, (tilasto, taso)) in tilastot.iter().zip(Vaikeus::KAIKKI).enumerate() {
            let y = y + (i as i32 + 1) * TAULUKON_RIVI;
            self.pieni(x, y, kaanna(TASOJEN_NIMET[i]), VALKOINEN);
            let paras = match tilasto.parhaat_pisteet(taso) {
                0 => "-".to_string(),
                pisteet => pisteet.to_string(),
            };
            let arvot = [
                tilasto.voitot.to_string(),
                tilasto.tappiot.to_string(),
                paras,
            ];
            for (arvo, (_, oikea)) in arvot.iter().zip(TAULUKON_SARAKKEET) {
                self.oikealle(x + oikea, y, arvo, VALKOINEN);
            }
        }
    }

    /// Top 3: otsikon alla lajin nimi, kun pelin lajeilla on omat listansa, ja viivan alla
    /// pelaajan oma paras, myös listalle yltämätön.
    fn listalaatikko(
        &mut self,
        netista: bool,
        laji: Option<&str>,
        parhaat: &[Tulos],
        oma: Option<u32>,
    ) {
        let (x, mut y) = self.laatikko(LISTA_X, LISTA_L);
        let leveys = LISTA_L - 2 * LAATIKKO_SISENNYS;
        self.pieni(x, y, listan_otsikko(netista), KELTAINEN);
        y += LAATIKON_OTSIKKO;
        if let Some(laji) = laji {
            self.pieni(x, y, laji, LAVENTELI);
            y += LAATIKON_OTSIKKO;
        }
        self.parhaat_rivit(x, y, leveys, LISTA_KOKO, parhaat);
        y += PARHAITA as i32 * PARHAAT_RIVI + OMA_VALI;
        self.k.laatikko(x, y, leveys, P, HIMMEA_REUNA);
        y += P + 2 * OMA_VALI;
        self.pieni(x, y, kaanna(OMA_PARAS), LAVENTELI);
        let oma = oma.map_or("-".to_string(), |oma| oma.to_string());
        self.oikealle(x + leveys, y, &oma, KELTAINEN);
    }

    /// Pieni teksti, jonka oikea reuna on kohdassa `oikea`.
    fn oikealle(&mut self, oikea: i32, y: i32, teksti: &str, vari: u32) {
        self.pieni(oikea - pieni_leveys(teksti), y, teksti, vari);
    }

    /// Näppäimet kuvina selitteineen ja versio oikeassa laidassa.
    fn ohjepalkki(&mut self, kysyy_lopetusta: bool) {
        let korkeus = RUUDUN_KORKEUS as i32 - ALAPALKKI_Y;
        self.k.laatikko(0, ALAPALKKI_Y, RUUTU_L, korkeus, PALKKI);
        self.k.laatikko(0, ALAPALKKI_Y, RUUTU_L, P, TAUSTA);
        let mut x = VASEN_REUNA;
        for (nappaimet, selite) in OHJEET {
            for nimi in nappaimet {
                self.ohjenappain(x, nimi);
                x += nappaimen_leveys(nimi) + NAPPAINTEN_VALI;
            }
            x += SELITTEEN_VALI - NAPPAINTEN_VALI;
            let (selite, vari) = ohjeen_selite(selite, kysyy_lopetusta);
            self.pieni(x, OHJE_TEKSTI_Y, selite, vari);
            x += pieni_leveys(selite) + RYHMIEN_VALI;
        }
        let versio = muotoile("valikko.versio", &[&VERSIO]);
        self.oikealle(OIKEA_REUNA, OHJE_TEKSTI_Y, &versio, REUNA);
    }

    fn ohjenappain(&mut self, x: i32, nimi: &str) {
        let y = OHJE_NAPPAIN_Y;
        let l = nappaimen_leveys(nimi);
        // 5x3 konepikselin kolmio keskellä näppäintä
        let kolmio = (x + (l - 5 * P) / 2, y + (NAPPAIN - 3 * P) / 2);
        match nimi {
            YLOS => {
                self.k.kehys(x, y, l, NAPPAIN, REUNA);
                self.kolmio(kolmio, &YLOS_KUVA, VALKOINEN);
            }
            ALAS => {
                self.k.kehys(x, y, l, NAPPAIN, REUNA);
                self.kolmio(kolmio, &ALAS_KUVA, VALKOINEN);
            }
            VALI => {
                let teksti = kaanna(VALI);
                self.k.laatikko(x, y, l, NAPPAIN, KOROSTUS);
                let tx = x + (l - pieni_leveys(teksti)) / 2;
                self.pieni(tx, OHJE_TEKSTI_Y, teksti, PALKKI);
            }
            _ => {
                self.k.kehys(x, y, l, NAPPAIN, REUNA);
                let tx = x + (l - pieni_leveys(nimi)) / 2;
                self.pieni(tx, OHJE_TEKSTI_Y, nimi, VALKOINEN);
            }
        }
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
pub use tests::{
    aanikytkimen_keskipiste, ilmoituksen_keskipiste, kielikytkimen_keskipiste, kortin_keskipiste,
    pelaa_napin_keskipiste,
};
