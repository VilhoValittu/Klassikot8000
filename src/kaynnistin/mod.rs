//! Pelin valinta ja käynnissä olevan pelin hallinta. Uusi peli lisätään tänne
//! korttina (`PELIT`) ja `Kaynnissa`-muunnelmana.

pub mod netti;
pub mod piirto;

use klassikot8000::laivanupotus::Vaikeus;
use klassikot8000::miinaharava;
use klassikot8000::pasianssi::{self, Variantti};

use self::netti::{Nettipeli, Pyynto, Vastaus};
use crate::pelit::asteroidit::peli::Asteroidit;
use crate::pelit::asteroidit::piirto::kivimalli;
use crate::pelit::blackjack::peli::{self as blackjack_peli, Blackjack};
use crate::pelit::blackjack::piirto::pelimerkkipinot;
use crate::pelit::holdem::peli::{self as holdem_peli, Holdem};
use crate::pelit::holdem::piirto::poytamalli;
use crate::pelit::laivanupotus::peli::Peli;
use crate::pelit::laivanupotus::piirto::alkumalli as laivamalli;
use crate::pelit::luvut::peli::Luvut;
use crate::pelit::luvut::piirto::lukupino;
use crate::pelit::luvut::tallennus::Tallennus;
use crate::pelit::miinaharava::peli::Miinaharava;
use crate::pelit::miinaharava::piirto::miinamalli;
use crate::pelit::palikka::peli::Palikkapeli;
use crate::pelit::palikka::piirto::jalokivi;
use crate::pelit::pasianssi::peli::{self as pasianssi_peli, Pasianssi, VARIANTTEJA};
use crate::pelit::pasianssi::piirto::korttipinot;
use crate::pelit::silmukka::peli::Silmukka;
use crate::pelit::silmukka::piirto::ouroboros;
use crate::pelit::tiilirikko::peli::Tiilirikko;
use crate::pelit::tiilirikko::piirto::tiilimuuri;
use crate::yhteinen::aanet::Aani;
use crate::yhteinen::kieli::{self, Kieli, kaanna};
use crate::yhteinen::piirto::ruudulta_kuvaan;
use crate::yhteinen::syote::Syote;
use crate::yhteinen::tulokset::{Tulos, lisaa_tulos};
use crate::yhteinen::vektori::Tahko;

/// Valikon mallia ja vilkkua piirretään harvemmin kuin silmukka kiertää: joka toinen
/// kierros riittää hitaaseen pyörintään ja puolittaa ruudun lähettämisen.
const VALIKON_ASKEL_MS: u32 = 66;

/// Pasianssin lajin nettilista.
fn pasianssin_nettipeli(variantti: Variantti) -> Nettipeli {
    match variantti {
        Variantti::Klondike => Nettipeli::Pasianssi,
        Variantti::Spider => Nettipeli::Spider,
        Variantti::Pyramidi => Nettipeli::Pyramidi,
    }
}

/// Kaikki, mistä paikallaan pysyvän näkymän kuva riippuu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KuvanAvain {
    Valikko(usize, bool, Kieli, bool, u32),
    Pasianssi {
        syotteita: u32,
        sekunnit: u32,
        vilkku: u32,
    },
    /// Vuoropeli, jonka kuva muuttuu syötteestä tai pelin omassa tahdissa.
    Vuoro {
        syotteita: u32,
        askel: u32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Laji {
    Laivanupotus,
    Luvut,
    Pasianssi,
    Tiilirikko,
    Asteroidit,
    Silmukka,
    Palikka,
    Miinaharava,
    Blackjack,
    Holdem,
}

pub struct Kortti {
    pub laji: Laji,
    pub nimi: &'static str,
    pub kuvaus: &'static str,
    pub malli: fn() -> Vec<Tahko>,
}

pub const PELIT: [Kortti; 10] = [
    Kortti {
        laji: Laji::Laivanupotus,
        nimi: "peli.laivanupotus",
        kuvaus: "valikko.laivanupotus",
        malli: laivamalli,
    },
    Kortti {
        laji: Laji::Luvut,
        nimi: "peli.luvut",
        kuvaus: "valikko.luvut",
        malli: lukupino,
    },
    Kortti {
        laji: Laji::Pasianssi,
        nimi: "peli.pasianssi",
        kuvaus: "valikko.pasianssi",
        malli: korttipinot,
    },
    Kortti {
        laji: Laji::Tiilirikko,
        nimi: "peli.tiilirikko",
        kuvaus: "valikko.tiilirikko",
        malli: tiilimuuri,
    },
    Kortti {
        laji: Laji::Asteroidit,
        nimi: "peli.asteroidit",
        kuvaus: "valikko.asteroidit",
        malli: kivimalli,
    },
    Kortti {
        laji: Laji::Silmukka,
        nimi: "peli.silmukka",
        kuvaus: "valikko.silmukka",
        malli: ouroboros,
    },
    Kortti {
        laji: Laji::Palikka,
        nimi: "peli.palikka",
        kuvaus: "valikko.palikka",
        malli: jalokivi,
    },
    Kortti {
        laji: Laji::Miinaharava,
        nimi: "peli.miinaharava",
        kuvaus: "valikko.miinaharava",
        malli: miinamalli,
    },
    Kortti {
        laji: Laji::Blackjack,
        nimi: "peli.blackjack",
        kuvaus: "valikko.blackjack",
        malli: pelimerkkipinot,
    },
    Kortti {
        laji: Laji::Holdem,
        nimi: "peli.holdem",
        kuvaus: "valikko.holdem",
        malli: poytamalli,
    },
];

pub enum Kaynnissa {
    Laivanupotus(Box<Peli>),
    Luvut(Box<Luvut>),
    Pasianssi(Box<Pasianssi>),
    Tiilirikko(Box<Tiilirikko>),
    Asteroidit(Box<Asteroidit>),
    Silmukka(Box<Silmukka>),
    Palikka(Box<Palikkapeli>),
    Miinaharava(Box<Miinaharava>),
    Blackjack(Box<Blackjack>),
    Holdem(Box<Holdem>),
}

/// Pelin valinta ja käynnissä oleva peli. Asetukset säilyvät, kun peli vaihtuu.
pub struct Kaynnistin {
    pub valittu: usize,
    /// Valitun pelin malli. Rakennetaan vain valinnan vaihtuessa.
    malli: Vec<Tahko>,
    /// Kellon lukema, kun valinta viimeksi vaihtui.
    valittu_ms: u32,
    pub peli: Option<Kaynnissa>,
    pub aanet_paalla: bool,
    pub lopeta: bool,
    /// Tosi, kun pelistä poistuttiin: sen viiveellä odottavat äänet eivät saa jäädä soimaan.
    pub hiljenna: bool,
    pub kello_ms: u32,
    /// 2048:n pysyvä tila, parhaat tulokset ja ääniasetus. Nykytila siirtyy tänne `paivita_tallennus`-kutsulla.
    pub tallennus: Tallennus,
    /// Tosi, kun `main.rs`:n kuuluu kirjoittaa tallennus tiedostoon.
    pub tallenna: bool,
    /// Pyynnöt, jotka `main.rs` välittää verkkosäikeelle.
    pub pyynnot: Vec<Pyynto>,
    /// Käsiteltyjen syötteiden määrä: kuva voi muuttua minkä tahansa syötteen jälkeen.
    syotteita: u32,
    /// Nettilistat, kun palvelin on vastannut.
    netti_luvut: Option<Vec<Tulos>>,
    netti_tiilirikko: Option<Vec<Tulos>>,
    netti_asteroidit: Option<Vec<Tulos>>,
    netti_silmukka: Option<Vec<Tulos>>,
    netti_palikka: Option<Vec<Tulos>>,
    /// Pasianssin lajit erikseen, järjestys kuten `Variantti::KAIKKI`.
    netti_pasianssi: [Option<Vec<Tulos>>; VARIANTTEJA],
    netti_blackjack: Option<Vec<Tulos>>,
    netti_holdem: Option<Vec<Tulos>>,
    /// Uusin julkaistu versio, jos se on uudempi kuin tämä ohjelma.
    uusi_versio: Option<String>,
    vaikeus: Vaikeus,
    pasianssin_vaikeus: pasianssi::Vaikeus,
    pasianssin_variantti: Variantti,
    miinaharavan_vaikeus: miinaharava::Vaikeus,
    siemen: u64,
}

impl Kaynnistin {
    pub fn new(siemen: u64) -> Self {
        Self {
            valittu: 0,
            malli: (PELIT[0].malli)(),
            valittu_ms: 0,
            peli: None,
            aanet_paalla: true,
            lopeta: false,
            hiljenna: false,
            kello_ms: 0,
            tallennus: Tallennus::default(),
            tallenna: false,
            pyynnot: Vec::new(),
            syotteita: 0,
            netti_luvut: None,
            netti_tiilirikko: None,
            netti_asteroidit: None,
            netti_silmukka: None,
            netti_palikka: None,
            netti_pasianssi: Default::default(),
            netti_blackjack: None,
            netti_holdem: None,
            uusi_versio: None,
            vaikeus: Vaikeus::Normaali,
            pasianssin_vaikeus: pasianssi::Vaikeus::Normaali,
            pasianssin_variantti: Variantti::Klondike,
            miinaharavan_vaikeus: miinaharava::Vaikeus::Helppo,
            siemen,
        }
    }

    pub fn valitse(&mut self, i: usize) {
        if i != self.valittu {
            self.valittu = i;
            self.valittu_ms = self.kello_ms;
            self.malli = (PELIT[i].malli)();
        }
    }

    /// Valitun pelin omat parhaat. Laivanupotuksella ja miinaharavalla ei ole pistelistaa.
    pub fn valitun_parhaat(&self) -> Option<&[Tulos]> {
        match PELIT[self.valittu].laji {
            Laji::Luvut => Some(&self.tallennus.parhaat),
            Laji::Tiilirikko => Some(&self.tallennus.tiilirikon_parhaat),
            Laji::Asteroidit => Some(&self.tallennus.asteroidien_parhaat),
            Laji::Silmukka => Some(&self.tallennus.silmukan_parhaat),
            Laji::Palikka => Some(&self.tallennus.palikan_parhaat),
            // Viimeksi valitun lajin lista: sen pelaaja haluaa todennäköisimmin nähdä.
            Laji::Pasianssi => Some(match self.pasianssin_variantti {
                Variantti::Klondike => &self.tallennus.pasianssin_parhaat,
                Variantti::Spider => &self.tallennus.spiderin_parhaat,
                Variantti::Pyramidi => &self.tallennus.pyramidin_parhaat,
            }),
            Laji::Blackjack => Some(&self.tallennus.blackjackin_parhaat),
            Laji::Holdem => Some(&self.tallennus.holdemin_parhaat),
            Laji::Laivanupotus | Laji::Miinaharava => None,
        }
    }

    /// Kaikki, mistä näkymän kuva riippuu, tai `None`, kun kuva elää joka ruudussa. Kun avain
    /// pysyy samana, näkymää ei tarvitse piirtää. Taustalla oleva valikko ei animoi.
    pub fn kuvan_avain(&self, aktiivinen: bool) -> Option<KuvanAvain> {
        match &self.peli {
            None => {
                let kasvaa =
                    self.kello_ms.saturating_sub(self.valittu_ms) < piirto::MALLIN_KASVU_MS;
                let askel = match (aktiivinen, kasvaa) {
                    (false, _) => 0,
                    // lyhyt kasvu näytetään sulavasti silmukan tahdissa
                    (true, true) => self.kello_ms,
                    (true, false) => self.kello_ms / VALIKON_ASKEL_MS,
                };
                Some(KuvanAvain::Valikko(
                    self.valittu,
                    self.aanet_paalla,
                    kieli::nykyinen(),
                    self.uusi_versio.is_some(),
                    askel,
                ))
            }
            Some(Kaynnissa::Pasianssi(peli)) => {
                peli.ajan_avain()
                    .map(|(sekunnit, vilkku)| KuvanAvain::Pasianssi {
                        syotteita: self.syotteita,
                        sekunnit,
                        vilkku,
                    })
            }
            Some(Kaynnissa::Palikka(peli)) => peli.kuvan_askel().map(|askel| KuvanAvain::Vuoro {
                syotteita: self.syotteita,
                askel,
            }),
            Some(Kaynnissa::Miinaharava(peli)) => {
                peli.kuvan_askel().map(|askel| KuvanAvain::Vuoro {
                    syotteita: self.syotteita,
                    askel,
                })
            }
            Some(Kaynnissa::Blackjack(peli)) => peli.kuvan_askel().map(|askel| KuvanAvain::Vuoro {
                syotteita: self.syotteita,
                askel,
            }),
            Some(Kaynnissa::Holdem(peli)) => peli.kuvan_askel().map(|askel| KuvanAvain::Vuoro {
                syotteita: self.syotteita,
                askel,
            }),
            Some(_) => None,
        }
    }

    pub fn otsikko(&self) -> &'static str {
        kaanna(match self.peli {
            Some(Kaynnissa::Laivanupotus(_)) => "peli.laivanupotus",
            Some(Kaynnissa::Luvut(_)) => "peli.luvut",
            Some(Kaynnissa::Pasianssi(_)) => "peli.pasianssi",
            Some(Kaynnissa::Tiilirikko(_)) => "peli.tiilirikko",
            Some(Kaynnissa::Asteroidit(_)) => "peli.asteroidit",
            Some(Kaynnissa::Silmukka(_)) => "peli.silmukka",
            Some(Kaynnissa::Palikka(_)) => "peli.palikka",
            Some(Kaynnissa::Miinaharava(_)) => "peli.miinaharava",
            Some(Kaynnissa::Blackjack(_)) => "peli.blackjack",
            Some(Kaynnissa::Holdem(_)) => "peli.holdem",
            None => "peli.pelikone",
        })
    }

    /// Reaaliaikainen peli tarvitsee tiheämmän päivityksen kuin vuoropelit, mutta sen
    /// alkuruudulle riittää sama tahti kuin muille alkuruuduille.
    pub fn toimintapeli(&self) -> bool {
        match &self.peli {
            Some(Kaynnissa::Tiilirikko(peli)) => {
                !matches!(peli.vaihe, crate::pelit::tiilirikko::peli::Vaihe::Alku)
            }
            Some(Kaynnissa::Asteroidit(peli)) => {
                !matches!(peli.vaihe, crate::pelit::asteroidit::peli::Vaihe::Alku)
            }
            Some(Kaynnissa::Silmukka(peli)) => {
                !matches!(peli.vaihe, crate::pelit::silmukka::peli::Vaihe::Alku)
            }
            // Palikat liikkuvat vain vaihdon jälkeen, muuten lauta seisoo.
            Some(Kaynnissa::Palikka(peli)) => peli.animoi(),
            _ => false,
        }
    }

    /// Pohjassa olevat nuolet. Näppäintoisto olisi mailan ohjaukseen liian nykivä.
    pub fn aseta_ohjaus(&mut self, vasen: bool, oikea: bool, ylos: bool) {
        let suunta = oikea as i8 - vasen as i8;
        match &mut self.peli {
            Some(Kaynnissa::Tiilirikko(peli)) => peli.ohjaus = suunta,
            Some(Kaynnissa::Asteroidit(peli)) => peli.aseta_nuolet(suunta, ylos),
            _ => {}
        }
    }

    /// Ikkuna ei ole aktiivinen, joten reaaliaikainen peli jää odottamaan.
    pub fn keskeyta(&mut self) {
        match &mut self.peli {
            Some(Kaynnissa::Tiilirikko(peli)) => peli.keskeyta(),
            Some(Kaynnissa::Asteroidit(peli)) => peli.keskeyta(),
            Some(Kaynnissa::Silmukka(peli)) => peli.keskeyta(),
            _ => {}
        }
    }

    /// Laivanupotuksessa nuoli liikuttaa tähtäintä, mutta 2048:ssa toisto tekisi
    /// pohjassa pidetystä nuolesta siirtosarjan, ja Silmukassa se täyttäisi kääntöjonon.
    pub fn nuolet_toistuvat(&self) -> bool {
        !matches!(
            self.peli,
            Some(Kaynnissa::Luvut(_) | Kaynnissa::Silmukka(_))
        )
    }

    /// Muuttaa ruudun pikselin sen näkymän koordinaateiksi, joka on nyt esillä.
    pub fn nakyman_piste(&self, x: i32, y: i32) -> (i32, i32) {
        match &self.peli {
            None => (x, y),
            Some(Kaynnissa::Pasianssi(peli)) if peli.vaihe != pasianssi_peli::Vaihe::Alku => (x, y),
            Some(Kaynnissa::Blackjack(peli)) if peli.vaihe != blackjack_peli::Vaihe::Alku => (x, y),
            Some(Kaynnissa::Holdem(peli)) if peli.vaihe != holdem_peli::Vaihe::Alku => (x, y),
            _ => ruudulta_kuvaan(x, y),
        }
    }

    pub fn kaynnista(&mut self, laji: Laji) {
        let siemen = self.siemen;
        // Sama siemen toistaisi saman alkutilanteen, kun peli avataan uudelleen.
        self.siemen = self.siemen.wrapping_add(1);
        self.peli = Some(match laji {
            Laji::Laivanupotus => {
                let mut peli = Peli::new(siemen);
                peli.vaikeus = self.vaikeus;
                peli.aanet_paalla = self.aanet_paalla;
                Kaynnissa::Laivanupotus(Box::new(peli))
            }
            Laji::Luvut => {
                let mut peli = Luvut::new(siemen);
                peli.paras = self.tallennus.paras;
                peli.parhaat = self.tallennus.parhaat.clone();
                peli.netti_parhaat = self.netti_luvut.clone();
                self.pyynnot.push(Pyynto::Hae(Nettipeli::Luvut));
                if let Some(kesken) = self.tallennus.kesken {
                    peli.palauta(kesken);
                }
                peli.aanet_paalla = self.aanet_paalla;
                Kaynnissa::Luvut(Box::new(peli))
            }
            Laji::Pasianssi => {
                let mut peli = Pasianssi::new(siemen);
                peli.vaikeus = self.pasianssin_vaikeus;
                peli.variantti = self.pasianssin_variantti;
                peli.parhaat = [
                    self.tallennus.pasianssin_parhaat.clone(),
                    self.tallennus.spiderin_parhaat.clone(),
                    self.tallennus.pyramidin_parhaat.clone(),
                ];
                peli.netti_parhaat = self.netti_pasianssi.clone();
                peli.aanet_paalla = self.aanet_paalla;
                for variantti in Variantti::KAIKKI {
                    self.pyynnot
                        .push(Pyynto::Hae(pasianssin_nettipeli(variantti)));
                }
                Kaynnissa::Pasianssi(Box::new(peli))
            }
            Laji::Tiilirikko => {
                let mut peli = Tiilirikko::new(siemen);
                peli.paras = self.tallennus.tiilirikko;
                peli.parhaat = self.tallennus.tiilirikon_parhaat.clone();
                peli.netti_parhaat = self.netti_tiilirikko.clone();
                peli.aanet_paalla = self.aanet_paalla;
                self.pyynnot.push(Pyynto::Hae(Nettipeli::Tiilirikko));
                Kaynnissa::Tiilirikko(Box::new(peli))
            }
            Laji::Asteroidit => {
                let mut peli = Asteroidit::new(siemen);
                peli.paras = self.tallennus.asteroidit;
                peli.parhaat = self.tallennus.asteroidien_parhaat.clone();
                peli.netti_parhaat = self.netti_asteroidit.clone();
                peli.aanet_paalla = self.aanet_paalla;
                self.pyynnot.push(Pyynto::Hae(Nettipeli::Asteroidit));
                Kaynnissa::Asteroidit(Box::new(peli))
            }
            Laji::Silmukka => {
                let mut peli = Silmukka::new(siemen);
                peli.paras = self.tallennus.silmukka;
                peli.parhaat = self.tallennus.silmukan_parhaat.clone();
                peli.netti_parhaat = self.netti_silmukka.clone();
                peli.aanet_paalla = self.aanet_paalla;
                self.pyynnot.push(Pyynto::Hae(Nettipeli::Silmukka));
                Kaynnissa::Silmukka(Box::new(peli))
            }
            Laji::Palikka => {
                let mut peli = Palikkapeli::new(siemen);
                peli.paras = self.tallennus.palikka;
                peli.parhaat = self.tallennus.palikan_parhaat.clone();
                peli.netti_parhaat = self.netti_palikka.clone();
                peli.aanet_paalla = self.aanet_paalla;
                self.pyynnot.push(Pyynto::Hae(Nettipeli::Palikka));
                Kaynnissa::Palikka(Box::new(peli))
            }
            Laji::Miinaharava => {
                let mut peli = Miinaharava::new(siemen);
                peli.vaikeus = self.miinaharavan_vaikeus;
                peli.ajat = self.tallennus.miinaajat;
                peli.aanet_paalla = self.aanet_paalla;
                Kaynnissa::Miinaharava(Box::new(peli))
            }
            Laji::Blackjack => {
                let mut peli = Blackjack::new(siemen);
                peli.parhaat = self.tallennus.blackjackin_parhaat.clone();
                peli.netti_parhaat = self.netti_blackjack.clone();
                peli.aanet_paalla = self.aanet_paalla;
                self.pyynnot.push(Pyynto::Hae(Nettipeli::Blackjack));
                Kaynnissa::Blackjack(Box::new(peli))
            }
            Laji::Holdem => {
                let mut peli = Holdem::new(siemen);
                peli.parhaat = self.tallennus.holdemin_parhaat.clone();
                peli.netti_parhaat = self.netti_holdem.clone();
                peli.aanet_paalla = self.aanet_paalla;
                self.pyynnot.push(Pyynto::Hae(Nettipeli::Holdem));
                Kaynnissa::Holdem(Box::new(peli))
            }
        });
    }

    /// Ottaa käyttöön tiedostosta luetun tallennuksen.
    pub fn lataa(&mut self, tallennus: Tallennus) {
        self.aanet_paalla = !tallennus.hiljaa;
        kieli::aseta(tallennus.kieli);
        // Edellisellä kerralla lähettämättä jääneet tulokset yritetään uudelleen.
        for (peli, jono) in [
            (Nettipeli::Luvut, &tallennus.luvut_jono),
            (Nettipeli::Tiilirikko, &tallennus.tiilirikon_jono),
            (Nettipeli::Asteroidit, &tallennus.asteroidien_jono),
            (Nettipeli::Silmukka, &tallennus.silmukan_jono),
            (Nettipeli::Palikka, &tallennus.palikan_jono),
            (Nettipeli::Pasianssi, &tallennus.pasianssin_jono),
            (Nettipeli::Spider, &tallennus.spiderin_jono),
            (Nettipeli::Pyramidi, &tallennus.pyramidin_jono),
            (Nettipeli::Blackjack, &tallennus.blackjackin_jono),
            (Nettipeli::Holdem, &tallennus.holdemin_jono),
        ] {
            let tulokset = jono.iter().cloned();
            self.pyynnot
                .extend(tulokset.map(|tulos| Pyynto::Laheta(peli, tulos)));
        }
        self.tallennus = tallennus;
    }

    /// Ottaa verkkosäikeen vastauksen. Lähetetty tulos poistuu jonosta.
    pub fn vastaanota(&mut self, vastaus: Vastaus) {
        match vastaus {
            Vastaus::Lista(Nettipeli::Luvut, lista) => {
                if let Some(Kaynnissa::Luvut(peli)) = &mut self.peli {
                    peli.netti_parhaat = Some(lista.clone());
                }
                self.netti_luvut = Some(lista);
            }
            Vastaus::Lista(Nettipeli::Tiilirikko, lista) => {
                if let Some(Kaynnissa::Tiilirikko(peli)) = &mut self.peli {
                    peli.netti_parhaat = Some(lista.clone());
                }
                self.netti_tiilirikko = Some(lista);
            }
            Vastaus::Lista(Nettipeli::Asteroidit, lista) => {
                if let Some(Kaynnissa::Asteroidit(peli)) = &mut self.peli {
                    peli.netti_parhaat = Some(lista.clone());
                }
                self.netti_asteroidit = Some(lista);
            }
            Vastaus::Lista(Nettipeli::Silmukka, lista) => {
                if let Some(Kaynnissa::Silmukka(peli)) = &mut self.peli {
                    peli.netti_parhaat = Some(lista.clone());
                }
                self.netti_silmukka = Some(lista);
            }
            Vastaus::Lista(Nettipeli::Palikka, lista) => {
                if let Some(Kaynnissa::Palikka(peli)) = &mut self.peli {
                    peli.netti_parhaat = Some(lista.clone());
                }
                self.netti_palikka = Some(lista);
            }
            Vastaus::Lista(
                netti @ (Nettipeli::Pasianssi | Nettipeli::Spider | Nettipeli::Pyramidi),
                lista,
            ) => {
                let variantti = match netti {
                    Nettipeli::Spider => Variantti::Spider,
                    Nettipeli::Pyramidi => Variantti::Pyramidi,
                    _ => Variantti::Klondike,
                };
                if let Some(Kaynnissa::Pasianssi(peli)) = &mut self.peli {
                    peli.netti_parhaat[variantti as usize] = Some(lista.clone());
                }
                self.netti_pasianssi[variantti as usize] = Some(lista);
            }
            Vastaus::Lista(Nettipeli::Blackjack, lista) => {
                if let Some(Kaynnissa::Blackjack(peli)) = &mut self.peli {
                    peli.netti_parhaat = Some(lista.clone());
                }
                self.netti_blackjack = Some(lista);
            }
            Vastaus::Lista(Nettipeli::Holdem, lista) => {
                if let Some(Kaynnissa::Holdem(peli)) = &mut self.peli {
                    peli.netti_parhaat = Some(lista.clone());
                }
                self.netti_holdem = Some(lista);
            }
            Vastaus::Lahetetty(peli, tulos) => {
                let jono = match peli {
                    Nettipeli::Luvut => &mut self.tallennus.luvut_jono,
                    Nettipeli::Tiilirikko => &mut self.tallennus.tiilirikon_jono,
                    Nettipeli::Asteroidit => &mut self.tallennus.asteroidien_jono,
                    Nettipeli::Silmukka => &mut self.tallennus.silmukan_jono,
                    Nettipeli::Palikka => &mut self.tallennus.palikan_jono,
                    Nettipeli::Pasianssi => &mut self.tallennus.pasianssin_jono,
                    Nettipeli::Spider => &mut self.tallennus.spiderin_jono,
                    Nettipeli::Pyramidi => &mut self.tallennus.pyramidin_jono,
                    Nettipeli::Blackjack => &mut self.tallennus.blackjackin_jono,
                    Nettipeli::Holdem => &mut self.tallennus.holdemin_jono,
                };
                jono.retain(|t| *t != tulos);
                self.tallenna = true;
            }
            Vastaus::UusiVersio(versio) => self.uusi_versio = Some(versio),
        }
    }

    pub fn paivita_tallennus(&mut self) {
        self.tallennus.hiljaa = !self.aanet_paalla;
        self.tallennus.kieli = kieli::nykyinen();
        match &mut self.peli {
            Some(Kaynnissa::Luvut(peli)) => {
                self.tallennus.paras = peli.paras;
                self.tallennus.parhaat = peli.parhaat.clone();
                self.tallennus.kesken = peli.kesken();
                self.netti_luvut = peli.netti_parhaat.clone();
                // Jono kirjataan tiedostoon, jottei tulos katoa, vaikka lähetys epäonnistuisi.
                if let Some(tulos) = peli.lahetys.take() {
                    lisaa_tulos(&mut self.tallennus.luvut_jono, tulos.clone());
                    self.pyynnot.push(Pyynto::Laheta(Nettipeli::Luvut, tulos));
                }
            }
            Some(Kaynnissa::Tiilirikko(peli)) => {
                self.tallennus.tiilirikko = peli.paras;
                self.tallennus.tiilirikon_parhaat = peli.parhaat.clone();
                self.netti_tiilirikko = peli.netti_parhaat.clone();
                if let Some(tulos) = peli.lahetys.take() {
                    lisaa_tulos(&mut self.tallennus.tiilirikon_jono, tulos.clone());
                    self.pyynnot
                        .push(Pyynto::Laheta(Nettipeli::Tiilirikko, tulos));
                }
            }
            Some(Kaynnissa::Asteroidit(peli)) => {
                self.tallennus.asteroidit = peli.paras;
                self.tallennus.asteroidien_parhaat = peli.parhaat.clone();
                self.netti_asteroidit = peli.netti_parhaat.clone();
                if let Some(tulos) = peli.lahetys.take() {
                    lisaa_tulos(&mut self.tallennus.asteroidien_jono, tulos.clone());
                    self.pyynnot
                        .push(Pyynto::Laheta(Nettipeli::Asteroidit, tulos));
                }
            }
            Some(Kaynnissa::Silmukka(peli)) => {
                self.tallennus.silmukka = peli.paras;
                self.tallennus.silmukan_parhaat = peli.parhaat.clone();
                self.netti_silmukka = peli.netti_parhaat.clone();
                if let Some(tulos) = peli.lahetys.take() {
                    lisaa_tulos(&mut self.tallennus.silmukan_jono, tulos.clone());
                    self.pyynnot
                        .push(Pyynto::Laheta(Nettipeli::Silmukka, tulos));
                }
            }
            Some(Kaynnissa::Palikka(peli)) => {
                self.tallennus.palikka = peli.paras;
                self.tallennus.palikan_parhaat = peli.parhaat.clone();
                self.netti_palikka = peli.netti_parhaat.clone();
                if let Some(tulos) = peli.lahetys.take() {
                    lisaa_tulos(&mut self.tallennus.palikan_jono, tulos.clone());
                    self.pyynnot.push(Pyynto::Laheta(Nettipeli::Palikka, tulos));
                }
            }
            Some(Kaynnissa::Pasianssi(peli)) => {
                let [klondike, spider, pyramidi] = peli.parhaat.clone();
                self.tallennus.pasianssin_parhaat = klondike;
                self.tallennus.spiderin_parhaat = spider;
                self.tallennus.pyramidin_parhaat = pyramidi;
                self.netti_pasianssi = peli.netti_parhaat.clone();
                if let Some(tulos) = peli.lahetys.take() {
                    let jono = match peli.variantti {
                        Variantti::Klondike => &mut self.tallennus.pasianssin_jono,
                        Variantti::Spider => &mut self.tallennus.spiderin_jono,
                        Variantti::Pyramidi => &mut self.tallennus.pyramidin_jono,
                    };
                    lisaa_tulos(jono, tulos.clone());
                    self.pyynnot
                        .push(Pyynto::Laheta(pasianssin_nettipeli(peli.variantti), tulos));
                }
            }
            Some(Kaynnissa::Blackjack(peli)) => {
                self.tallennus.blackjackin_parhaat = peli.parhaat.clone();
                self.netti_blackjack = peli.netti_parhaat.clone();
                if let Some(tulos) = peli.lahetys.take() {
                    lisaa_tulos(&mut self.tallennus.blackjackin_jono, tulos.clone());
                    self.pyynnot
                        .push(Pyynto::Laheta(Nettipeli::Blackjack, tulos));
                }
            }
            Some(Kaynnissa::Holdem(peli)) => {
                self.tallennus.holdemin_parhaat = peli.parhaat.clone();
                self.netti_holdem = peli.netti_parhaat.clone();
                if let Some(tulos) = peli.lahetys.take() {
                    lisaa_tulos(&mut self.tallennus.holdemin_jono, tulos.clone());
                    self.pyynnot.push(Pyynto::Laheta(Nettipeli::Holdem, tulos));
                }
            }
            Some(Kaynnissa::Miinaharava(peli)) => {
                self.tallennus.miinaajat = peli.ajat;
            }
            _ => {}
        }
    }

    /// Ikkuna suljetaan: nimeä odottava tulos kirjataan, ettei se katoa.
    pub fn viimeistele(&mut self) {
        match &mut self.peli {
            Some(Kaynnissa::Luvut(peli)) => peli.vahvista_nimi(),
            Some(Kaynnissa::Tiilirikko(peli)) => {
                peli.poistu();
                peli.vahvista_nimi();
            }
            Some(Kaynnissa::Asteroidit(peli)) => {
                peli.poistu();
                peli.vahvista_nimi();
            }
            Some(Kaynnissa::Silmukka(peli)) => {
                peli.poistu();
                peli.vahvista_nimi();
            }
            Some(Kaynnissa::Palikka(peli)) => {
                peli.poistu();
                peli.vahvista_nimi();
            }
            Some(Kaynnissa::Pasianssi(peli)) => peli.vahvista_nimi(),
            Some(Kaynnissa::Blackjack(peli)) => {
                peli.poistu();
                peli.vahvista_nimi();
            }
            Some(Kaynnissa::Holdem(peli)) => {
                peli.poistu();
                peli.vahvista_nimi();
            }
            _ => {}
        }
        self.paivita_tallennus();
    }

    pub fn edista(&mut self, ms: u32) {
        self.kello_ms = self.kello_ms.saturating_add(ms);
        match &mut self.peli {
            Some(Kaynnissa::Laivanupotus(peli)) => peli.edista(ms),
            Some(Kaynnissa::Luvut(peli)) => peli.edista(ms),
            Some(Kaynnissa::Pasianssi(peli)) => peli.edista(ms),
            Some(Kaynnissa::Tiilirikko(peli)) => {
                peli.edista(ms);
                // Tiilirikko päättyy ajan kuluessa eikä syötteeseen, joten ennätys kirjataan täällä.
                if std::mem::take(&mut peli.uusi_tulos) {
                    self.paivita_tallennus();
                    self.tallenna = true;
                }
            }
            Some(Kaynnissa::Asteroidit(peli)) => {
                peli.edista(ms);
                // Sama kuin Tiilirikossa: peli päättyy ajan kuluessa eikä syötteeseen.
                if std::mem::take(&mut peli.uusi_tulos) {
                    self.paivita_tallennus();
                    self.tallenna = true;
                }
            }
            Some(Kaynnissa::Silmukka(peli)) => {
                peli.edista(ms);
                // Sama kuin Tiilirikossa: peli päättyy ajan kuluessa eikä syötteeseen.
                if std::mem::take(&mut peli.uusi_tulos) {
                    self.paivita_tallennus();
                    self.tallenna = true;
                }
            }
            Some(Kaynnissa::Palikka(peli)) => {
                peli.edista(ms);
                // Sama kuin Tiilirikossa: peli päättyy ajan kuluessa eikä syötteeseen.
                if std::mem::take(&mut peli.uusi_tulos) {
                    self.paivita_tallennus();
                    self.tallenna = true;
                }
            }
            Some(Kaynnissa::Miinaharava(peli)) => peli.edista(ms),
            Some(Kaynnissa::Blackjack(peli)) => peli.edista(ms),
            Some(Kaynnissa::Holdem(peli)) => peli.edista(ms),
            None => {}
        }
    }

    pub fn ota_aanet(&mut self) -> Vec<(Aani, u32)> {
        match &mut self.peli {
            Some(Kaynnissa::Laivanupotus(peli)) => std::mem::take(&mut peli.aanijono),
            Some(Kaynnissa::Luvut(peli)) => std::mem::take(&mut peli.aanijono),
            Some(Kaynnissa::Pasianssi(peli)) => std::mem::take(&mut peli.aanijono),
            Some(Kaynnissa::Tiilirikko(peli)) => std::mem::take(&mut peli.aanijono),
            Some(Kaynnissa::Asteroidit(peli)) => std::mem::take(&mut peli.aanijono),
            Some(Kaynnissa::Silmukka(peli)) => std::mem::take(&mut peli.aanijono),
            Some(Kaynnissa::Palikka(peli)) => std::mem::take(&mut peli.aanijono),
            Some(Kaynnissa::Miinaharava(peli)) => std::mem::take(&mut peli.aanijono),
            Some(Kaynnissa::Blackjack(peli)) => std::mem::take(&mut peli.aanijono),
            Some(Kaynnissa::Holdem(peli)) => std::mem::take(&mut peli.aanijono),
            None => Vec::new(),
        }
    }

    pub fn kasittele(&mut self, syote: Syote) {
        self.syotteita = self.syotteita.wrapping_add(1);
        let aanet_ennen = self.aanet_paalla;
        let kieli_ennen = kieli::nykyinen();
        self.kasittele_nakyma(syote);
        let uusi_tulos = match &mut self.peli {
            Some(Kaynnissa::Luvut(peli)) => std::mem::take(&mut peli.uusi_tulos),
            Some(Kaynnissa::Tiilirikko(peli)) => std::mem::take(&mut peli.uusi_tulos),
            Some(Kaynnissa::Asteroidit(peli)) => std::mem::take(&mut peli.uusi_tulos),
            Some(Kaynnissa::Silmukka(peli)) => std::mem::take(&mut peli.uusi_tulos),
            Some(Kaynnissa::Palikka(peli)) => std::mem::take(&mut peli.uusi_tulos),
            Some(Kaynnissa::Miinaharava(peli)) => std::mem::take(&mut peli.uusi_tulos),
            Some(Kaynnissa::Pasianssi(peli)) => std::mem::take(&mut peli.uusi_tulos),
            Some(Kaynnissa::Blackjack(peli)) => std::mem::take(&mut peli.uusi_tulos),
            Some(Kaynnissa::Holdem(peli)) => std::mem::take(&mut peli.uusi_tulos),
            _ => false,
        };
        if uusi_tulos || self.aanet_paalla != aanet_ennen || kieli::nykyinen() != kieli_ennen {
            self.paivita_tallennus();
            self.tallenna = true;
        }
    }

    fn kasittele_nakyma(&mut self, syote: Syote) {
        let poistuu = match &mut self.peli {
            Some(Kaynnissa::Laivanupotus(peli)) => {
                peli.kasittele(syote);
                self.aanet_paalla = peli.aanet_paalla;
                self.vaikeus = peli.vaikeus;
                peli.lopeta
            }
            Some(Kaynnissa::Luvut(peli)) => {
                peli.kasittele(syote);
                self.aanet_paalla = peli.aanet_paalla;
                peli.lopeta
            }
            Some(Kaynnissa::Pasianssi(peli)) => {
                peli.kasittele(syote);
                self.aanet_paalla = peli.aanet_paalla;
                self.pasianssin_vaikeus = peli.vaikeus;
                self.pasianssin_variantti = peli.variantti;
                peli.lopeta
            }
            Some(Kaynnissa::Tiilirikko(peli)) => {
                peli.kasittele(syote);
                self.aanet_paalla = peli.aanet_paalla;
                peli.lopeta
            }
            Some(Kaynnissa::Asteroidit(peli)) => {
                peli.kasittele(syote);
                self.aanet_paalla = peli.aanet_paalla;
                peli.lopeta
            }
            Some(Kaynnissa::Silmukka(peli)) => {
                peli.kasittele(syote);
                self.aanet_paalla = peli.aanet_paalla;
                peli.lopeta
            }
            Some(Kaynnissa::Palikka(peli)) => {
                peli.kasittele(syote);
                self.aanet_paalla = peli.aanet_paalla;
                peli.lopeta
            }
            Some(Kaynnissa::Miinaharava(peli)) => {
                peli.kasittele(syote);
                self.aanet_paalla = peli.aanet_paalla;
                self.miinaharavan_vaikeus = peli.vaikeus;
                peli.lopeta
            }
            Some(Kaynnissa::Blackjack(peli)) => {
                peli.kasittele(syote);
                self.aanet_paalla = peli.aanet_paalla;
                peli.lopeta
            }
            Some(Kaynnissa::Holdem(peli)) => {
                peli.kasittele(syote);
                self.aanet_paalla = peli.aanet_paalla;
                peli.lopeta
            }
            None => {
                self.kasittele_valikko(syote);
                return;
            }
        };
        if poistuu {
            self.paivita_tallennus();
            self.tallenna = true;
            self.peli = None;
            self.hiljenna = true;
        }
    }

    fn kasittele_valikko(&mut self, syote: Syote) {
        let kytkimella = matches!(syote, Syote::Klikkaus(x, y) if piirto::aanikytkimella(x, y));
        if syote == Syote::VaihdaAanet || kytkimella {
            self.aanet_paalla = !self.aanet_paalla;
            return;
        }
        // Vain valikossa: pelien viestit on jo muotoiltu sen hetken kielellä.
        let kielikytkimella =
            matches!(syote, Syote::Klikkaus(x, y) if piirto::kielikytkimella(x, y));
        if syote == Syote::Kirjain('L') || kielikytkimella {
            kieli::aseta(kieli::nykyinen().toinen());
            return;
        }
        match syote {
            Syote::Peru => self.lopeta = true,
            Syote::Ammu => self.kaynnista(PELIT[self.valittu].laji),
            Syote::Hiiri(x, y) => {
                if let Some(i) = piirto::kortti_pisteesta(x, y) {
                    self.valitse(i);
                }
            }
            Syote::Klikkaus(x, y) => {
                if let Some(i) = piirto::kortti_pisteesta(x, y) {
                    self.valitse(i);
                    self.kaynnista(PELIT[i].laji);
                }
            }
            Syote::Numero(numero) => {
                // Kymmenes kortti on nolla, kuten numerorivillä.
                let i = (numero as usize + 9) % 10;
                if i < PELIT.len() {
                    self.valitse(i);
                }
            }
            Syote::Nuoli(drivi, _) => {
                let i = self.valittu.saturating_add_signed(drivi as isize);
                self.valitse(i.min(PELIT.len() - 1));
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pelit::laivanupotus::peli::Vaihe;
    use crate::pelit::luvut::peli as luvut;
    use crate::yhteinen::piirto::{Kehys, VILKKU_MS};
    use klassikot8000::laivanupotus::{KOKO, Ruutu};
    use klassikot8000::luvut::Lukulauta;

    fn laivaruudut(peli: &Peli) -> Vec<bool> {
        (0..KOKO * KOKO)
            .map(|i| peli.oma.on_laiva(Ruutu::new(i / KOKO, i % KOKO).unwrap()))
            .collect()
    }

    fn laivanupotus(k: &Kaynnistin) -> &Peli {
        match &k.peli {
            Some(Kaynnissa::Laivanupotus(peli)) => peli,
            _ => panic!("laivanupotus ei ole käynnissä"),
        }
    }

    fn luvut(k: &mut Kaynnistin) -> &mut Luvut {
        match &mut k.peli {
            Some(Kaynnissa::Luvut(peli)) => peli,
            _ => panic!("2048 ei ole käynnissä"),
        }
    }

    #[test]
    fn valikko_kaynnistaa_pelin_ja_esc_palaa_valikkoon() {
        let mut k = Kaynnistin::new(1);
        assert!(k.peli.is_none());
        k.kasittele(Syote::Ammu);
        assert_eq!(laivanupotus(&k).vaihe, Vaihe::Alku);
        assert_eq!(k.otsikko(), "Laivanupotus");

        k.kasittele(Syote::Peru);
        assert!(k.peli.is_none());
        assert!(k.hiljenna);
        assert!(!k.lopeta);

        k.kasittele(Syote::Peru);
        assert!(k.lopeta);
    }

    #[test]
    fn jokainen_peli_valitaan_ja_kaynnistetaan_korttinsa_kautta() {
        let mut k = Kaynnistin::new(1);
        k.kasittele(Syote::Numero(2));
        assert_eq!(k.valittu, 1);
        k.kasittele(Syote::Ammu);
        assert_eq!(luvut(&mut k).vaihe, luvut::Vaihe::Alku);
        assert_eq!(k.otsikko(), "2048");
        k.kasittele(Syote::Peru);

        k.kasittele(Syote::Nuoli(-1, 0));
        k.kasittele(Syote::Nuoli(-1, 0));
        assert_eq!(k.valittu, 0);
        k.kasittele(Syote::Nuoli(1, 0));
        k.kasittele(Syote::Nuoli(1, 0));
        k.kasittele(Syote::Nuoli(1, 0));
        k.kasittele(Syote::Nuoli(1, 0));
        k.kasittele(Syote::Nuoli(1, 0));
        assert_eq!(k.valittu, 5);
        k.kasittele(Syote::Nuoli(1, 0));
        k.kasittele(Syote::Nuoli(1, 0));
        k.kasittele(Syote::Nuoli(1, 0));
        k.kasittele(Syote::Nuoli(1, 0));
        k.kasittele(Syote::Nuoli(1, 0));
        assert_eq!(k.valittu, 9);
        k.kasittele(Syote::Numero(1));
        assert_eq!(k.valittu, 0);
        k.kasittele(Syote::Numero(0));
        assert_eq!(k.valittu, 9);

        let (x, y) = piirto::kortin_keskipiste(0);
        k.kasittele(Syote::Hiiri(x, y));
        assert_eq!(k.valittu, 0);
        let (x, y) = piirto::kortin_keskipiste(1);
        k.kasittele(Syote::Klikkaus(x, y));
        assert!(matches!(k.peli, Some(Kaynnissa::Luvut(_))));
        k.kasittele(Syote::Peru);

        let (x, y) = piirto::kortin_keskipiste(2);
        k.kasittele(Syote::Klikkaus(x, y));
        assert!(matches!(k.peli, Some(Kaynnissa::Pasianssi(_))));
        assert_eq!(k.otsikko(), "Pasianssi");
        k.kasittele(Syote::Peru);

        let (x, y) = piirto::kortin_keskipiste(3);
        k.kasittele(Syote::Klikkaus(x, y));
        assert!(matches!(k.peli, Some(Kaynnissa::Tiilirikko(_))));
        assert_eq!(k.otsikko(), "Laatan murskain");
        k.kasittele(Syote::Peru);

        let (x, y) = piirto::kortin_keskipiste(4);
        k.kasittele(Syote::Klikkaus(x, y));
        assert!(matches!(k.peli, Some(Kaynnissa::Asteroidit(_))));
        assert_eq!(k.otsikko(), "Asteroidit");
        k.kasittele(Syote::Peru);

        let (x, y) = piirto::kortin_keskipiste(5);
        k.kasittele(Syote::Klikkaus(x, y));
        assert!(matches!(k.peli, Some(Kaynnissa::Silmukka(_))));
        assert_eq!(k.otsikko(), "Mato8000");
        assert!(!k.nuolet_toistuvat());
        k.kasittele(Syote::Peru);

        let (x, y) = piirto::kortin_keskipiste(6);
        k.kasittele(Syote::Klikkaus(x, y));
        assert!(matches!(k.peli, Some(Kaynnissa::Palikka(_))));
        assert_eq!(k.otsikko(), "Palikka8000");
        assert!(k.nuolet_toistuvat());
        k.kasittele(Syote::Peru);

        let (x, y) = piirto::kortin_keskipiste(7);
        k.kasittele(Syote::Klikkaus(x, y));
        assert!(matches!(k.peli, Some(Kaynnissa::Miinaharava(_))));
        assert_eq!(k.otsikko(), "Miinaharava");
        k.kasittele(Syote::Peru);

        let (x, y) = piirto::kortin_keskipiste(8);
        k.kasittele(Syote::Klikkaus(x, y));
        assert!(matches!(k.peli, Some(Kaynnissa::Blackjack(_))));
        assert_eq!(k.otsikko(), "Blackjack");
        k.kasittele(Syote::Peru);

        let (x, y) = piirto::kortin_keskipiste(9);
        k.kasittele(Syote::Klikkaus(x, y));
        assert!(matches!(k.peli, Some(Kaynnissa::Holdem(_))));
        assert_eq!(k.otsikko(), "Texas Hold'em");
        k.kasittele(Syote::Peru);

        k.kasittele(Syote::Numero(9));
        assert_eq!(k.valittu, 8);
    }

    #[test]
    fn toimintapeli_nopeutuu_vasta_alkuruudun_jalkeen() {
        for laji in [Laji::Tiilirikko, Laji::Asteroidit, Laji::Silmukka] {
            let mut k = Kaynnistin::new(1);
            assert!(!k.toimintapeli());
            k.kaynnista(laji);
            assert!(!k.toimintapeli(), "{laji:?}");
            k.kasittele(Syote::Ammu);
            assert!(k.toimintapeli(), "{laji:?}");
        }
        let mut k = Kaynnistin::new(1);
        k.kaynnista(Laji::Luvut);
        k.kasittele(Syote::Ammu);
        assert!(!k.toimintapeli());
    }

    #[test]
    fn valikon_avain_vaihtuu_vain_kun_kuva_voi_muuttua() {
        let mut k = Kaynnistin::new(1);
        // kasvun aikana kuva vaihtuu joka kierroksella
        let kasvussa = k.kuvan_avain(true);
        k.edista(1);
        assert_ne!(k.kuvan_avain(true), kasvussa);

        k.edista(10 * VALIKON_ASKEL_MS - 1);
        let alku = k.kuvan_avain(true);
        assert!(alku.is_some());
        k.edista(VALIKON_ASKEL_MS - 1);
        assert_eq!(k.kuvan_avain(true), alku);
        k.edista(1);
        assert_ne!(k.kuvan_avain(true), alku);

        // taustalla kello ei näy kuvassa, mutta hiiren valinta ja asetukset näkyvät
        let taustalla = k.kuvan_avain(false);
        k.edista(10 * VALIKON_ASKEL_MS);
        assert_eq!(k.kuvan_avain(false), taustalla);
        k.kasittele(Syote::Numero(2));
        assert_ne!(k.kuvan_avain(false), taustalla);

        // ilmoitus uudesta versiosta tulee näkyviin, vaikka valikko olisi taustalla
        let ilman_ilmoitusta = k.kuvan_avain(false);
        k.vastaanota(Vastaus::UusiVersio("9.9.9".to_string()));
        assert_ne!(k.kuvan_avain(false), ilman_ilmoitusta);

        k.kasittele(Syote::Ammu);
        assert_eq!(k.kuvan_avain(true), None);
    }

    #[test]
    fn pasianssin_avain_vaihtuu_syotteesta_kellosta_ja_vilkusta() {
        let mut k = Kaynnistin::new(1);
        k.kaynnista(Laji::Pasianssi);
        // alkuruudun malli pyörii
        assert_eq!(k.kuvan_avain(true), None);
        k.kasittele(Syote::Ammu);
        let Some(Kaynnissa::Pasianssi(peli)) = &mut k.peli else {
            panic!("pasianssi ei käynnistynyt");
        };
        assert_eq!(peli.vaihe, pasianssi_peli::Vaihe::Pelissa);

        // ilman kursoria vain näkyvä peliaika muuttaa kuvaa
        let alku = k.kuvan_avain(true);
        assert!(alku.is_some());
        k.edista(999);
        assert_eq!(k.kuvan_avain(true), alku);
        k.edista(1);
        assert_ne!(k.kuvan_avain(true), alku);

        let ennen = k.kuvan_avain(true);
        k.kasittele(Syote::Hiiri(5, 5));
        assert_ne!(k.kuvan_avain(true), ennen);

        // näkyvä kursori vilkkuu samassa tahdissa kuin se piirretään
        k.kasittele(Syote::Nuoli(0, 1));
        let Some(Kaynnissa::Pasianssi(peli)) = &k.peli else {
            panic!("pasianssi loppui");
        };
        assert!(peli.kursori_nakyy);
        let alku_ms = peli.kello_ms;
        let seuraava_vilkku = (alku_ms / VILKKU_MS + 1) * VILKKU_MS;
        let ennen = k.kuvan_avain(true);
        k.edista(seuraava_vilkku - alku_ms);
        assert_ne!(k.kuvan_avain(true), ennen);

        // voittojuhla liikkuu joka ruudussa
        let Some(Kaynnissa::Pasianssi(peli)) = &mut k.peli else {
            panic!("pasianssi loppui");
        };
        peli.vaihe = pasianssi_peli::Vaihe::Voitto;
        assert_eq!(k.kuvan_avain(true), None);
    }

    #[test]
    fn blackjackin_kuva_vaihtuu_vain_syotteesta_ja_jakajan_siirroista() {
        let mut k = Kaynnistin::new(1);
        k.kaynnista(Laji::Blackjack);
        // alkuruudun malli pyörii
        assert_eq!(k.kuvan_avain(true), None);
        assert_eq!(k.nakyman_piste(285, 0), (0, 0));
        k.kasittele(Syote::Ammu);
        assert_eq!(k.nakyman_piste(285, 7), (285, 7));
        assert!(!k.toimintapeli());

        let lepo = k.kuvan_avain(true);
        assert!(lepo.is_some());
        k.edista(60_000);
        assert_eq!(k.kuvan_avain(true), lepo);
        k.kasittele(Syote::Nuoli(0, 1));
        assert_ne!(k.kuvan_avain(true), lepo);

        // jakaja siirtää ilman syötettä, joten kuva vaihtuu pelkästä ajasta
        let Some(Kaynnissa::Blackjack(peli)) = &mut k.peli else {
            panic!("blackjack ei ole käynnissä");
        };
        let mut pakka = vec![
            klassikot8000::blackjack::Kortti {
                maa: klassikot8000::blackjack::Maa::Pata,
                arvo: 10,
            };
            100
        ];
        pakka[99].arvo = 9;
        peli.poyta.kenka = pakka;
        k.kasittele(Syote::Ammu);
        k.kasittele(Syote::Kirjain('J'));
        let ennen = k.kuvan_avain(true);
        k.edista(1000);
        assert_ne!(k.kuvan_avain(true), ennen);
    }

    #[test]
    fn holdemin_kuva_vaihtuu_vain_syotteesta_ja_siirroista() {
        let mut k = Kaynnistin::new(1);
        k.kaynnista(Laji::Holdem);
        // alkuruudun malli pyörii
        assert_eq!(k.kuvan_avain(true), None);
        assert_eq!(k.nakyman_piste(285, 0), (0, 0));
        k.kasittele(Syote::Ammu);
        assert_eq!(k.nakyman_piste(285, 7), (285, 7));
        assert!(!k.toimintapeli());

        // Koneet siirtävät ilman syötettä, ja pelaajan vuorolla kuva pysyy paikallaan.
        k.edista(1_000_000);
        let lepo = k.kuvan_avain(true);
        assert!(lepo.is_some());
        k.edista(60_000);
        assert_eq!(k.kuvan_avain(true), lepo);
        k.kasittele(Syote::Hiiri(0, 0));
        assert_ne!(k.kuvan_avain(true), lepo);
    }

    #[test]
    fn valinta_vaihtaa_mallin() {
        let mut k = Kaynnistin::new(1);
        let laiva = k.malli.len();
        assert!(k.valitun_parhaat().is_none());
        k.edista(1000);
        k.kasittele(Syote::Numero(5));
        assert_ne!(k.malli.len(), laiva);
        assert!(!k.malli.is_empty());
        assert_eq!(k.valittu_ms, 1000);

        k.tallennus.asteroidien_parhaat = vec![Tulos {
            nimi: "A".to_string(),
            pisteet: 100,
        }];
        assert_eq!(k.valitun_parhaat().unwrap().len(), 1);
        k.kasittele(Syote::Numero(8));
        assert!(k.valitun_parhaat().is_none());
    }

    /// Nimi vahvistetaan, ja tulos on kirjattu tallennukseen ja jonoon. Sama läpi uuden käynnistyksen.
    fn tarkista_korttipelin_tulos(k: &mut Kaynnistin, peli: Nettipeli, pisteet: u32) {
        k.kasittele(Syote::Kirjain('A'));
        k.kasittele(Syote::Ammu);
        let tulos = Tulos {
            nimi: "A".to_string(),
            pisteet,
        };
        assert!(k.pyynnot.contains(&Pyynto::Laheta(peli, tulos.clone())));
        assert!(k.tallenna);

        let mut uusi = Kaynnistin::new(2);
        uusi.lataa(Tallennus::tekstista(&k.tallennus.tekstiksi()));
        assert_eq!(uusi.pyynnot, [Pyynto::Laheta(peli, tulos.clone())]);
        uusi.vastaanota(Vastaus::Lahetetty(peli, tulos.clone()));
        let (parhaat, jono) = match peli {
            Nettipeli::Pasianssi => (
                &uusi.tallennus.pasianssin_parhaat,
                &uusi.tallennus.pasianssin_jono,
            ),
            Nettipeli::Spider => (
                &uusi.tallennus.spiderin_parhaat,
                &uusi.tallennus.spiderin_jono,
            ),
            Nettipeli::Pyramidi => (
                &uusi.tallennus.pyramidin_parhaat,
                &uusi.tallennus.pyramidin_jono,
            ),
            Nettipeli::Blackjack => (
                &uusi.tallennus.blackjackin_parhaat,
                &uusi.tallennus.blackjackin_jono,
            ),
            _ => (
                &uusi.tallennus.holdemin_parhaat,
                &uusi.tallennus.holdemin_jono,
            ),
        };
        assert_eq!(parhaat[..], [tulos]);
        assert!(jono.is_empty());
    }

    #[test]
    fn pasianssin_voitto_kirjautuu_ajan_mukaan() {
        let mut k = Kaynnistin::new(1);
        k.kaynnista(Laji::Pasianssi);
        assert!(k.pyynnot.contains(&Pyynto::Hae(Nettipeli::Pasianssi)));
        k.kasittele(Syote::Ammu);
        let Some(Kaynnissa::Pasianssi(peli)) = &mut k.peli else {
            panic!("pasianssi ei ole käynnissä");
        };
        peli.peliaika_ms = 300_000;
        peli.vaihe = pasianssi_peli::Vaihe::Nimi;
        tarkista_korttipelin_tulos(
            &mut k,
            Nettipeli::Pasianssi,
            pasianssi::aikapisteet(pasianssi::Vaikeus::Normaali, 300),
        );
    }

    #[test]
    fn spiderin_ja_pyramidin_tulokset_menevat_omille_listoilleen() {
        for (variantti, netti) in [
            (Variantti::Spider, Nettipeli::Spider),
            (Variantti::Pyramidi, Nettipeli::Pyramidi),
        ] {
            let mut k = Kaynnistin::new(1);
            k.kaynnista(Laji::Pasianssi);
            for haettava in [Nettipeli::Pasianssi, Nettipeli::Spider, Nettipeli::Pyramidi] {
                assert!(k.pyynnot.contains(&Pyynto::Hae(haettava)));
            }
            k.pyynnot.clear();
            let Some(Kaynnissa::Pasianssi(peli)) = &mut k.peli else {
                panic!("pasianssi ei ole käynnissä");
            };
            peli.variantti = variantti;
            peli.aloita();
            peli.peliaika_ms = 300_000;
            peli.vaihe = pasianssi_peli::Vaihe::Nimi;
            tarkista_korttipelin_tulos(
                &mut k,
                netti,
                pasianssi::aikapisteet(pasianssi::Vaikeus::Normaali, 300),
            );
            assert!(k.tallennus.pasianssin_parhaat.is_empty());
        }
    }

    #[test]
    fn valikko_nayttaa_viimeksi_valitun_lajin_listan_ja_nettilista_menee_lajille() {
        let mut k = Kaynnistin::new(1);
        k.tallennus.spiderin_parhaat = vec![Tulos {
            nimi: "S".to_string(),
            pisteet: 9,
        }];
        k.valitse(
            PELIT
                .iter()
                .position(|kortti| kortti.laji == Laji::Pasianssi)
                .unwrap(),
        );
        assert!(k.valitun_parhaat().unwrap().is_empty());
        k.kaynnista(Laji::Pasianssi);
        k.kasittele(Syote::Nuoli(1, 0));
        let lista = vec![Tulos {
            nimi: "NETTI".to_string(),
            pisteet: 7,
        }];
        k.vastaanota(Vastaus::Lista(Nettipeli::Spider, lista.clone()));
        let Some(Kaynnissa::Pasianssi(peli)) = &k.peli else {
            panic!("pasianssi ei ole käynnissä");
        };
        assert_eq!(peli.nakyva_lista(), &lista[..]);
        assert!(peli.netti_parhaat[Variantti::Klondike as usize].is_none());
        k.kasittele(Syote::Peru);
        assert_eq!(k.valitun_parhaat().unwrap()[0].nimi, "S");
    }

    #[test]
    fn blackjackin_voitto_kirjautuu_poistuttaessa_ja_tappio_ei() {
        let mut k = Kaynnistin::new(1);
        k.kaynnista(Laji::Blackjack);
        k.kasittele(Syote::Ammu);
        k.kasittele(Syote::Peru);
        assert!(k.peli.is_none());
        assert!(k.tallennus.blackjackin_parhaat.is_empty());

        k.kaynnista(Laji::Blackjack);
        k.kasittele(Syote::Ammu);
        let Some(Kaynnissa::Blackjack(peli)) = &mut k.peli else {
            panic!("blackjack ei ole käynnissä");
        };
        peli.poyta.rahat += 40 * klassikot8000::blackjack::YKSI;
        k.kasittele(Syote::Peru);
        assert!(k.peli.is_some());
        tarkista_korttipelin_tulos(&mut k, Nettipeli::Blackjack, 40);
        assert!(k.peli.is_none());
    }

    #[test]
    fn holdemin_voitto_kirjautuu_poistuttaessa() {
        let mut k = Kaynnistin::new(1);
        k.kaynnista(Laji::Holdem);
        k.kasittele(Syote::Ammu);
        let Some(Kaynnissa::Holdem(peli)) = &mut k.peli else {
            panic!("hold'em ei ole käynnissä");
        };
        peli.poyta.pelaajat[klassikot8000::holdem::PELAAJA].rahat =
            klassikot8000::holdem::ALKURAHA + 250;
        k.kasittele(Syote::Kirjain('E'));
        assert!(k.peli.is_some());
        tarkista_korttipelin_tulos(&mut k, Nettipeli::Holdem, 250);
        assert!(k.peli.is_none());
    }

    #[test]
    fn suljettu_ikkuna_kirjaa_korttipelin_voiton() {
        let mut k = Kaynnistin::new(1);
        k.kaynnista(Laji::Blackjack);
        k.kasittele(Syote::Ammu);
        let Some(Kaynnissa::Blackjack(peli)) = &mut k.peli else {
            panic!("blackjack ei ole käynnissä");
        };
        peli.poyta.rahat += 7 * klassikot8000::blackjack::YKSI;
        k.viimeistele();
        assert_eq!(k.tallennus.blackjackin_parhaat[0].pisteet, 7);
    }

    #[test]
    fn palikan_tulos_tallentuu_jonottaa_ja_palautuu() {
        let mut k = Kaynnistin::new(1);
        k.kaynnista(Laji::Palikka);
        assert_eq!(
            std::mem::take(&mut k.pyynnot),
            [Pyynto::Hae(Nettipeli::Palikka)]
        );
        k.kasittele(Syote::Ammu);
        let Some(Kaynnissa::Palikka(peli)) = &mut k.peli else {
            panic!("palikka ei ole käynnissä");
        };
        peli.lauta.pisteet = 350;
        peli.paras = 350;
        peli.lauta.ohi = true;
        k.edista(1);
        assert!(k.tallenna);
        assert_eq!(k.tallennus.palikka, 350);
        k.kasittele(Syote::Kirjain('A'));
        k.kasittele(Syote::Ammu);

        let tulos = Tulos {
            nimi: "A".to_string(),
            pisteet: 350,
        };
        assert_eq!(
            std::mem::take(&mut k.pyynnot),
            [Pyynto::Laheta(Nettipeli::Palikka, tulos.clone())]
        );

        let mut uusi = Kaynnistin::new(2);
        uusi.lataa(Tallennus::tekstista(&k.tallennus.tekstiksi()));
        assert_eq!(
            uusi.pyynnot,
            [Pyynto::Laheta(Nettipeli::Palikka, tulos.clone())]
        );
        uusi.vastaanota(Vastaus::Lahetetty(Nettipeli::Palikka, tulos));
        assert!(uusi.tallennus.palikan_jono.is_empty());
        uusi.kaynnista(Laji::Palikka);
        let Some(Kaynnissa::Palikka(peli)) = &uusi.peli else {
            panic!("palikka ei ole käynnissä");
        };
        assert_eq!(peli.paras, 350);
        assert_eq!(peli.parhaat[0].nimi, "A");
    }

    #[test]
    fn palikan_avain_vaihtuu_vain_toistossa_ja_vilkussa() {
        let mut k = Kaynnistin::new(1);
        k.kaynnista(Laji::Palikka);
        assert_eq!(k.kuvan_avain(true), None);
        k.kasittele(Syote::Ammu);
        assert!(!k.toimintapeli());
        let lepo = k.kuvan_avain(true);
        assert!(lepo.is_some());
        k.edista(1);
        assert_eq!(k.kuvan_avain(true), lepo);

        let Some(Kaynnissa::Palikka(peli)) = &mut k.peli else {
            panic!("palikka ei ole käynnissä");
        };
        let (a, b) = peli.lauta.vihje().unwrap();
        peli.vaihda(a, b);
        // toisto saa sulavan päivityksen, muuten taajuus pysyy matalana
        assert!(k.toimintapeli());
        let toisto = k.kuvan_avain(true);
        k.edista(1);
        assert_ne!(k.kuvan_avain(true), toisto);
    }

    #[test]
    fn miinaharavan_taso_ja_ennatys_sailyvat() {
        let mut k = Kaynnistin::new(1);
        k.kaynnista(Laji::Miinaharava);
        k.kasittele(Syote::Numero(2));
        k.kasittele(Syote::Peru);
        k.kaynnista(Laji::Miinaharava);
        k.kasittele(Syote::Ammu);
        let Some(Kaynnissa::Miinaharava(peli)) = &mut k.peli else {
            panic!("miinaharava ei ole käynnissä");
        };
        assert_eq!(peli.lauta.vaikeus, miinaharava::Vaikeus::Normaali);
        peli.lauta = miinaharava::Miinakentta::miinoista(miinaharava::Vaikeus::Normaali, &[(0, 0)]);
        peli.avaa((1, 1));
        peli.edista(12_000);
        peli.avaa((15, 15));
        assert_eq!(peli.vaihe, crate::pelit::miinaharava::peli::Vaihe::Voitto);
        k.kasittele(Syote::Hiiri(0, 0));
        assert!(k.tallenna);
        assert_eq!(k.tallennus.miinaajat, [0, 12, 0]);

        let mut uusi = Kaynnistin::new(2);
        uusi.lataa(Tallennus::tekstista(&k.tallennus.tekstiksi()));
        uusi.kaynnista(Laji::Miinaharava);
        let Some(Kaynnissa::Miinaharava(peli)) = &uusi.peli else {
            panic!("miinaharava ei ole käynnissä");
        };
        assert_eq!(peli.ajat, [0, 12, 0]);
    }

    #[test]
    fn silmukan_tulos_tallentuu_jonottaa_ja_palautuu() {
        use klassikot8000::silmukka::{ASKEL_MS, Laji as Otuslaji, Otus};

        let mut k = Kaynnistin::new(1);
        k.kaynnista(Laji::Silmukka);
        assert_eq!(
            std::mem::take(&mut k.pyynnot),
            [Pyynto::Hae(Nettipeli::Silmukka)]
        );
        k.kasittele(Syote::Ammu);
        let Some(Kaynnissa::Silmukka(peli)) = &mut k.peli else {
            panic!("silmukka ei ole käynnissä");
        };
        let paa = peli.lauta.paa();
        peli.lauta.pisteet = 225;
        let mut itikka = Otus::uusi(Otuslaji::Itikka, (paa.0 + 1, paa.1), (1, 1), 0);
        itikka.odotus_ms = 100_000;
        itikka.jakso_ms = 100_000;
        peli.lauta.otukset = vec![itikka];
        peli.lauta.odotus_ms = ASKEL_MS;
        k.edista(ASKEL_MS);
        assert!(k.tallenna);
        assert_eq!(k.tallennus.silmukka, 225);
        k.kasittele(Syote::Kirjain('A'));
        k.kasittele(Syote::Ammu);

        let tulos = Tulos {
            nimi: "A".to_string(),
            pisteet: 225,
        };
        assert_eq!(
            std::mem::take(&mut k.pyynnot),
            [Pyynto::Laheta(Nettipeli::Silmukka, tulos.clone())]
        );

        let mut uusi = Kaynnistin::new(2);
        uusi.lataa(Tallennus::tekstista(&k.tallennus.tekstiksi()));
        assert_eq!(
            uusi.pyynnot,
            [Pyynto::Laheta(Nettipeli::Silmukka, tulos.clone())]
        );
        uusi.vastaanota(Vastaus::Lahetetty(Nettipeli::Silmukka, tulos));
        assert!(uusi.tallennus.silmukan_jono.is_empty());
        uusi.kaynnista(Laji::Silmukka);
        let Some(Kaynnissa::Silmukka(peli)) = &uusi.peli else {
            panic!("silmukka ei ole käynnissä");
        };
        assert_eq!(peli.paras, 225);
        assert_eq!(peli.parhaat[0].nimi, "A");
    }

    #[test]
    fn asteroidien_tulos_tallentuu_jonottaa_ja_palautuu() {
        use klassikot8000::asteroidit::{Kivi, Koko};

        let mut k = Kaynnistin::new(1);
        k.kaynnista(Laji::Asteroidit);
        assert_eq!(
            std::mem::take(&mut k.pyynnot),
            [Pyynto::Hae(Nettipeli::Asteroidit)]
        );
        k.kasittele(Syote::Ammu);
        let Some(Kaynnissa::Asteroidit(peli)) = &mut k.peli else {
            panic!("asteroidit ei ole käynnissä");
        };
        peli.lauta.elamat = 1;
        peli.lauta.kivet = vec![Kivi {
            paikka: peli.lauta.alus.paikka,
            nopeus: (0.0, 0.0),
            koko: Koko::Pieni,
            kulma: 0.0,
            pyorinta: 0.0,
            muoto: 0,
        }];
        k.edista(5);
        assert!(k.tallenna);
        assert_eq!(k.tallennus.asteroidit, 100);
        k.kasittele(Syote::Kirjain('A'));
        k.kasittele(Syote::Ammu);

        let tulos = Tulos {
            nimi: "A".to_string(),
            pisteet: 100,
        };
        assert_eq!(
            std::mem::take(&mut k.pyynnot),
            [Pyynto::Laheta(Nettipeli::Asteroidit, tulos.clone())]
        );

        let mut uusi = Kaynnistin::new(2);
        uusi.lataa(Tallennus::tekstista(&k.tallennus.tekstiksi()));
        assert_eq!(
            uusi.pyynnot,
            [Pyynto::Laheta(Nettipeli::Asteroidit, tulos.clone())]
        );
        uusi.vastaanota(Vastaus::Lahetetty(Nettipeli::Asteroidit, tulos));
        assert!(uusi.tallennus.asteroidien_jono.is_empty());
        uusi.kaynnista(Laji::Asteroidit);
        let Some(Kaynnissa::Asteroidit(peli)) = &uusi.peli else {
            panic!("asteroidit ei ole käynnissä");
        };
        assert_eq!(peli.paras, 100);
        assert_eq!(peli.parhaat[0].nimi, "A");
    }

    #[test]
    fn nuolet_ohjaavat_asteroideja_pohjassa_ollessaan() {
        let mut k = Kaynnistin::new(1);
        k.kaynnista(Laji::Asteroidit);
        k.aseta_ohjaus(true, false, true);
        let Some(Kaynnissa::Asteroidit(peli)) = &k.peli else {
            panic!("asteroidit ei ole käynnissä");
        };
        assert_eq!(peli.ohjaus.kaanto, -1);
        assert!(peli.ohjaus.tyonto);
    }

    #[test]
    fn tiilirikon_ennatys_tallentuu_pelin_paattyessa_ja_palautuu() {
        let mut k = Kaynnistin::new(1);
        k.kaynnista(Laji::Tiilirikko);
        k.kasittele(Syote::Ammu);
        k.kasittele(Syote::Ammu);
        let Some(Kaynnissa::Tiilirikko(peli)) = &mut k.peli else {
            panic!("tiilirikko ei ole käynnissä");
        };
        peli.paras = 70;
        peli.lauta.elamat = 1;
        peli.lauta.pallot[0].paikka = (300.0, 600.0);
        peli.lauta.pallot[0].nopeus = (0.0, 300.0);
        k.edista(5);
        assert!(k.tallenna);
        assert_eq!(k.tallennus.tiilirikko, 70);

        let mut uusi = Kaynnistin::new(2);
        uusi.lataa(Tallennus::tekstista(&k.tallennus.tekstiksi()));
        uusi.kaynnista(Laji::Tiilirikko);
        let Some(Kaynnissa::Tiilirikko(peli)) = &uusi.peli else {
            panic!("tiilirikko ei ole käynnissä");
        };
        assert_eq!(peli.paras, 70);
    }

    #[test]
    fn tiilirikon_top3_tallentuu_nimen_jalkeen_ja_sulkiessa_ja_palautuu() {
        let paattyva = |k: &mut Kaynnistin| {
            k.kaynnista(Laji::Tiilirikko);
            k.kasittele(Syote::Ammu);
            k.kasittele(Syote::Ammu);
            let Some(Kaynnissa::Tiilirikko(peli)) = &mut k.peli else {
                panic!("tiilirikko ei ole käynnissä");
            };
            peli.lauta.pisteet = 70;
            peli.lauta.elamat = 1;
            peli.lauta.pallot[0].paikka = (300.0, 600.0);
            peli.lauta.pallot[0].nopeus = (0.0, 300.0);
            k.edista(5);
        };

        let mut k = Kaynnistin::new(1);
        paattyva(&mut k);
        k.tallenna = false;
        k.kasittele(Syote::Kirjain('A'));
        k.kasittele(Syote::Ammu);
        assert!(k.tallenna);
        assert_eq!(k.tallennus.tiilirikon_parhaat[0].nimi, "A");

        let mut uusi = Kaynnistin::new(2);
        uusi.lataa(Tallennus::tekstista(&k.tallennus.tekstiksi()));
        uusi.kaynnista(Laji::Tiilirikko);
        let Some(Kaynnissa::Tiilirikko(peli)) = &uusi.peli else {
            panic!("tiilirikko ei ole käynnissä");
        };
        assert_eq!(peli.parhaat[0].pisteet, 70);

        let mut k = Kaynnistin::new(1);
        paattyva(&mut k);
        k.viimeistele();
        assert_eq!(k.tallennus.tiilirikon_parhaat[0].nimi, "NIMETÖN");
    }

    #[test]
    fn nettitulos_jonottaa_tiedostossa_kunnes_lahetys_on_kuitattu() {
        let mut k = Kaynnistin::new(1);
        k.kaynnista(Laji::Tiilirikko);
        assert_eq!(
            std::mem::take(&mut k.pyynnot),
            [Pyynto::Hae(Nettipeli::Tiilirikko)]
        );
        k.vastaanota(Vastaus::Lista(Nettipeli::Tiilirikko, Vec::new()));
        k.kasittele(Syote::Ammu);
        k.kasittele(Syote::Ammu);
        let Some(Kaynnissa::Tiilirikko(peli)) = &mut k.peli else {
            panic!("tiilirikko ei ole käynnissä");
        };
        assert_eq!(peli.netti_parhaat, Some(Vec::new()));
        peli.lauta.pisteet = 70;
        peli.lauta.elamat = 1;
        peli.lauta.pallot[0].paikka = (300.0, 600.0);
        peli.lauta.pallot[0].nopeus = (0.0, 300.0);
        k.edista(5);
        k.kasittele(Syote::Kirjain('A'));
        k.kasittele(Syote::Ammu);

        let tulos = Tulos {
            nimi: "A".to_string(),
            pisteet: 70,
        };
        assert_eq!(
            std::mem::take(&mut k.pyynnot),
            [Pyynto::Laheta(Nettipeli::Tiilirikko, tulos.clone())]
        );
        assert_eq!(k.tallennus.tiilirikon_jono.len(), 1);

        // lähetys ei ehtinyt perille: seuraava käynnistys yrittää uudelleen
        let mut uusi = Kaynnistin::new(2);
        uusi.lataa(Tallennus::tekstista(&k.tallennus.tekstiksi()));
        assert_eq!(
            uusi.pyynnot,
            [Pyynto::Laheta(Nettipeli::Tiilirikko, tulos.clone())]
        );

        uusi.vastaanota(Vastaus::Lahetetty(Nettipeli::Tiilirikko, tulos));
        assert!(uusi.tallenna);
        assert!(uusi.tallennus.tiilirikon_jono.is_empty());
    }

    #[test]
    fn luvut_ja_tiilirikko_pitavat_nettilistansa_ja_jononsa_erillaan() {
        let tulos = |nimi: &str, pisteet| Tulos {
            nimi: nimi.to_string(),
            pisteet,
        };
        let lukkoon = [
            [2, 2, 16, 8],
            [16, 32, 64, 16],
            [4, 16, 8, 32],
            [16, 32, 64, 128],
        ];
        let mut k = Kaynnistin::new(1);
        k.vastaanota(Vastaus::Lista(Nettipeli::Tiilirikko, vec![tulos("T", 9)]));
        k.kaynnista(Laji::Luvut);
        assert_eq!(
            std::mem::take(&mut k.pyynnot),
            [Pyynto::Hae(Nettipeli::Luvut)]
        );
        assert_eq!(luvut(&mut k).netti_parhaat, None);
        k.vastaanota(Vastaus::Lista(Nettipeli::Luvut, Vec::new()));
        assert_eq!(luvut(&mut k).netti_parhaat, Some(Vec::new()));

        k.kasittele(Syote::Ammu);
        luvut(&mut k).lauta = Lukulauta::ruuduista(lukkoon);
        k.kasittele(Syote::Nuoli(0, -1));
        k.kasittele(Syote::Kirjain('A'));
        k.kasittele(Syote::Ammu);
        let odotettu = tulos("A", 4);
        assert_eq!(
            std::mem::take(&mut k.pyynnot),
            [Pyynto::Laheta(Nettipeli::Luvut, odotettu.clone())]
        );
        assert_eq!(k.tallennus.luvut_jono.len(), 1);
        assert!(k.tallennus.tiilirikon_jono.is_empty());

        // lähetys ei ehtinyt perille: seuraava käynnistys yrittää uudelleen
        let mut uusi = Kaynnistin::new(2);
        uusi.lataa(Tallennus::tekstista(&k.tallennus.tekstiksi()));
        assert_eq!(
            uusi.pyynnot,
            [Pyynto::Laheta(Nettipeli::Luvut, odotettu.clone())]
        );
        uusi.vastaanota(Vastaus::Lahetetty(Nettipeli::Luvut, odotettu));
        assert!(uusi.tallennus.luvut_jono.is_empty());
    }

    #[test]
    fn nuolet_ohjaavat_tiilirikkoa_pohjassa_ollessaan() {
        let mut k = Kaynnistin::new(1);
        k.kaynnista(Laji::Tiilirikko);
        k.aseta_ohjaus(true, false, false);
        let Some(Kaynnissa::Tiilirikko(peli)) = &k.peli else {
            panic!("tiilirikko ei ole käynnissä");
        };
        assert_eq!(peli.ohjaus, -1);
        k.aseta_ohjaus(true, true, false);
        let Some(Kaynnissa::Tiilirikko(peli)) = &k.peli else {
            panic!("tiilirikko ei ole käynnissä");
        };
        assert_eq!(peli.ohjaus, 0);
    }

    #[test]
    fn pasianssin_taso_sailyy_ja_hiiri_muuntuu_nakyman_mukaan() {
        let mut k = Kaynnistin::new(1);
        assert_eq!(k.nakyman_piste(285, 7), (285, 7));
        k.kaynnista(Laji::Pasianssi);
        assert_eq!(k.nakyman_piste(285, 0), (0, 0));
        k.kasittele(Syote::Numero(3));
        k.kasittele(Syote::Ammu);
        assert_eq!(k.nakyman_piste(285, 7), (285, 7));
        k.kasittele(Syote::Peru);

        k.kaynnista(Laji::Pasianssi);
        k.kasittele(Syote::Ammu);
        let Some(Kaynnissa::Pasianssi(peli)) = &k.peli else {
            panic!("pasianssi ei ole käynnissä");
        };
        assert_eq!(peli.lauta.vaikeus, pasianssi::Vaikeus::Vaikea);
    }

    #[test]
    fn asetukset_sailyvat_kun_peli_vaihtuu_ja_laivasto_ei_toistu() {
        let mut k = Kaynnistin::new(1);
        let (x, y) = piirto::aanikytkimen_keskipiste();
        k.kasittele(Syote::Klikkaus(x, y));
        assert!(!k.aanet_paalla);
        assert!(k.peli.is_none());

        k.kasittele(Syote::Ammu);
        let ensimmainen = laivaruudut(laivanupotus(&k));
        assert!(!laivanupotus(&k).aanet_paalla);
        k.kasittele(Syote::Numero(3));
        k.kasittele(Syote::VaihdaAanet);
        k.kasittele(Syote::Peru);
        assert!(k.aanet_paalla);

        k.kasittele(Syote::Ammu);
        let peli = laivanupotus(&k);
        assert_eq!(peli.vaikeus, Vaikeus::Vaikea);
        assert!(peli.aanet_paalla);
        assert_ne!(laivaruudut(peli), ensimmainen);
    }

    #[test]
    fn paras_tulos_ja_aanivalinta_sailyvat_2048_pelin_yli() {
        let mut k = Kaynnistin::new(1);
        k.kaynnista(Laji::Luvut);
        k.kasittele(Syote::Ammu);
        luvut(&mut k).lauta = Lukulauta::ruuduista([[4, 4, 0, 0], [0; 4], [0; 4], [0; 4]]);
        k.kasittele(Syote::Nuoli(0, -1));
        k.kasittele(Syote::VaihdaAanet);
        assert!(!k.nuolet_toistuvat());
        k.kasittele(Syote::Peru);
        assert!(k.nuolet_toistuvat());

        k.kaynnista(Laji::Luvut);
        assert_eq!(luvut(&mut k).paras, 8);
        assert!(!luvut(&mut k).aanet_paalla);
    }

    #[test]
    fn keskenerainen_2048_palautuu_valikon_kautta_ja_tiedoston_tilasta() {
        let mut k = Kaynnistin::new(1);
        k.kaynnista(Laji::Luvut);
        k.kasittele(Syote::Ammu);
        luvut(&mut k).lauta = Lukulauta::ruuduista([[4, 4, 0, 0], [0; 4], [0; 4], [0; 4]]);
        k.kasittele(Syote::Nuoli(0, -1));
        let lauta = luvut(&mut k).lauta.clone();
        k.kasittele(Syote::Peru);
        assert!(k.tallenna);

        // sama kuin ohjelman uudelleenkäynnistys: tila kulkee tekstin kautta
        let mut uusi = Kaynnistin::new(2);
        uusi.tallennus = Tallennus::tekstista(&k.tallennus.tekstiksi());
        uusi.kaynnista(Laji::Luvut);
        assert!(luvut(&mut uusi).jatkuu);
        uusi.kasittele(Syote::Ammu);
        assert_eq!(luvut(&mut uusi).lauta, lauta);
        assert_eq!(luvut(&mut uusi).paras, 8);

        // ikkunan sulkeminen kesken pelin: main.rs päivittää ennen kirjoitusta
        uusi.kasittele(Syote::Nuoli(1, 0));
        uusi.paivita_tallennus();
        assert_eq!(uusi.tallennus.kesken.unwrap().siirrot, 2);
    }

    #[test]
    fn aanivalinta_tallentuu_heti_ja_palautuu_tiedoston_tilasta() {
        let mut k = Kaynnistin::new(1);
        k.kasittele(Syote::VaihdaAanet);
        assert!(k.tallenna);
        assert!(k.tallennus.hiljaa);

        // sama kuin ohjelman uudelleenkäynnistys: tila kulkee tekstin kautta
        let mut uusi = Kaynnistin::new(2);
        uusi.lataa(Tallennus::tekstista(&k.tallennus.tekstiksi()));
        assert!(!uusi.aanet_paalla);
        uusi.kaynnista(Laji::Laivanupotus);
        assert!(!laivanupotus(&uusi).aanet_paalla);

        // pelin sisällä vaihdettu valinta tallentuu sekin
        uusi.kasittele(Syote::VaihdaAanet);
        assert!(uusi.tallenna);
        assert!(!uusi.tallennus.hiljaa);
    }

    #[test]
    fn kieli_vaihtuu_valikossa_tallentuu_ja_palautuu() {
        let mut k = Kaynnistin::new(1);
        assert_eq!(k.otsikko(), "Pelikone");
        k.kasittele(Syote::Kirjain('L'));
        assert_eq!(k.otsikko(), "Game Machine");
        assert!(k.tallenna);
        assert_eq!(k.tallennus.kieli, kieli::Kieli::Englanti);

        // sama kuin ohjelman uudelleenkäynnistys: tila kulkee tekstin kautta
        kieli::aseta(kieli::Kieli::Suomi);
        let mut uusi = Kaynnistin::new(2);
        uusi.lataa(Tallennus::tekstista(&k.tallennus.tekstiksi()));
        assert_eq!(uusi.otsikko(), "Game Machine");

        let (x, y) = piirto::kielikytkimen_keskipiste();
        uusi.kasittele(Syote::Klikkaus(x, y));
        assert_eq!(uusi.otsikko(), "Pelikone");

        // pelin sisällä L ei vaihda kieltä
        uusi.kaynnista(Laji::Laivanupotus);
        uusi.kasittele(Syote::Kirjain('L'));
        assert_eq!(kieli::nykyinen(), kieli::Kieli::Suomi);
    }

    #[test]
    fn top3_tallentuu_heti_ja_palautuu_tiedoston_tilasta() {
        let lukkoon = [
            [2, 2, 16, 8],
            [16, 32, 64, 16],
            [4, 16, 8, 32],
            [16, 32, 64, 128],
        ];
        let mut k = Kaynnistin::new(1);
        k.kaynnista(Laji::Luvut);
        k.kasittele(Syote::Ammu);
        luvut(&mut k).lauta = Lukulauta::ruuduista(lukkoon);
        k.kasittele(Syote::Nuoli(0, -1));
        k.kasittele(Syote::Kirjain('Ö'));
        assert!(!k.tallenna);
        k.kasittele(Syote::Ammu);
        assert!(k.tallenna);

        // sama kuin ohjelman uudelleenkäynnistys: tila kulkee tekstin kautta
        let mut uusi = Kaynnistin::new(2);
        uusi.lataa(Tallennus::tekstista(&k.tallennus.tekstiksi()));
        uusi.kaynnista(Laji::Luvut);
        assert_eq!(luvut(&mut uusi).parhaat[0].nimi, "Ö");
        assert_eq!(luvut(&mut uusi).parhaat[0].pisteet, 4);

        // ikkunan sulkeminen nimeä kirjoittaessa ei hävitä tulosta
        uusi.kasittele(Syote::Ammu);
        luvut(&mut uusi).lauta = Lukulauta::ruuduista(lukkoon);
        uusi.kasittele(Syote::Nuoli(0, -1));
        uusi.viimeistele();
        assert_eq!(uusi.tallennus.parhaat.len(), 2);
    }

    #[test]
    fn aloittamaton_uusi_peli_ei_kumoa_parasta() {
        let mut k = Kaynnistin::new(1);
        k.tallennus.paras = 900;
        k.kaynnista(Laji::Luvut);
        k.kasittele(Syote::Peru);
        assert_eq!(k.tallennus.paras, 900);
        assert_eq!(k.tallennus.kesken, None);
    }

    #[test]
    fn pelin_lopun_e_palaa_valikkoon() {
        let mut k = Kaynnistin::new(1);
        k.kaynnista(Laji::Laivanupotus);
        if let Some(Kaynnissa::Laivanupotus(peli)) = &mut k.peli {
            peli.vaihe = Vaihe::Voitto;
        }
        k.kasittele(Syote::Kirjain('E'));
        assert!(k.peli.is_none());
        assert!(!k.lopeta);
    }

    #[test]
    fn nakymat_piirtyvat_kaatumatta() {
        let mut kehys = Kehys::default();
        let mut k = Kaynnistin::new(1);
        piirto::piirra(&k, &mut kehys);
        for laji in [
            Laji::Laivanupotus,
            Laji::Luvut,
            Laji::Pasianssi,
            Laji::Tiilirikko,
            Laji::Asteroidit,
            Laji::Silmukka,
            Laji::Palikka,
            Laji::Miinaharava,
            Laji::Blackjack,
            Laji::Holdem,
        ] {
            k.kaynnista(laji);
            k.edista(100);
            piirto::piirra(&k, &mut kehys);
            k.kasittele(Syote::Ammu);
            piirto::piirra(&k, &mut kehys);
        }
    }
}
