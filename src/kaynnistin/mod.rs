//! Pelin valinta ja käynnissä olevan pelin hallinta. Uusi peli lisätään tänne korttina
//! (`PELIT`) ja `Kaynnissa`-muunnelmana; muun käynnistin hoitaa `Pelinakyma`-traitin kautta.

pub mod netti;
pub mod piirto;
pub mod tallennus;

use self::netti::{Pyynto, Vastaus};
use self::tallennus::Tallennus;
use crate::pelit::asteroidit::peli::Asteroidit;
use crate::pelit::asteroidit::piirto::kivimalli;
use crate::pelit::blackjack::peli::Blackjack;
use crate::pelit::blackjack::piirto::pelimerkkipinot;
use crate::pelit::holdem::peli::Holdem;
use crate::pelit::holdem::piirto::poytamalli;
use crate::pelit::laivanupotus::peli::{Peli, Tilasto};
use crate::pelit::laivanupotus::piirto::alkumalli as laivamalli;
use crate::pelit::luvut::peli::Luvut;
use crate::pelit::luvut::piirto::lukupino;
use crate::pelit::miinaharava::peli::Miinaharava;
use crate::pelit::miinaharava::piirto::miinamalli;
use crate::pelit::palikka::peli::Palikkapeli;
use crate::pelit::palikka::piirto::jalokivi;
use crate::pelit::pasianssi::peli::{self as pasianssi_peli, Pasianssi};
use crate::pelit::pasianssi::piirto::{korttipinot, lajin_nimi};
use crate::pelit::silmukka::peli::Silmukka;
use crate::pelit::silmukka::piirto::ouroboros;
use crate::pelit::tiilirikko::peli::Tiilirikko;
use crate::pelit::tiilirikko::piirto::tiilimuuri;
use crate::yhteinen::aanet::Aani;
use crate::yhteinen::kieli::{self, Kieli, kaanna};
use crate::yhteinen::pelinakyma::Pelinakyma;
use crate::yhteinen::piirto::ruudulta_kuvaan;
use crate::yhteinen::syote::Syote;
use crate::yhteinen::tulokset::{NETTIPELEJA, Nettipeli, Tulos, lisaa_tulos, paivita_nimen_tulos};
use crate::yhteinen::vektori::Tahko;
use klassikot8000::{laivanupotus, miinaharava};

/// Valikon mallia ja vilkkua piirretään harvemmin kuin silmukka kiertää: joka toinen
/// kierros riittää hitaaseen pyörintään ja puolittaa ruudun lähettämisen.
const VALIKON_ASKEL_MS: u32 = 66;

/// Kaikki, mistä paikallaan pysyvän näkymän kuva riippuu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KuvanAvain {
    Valikko {
        valittu: usize,
        alku: usize,
        aanet: bool,
        kieli: Kieli,
        uusi_versio: bool,
        ilmoitus_alla: bool,
        listoja: u32,
        askel: u32,
    },
    /// Peli, jonka kuva muuttuu syötteestä, nettilistasta tai pelin omassa tahdissa.
    Peli {
        syotteita: u32,
        listoja: u32,
        avain: (u32, u32),
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

/// Ylin näkyvä kortti: lista vierii vain sen verran, että valinta pysyy näkyvissä.
fn nakyva_alku(alku: usize, valittu: usize, maara: usize, nakyvia: usize) -> usize {
    let alku = if valittu < alku {
        valittu
    } else if valittu >= alku + nakyvia {
        valittu + 1 - nakyvia
    } else {
        alku
    };
    alku.min(maara.saturating_sub(nakyvia))
}

fn kortti(laji: Laji) -> &'static Kortti {
    PELIT
        .iter()
        .find(|kortti| kortti.laji == laji)
        .unwrap_or(&PELIT[0])
}

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

impl Kaynnissa {
    pub fn nakyma(&self) -> &dyn Pelinakyma {
        match self {
            Kaynnissa::Laivanupotus(peli) => &**peli,
            Kaynnissa::Luvut(peli) => &**peli,
            Kaynnissa::Pasianssi(peli) => &**peli,
            Kaynnissa::Tiilirikko(peli) => &**peli,
            Kaynnissa::Asteroidit(peli) => &**peli,
            Kaynnissa::Silmukka(peli) => &**peli,
            Kaynnissa::Palikka(peli) => &**peli,
            Kaynnissa::Miinaharava(peli) => &**peli,
            Kaynnissa::Blackjack(peli) => &**peli,
            Kaynnissa::Holdem(peli) => &**peli,
        }
    }

    pub fn nakyma_mut(&mut self) -> &mut dyn Pelinakyma {
        match self {
            Kaynnissa::Laivanupotus(peli) => &mut **peli,
            Kaynnissa::Luvut(peli) => &mut **peli,
            Kaynnissa::Pasianssi(peli) => &mut **peli,
            Kaynnissa::Tiilirikko(peli) => &mut **peli,
            Kaynnissa::Asteroidit(peli) => &mut **peli,
            Kaynnissa::Silmukka(peli) => &mut **peli,
            Kaynnissa::Palikka(peli) => &mut **peli,
            Kaynnissa::Miinaharava(peli) => &mut **peli,
            Kaynnissa::Blackjack(peli) => &mut **peli,
            Kaynnissa::Holdem(peli) => &mut **peli,
        }
    }

    pub fn laji(&self) -> Laji {
        match self {
            Kaynnissa::Laivanupotus(_) => Laji::Laivanupotus,
            Kaynnissa::Luvut(_) => Laji::Luvut,
            Kaynnissa::Pasianssi(_) => Laji::Pasianssi,
            Kaynnissa::Tiilirikko(_) => Laji::Tiilirikko,
            Kaynnissa::Asteroidit(_) => Laji::Asteroidit,
            Kaynnissa::Silmukka(_) => Laji::Silmukka,
            Kaynnissa::Palikka(_) => Laji::Palikka,
            Kaynnissa::Miinaharava(_) => Laji::Miinaharava,
            Kaynnissa::Blackjack(_) => Laji::Blackjack,
            Kaynnissa::Holdem(_) => Laji::Holdem,
        }
    }
}

/// Pelin valinta ja käynnissä oleva peli. Asetukset säilyvät, kun peli vaihtuu.
pub struct Kaynnistin {
    pub valittu: usize,
    /// Ylin näkyvä kortti, kun pelejä on enemmän kuin valikkoon mahtuu.
    pub alku: usize,
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
    /// Asetukset, ennätykset, listat ja jonot. Nykytila siirtyy tänne `paivita_tallennus`-kutsulla.
    pub tallennus: Tallennus,
    /// Tosi, kun `main.rs`:n kuuluu kirjoittaa tallennus tiedostoon.
    pub tallenna: bool,
    /// Pyynnöt, jotka `main.rs` välittää verkkosäikeelle.
    pub pyynnot: Vec<Pyynto>,
    /// Käsiteltyjen syötteiden määrä: kuva voi muuttua minkä tahansa syötteen jälkeen.
    syotteita: u32,
    /// Nettilistat, kun palvelin on vastannut. Järjestys kuten `Nettipeli::KAIKKI`.
    netti: [Option<Vec<Tulos>>; NETTIPELEJA],
    /// Listat, jotka on jo pyydetty tämän käynnistyksen aikana.
    haettu: [bool; NETTIPELEJA],
    /// Saapuneiden nettilistojen määrä: uusi lista voi muuttaa kuvaa.
    listoja: u32,
    /// Uusin julkaistu versio, jos se on uudempi kuin tämä ohjelma.
    uusi_versio: Option<String>,
    /// Hiiri on uuden version ilmoituksen päällä, joten ilmoitus näyttää napilta.
    ilmoitus_alla: bool,
    /// Tosi, kun `main.rs`:n kuuluu avata julkaisusivu selaimeen.
    pub avaa_julkaisut: bool,
    siemen: u64,
}

impl Kaynnistin {
    pub fn new(siemen: u64) -> Self {
        Self {
            valittu: 0,
            alku: 0,
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
            netti: Default::default(),
            haettu: [false; NETTIPELEJA],
            listoja: 0,
            uusi_versio: None,
            ilmoitus_alla: false,
            avaa_julkaisut: false,
            siemen,
        }
    }

    pub fn valitse(&mut self, i: usize) {
        if i != self.valittu && i < PELIT.len() {
            self.valittu = i;
            self.valittu_ms = self.kello_ms;
            self.malli = (PELIT[i].malli)();
            self.alku = nakyva_alku(self.alku, i, PELIT.len(), piirto::NAKYVIA);
        }
    }

    /// Valikon näkyvän paikan kortti, jos paikalla on peli.
    fn paikan_kortti(&self, paikka: usize) -> Option<usize> {
        let i = self.alku + paikka;
        (paikka < piirto::NAKYVIA && i < PELIT.len()).then_some(i)
    }

    /// Lista, jonka valikon paneeli näyttää valitulle pelille.
    pub fn valikon_lista(&self) -> Option<Nettipeli> {
        Some(match PELIT[self.valittu].laji {
            Laji::Laivanupotus => Nettipeli::Laivanupotus,
            Laji::Luvut => Nettipeli::Luvut,
            Laji::Tiilirikko => Nettipeli::Tiilirikko,
            Laji::Asteroidit => Nettipeli::Asteroidit,
            Laji::Silmukka => Nettipeli::Silmukka,
            Laji::Palikka => Nettipeli::Palikka,
            // Viimeksi valitun lajin lista: sen pelaaja haluaa todennäköisimmin nähdä.
            Laji::Pasianssi => pasianssi_peli::nettipeli(self.tallennus.asetukset.variantti),
            Laji::Miinaharava => Nettipeli::Miinaharava,
            Laji::Blackjack => Nettipeli::Blackjack,
            Laji::Holdem => Nettipeli::Holdem,
        })
    }

    /// Listan laji, kun saman pelin lajeilla on omat listansa.
    pub fn valikon_listan_laji(&self) -> Option<&'static str> {
        (PELIT[self.valittu].laji == Laji::Pasianssi)
            .then(|| lajin_nimi(self.tallennus.asetukset.variantti))
    }

    /// Valitun pelin oma paras tulos, myös listalle yltämätön. `None`, jos tulosta ei ole.
    pub fn valitun_oma_paras(&self) -> Option<u32> {
        let lista = self.valikon_lista()?;
        let tallennettu = self.tallennus.lista(lista);
        let tilastoista = match lista {
            Nettipeli::Laivanupotus => self
                .tallennus
                .laivatilasto
                .iter()
                .zip(laivanupotus::Vaikeus::KAIKKI)
                .map(|(tilasto, taso)| tilasto.parhaat_pisteet(taso))
                .max(),
            Nettipeli::Miinaharava => self
                .tallennus
                .miinaajat
                .iter()
                .zip(miinaharava::Vaikeus::KAIKKI)
                .filter(|&(&aika, _)| aika > 0)
                .map(|(&aika, taso)| miinaharava::aikapisteet(taso, aika))
                .max(),
            _ => None,
        };
        let listalta = tallennettu.parhaat.iter().map(|tulos| tulos.pisteet).max();
        [Some(tallennettu.paras), tilastoista, listalta]
            .into_iter()
            .flatten()
            .max()
            .filter(|&paras| paras > 0)
    }

    /// Laivanupotuksen voitot, tappiot ja ennätykset tasoittain, kun se on valittuna.
    pub fn valitun_tilastot(&self) -> Option<&[Tilasto; 3]> {
        (PELIT[self.valittu].laji == Laji::Laivanupotus).then_some(&self.tallennus.laivatilasto)
    }

    pub fn uusi_versio(&self) -> Option<&str> {
        self.uusi_versio.as_deref()
    }

    pub fn ilmoitus_alla(&self) -> bool {
        self.ilmoitus_alla
    }

    /// Valitun pelin top 3: nettilista, kun se on saapunut, muuten omat tulokset. Tosi
    /// kertoo nettilistasta.
    pub fn valitun_parhaat(&self) -> Option<(bool, &[Tulos])> {
        let lista = self.valikon_lista()?;
        Some(match &self.netti[lista as usize] {
            Some(netti) => (true, netti),
            None => (false, &self.tallennus.lista(lista).parhaat),
        })
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
                Some(KuvanAvain::Valikko {
                    valittu: self.valittu,
                    alku: self.alku,
                    aanet: self.aanet_paalla,
                    kieli: kieli::nykyinen(),
                    uusi_versio: self.uusi_versio.is_some(),
                    ilmoitus_alla: self.ilmoitus_alla,
                    listoja: self.listoja,
                    askel,
                })
            }
            Some(peli) => peli.nakyma().kuvan_avain().map(|avain| KuvanAvain::Peli {
                syotteita: self.syotteita,
                listoja: self.listoja,
                avain,
            }),
        }
    }

    pub fn otsikko(&self) -> &'static str {
        kaanna(
            self.peli
                .as_ref()
                .map_or("peli.pelikone", |peli| kortti(peli.laji()).nimi),
        )
    }

    /// Reaaliaikainen peli tarvitsee tiheämmän päivityksen kuin vuoropelit, mutta sen
    /// alkuruudulle riittää sama tahti kuin muille alkuruuduille.
    pub fn toimintapeli(&self) -> bool {
        self.peli
            .as_ref()
            .is_some_and(|peli| peli.nakyma().toimintapeli())
    }

    /// Pohjassa olevat suunnat. Näppäintoisto olisi mailan ohjaukseen liian nykivä.
    pub fn aseta_ohjaus(&mut self, vasen: bool, oikea: bool, ylos: bool) {
        if let Some(peli) = &mut self.peli {
            peli.nakyma_mut()
                .aseta_ohjaus(oikea as i8 - vasen as i8, ylos);
        }
    }

    /// Ikkuna ei ole aktiivinen, joten reaaliaikainen peli jää odottamaan.
    pub fn keskeyta(&mut self) {
        if let Some(peli) = &mut self.peli {
            peli.nakyma_mut().keskeyta();
        }
    }

    pub fn nuolet_toistuvat(&self) -> bool {
        self.peli
            .as_ref()
            .is_none_or(|peli| peli.nakyma().nuolet_toistuvat())
    }

    /// Muuttaa ruudun pikselin sen näkymän koordinaateiksi, joka on nyt esillä.
    pub fn nakyman_piste(&self, x: i32, y: i32) -> (i32, i32) {
        match &self.peli {
            Some(peli) if !peli.nakyma().suoraan_ruudulle() => ruudulta_kuvaan(x, y),
            _ => (x, y),
        }
    }

    pub fn kaynnista(&mut self, laji: Laji) {
        let siemen = self.siemen;
        // Sama siemen toistaisi saman alkutilanteen, kun peli avataan uudelleen.
        self.siemen = self.siemen.wrapping_add(1);
        let asetukset = self.tallennus.asetukset;
        let mut peli = match laji {
            Laji::Laivanupotus => {
                let mut peli = Peli::new(siemen);
                peli.vaikeus = asetukset.laivanupotus;
                peli.tilastot = self.tallennus.laivatilasto;
                Kaynnissa::Laivanupotus(Box::new(peli))
            }
            Laji::Luvut => {
                let mut peli = Luvut::new(siemen);
                if let Some(kesken) = self.tallennus.kesken {
                    peli.palauta(kesken);
                }
                Kaynnissa::Luvut(Box::new(peli))
            }
            Laji::Pasianssi => {
                let mut peli = Pasianssi::new(siemen);
                peli.vaikeus = asetukset.pasianssi;
                peli.variantti = asetukset.variantti;
                peli.haastepisteet = self.tallennus.haastepisteet;
                peli.haastenimi = self.tallennus.haastenimi.clone();
                Kaynnissa::Pasianssi(Box::new(peli))
            }
            Laji::Tiilirikko => Kaynnissa::Tiilirikko(Box::new(Tiilirikko::new(siemen))),
            Laji::Asteroidit => Kaynnissa::Asteroidit(Box::new(Asteroidit::new(siemen))),
            Laji::Silmukka => Kaynnissa::Silmukka(Box::new(Silmukka::new(siemen))),
            Laji::Palikka => Kaynnissa::Palikka(Box::new(Palikkapeli::new(siemen))),
            Laji::Miinaharava => {
                let mut peli = Miinaharava::new(siemen);
                peli.vaikeus = asetukset.miinaharava;
                peli.ajat = self.tallennus.miinaajat;
                Kaynnissa::Miinaharava(Box::new(peli))
            }
            Laji::Blackjack => Kaynnissa::Blackjack(Box::new(Blackjack::new(siemen))),
            Laji::Holdem => Kaynnissa::Holdem(Box::new(Holdem::new(siemen))),
        };
        let nakyma = peli.nakyma_mut();
        nakyma.aseta_aanet(self.aanet_paalla);
        for lista in nakyma.listat() {
            let tallennettu = self.tallennus.lista(lista.peli);
            if let Some(paras) = lista.paras {
                *paras = tallennettu.paras;
            }
            *lista.parhaat = tallennettu.parhaat.clone();
            *lista.netti = self.netti[lista.peli as usize].clone();
            // Lista voi olla muuttunut valikossa olon aikana, joten se haetaan uudelleen.
            let pyynto = Pyynto::Hae(lista.peli);
            if !self.pyynnot.contains(&pyynto) {
                self.pyynnot.push(pyynto);
            }
            self.haettu[lista.peli as usize] = true;
        }
        self.peli = Some(peli);
    }

    /// Ottaa käyttöön tiedostosta luetun tallennuksen.
    pub fn lataa(&mut self, tallennus: Tallennus) {
        self.aanet_paalla = !tallennus.hiljaa;
        kieli::aseta(tallennus.kieli);
        // Edellisellä kerralla lähettämättä jääneet tulokset yritetään uudelleen.
        for peli in Nettipeli::KAIKKI {
            let jono = tallennus.lista(peli).jono.iter().cloned();
            self.pyynnot
                .extend(jono.map(|tulos| Pyynto::Laheta(peli, tulos)));
        }
        let valittu = tallennus.asetukset.valittu;
        self.tallennus = tallennus;
        self.valitse(valittu);
    }

    /// Ottaa verkkosäikeen vastauksen. Lähetetty tulos poistuu jonosta.
    pub fn vastaanota(&mut self, vastaus: Vastaus) {
        match vastaus {
            Vastaus::Lista(peli, lista) => {
                if let Some(kaynnissa) = &mut self.peli {
                    for oma in kaynnissa.nakyma_mut().listat() {
                        if oma.peli == peli {
                            *oma.netti = Some(lista.clone());
                        }
                    }
                }
                self.netti[peli as usize] = Some(lista);
                self.listoja = self.listoja.wrapping_add(1);
            }
            Vastaus::Lahetetty(peli, tulos) => {
                self.tallennus.lista_mut(peli).jono.retain(|t| *t != tulos);
                self.tallenna = true;
            }
            Vastaus::UusiVersio(versio) => self.uusi_versio = Some(versio),
        }
    }

    pub fn paivita_tallennus(&mut self) {
        self.tallennus.hiljaa = !self.aanet_paalla;
        self.tallennus.kieli = kieli::nykyinen();
        self.tallennus.asetukset.valittu = self.valittu;
        let Some(peli) = &mut self.peli else {
            return;
        };
        match peli {
            Kaynnissa::Laivanupotus(peli) => self.tallennus.laivatilasto = peli.tilastot,
            Kaynnissa::Luvut(peli) => self.tallennus.kesken = peli.kesken(),
            Kaynnissa::Pasianssi(peli) => {
                self.tallennus.haastepisteet = peli.haastepisteet;
                self.tallennus.haastenimi = peli.haastenimi.clone();
            }
            Kaynnissa::Miinaharava(peli) => self.tallennus.miinaajat = peli.ajat,
            _ => {}
        }
        for lista in peli.nakyma_mut().listat() {
            let tallennettu = self.tallennus.lista_mut(lista.peli);
            if let Some(paras) = lista.paras {
                tallennettu.paras = *paras;
            }
            tallennettu.parhaat = lista.parhaat.clone();
            self.netti[lista.peli as usize] = lista.netti.clone();
            // Jono kirjataan tiedostoon, jottei tulos katoa, vaikka lähetys epäonnistuisi.
            if let Some(tulos) = lista.lahetys.and_then(Option::take) {
                if lista.peli.yksi_per_nimi() {
                    // Uusi tulos korvaa saman nimen vanhan, joten jonoon jää vain uusin.
                    paivita_nimen_tulos(&mut tallennettu.jono, tulos.clone());
                } else {
                    lisaa_tulos(&mut tallennettu.jono, tulos.clone());
                }
                self.pyynnot.push(Pyynto::Laheta(lista.peli, tulos));
            }
        }
    }

    /// Ikkuna suljetaan: nimeä odottava tulos kirjataan, ettei se katoa.
    pub fn viimeistele(&mut self) {
        if let Some(peli) = &mut self.peli {
            peli.nakyma_mut().viimeistele();
        }
        self.paivita_tallennus();
    }

    pub fn edista(&mut self, ms: u32) {
        self.kello_ms = self.kello_ms.saturating_add(ms);
        let Some(peli) = &mut self.peli else {
            self.hae_valikon_lista();
            return;
        };
        let nakyma = peli.nakyma_mut();
        nakyma.edista(ms);
        // Toimintapeli päättyy ajan kuluessa eikä syötteeseen, joten tulos kirjataan täällä.
        if nakyma.ota_uusi_tulos() {
            self.paivita_tallennus();
            self.tallenna = true;
        }
    }

    /// Valikon paneeli näyttää valitun pelin nettilistan, joten se haetaan kerran.
    fn hae_valikon_lista(&mut self) {
        if let Some(lista) = self.valikon_lista()
            && !self.haettu[lista as usize]
        {
            self.haettu[lista as usize] = true;
            self.pyynnot.push(Pyynto::Hae(lista));
        }
    }

    pub fn ota_aanet(&mut self) -> Vec<(Aani, u32)> {
        self.peli.as_mut().map_or_else(Vec::new, |peli| {
            std::mem::take(peli.nakyma_mut().aanijono())
        })
    }

    pub fn kasittele(&mut self, syote: Syote) {
        self.syotteita = self.syotteita.wrapping_add(1);
        let aanet_ennen = self.aanet_paalla;
        let kieli_ennen = kieli::nykyinen();
        self.kasittele_nakyma(syote);
        let uusi_tulos = self
            .peli
            .as_mut()
            .is_some_and(|peli| peli.nakyma_mut().ota_uusi_tulos());
        if uusi_tulos || self.aanet_paalla != aanet_ennen || kieli::nykyinen() != kieli_ennen {
            self.paivita_tallennus();
            self.tallenna = true;
        }
    }

    fn kasittele_nakyma(&mut self, syote: Syote) {
        let Some(peli) = &mut self.peli else {
            self.kasittele_valikko(syote);
            return;
        };
        let nakyma = peli.nakyma_mut();
        nakyma.kasittele(syote);
        self.aanet_paalla = nakyma.aanet_paalla();
        let poistuu = nakyma.lopeta();
        // Pelin sisällä vaihdettu taso tai laji on seuraavallakin kerralla valmiina.
        let asetukset = &mut self.tallennus.asetukset;
        match peli {
            Kaynnissa::Laivanupotus(peli) => asetukset.laivanupotus = peli.vaikeus,
            Kaynnissa::Pasianssi(peli) => {
                asetukset.pasianssi = peli.vaikeus;
                asetukset.variantti = peli.variantti;
            }
            Kaynnissa::Miinaharava(peli) => asetukset.miinaharava = peli.vaikeus,
            _ => {}
        }
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
        let ilmoituksella = |x, y| {
            self.uusi_versio
                .as_deref()
                .is_some_and(|versio| piirto::ilmoituksella(x, y, versio))
        };
        match syote {
            Syote::Hiiri(x, y) => self.ilmoitus_alla = ilmoituksella(x, y),
            Syote::Klikkaus(x, y) if ilmoituksella(x, y) => {
                self.avaa_julkaisut = true;
                return;
            }
            _ => {}
        }
        match syote {
            Syote::Peru => self.lopeta = true,
            Syote::Ammu => self.kaynnista(PELIT[self.valittu].laji),
            Syote::Klikkaus(x, y) if piirto::pelaa_napilla(x, y, self.valittu) => {
                self.kaynnista(PELIT[self.valittu].laji)
            }
            Syote::Hiiri(x, y) => {
                if let Some(i) = piirto::kortti_pisteesta(x, y).and_then(|p| self.paikan_kortti(p))
                {
                    self.valitse(i);
                }
            }
            Syote::Klikkaus(x, y) => {
                if let Some(i) = piirto::kortti_pisteesta(x, y).and_then(|p| self.paikan_kortti(p))
                {
                    self.valitse(i);
                    self.kaynnista(PELIT[i].laji);
                }
            }
            Syote::Numero(numero) => {
                // Numerot ovat näkyvien korttien omat, ja kymmenes on nolla kuten numerorivillä.
                if let Some(i) = self.paikan_kortti((numero as usize + 9) % 10) {
                    self.valitse(i);
                }
            }
            Syote::Nuoli(drivi, _) => {
                // Lista kiertää: ylimmästä ylös pääsee alimpaan.
                let maara = PELIT.len() as isize;
                let i = (self.valittu as isize + drivi as isize).rem_euclid(maara);
                self.valitse(i as usize);
            }
            // Rulla ei kierrä, jottei pitkä pyöritys hyppää listan toiseen päähän.
            Syote::Rulla(suunta) => {
                let i = self.valittu.saturating_add_signed(suunta as isize);
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
    use klassikot8000::laivanupotus::Vaikeus;
    use klassikot8000::laivanupotus::{KOKO, Ruutu};
    use klassikot8000::luvut::Lukulauta;
    use klassikot8000::miinaharava;
    use klassikot8000::pasianssi::{self, Variantti};

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
    fn pelaa_nappi_kaynnistaa_valitun_pelin() {
        let mut k = Kaynnistin::new(1);
        k.kasittele(Syote::Numero(2));
        let (x, y) = piirto::pelaa_napin_keskipiste(1);
        k.kasittele(Syote::Hiiri(x, y));
        assert!(k.peli.is_none());
        k.kasittele(Syote::Klikkaus(x, y));
        assert!(matches!(k.peli, Some(Kaynnissa::Luvut(_))));
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
        assert_eq!(k.valittu, 0);
        // lista kiertää ympäri kumpaankin suuntaan
        k.kasittele(Syote::Nuoli(-1, 0));
        assert_eq!(k.valittu, PELIT.len() - 1);
        k.kasittele(Syote::Nuoli(1, 0));
        assert_eq!(k.valittu, 0);
        for _ in 0..5 {
            k.kasittele(Syote::Nuoli(1, 0));
        }
        assert_eq!(k.valittu, 5);
        for _ in 0..4 {
            k.kasittele(Syote::Nuoli(1, 0));
        }
        assert_eq!(k.valittu, 9);
        k.kasittele(Syote::Nuoli(1, 0));
        assert_eq!(k.valittu, 0);
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
    fn pitka_lista_vierii_niin_etta_valinta_pysyy_nakyvissa() {
        assert_eq!(nakyva_alku(0, 9, 10, 10), 0);
        // 14 peliä, 10 näkyvissä
        assert_eq!(nakyva_alku(0, 9, 14, 10), 0);
        assert_eq!(nakyva_alku(0, 10, 14, 10), 1);
        assert_eq!(nakyva_alku(0, 13, 14, 10), 4);
        assert_eq!(nakyva_alku(4, 12, 14, 10), 4);
        assert_eq!(nakyva_alku(4, 3, 14, 10), 3);
        assert_eq!(nakyva_alku(4, 0, 14, 10), 0);
        // lista ei vieri tyhjän yli, vaikka alku olisi jäänyt liian alas
        assert_eq!(nakyva_alku(9, 9, 14, 10), 4);

        let mut k = Kaynnistin::new(1);
        k.kasittele(Syote::Rulla(-1));
        assert_eq!(k.valittu, 0);
        k.kasittele(Syote::Rulla(1));
        k.kasittele(Syote::Rulla(1));
        assert_eq!(k.valittu, 2);
        for _ in 0..20 {
            k.kasittele(Syote::Rulla(1));
        }
        assert_eq!(k.valittu, PELIT.len() - 1);
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
        assert_eq!(k.valitun_parhaat(), Some((false, &[][..])));
        k.edista(1000);
        k.kasittele(Syote::Numero(5));
        assert_ne!(k.malli.len(), laiva);
        assert!(!k.malli.is_empty());
        assert_eq!(k.valittu_ms, 1000);

        k.tallennus.lista_mut(Nettipeli::Asteroidit).parhaat = vec![Tulos {
            nimi: "A".to_string(),
            pisteet: 100,
        }];
        let (netista, parhaat) = k.valitun_parhaat().unwrap();
        assert!(!netista);
        assert_eq!(parhaat.len(), 1);
        k.kasittele(Syote::Numero(1));
        assert_eq!(k.valitun_parhaat(), Some((false, &[][..])));
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
        let lista = uusi.tallennus.lista(peli);
        assert_eq!(lista.parhaat[..], [tulos]);
        assert!(lista.jono.is_empty());
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
    fn haasteen_tulos_tallentuu_ja_lahtee_omalle_listalleen() {
        let mut k = Kaynnistin::new(1);
        k.kaynnista(Laji::Pasianssi);
        assert!(k.pyynnot.contains(&Pyynto::Hae(Nettipeli::Haasteet)));
        let Some(Kaynnissa::Pasianssi(peli)) = &mut k.peli else {
            panic!("pasianssi ei ole käynnissä");
        };
        peli.haastepisteet[1][4] = 3000;
        peli.haastenimi = "V".to_string();
        let tulos = Tulos {
            nimi: "V".to_string(),
            pisteet: 3000,
        };
        peli.haasteiden_parhaat = vec![tulos.clone()];
        peli.haaste_lahetys = Some(tulos.clone());
        k.paivita_tallennus();
        assert_eq!(k.tallennus.haastepisteet[1][4], 3000);
        assert_eq!(k.tallennus.haastenimi, "V");
        let haasteet = k.tallennus.lista(Nettipeli::Haasteet);
        assert_eq!(haasteet.parhaat, std::slice::from_ref(&tulos));
        assert_eq!(haasteet.jono, std::slice::from_ref(&tulos));
        assert!(
            k.pyynnot
                .contains(&Pyynto::Laheta(Nettipeli::Haasteet, tulos.clone()))
        );
        assert!(k.tallennus.lista(Nettipeli::Pasianssi).parhaat.is_empty());

        // Kuitattu lähetys poistuu jonosta, ja seuraava peli jatkaa samoista tiedoista.
        k.vastaanota(Vastaus::Lahetetty(Nettipeli::Haasteet, tulos.clone()));
        assert!(k.tallennus.lista(Nettipeli::Haasteet).jono.is_empty());
        k.vastaanota(Vastaus::Lista(Nettipeli::Haasteet, vec![tulos.clone()]));
        k.peli = None;
        k.kaynnista(Laji::Pasianssi);
        let Some(Kaynnissa::Pasianssi(peli)) = &k.peli else {
            panic!("pasianssi ei ole käynnissä");
        };
        assert_eq!(peli.haastepisteet[1][4], 3000);
        assert_eq!(peli.haastenimi, "V");
        assert_eq!(peli.haastelista(), [tulos]);
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
            assert!(k.tallennus.lista(Nettipeli::Pasianssi).parhaat.is_empty());
        }
    }

    #[test]
    fn valikko_nayttaa_viimeksi_valitun_lajin_listan_ja_nettilista_menee_lajille() {
        let mut k = Kaynnistin::new(1);
        k.tallennus.lista_mut(Nettipeli::Spider).parhaat = vec![Tulos {
            nimi: "S".to_string(),
            pisteet: 9,
        }];
        k.valitse(
            PELIT
                .iter()
                .position(|kortti| kortti.laji == Laji::Pasianssi)
                .unwrap(),
        );
        assert!(k.valitun_parhaat().unwrap().1.is_empty());
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
        // valikko näyttää saapuneen nettilistan, omat tulokset säilyvät tallennuksessa
        assert_eq!(k.valitun_parhaat(), Some((true, &lista[..])));
        assert_eq!(k.tallennus.lista(Nettipeli::Spider).parhaat[0].nimi, "S");
    }

    #[test]
    fn blackjackin_voitto_kirjautuu_poistuttaessa_ja_tappio_ei() {
        let mut k = Kaynnistin::new(1);
        k.kaynnista(Laji::Blackjack);
        k.kasittele(Syote::Ammu);
        k.kasittele(Syote::Peru);
        assert!(k.peli.is_none());
        assert!(k.tallennus.lista(Nettipeli::Blackjack).parhaat.is_empty());

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
        assert_eq!(k.tallennus.lista(Nettipeli::Holdem).paras, 250);
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
        assert_eq!(
            k.tallennus.lista(Nettipeli::Blackjack).parhaat[0].pisteet,
            7
        );
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
        assert_eq!(k.tallennus.lista(Nettipeli::Palikka).paras, 350);
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
        assert!(uusi.tallennus.lista(Nettipeli::Palikka).jono.is_empty());
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
        assert_eq!(peli.vaihe, crate::pelit::miinaharava::peli::Vaihe::Nimi);
        k.kasittele(Syote::Hiiri(0, 0));
        assert!(k.tallenna);
        assert_eq!(k.tallennus.miinaajat, [0, 12, 0]);
        // voitto pääsee omalle nettilistalleen pisteinä
        let pisteet = miinaharava::aikapisteet(miinaharava::Vaikeus::Normaali, 12);
        tarkista_korttipelin_tulos(&mut k, Nettipeli::Miinaharava, pisteet);

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
        assert_eq!(k.tallennus.lista(Nettipeli::Silmukka).paras, 225);
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
        assert!(uusi.tallennus.lista(Nettipeli::Silmukka).jono.is_empty());
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
        assert_eq!(k.tallennus.lista(Nettipeli::Asteroidit).paras, 100);
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
        assert!(uusi.tallennus.lista(Nettipeli::Asteroidit).jono.is_empty());
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
        assert_eq!(k.tallennus.lista(Nettipeli::Tiilirikko).paras, 70);

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
        assert_eq!(
            k.tallennus.lista(Nettipeli::Tiilirikko).parhaat[0].nimi,
            "A"
        );

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
        assert_eq!(
            k.tallennus.lista(Nettipeli::Tiilirikko).parhaat[0].nimi,
            "NIMETÖN"
        );
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
        assert_eq!(k.tallennus.lista(Nettipeli::Tiilirikko).jono.len(), 1);

        // lähetys ei ehtinyt perille: seuraava käynnistys yrittää uudelleen
        let mut uusi = Kaynnistin::new(2);
        uusi.lataa(Tallennus::tekstista(&k.tallennus.tekstiksi()));
        assert_eq!(
            uusi.pyynnot,
            [Pyynto::Laheta(Nettipeli::Tiilirikko, tulos.clone())]
        );

        uusi.vastaanota(Vastaus::Lahetetty(Nettipeli::Tiilirikko, tulos));
        assert!(uusi.tallenna);
        assert!(uusi.tallennus.lista(Nettipeli::Tiilirikko).jono.is_empty());
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
        assert_eq!(k.tallennus.lista(Nettipeli::Luvut).jono.len(), 1);
        assert!(k.tallennus.lista(Nettipeli::Tiilirikko).jono.is_empty());

        // lähetys ei ehtinyt perille: seuraava käynnistys yrittää uudelleen
        let mut uusi = Kaynnistin::new(2);
        uusi.lataa(Tallennus::tekstista(&k.tallennus.tekstiksi()));
        assert_eq!(
            uusi.pyynnot,
            [Pyynto::Laheta(Nettipeli::Luvut, odotettu.clone())]
        );
        uusi.vastaanota(Vastaus::Lahetetty(Nettipeli::Luvut, odotettu));
        assert!(uusi.tallennus.lista(Nettipeli::Luvut).jono.is_empty());
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
    fn tasot_laji_ja_valinta_sailyvat_uudelleenkaynnistyksen_yli() {
        let mut k = Kaynnistin::new(1);
        k.kasittele(Syote::Numero(3));
        k.kaynnista(Laji::Laivanupotus);
        k.kasittele(Syote::Numero(1));
        k.kasittele(Syote::Peru);
        k.kaynnista(Laji::Pasianssi);
        k.kasittele(Syote::Numero(3));
        k.kasittele(Syote::Nuoli(1, 0));
        k.kasittele(Syote::Peru);
        k.kaynnista(Laji::Miinaharava);
        k.kasittele(Syote::Numero(3));
        k.kasittele(Syote::Peru);
        assert!(k.tallenna);

        // sama kuin ohjelman uudelleenkäynnistys: tila kulkee tekstin kautta
        let mut uusi = Kaynnistin::new(2);
        uusi.lataa(Tallennus::tekstista(&k.tallennus.tekstiksi()));
        assert_eq!(uusi.valittu, 2);
        uusi.kaynnista(Laji::Laivanupotus);
        assert_eq!(laivanupotus(&uusi).vaikeus, Vaikeus::Helppo);
        uusi.kaynnista(Laji::Pasianssi);
        let Some(Kaynnissa::Pasianssi(peli)) = &uusi.peli else {
            panic!("pasianssi ei ole käynnissä");
        };
        assert_eq!(peli.vaikeus, pasianssi::Vaikeus::Vaikea);
        assert_eq!(peli.variantti, Variantti::Spider);
        uusi.kaynnista(Laji::Miinaharava);
        let Some(Kaynnissa::Miinaharava(peli)) = &uusi.peli else {
            panic!("miinaharava ei ole käynnissä");
        };
        assert_eq!(peli.vaikeus, miinaharava::Vaikeus::Vaikea);
    }

    #[test]
    fn laivanupotuksen_tilasto_tallentuu_ja_palautuu() {
        use crate::pelit::laivanupotus::peli::Tilasto;

        let mut k = Kaynnistin::new(1);
        k.kaynnista(Laji::Laivanupotus);
        k.kasittele(Syote::Ammu);
        k.kasittele(Syote::Ammu);
        let Some(Kaynnissa::Laivanupotus(peli)) = &mut k.peli else {
            panic!("laivanupotus ei ole käynnissä");
        };
        let laivat: Vec<Ruutu> = (0..KOKO * KOKO)
            .filter_map(|i| Ruutu::new(i / KOKO, i % KOKO))
            .filter(|&r| peli.vastustaja.on_laiva(r))
            .collect();
        for ruutu in laivat {
            peli.ammu(ruutu);
        }
        // Tyhjälle listalle voitto yltää aina, joten se odottaa nimeä.
        assert_eq!(peli.vaihe, Vaihe::Nimi);
        k.kasittele(Syote::Hiiri(0, 0));
        assert!(k.tallenna);
        let odotettu = Tilasto {
            voitot: 1,
            tappiot: 0,
            vahiten_laukauksia: 17,
        };
        assert_eq!(k.tallennus.laivatilasto[1], odotettu);
        let pisteet = klassikot8000::laivanupotus::pisteet(Vaikeus::Normaali, 17);
        tarkista_korttipelin_tulos(&mut k, Nettipeli::Laivanupotus, pisteet);

        let mut uusi = Kaynnistin::new(2);
        uusi.lataa(Tallennus::tekstista(&k.tallennus.tekstiksi()));
        assert_eq!(uusi.valitun_oma_paras(), Some(pisteet));
        uusi.kaynnista(Laji::Laivanupotus);
        assert_eq!(laivanupotus(&uusi).tilastot[1], odotettu);
        assert_eq!(laivanupotus(&uusi).parhaat[0].pisteet, pisteet);
    }

    #[test]
    fn valikko_nayttaa_oman_ennatyksen_myos_listalle_yltamattomana() {
        let mut k = Kaynnistin::new(1);
        assert_eq!(k.valitun_oma_paras(), None);
        assert!(k.valitun_tilastot().is_some());
        // Laivanupotuksen ennätys on parhaan tason voitto pisteinä.
        k.tallennus.laivatilasto[0].vahiten_laukauksia = 20;
        k.tallennus.laivatilasto[2].vahiten_laukauksia = 60;
        assert_eq!(k.valitun_oma_paras(), Some(123));

        let paikka = |laji| PELIT.iter().position(|kortti| kortti.laji == laji).unwrap();
        k.valitse(paikka(Laji::Miinaharava));
        assert!(k.valitun_tilastot().is_none());
        assert_eq!(k.valitun_oma_paras(), None);
        k.tallennus.miinaajat = [0, 30, 0];
        let ajasta = miinaharava::aikapisteet(miinaharava::Vaikeus::Normaali, 30);
        assert_eq!(k.valitun_oma_paras(), Some(ajasta));

        // Erillinen ennätys voittaa listan, ja listan paras käy, jos ennätystä ei ole.
        k.valitse(paikka(Laji::Blackjack));
        let lista = k.tallennus.lista_mut(Nettipeli::Blackjack);
        lista.parhaat = vec![Tulos {
            nimi: "A".to_string(),
            pisteet: 40,
        }];
        assert_eq!(k.valitun_oma_paras(), Some(40));
        k.tallennus.lista_mut(Nettipeli::Blackjack).paras = 90;
        assert_eq!(k.valitun_oma_paras(), Some(90));
    }

    #[test]
    fn listalle_yltamaton_poytavoitto_jaa_omaksi_ennatykseksi() {
        let mut k = Kaynnistin::new(1);
        k.kaynnista(Laji::Blackjack);
        k.kasittele(Syote::Ammu);
        let Some(Kaynnissa::Blackjack(peli)) = &mut k.peli else {
            panic!("blackjack ei ole käynnissä");
        };
        let taysi = vec![
            Tulos {
                nimi: "X".to_string(),
                pisteet: 1000,
            };
            3
        ];
        peli.netti_parhaat = Some(taysi);
        peli.poyta.rahat += 40 * klassikot8000::blackjack::YKSI;
        k.kasittele(Syote::Peru);
        assert!(k.peli.is_none());
        assert!(k.tallenna);
        let lista = k.tallennus.lista(Nettipeli::Blackjack);
        assert_eq!(lista.paras, 40);
        assert!(lista.parhaat.is_empty() && lista.jono.is_empty());
    }

    #[test]
    fn pasianssin_lajien_ennatykset_tallentuvat_ja_valikko_nimeaa_lajin() {
        let mut k = Kaynnistin::new(1);
        assert_eq!(k.valikon_listan_laji(), None);
        k.valitse(
            PELIT
                .iter()
                .position(|kortti| kortti.laji == Laji::Pasianssi)
                .unwrap(),
        );
        assert_eq!(k.valikon_listan_laji(), Some("Klondike"));
        k.kaynnista(Laji::Pasianssi);
        let Some(Kaynnissa::Pasianssi(peli)) = &mut k.peli else {
            panic!("pasianssi ei ole käynnissä");
        };
        peli.omat_parhaat[Variantti::Spider as usize] = 77;
        // alas vaihtaa alkuruudulla lajin Klondikesta Spideriin
        k.kasittele(Syote::Nuoli(1, 0));
        k.kasittele(Syote::Peru);
        assert_eq!(k.tallennus.lista(Nettipeli::Spider).paras, 77);
        assert_eq!(k.valikon_listan_laji(), Some("Spider"));
        assert_eq!(k.valitun_oma_paras(), Some(77));

        let mut uusi = Kaynnistin::new(2);
        uusi.lataa(Tallennus::tekstista(&k.tallennus.tekstiksi()));
        uusi.kaynnista(Laji::Pasianssi);
        let Some(Kaynnissa::Pasianssi(peli)) = &uusi.peli else {
            panic!("pasianssi ei ole käynnissä");
        };
        assert_eq!(peli.omat_parhaat, [0, 77, 0]);
    }

    #[test]
    fn uuden_version_ilmoitus_korostuu_hiiren_alla_ja_klikkaus_avaa_sivun() {
        let mut k = Kaynnistin::new(1);
        let (x, y) = piirto::ilmoituksen_keskipiste("9.9.9");
        // Ilman ilmoitusta kohta on tyhjää.
        k.kasittele(Syote::Hiiri(x, y));
        k.kasittele(Syote::Klikkaus(x, y));
        assert!(!k.ilmoitus_alla && !k.avaa_julkaisut);

        k.vastaanota(Vastaus::UusiVersio("9.9.9".to_string()));
        let ennen = k.kuvan_avain(false);
        k.kasittele(Syote::Hiiri(x, y));
        assert!(k.ilmoitus_alla);
        assert_ne!(k.kuvan_avain(false), ennen);
        k.kasittele(Syote::Klikkaus(x, y));
        assert!(k.avaa_julkaisut);
        assert!(k.peli.is_none());
        assert_eq!(k.valittu, 0);

        let (x, y) = piirto::kortin_keskipiste(0);
        k.kasittele(Syote::Hiiri(x, y));
        assert!(!k.ilmoitus_alla);
    }

    #[test]
    fn valikko_hakee_valitun_pelin_nettilistan_kerran_ja_nayttaa_sen() {
        let mut k = Kaynnistin::new(1);
        k.edista(1);
        assert_eq!(
            std::mem::take(&mut k.pyynnot),
            [Pyynto::Hae(Nettipeli::Laivanupotus)]
        );
        k.kasittele(Syote::Numero(4));
        k.edista(1);
        assert_eq!(
            std::mem::take(&mut k.pyynnot),
            [Pyynto::Hae(Nettipeli::Tiilirikko)]
        );
        k.edista(1);
        assert!(k.pyynnot.is_empty());

        let ennen = k.kuvan_avain(false);
        let lista = vec![Tulos {
            nimi: "N".to_string(),
            pisteet: 5,
        }];
        k.vastaanota(Vastaus::Lista(Nettipeli::Tiilirikko, lista.clone()));
        assert_ne!(k.kuvan_avain(false), ennen);
        assert_eq!(k.valitun_parhaat(), Some((true, &lista[..])));

        // klikkaus valitsee ja käynnistää samalla kierroksella, eikä lista lähde kahdesti
        k.kasittele(Syote::Numero(1));
        let (x, y) = piirto::kortin_keskipiste(4);
        k.kasittele(Syote::Klikkaus(x, y));
        assert_eq!(k.pyynnot, [Pyynto::Hae(Nettipeli::Asteroidit)]);
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
        assert_eq!(uusi.tallennus.lista(Nettipeli::Luvut).parhaat.len(), 2);
    }

    #[test]
    fn aloittamaton_uusi_peli_ei_kumoa_parasta() {
        let mut k = Kaynnistin::new(1);
        k.tallennus.lista_mut(Nettipeli::Luvut).paras = 900;
        k.kaynnista(Laji::Luvut);
        k.kasittele(Syote::Peru);
        assert_eq!(k.tallennus.lista(Nettipeli::Luvut).paras, 900);
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
