//! Majakka8000:n säännöt ilman ikkunaa, piirtoa tai syötettä. Mitat ovat kentän pikseleitä
//! vasemmasta yläkulmasta, kulmat radiaaneja (0 on oikealle ja kulma kasvaa myötäpäivään, joten
//! -π/2 on ylös), ja aika etenee kiinteinä askelina, joten sama siemen ja ohjaus toistavat saman yön.
//!
//! Laivat tulevat pimeästä kohti satamaa. Pimeässä laiva pitää suuntansa, ja virta ja tuuli
//! siirtävät sitä. Keilan valaisema kapteeni näkee karikot, kiertää ne ja kääntyy satamaan, ja
//! muistaa näkemänsä vielä hetken.

use std::f32::consts::{PI, TAU};

use crate::Arpa;

pub const LEVEYS: f32 = 660.0;
pub const KORKEUS: f32 = 540.0;
pub const ASKEL_MS: u32 = 5;
/// Pelastusrenkaat: näin monta haaksirikkoa päättää pelin.
pub const RENKAITA: u32 = 3;
pub const YON_KESTO_MS: u32 = 90_000;
/// Uusia laivoja ei tule enää näin myöhään, jotta viimeiset ehtivät satamaan ennen aamua.
pub const TULO_LOPPUU_MS: u32 = 75_000;
/// Aamun valo levittäytyy näin kauan, ja ilta pimenee yön alussa yhtä nopeasti.
pub const HAMARA_MS: u32 = 2000;
/// Aamun tauko ennen seuraavaa yötä.
pub const AAMUN_TAUKO_MS: u32 = 4000;
/// Satamaan jääneet laivat saavat näin kauan aikaa aamulla, sitten ne lasketaan pois.
const SARASTUKSEN_ENIMMAISAIKA_MS: u32 = 30_000;
/// Kuusi numeroa mahtuu sivupalkkiin, ja nettilista hylkää tätä suuremmat tulokset.
pub const SUURIN_PISTEMAARA: u32 = 999_999;

/// Majakan lamppu: keila lähtee tästä.
pub const MAJAKKA: Piste = (365.0, 470.0);
/// Satamansuun keskikohta. Suulla ei ole rantaa, ja laiva on perillä ylitettyään tämän rivin.
pub const SATAMA: Piste = (300.0, 500.0);
pub const SUUN_PUOLIKAS: f32 = 42.0;
/// Laivat ohjaavat ensin suun eteen ja sieltä suoraan sisään, jottei ranta jää tielle.
pub const ULAPPA: Piste = (300.0, 425.0);
/// Keila kääntyy merelle, ei maalle.
pub const KEILAN_KULMAT: (f32, f32) = (-PI + 0.12, -0.12);
pub const TORVEN_KESTO_MS: u32 = 3500;
pub const TORVEN_HIDASTUS: f32 = 0.35;
/// Salakuljettaja jää kiinni, kun se on ollut keilassa yhteensä näin kauan.
pub const KIINNIOTTO_MS: u32 = 1500;
/// Uusi yö tuo yhden karikon lisää, kunnes meri on täynnä.
pub const ENITEN_KARIKOITA: usize = 14;
/// Karikkojen väli: rahtilaivakin mahtuu kulkemaan kahden karikon välistä.
pub const VAYLA: f32 = 48.0;
pub const SALAKULJETUS_ALKAA: u32 = 1;

const ASKEL_S: f32 = ASKEL_MS as f32 / 1000.0;
/// Kapteeni miettii reittiä näin monen askeleen välein: 50 ms riittää laivan vauhdilla.
const OHJAUKSEN_VALI: u32 = 10;
/// Kapteeni katsoo reitin näin pitkälle eteenpäin.
const NAKO: f32 = 100.0;
/// Turvaväli karikkoon ja rantaan reittiä suunniteltaessa.
const VARA: f32 = 10.0;
/// Reitin arvioinnin askel pikseleinä.
const REITIN_ASKEL: f32 = 4.0;
const KOKEILUN_ASKEL: f32 = 0.14;
const KOKEILUJA: usize = 18;
/// Näin paljon pidempi kierto kelpaa, jos laiva on jo kääntymässä sille puolelle.
const PITO: f32 = 0.5;
/// Osuma lasketaan rungon keskeltä: kylki saa hipaista karikkoa.
const OSUMAN_OSUUS: f32 = 0.6;
const ENSIMMAINEN_TULO_MS: u32 = 1500;
const SUUNNAN_VIRHE: f32 = 0.55;
const SALAKULJETTAJAN_VALI_MS: (usize, usize) = (18_000, 32_000);
const PUUSKAN_VALI_MS: (usize, usize) = (9_000, 15_000);
const PUUSKAN_KESTO_MS: u32 = 2500;
const PUUSKAN_TYONTO: f32 = 12.0;

pub type Piste = (f32, f32);

pub fn etaisyys(a: Piste, b: Piste) -> f32 {
    (a.0 - b.0).hypot(a.1 - b.1)
}

/// Kulmien ero väliltä -π..π.
pub fn kulmaero(a: f32, b: f32) -> f32 {
    (a - b + PI).rem_euclid(TAU) - PI
}

/// Rannan rivi sarakkeessa `x`. Satamansuulla rantaa ei ole, ja majakka seisoo niemen nokassa.
pub fn ranta_y(x: f32) -> f32 {
    let aalto = 503.0 + 7.0 * (x * 0.029).sin() + 4.0 * (x * 0.071 + 1.3).sin();
    let niemi = (x - MAJAKKA.0).abs();
    if niemi < 28.0 {
        aalto.min(MAJAKKA.1 + 6.0 + niemi * niemi / 40.0)
    } else {
        aalto
    }
}

pub fn suulla(x: f32) -> bool {
    (x - SATAMA.0).abs() < SUUN_PUOLIKAS
}

/// Suunta majakalta pisteeseen keilan sallimissa rajoissa. Majakan alapuolella keila jää
/// lähemmän reunan puolelle.
pub fn suunta_majakalta(p: Piste) -> f32 {
    let kulma = (p.1 - MAJAKKA.1).atan2(p.0 - MAJAKKA.0);
    if kulma > 0.0 {
        if kulma > PI / 2.0 {
            KEILAN_KULMAT.0
        } else {
            KEILAN_KULMAT.1
        }
    } else {
        kulma.clamp(KEILAN_KULMAT.0, KEILAN_KULMAT.1)
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

    pub fn kerroin(self) -> u32 {
        match self {
            Vaikeus::Helppo => 1,
            Vaikeus::Normaali => 2,
            Vaikeus::Vaikea => 3,
        }
    }

    /// Keilan puolikas leveys radiaaneina.
    pub fn keilan_puolikas(self) -> f32 {
        match self {
            Vaikeus::Helppo => 0.30,
            Vaikeus::Normaali => 0.22,
            Vaikeus::Vaikea => 0.16,
        }
    }

    /// Radiaania sekunnissa: lamppu on raskas.
    pub fn kaantonopeus(self) -> f32 {
        match self {
            Vaikeus::Helppo => 1.7,
            Vaikeus::Normaali => 1.3,
            Vaikeus::Vaikea => 1.05,
        }
    }

    /// Vaikealla sumu syö valon ennen meren kaukaisinta kulmaa.
    pub fn kantama(self) -> f32 {
        match self {
            Vaikeus::Helppo | Vaikeus::Normaali => 720.0,
            Vaikeus::Vaikea => 560.0,
        }
    }

    /// Kauanko kapteeni muistaa valaistut karikot.
    pub fn opastus_ms(self) -> u32 {
        match self {
            Vaikeus::Helppo => 2500,
            Vaikeus::Normaali => 1500,
            Vaikeus::Vaikea => 900,
        }
    }

    pub fn torven_lataus_ms(self) -> u32 {
        match self {
            Vaikeus::Helppo => 18_000,
            Vaikeus::Normaali => 24_000,
            Vaikeus::Vaikea => 30_000,
        }
    }

    pub fn karikoita(self, yo: u32) -> usize {
        let alku = match self {
            Vaikeus::Helppo => 7,
            Vaikeus::Normaali => 9,
            Vaikeus::Vaikea => 11,
        };
        (alku + yo as usize).min(ENITEN_KARIKOITA)
    }

    /// Laivojen tuloväli lyhenee yö yöltä.
    pub fn tulovali_ms(self, yo: u32) -> u32 {
        let alku = match self {
            Vaikeus::Helppo => 9000.0,
            Vaikeus::Normaali => 7500.0,
            Vaikeus::Vaikea => 6500.0,
        };
        ((alku * 0.88f32.powi(yo.min(20) as i32)) as u32).max(3000)
    }

    pub fn tuulinen(self, yo: u32) -> bool {
        match self {
            Vaikeus::Helppo => false,
            Vaikeus::Normaali => yo >= 2,
            Vaikeus::Vaikea => true,
        }
    }

    /// Merivirran voimakkuus, pikseliä sekunnissa.
    pub fn virta(self, yo: u32) -> f32 {
        match self {
            Vaikeus::Helppo => 0.0,
            Vaikeus::Normaali => (3 + yo).min(8) as f32,
            Vaikeus::Vaikea => (5 + yo).min(10) as f32,
        }
    }
}

/// Yhtä aikaa merellä olevien laivojen enimmäismäärä salakuljettajaa lukuun ottamatta.
pub fn enintaan_laivoja(yo: u32) -> usize {
    (3 + yo as usize).min(7)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Laji {
    Vene,
    Kalastaja,
    Rahtilaiva,
    /// Kulkee ilman valoja meren poikki eikä aja karille, koska tuntee vedet.
    Salakuljettaja,
}

impl Laji {
    pub fn sade(self) -> f32 {
        match self {
            Laji::Vene => 6.0,
            Laji::Kalastaja => 8.0,
            Laji::Rahtilaiva => 12.0,
            Laji::Salakuljettaja => 7.0,
        }
    }

    pub fn vauhti(self) -> f32 {
        match self {
            Laji::Vene => 34.0,
            Laji::Kalastaja => 28.0,
            Laji::Rahtilaiva => 21.0,
            Laji::Salakuljettaja => 32.0,
        }
    }

    /// Radiaania sekunnissa: rahtilaiva kääntyy hitaimmin.
    pub fn kaanto(self) -> f32 {
        match self {
            Laji::Vene => 1.8,
            Laji::Kalastaja => 1.3,
            Laji::Rahtilaiva => 0.7,
            Laji::Salakuljettaja => 1.6,
        }
    }

    pub fn pisteet(self) -> u32 {
        match self {
            Laji::Vene => 10,
            Laji::Kalastaja => 20,
            Laji::Rahtilaiva => 50,
            Laji::Salakuljettaja => 100,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Kari {
    pub paikka: Piste,
    pub sade: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Laiva {
    pub laji: Laji,
    pub paikka: Piste,
    pub suunta: f32,
    /// Laiva on keilassa tai päivänvalossa.
    pub valaistu: bool,
    /// Kapteeni muistaa valaistut karikot vielä näin kauan.
    pub opastus_ms: u32,
    /// Salakuljettajan aika keilassa.
    pub keilassa_ms: u32,
    /// Salakuljettajan määränpää vastarannalla.
    pub kohde: Piste,
    haluttu: f32,
    /// Laiva on käynyt kokonaan kentällä, joten reunan yli lähtö on eksymistä.
    sisalla: bool,
}

impl Laiva {
    pub fn uusi(laji: Laji, paikka: Piste, suunta: f32) -> Self {
        Self {
            laji,
            paikka,
            suunta,
            valaistu: false,
            opastus_ms: 0,
            keilassa_ms: 0,
            kohde: SATAMA,
            haluttu: suunta,
            sisalla: kentalla(paikka, laji.sade()),
        }
    }

    pub fn salakuljettaja(paikka: Piste, kohde: Piste) -> Self {
        let suunta = (kohde.1 - paikka.1).atan2(kohde.0 - paikka.0);
        Self {
            kohde,
            ..Self::uusi(Laji::Salakuljettaja, paikka, suunta)
        }
    }

    /// Kapteeni tietää karikot: salakuljettaja aina, muut keilan muistin varassa.
    pub fn tietaa(&self) -> bool {
        self.laji == Laji::Salakuljettaja || self.opastus_ms > 0
    }

    /// Satamaan pyrkivän laivan seuraava välietappi.
    fn etappi(&self) -> Piste {
        if self.laji == Laji::Salakuljettaja {
            return self.kohde;
        }
        let suulla = (self.paikka.0 - SATAMA.0).abs() < SUUN_PUOLIKAS - self.laji.sade();
        if suulla && self.paikka.1 >= ULAPPA.1 - 10.0 {
            (SATAMA.0, SATAMA.1 + 20.0)
        } else {
            ULAPPA
        }
    }
}

fn kentalla(p: Piste, vara: f32) -> bool {
    p.0 >= vara && p.0 <= LEVEYS - vara && p.1 >= vara
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Keila {
    pub kulma: f32,
    pub puolikas: f32,
    pub kantama: f32,
}

impl Keila {
    /// Osuuko keila `sade`-säteiseen kappaleeseen.
    pub fn valaisee(&self, p: Piste, sade: f32) -> bool {
        let (dx, dy) = (p.0 - MAJAKKA.0, p.1 - MAJAKKA.1);
        let d = dx.hypot(dy);
        if d > self.kantama + sade {
            return false;
        }
        if d < sade + 4.0 {
            return true;
        }
        kulmaero(dy.atan2(dx), self.kulma).abs() <= self.puolikas + (sade / d).min(1.0).asin()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Ohjaus {
    /// -1 vasemmalle, 1 oikealle, 0 ei kääntöä. Ohittaa tähtäimen.
    pub kaanto: i8,
    /// Kulma, johon keila kääntyy, kun nuolia ei paineta.
    pub tahtain: Option<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Puuska {
    /// Pimeässä kulkevan laivan kääntyminen, radiaania sekunnissa.
    pub kierto: f32,
    /// Tuulen työntö, pikseliä sekunnissa.
    pub tyonto: Piste,
    pub jaljella_ms: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vuorokausi {
    Yo,
    /// Aamu valaisee meren, ja merellä olevat laivat löytävät kotiin.
    Sarastus,
    /// Tauko ennen seuraavaa yötä.
    Aamu,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Tapahtuma {
    Satamassa {
        laji: Laji,
        pisteet: u32,
    },
    Karille {
        laji: Laji,
        paikka: Piste,
    },
    Eksyi {
        laji: Laji,
    },
    Kiinni {
        pisteet: u32,
        paikka: Piste,
    },
    Karkasi,
    Puuska,
    TorviLatautui,
    /// Yö päättyi: satamaan päässeet ja bonus, jos yksikään ei haaksirikkoutunut.
    Aamu {
        satamassa: u32,
        bonus: u32,
    },
    UusiYo,
    Ohi,
}

pub struct Meri {
    pub vaikeus: Vaikeus,
    /// Yön numero nollasta alkaen.
    pub yo: u32,
    pub vuorokausi: Vuorokausi,
    /// Yön kello. Sarastuksessa aika aamun alusta, aamulla jäljellä oleva tauko.
    pub aika_ms: u32,
    pub keila: Keila,
    pub laivat: Vec<Laiva>,
    pub karikot: Vec<Kari>,
    pub pisteet: u32,
    pub renkaat: u32,
    /// Peräkkäin perille päässeet laivat. Haaksirikko ja eksyminen nollaavat sen.
    pub ketju: u32,
    /// Sumutorven hidastusta jäljellä.
    pub torvi_ms: u32,
    /// Torven latautumista jäljellä.
    pub lataus_ms: u32,
    pub virta: Piste,
    pub puuska: Option<Puuska>,
    pub satamassa_yolla: u32,
    pub rikkoja_yolla: u32,
    pub ohi: bool,
    puuskan_viive_ms: u32,
    tulo_ms: u32,
    salakuljetus_ms: u32,
    askelia: u32,
}

impl Meri {
    /// Yö `yo` alkaa tyhjällä merellä.
    pub fn new(vaikeus: Vaikeus, yo: u32, arpa: &mut Arpa) -> Self {
        let mut meri = Self {
            vaikeus,
            yo,
            vuorokausi: Vuorokausi::Yo,
            aika_ms: 0,
            keila: Keila {
                kulma: -PI / 2.0,
                puolikas: vaikeus.keilan_puolikas(),
                kantama: vaikeus.kantama(),
            },
            laivat: Vec::new(),
            karikot: Vec::new(),
            pisteet: 0,
            renkaat: RENKAITA,
            ketju: 0,
            torvi_ms: 0,
            lataus_ms: 0,
            virta: (0.0, 0.0),
            puuska: None,
            satamassa_yolla: 0,
            rikkoja_yolla: 0,
            ohi: false,
            puuskan_viive_ms: 0,
            tulo_ms: 0,
            salakuljetus_ms: 0,
            askelia: 0,
        };
        meri.aloita_yo(yo, arpa);
        meri
    }

    fn aloita_yo(&mut self, yo: u32, arpa: &mut Arpa) {
        self.yo = yo;
        self.vuorokausi = Vuorokausi::Yo;
        self.aika_ms = 0;
        self.laivat.clear();
        self.karikot = karikot(self.vaikeus, yo, arpa);
        let voima = self.vaikeus.virta(yo);
        // Virta kulkee rannan suuntaisesti, vähän vinoon.
        let kulma =
            if arpa.alle(2) == 0 { 0.0 } else { PI } + (arpa.alle(61) as f32 - 30.0) / 100.0;
        self.virta = (kulma.cos() * voima, kulma.sin() * voima);
        self.puuska = None;
        self.puuskan_viive_ms = arvo(arpa, PUUSKAN_VALI_MS);
        self.satamassa_yolla = 0;
        self.rikkoja_yolla = 0;
        self.tulo_ms = ENSIMMAINEN_TULO_MS;
        self.salakuljetus_ms = arvo(arpa, SALAKULJETTAJAN_VALI_MS);
    }

    /// 0 on pimeä yö ja 1 täysi päivä. Ilta pimenee yön alussa ja aamu valkenee sarastaessa.
    pub fn valoisuus(&self) -> f32 {
        let osuus = |ms: u32| (ms as f32 / HAMARA_MS as f32).min(1.0);
        match self.vuorokausi {
            Vuorokausi::Yo => 1.0 - osuus(self.aika_ms),
            Vuorokausi::Sarastus => osuus(self.aika_ms),
            Vuorokausi::Aamu => 1.0,
        }
    }

    /// Kulunut osuus yöstä.
    pub fn yon_osuus(&self) -> f32 {
        match self.vuorokausi {
            Vuorokausi::Yo => self.aika_ms as f32 / YON_KESTO_MS as f32,
            _ => 1.0,
        }
    }

    /// Torven latauksen valmius 0..1.
    pub fn torven_valmius(&self) -> f32 {
        1.0 - self.lataus_ms as f32 / self.vaikeus.torven_lataus_ms() as f32
    }

    /// Satamaan pääsevän laivan pistekerroin peräkkäisistä onnistumisista.
    pub fn ketjukerroin(&self) -> u32 {
        1 + self.ketju.min(12) / 3
    }

    /// Sumutorvi hidastaa kaikkia laivoja hetkeksi. Palauttaa tiedon, soiko torvi.
    pub fn soita_torvi(&mut self) -> bool {
        if self.ohi || self.lataus_ms > 0 || self.vuorokausi == Vuorokausi::Aamu {
            return false;
        }
        self.torvi_ms = TORVEN_KESTO_MS;
        self.lataus_ms = self.vaikeus.torven_lataus_ms();
        true
    }

    fn lisaa_pisteet(&mut self, pisteet: u32) {
        self.pisteet = self.pisteet.saturating_add(pisteet).min(SUURIN_PISTEMAARA);
    }

    pub fn askel(&mut self, ohjaus: Ohjaus, arpa: &mut Arpa) -> Vec<Tapahtuma> {
        let mut tapahtumat = Vec::new();
        if self.ohi {
            return tapahtumat;
        }
        self.askelia = self.askelia.wrapping_add(1);
        self.kaanna_keila(ohjaus);
        self.ajastimet(arpa, &mut tapahtumat);
        match self.vuorokausi {
            Vuorokausi::Yo => {
                self.aika_ms += ASKEL_MS;
                if self.aika_ms < TULO_LOPPUU_MS {
                    self.tuo_laivoja(arpa);
                }
                if self.aika_ms >= YON_KESTO_MS {
                    self.vuorokausi = Vuorokausi::Sarastus;
                    self.aika_ms = 0;
                    self.puuska = None;
                }
            }
            Vuorokausi::Sarastus => {
                self.aika_ms += ASKEL_MS;
                let kotona = self.laivat.is_empty() && self.aika_ms >= HAMARA_MS;
                if kotona || self.aika_ms >= SARASTUKSEN_ENIMMAISAIKA_MS {
                    tapahtumat.push(self.paata_yo());
                }
            }
            Vuorokausi::Aamu => {
                self.aika_ms = self.aika_ms.saturating_sub(ASKEL_MS);
                if self.aika_ms == 0 {
                    self.aloita_yo(self.yo + 1, arpa);
                    tapahtumat.push(Tapahtuma::UusiYo);
                }
            }
        }
        self.liikuta_laivoja(&mut tapahtumat);
        tapahtumat
    }

    fn paata_yo(&mut self) -> Tapahtuma {
        self.laivat.clear();
        let bonus = if self.rikkoja_yolla == 0 && self.satamassa_yolla > 0 {
            100 * (self.yo + 1) * self.vaikeus.kerroin()
        } else {
            0
        };
        self.lisaa_pisteet(bonus);
        self.vuorokausi = Vuorokausi::Aamu;
        self.aika_ms = AAMUN_TAUKO_MS;
        Tapahtuma::Aamu {
            satamassa: self.satamassa_yolla,
            bonus,
        }
    }

    fn kaanna_keila(&mut self, ohjaus: Ohjaus) {
        let suurin = self.vaikeus.kaantonopeus() * ASKEL_S;
        let muutos = if ohjaus.kaanto != 0 {
            ohjaus.kaanto.signum() as f32 * suurin
        } else if let Some(tahtain) = ohjaus.tahtain {
            (tahtain - self.keila.kulma).clamp(-suurin, suurin)
        } else {
            0.0
        };
        self.keila.kulma = (self.keila.kulma + muutos).clamp(KEILAN_KULMAT.0, KEILAN_KULMAT.1);
    }

    fn ajastimet(&mut self, arpa: &mut Arpa, tapahtumat: &mut Vec<Tapahtuma>) {
        self.torvi_ms = self.torvi_ms.saturating_sub(ASKEL_MS);
        if self.lataus_ms > 0 {
            self.lataus_ms = self.lataus_ms.saturating_sub(ASKEL_MS);
            if self.lataus_ms == 0 {
                tapahtumat.push(Tapahtuma::TorviLatautui);
            }
        }
        if let Some(puuska) = &mut self.puuska {
            puuska.jaljella_ms = puuska.jaljella_ms.saturating_sub(ASKEL_MS);
            if puuska.jaljella_ms == 0 {
                self.puuska = None;
            }
        }
        if self.vuorokausi != Vuorokausi::Yo || !self.vaikeus.tuulinen(self.yo) {
            return;
        }
        self.puuskan_viive_ms = self.puuskan_viive_ms.saturating_sub(ASKEL_MS);
        if self.puuskan_viive_ms == 0 {
            self.puuskan_viive_ms = arvo(arpa, PUUSKAN_VALI_MS);
            let kierto = (25 + arpa.alle(21)) as f32 / 100.0;
            let kierto = if arpa.alle(2) == 0 { kierto } else { -kierto };
            let kulma = arpa.alle(360) as f32 / 360.0 * TAU;
            self.puuska = Some(Puuska {
                kierto,
                tyonto: (kulma.cos() * PUUSKAN_TYONTO, kulma.sin() * PUUSKAN_TYONTO),
                jaljella_ms: PUUSKAN_KESTO_MS,
            });
            tapahtumat.push(Tapahtuma::Puuska);
        }
    }

    fn tuo_laivoja(&mut self, arpa: &mut Arpa) {
        self.tulo_ms = self.tulo_ms.saturating_sub(ASKEL_MS);
        let tavallisia = self
            .laivat
            .iter()
            .filter(|l| l.laji != Laji::Salakuljettaja)
            .count();
        if self.tulo_ms == 0 && tavallisia < enintaan_laivoja(self.yo) {
            self.tulo_ms = self.vaikeus.tulovali_ms(self.yo);
            if let Some(laiva) = self.uusi_laiva(arpa) {
                self.laivat.push(laiva);
            }
        }
        if self.yo < SALAKULJETUS_ALKAA {
            return;
        }
        self.salakuljetus_ms = self.salakuljetus_ms.saturating_sub(ASKEL_MS);
        let salakuljettaja = self.laivat.iter().any(|l| l.laji == Laji::Salakuljettaja);
        if self.salakuljetus_ms == 0 && !salakuljettaja {
            self.salakuljetus_ms = arvo(arpa, SALAKULJETTAJAN_VALI_MS);
            let (vasen, oikea) = (
                (-8.0, (60 + arpa.alle(271)) as f32),
                (LEVEYS + 8.0, (60 + arpa.alle(271)) as f32),
            );
            let laiva = if arpa.alle(2) == 0 {
                Laiva::salakuljettaja(vasen, (oikea.0 + 20.0, oikea.1))
            } else {
                Laiva::salakuljettaja(oikea, (vasen.0 - 20.0, vasen.1))
            };
            self.laivat.push(laiva);
        }
    }

    /// Laiva tulee ylälaidasta tai sivuilta ja suuntaa suunnilleen kohti satamaa.
    fn uusi_laiva(&self, arpa: &mut Arpa) -> Option<Laiva> {
        let rahti = (10 + 6 * self.yo).min(40) as usize;
        let arpa_laji = arpa.alle(80 + rahti);
        let laji = if arpa_laji < 45 {
            Laji::Vene
        } else if arpa_laji < 80 {
            Laji::Kalastaja
        } else {
            Laji::Rahtilaiva
        };
        for _ in 0..8 {
            let paikka = match arpa.alle(10) {
                0..=5 => ((40 + arpa.alle(581)) as f32, -8.0),
                6 | 7 => (-8.0, (20 + arpa.alle(201)) as f32),
                _ => (LEVEYS + 8.0, (20 + arpa.alle(201)) as f32),
            };
            let vapaa = self
                .laivat
                .iter()
                .all(|l| etaisyys(l.paikka, paikka) > 40.0);
            if vapaa {
                let kohti = (ULAPPA.1 - paikka.1).atan2(ULAPPA.0 - paikka.0);
                let virhe = (arpa.alle(201) as f32 / 100.0 - 1.0) * SUUNNAN_VIRHE;
                return Some(Laiva::uusi(laji, paikka, kohti + virhe));
            }
        }
        None
    }

    fn liikuta_laivoja(&mut self, tapahtumat: &mut Vec<Tapahtuma>) {
        let paivalla = self.vuorokausi != Vuorokausi::Yo;
        let mieti = self.askelia.is_multiple_of(OHJAUKSEN_VALI);
        let hidastus = if self.torvi_ms > 0 {
            TORVEN_HIDASTUS
        } else {
            1.0
        };
        let opastus = self.vaikeus.opastus_ms();
        let mut i = 0;
        while i < self.laivat.len() {
            let mut laiva = self.laivat[i];
            let laji = laiva.laji;
            let keilassa = self.keila.valaisee(laiva.paikka, laji.sade());
            laiva.valaistu = paivalla || keilassa;
            if laiva.valaistu {
                laiva.opastus_ms = opastus;
            }
            if laji == Laji::Salakuljettaja {
                // Kiinni saa vain keilalla. Keilasta päässyt toipuu hitaammin kuin jää kiinni.
                laiva.keilassa_ms = if keilassa && !paivalla {
                    laiva.keilassa_ms + ASKEL_MS
                } else {
                    laiva.keilassa_ms.saturating_sub(2)
                };
                if laiva.keilassa_ms >= KIINNIOTTO_MS {
                    let pisteet = laji.pisteet() * self.vaikeus.kerroin();
                    self.lisaa_pisteet(pisteet);
                    tapahtumat.push(Tapahtuma::Kiinni {
                        pisteet,
                        paikka: laiva.paikka,
                    });
                    self.laivat.remove(i);
                    continue;
                }
            }
            let tyonto = self.puuska.map_or((0.0, 0.0), |p| p.tyonto);
            if laiva.tietaa() {
                if mieti {
                    let ajo = (self.virta.0 + tyonto.0, self.virta.1 + tyonto.1);
                    laiva.haluttu = ohjaa(&laiva, &self.karikot, ajo);
                }
                let suurin = laji.kaanto() * ASKEL_S;
                laiva.suunta += kulmaero(laiva.haluttu, laiva.suunta).clamp(-suurin, suurin);
                laiva.opastus_ms = laiva.opastus_ms.saturating_sub(ASKEL_MS);
            } else if let Some(puuska) = self.puuska {
                laiva.suunta += puuska.kierto * ASKEL_S;
            }
            let vauhti = laji.vauhti() * hidastus;
            laiva.paikka.0 += (laiva.suunta.cos() * vauhti + self.virta.0 + tyonto.0) * ASKEL_S;
            laiva.paikka.1 += (laiva.suunta.sin() * vauhti + self.virta.1 + tyonto.1) * ASKEL_S;
            if !laiva.sisalla && kentalla(laiva.paikka, laji.sade()) {
                laiva.sisalla = true;
            }
            self.laivat[i] = laiva;
            match self.laivan_kohtalo(&laiva) {
                None => i += 1,
                Some(tapahtuma) => {
                    self.laivat.remove(i);
                    self.kirjaa(tapahtuma, tapahtumat);
                    if self.ohi {
                        return;
                    }
                }
            }
        }
    }

    /// Laiva pääsi perille, haaksirikkoutui tai lähti kentältä.
    fn laivan_kohtalo(&self, laiva: &Laiva) -> Option<Tapahtuma> {
        let laji = laiva.laji;
        let (x, y) = laiva.paikka;
        let ulkona = !(-25.0..=LEVEYS + 25.0).contains(&x) || y < -25.0;
        if laji == Laji::Salakuljettaja {
            let perilla = etaisyys(laiva.paikka, laiva.kohde) < 12.0;
            return (perilla || (laiva.sisalla && ulkona)).then_some(Tapahtuma::Karkasi);
        }
        if suulla(x) && y >= SATAMA.1 - 8.0 {
            return Some(Tapahtuma::Satamassa { laji, pisteet: 0 });
        }
        let osuma = laji.sade() * OSUMAN_OSUUS;
        let karilla = self
            .karikot
            .iter()
            .any(|k| etaisyys(k.paikka, laiva.paikka) < k.sade + osuma);
        let rannalla = !suulla(x) && y + osuma > ranta_y(x);
        if karilla || rannalla || y > KORKEUS {
            return Some(Tapahtuma::Karille {
                laji,
                paikka: laiva.paikka,
            });
        }
        (laiva.sisalla && ulkona).then_some(Tapahtuma::Eksyi { laji })
    }

    fn kirjaa(&mut self, tapahtuma: Tapahtuma, tapahtumat: &mut Vec<Tapahtuma>) {
        let tapahtuma = match tapahtuma {
            Tapahtuma::Satamassa { laji, .. } => {
                let pisteet = laji.pisteet() * self.ketjukerroin() * self.vaikeus.kerroin();
                self.lisaa_pisteet(pisteet);
                self.ketju += 1;
                self.satamassa_yolla += 1;
                Tapahtuma::Satamassa { laji, pisteet }
            }
            Tapahtuma::Karille { .. } => {
                self.ketju = 0;
                self.rikkoja_yolla += 1;
                self.renkaat = self.renkaat.saturating_sub(1);
                if self.renkaat == 0 {
                    self.ohi = true;
                    tapahtumat.push(tapahtuma);
                    tapahtumat.push(Tapahtuma::Ohi);
                    return;
                }
                tapahtuma
            }
            Tapahtuma::Eksyi { .. } => {
                self.ketju = 0;
                tapahtuma
            }
            muu => muu,
        };
        tapahtumat.push(tapahtuma);
    }
}

fn arvo(arpa: &mut Arpa, (alin, ylin): (usize, usize)) -> u32 {
    (alin + arpa.alle(ylin - alin + 1)) as u32
}

/// Pääseekö laiva matkan `pituus` osumatta karikkoon tai rantaan, kun se kääntää keulaa kohti
/// suuntaa `kulma`, ja reitin pienin väli karikkoon. Reitti lasketaan laivan omalla
/// kääntönopeudella ja virran kanssa, koska hidas rahtilaiva kulkee leveän kaaren. Turvavälin
/// sisälle ajautuneelle kelpaa reitti, joka ei vie karikkoa lähemmäs.
fn reitin_vali(
    laiva: &Laiva,
    kulma: f32,
    pituus: f32,
    karikot: &[Kari],
    virta: Piste,
) -> (bool, f32) {
    let laji = laiva.laji;
    let sade = laji.sade();
    let vauhti = laji.vauhti();
    let ds = REITIN_ASKEL;
    let dt = ds / vauhti;
    let kaanto = laji.kaanto() * dt;
    let (mut p, mut suunta) = (laiva.paikka, laiva.suunta);
    let alussa: Vec<f32> = karikot
        .iter()
        .map(|k| etaisyys(k.paikka, p) - 0.5)
        .collect();
    let mut vapaa = true;
    let mut pienin = f32::MAX;
    for _ in 0..(pituus / ds).ceil() as usize {
        suunta += kulmaero(kulma, suunta).clamp(-kaanto, kaanto);
        p.0 += (suunta.cos() * vauhti + virta.0) * dt;
        p.1 += (suunta.sin() * vauhti + virta.1) * dt;
        if !suulla(p.0) && p.1 + sade + 4.0 >= ranta_y(p.0) {
            return (false, f32::MIN);
        }
        for (kari, &alku) in karikot.iter().zip(&alussa) {
            let d = etaisyys(kari.paikka, p);
            pienin = pienin.min(d - kari.sade - sade);
            if d < kari.sade + sade + VARA && d < alku {
                vapaa = false;
            }
        }
    }
    (vapaa, pienin)
}

/// Kapteenin valitsema suunta: lähin vapaa suunta kohti seuraavaa etappia. Virta korjataan
/// ohjaamalla vähän sitä vastaan.
fn ohjaa(laiva: &Laiva, karikot: &[Kari], virta: Piste) -> f32 {
    let kohde = laiva.etappi();
    let (dx, dy) = (kohde.0 - laiva.paikka.0, kohde.1 - laiva.paikka.1);
    let matka = dx.hypot(dy).max(0.001);
    let vauhti = laiva.laji.vauhti();
    let suora = (dy / matka * vauhti - virta.1).atan2(dx / matka * vauhti - virta.0);
    let pituus = NAKO.min(matka + 20.0);
    // Puolta ei vaihdeta kesken väistön: hidas laiva ei ehtisi kääntyä toiselle puolelle.
    // Edellinen suunta pidetään, kun se on vapaa eikä vie selvästi pidempää reittiä.
    let edellinen = suora + kulmaero(laiva.haluttu, suora);
    let edellinen_vapaa = reitin_vali(laiva, edellinen, pituus, karikot, virta).0;
    let puoli = if kulmaero(laiva.haluttu, suora) >= 0.0 {
        1.0
    } else {
        -1.0
    };
    // Ellei vapaata suuntaa ole, väljin on paras.
    let mut paras = (f32::MIN, suora);
    for k in 0..=KOKEILUJA {
        for merkki in [puoli, -puoli] {
            let kulma = suora + merkki * k as f32 * KOKEILUN_ASKEL;
            let (vapaa, vali) = reitin_vali(laiva, kulma, pituus, karikot, virta);
            if vapaa {
                let poikkeama = kulmaero(edellinen, suora).abs();
                if edellinen_vapaa && poikkeama <= k as f32 * KOKEILUN_ASKEL + PITO {
                    return edellinen;
                }
                return kulma;
            }
            if vali > paras.0 {
                paras = (vali, kulma);
            }
            if k == 0 {
                break;
            }
        }
    }
    paras.1
}

/// Uuden yön karikot. Karikoiden välistä mahtuu kulkemaan, eikä niitä ole sataman edessä
/// eikä majakan juurella.
pub fn karikot(vaikeus: Vaikeus, yo: u32, arpa: &mut Arpa) -> Vec<Kari> {
    let maara = vaikeus.karikoita(yo);
    let mut karikot: Vec<Kari> = Vec::with_capacity(maara);
    for _ in 0..5000 {
        if karikot.len() == maara {
            break;
        }
        let sade = (9 + arpa.alle(10)) as f32;
        let paikka = ((50 + arpa.alle(561)) as f32, (80 + arpa.alle(331)) as f32);
        let kari = Kari { paikka, sade };
        if karikko_kelpaa(&kari, &karikot) {
            karikot.push(kari);
        }
    }
    karikot
}

pub fn karikko_kelpaa(kari: &Kari, muut: &[Kari]) -> bool {
    let (p, sade) = (kari.paikka, kari.sade);
    let valit = muut
        .iter()
        .all(|k| etaisyys(k.paikka, p) >= k.sade + sade + VAYLA);
    let ulappa = etaisyys(p, ULAPPA) >= sade + 70.0;
    let vayla = !((p.0 - SATAMA.0).abs() < sade + 50.0 && p.1 > 350.0);
    let majakka = etaisyys(p, MAJAKKA) >= sade + 60.0;
    valit && ulappa && vayla && majakka
}
