//! Texas Hold'em ilman ikkunaa, piirtoa tai syötettä. No limit -säännöt noudattavat Poker
//! TDA:n turnaussääntöjä: kuollut nappi (34, 36-B), kaksinpeli (36-C), näyttöjärjestys (18-A),
//! pienin korotus (45-A), vajaa all-in ei avaa panostusta uudelleen (49-A), blindit lasketaan
//! täysinä (56-B), sivupotit jaetaan erikseen (23) ja pariton merkki napista vasemmalle (21).

use crate::Arpa;
use crate::kortit::{KUNINGAS, sekoitettu};
pub use crate::kortit::{Kortti, Maa};

pub const PELAAJIA: usize = 5;
/// Ihmisen paikka. Muut paikat ovat tietokoneen.
pub const PELAAJA: usize = 0;
pub const ALKURAHA: u32 = 1000;
pub const PIENI_ALUSSA: u32 = 10;
pub const ISO_ALUSSA: u32 = 20;
/// Blindit tuplaantuvat näin monen jaon välein, joten peli päättyy aina.
pub const TASON_JAKOJA: u32 = 10;
const SUURIN_TASO: u32 = 16;

const ASSA: u8 = KUNINGAS + 1;
const KATEEN: usize = 5;
const YHTEISIA: usize = 5;

/// Pienimmästä suurimpaan: järjestys ratkaisee käden luokan.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Luokka {
    Hai,
    Pari,
    KaksiParia,
    Kolmoset,
    Suora,
    Vari,
    Tayskasi,
    Neloset,
    Varisuora,
}

/// Luokan jälkeen käsiä verrataan arvoilla vasemmalta oikealle. Maa ei ratkaise koskaan.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Kadenarvo {
    pub luokka: Luokka,
    pub arvot: [u8; KATEEN],
}

impl Kadenarvo {
    pub fn on_kuningasvarisuora(&self) -> bool {
        self.luokka == Luokka::Varisuora && self.arvot[0] == ASSA
    }
}

fn arvo_ylos(kortti: Kortti) -> u8 {
    if kortti.arvo == 1 { ASSA } else { kortti.arvo }
}

fn viiden_arvo(kortit: &[Kortti; KATEEN]) -> Kadenarvo {
    let mut maarat = [0u8; ASSA as usize + 1];
    for &kortti in kortit {
        maarat[arvo_ylos(kortti) as usize] += 1;
    }
    // Isoin ryhmä ensin, saman kokoisista korkein ensin: neloset ennen potkuria.
    let mut arvot = [0u8; KATEEN];
    let mut ryhmat = [0u8; KATEEN];
    let mut ryhmia = 0;
    for koko in (1..=4).rev() {
        for arvo in (2..=ASSA).rev() {
            if maarat[arvo as usize] == koko {
                arvot[ryhmia] = arvo;
                ryhmat[ryhmia] = koko;
                ryhmia += 1;
            }
        }
    }
    let vari = kortit.iter().all(|k| k.maa == kortit[0].maa);
    let suoran_karki = if ryhmia < KATEEN {
        None
    } else if arvot[0] - arvot[4] == 4 {
        Some(arvot[0])
    } else if arvot == [ASSA, 5, 4, 3, 2] {
        // Ässä käy suoran alapäähän, ja silloin viitonen on suoran kärki.
        Some(5)
    } else {
        None
    };
    let luokka = match (ryhmat[0], ryhmat[1], suoran_karki, vari) {
        (_, _, Some(_), true) => Luokka::Varisuora,
        (4, ..) => Luokka::Neloset,
        (3, 2, ..) => Luokka::Tayskasi,
        (_, _, None, true) => Luokka::Vari,
        (_, _, Some(_), false) => Luokka::Suora,
        (3, ..) => Luokka::Kolmoset,
        (2, 2, ..) => Luokka::KaksiParia,
        (2, ..) => Luokka::Pari,
        _ => Luokka::Hai,
    };
    if let Some(karki) = suoran_karki {
        arvot = [karki, 0, 0, 0, 0];
    }
    Kadenarvo { luokka, arvot }
}

/// Paras viiden kortin käsi. Se voi käyttää kahta, yhtä tai ei yhtään omaa korttia.
/// `None`, jos kortteja on alle viisi.
pub fn paras_kasi(kortit: &[Kortti]) -> Option<(Kadenarvo, [Kortti; KATEEN])> {
    let mut paras: Option<(Kadenarvo, [Kortti; KATEEN])> = None;
    for maski in 0u32..1 << kortit.len() {
        if maski.count_ones() as usize != KATEEN {
            continue;
        }
        let mut viisi = [kortit[0]; KATEEN];
        let valitut = (0..kortit.len()).filter(|i| maski >> i & 1 != 0);
        for (paikka, i) in viisi.iter_mut().zip(valitut) {
            *paikka = kortit[i];
        }
        let arvo = viiden_arvo(&viisi);
        if paras.is_none_or(|(vanha, _)| arvo > vanha) {
            paras = Some((arvo, viisi));
        }
    }
    paras
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tila {
    /// Rahat loppuivat aiemmassa jaossa.
    Poissa,
    Kippasi,
    /// Jaossa mukana. Ilman rahaa pelaaja on all-in.
    Mukana,
}

/// Viimeisin teko näytetään pelaajan kohdalla. Summa on panos tällä kadulla.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Teko {
    Tarkisti,
    Maksoi(u32),
    Panosti(u32),
    Korotti(u32),
    Kippasi,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Toimi {
    Kippaa,
    /// Tarkistaa, jos maksettavaa ei ole. Muuten maksaa niin paljon kuin rahat riittävät.
    Maksa,
    /// Nostaa oman kokonaispanoksen tällä kadulla tähän summaan.
    Korota(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kulku {
    Panostus,
    /// Kierros on panostettu, ja seuraavat yhteiset kortit odottavat kääntämistä.
    Kaanto,
    /// Kortit näytetään yksi käsi kerrallaan.
    Naytto,
    /// Potit on jaettu.
    Ohi,
}

#[derive(Clone, Debug)]
pub struct Pelaaja {
    pub rahat: u32,
    /// Panos tällä kadulla.
    pub panos: u32,
    /// Koko jaon panokset: potit lasketaan näistä.
    pub yhteensa: u32,
    pub tila: Tila,
    pub kortit: Vec<Kortti>,
    pub teko: Option<Teko>,
    /// Kortit ovat kaikkien nähtävillä.
    pub naytetty: bool,
    pub kasi: Option<(Kadenarvo, [Kortti; KATEEN])>,
    /// Tämän jaon voitto potista, oma panos mukaan lukien.
    pub voitto: u32,
    /// Osa voitosta, joka tuli pääpotista. Loppu tuli sivupoteista.
    pub paapotista: u32,
    toiminut: bool,
    /// Korkein panos, kun pelaaja viimeksi toimi: korotusoikeus palaa vasta täydestä korotuksesta.
    nahty: u32,
}

/// Potti ja pelaajat, jotka voivat sen voittaa.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Potti {
    pub summa: u32,
    pub oikeutetut: Vec<usize>,
}

/// Koneen pelityyli.
#[derive(Clone, Copy)]
struct Tyyli {
    /// Lisää vaadittua voiton todennäköisyyttä.
    tiukkuus: f32,
    /// Todennäköisyys korottaa vahvalla kädellä.
    aggressio: f32,
    /// Todennäköisyys panostaa heikolla kädellä.
    bluffi: f32,
}

const TYYLIT: [Tyyli; PELAAJIA] = [
    Tyyli {
        tiukkuus: 0.0,
        aggressio: 0.5,
        bluffi: 0.05,
    },
    Tyyli {
        tiukkuus: 0.08,
        aggressio: 0.35,
        bluffi: 0.02,
    },
    Tyyli {
        tiukkuus: -0.04,
        aggressio: 0.85,
        bluffi: 0.12,
    },
    Tyyli {
        tiukkuus: 0.03,
        aggressio: 0.7,
        bluffi: 0.05,
    },
    Tyyli {
        tiukkuus: -0.06,
        aggressio: 0.3,
        bluffi: 0.03,
    },
];
const KOKEITA: usize = 200;

pub struct Poyta {
    pub pelaajat: [Pelaaja; PELAAJIA],
    pub yhteiset: Vec<Kortti>,
    /// Päällimmäinen kortti on viimeisenä.
    pub pakka: Vec<Kortti>,
    pub kulku: Kulku,
    pub vuorossa: Option<usize>,
    /// Nappi voi olla tyhjällä paikalla (kuollut nappi).
    pub nappi: usize,
    pub pieni: usize,
    pub iso: usize,
    /// Monesko jako on menossa, ykkösestä alkaen.
    pub jako: u32,
    /// Korkein panos tällä kadulla. Blindikierroksella vähintään iso blindi, vaikka se olisi vajaa.
    pub korkein: u32,
    /// Viimeisin täysi panos tai korotus. Sitä pienempi all-in ei muuta tätä.
    pub pienin_korotus: u32,
    pub viime_hyokkaaja: Option<usize>,
    korotuksia: u32,
    jarjestys: Vec<usize>,
}

fn edellinen(i: usize) -> usize {
    (i + PELAAJIA - 1) % PELAAJIA
}

/// Paikat myötäpäivään `i`:n jälkeen, `i` itse viimeisenä.
fn kierros(i: usize) -> impl Iterator<Item = usize> {
    (1..=PELAAJIA).map(move |askel| (i + askel) % PELAAJIA)
}

impl Poyta {
    /// Kukaan ei ole vielä saanut kortteja: ensimmäinen jako alkaa `jaa`-kutsusta.
    pub fn new(arpa: &mut Arpa) -> Self {
        let pelaaja = Pelaaja {
            rahat: ALKURAHA,
            panos: 0,
            yhteensa: 0,
            tila: Tila::Mukana,
            kortit: Vec::new(),
            teko: None,
            naytetty: false,
            kasi: None,
            voitto: 0,
            paapotista: 0,
            toiminut: false,
            nahty: 0,
        };
        let iso = arpa.alle(PELAAJIA);
        Self {
            pelaajat: std::array::from_fn(|_| pelaaja.clone()),
            yhteiset: Vec::new(),
            pakka: Vec::new(),
            kulku: Kulku::Ohi,
            vuorossa: None,
            nappi: edellinen(edellinen(iso)),
            pieni: edellinen(iso),
            iso,
            jako: 0,
            korkein: 0,
            pienin_korotus: ISO_ALUSSA,
            viime_hyokkaaja: None,
            korotuksia: 0,
            jarjestys: Vec::new(),
        }
    }

    /// Blindit (pieni, iso) jaon numeron mukaan. Uusi taso alkaa aina uudesta jaosta.
    pub fn blindit(&self) -> (u32, u32) {
        let taso = self.taso().min(SUURIN_TASO);
        (PIENI_ALUSSA << taso, ISO_ALUSSA << taso)
    }

    pub fn taso(&self) -> u32 {
        self.jako.saturating_sub(1) / TASON_JAKOJA
    }

    /// Montako jakoa tämän jälkeen pelataan vielä samoilla blindeillä.
    pub fn jakoja_tasolla(&self) -> u32 {
        (self.taso() + 1) * TASON_JAKOJA - self.jako.max(1)
    }

    pub fn potti(&self) -> u32 {
        self.pelaajat.iter().map(|p| p.yhteensa).sum()
    }

    pub fn rahoissa(&self) -> usize {
        self.pelaajat.iter().filter(|p| p.rahat > 0).count()
    }

    fn mukana(&self, i: usize) -> bool {
        self.pelaajat[i].tila == Tila::Mukana
    }

    /// Pelaaja voi vielä päättää jotain: hän on mukana eikä ole all-in.
    fn paattaa(&self, i: usize) -> bool {
        self.mukana(i) && self.pelaajat[i].rahat > 0
    }

    fn muita_paattajia(&self, i: usize) -> bool {
        (0..PELAAJIA).any(|j| j != i && self.paattaa(j))
    }

    fn muiden_suurin_panos(&self, i: usize) -> u32 {
        (0..PELAAJIA)
            .filter(|&j| j != i && self.mukana(j))
            .map(|j| self.pelaajat[j].panos)
            .max()
            .unwrap_or(0)
    }

    /// Paljonko pelaajan pitää maksaa pysyäkseen mukana. Jos kaikki muut ovat all-in,
    /// heidän panoksensa ylittävää osaa ei tarvitse edes laittaa pöytään.
    pub fn maksu(&self, i: usize) -> u32 {
        let taso = if self.muita_paattajia(i) {
            self.korkein
        } else {
            self.korkein.min(self.muiden_suurin_panos(i))
        };
        let pelaaja = &self.pelaajat[i];
        taso.saturating_sub(pelaaja.panos).min(pelaaja.rahat)
    }

    /// Korotus vaatii rahaa korkeimman panoksen yli ja jonkun, joka voi vielä vastata. Jo toiminut
    /// pelaaja saa korottaa vain, jos häntä vastassa on vähintään täysi korotus.
    pub fn voi_korottaa(&self, i: usize) -> bool {
        let pelaaja = &self.pelaajat[i];
        self.paattaa(i)
            && self.muita_paattajia(i)
            && pelaaja.panos + pelaaja.rahat > self.korkein
            && (!pelaaja.toiminut
                || self.korkein.saturating_sub(pelaaja.nahty) >= self.pienin_korotus)
    }

    /// Suurin summa, johon pelaaja voi korottaa: kaikki rahat.
    pub fn suurin_taso(&self, i: usize) -> u32 {
        self.pelaajat[i].panos + self.pelaajat[i].rahat
    }

    /// Pienin täysi korotus, tai kaikki rahat, jos ne eivät siihen riitä.
    pub fn pienin_taso(&self, i: usize) -> u32 {
        (self.korkein + self.pienin_korotus).min(self.suurin_taso(i))
    }

    fn kelpaa(&self, i: usize, taso: u32) -> bool {
        self.voi_korottaa(i) && taso >= self.pienin_taso(i) && taso <= self.suurin_taso(i)
    }

    /// Paljonko pottiin tulee sivupotteja: all-in-pelaajan panos rajaa oman pottinsa.
    pub fn sivupotteja(&self) -> usize {
        let huippu = (0..PELAAJIA)
            .filter(|&i| self.mukana(i))
            .map(|i| self.pelaajat[i].yhteensa)
            .max()
            .unwrap_or(0);
        let mut tasot: Vec<u32> = (0..PELAAJIA)
            .filter(|&i| self.mukana(i) && self.pelaajat[i].rahat == 0)
            .map(|i| self.pelaajat[i].yhteensa)
            .filter(|&summa| summa < huippu)
            .collect();
        tasot.sort_unstable();
        tasot.dedup();
        tasot.len()
    }

    /// Pääpotti ensin. Kippaajien rahat jäävät pottiin, mutta he eivät voi voittaa niitä.
    pub fn potit(&self) -> Vec<Potti> {
        let mut tasot: Vec<u32> = (0..PELAAJIA)
            .filter(|&i| self.mukana(i) && self.pelaajat[i].yhteensa > 0)
            .map(|i| self.pelaajat[i].yhteensa)
            .collect();
        tasot.sort_unstable();
        tasot.dedup();
        let mut potit: Vec<Potti> = Vec::new();
        let mut edellinen_taso = 0;
        for &taso in &tasot {
            let summa = (self.pelaajat.iter())
                .map(|p| p.yhteensa.min(taso) - p.yhteensa.min(edellinen_taso))
                .sum();
            let oikeutetut = (0..PELAAJIA)
                .filter(|&i| self.mukana(i) && self.pelaajat[i].yhteensa >= taso)
                .collect();
            potit.push(Potti { summa, oikeutetut });
            edellinen_taso = taso;
        }
        let yli: u32 = (self.pelaajat.iter())
            .map(|p| p.yhteensa.saturating_sub(edellinen_taso))
            .sum();
        if let Some(viimeinen) = potit.last_mut() {
            viimeinen.summa += yli;
        }
        potit
    }

    /// Aloittaa seuraavan jaon uudesta pakasta. Epätosi, jos peli on jo ratkennut.
    pub fn jaa(&mut self, arpa: &mut Arpa) -> bool {
        self.jaa_pakasta(sekoitettu(1, arpa))
    }

    /// Kuten `jaa`, mutta kortit nousevat annetusta pakasta.
    pub fn jaa_pakasta(&mut self, pakka: Vec<Kortti>) -> bool {
        if self.rahoissa() < 2 {
            return false;
        }
        self.pakka = pakka;
        self.jako += 1;
        self.yhteiset.clear();
        self.viime_hyokkaaja = None;
        self.korotuksia = 0;
        self.jarjestys.clear();
        for pelaaja in &mut self.pelaajat {
            pelaaja.tila = if pelaaja.rahat > 0 {
                Tila::Mukana
            } else {
                Tila::Poissa
            };
            pelaaja.panos = 0;
            pelaaja.yhteensa = 0;
            pelaaja.kortit.clear();
            pelaaja.teko = None;
            pelaaja.naytetty = false;
            pelaaja.kasi = None;
            pelaaja.voitto = 0;
            pelaaja.paapotista = 0;
            pelaaja.toiminut = false;
            pelaaja.nahty = 0;
        }
        self.siirra_nappi();

        // Kortit yksi kerrallaan napista vasemmalle, joten nappi saa viimeisen.
        for _ in 0..2 {
            for i in kierros(self.nappi) {
                if self.mukana(i) {
                    let kortti = self.nosta();
                    self.pelaajat[i].kortit.push(kortti);
                }
            }
        }

        let (pieni, iso) = self.blindit();
        // Kuolleen pienen blindin paikalla ei ole ketään, joka maksaisi.
        if self.mukana(self.pieni) {
            self.lisaa_panos(self.pieni, pieni);
        }
        self.lisaa_panos(self.iso, iso);
        // Vajaakin iso blindi pitää maksaa täytenä.
        self.korkein = iso;
        self.pienin_korotus = iso;
        self.kulku = Kulku::Panostus;
        self.vuorossa = self.seuraava_toimija(self.iso);
        if self.vuorossa.is_none() {
            self.kadun_loppu();
        }
        true
    }

    /// Iso blindi siirtyy aina seuraavalle rahoissa olevalle, joten kukaan ei pääse sitä karkuun.
    fn siirra_nappi(&mut self) {
        let edellinen_iso = self.iso;
        let edellinen_pieni = self.pieni;
        self.iso = (kierros(edellinen_iso).find(|&i| self.mukana(i))).unwrap_or(edellinen_iso);
        if self
            .pelaajat
            .iter()
            .filter(|p| p.tila == Tila::Mukana)
            .count()
            == 2
        {
            // Kaksinpelissä nappi on pieni blindi.
            self.pieni = (kierros(self.iso).find(|&i| self.mukana(i))).unwrap_or(self.iso);
            self.nappi = self.pieni;
        } else {
            self.pieni = edellinen_iso;
            self.nappi = if self.mukana(edellinen_pieni) {
                edellinen_pieni
            } else {
                edellinen(self.pieni)
            };
        }
    }

    fn nosta(&mut self) -> Kortti {
        self.pakka.pop().expect("pakka riittää koko jakoon")
    }

    fn lisaa_panos(&mut self, i: usize, maara: u32) {
        let pelaaja = &mut self.pelaajat[i];
        let maara = maara.min(pelaaja.rahat);
        pelaaja.rahat -= maara;
        pelaaja.panos += maara;
        pelaaja.yhteensa += maara;
    }

    /// Kaikkien ollessa all-in viimeisen on tehtävä päätös vain, jos hän on jäljessä.
    fn pitaa_toimia(&self, i: usize) -> bool {
        let pelaaja = &self.pelaajat[i];
        self.paattaa(i)
            && (!pelaaja.toiminut || pelaaja.panos < self.korkein)
            && (self.muita_paattajia(i) || pelaaja.panos < self.muiden_suurin_panos(i))
    }

    fn seuraava_toimija(&self, jalkeen: usize) -> Option<usize> {
        kierros(jalkeen).find(|&i| self.pitaa_toimia(i))
    }

    /// Vuorossa olevan pelaajan teko. Epätosi, jos teko ei kelpaa, eikä mikään silloin muutu.
    pub fn toimi(&mut self, toimi: Toimi) -> bool {
        let Some(i) = self.vuorossa.filter(|_| self.kulku == Kulku::Panostus) else {
            return false;
        };
        match toimi {
            Toimi::Kippaa => {
                self.pelaajat[i].tila = Tila::Kippasi;
                self.pelaajat[i].teko = Some(Teko::Kippasi);
            }
            Toimi::Maksa => {
                let maksu = self.maksu(i);
                self.lisaa_panos(i, maksu);
                self.pelaajat[i].teko = Some(if maksu == 0 {
                    Teko::Tarkisti
                } else {
                    Teko::Maksoi(self.pelaajat[i].panos)
                });
            }
            Toimi::Korota(taso) => {
                if !self.kelpaa(i, taso) {
                    return false;
                }
                self.lisaa_panos(i, taso - self.pelaajat[i].panos);
                let lisays = taso - self.korkein;
                // Vajaa all-in nostaa panosta, mutta seuraava korotus mitataan viime täydestä.
                if lisays >= self.pienin_korotus {
                    self.pienin_korotus = lisays;
                }
                self.pelaajat[i].teko = Some(if self.korkein == 0 {
                    Teko::Panosti(taso)
                } else {
                    Teko::Korotti(taso)
                });
                self.korkein = taso;
                self.viime_hyokkaaja = Some(i);
                self.korotuksia += 1;
            }
        }
        self.pelaajat[i].toiminut = true;
        self.pelaajat[i].nahty = self.korkein;
        if (0..PELAAJIA).filter(|&j| self.mukana(j)).count() == 1 {
            self.muut_kippasivat();
            return true;
        }
        self.vuorossa = self.seuraava_toimija(i);
        if self.vuorossa.is_none() {
            self.kadun_loppu();
        }
        true
    }

    /// Yksin korkeimman panoksen laittanut saa maksamatta jääneen osan takaisin.
    fn palauta_ylijaama(&mut self) {
        let mut jarjestetty: Vec<usize> = (0..PELAAJIA).collect();
        jarjestetty.sort_by_key(|&i| std::cmp::Reverse(self.pelaajat[i].yhteensa));
        let (ylin, toinen) = (jarjestetty[0], jarjestetty[1]);
        let yli = self.pelaajat[ylin].yhteensa - self.pelaajat[toinen].yhteensa;
        if self.mukana(ylin) {
            let pelaaja = &mut self.pelaajat[ylin];
            pelaaja.rahat += yli;
            pelaaja.yhteensa -= yli;
            pelaaja.panos = pelaaja.panos.saturating_sub(yli);
        }
    }

    fn muut_kippasivat(&mut self) {
        self.palauta_ylijaama();
        let potti = self.potti();
        if let Some(i) = (0..PELAAJIA).find(|&i| self.mukana(i)) {
            self.pelaajat[i].rahat += potti;
            self.pelaajat[i].voitto = potti;
        }
        self.tyhjenna_panokset();
        self.vuorossa = None;
        self.kulku = Kulku::Ohi;
    }

    fn tyhjenna_panokset(&mut self) {
        for pelaaja in &mut self.pelaajat {
            pelaaja.panos = 0;
            pelaaja.yhteensa = 0;
        }
    }

    fn kadun_loppu(&mut self) {
        self.palauta_ylijaama();
        self.vuorossa = None;
        // Kun panostus on ohi all-inien takia, kortit käännetään heti esiin.
        if (0..PELAAJIA).filter(|&i| self.paattaa(i)).count() <= 1 {
            for i in 0..PELAAJIA {
                if self.mukana(i) {
                    self.pelaajat[i].naytetty = true;
                }
            }
        }
        if self.yhteiset.len() == YHTEISIA {
            self.aloita_naytto();
        } else {
            self.kulku = Kulku::Kaanto;
        }
    }

    /// Viimeisen kierroksen viimeinen panostaja näyttää ensin. Jos kukaan ei panostanut,
    /// aloittaa ensimmäinen napista vasemmalle. Siitä jatketaan myötäpäivään.
    pub fn nayttojarjestys(&self) -> Vec<usize> {
        let alku = (self.viime_hyokkaaja.filter(|&i| self.mukana(i)))
            .or_else(|| kierros(self.nappi).find(|&i| self.mukana(i)))
            .unwrap_or(self.nappi);
        (0..PELAAJIA)
            .map(|askel| (alku + askel) % PELAAJIA)
            .filter(|&i| self.mukana(i))
            .collect()
    }

    fn aloita_naytto(&mut self) {
        self.jarjestys = self.nayttojarjestys();
        self.jarjestys.retain(|&i| !self.pelaajat[i].naytetty);
        self.jarjestys.reverse();
        self.kulku = Kulku::Naytto;
    }

    /// Polttaa kortin ja kääntää seuraavan kadun yhteiset kortit.
    fn kaanna_kortit(&mut self) {
        self.nosta();
        let maara = if self.yhteiset.is_empty() { 3 } else { 1 };
        for _ in 0..maara {
            let kortti = self.nosta();
            self.yhteiset.push(kortti);
        }
        self.korkein = 0;
        self.pienin_korotus = self.blindit().1;
        self.viime_hyokkaaja = None;
        self.korotuksia = 0;
        for pelaaja in &mut self.pelaajat {
            pelaaja.panos = 0;
            pelaaja.toiminut = false;
            pelaaja.nahty = 0;
            if pelaaja.tila == Tila::Mukana {
                pelaaja.teko = None;
            }
        }
        self.kulku = Kulku::Panostus;
        self.vuorossa = self.seuraava_toimija(self.nappi);
        if self.vuorossa.is_none() {
            self.kadun_loppu();
        }
    }

    fn laske_kasi(&mut self, i: usize) {
        let mut kortit = self.pelaajat[i].kortit.clone();
        kortit.extend_from_slice(&self.yhteiset);
        self.pelaajat[i].kasi = paras_kasi(&kortit);
    }

    fn jaa_potit(&mut self) {
        for i in 0..PELAAJIA {
            if self.mukana(i) {
                self.laske_kasi(i);
            }
        }
        for (monesko, potti) in self.potit().into_iter().enumerate() {
            let arvo = |i: usize| self.pelaajat[i].kasi.map(|(arvo, _)| arvo);
            let paras = potti.oikeutetut.iter().map(|&i| arvo(i)).max().flatten();
            // Parittomat merkit menevät voittajille napista vasemmalle lukien.
            let voittajat: Vec<usize> = kierros(self.nappi)
                .filter(|i| potti.oikeutetut.contains(i) && arvo(*i) == paras)
                .collect();
            let osuus = potti.summa / voittajat.len() as u32;
            let parittomia = (potti.summa % voittajat.len() as u32) as usize;
            for (sija, &i) in voittajat.iter().enumerate() {
                let maara = osuus + (sija < parittomia) as u32;
                self.pelaajat[i].rahat += maara;
                self.pelaajat[i].voitto += maara;
                if monesko == 0 {
                    self.pelaajat[i].paapotista = maara;
                }
            }
        }
        self.tyhjenna_panokset();
        self.kulku = Kulku::Ohi;
    }

    /// Pelaajan paras käsi nyt, jos yhteisiä kortteja on jo näkyvissä.
    pub fn nykyinen_kasi(&self, i: usize) -> Option<Kadenarvo> {
        let mut kortit = self.pelaajat[i].kortit.clone();
        kortit.extend_from_slice(&self.yhteiset);
        paras_kasi(&kortit).map(|(arvo, _)| arvo)
    }

    /// Yksi kellon tahdissa tapahtuva siirto: koneen teko, kortin käännös tai käden näyttö.
    pub fn etene(&mut self, arpa: &mut Arpa) {
        match self.kulku {
            Kulku::Panostus => {
                if let Some(i) = self.vuorossa {
                    let toimi = self.koneen_paatos(i, arpa);
                    self.toimi(toimi);
                }
            }
            Kulku::Kaanto => self.kaanna_kortit(),
            Kulku::Naytto => match self.jarjestys.pop() {
                Some(i) => {
                    self.pelaajat[i].naytetty = true;
                    self.laske_kasi(i);
                }
                None => self.jaa_potit(),
            },
            Kulku::Ohi => {}
        }
    }

    /// Koneen arvio voiton todennäköisyydestä. Se tuntee vain omat ja yhteiset kortit: muiden
    /// kortit ja loppupakka arvotaan kokeisiin kaikista korteista, joita se ei näe.
    pub fn voittotodennakoisyys(
        &self,
        i: usize,
        vastustajia: usize,
        kokeita: usize,
        arpa: &mut Arpa,
    ) -> f32 {
        let tunnetut: Vec<Kortti> = (self.pelaajat[i].kortit.iter())
            .chain(&self.yhteiset)
            .copied()
            .collect();
        let mut muut: Vec<Kortti> = Maa::KAIKKI
            .iter()
            .flat_map(|&maa| (1..=KUNINGAS).map(move |arvo| Kortti { maa, arvo }))
            .filter(|kortti| !tunnetut.contains(kortti))
            .collect();
        let lisaa = YHTEISIA - self.yhteiset.len();
        let tarve = lisaa + 2 * vastustajia;
        let mut tulos = 0.0;
        for _ in 0..kokeita {
            for j in 0..tarve {
                let toinen = j + arpa.alle(muut.len() - j);
                muut.swap(j, toinen);
            }
            let mut kortit = tunnetut.clone();
            kortit.extend_from_slice(&muut[..lisaa]);
            let oma = paras_kasi(&kortit).map(|(arvo, _)| arvo);
            let mut tasan = 1.0;
            let mut havisi = false;
            for pari in muut[lisaa..tarve].chunks_exact(2) {
                kortit[..2].copy_from_slice(pari);
                let toinen = paras_kasi(&kortit).map(|(arvo, _)| arvo);
                havisi |= toinen > oma;
                tasan += (toinen == oma) as u32 as f32;
            }
            if !havisi {
                tulos += 1.0 / tasan;
            }
        }
        tulos / kokeita as f32
    }

    /// Koneen teko. Ei katso muiden kortteja eikä pakkaa, vain sen minkä pöydässä näkisi.
    pub fn koneen_paatos(&self, i: usize, arpa: &mut Arpa) -> Toimi {
        let tyyli = TYYLIT[i];
        let vastustajia = (0..PELAAJIA)
            .filter(|&j| j != i && self.mukana(j))
            .count()
            .max(1);
        let voitto = self.voittotodennakoisyys(i, vastustajia, KOKEITA, arpa);
        // Nolla on keskimääräinen käsi tässä pelaajamäärässä, yksi varma voitto.
        let reilu = 1.0 / (vastustajia + 1) as f32;
        let ylivoima = (voitto - reilu) / (1.0 - reilu);
        let maksu = self.maksu(i);
        let potti = self.potti();
        let mut noppa = || arpa.alle(1000) as f32 / 1000.0;
        // Jokainen korotus kertoo vahvasta kädestä, joten seuraava vaatii enemmän.
        let vaadittu = 0.25 + tyyli.tiukkuus + self.korotuksia as f32 * 0.15;

        let korottaa = self.voi_korottaa(i)
            && if maksu == 0 {
                ylivoima > vaadittu || noppa() < tyyli.bluffi
            } else {
                (ylivoima > vaadittu + 0.2 && noppa() < tyyli.aggressio)
                    || (maksu * 4 < potti && noppa() < tyyli.bluffi / 2.0)
            };
        if korottaa {
            let koko = (potti + maksu) as f32 * (0.5 + noppa() * 0.5 * tyyli.aggressio);
            let taso = self.korkein + koko as u32;
            let (pienin, suurin) = (self.pienin_taso(i), self.suurin_taso(i));
            // Lähes koko pino pöydässä: loput eivät enää muuta mitään.
            return Toimi::Korota(if taso * 10 >= suurin * 7 {
                suurin
            } else {
                taso.clamp(pienin, suurin)
            });
        }
        if maksu == 0 {
            return Toimi::Maksa;
        }
        let kertoimet = maksu as f32 / (potti + maksu) as f32;
        // Ennen floppia tuleva voitto kasvattaa pottia, joten kertoimista saa tinkiä.
        let tinki = if self.yhteiset.is_empty() { 0.7 } else { 0.9 };
        let pino = self.pelaajat[i].rahat + self.pelaajat[i].panos;
        let sitoutuminen = 0.25 * maksu as f32 / pino as f32;
        // Selvästi keskimääräistä vahvempaa kättä ei kipata panoksen koosta välittämättä.
        if ylivoima > 0.35 + tyyli.tiukkuus
            || voitto >= kertoimet * tinki + tyyli.tiukkuus + sitoutuminen
        {
            Toimi::Maksa
        } else {
            Toimi::Kippaa
        }
    }
}
