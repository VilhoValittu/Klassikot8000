//! 5 suoran säännöt ja tietokonevastustaja ilman piirtoa. Lauta on 15x15, risti aloittaa,
//! ja viisi tai useampi omaa merkkiä vaaka-, pysty- tai vinorivissä voittaa.

use crate::Arpa;

pub const KOKO: usize = 15;
pub const RUUTUJA: usize = KOKO * KOKO;
/// Aloittajan siirtoja on enintään puolet laudasta ylöspäin pyöristettynä.
pub const SIIRTOJA_ENINTAAN: u32 = RUUTUJA.div_ceil(2) as u32;
pub const SUORA: usize = 5;

/// (rivi, sarake)
pub type Paikka = (usize, usize);

/// Rivin suunnat: vaaka, pysty ja kaksi vinoa, (rivin muutos, sarakkeen muutos).
pub const SUUNNAT: [(i32, i32); 4] = [(0, 1), (1, 0), (1, 1), (1, -1)];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Merkki {
    Risti,
    Nolla,
}

impl Merkki {
    pub fn vastustaja(self) -> Merkki {
        match self {
            Merkki::Risti => Merkki::Nolla,
            Merkki::Nolla => Merkki::Risti,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tila {
    Kesken,
    Voitto(Merkki),
    /// Lauta täyttyi ilman suoraa.
    Tasapeli,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vaikeus {
    Helppo,
    Normaali,
    Vaikea,
}

impl Vaikeus {
    pub const KAIKKI: [Vaikeus; 3] = [Vaikeus::Helppo, Vaikeus::Normaali, Vaikeus::Vaikea];

    pub fn nimi(self) -> &'static str {
        match self {
            Vaikeus::Helppo => "Helppo",
            Vaikeus::Normaali => "Normaali",
            Vaikeus::Vaikea => "Vaikea",
        }
    }

    pub const fn kerroin(self) -> u32 {
        match self {
            Vaikeus::Helppo => 1,
            Vaikeus::Normaali => 2,
            Vaikeus::Vaikea => 3,
        }
    }
}

/// Voiton pisteet: mitä vähemmällä omalla siirrolla, sitä enemmän, ja taso kertoo ne.
pub const fn pisteet(vaikeus: Vaikeus, siirtoja: u32) -> u32 {
    let pohja = (SIIRTOJA_ENINTAAN + 1).saturating_sub(siirtoja);
    let pohja = if pohja == 0 { 1 } else { pohja };
    pohja * vaikeus.kerroin()
}

/// Nopein mahdollinen voitto vaikealla tasolla: viisi siirtoa.
pub const SUURIN_TULOS: u32 = pisteet(Vaikeus::Vaikea, SUORA as u32);

type Ruudukko = [Option<Merkki>; RUUTUJA];

fn indeksi((rivi, sarake): Paikka) -> usize {
    rivi * KOKO + sarake
}

fn laudalla(rivi: i32, sarake: i32) -> Option<Paikka> {
    let koko = KOKO as i32;
    ((0..koko).contains(&rivi) && (0..koko).contains(&sarake))
        .then_some((rivi as usize, sarake as usize))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lauta {
    ruudut: Ruudukko,
    pub vuoro: Merkki,
    pub tila: Tila,
    /// Siirrot järjestyksessä. Viimeinen korostetaan laudalla.
    pub siirrot: Vec<Paikka>,
    /// Voittaneen suoran ruudut päästä päähän.
    pub voittorivi: Vec<Paikka>,
}

impl Default for Lauta {
    fn default() -> Self {
        Self::new()
    }
}

impl Lauta {
    pub fn new() -> Self {
        Self {
            ruudut: [None; RUUTUJA],
            vuoro: Merkki::Risti,
            tila: Tila::Kesken,
            siirrot: Vec::new(),
            voittorivi: Vec::new(),
        }
    }

    /// Lauta valmiilla merkeillä, kokeita ja kehityslippuja varten. Vuoro on sillä,
    /// jolla on vähemmän merkkejä (tasan: risti). Voittoa ei tarkisteta.
    pub fn merkeista(ristit: &[Paikka], nollat: &[Paikka]) -> Self {
        let mut lauta = Self::new();
        for (&paikka, merkki) in ristit
            .iter()
            .map(|p| (p, Merkki::Risti))
            .chain(nollat.iter().map(|p| (p, Merkki::Nolla)))
        {
            lauta.ruudut[indeksi(paikka)] = Some(merkki);
            lauta.siirrot.push(paikka);
        }
        lauta.vuoro = if ristit.len() > nollat.len() {
            Merkki::Nolla
        } else {
            Merkki::Risti
        };
        lauta
    }

    pub fn ruutu(&self, paikka: Paikka) -> Option<Merkki> {
        self.ruudut[indeksi(paikka)]
    }

    pub fn viimeisin(&self) -> Option<Paikka> {
        self.siirrot.last().copied()
    }

    /// Merkin tekemät siirrot.
    pub fn siirtoja(&self, merkki: Merkki) -> u32 {
        self.ruudut.iter().filter(|&&r| r == Some(merkki)).count() as u32
    }

    /// Vuorossa oleva merkitsee ruudun. `None`, jos ruutu on varattu, laudan ulkopuolella
    /// tai peli on jo ohi.
    pub fn aseta(&mut self, paikka: Paikka) -> Option<Tila> {
        if self.tila != Tila::Kesken
            || paikka.0 >= KOKO
            || paikka.1 >= KOKO
            || self.ruutu(paikka).is_some()
        {
            return None;
        }
        let merkki = self.vuoro;
        self.ruudut[indeksi(paikka)] = Some(merkki);
        self.siirrot.push(paikka);
        if let Some(rivi) = self.suora(paikka, merkki) {
            self.voittorivi = rivi;
            self.tila = Tila::Voitto(merkki);
        } else if self.siirrot.len() == RUUTUJA {
            self.tila = Tila::Tasapeli;
        } else {
            self.vuoro = merkki.vastustaja();
        }
        Some(self.tila)
    }

    /// Ruudun kautta kulkeva vähintään viiden suora, jos sellainen on.
    fn suora(&self, paikka: Paikka, merkki: Merkki) -> Option<Vec<Paikka>> {
        for (dr, ds) in SUUNNAT {
            let jatko = |suunta: i32| {
                let mut ruudut = Vec::new();
                let mut askel = 1;
                while let Some(p) = laudalla(
                    paikka.0 as i32 + dr * suunta * askel,
                    paikka.1 as i32 + ds * suunta * askel,
                ) {
                    if self.ruutu(p) != Some(merkki) {
                        break;
                    }
                    ruudut.push(p);
                    askel += 1;
                }
                ruudut
            };
            let (taakse, eteen) = (jatko(-1), jatko(1));
            if taakse.len() + 1 + eteen.len() >= SUORA {
                let mut rivi: Vec<Paikka> = taakse.into_iter().rev().collect();
                rivi.push(paikka);
                rivi.extend(eteen);
                return Some(rivi);
            }
        }
        None
    }
}

// Kone arvioi ruudun katsomalla sen kautta kulkevat neljä riviä, kustakin yhdeksän ruutua.

const TYHJA: u8 = 0;
const OMA: u8 = 1;
const ESTE: u8 = 2;

/// Rivin muoto, kun ruutuun asetetaan oma merkki. Järjestys on vahvuusjärjestys.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Muoto {
    Ei,
    Kaksi,
    /// Voi kasvaa avoimeksi kolmoseksi.
    AvoinKaksi,
    Kolme,
    /// Voi kasvaa avoimeksi neloseksi, joten se on torjuttava heti.
    AvoinKolme,
    /// Yksi ruutu puuttuu viidestä.
    Nelja,
    /// Kaksi eri ruutua tekisi viiden: torjumaton.
    AvoinNelja,
    Viisi,
}

impl Muoto {
    fn arvo(self) -> u64 {
        match self {
            Muoto::Ei => 0,
            Muoto::Kaksi => 10,
            Muoto::AvoinKaksi => 60,
            Muoto::Kolme => 80,
            Muoto::AvoinKolme => 600,
            Muoto::Nelja => 700,
            Muoto::AvoinNelja => 20_000,
            Muoto::Viisi => 1_000_000,
        }
    }
}

type Linja = [u8; 9];

/// Yhdeksän ruudun rivi keskellään `paikka`, johon oletetaan oma merkki.
fn linja(r: &Ruudukko, paikka: Paikka, (dr, ds): (i32, i32), oma: Merkki) -> Linja {
    let mut l = [ESTE; 9];
    for (i, solu) in l.iter_mut().enumerate() {
        let k = i as i32 - 4;
        if let Some(p) = laudalla(paikka.0 as i32 + dr * k, paikka.1 as i32 + ds * k) {
            *solu = match r[indeksi(p)] {
                None => TYHJA,
                Some(m) if m == oma => OMA,
                Some(_) => ESTE,
            };
        }
    }
    l[4] = OMA;
    l
}

fn viisi(l: &Linja) -> bool {
    let vasen = l[..4].iter().rev().take_while(|&&c| c == OMA).count();
    let oikea = l[5..].iter().take_while(|&&c| c == OMA).count();
    vasen + 1 + oikea >= SUORA
}

/// Eri ruudut, joista keskikohdan kautta kulkeva rivi täydentyisi viideksi.
fn voittopisteet(l: &Linja) -> u32 {
    let mut pisteet = 0u16;
    for alku in 0..=4 {
        let ikkuna = &l[alku..alku + SUORA];
        if ikkuna.contains(&ESTE) || ikkuna.iter().filter(|&&c| c == OMA).count() != 4 {
            continue;
        }
        if let Some(tyhja) = ikkuna.iter().position(|&c| c == TYHJA) {
            pisteet |= 1 << (alku + tyhja);
        }
    }
    pisteet.count_ones()
}

/// Paras muoto, joka syntyy yhdellä lisämerkillä rivin tyhjään ruutuun.
fn parannus(l: &Linja, muoto: impl Fn(&Linja) -> Muoto) -> Muoto {
    let mut paras = Muoto::Ei;
    for i in 0..l.len() {
        if l[i] == TYHJA {
            let mut k = *l;
            k[i] = OMA;
            paras = paras.max(muoto(&k));
        }
    }
    paras
}

fn nelja(l: &Linja) -> Muoto {
    if viisi(l) {
        return Muoto::Viisi;
    }
    match voittopisteet(l) {
        0 => Muoto::Ei,
        1 => Muoto::Nelja,
        _ => Muoto::AvoinNelja,
    }
}

fn kolme(l: &Linja) -> Muoto {
    match nelja(l) {
        Muoto::AvoinNelja | Muoto::Viisi => Muoto::AvoinKolme,
        Muoto::Nelja => Muoto::Kolme,
        _ => Muoto::Ei,
    }
}

fn muoto(l: &Linja) -> Muoto {
    let neloset = nelja(l);
    if neloset != Muoto::Ei {
        return neloset;
    }
    let kolmoset = parannus(l, kolme);
    if kolmoset != Muoto::Ei {
        return kolmoset;
    }
    match parannus(l, |k| parannus(k, kolme)) {
        Muoto::AvoinKolme => Muoto::AvoinKaksi,
        Muoto::Kolme => Muoto::Kaksi,
        _ => Muoto::Ei,
    }
}

/// Merkin tekemä muoto ruutuun kussakin suunnassa.
pub fn muodot(lauta: &Lauta, paikka: Paikka, merkki: Merkki) -> [Muoto; 4] {
    SUUNNAT.map(|d| muoto(&linja(&lauta.ruudut, paikka, d, merkki)))
}

/// Ruudun arvo merkille: rivien muodot ja niiden yhdistelmät. Kaksi nelosta tai nelonen
/// ja avoin kolmonen voittavat yhtä varmasti kuin avoin nelonen.
fn arvio(r: &Ruudukko, paikka: Paikka, merkki: Merkki) -> u64 {
    let (mut summa, mut neloset, mut kolmoset) = (0, 0, 0);
    for d in SUUNNAT {
        let m = muoto(&linja(r, paikka, d, merkki));
        summa += m.arvo();
        match m {
            Muoto::Nelja | Muoto::AvoinNelja => neloset += 1,
            Muoto::AvoinKolme => kolmoset += 1,
            _ => {}
        }
    }
    summa
        + if neloset >= 2 {
            20_000
        } else if neloset == 1 && kolmoset >= 1 {
            15_000
        } else if kolmoset >= 2 {
            5_000
        } else {
            0
        }
}

/// Merkin paras uhka ruutuun: vain viitonen ja neloset, joten tämä on haun nopea tarkistus.
fn uhka(r: &Ruudukko, paikka: Paikka, merkki: Merkki) -> (Muoto, u32) {
    let (mut paras, mut neloset) = (Muoto::Ei, 0);
    for d in SUUNNAT {
        let m = nelja(&linja(r, paikka, d, merkki));
        if m >= Muoto::Nelja {
            neloset += 1;
        }
        paras = paras.max(m);
    }
    (paras, neloset)
}

/// Tyhjät ruudut kahden ruudun päässä merkeistä. Tyhjällä laudalla keskiruutu.
fn ehdokkaat(r: &Ruudukko) -> Vec<Paikka> {
    let mut lahella = [false; RUUTUJA];
    let mut merkkeja = 0;
    for (i, ruutu) in r.iter().enumerate() {
        if ruutu.is_none() {
            continue;
        }
        merkkeja += 1;
        let (rivi, sarake) = ((i / KOKO) as i32, (i % KOKO) as i32);
        for dr in -2..=2 {
            for ds in -2..=2 {
                if let Some(p) = laudalla(rivi + dr, sarake + ds) {
                    lahella[indeksi(p)] = true;
                }
            }
        }
    }
    if merkkeja == 0 {
        return vec![(KOKO / 2, KOKO / 2)];
    }
    (0..RUUTUJA)
        .filter(|&i| lahella[i] && r[i].is_none())
        .map(|i| (i / KOKO, i % KOKO))
        .collect()
}

fn viiden_ruutu(r: &Ruudukko, ehdokkaat: &[Paikka], merkki: Merkki) -> Option<Paikka> {
    ehdokkaat
        .iter()
        .copied()
        .find(|&p| uhka(r, p, merkki).0 == Muoto::Viisi)
}

/// Haun solmurajat: vaikeankin koneen siirto pysyy millisekuntien työnä.
const OMA_HAKU: u32 = 600;
const VASTA_HAKU: u32 = 100;
/// Näin moni paras ehdokas tarkistetaan vastustajan uhkasarjan varalta.
const TARKISTETTAVIA: usize = 6;
const HAUN_SYVYYS: u32 = 8;

/// Tekeekö merkki ruutuun avoimen kolmosen suuntaan `d`. Rivillä on oltava ainakin kaksi
/// muuta omaa merkkiä, joten raskas tarkistus jää useimmilta ruuduilta pois.
fn avoin_kolme(r: &Ruudukko, p: Paikka, d: (i32, i32), merkki: Merkki) -> bool {
    let l = linja(r, p, d, merkki);
    l.iter().filter(|&&c| c == OMA).count() >= 3 && parannus(&l, nelja) >= Muoto::AvoinNelja
}

/// Ruudut, joista vastustaja estää kolmosen kasvun avoimeksi neloseksi.
fn torjunnat(r: &Ruudukko, p: Paikka, (dr, ds): (i32, i32), merkki: Merkki) -> Vec<Paikka> {
    let l = linja(r, p, (dr, ds), merkki);
    (0..l.len())
        .filter(|&i| {
            let mut k = l;
            k[i] = ESTE;
            l[i] == TYHJA && parannus(&k, nelja) < Muoto::AvoinNelja
        })
        .filter_map(|i| {
            let askel = i as i32 - 4;
            laudalla(p.0 as i32 + dr * askel, p.1 as i32 + ds * askel)
        })
        .collect()
}

/// Voittava uhkasarja: joka siirto tekee nelosen tai avoimen kolmosen, kunnes syntyy
/// viitonen tai torjumaton uhka. Nelosella torjuntoja on yksi, kolmosella kaikki rivin
/// torjunnat kokeillaan. Kolmonen ei pakota, jos vastustaja voi vastata nelosella, joten
/// silloin haku tyytyy neloksiin. `kolmoset` epätosi: pelkät neloset (VCF).
/// Palauttaa sarjan ensimmäisen siirron.
fn uhkasarja(
    r: &mut Ruudukko,
    merkki: Merkki,
    syvyys: u32,
    solmut: &mut u32,
    kolmoset: bool,
) -> Option<Paikka> {
    if *solmut == 0 {
        return None;
    }
    *solmut -= 1;
    let vastus = merkki.vastustaja();
    // Kummankin uhat lasketaan kerran: ne ovat haun raskain osa.
    let ehdokkaat: Vec<(Paikka, (Muoto, u32), Muoto)> = ehdokkaat(r)
        .into_iter()
        .map(|p| (p, uhka(r, p, merkki), uhka(r, p, vastus).0))
        .collect();
    if let Some(&(p, ..)) = ehdokkaat.iter().find(|e| e.1.0 == Muoto::Viisi) {
        return Some(p);
    }
    // Vastustajan valmis nelonen voittaa ennen meidän uhkaamme.
    if syvyys == 0 || ehdokkaat.iter().any(|e| e.2 == Muoto::Viisi) {
        return None;
    }
    let kolmoset = kolmoset && !ehdokkaat.iter().any(|e| e.2 >= Muoto::Nelja);
    let jatkuu = |r: &mut Ruudukko, solmut: &mut u32, vastaukset: &[Paikka]| {
        !vastaukset.is_empty()
            && vastaukset.iter().all(|&q| {
                r[indeksi(q)] = Some(vastus);
                let jatko = uhkasarja(r, merkki, syvyys - 1, solmut, kolmoset);
                r[indeksi(q)] = None;
                jatko.is_some()
            })
    };
    for &(p, (muoto, neloset), _) in &ehdokkaat {
        if muoto == Muoto::AvoinNelja || neloset >= 2 {
            return Some(p);
        }
        let vastaukset = if neloset == 1 {
            r[indeksi(p)] = Some(merkki);
            // Viitosen ruutu on uuden nelosen ikkunassa, joten se on jo ehdokas.
            let torjunta = ehdokkaat
                .iter()
                .map(|e| e.0)
                .find(|&q| q != p && uhka(r, q, merkki).0 == Muoto::Viisi);
            r[indeksi(p)] = None;
            torjunta.into_iter().collect()
        } else if kolmoset {
            let mut vastaukset = Vec::new();
            for d in SUUNNAT {
                if avoin_kolme(r, p, d, merkki) {
                    r[indeksi(p)] = Some(merkki);
                    vastaukset.extend(torjunnat(r, p, d, merkki));
                    r[indeksi(p)] = None;
                }
            }
            vastaukset.sort_unstable();
            vastaukset.dedup();
            vastaukset
        } else {
            continue;
        };
        if vastaukset.is_empty() {
            continue;
        }
        r[indeksi(p)] = Some(merkki);
        let voittaa = jatkuu(r, solmut, &vastaukset);
        r[indeksi(p)] = None;
        if voittaa {
            return Some(p);
        }
    }
    None
}

/// Tasojen erot: puolustuksen paino, satunnaisuus ja huolimattomuus.
struct Tyyli {
    /// Vastustajan uhkan paino omaan hyökkäykseen nähden, tuhannesosina.
    puolustus: u64,
    /// Arvoa vähennetään satunnaisesti enintään näin monta tuhannesosaa.
    kohina: usize,
    /// Joka n:s siirto valitaan sattumalta viiden parhaan joukosta. Nolla: ei koskaan.
    huolimaton: usize,
}

impl Vaikeus {
    fn tyyli(self) -> Tyyli {
        match self {
            Vaikeus::Helppo => Tyyli {
                puolustus: 500,
                kohina: 500,
                huolimaton: 4,
            },
            Vaikeus::Normaali => Tyyli {
                puolustus: 900,
                kohina: 100,
                huolimaton: 0,
            },
            Vaikeus::Vaikea => Tyyli {
                puolustus: 950,
                kohina: 30,
                huolimaton: 0,
            },
        }
    }
}

/// Vuorossa olevan koneen siirto. `None` vain, kun peli on ohi.
pub fn koneen_siirto(vaikeus: Vaikeus, lauta: &Lauta, arpa: &mut Arpa) -> Option<Paikka> {
    if lauta.tila != Tila::Kesken {
        return None;
    }
    let mut r = lauta.ruudut;
    let oma = lauta.vuoro;
    let vastus = oma.vastustaja();
    let ehdokkaat = ehdokkaat(&r);
    if let Some(p) = viiden_ruutu(&r, &ehdokkaat, oma) {
        return Some(p);
    }
    let tyyli = vaikeus.tyyli();
    let mut arvot: Vec<(u64, Paikka)> = ehdokkaat
        .iter()
        .map(|&p| {
            let arvo = arvio(&r, p, oma) + arvio(&r, p, vastus) * tyyli.puolustus / 1000;
            let vahennys = if tyyli.kohina > 0 {
                arpa.alle(tyyli.kohina) as u64
            } else {
                0
            };
            // Ykkönen pitää tyhjänkin ruudun valittavana, ja kohina erottaa tasaväkiset.
            ((arvo + 1) * (1000 - vahennys), p)
        })
        .collect();
    arvot.sort_by_key(|&(arvo, _)| std::cmp::Reverse(arvo));
    if tyyli.huolimaton > 0 && arpa.alle(tyyli.huolimaton) == 0 {
        let i = arpa.alle(arvot.len().min(5));
        return Some(arvot[i].1);
    }
    if vaikeus == Vaikeus::Vaikea && viiden_ruutu(&r, &ehdokkaat, vastus).is_none() {
        if let Some(p) = uhkasarja(&mut r, oma, HAUN_SYVYYS, &mut { OMA_HAKU }, true) {
            return Some(p);
        }
        // Parhaista ehdokkaista ensimmäinen, joka ei jätä vastustajalle voittavaa uhkasarjaa.
        for &(_, p) in arvot.iter().take(TARKISTETTAVIA) {
            r[indeksi(p)] = Some(oma);
            let vaarallinen = uhkasarja(&mut r, vastus, HAUN_SYVYYS, &mut { VASTA_HAKU }, true);
            r[indeksi(p)] = None;
            if vaarallinen.is_none() {
                return Some(p);
            }
        }
    }
    arvot.first().map(|&(_, p)| p)
}
