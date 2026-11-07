//! Kaivo8000:n säännöt ilman ikkunaa, piirtoa tai syötettä. Kaivos on ruudukko, jonka rivi 0 on
//! taivasta ja rivi `PINTA` maanpintaa, jolla hissi seisoo. Aika etenee kiinteinä askelina, joten
//! sama siemen ja samat painallukset toistavat saman pelin.
//!
//! Maa kantaa itseään: kiinteä ruutu on tuettu, kun sen alla on tuettu ruutu (pilari), tai kun
//! samalla rivillä on pilari enintään materiaalin kantavuuden päässä yhtenäistä kiinteää pitkin.
//! Tukematon ruutu ritisee hetken ja putoaa sitten. Pelaajan tuki on pilari, jota pitkin voi
//! myös kiivetä.

use crate::Arpa;

pub const SARAKKEITA: usize = 22;
pub const RIVEJA: usize = 18;
/// Maanpinnan rivi: kaivaja kävelee tällä rivillä, ja maa alkaa sen alta.
pub const PINTA: i32 = 1;
pub const MAA_ALKAA: i32 = 2;
/// Hissi seisoo pinnalla vasemmassa reunassa.
pub const HISSI: Ruutu = (1, PINTA);
/// Kaivaja aloittaa ja palaa tähän sortuman jälkeen.
pub const ALKU: Ruutu = (3, PINTA);
pub const ASKEL_MS: u32 = 5;
pub const KYPARIA: u32 = 3;
/// Kuusi numeroa mahtuu sivupalkkiin, ja nettilista hylkää tätä suuremmat tulokset.
pub const SUURIN_PISTEMAARA: u32 = 999_999;
pub const KAVELY_MS: u32 = 120;
pub const KIIPEAMINEN_MS: u32 = 170;
/// Putoava ruutu ja kaivaja putoavat yhden ruudun näin nopeasti.
pub const PUTOAMINEN_MS: u32 = 70;
/// Näin monen ruudun pudotus ei vielä satu.
pub const TURVALLINEN_PUDOTUS: u32 = 4;
/// Hissi vie seuraavalle tasolle näin kauan.
pub const HISSIN_MATKA_MS: u32 = 2500;
/// Pinnalla lamppu täyttyy näin monta kertaa nopeammin kuin se palaa maan alla.
const TAYTTOKERROIN: u32 = 8;
/// Öljykannu täyttää lampusta tämän osan.
const KANNUN_OSUUS: f32 = 0.4;
pub const PUITA_NIPUSSA: u32 = 4;
/// Valon säde ruutuina, kun öljy on lopussa.
pub const PIENIN_VALO: f32 = 1.3;

pub type Ruutu = (i32, i32);

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

    pub fn tukia(self) -> u32 {
        match self {
            Vaikeus::Helppo => 20,
            Vaikeus::Normaali => 15,
            Vaikeus::Vaikea => 11,
        }
    }

    /// Täysi lamppu palaa maan alla näin kauan.
    pub fn oljya_ms(self) -> u32 {
        match self {
            Vaikeus::Helppo => 150_000,
            Vaikeus::Normaali => 110_000,
            Vaikeus::Vaikea => 80_000,
        }
    }

    /// Valon säde ruutuina täydellä lampulla.
    pub fn suurin_valo(self) -> f32 {
        match self {
            Vaikeus::Helppo => 5.5,
            Vaikeus::Normaali => 4.8,
            Vaikeus::Vaikea => 4.2,
        }
    }

    /// Ritinän kesto prosentteina: helpolla katto kestää pidempään.
    fn ritinan_prosentti(self) -> u32 {
        match self {
            Vaikeus::Helppo => 140,
            Vaikeus::Normaali => 100,
            Vaikeus::Vaikea => 75,
        }
    }

    /// Hiekkaa ja kiviä on vaikeammalla enemmän.
    fn vaaroja(self) -> usize {
        match self {
            Vaikeus::Helppo => 0,
            Vaikeus::Normaali => 1,
            Vaikeus::Vaikea => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Solu {
    Tyhja,
    Maa,
    /// Sitkeä savi kantaa leveämmän katon, mutta sen kaivaminen kestää.
    Savi,
    /// Hiekka ei kanna mitään: alta kaivettu hiekka valuu heti.
    Hiekka,
    /// Kiveä ei voi kaivaa, ja se putoaa, kun sen alta kaivetaan.
    Kivi,
    /// Peruskallio ei kaivaudu eikä putoa.
    Kallio,
    /// Pelaajan tuki: pilari, joka kantaa kattoa ja jota pitkin kiivetään.
    Tuki,
    Kulta,
    Timantti,
    /// Haudattu öljykannu täyttää lamppua.
    Oljy,
    /// Haudattu puunippu antaa lisää tukia.
    Puut,
}

impl Solu {
    pub fn kiintea(self) -> bool {
        self != Solu::Tyhja
    }

    /// Kaivaja voi olla tässä ruudussa.
    pub fn lapaistava(self) -> bool {
        matches!(self, Solu::Tyhja | Solu::Tuki)
    }

    pub fn kaivettava(self) -> bool {
        matches!(
            self,
            Solu::Maa
                | Solu::Savi
                | Solu::Hiekka
                | Solu::Kulta
                | Solu::Timantti
                | Solu::Oljy
                | Solu::Puut
        )
    }

    pub fn kaivuun_kesto(self) -> u32 {
        match self {
            Solu::Hiekka => 100,
            Solu::Maa | Solu::Oljy | Solu::Puut => 170,
            Solu::Kulta => 260,
            Solu::Savi => 380,
            Solu::Timantti => 450,
            _ => 0,
        }
    }

    /// Montako ruutua sivulle ruutu kantaa itseään ilman pilaria.
    pub fn kantavuus(self) -> u32 {
        match self {
            Solu::Kallio => u32::MAX,
            Solu::Savi => 3,
            Solu::Maa | Solu::Kulta | Solu::Timantti | Solu::Oljy | Solu::Puut => 2,
            Solu::Tyhja | Solu::Hiekka | Solu::Kivi | Solu::Tuki => 0,
        }
    }

    /// Ritinän kesto normaalilla tasolla ennen kuin tukematon ruutu putoaa.
    pub fn ritinan_kesto(self) -> u32 {
        match self {
            Solu::Hiekka => 150,
            Solu::Tuki => 400,
            Solu::Kivi => 550,
            Solu::Savi => 2000,
            _ => 1400,
        }
    }
}

/// Kaivajan meneillään oleva toimi.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Toimi {
    Seisoo,
    /// Siirtyy viereiseen ruutuun suuntaan (sarake, rivi).
    Liikkuu(i8, i8),
    Kaivaa(i8, i8),
    Putoaa,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Kaivaja {
    pub paikka: Ruutu,
    pub toimi: Toimi,
    /// Toimen kulunut aika.
    pub ms: u32,
    /// -1 vasemmalle, 1 oikealle.
    pub katse: i8,
    /// Pudotut ruudut ennen alastuloa.
    pub pudotus: u32,
}

impl Kaivaja {
    fn new(paikka: Ruutu) -> Self {
        Self {
            paikka,
            toimi: Toimi::Seisoo,
            ms: 0,
            katse: 1,
            pudotus: 0,
        }
    }

    /// Toimen kesto, jotta näkymä voi liu'uttaa kaivajaa ruudusta toiseen.
    pub fn kesto(&self, solut: &Solut) -> u32 {
        match self.toimi {
            Toimi::Seisoo => 1,
            Toimi::Liikkuu(_, 0) => KAVELY_MS,
            Toimi::Liikkuu(..) => KIIPEAMINEN_MS,
            Toimi::Kaivaa(dx, dy) => {
                let kohde = (self.paikka.0 + dx as i32, self.paikka.1 + dy as i32);
                solu(solut, kohde).kaivuun_kesto().max(1)
            }
            Toimi::Putoaa => PUTOAMINEN_MS,
        }
    }

    /// Kaivajan paikka ruutuina liikkeen aikana.
    pub fn sijainti(&self, solut: &Solut) -> (f32, f32) {
        let osuus = (self.ms as f32 / self.kesto(solut) as f32).min(1.0);
        let (x, y) = (self.paikka.0 as f32, self.paikka.1 as f32);
        match self.toimi {
            Toimi::Liikkuu(dx, dy) => (x + dx as f32 * osuus, y + dy as f32 * osuus),
            Toimi::Putoaa => (x, y + osuus),
            _ => (x, y),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Putoava {
    pub solu: Solu,
    pub paikka: Ruutu,
    /// Aika siitä, kun ruutu tuli nykyiseen ruutuunsa.
    pub ms: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Loyto {
    Kulta(u32),
    Timantti(u32),
    Oljy,
    Puut(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tapahtuma {
    Kaivoi(Solu),
    Loysi {
        loyto: Loyto,
        paikka: Ruutu,
    },
    /// Säkissä on nyt tarpeeksi tason kiintiöön.
    KiintioTaynna,
    /// Uusi ruutu alkoi ritistä.
    Ritisee(Ruutu),
    /// Putoava ruutu osui pohjaan.
    Sortui {
        solu: Solu,
        paikka: Ruutu,
    },
    /// Kaivaja jäi putoavan ruudun alle ja menetti säkin.
    Murskautui {
        paikka: Ruutu,
        menetetty: u32,
    },
    /// Kaivaja putosi liian korkealta ja menetti säkin.
    Putosi {
        paikka: Ruutu,
        menetetty: u32,
    },
    /// Hätäköysi nosti kaivajan pinnalle, säkki jäi kuoppaan.
    Koysi {
        menetetty: u32,
    },
    /// Säkki tyhjennettiin hissiin.
    Nosti {
        pisteet: u32,
    },
    TasoValmis {
        bonus: u32,
    },
    UusiTaso,
    OljyLoppui,
    Ohi,
}

pub type Solut = [[Solu; SARAKKEITA]; RIVEJA];

pub fn alueella((x, y): Ruutu) -> bool {
    (0..SARAKKEITA as i32).contains(&x) && (0..RIVEJA as i32).contains(&y)
}

/// Ruutu kentän ulkopuolella on kalliota.
pub fn solu(solut: &Solut, (x, y): Ruutu) -> Solu {
    if alueella((x, y)) {
        solut[y as usize][x as usize]
    } else {
        Solu::Kallio
    }
}

/// Kullan arvo ennen tason kerrointa: syvemmällä kulta on arvokkaampaa.
pub fn kullan_arvo(rivi: i32) -> u32 {
    10 + 5 * ((rivi - MAA_ALKAA).max(0) as u32 / 3)
}

pub fn timantin_arvo(taso: u32) -> u32 {
    100 + 25 * taso.min(8)
}

/// Kantaako maa itsensä. Pohjarivi ja kentän reunat ovat kalliota.
pub fn tuenta(solut: &Solut) -> [[bool; SARAKKEITA]; RIVEJA] {
    let mut tuettu = [[false; SARAKKEITA]; RIVEJA];
    const KAUKANA: u32 = u32::MAX / 2;
    for y in (0..RIVEJA).rev() {
        let rivi = &solut[y];
        let pilari: [bool; SARAKKEITA] = std::array::from_fn(|x| {
            let s = rivi[x];
            s == Solu::Kallio
                || (s.kiintea()
                    && (y + 1 == RIVEJA || (solut[y + 1][x].kiintea() && tuettu[y + 1][x])))
        });
        // Etäisyys lähimpään pilariin yhtenäistä kiinteää pitkin, kumpaankin suuntaan.
        // Kentän reuna on kalliota, joten reunassa kiinni oleva ruutu on yhden päässä pilarista.
        let mut etaisyys = [KAUKANA; SARAKKEITA];
        let mut edellinen = 1;
        for x in 0..SARAKKEITA {
            edellinen = if !rivi[x].kiintea() {
                KAUKANA
            } else if pilari[x] {
                0
            } else {
                edellinen.saturating_add(1).min(KAUKANA)
            };
            etaisyys[x] = edellinen;
        }
        edellinen = 1;
        for x in (0..SARAKKEITA).rev() {
            edellinen = if !rivi[x].kiintea() {
                KAUKANA
            } else if pilari[x] {
                0
            } else {
                edellinen.saturating_add(1).min(KAUKANA)
            };
            etaisyys[x] = etaisyys[x].min(edellinen);
        }
        for x in 0..SARAKKEITA {
            let s = rivi[x];
            tuettu[y][x] = s.kiintea() && (pilari[x] || etaisyys[x] <= s.kantavuus());
        }
    }
    tuettu
}

pub struct Kaivos {
    pub vaikeus: Vaikeus,
    /// Taso nollasta alkaen.
    pub taso: u32,
    pub solut: Solut,
    /// Ritinää jäljellä millisekunteina, 0 on vakaa.
    pub ritina: [[u32; SARAKKEITA]; RIVEJA],
    pub putoavat: Vec<Putoava>,
    pub kaivaja: Kaivaja,
    pub pisteet: u32,
    /// Säkissä olevat pisteet. Ne kirjautuvat vasta hissillä ja katoavat sortumassa.
    pub saalis: u32,
    /// Tällä tasolla hissiin viedyt pisteet.
    pub nostettu: u32,
    pub kiintio: u32,
    pub tuet: u32,
    pub kyparat: u32,
    pub oljy_ms: u32,
    /// Kasvaa aina, kun jokin ruutu muuttuu: näkymä laskee kaivoksen kuvan silloin uudelleen.
    pub versio: u32,
    /// Hissi on matkalla seuraavalle tasolle, ja matkaa on jäljellä näin kauan.
    pub hissi_ms: Option<u32>,
    pub ohi: bool,
    /// Viimeisin painallus odottaa, kunnes kaivaja on vapaa.
    painallus: Option<(i8, i8)>,
    tuenta_vanha: bool,
    kiintio_ilmoitettu: bool,
}

impl Kaivos {
    pub fn new(vaikeus: Vaikeus, taso: u32, arpa: &mut Arpa) -> Self {
        let mut kaivos = Self {
            vaikeus,
            taso,
            solut: [[Solu::Tyhja; SARAKKEITA]; RIVEJA],
            ritina: [[0; SARAKKEITA]; RIVEJA],
            putoavat: Vec::new(),
            kaivaja: Kaivaja::new(ALKU),
            pisteet: 0,
            saalis: 0,
            nostettu: 0,
            kiintio: 0,
            tuet: vaikeus.tukia(),
            kyparat: KYPARIA,
            oljy_ms: vaikeus.oljya_ms(),
            versio: 0,
            hissi_ms: None,
            ohi: false,
            painallus: None,
            tuenta_vanha: true,
            kiintio_ilmoitettu: false,
        };
        kaivos.luo_taso(arpa);
        kaivos
    }

    /// Uusi kenttä tasolle `self.taso`. Pisteet ja kypärät säilyvät.
    fn luo_taso(&mut self, arpa: &mut Arpa) {
        self.solut = luo_kentta(self.vaikeus, self.taso, arpa);
        self.ritina = [[0; SARAKKEITA]; RIVEJA];
        self.putoavat.clear();
        self.kaivaja = Kaivaja::new(ALKU);
        self.saalis = 0;
        self.nostettu = 0;
        self.tuet = self.vaikeus.tukia();
        self.oljy_ms = self.vaikeus.oljya_ms();
        self.painallus = None;
        self.tuenta_vanha = true;
        self.kiintio_ilmoitettu = false;
        let kultaa: u32 = (0..RIVEJA as i32)
            .flat_map(|y| (0..SARAKKEITA as i32).map(move |x| (x, y)))
            .filter(|&p| solu(&self.solut, p) == Solu::Kulta)
            .map(|(_, y)| self.arvo(kullan_arvo(y)))
            .sum();
        let osuus = (40 + 4 * self.taso.min(7)) as f32 / 100.0;
        let kymmenet = 10 * self.vaikeus.kerroin();
        self.kiintio = ((kultaa as f32 * osuus) as u32 / kymmenet * kymmenet).max(kymmenet);
        self.versio = self.versio.wrapping_add(1);
    }

    fn arvo(&self, perus: u32) -> u32 {
        perus * self.vaikeus.kerroin()
    }

    pub fn solu(&self, paikka: Ruutu) -> Solu {
        solu(&self.solut, paikka)
    }

    /// Vaihtaa ruudun. Tuenta lasketaan seuraavalla askeleella uudelleen.
    pub fn aseta(&mut self, (x, y): Ruutu, uusi: Solu) {
        if alueella((x, y)) {
            self.solut[y as usize][x as usize] = uusi;
            self.ritina[y as usize][x as usize] = 0;
            self.tuenta_vanha = true;
            self.versio = self.versio.wrapping_add(1);
        }
    }

    /// Lampun öljyä jäljellä 0..1.
    pub fn oljyn_osuus(&self) -> f32 {
        self.oljy_ms as f32 / self.vaikeus.oljya_ms() as f32
    }

    /// Lampun valon säde ruutuina. Se kutistuu öljyn mukana.
    pub fn valon_sade(&self) -> f32 {
        let suurin = self.vaikeus.suurin_valo();
        PIENIN_VALO + (suurin - PIENIN_VALO) * self.oljyn_osuus().sqrt()
    }

    pub fn putoava_kohdassa(&self, paikka: Ruutu) -> bool {
        self.putoavat.iter().any(|p| p.paikka == paikka)
    }

    /// Kiintiö on täynnä, kun hissiin viety ja säkissä oleva riittävät.
    pub fn kiintio_taynna(&self) -> bool {
        self.nostettu + self.saalis >= self.kiintio
    }

    /// Nuoli tai klikkaus: kaivaja toimii heti, kun se on vapaa.
    pub fn paina(&mut self, suunta: (i8, i8)) {
        if suunta != (0, 0) {
            self.painallus = Some(suunta);
        }
    }

    /// Asettaa tuen kaivajan ruutuun tai purkaa siinä olevan. Tosi, jos jotain tapahtui.
    pub fn vaihda_tuki(&mut self) -> bool {
        let k = self.kaivaja;
        if self.ohi || self.hissi_ms.is_some() || k.toimi != Toimi::Seisoo {
            return false;
        }
        match self.solu(k.paikka) {
            Solu::Tuki => {
                self.aseta(k.paikka, Solu::Tyhja);
                self.tuet += 1;
                true
            }
            Solu::Tyhja if self.tuet > 0 && k.paikka.1 >= MAA_ALKAA => {
                self.aseta(k.paikka, Solu::Tuki);
                self.tuet -= 1;
                true
            }
            _ => false,
        }
    }

    /// Hätäköysi nostaa kaivajan pinnalle kypärän hinnalla. Tosi, jos köysi heitettiin.
    pub fn heita_koysi(&mut self, tapahtumat: &mut Vec<Tapahtuma>) -> bool {
        if self.ohi || self.hissi_ms.is_some() || self.kaivaja.paikka.1 < MAA_ALKAA {
            return false;
        }
        let menetetty = self.menetys();
        tapahtumat.push(Tapahtuma::Koysi { menetetty });
        self.tarkista_loppu(tapahtumat);
        true
    }

    /// Kypärä ja säkki menevät, ja kaivaja aloittaa pinnalta.
    fn menetys(&mut self) -> u32 {
        self.kyparat = self.kyparat.saturating_sub(1);
        let menetetty = std::mem::take(&mut self.saalis);
        self.kaivaja = Kaivaja::new(ALKU);
        self.painallus = None;
        self.kiintio_ilmoitettu = self.kiintio_taynna();
        menetetty
    }

    fn tarkista_loppu(&mut self, tapahtumat: &mut Vec<Tapahtuma>) {
        if self.kyparat == 0 && !self.ohi {
            self.ohi = true;
            tapahtumat.push(Tapahtuma::Ohi);
        }
    }

    pub fn askel(&mut self, arpa: &mut Arpa) -> Vec<Tapahtuma> {
        let mut tapahtumat = Vec::new();
        if self.ohi {
            return tapahtumat;
        }
        if let Some(jaljella) = self.hissi_ms {
            let jaljella = jaljella.saturating_sub(ASKEL_MS);
            if jaljella == 0 {
                self.hissi_ms = None;
                self.taso += 1;
                self.luo_taso(arpa);
                tapahtumat.push(Tapahtuma::UusiTaso);
            } else {
                self.hissi_ms = Some(jaljella);
            }
            return tapahtumat;
        }
        self.kaivajan_askel(&mut tapahtumat);
        if self.ohi || self.hissi_ms.is_some() {
            return tapahtumat;
        }
        self.putoavien_askel(&mut tapahtumat);
        if self.ohi {
            return tapahtumat;
        }
        self.ritinan_askel(&mut tapahtumat);
        self.lampun_askel(&mut tapahtumat);
        tapahtumat
    }

    fn lampun_askel(&mut self, tapahtumat: &mut Vec<Tapahtuma>) {
        let taysi = self.vaikeus.oljya_ms();
        if self.kaivaja.paikka.1 >= MAA_ALKAA {
            if self.oljy_ms > 0 {
                self.oljy_ms = self.oljy_ms.saturating_sub(ASKEL_MS);
                if self.oljy_ms == 0 {
                    tapahtumat.push(Tapahtuma::OljyLoppui);
                }
            }
        } else {
            self.oljy_ms = (self.oljy_ms + ASKEL_MS * TAYTTOKERROIN).min(taysi);
        }
    }

    /// Kaivaja seisoo tukevasti: omassa ruudussa on tuki tai alla on jotain kiinteää.
    fn seisoo_tukevasti(&self, (x, y): Ruutu) -> bool {
        self.solu((x, y)) == Solu::Tuki
            || self.solu((x, y + 1)).kiintea()
            || self.putoava_kohdassa((x, y + 1))
    }

    fn kaivajan_askel(&mut self, tapahtumat: &mut Vec<Tapahtuma>) {
        let kesto = self.kaivaja.kesto(&self.solut);
        let k = &mut self.kaivaja;
        let toimi = k.toimi;
        match toimi {
            Toimi::Seisoo => {}
            Toimi::Liikkuu(dx, dy) => {
                k.ms += ASKEL_MS;
                if k.ms >= kesto {
                    k.paikka = (k.paikka.0 + dx as i32, k.paikka.1 + dy as i32);
                    k.toimi = Toimi::Seisoo;
                    k.ms = 0;
                }
            }
            Toimi::Kaivaa(dx, dy) => {
                let kohde = (k.paikka.0 + dx as i32, k.paikka.1 + dy as i32);
                k.ms += ASKEL_MS;
                if !self.solu(kohde).kaivettava() {
                    // Ruutu putosi tai valui pois kesken kaivuun.
                    self.kaivaja.toimi = Toimi::Seisoo;
                    self.kaivaja.ms = 0;
                } else if self.kaivaja.ms >= kesto {
                    self.kaivaja.toimi = if dy == 0 {
                        Toimi::Liikkuu(dx, 0)
                    } else {
                        Toimi::Seisoo
                    };
                    self.kaivaja.ms = 0;
                    self.kaiva(kohde, tapahtumat);
                }
            }
            Toimi::Putoaa => {
                k.ms += ASKEL_MS;
                if k.ms >= PUTOAMINEN_MS {
                    k.paikka.1 += 1;
                    k.pudotus += 1;
                    k.toimi = Toimi::Seisoo;
                    k.ms = 0;
                    if self.putoava_kohdassa(self.kaivaja.paikka) {
                        self.murskaa(tapahtumat);
                        return;
                    }
                }
            }
        }
        if self.kaivaja.toimi == Toimi::Seisoo {
            self.paata(tapahtumat);
        }
    }

    fn kaiva(&mut self, kohde: Ruutu, tapahtumat: &mut Vec<Tapahtuma>) {
        let loydetty = self.solu(kohde);
        self.aseta(kohde, Solu::Tyhja);
        tapahtumat.push(Tapahtuma::Kaivoi(loydetty));
        let loyto = match loydetty {
            Solu::Kulta => Loyto::Kulta(self.arvo(kullan_arvo(kohde.1))),
            Solu::Timantti => Loyto::Timantti(self.arvo(timantin_arvo(self.taso))),
            Solu::Oljy => Loyto::Oljy,
            Solu::Puut => Loyto::Puut(PUITA_NIPUSSA),
            _ => return,
        };
        match loyto {
            Loyto::Kulta(arvo) | Loyto::Timantti(arvo) => self.saalis += arvo,
            Loyto::Oljy => {
                let taysi = self.vaikeus.oljya_ms();
                self.oljy_ms = (self.oljy_ms + (taysi as f32 * KANNUN_OSUUS) as u32).min(taysi);
            }
            Loyto::Puut(maara) => self.tuet += maara,
        }
        tapahtumat.push(Tapahtuma::Loysi {
            loyto,
            paikka: kohde,
        });
        if !self.kiintio_ilmoitettu && self.kiintio_taynna() {
            self.kiintio_ilmoitettu = true;
            tapahtumat.push(Tapahtuma::KiintioTaynna);
        }
    }

    /// Seisova kaivaja: painovoima, hissi ja seuraava toimi.
    fn paata(&mut self, tapahtumat: &mut Vec<Tapahtuma>) {
        let paikka = self.kaivaja.paikka;
        if !self.seisoo_tukevasti(paikka) {
            self.kaivaja.toimi = Toimi::Putoaa;
            self.kaivaja.ms = 0;
            return;
        }
        if self.kaivaja.pudotus > 0 {
            let pudotus = std::mem::take(&mut self.kaivaja.pudotus);
            if pudotus > TURVALLINEN_PUDOTUS {
                let menetetty = self.menetys();
                tapahtumat.push(Tapahtuma::Putosi { paikka, menetetty });
                self.tarkista_loppu(tapahtumat);
                return;
            }
        }
        if paikka == HISSI && self.nosta(tapahtumat) {
            return;
        }
        // Yksi painallus on yksi toimi: pohjassa pidetty näppäin ei jatka liikettä.
        if let Some(suunta) = self.painallus.take() {
            self.yrita(suunta);
        }
    }

    /// Säkki hissiin. Tosi, kun kiintiö täyttyi ja hissi lähtee.
    fn nosta(&mut self, tapahtumat: &mut Vec<Tapahtuma>) -> bool {
        if self.saalis > 0 {
            let saalis = std::mem::take(&mut self.saalis);
            self.nostettu += saalis;
            self.pisteet = (self.pisteet + saalis).min(SUURIN_PISTEMAARA);
            tapahtumat.push(Tapahtuma::Nosti { pisteet: saalis });
        }
        if self.nostettu < self.kiintio {
            return false;
        }
        let bonus = self.arvo(50 * (self.taso + 1) + 10 * self.tuet);
        self.pisteet = (self.pisteet + bonus).min(SUURIN_PISTEMAARA);
        tapahtumat.push(Tapahtuma::TasoValmis { bonus });
        self.hissi_ms = Some(HISSIN_MATKA_MS);
        self.painallus = None;
        true
    }

    fn yrita(&mut self, (dx, dy): (i8, i8)) {
        let k = self.kaivaja;
        if dx != 0 {
            self.kaivaja.katse = dx;
        }
        let kohde = (k.paikka.0 + dx as i32, k.paikka.1 + dy as i32);
        if !alueella(kohde) || kohde.1 < PINTA || self.putoava_kohdassa(kohde) {
            return;
        }
        let oma = self.solu(k.paikka);
        let toinen = self.solu(kohde);
        let toimi = match (dx, dy) {
            (_, 0) if toinen.lapaistava() => Toimi::Liikkuu(dx, 0),
            (_, 0) if toinen.kaivettava() => Toimi::Kaivaa(dx, 0),
            (0, -1) if toinen.kaivettava() => Toimi::Kaivaa(0, -1),
            (0, -1) if toinen.lapaistava() && oma == Solu::Tuki => Toimi::Liikkuu(0, -1),
            // Tuessa seisova ei kaiva jalkojensa alta: tuki putoaisi hänen kanssaan.
            (0, 1) if toinen.kaivettava() && oma != Solu::Tuki => Toimi::Kaivaa(0, 1),
            (0, 1) if toinen == Solu::Tuki || (oma == Solu::Tuki && toinen == Solu::Tyhja) => {
                Toimi::Liikkuu(0, 1)
            }
            _ => return,
        };
        self.kaivaja.toimi = toimi;
        self.kaivaja.ms = 0;
    }

    /// Kaivajan ruudut, joihin putoava ruutu voi osua: oma ja liikkeen kohde.
    fn kaivajan_ruudut(&self) -> [Ruutu; 2] {
        let k = self.kaivaja;
        let kohde = match k.toimi {
            Toimi::Liikkuu(dx, dy) => (k.paikka.0 + dx as i32, k.paikka.1 + dy as i32),
            _ => k.paikka,
        };
        [k.paikka, kohde]
    }

    fn murskaa(&mut self, tapahtumat: &mut Vec<Tapahtuma>) {
        let paikka = self.kaivaja.paikka;
        let menetetty = self.menetys();
        tapahtumat.push(Tapahtuma::Murskautui { paikka, menetetty });
        self.tarkista_loppu(tapahtumat);
    }

    fn putoavien_askel(&mut self, tapahtumat: &mut Vec<Tapahtuma>) {
        // Alimmat ensin, jotta päällekkäiset putoavat liikkuvat yhtenä pinona.
        self.putoavat.sort_by_key(|p| std::cmp::Reverse(p.paikka.1));
        let mut i = 0;
        while i < self.putoavat.len() {
            let mut p = self.putoavat[i];
            p.ms += ASKEL_MS;
            if p.ms < PUTOAMINEN_MS {
                self.putoavat[i] = p;
                i += 1;
                continue;
            }
            let alla = (p.paikka.0, p.paikka.1 + 1);
            if self.solu(alla).kiintea() {
                self.putoavat.remove(i);
                self.aseta(p.paikka, p.solu);
                tapahtumat.push(Tapahtuma::Sortui {
                    solu: p.solu,
                    paikka: p.paikka,
                });
                continue;
            }
            if self.putoava_kohdassa(alla) {
                // Alempi pysähtyi juuri: odotetaan, että se laskeutuu.
                p.ms = PUTOAMINEN_MS;
                self.putoavat[i] = p;
                i += 1;
                continue;
            }
            p.paikka = alla;
            p.ms = 0;
            self.putoavat[i] = p;
            i += 1;
            if self.kaivajan_ruudut().contains(&alla) {
                self.murskaa(tapahtumat);
                if self.ohi {
                    return;
                }
            }
        }
    }

    fn ritinan_askel(&mut self, tapahtumat: &mut Vec<Tapahtuma>) {
        if std::mem::take(&mut self.tuenta_vanha) {
            let tuettu = tuenta(&self.solut);
            let prosentti = self.vaikeus.ritinan_prosentti();
            let mut uusi = None;
            let rivit = self.solut.iter().zip(&tuettu).zip(&mut self.ritina);
            for (y, ((solut, tuetut), ritinat)) in rivit.enumerate() {
                let sarakkeet = solut.iter().zip(tuetut).zip(ritinat.iter_mut());
                for (x, ((&s, &tuettu), ritina)) in sarakkeet.enumerate() {
                    if tuettu || !s.kiintea() {
                        *ritina = 0;
                    } else if *ritina == 0 {
                        let kesto = s.ritinan_kesto() * prosentti / 100;
                        *ritina = kesto.max(ASKEL_MS);
                        uusi.get_or_insert((x as i32, y as i32));
                    }
                }
            }
            if let Some(paikka) = uusi {
                tapahtumat.push(Tapahtuma::Ritisee(paikka));
            }
        }
        let mut putoavat = Vec::new();
        for y in 0..RIVEJA {
            for x in 0..SARAKKEITA {
                let ritina = &mut self.ritina[y][x];
                if *ritina > 0 {
                    *ritina = ritina.saturating_sub(ASKEL_MS);
                    if *ritina == 0 {
                        putoavat.push((x as i32, y as i32));
                    }
                }
            }
        }
        for paikka in putoavat {
            let solu = self.solu(paikka);
            self.aseta(paikka, Solu::Tyhja);
            self.putoavat.push(Putoava {
                solu,
                paikka,
                ms: 0,
            });
        }
    }
}

/// Arpoo tason kentän. Ylin maarivi hissin ja alun ympärillä on turvallista maata.
pub fn luo_kentta(vaikeus: Vaikeus, taso: u32, arpa: &mut Arpa) -> Solut {
    let mut s = [[Solu::Tyhja; SARAKKEITA]; RIVEJA];
    let ala = RIVEJA as i32 - 1;
    for rivi in s.iter_mut().take(RIVEJA - 1).skip(MAA_ALKAA as usize) {
        rivi.fill(Solu::Maa);
    }
    s[RIVEJA - 1].fill(Solu::Kallio);
    let syvyys = ala - MAA_ALKAA;
    let t = taso.min(10) as usize;
    let vaarat = vaikeus.vaaroja();
    let mut arvo =
        |pienin: i32, suurin: i32| pienin + arpa.alle((suurin - pienin + 1) as usize) as i32;
    let turvassa = |x: i32, y: i32| x <= 5 && y <= MAA_ALKAA + 2;

    let aseta_alue = |s: &mut Solut, x: i32, y: i32, l: i32, k: i32, solu: Solu| {
        for yy in y..y + k {
            for xx in x..x + l {
                if alueella((xx, yy)) && yy > MAA_ALKAA && yy < ala && !turvassa(xx, yy) {
                    s[yy as usize][xx as usize] = solu;
                }
            }
        }
    };

    // Savilinssit, syvemmällä enemmän.
    let mut arvottavat: Vec<(i32, i32, i32, i32, Solu)> = Vec::new();
    for _ in 0..3 + t / 2 {
        let y = arvo(MAA_ALKAA + 4, ala - 1);
        arvottavat.push((arvo(0, 18), y, arvo(3, 6), arvo(1, 2), Solu::Savi));
    }
    // Kallioniemekkeet antavat tukea syvällä.
    for _ in 0..2 + t / 3 {
        arvottavat.push((
            arvo(0, 20),
            arvo(MAA_ALKAA + 5, ala - 1),
            arvo(1, 3),
            arvo(1, 2),
            Solu::Kallio,
        ));
    }
    // Hiekkataskut.
    for _ in 0..2 + t + vaarat {
        arvottavat.push((
            arvo(0, 19),
            arvo(MAA_ALKAA + 1, ala - 2),
            arvo(2, 4),
            arvo(1, 3),
            Solu::Hiekka,
        ));
    }
    for (x, y, l, k, solu) in arvottavat {
        aseta_alue(&mut s, x, y, l, k, solu);
    }
    // Irtokivet.
    for _ in 0..3 + t + 2 * vaarat {
        let (x, y) = (arvo(0, 21), arvo(MAA_ALKAA + 1, ala - 1));
        aseta_alue(&mut s, x, y, 1, 1, Solu::Kivi);
    }
    // Hissin perustus.
    s[MAA_ALKAA as usize][..=2].fill(Solu::Kallio);

    // Luolat toiselta tasolta alkaen. Katto ja pohja ovat maata, ja luola, jonka katto ei
    // kantaisi (esimerkiksi kaksi luolaa yhteen sulautuneena), jätetään kaivamatta.
    if taso >= 1 {
        for _ in 0..1 + t.min(3) {
            let (x, y, l) = (arvo(6, 18), arvo(MAA_ALKAA + 3, ala - 2), arvo(2, 4));
            let ennen = s;
            for xx in x..x + l {
                s[y as usize][xx as usize] = Solu::Tyhja;
                for viereinen in [y - 1, y + 1] {
                    let solu = &mut s[viereinen as usize][xx as usize];
                    if matches!(*solu, Solu::Hiekka | Solu::Kivi) {
                        *solu = Solu::Maa;
                    }
                }
            }
            let tuettu = tuenta(&s);
            let kantaa = s
                .iter()
                .flatten()
                .zip(tuettu.iter().flatten())
                .all(|(solu, &tuettu)| tuettu || !solu.kiintea());
            if !kantaa {
                s = ennen;
            }
        }
    }

    // Aarteet vain maahan. Kultaa on syvemmällä tiheämmin.
    let mut sijoita = |s: &mut Solut, solu: Solu, maara: usize, ylin: i32| {
        let mut jaljella = maara;
        let mut yrityksia = 0;
        while jaljella > 0 && yrityksia < 2000 {
            yrityksia += 1;
            let y = arvo(ylin, ala - 1);
            // Kaksi arvontaa ja syvempi: kulta painottuu alas.
            let y = if solu == Solu::Kulta {
                y.max(arvo(ylin, ala - 1))
            } else {
                y
            };
            let x = arvo(0, SARAKKEITA as i32 - 1);
            if s[y as usize][x as usize] == Solu::Maa && !turvassa(x, y) {
                s[y as usize][x as usize] = solu;
                jaljella -= 1;
            }
        }
    };
    sijoita(&mut s, Solu::Kulta, 14 + t, MAA_ALKAA + 1);
    if taso >= 1 {
        sijoita(&mut s, Solu::Timantti, 1 + t / 3, MAA_ALKAA + syvyys / 2);
    }
    sijoita(&mut s, Solu::Oljy, 2, MAA_ALKAA + 4);
    sijoita(
        &mut s,
        Solu::Puut,
        if vaikeus == Vaikeus::Vaikea { 2 } else { 3 },
        MAA_ALKAA + 3,
    );
    s
}
