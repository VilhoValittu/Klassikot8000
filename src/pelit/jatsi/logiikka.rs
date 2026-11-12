//! Jatsi pohjoismaisin säännöin: nopat, pöytäkirjat, pöydän vuorot ja tietokonepelaajan
//! päätökset. Ei piirtoa eikä syötettä.

use std::sync::OnceLock;

use crate::Arpa;

pub const NOPPIA: usize = 5;
/// Heittoja vuorossa enintään.
pub const HEITTOJA: u8 = 3;
pub const PELAAJIA: usize = 4;
pub const BONUSRAJA: u32 = 63;
pub const BONUS: u32 = 50;
/// Suurin mahdollinen pöytäkirjan summa: täydet rivit ja bonus.
pub const SUURIN_SUMMA: u32 = 374;

pub type Nopat = [u8; NOPPIA];
/// Silmälukujen määrät, indeksi on silmäluku 1..=6. Nolla jää käyttämättä.
type Laskut = [u8; 7];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Rivi {
    Ykkoset,
    Kakkoset,
    Kolmoset,
    Neloset,
    Vitoset,
    Kuutoset,
    Pari,
    KaksiParia,
    KolmeSamaa,
    NeljaSamaa,
    PieniSuora,
    SuuriSuora,
    Tayskasi,
    Sattuma,
    Jatsi,
}

pub const RIVEJA: usize = Rivi::KAIKKI.len();

impl Rivi {
    /// Pöytäkirjan järjestys ylhäältä alas.
    pub const KAIKKI: [Rivi; 15] = [
        Rivi::Ykkoset,
        Rivi::Kakkoset,
        Rivi::Kolmoset,
        Rivi::Neloset,
        Rivi::Vitoset,
        Rivi::Kuutoset,
        Rivi::Pari,
        Rivi::KaksiParia,
        Rivi::KolmeSamaa,
        Rivi::NeljaSamaa,
        Rivi::PieniSuora,
        Rivi::SuuriSuora,
        Rivi::Tayskasi,
        Rivi::Sattuma,
        Rivi::Jatsi,
    ];

    pub fn indeksi(self) -> usize {
        self as usize
    }

    /// Yläosan rivin silmäluku. Alaosan riveillä `None`.
    pub fn silmaluku(self) -> Option<u8> {
        (self.indeksi() < 6).then(|| self.indeksi() as u8 + 1)
    }
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

    /// Voitto koneista kertoo tuloksen tällä.
    pub fn kerroin(self) -> u32 {
        match self {
            Vaikeus::Helppo => 1,
            Vaikeus::Normaali => 2,
            Vaikeus::Vaikea => 3,
        }
    }
}

fn laskut(nopat: &Nopat) -> Laskut {
    let mut laskut = [0; 7];
    for &noppa in nopat {
        laskut[noppa as usize] += 1;
    }
    laskut
}

/// Rivin pisteet näillä nopilla. Ehdon täyttämätön rivi antaa nollan.
pub fn pisteet(rivi: Rivi, nopat: &Nopat) -> u32 {
    laskujen_pisteet(rivi, &laskut(nopat))
}

fn laskujen_pisteet(rivi: Rivi, laskut: &Laskut) -> u32 {
    let summa: u32 = (1..=6).map(|s| s as u32 * laskut[s] as u32).sum();
    // Suurin silmäluku, jota on vähintään `n`, ohittaen silmäluvun `paitsi`.
    let suurin = |n: u8, paitsi: usize| (1..=6).rev().find(|&s| s != paitsi && laskut[s] >= n);
    match rivi {
        Rivi::Pari => suurin(2, 0).map_or(0, |s| 2 * s as u32),
        Rivi::KaksiParia => match suurin(2, 0) {
            Some(a) => suurin(2, a).map_or(0, |b| 2 * (a + b) as u32),
            None => 0,
        },
        Rivi::KolmeSamaa => suurin(3, 0).map_or(0, |s| 3 * s as u32),
        Rivi::NeljaSamaa => suurin(4, 0).map_or(0, |s| 4 * s as u32),
        Rivi::PieniSuora if (1..=5).all(|s| laskut[s] == 1) => 15,
        Rivi::SuuriSuora if (2..=6).all(|s| laskut[s] == 1) => 20,
        // Täyskädessä on kaksi eri silmälukua, joten jatsi ei käy siksi.
        Rivi::Tayskasi => match (suurin(3, 0), suurin(2, 0)) {
            (Some(kolme), Some(_)) if suurin(2, kolme).is_some() => summa,
            _ => 0,
        },
        Rivi::Sattuma => summa,
        Rivi::Jatsi if suurin(5, 0).is_some() => 50,
        Rivi::PieniSuora | Rivi::SuuriSuora | Rivi::Jatsi => 0,
        _ => {
            let silmaluku = rivi.silmaluku().unwrap_or(0) as usize;
            silmaluku as u32 * laskut[silmaluku] as u32
        }
    }
}

/// Voiko rivillä olla nämä pisteet. Tallennuksen lukija hylkää muut.
pub fn mahdollinen(rivi: Rivi, arvo: u32) -> bool {
    (0..6usize.pow(NOPPIA as u32)).any(|mut n| {
        let mut nopat = [0; NOPPIA];
        for noppa in &mut nopat {
            *noppa = (n % 6) as u8 + 1;
            n /= 6;
        }
        pisteet(rivi, &nopat) == arvo
    })
}

/// Yhden pelaajan pöytäkirja. Kirjaamaton rivi on `None`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Poytakirja {
    rivit: [Option<u8>; RIVEJA],
}

impl Poytakirja {
    /// Tallennetut rivit. Mahdottomat pisteet hylkäävät koko kirjan.
    pub fn riveista(rivit: [Option<u8>; RIVEJA]) -> Option<Self> {
        let kelpaa = Rivi::KAIKKI
            .iter()
            .zip(rivit)
            .all(|(&rivi, arvo)| arvo.is_none_or(|arvo| mahdollinen(rivi, arvo as u32)));
        kelpaa.then_some(Self { rivit })
    }

    pub fn rivit(&self) -> [Option<u8>; RIVEJA] {
        self.rivit
    }

    pub fn arvo(&self, rivi: Rivi) -> Option<u32> {
        self.rivit[rivi.indeksi()].map(u32::from)
    }

    pub fn vapaa(&self, rivi: Rivi) -> bool {
        self.rivit[rivi.indeksi()].is_none()
    }

    pub fn vapaat(&self) -> impl Iterator<Item = Rivi> + '_ {
        Rivi::KAIKKI.into_iter().filter(|&rivi| self.vapaa(rivi))
    }

    pub fn kirjattuja(&self) -> usize {
        self.rivit.iter().filter(|arvo| arvo.is_some()).count()
    }

    pub fn taynna(&self) -> bool {
        self.kirjattuja() == RIVEJA
    }

    pub fn ylaosa(&self) -> u32 {
        Rivi::KAIKKI[..6].iter().filter_map(|&r| self.arvo(r)).sum()
    }

    pub fn bonus(&self) -> u32 {
        if self.ylaosa() >= BONUSRAJA { BONUS } else { 0 }
    }

    pub fn summa(&self) -> u32 {
        Rivi::KAIKKI
            .iter()
            .filter_map(|&r| self.arvo(r))
            .sum::<u32>()
            + self.bonus()
    }

    /// Kirjaa pisteet vapaalle riville. Palauttaa, kirjautuiko.
    pub fn kirjaa(&mut self, rivi: Rivi, pisteet: u32) -> bool {
        if !self.vapaa(rivi) {
            return false;
        }
        self.rivit[rivi.indeksi()] = Some(pisteet as u8);
        true
    }
}

/// Yhden kirjauksen tulos.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Kirjaus {
    pub pelaaja: usize,
    pub rivi: Rivi,
    pub pisteet: u32,
    /// Kirjaus nosti yläosan bonusrajan yli.
    pub bonus: bool,
}

/// Pöytä: pelaajien kirjat, vuorossa oleva pelaaja ja hänen noppansa.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Poyta {
    pub kirjat: Vec<Poytakirja>,
    /// Tosi koneen paikalla.
    pub koneet: Vec<bool>,
    pub vuorossa: usize,
    pub nopat: Nopat,
    pub lukitut: [bool; NOPPIA],
    /// Vuoron heitot tähän asti, 0..=`HEITTOJA`.
    pub heitetty: u8,
}

impl Poyta {
    /// Uusi peli. `koneet` kertoo paikkojen lajit, ja aloittaja on paikan indeksi.
    pub fn new(koneet: Vec<bool>, aloittaja: usize) -> Self {
        let pelaajia = koneet.len().clamp(1, PELAAJIA);
        Self {
            kirjat: vec![Poytakirja::default(); pelaajia],
            koneet: koneet.into_iter().take(pelaajia).collect(),
            vuorossa: aloittaja % pelaajia,
            nopat: [1; NOPPIA],
            lukitut: [false; NOPPIA],
            heitetty: 0,
        }
    }

    pub fn pelaajia(&self) -> usize {
        self.kirjat.len()
    }

    pub fn kone_vuorossa(&self) -> bool {
        self.koneet[self.vuorossa]
    }

    pub fn kirja(&self) -> &Poytakirja {
        &self.kirjat[self.vuorossa]
    }

    pub fn ohi(&self) -> bool {
        self.kirjat.iter().all(Poytakirja::taynna)
    }

    pub fn voi_heittaa(&self) -> bool {
        self.heitetty < HEITTOJA && !self.ohi()
    }

    /// Heittää lukitsemattomat nopat. Vuoron ensimmäinen heitto heittää kaikki.
    pub fn heita(&mut self, arpa: &mut Arpa) -> bool {
        if !self.voi_heittaa() {
            return false;
        }
        if self.heitetty == 0 {
            self.lukitut = [false; NOPPIA];
        }
        for (noppa, &lukittu) in self.nopat.iter_mut().zip(&self.lukitut) {
            if !lukittu {
                *noppa = arpa.alle(6) as u8 + 1;
            }
        }
        self.heitetty += 1;
        true
    }

    /// Lukitus vaihtuu vain heittojen välissä.
    pub fn voi_lukita(&self) -> bool {
        (1..HEITTOJA).contains(&self.heitetty)
    }

    pub fn lukitse(&mut self, noppa: usize) -> bool {
        if !self.voi_lukita() || noppa >= NOPPIA {
            return false;
        }
        self.lukitut[noppa] = !self.lukitut[noppa];
        true
    }

    pub fn voi_kirjata(&self, rivi: Rivi) -> bool {
        self.heitetty > 0 && self.kirja().vapaa(rivi)
    }

    /// Kirjaa vuorossa olevalle ja siirtää vuoron seuraavalle, jolla on vapaita rivejä.
    pub fn kirjaa(&mut self, rivi: Rivi) -> Option<Kirjaus> {
        if !self.voi_kirjata(rivi) {
            return None;
        }
        let pelaaja = self.vuorossa;
        let kirja = &mut self.kirjat[pelaaja];
        let bonus_ennen = kirja.bonus();
        let pisteet = pisteet(rivi, &self.nopat);
        kirja.kirjaa(rivi, pisteet);
        let bonus = kirja.bonus() > bonus_ennen;
        self.heitetty = 0;
        self.lukitut = [false; NOPPIA];
        if !self.ohi() {
            let n = self.pelaajia();
            self.vuorossa = (1..=n)
                .map(|askel| (pelaaja + askel) % n)
                .find(|&p| !self.kirjat[p].taynna())
                .unwrap_or(pelaaja);
        }
        Some(Kirjaus {
            pelaaja,
            rivi,
            pisteet,
            bonus,
        })
    }

    pub fn summat(&self) -> Vec<u32> {
        self.kirjat.iter().map(Poytakirja::summa).collect()
    }

    /// Suurimman summan pelaajat. Jaettu kärki on kaikkien voitto.
    pub fn voittajat(&self) -> Vec<usize> {
        let summat = self.summat();
        let paras = summat.iter().copied().max().unwrap_or(0);
        (0..summat.len()).filter(|&p| summat[p] == paras).collect()
    }

    /// Pelaajan sija 1..: montako on saanut enemmän, plus yksi.
    pub fn sija(&self, pelaaja: usize) -> usize {
        let summat = self.summat();
        summat.iter().filter(|&&s| s > summat[pelaaja]).count() + 1
    }
}

/// Koneen seuraava askel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Toimi {
    Heita,
    /// Lukitsee nämä nopat ennen seuraavaa heittoa.
    Lukitse([bool; NOPPIA]),
    Kirjaa(Rivi),
}

/// Mitä kone tekee seuraavaksi. Kone näkee vain oman kirjansa ja nopat.
pub fn koneen_toimi(vaikeus: Vaikeus, poyta: &Poyta) -> Toimi {
    if poyta.heitetty == 0 {
        return Toimi::Heita;
    }
    let kirja = poyta.kirja();
    let nopat = &poyta.nopat;
    let pito = if poyta.heitetty < HEITTOJA {
        let jaljella = HEITTOJA - poyta.heitetty;
        match vaikeus {
            Vaikeus::Helppo => ahne_pito(kirja, nopat),
            Vaikeus::Normaali => Laskin::new(kirja, false).pito(nopat, 1),
            Vaikeus::Vaikea => Laskin::new(kirja, true).pito(nopat, jaljella),
        }
    } else {
        None
    };
    match pito {
        Some(lukitut) if lukitut != poyta.lukitut => Toimi::Lukitse(lukitut),
        Some(_) => Toimi::Heita,
        None => Toimi::Kirjaa(koneen_rivi(vaikeus, kirja, nopat)),
    }
}

/// Rivi, jolle kone kirjaa nämä nopat.
pub fn koneen_rivi(vaikeus: Vaikeus, kirja: &Poytakirja, nopat: &Nopat) -> Rivi {
    let laskut = laskut(nopat);
    let arvo = |rivi: Rivi| match vaikeus {
        Vaikeus::Helppo => laskujen_pisteet(rivi, &laskut) as f32,
        Vaikeus::Normaali => laskujen_pisteet(rivi, &laskut) as f32 - ODOTUS[rivi.indeksi()],
        Vaikeus::Vaikea => rivin_arvo(kirja, rivi, &laskut),
    };
    let mut paras = None;
    for rivi in kirja.vapaat() {
        let a = arvo(rivi);
        if paras.is_none_or(|(_, b)| a > b) {
            paras = Some((rivi, a));
        }
    }
    paras.map_or(Rivi::Sattuma, |(rivi, _)| rivi)
}

/// Helppo kone pitää yleisimmän silmäluvun, ja tasatilanteessa suuremman. Valmis suora,
/// täyskäsi tai jatsi vapaalle riville kelpaa sellaisenaan. `None` lopettaa heittämisen.
fn ahne_pito(kirja: &Poytakirja, nopat: &Nopat) -> Option<[bool; NOPPIA]> {
    let laskut = laskut(nopat);
    let valmis = [
        Rivi::Jatsi,
        Rivi::SuuriSuora,
        Rivi::PieniSuora,
        Rivi::Tayskasi,
    ]
    .into_iter()
    .any(|rivi| kirja.vapaa(rivi) && laskujen_pisteet(rivi, &laskut) > 0);
    if valmis || laskut[1..].contains(&5) {
        return None;
    }
    let yleisin = (1..=6).rev().max_by_key(|&s| laskut[s]).unwrap_or(6);
    Some(nopat.map(|noppa| noppa as usize == yleisin))
}

/// Rivin keskimääräinen arvo, jos se jää myöhemmäksi. Kirjaus vertaa pisteitä tähän:
/// halvin rivi viivataan ensin, ja hyvä tulos menee riville, jolla se on harvinainen.
const ODOTUS: [f32; RIVEJA] = [
    2.0, 5.0, 8.0, 11.0, 14.0, 17.0, 9.0, 14.0, 10.0, 6.0, 6.0, 8.0, 13.0, 21.0, 12.0,
];
/// Yläosan bonuksen todennäköisyysarvion jyrkkyys pisteinä.
const BONUKSEN_LIUKU: f32 = 5.0;

/// Arvio bonuksen todennäköisyydestä: kirjaamattomat yläosan rivit tuovat keskimäärin
/// kolme noppaa kukin.
fn bonuksen_mahdollisuus(ylaosa: u32, vapaat_silmaluvut: u32) -> f32 {
    if ylaosa >= BONUSRAJA {
        return 1.0;
    }
    if vapaat_silmaluvut == 0 {
        return 0.0;
    }
    let ero = (ylaosa + 3 * vapaat_silmaluvut) as f32 - BONUSRAJA as f32;
    1.0 / (1.0 + (-ero / BONUKSEN_LIUKU).exp())
}

/// Kirjauksen arvo: pisteet miinus rivin odotus, ja yläosassa muutos bonuksen arviossa.
fn rivin_arvo(kirja: &Poytakirja, rivi: Rivi, laskut: &Laskut) -> f32 {
    let pisteet = laskujen_pisteet(rivi, laskut);
    let mut arvo = pisteet as f32 - ODOTUS[rivi.indeksi()];
    if let Some(silmaluku) = rivi.silmaluku() {
        let ylaosa = kirja.ylaosa();
        let vapaat: u32 = Rivi::KAIKKI[..6]
            .iter()
            .filter(|&&r| kirja.vapaa(r))
            .filter_map(|r| r.silmaluku())
            .map(u32::from)
            .sum();
        let ennen = bonuksen_mahdollisuus(ylaosa, vapaat);
        let jalkeen = bonuksen_mahdollisuus(ylaosa + pisteet, vapaat - silmaluku as u32);
        arvo += BONUS as f32 * (jalkeen - ennen);
    }
    arvo
}

/// Kaikkien enintään viiden nopan yhdistelmien yhteiset taulukot, laskettu kerran.
struct Taulut {
    /// Heittojen tulokset noppien määrän mukaan: (määrät, todennäköisyys).
    heitot: [Vec<(Laskut, f32)>; NOPPIA + 1],
    /// Määrien avaimesta tiivis indeksi välimuisteihin.
    indeksit: Vec<u16>,
    yhdistelmia: usize,
}

/// Noppien määrät avaimeksi: kantaluku 6 riittää, koska kutakin silmälukua on enintään viisi.
fn avain(laskut: &Laskut) -> usize {
    (1..=6).fold(0, |a, s| a * 6 + laskut[s] as usize)
}

const AVAIMIA: usize = 6 * 6 * 6 * 6 * 6 * 6;

fn taulut() -> &'static Taulut {
    static TAULUT: OnceLock<Taulut> = OnceLock::new();
    TAULUT.get_or_init(|| {
        let heitot = std::array::from_fn(heittojen_tulokset);
        let mut indeksit = vec![u16::MAX; AVAIMIA];
        let mut yhdistelmia = 0;
        for tulokset in &heitot {
            for (laskut, _) in tulokset {
                indeksit[avain(laskut)] = yhdistelmia as u16;
                yhdistelmia += 1;
            }
        }
        Taulut {
            heitot,
            indeksit,
            yhdistelmia,
        }
    })
}

/// Odotusarvolaskin: paras pito, kun heittoja on vielä jäljellä. Välimuistit kattavat kaikki
/// noppien yhdistelmät ja pidot, joita on yhteensä vain 462.
struct Laskin<'a> {
    kirja: &'a Poytakirja,
    /// Arvioiko kirjaus yläosan bonuksen.
    bonus: bool,
    taulut: &'static Taulut,
    /// Indeksillä ja jäljellä olevilla heitoilla: viiden nopan paras arvo, tai pidon odotusarvo.
    arvot: [Vec<f32>; HEITTOJA as usize],
    odotukset: [Vec<f32>; HEITTOJA as usize],
}

impl<'a> Laskin<'a> {
    fn new(kirja: &'a Poytakirja, bonus: bool) -> Self {
        let taulut = taulut();
        let n = taulut.yhdistelmia;
        Self {
            kirja,
            bonus,
            taulut,
            arvot: std::array::from_fn(|_| vec![f32::NAN; n]),
            odotukset: std::array::from_fn(|_| vec![f32::NAN; n]),
        }
    }

    fn indeksi(&self, laskut: &Laskut) -> usize {
        self.taulut.indeksit[avain(laskut)] as usize
    }

    /// Viiden nopan arvo, kun heittoja on `jaljella`. Nollalla kirjataan heti.
    fn arvo(&mut self, laskut: &Laskut, jaljella: u8) -> f32 {
        let k = self.indeksi(laskut);
        let muisti = self.arvot[jaljella as usize][k];
        if !muisti.is_nan() {
            return muisti;
        }
        let arvo = if jaljella == 0 {
            let kirja = self.kirja;
            kirja
                .vapaat()
                .map(|rivi| {
                    if self.bonus {
                        rivin_arvo(kirja, rivi, laskut)
                    } else {
                        laskujen_pisteet(rivi, laskut) as f32 - ODOTUS[rivi.indeksi()]
                    }
                })
                .fold(f32::MIN, f32::max)
        } else {
            let mut paras = f32::MIN;
            for pito in alijoukot(laskut) {
                paras = paras.max(self.odotus(&pito, jaljella - 1));
            }
            paras
        };
        self.arvot[jaljella as usize][k] = arvo;
        arvo
    }

    /// Pidon odotusarvo, kun loput nopat heitetään ja sen jälkeen heittoja on `jaljella`.
    fn odotus(&mut self, pito: &Laskut, jaljella: u8) -> f32 {
        let k = self.indeksi(pito);
        let muisti = self.odotukset[jaljella as usize][k];
        if !muisti.is_nan() {
            return muisti;
        }
        let pidetty: usize = pito[1..].iter().map(|&n| n as usize).sum();
        let taulut = self.taulut;
        let mut summa = 0.0;
        for (tulos, osuus) in &taulut.heitot[NOPPIA - pidetty] {
            let mut laskut = *pito;
            for s in 1..=6 {
                laskut[s] += tulos[s];
            }
            summa += osuus * self.arvo(&laskut, jaljella);
        }
        self.odotukset[jaljella as usize][k] = summa;
        summa
    }

    /// Paras pito, kun heittoja on `jaljella` (1 tai 2). `None`, kun kaikki kannattaa pitää.
    fn pito(&mut self, nopat: &Nopat, jaljella: u8) -> Option<[bool; NOPPIA]> {
        let laskut = laskut(nopat);
        let mut paras = (laskut, self.arvo(&laskut, 0));
        for pito in alijoukot(&laskut) {
            let arvo = self.odotus(&pito, jaljella - 1);
            // Pieni etu nykyiselle pidolle, jottei pyöristys heitä valmista käsiä.
            if arvo > paras.1 + 1e-4 {
                paras = (pito, arvo);
            }
        }
        if paras.0 == laskut {
            return None;
        }
        let mut jaljella_pitoon = paras.0;
        Some(nopat.map(|noppa| {
            let maara = &mut jaljella_pitoon[noppa as usize];
            let pidetaan = *maara > 0;
            if pidetaan {
                *maara -= 1;
            }
            pidetaan
        }))
    }
}

/// Kaikki tavat pitää osa nopista määrinä, myös tyhjä ja kaikki.
fn alijoukot(laskut: &Laskut) -> Vec<Laskut> {
    let mut joukot = vec![[0; 7]];
    for s in 1..=6 {
        let mut uudet = Vec::new();
        for joukko in &joukot {
            for n in 0..=laskut[s] {
                let mut uusi = *joukko;
                uusi[s] = n;
                uudet.push(uusi);
            }
        }
        joukot = uudet;
    }
    joukot
}

/// `n` nopan heiton kaikki tulokset määrinä todennäköisyyksineen.
fn heittojen_tulokset(n: usize) -> Vec<(Laskut, f32)> {
    let kaikkia = 6usize.pow(n as u32);
    let mut tulokset: Vec<(Laskut, f32)> = Vec::new();
    for mut luku in 0..kaikkia {
        let mut laskut = [0; 7];
        for _ in 0..n {
            laskut[luku % 6 + 1] += 1;
            luku /= 6;
        }
        match tulokset.iter_mut().find(|(l, _)| *l == laskut) {
            Some((_, osuus)) => *osuus += 1.0,
            None => tulokset.push((laskut, 1.0)),
        }
    }
    for (_, osuus) in &mut tulokset {
        *osuus /= kaikkia as f32;
    }
    tulokset
}
