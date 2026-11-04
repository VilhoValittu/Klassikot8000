//! Silmukan säännöt ilman ikkunaa, piirtoa tai syötettä. Mato ei kuole osuessaan itseensä:
//! osuma sulkee silmukan, joka pyydystää kaiken sisäänsä jäävän. Saaliin kanssa silmukka
//! maksaa neljänneksen pituudestaan, tyhjänä koko pituuden. Aika etenee kiinteinä askelina, joten sama siemen ja ohjaus toistavat saman pelin.

use std::collections::VecDeque;

use crate::Arpa;

pub const SARAKKEITA: i32 = 27;
pub const RIVEJA: i32 = 22;
pub const ASKEL_MS: u32 = 5;
pub const ALKUPITUUS: usize = 8;
/// Kuusi numeroa mahtuu sivupalkkiin, ja nettilista hylkää tätä suuremmat tulokset.
pub const SUURIN_PISTEMAARA: u32 = 999_999;
pub const TASON_KESTO_MS: u32 = 20_000;
pub const KUORIUTUMINEN_MS: u32 = 1500;
pub const KULLAN_KESTO_MS: u32 = 7000;

const OMENOITA: usize = 3;
const OMENAN_KASVU: u32 = 3;
const SAALIIN_KASVU: u32 = 2;
// Saalista tuottava silmukka maksaa vain tämän osan pituudestaan: loput kasvaa takaisin.
// Tyhjä silmukka kuluttaa koko pituutensa.
const SILMUKAN_HINTA_JAKAJA: usize = 4;
const SILMUKAN_HINTA_VAHINTAAN: usize = 2;
// Silmukan jälkeen madossa on aina tarpeeksi pituutta seuraavan alkuun.
const VAHIN_PITUUS: usize = 5;
const OTUKSIA_ENINTAAN: usize = 16;
// Nopeudet ovat lattiassa tällä tasolla, joten siitä eteenpäin vaikeus tulee parvesta.
const PARVEN_KASVU_TASO: u32 = 13;
const PARVI_ENINTAAN: usize = 30;
// Itikka lentää hetken ja laskeutuu lepäämään: levossa sen ehtii kiertää.
const LENNON_ASKELIA: u32 = 9;
pub const LEPO_MS: u32 = 2200;
// Otus ei saa kuoriutua madon nenän eteen.
const VALI_PAASTA: i32 = 7;
const PAARMAN_ENSI_TASO: u32 = 3;
const PAARMA_JOKA: u32 = 3;
const PAARMA_JOKA_LOPUSSA: u32 = 2;
const KULLAN_VALI_MS: (usize, usize) = (18_000, 30_000);
// Näin pitkälle suoraan edessä oleva oma vartalo näytetään tulevana silmukkana.
const ENNUSTEEN_MATKA: usize = 12;
const ARVAUKSIA: usize = 200;

pub type Ruutu = (i32, i32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Suunta {
    Ylos,
    Alas,
    Vasen,
    Oikea,
}

impl Suunta {
    pub fn siirto(self) -> Ruutu {
        match self {
            Suunta::Ylos => (0, -1),
            Suunta::Alas => (0, 1),
            Suunta::Vasen => (-1, 0),
            Suunta::Oikea => (1, 0),
        }
    }

    fn vastakkainen(self) -> Suunta {
        match self {
            Suunta::Ylos => Suunta::Alas,
            Suunta::Alas => Suunta::Ylos,
            Suunta::Vasen => Suunta::Oikea,
            Suunta::Oikea => Suunta::Vasen,
        }
    }

    fn sivut(self) -> [Suunta; 2] {
        match self {
            Suunta::Ylos | Suunta::Alas => [Suunta::Vasen, Suunta::Oikea],
            Suunta::Vasen | Suunta::Oikea => [Suunta::Ylos, Suunta::Alas],
        }
    }
}

fn plus(a: Ruutu, b: Ruutu) -> Ruutu {
    (a.0 + b.0, a.1 + b.1)
}

pub fn kentalla(r: Ruutu) -> bool {
    (0..SARAKKEITA).contains(&r.0) && (0..RIVEJA).contains(&r.1)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Laji {
    /// Kimpoilee vinottain seinistä ja madosta.
    Itikka,
    /// Jahtaa madon päätä, mutta ei pääse vartalon läpi.
    Paarma,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Otus {
    pub laji: Laji,
    pub paikka: Ruutu,
    /// Piirto liu'uttaa otuksen tästä nykyiseen paikkaan.
    pub edellinen: Ruutu,
    pub suunta: Ruutu,
    /// Yli nolla: otus on vielä muna, joka ei liiku eikä vahingoita.
    pub kuoriutuu_ms: u32,
    pub odotus_ms: u32,
    pub jakso_ms: u32,
    /// Yli nolla: itikka istuu paikallaan, mutta pistää yhä, jos siihen törmää.
    pub lepo_ms: u32,
    lentoa: u32,
}

impl Otus {
    pub fn uusi(laji: Laji, paikka: Ruutu, suunta: Ruutu, kuoriutuu_ms: u32) -> Self {
        Self {
            laji,
            paikka,
            edellinen: paikka,
            suunta,
            kuoriutuu_ms,
            odotus_ms: 0,
            jakso_ms: 1,
            lepo_ms: 0,
            lentoa: LENNON_ASKELIA,
        }
    }

    pub fn on_muna(&self) -> bool {
        self.kuoriutuu_ms > 0
    }

    /// Liu'un eteneminen välillä 0..=1.
    pub fn vaihe(&self) -> f32 {
        1.0 - self.odotus_ms as f32 / self.jakso_ms as f32
    }

    fn kohde(&self) -> Kohde {
        match (self.on_muna(), self.laji) {
            (true, _) => Kohde::Muna,
            (false, Laji::Itikka) => Kohde::Itikka,
            (false, Laji::Paarma) => Kohde::Paarma,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Herkku {
    pub paikka: Ruutu,
    /// Kultaomena katoaa ajan kuluessa, tavallinen omena pysyy.
    pub kulta_ms: Option<u32>,
}

impl Herkku {
    fn kohde(&self) -> Kohde {
        if self.kulta_ms.is_some() {
            Kohde::Kulta
        } else {
            Kohde::Omena
        }
    }
}

/// Silmukkaan jäävä asia ja sen pistearvo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kohde {
    Omena,
    Kulta,
    Muna,
    Itikka,
    Paarma,
}

impl Kohde {
    pub fn arvo(self) -> u32 {
        match self {
            Kohde::Omena => 10,
            Kohde::Muna => 15,
            Kohde::Itikka => 25,
            Kohde::Paarma => 60,
            Kohde::Kulta => 100,
        }
    }
}

/// Silmukan pisteet: arvot lasketaan yhteen ja kerrotaan määrällä, joten yhdellä
/// kertaa pyydystäminen on paljon arvokkaampaa kuin sama saalis osissa.
pub fn saaliin_pisteet(kohteet: impl IntoIterator<Item = Kohde>) -> u32 {
    let (mut summa, mut maara) = (0, 0);
    for kohde in kohteet {
        summa += kohde.arvo();
        maara += 1;
    }
    summa * maara
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Saalis {
    /// Silmukan ruudut. Ensimmäinen on pää, viimeinen on risteys, johon pää osui.
    pub reitti: Vec<Ruutu>,
    pub sisus: Vec<Ruutu>,
    pub kohteet: Vec<(Ruutu, Kohde)>,
    pub pisteet: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Tapahtuma {
    Omena {
        paikka: Ruutu,
        pisteet: u32,
    },
    Kulta {
        paikka: Ruutu,
        pisteet: u32,
    },
    /// Mato litisti munan ennen kuoriutumista.
    Muna {
        paikka: Ruutu,
        pisteet: u32,
    },
    Silmukka(Saalis),
    /// Silmukan sisällä ei ollut mitään: pituus kului hukkaan.
    TyhjaSilmukka {
        reitti: Vec<Ruutu>,
    },
    Kuoriutui {
        paikka: Ruutu,
    },
    KultaIlmestyi {
        paikka: Ruutu,
    },
    KultaKatosi {
        paikka: Ruutu,
    },
    UusiTaso,
    /// Pää seinää vasten: kääntymiseen on vielä yksi askel aikaa.
    Seina,
    Kuolema {
        paikka: Ruutu,
    },
}

/// Silmukka, joka syntyy, jos mato jatkaa suoraan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ennuste {
    pub reitti: Vec<Ruutu>,
    /// Montako ruutua reitin alusta on vielä kulkematta.
    pub suora: usize,
    pub sisus: Vec<Ruutu>,
    pub saalis: usize,
    pub pisteet: u32,
}

pub struct Niitty {
    /// Pää ensin. Ruudut ovat aina erillisiä ja peräkkäiset vierekkäin.
    pub mato: VecDeque<Ruutu>,
    /// Ruutu, josta häntä juuri lähti. Piirto liu'uttaa hännän siitä.
    pub jalki: Ruutu,
    pub suunta: Suunta,
    pub jono: Vec<Suunta>,
    /// Hiiren ruutu. Mato kääntyy sitä kohti, kun nuolia ei ole painettu.
    pub tahtain: Option<Ruutu>,
    /// Mato on käynyt tähtäimellä: se kiertää paikkaa syömättä itseään, kunnes hiiri liikkuu.
    perilla: bool,
    pub kasvu: u32,
    pub otukset: Vec<Otus>,
    pub herkut: Vec<Herkku>,
    pub pisteet: u32,
    pub taso: u32,
    pub ohi: bool,
    /// Tosi, kun mato ei edennyt viime askelella, eikä sitä pidä liu'uttaa.
    pub paikallaan: bool,
    pub odotus_ms: u32,
    pub jakso_ms: u32,
    /// Peliaika, jonka mukaan taso nousee.
    pub peliaika_ms: u32,
    armo: bool,
    muninta_ms: u32,
    kulta_ms: u32,
    munia: u32,
}

impl Niitty {
    pub fn new(arpa: &mut Arpa) -> Self {
        let rivi = RIVEJA / 2;
        let paa = SARAKKEITA / 2;
        let mato: VecDeque<Ruutu> = (0..ALKUPITUUS as i32).map(|i| (paa - i, rivi)).collect();
        let jakso_ms = maton_jakso(1);
        let mut niitty = Self {
            jalki: (paa - ALKUPITUUS as i32 + 1, rivi),
            mato,
            suunta: Suunta::Oikea,
            jono: Vec::new(),
            tahtain: None,
            perilla: false,
            kasvu: 0,
            otukset: Vec::new(),
            herkut: Vec::new(),
            pisteet: 0,
            taso: 1,
            ohi: false,
            paikallaan: false,
            odotus_ms: jakso_ms,
            jakso_ms,
            peliaika_ms: 0,
            armo: false,
            muninta_ms: munintavali(1),
            kulta_ms: 0,
            munia: 0,
        };
        niitty.kulta_ms = arvo_vali(KULLAN_VALI_MS, arpa);
        niitty.lisaa_omenat(arpa);
        niitty.muni(arpa);
        niitty
    }

    pub fn paa(&self) -> Ruutu {
        self.mato[0]
    }

    /// Madon liu'un eteneminen välillä 0..=1.
    pub fn vaihe(&self) -> f32 {
        if self.paikallaan {
            1.0
        } else {
            1.0 - self.odotus_ms as f32 / self.jakso_ms as f32
        }
    }

    /// Nuolen painallus. Kaksi käännöstä mahtuu jonoon, jotta nopea U-käännös onnistuu,
    /// vaikka näppäimet osuisivat saman askeleen sisään.
    pub fn kaanny(&mut self, suunta: Suunta) {
        self.tahtain = None;
        let viimeinen = self.jono.last().copied().unwrap_or(self.suunta);
        if suunta != viimeinen && suunta != viimeinen.vastakkainen() && self.jono.len() < 2 {
            self.jono.push(suunta);
        }
    }

    /// Hiiren ruutu. Sama ruutu uudelleen ei muuta mitään, joten kierto osoittimen ympärillä jatkuu.
    pub fn aseta_tahtain(&mut self, tahtain: Ruutu) {
        if self.tahtain != Some(tahtain) {
            self.tahtain = Some(tahtain);
            self.perilla = false;
        }
    }

    /// Suunta, johon mato lähtee seuraavalla askeleella.
    pub fn seuraava_suunta(&self) -> Suunta {
        if let Some(&suunta) = self.jono.first() {
            suunta
        } else if let Some(tahtain) = self.tahtain {
            self.tahtaimen_suunta(tahtain)
        } else {
            self.suunta
        }
    }

    fn tahtaimen_suunta(&self, tahtain: Ruutu) -> Suunta {
        let (dx, dy) = (tahtain.0 - self.paa().0, tahtain.1 - self.paa().1);
        let vaaka = if dx > 0 { Suunta::Oikea } else { Suunta::Vasen };
        let pysty = if dy > 0 { Suunta::Alas } else { Suunta::Ylos };
        let mut ehdokkaat = Vec::with_capacity(5);
        if dx.abs() >= dy.abs() {
            ehdokkaat.extend((dx != 0).then_some(vaaka));
            ehdokkaat.extend((dy != 0).then_some(pysty));
        } else {
            ehdokkaat.extend((dy != 0).then_some(pysty));
            ehdokkaat.extend((dx != 0).then_some(vaaka));
        }
        // Takana oleva tähtäin kierretään sivun kautta.
        ehdokkaat.extend(self.suunta.sivut());
        ehdokkaat.push(self.suunta);
        ehdokkaat
            .into_iter()
            .filter(|&s| s != self.suunta.vastakkainen())
            .find(|&s| {
                let kohta = plus(self.paa(), s.siirto());
                kentalla(kohta) && !(self.perilla && self.vartalon_kohta(kohta, 0).is_some())
            })
            .unwrap_or(self.suunta)
    }

    /// Kohdan paikka madossa, kun häntä on ehtinyt lähteä `poistuneet` ruutua.
    fn vartalon_kohta(&self, kohta: Ruutu, askeleita: usize) -> Option<usize> {
        let poistuneet = (askeleita + 1).saturating_sub(self.kasvu as usize);
        let raja = self.mato.len().saturating_sub(poistuneet);
        self.mato.iter().take(raja).position(|&r| r == kohta)
    }

    fn on_madossa(&self, kohta: Ruutu) -> bool {
        self.mato.contains(&kohta)
    }

    /// Yksi kiinteä askel. Pelin päätyttyä otukset lentävät vielä, mutta mikään ei muutu.
    pub fn askel(&mut self, arpa: &mut Arpa) -> Vec<Tapahtuma> {
        let mut tapahtumat = Vec::new();
        self.liikuta_otukset(&mut tapahtumat);
        if self.ohi {
            return tapahtumat;
        }
        self.etene_aika(arpa, &mut tapahtumat);
        self.odotus_ms = self.odotus_ms.saturating_sub(ASKEL_MS);
        if self.odotus_ms == 0 {
            self.jakso_ms = maton_jakso(self.taso);
            self.odotus_ms = self.jakso_ms;
            self.liikuta_matoa(arpa, &mut tapahtumat);
        }
        tapahtumat
    }

    fn etene_aika(&mut self, arpa: &mut Arpa, tapahtumat: &mut Vec<Tapahtuma>) {
        self.peliaika_ms += ASKEL_MS;
        let taso = self.peliaika_ms / TASON_KESTO_MS + 1;
        if taso != self.taso {
            self.taso = taso;
            tapahtumat.push(Tapahtuma::UusiTaso);
        }
        self.muninta_ms = self.muninta_ms.saturating_sub(ASKEL_MS);
        if self.muninta_ms == 0 {
            self.muninta_ms = munintavali(self.taso);
            if self.otukset.len() < otuksia_enintaan(self.taso) {
                self.muni(arpa);
            }
        }
        self.kulta_ms = self.kulta_ms.saturating_sub(ASKEL_MS);
        if self.kulta_ms == 0 {
            self.kulta_ms = arvo_vali(KULLAN_VALI_MS, arpa);
            if let Some(paikka) = self.vapaa_ruutu(0, arpa) {
                self.herkut.push(Herkku {
                    paikka,
                    kulta_ms: Some(KULLAN_KESTO_MS),
                });
                tapahtumat.push(Tapahtuma::KultaIlmestyi { paikka });
            }
        }
        for herkku in &mut self.herkut {
            if let Some(jaljella) = &mut herkku.kulta_ms {
                *jaljella = jaljella.saturating_sub(ASKEL_MS);
                if *jaljella == 0 {
                    let paikka = herkku.paikka;
                    tapahtumat.push(Tapahtuma::KultaKatosi { paikka });
                }
            }
        }
        self.herkut.retain(|h| h.kulta_ms != Some(0));
    }

    fn vapaa_ruutu(&self, vali_paasta: i32, arpa: &mut Arpa) -> Option<Ruutu> {
        let paa = self.paa();
        (0..ARVAUKSIA)
            .map(|_| {
                (
                    arpa.alle(SARAKKEITA as usize) as i32,
                    arpa.alle(RIVEJA as usize) as i32,
                )
            })
            .find(|&r| {
                (r.0 - paa.0).abs() + (r.1 - paa.1).abs() >= vali_paasta
                    && !self.on_madossa(r)
                    && !self.herkut.iter().any(|h| h.paikka == r)
                    && !self.otukset.iter().any(|o| o.paikka == r)
            })
    }

    fn lisaa_omenat(&mut self, arpa: &mut Arpa) {
        while self.herkut.iter().filter(|h| h.kulta_ms.is_none()).count() < OMENOITA {
            let Some(paikka) = self.vapaa_ruutu(2, arpa) else {
                return;
            };
            self.herkut.push(Herkku {
                paikka,
                kulta_ms: None,
            });
        }
    }

    fn muni(&mut self, arpa: &mut Arpa) {
        let Some(paikka) = self.vapaa_ruutu(VALI_PAASTA, arpa) else {
            return;
        };
        self.munia += 1;
        let joka = if self.taso >= PARVEN_KASVU_TASO {
            PAARMA_JOKA_LOPUSSA
        } else {
            PAARMA_JOKA
        };
        let laji = if self.taso >= PAARMAN_ENSI_TASO && self.munia.is_multiple_of(joka) {
            Laji::Paarma
        } else {
            Laji::Itikka
        };
        let vinot = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
        let suunta = vinot[arpa.alle(vinot.len())];
        self.otukset
            .push(Otus::uusi(laji, paikka, suunta, KUORIUTUMINEN_MS));
    }

    fn liikuta_otukset(&mut self, tapahtumat: &mut Vec<Tapahtuma>) {
        for i in 0..self.otukset.len() {
            let mut otus = self.otukset[i];
            if otus.on_muna() {
                otus.kuoriutuu_ms = otus.kuoriutuu_ms.saturating_sub(ASKEL_MS);
                if otus.kuoriutuu_ms == 0 {
                    tapahtumat.push(Tapahtuma::Kuoriutui {
                        paikka: otus.paikka,
                    });
                }
            } else if otus.lepo_ms > 0 {
                otus.lepo_ms = otus.lepo_ms.saturating_sub(ASKEL_MS);
            } else {
                otus.odotus_ms = otus.odotus_ms.saturating_sub(ASKEL_MS);
                if otus.odotus_ms == 0 && otus.laji == Laji::Itikka && otus.lentoa == 0 {
                    otus.lepo_ms = LEPO_MS;
                    otus.lentoa = LENNON_ASKELIA;
                    otus.edellinen = otus.paikka;
                } else if otus.odotus_ms == 0 {
                    otus.lentoa = otus.lentoa.saturating_sub(1);
                    otus.jakso_ms = otuksen_jakso(otus.laji, self.taso);
                    otus.odotus_ms = otus.jakso_ms;
                    otus.edellinen = otus.paikka;
                    match otus.laji {
                        Laji::Itikka => self.kimpoile(&mut otus),
                        Laji::Paarma => self.jahtaa(&mut otus),
                    }
                    if !self.ohi && otus.paikka == self.paa() {
                        self.kuole(tapahtumat);
                    }
                }
            }
            self.otukset[i] = otus;
        }
    }

    fn este(&self, kohta: Ruutu) -> bool {
        !kentalla(kohta) || self.on_madossa(kohta)
    }

    /// Vinon kulun kumpikin sivuruutu tutkitaan, jottei itikka pujahda vartalon nurkan läpi.
    /// Vain suora osuma päähän tappaa: sivusta pää torjuu itikan kuten muukin vartalo.
    fn kimpoile(&self, otus: &mut Otus) {
        let (x, y) = otus.paikka;
        let (mut dx, mut dy) = otus.suunta;
        for _ in 0..3 {
            let kohde = (x + dx, y + dy);
            let vaaka = self.este((x + dx, y));
            let pysty = self.este((x, y + dy));
            match (vaaka, pysty) {
                (false, false) if kohde == self.paa() || !self.este(kohde) => {
                    otus.paikka = kohde;
                    otus.suunta = (dx, dy);
                    return;
                }
                (true, false) => dx = -dx,
                (false, true) => dy = -dy,
                _ => (dx, dy) = (-dx, -dy),
            }
        }
        otus.suunta = (dx, dy);
    }

    fn jahtaa(&self, otus: &mut Otus) {
        let paa = self.paa();
        let (dx, dy) = (paa.0 - otus.paikka.0, paa.1 - otus.paikka.1);
        let vaaka = (dx.signum(), 0);
        let pysty = (0, dy.signum());
        let ehdokkaat = if dx.abs() >= dy.abs() {
            [vaaka, pysty]
        } else {
            [pysty, vaaka]
        };
        for siirto in ehdokkaat {
            let kohde = plus(otus.paikka, siirto);
            if siirto != (0, 0) && (kohde == paa || !self.este(kohde)) {
                otus.paikka = kohde;
                return;
            }
        }
    }

    fn kuole(&mut self, tapahtumat: &mut Vec<Tapahtuma>) {
        self.ohi = true;
        self.jono.clear();
        tapahtumat.push(Tapahtuma::Kuolema { paikka: self.paa() });
    }

    fn liikuta_matoa(&mut self, arpa: &mut Arpa, tapahtumat: &mut Vec<Tapahtuma>) {
        let suunta = self.seuraava_suunta();
        if !self.jono.is_empty() {
            self.jono.remove(0);
        }
        self.suunta = suunta;
        let uusi = plus(self.paa(), suunta.siirto());
        if let Some(tahtain) = self.tahtain {
            self.perilla |= (tahtain.0 - uusi.0).abs() + (tahtain.1 - uusi.1).abs() <= 1;
        }
        self.paikallaan = false;

        if !kentalla(uusi) {
            if self.armo {
                self.kuole(tapahtumat);
            } else {
                self.armo = true;
                tapahtumat.push(Tapahtuma::Seina);
            }
            self.jalki = self.mato[self.mato.len() - 1];
            self.paikallaan = true;
            return;
        }
        self.armo = false;

        if self
            .otukset
            .iter()
            .any(|o| o.paikka == uusi && !o.on_muna())
        {
            self.jalki = self.mato[self.mato.len() - 1];
            self.paikallaan = true;
            self.kuole(tapahtumat);
            return;
        }

        if let Some(kohta) = self.vartalon_kohta(uusi, 0) {
            self.sulje_silmukka(uusi, kohta, arpa, tapahtumat);
            return;
        }

        self.mato.push_front(uusi);
        if self.kasvu > 0 {
            self.kasvu -= 1;
            self.jalki = self.mato[self.mato.len() - 1];
        } else if let Some(hanta) = self.mato.pop_back() {
            self.jalki = hanta;
        }
        self.syo(uusi, arpa, tapahtumat);
    }

    fn lisaa_pisteet(&mut self, pisteet: u32) {
        self.pisteet = (self.pisteet + pisteet).min(SUURIN_PISTEMAARA);
    }

    fn syo(&mut self, paikka: Ruutu, arpa: &mut Arpa, tapahtumat: &mut Vec<Tapahtuma>) {
        if let Some(i) = self.herkut.iter().position(|h| h.paikka == paikka) {
            let herkku = self.herkut.remove(i);
            let pisteet = herkku.kohde().arvo();
            self.lisaa_pisteet(pisteet);
            self.kasvu += OMENAN_KASVU;
            tapahtumat.push(if herkku.kulta_ms.is_some() {
                Tapahtuma::Kulta { paikka, pisteet }
            } else {
                Tapahtuma::Omena { paikka, pisteet }
            });
            self.lisaa_omenat(arpa);
        }
        if let Some(i) = self.otukset.iter().position(|o| o.paikka == paikka) {
            self.otukset.remove(i);
            let pisteet = Kohde::Muna.arvo();
            self.lisaa_pisteet(pisteet);
            tapahtumat.push(Tapahtuma::Muna { paikka, pisteet });
        }
    }

    fn sulje_silmukka(
        &mut self,
        risteys: Ruutu,
        kohta: usize,
        arpa: &mut Arpa,
        tapahtumat: &mut Vec<Tapahtuma>,
    ) {
        let reitti: Vec<Ruutu> = self.mato.iter().take(kohta + 1).copied().collect();
        let sisus = sisapuoli(&reitti);
        let kohteet = self.poimi_sisalta(&sisus);
        self.mato.drain(..kohta);
        debug_assert_eq!(self.paa(), risteys);
        self.jalki = self.mato[self.mato.len() - 1];
        self.paikallaan = true;

        // Mato lyhenee heti, sillä pää siirtyy risteykseen, mutta häntä odottaa paikallaan,
        // kunnes palautettu pituus on kasvanut takaisin.
        let palautus = if kohteet.is_empty() {
            0
        } else {
            let hinta = (kohta / SILMUKAN_HINTA_JAKAJA).max(SILMUKAN_HINTA_VAHINTAAN);
            kohta.saturating_sub(hinta) as u32
        };
        let kasvu = self.kasvu + palautus + kohteet.len() as u32 * SAALIIN_KASVU;
        let vaje = VAHIN_PITUUS.saturating_sub(self.mato.len()) as u32;
        self.kasvu = kasvu.max(vaje);

        if kohteet.is_empty() {
            tapahtumat.push(Tapahtuma::TyhjaSilmukka { reitti });
            return;
        }
        let pisteet = saaliin_pisteet(kohteet.iter().map(|k| k.1));
        self.lisaa_pisteet(pisteet);
        tapahtumat.push(Tapahtuma::Silmukka(Saalis {
            reitti,
            sisus,
            kohteet,
            pisteet,
        }));
        self.lisaa_omenat(arpa);
    }

    /// Poistaa kentältä kaiken, mikä jäi sisäpuolelle.
    fn poimi_sisalta(&mut self, sisus: &[Ruutu]) -> Vec<(Ruutu, Kohde)> {
        let mut kohteet = Vec::new();
        self.otukset.retain(|otus| {
            let sisalla = sisus.contains(&otus.paikka);
            if sisalla {
                kohteet.push((otus.paikka, otus.kohde()));
            }
            !sisalla
        });
        self.herkut.retain(|herkku| {
            let sisalla = sisus.contains(&herkku.paikka);
            if sisalla {
                kohteet.push((herkku.paikka, herkku.kohde()));
            }
            !sisalla
        });
        kohteet
    }

    /// Silmukka, joka sulkeutuu, jos mato jatkaa nykyiseen suuntaansa ilman käännöksiä.
    /// Liikkuvat otukset voivat tietysti ehtiä muuttaa saalista.
    pub fn ennuste(&self) -> Option<Ennuste> {
        let suunta = self.seuraava_suunta();
        let mut suora = Vec::new();
        let mut kohta = self.paa();
        for askeleita in 0..ENNUSTEEN_MATKA {
            kohta = plus(kohta, suunta.siirto());
            if !kentalla(kohta) {
                return None;
            }
            if let Some(kohdassa) = self.vartalon_kohta(kohta, askeleita) {
                return Some(self.ennuste_kohdasta(suora, kohdassa));
            }
            suora.push(kohta);
        }
        None
    }

    fn ennuste_kohdasta(&self, suora: Vec<Ruutu>, kohdassa: usize) -> Ennuste {
        let matkalla = suora.len();
        let mut reitti = suora;
        reitti.extend(self.mato.iter().take(kohdassa + 1));
        let sisus = sisapuoli(&reitti);
        let otukset = self.otukset.iter().filter(|o| sisus.contains(&o.paikka));
        let herkut = self.herkut.iter().filter(|h| sisus.contains(&h.paikka));
        let kohteet: Vec<Kohde> = otukset
            .map(Otus::kohde)
            .chain(herkut.map(Herkku::kohde))
            .collect();
        Ennuste {
            reitti,
            suora: matkalla,
            sisus,
            saalis: kohteet.len(),
            pisteet: saaliin_pisteet(kohteet),
        }
    }
}

/// Ruudut, joista ei pääse kentän ulkopuolelle kulkematta silmukan läpi.
pub fn sisapuoli(silmukka: &[Ruutu]) -> Vec<Ruutu> {
    // Kentän ympärillä on yhden ruudun vapaa kaista, josta täyttö alkaa.
    let (l, k) = (SARAKKEITA + 2, RIVEJA + 2);
    let indeksi = |r: Ruutu| ((r.1 + 1) * l + r.0 + 1) as usize;
    let mut nahty = vec![false; (l * k) as usize];
    for &ruutu in silmukka {
        nahty[indeksi(ruutu)] = true;
    }
    let mut pino = vec![(-1, -1)];
    nahty[indeksi((-1, -1))] = true;
    while let Some(kohta) = pino.pop() {
        for siirto in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let seuraava = plus(kohta, siirto);
            let alueella =
                (-1..=SARAKKEITA).contains(&seuraava.0) && (-1..=RIVEJA).contains(&seuraava.1);
            if alueella && !nahty[indeksi(seuraava)] {
                nahty[indeksi(seuraava)] = true;
                pino.push(seuraava);
            }
        }
    }
    let mut sisus = Vec::new();
    for y in 0..RIVEJA {
        for x in 0..SARAKKEITA {
            if !nahty[indeksi((x, y))] {
                sisus.push((x, y));
            }
        }
    }
    sisus
}

/// Parven katto kasvaa yhdellä tasoa kohti, kun nopeudet eivät enää kasva.
pub fn otuksia_enintaan(taso: u32) -> usize {
    (OTUKSIA_ENINTAAN + taso.saturating_sub(PARVEN_KASVU_TASO) as usize).min(PARVI_ENINTAAN)
}

fn maton_jakso(taso: u32) -> u32 {
    135u32.saturating_sub(6 * (taso - 1)).max(80)
}

fn otuksen_jakso(laji: Laji, taso: u32) -> u32 {
    match laji {
        Laji::Itikka => 360u32.saturating_sub(15 * (taso - 1)).max(200),
        Laji::Paarma => 430u32.saturating_sub(15 * (taso - 1)).max(260),
    }
}

fn munintavali(taso: u32) -> u32 {
    7000u32.saturating_sub(450 * (taso - 1)).max(2500)
}

fn arvo_vali((alin, ylin): (usize, usize), arpa: &mut Arpa) -> u32 {
    (alin + arpa.alle(ylin - alin)) as u32
}
