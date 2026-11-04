//! Mahjonggin (laattapasianssin) säännöt ilman piirtoa. Laatat ovat kerroksittain pohjalla, ja
//! pois saa ottaa kaksi samaa vapaata laattaa. Jako tehdään purkamalla pohja satunnaisina
//! vapaina pareina, joten jokainen jako on ratkaistavissa.

use crate::Arpa;

/// Erilaisia laattoja: kolme numerosarjaa, tuulet, lohikäärmeet, kukat ja vuodenajat.
pub const LAJEJA: u8 = 42;
/// Täydessä pelissä on 144 laattaa eli 72 paria.
pub const PAREJA_ENINTAAN: usize = 72;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sarja {
    Pallot,
    Bambut,
    Merkit,
    Tuulet,
    Lohikaarmeet,
    Kukat,
    Vuodenajat,
}

/// Laatan laji 0..42: pallot 0-8, bambut 9-17, merkit 18-26, tuulet 27-30, lohikäärmeet
/// 31-33, kukat 34-37 ja vuodenajat 38-41.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Laatta(pub u8);

impl Laatta {
    pub fn sarja(self) -> Sarja {
        match self.0 {
            0..=8 => Sarja::Pallot,
            9..=17 => Sarja::Bambut,
            18..=26 => Sarja::Merkit,
            27..=30 => Sarja::Tuulet,
            31..=33 => Sarja::Lohikaarmeet,
            34..=37 => Sarja::Kukat,
            _ => Sarja::Vuodenajat,
        }
    }

    /// Järjestys sarjan sisällä, ensimmäinen on 1.
    pub fn arvo(self) -> u8 {
        let alku = match self.sarja() {
            Sarja::Pallot => 0,
            Sarja::Bambut => 9,
            Sarja::Merkit => 18,
            Sarja::Tuulet => 27,
            Sarja::Lohikaarmeet => 31,
            Sarja::Kukat => 34,
            Sarja::Vuodenajat => 38,
        };
        self.0 - alku + 1
    }

    /// Kukat ja vuodenajat ovat kukin ainoita lajissaan, mutta käyvät pariksi sarjansa kanssa.
    pub fn bonus(self) -> bool {
        matches!(self.sarja(), Sarja::Kukat | Sarja::Vuodenajat)
    }

    fn luokka(self) -> u8 {
        match self.sarja() {
            Sarja::Kukat => 34,
            Sarja::Vuodenajat => 38,
            _ => self.0,
        }
    }

    pub fn sopii(self, toinen: Laatta) -> bool {
        self.luokka() == toinen.luokka()
    }
}

/// Kaikki 72 paria: tavallisesta lajista kaksi paria, kukista ja vuodenajoista kaksi kustakin sarjasta.
fn kaikki_parit() -> Vec<(Laatta, Laatta)> {
    let mut parit = Vec::with_capacity(PAREJA_ENINTAAN);
    for laji in 0..34 {
        for _ in 0..2 {
            parit.push((Laatta(laji), Laatta(laji)));
        }
    }
    for alku in [34, 36, 38, 40] {
        parit.push((Laatta(alku), Laatta(alku + 1)));
    }
    parit
}

/// Laatan paikka puolen laatan ruudukossa: laatta peittää 2x2 ruutua kerroksessa `z`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Paikka {
    pub x: i8,
    pub y: i8,
    pub z: u8,
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

    /// Pisteiden kerroin: isompi ja korkeampi pohja on työläämpi purkaa.
    pub fn kerroin(self) -> u32 {
        match self {
            Vaikeus::Helppo => 1,
            Vaikeus::Normaali => 2,
            Vaikeus::Vaikea => 3,
        }
    }

    pub fn sekoituksia(self) -> u32 {
        match self {
            Vaikeus::Helppo => 3,
            Vaikeus::Normaali => 2,
            Vaikeus::Vaikea => 1,
        }
    }

    /// Vaikealla tasolla siirtoa ei voi perua.
    pub fn voi_perua(self) -> bool {
        self != Vaikeus::Vaikea
    }

    /// Helpolla tasolla vapaat laatat erottuvat muista.
    pub fn nayta_vapaat(self) -> bool {
        self == Vaikeus::Helppo
    }

    /// Aikabonus kertyy, kunnes tämä aika on kulunut.
    pub fn aikaraja_s(self) -> u32 {
        match self {
            Vaikeus::Helppo => 300,
            Vaikeus::Normaali => 600,
            Vaikeus::Vaikea => 900,
        }
    }

    /// Pohjan laattojen paikat piirtojärjestyksessä: kerros, rivi ja sarake kasvavat.
    pub fn pohja(self) -> Vec<Paikka> {
        let mut paikat = Vec::new();
        // (kerros, ensimmäisen rivin y, rivien pituudet laattoina, keskikohta x)
        let rivit: &[(u8, i8, &[i8], i8)] = match self {
            // Matala pyramidi, 72 laattaa.
            Vaikeus::Helppo => &[
                (0, 0, &[8, 8, 8, 8, 8, 8], 8),
                (1, 3, &[6, 6, 6], 8),
                (2, 4, &[3, 3], 8),
            ],
            // Linnoitus siipineen ja tornineen, 124 laattaa.
            Vaikeus::Normaali => &[
                (0, 0, &[12, 12, 12, 12, 12, 12], 12),
                (1, 2, &[10, 10, 10, 10], 12),
                (2, 4, &[4, 4], 12),
                (3, 5, &[2], 12),
            ],
            // Korkea vuori, kaikki 144 laattaa.
            Vaikeus::Vaikea => &[
                (0, 0, &[8, 10, 12, 12, 12, 12, 10, 8], 12),
                (1, 2, &[4, 8, 8, 8, 8, 4], 12),
                (2, 4, &[4, 4, 4, 4], 12),
                (3, 6, &[2, 2], 12),
            ],
        };
        for &(z, y0, pituudet, keski) in rivit {
            for (i, &pituus) in pituudet.iter().enumerate() {
                let y = y0 + 2 * i as i8;
                for s in 0..pituus {
                    paikat.push(Paikka {
                        x: keski - pituus + 2 * s,
                        y,
                        z,
                    });
                }
            }
        }
        if self == Vaikeus::Normaali {
            // Siivet pohjakerroksen päissä puolen laatan korkeudella.
            paikat.insert(0, Paikka { x: -2, y: 5, z: 0 });
            paikat.insert(1, Paikka { x: 24, y: 5, z: 0 });
        }
        paikat
    }
}

pub const PARIN_PISTEET: u32 = 10;
pub const BONUSPARIN_PISTEET: u32 = 20;
/// Pari, joka löytyy tämän ajan sisällä edellisestä, kasvattaa ketjua.
pub const KETJU_MS: u32 = 4000;
pub const SUURIN_KETJU: u32 = 5;
pub const VIHJEEN_HINTA: u32 = 20;
pub const SEKOITUKSEN_HINTA: u32 = 50;
pub const TYHJENNYSBONUS: u32 = 500;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tila {
    Kesken,
    /// Vapaita pareja ei ole, mutta sekoituksia on jäljellä.
    Jumissa,
    Voitto,
    /// Vapaita pareja ei ole eikä sekoituksia.
    Tappio,
}

/// Otettu pari peruutusta varten.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Poisto {
    a: usize,
    b: usize,
    laatat: (Laatta, Laatta),
    pisteet: u32,
}

/// Voiton lisäpisteet eriteltyinä.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Loppubonus {
    pub tyhjennys: u32,
    pub aika: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lauta {
    pub vaikeus: Vaikeus,
    pub paikat: Vec<Paikka>,
    /// `None`, kun laatta on otettu pois. Sama järjestys kuin `paikat`.
    pub laatat: Vec<Option<Laatta>>,
    pub tila: Tila,
    pub pisteet: u32,
    /// Peräkkäisten nopeiden parien määrä, 0 kun ketjua ei ole.
    pub ketju: u32,
    pub sekoituksia: u32,
    pub vihjeita: u32,
    pub bonus: Loppubonus,
    /// Jaon tehneet poistot järjestyksessä: niillä lauta ratkeaa varmasti.
    pub ratkaisu: Vec<(usize, usize)>,
    edellinen_pari_ms: Option<u32>,
    historia: Vec<Poisto>,
}

/// Purkuyrityksiä ennen kuin jako tehdään umpimähkään. Täysi pohja purkautuu lähes aina
/// ensimmäisellä, mutta pelin lopun jäänteet (kaksi laattaa päällekkäin) eivät koskaan.
const JAKOYRITYKSIA: usize = 300;

/// Jakaa parit paikoille `kaytossa` purkamalla pohjan vapaina pareina. Palauttaa jaon ja
/// poistojärjestyksen. Jumiin päätynyt purku aloitetaan alusta, ja mahdoton jäänne jaetaan
/// satunnaisesti ilman ratkaisua.
fn jaa_ratkeava(
    paikat: &[Paikka],
    kaytossa: &[bool],
    parit: &[(Laatta, Laatta)],
    arpa: &mut Arpa,
) -> (Vec<Option<Laatta>>, Vec<(usize, usize)>) {
    for _ in 0..JAKOYRITYKSIA {
        let mut jaljella = kaytossa.to_vec();
        let mut laatat = vec![None; paikat.len()];
        let mut ratkaisu = Vec::with_capacity(parit.len());
        let mut parit = parit.to_vec();
        sekoita(&mut parit, arpa);
        let mut onnistui = true;
        for &(a, b) in &parit {
            let mut vapaat: Vec<usize> = (0..paikat.len())
                .filter(|&i| jaljella[i] && on_vapaa(paikat, &jaljella, i))
                .collect();
            if vapaat.len() < 2 {
                onnistui = false;
                break;
            }
            let i = vapaat.swap_remove(arpa.alle(vapaat.len()));
            let j = vapaat.swap_remove(arpa.alle(vapaat.len()));
            laatat[i] = Some(a);
            laatat[j] = Some(b);
            jaljella[i] = false;
            jaljella[j] = false;
            ratkaisu.push((i, j));
        }
        if onnistui {
            return (laatat, ratkaisu);
        }
    }
    let mut paikat_kaytossa: Vec<usize> = (0..paikat.len()).filter(|&i| kaytossa[i]).collect();
    sekoita(&mut paikat_kaytossa, arpa);
    let mut laatat = vec![None; paikat.len()];
    let jarjestys = parit.iter().flat_map(|&(a, b)| [a, b]);
    for (i, laatta) in paikat_kaytossa.into_iter().zip(jarjestys) {
        laatat[i] = Some(laatta);
    }
    (laatat, Vec::new())
}

fn sekoita<T>(lista: &mut [T], arpa: &mut Arpa) {
    for i in (1..lista.len()).rev() {
        lista.swap(i, arpa.alle(i + 1));
    }
}

/// Laatta on vapaa, kun mikään ei ole sen päällä ja ainakin toinen sivu on auki.
fn on_vapaa(paikat: &[Paikka], jaljella: &[bool], i: usize) -> bool {
    let p = paikat[i];
    let mut vasen = false;
    let mut oikea = false;
    for (j, q) in paikat.iter().enumerate() {
        if j == i || !jaljella[j] {
            continue;
        }
        let (dx, dy) = (q.x - p.x, q.y - p.y);
        if dy.abs() >= 2 {
            continue;
        }
        if q.z > p.z && dx.abs() < 2 {
            return false;
        }
        if q.z == p.z {
            vasen |= dx == -2;
            oikea |= dx == 2;
        }
    }
    !(vasen && oikea)
}

impl Lauta {
    pub fn new(vaikeus: Vaikeus, arpa: &mut Arpa) -> Self {
        let paikat = vaikeus.pohja();
        let mut parit = kaikki_parit();
        sekoita(&mut parit, arpa);
        parit.truncate(paikat.len() / 2);
        let kaytossa = vec![true; paikat.len()];
        let (laatat, ratkaisu) = jaa_ratkeava(&paikat, &kaytossa, &parit, arpa);
        Self {
            vaikeus,
            paikat,
            laatat,
            tila: Tila::Kesken,
            pisteet: 0,
            ketju: 0,
            sekoituksia: vaikeus.sekoituksia(),
            vihjeita: 0,
            bonus: Loppubonus::default(),
            ratkaisu,
            edellinen_pari_ms: None,
            historia: Vec::new(),
        }
    }

    /// Lauta valmiilla laatoilla, kokeita varten. Tila lasketaan laatoista.
    pub fn laatoista(vaikeus: Vaikeus, paikat: Vec<Paikka>, laatat: Vec<Option<Laatta>>) -> Self {
        let mut lauta = Self {
            vaikeus,
            paikat,
            laatat,
            tila: Tila::Kesken,
            pisteet: 0,
            ketju: 0,
            sekoituksia: vaikeus.sekoituksia(),
            vihjeita: 0,
            bonus: Loppubonus::default(),
            ratkaisu: Vec::new(),
            edellinen_pari_ms: None,
            historia: Vec::new(),
        };
        lauta.paivita_tila();
        lauta
    }

    fn jaljella_kartta(&self) -> Vec<bool> {
        self.laatat.iter().map(Option::is_some).collect()
    }

    pub fn jaljella(&self) -> usize {
        self.laatat.iter().flatten().count()
    }

    pub fn paattynyt(&self) -> bool {
        matches!(self.tila, Tila::Voitto | Tila::Tappio)
    }

    pub fn vapaa(&self, i: usize) -> bool {
        self.laatat.get(i).is_some_and(Option::is_some)
            && on_vapaa(&self.paikat, &self.jaljella_kartta(), i)
    }

    /// Jokaisen paikan vapaus kerralla: piirto tarvitsee kaikki, ja yksi kartta riittää.
    pub fn vapaat(&self) -> Vec<bool> {
        let jaljella = self.jaljella_kartta();
        (0..self.paikat.len())
            .map(|i| jaljella[i] && on_vapaa(&self.paikat, &jaljella, i))
            .collect()
    }

    /// Kaikki vapaat parit, joita voi ottaa juuri nyt.
    pub fn vapaat_parit(&self) -> Vec<(usize, usize)> {
        let vapaat = self.vapaat();
        let valittavat: Vec<(usize, Laatta)> = self
            .laatat
            .iter()
            .enumerate()
            .filter_map(|(i, laatta)| laatta.filter(|_| vapaat[i]).map(|l| (i, l)))
            .collect();
        let mut parit = Vec::new();
        for (k, &(i, a)) in valittavat.iter().enumerate() {
            for &(j, b) in &valittavat[k + 1..] {
                if a.sopii(b) {
                    parit.push((i, j));
                }
            }
        }
        parit
    }

    /// Voiko parin ottaa.
    pub fn kelpaa(&self, a: usize, b: usize) -> bool {
        a != b
            && !self.paattynyt()
            && matches!((self.laatat.get(a), self.laatat.get(b)), (Some(Some(x)), Some(Some(y))) if x.sopii(*y))
            && self.vapaa(a)
            && self.vapaa(b)
    }

    /// Ottaa parin pois hetkellä `aika_ms` pelin alusta. Palauttaa parista saadut pisteet.
    pub fn poista(&mut self, a: usize, b: usize, aika_ms: u32) -> Option<u32> {
        if !self.kelpaa(a, b) {
            return None;
        }
        let laatat = (self.laatat[a]?, self.laatat[b]?);
        let nopea = self
            .edellinen_pari_ms
            .is_some_and(|ennen| aika_ms.saturating_sub(ennen) <= KETJU_MS);
        self.ketju = if nopea {
            (self.ketju + 1).min(SUURIN_KETJU)
        } else {
            1
        };
        self.edellinen_pari_ms = Some(aika_ms);
        let perus = if laatat.0.bonus() {
            BONUSPARIN_PISTEET
        } else {
            PARIN_PISTEET
        };
        let pisteet = perus * self.ketju * self.vaikeus.kerroin();
        self.pisteet += pisteet;
        self.laatat[a] = None;
        self.laatat[b] = None;
        self.historia.push(Poisto {
            a,
            b,
            laatat,
            pisteet,
        });
        self.paivita_tila();
        if self.tila == Tila::Voitto {
            let kerroin = self.vaikeus.kerroin();
            let jaljella_s = self.vaikeus.aikaraja_s().saturating_sub(aika_ms / 1000);
            self.bonus = Loppubonus {
                tyhjennys: TYHJENNYSBONUS * kerroin,
                aika: jaljella_s * kerroin,
            };
            self.pisteet += self.bonus.tyhjennys + self.bonus.aika;
        }
        Some(pisteet)
    }

    fn paivita_tila(&mut self) {
        self.tila = if self.jaljella() == 0 {
            Tila::Voitto
        } else if !self.vapaat_parit().is_empty() {
            Tila::Kesken
        } else if self.sekoituksia > 0 {
            Tila::Jumissa
        } else {
            Tila::Tappio
        };
    }

    fn katkaise_ketju(&mut self) {
        self.ketju = 0;
        self.edellinen_pari_ms = None;
    }

    pub fn voi_perua(&self) -> bool {
        self.vaikeus.voi_perua() && !self.paattynyt() && !self.historia.is_empty()
    }

    /// Palauttaa viimeksi otetun parin ja vähentää sen pisteet.
    pub fn peru(&mut self) -> bool {
        if !self.voi_perua() {
            return false;
        }
        let Some(poisto) = self.historia.pop() else {
            return false;
        };
        self.laatat[poisto.a] = Some(poisto.laatat.0);
        self.laatat[poisto.b] = Some(poisto.laatat.1);
        self.pisteet = self.pisteet.saturating_sub(poisto.pisteet);
        self.katkaise_ketju();
        self.paivita_tila();
        true
    }

    /// Vapaa pari maksua vastaan, tai `None`, jos pareja ei ole.
    pub fn vihje(&mut self) -> Option<(usize, usize)> {
        if self.paattynyt() {
            return None;
        }
        let pari = self.vapaat_parit().first().copied()?;
        self.pisteet = self.pisteet.saturating_sub(VIHJEEN_HINTA);
        self.vihjeita += 1;
        self.katkaise_ketju();
        Some(pari)
    }

    /// Jakaa jäljellä olevat laatat uudelleen niin, että ne ratkeavat. Peruutushistoria
    /// nollautuu, koska vanhat parit eivät enää ole paikoillaan.
    pub fn sekoita(&mut self, arpa: &mut Arpa) -> bool {
        if self.paattynyt() || self.sekoituksia == 0 {
            return false;
        }
        let mut luokittain: Vec<Vec<Laatta>> = vec![Vec::new(); LAJEJA as usize];
        for laatta in self.laatat.iter().flatten() {
            luokittain[laatta.luokka() as usize].push(*laatta);
        }
        let parit: Vec<(Laatta, Laatta)> = luokittain
            .iter()
            .flat_map(|laatat| laatat.chunks_exact(2).map(|pari| (pari[0], pari[1])))
            .collect();
        let kaytossa = self.jaljella_kartta();
        let (laatat, ratkaisu) = jaa_ratkeava(&self.paikat, &kaytossa, &parit, arpa);
        self.laatat = laatat;
        self.ratkaisu = ratkaisu;
        self.sekoituksia -= 1;
        self.pisteet = self.pisteet.saturating_sub(SEKOITUKSEN_HINTA);
        self.historia.clear();
        self.katkaise_ketju();
        self.paivita_tila();
        true
    }
}

/// Suurin mahdollinen tulos: jokainen pari täydellä ketjulla ja voitto heti. Palvelimen raja.
pub fn suurin_tulos() -> u32 {
    Vaikeus::KAIKKI
        .iter()
        .map(|&taso| {
            let parit = taso.pohja().len() as u32 / 2;
            let kerroin = taso.kerroin();
            parit * BONUSPARIN_PISTEET * SUURIN_KETJU * kerroin
                + (TYHJENNYSBONUS + taso.aikaraja_s()) * kerroin
        })
        .max()
        .unwrap_or(0)
}
