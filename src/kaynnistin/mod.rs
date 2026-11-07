//! Pelin valinta ja käynnissä olevan pelin hallinta. Uusi peli lisätään tänne rivinä
//! `pelit!`-luetteloon; muun käynnistin hoitaa `Pelinakyma`-traitin kautta.

mod allekirjoitus;
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
use crate::pelit::kaivo::peli::Kaivo;
use crate::pelit::kaivo::piirto::kaivosmalli;
use crate::pelit::laivanupotus::peli::{Peli, Tilasto};
use crate::pelit::laivanupotus::piirto::alkumalli as laivamalli;
use crate::pelit::luvut::peli::Luvut;
use crate::pelit::luvut::piirto::lukupino;
use crate::pelit::mahjong::peli::Mahjong;
use crate::pelit::mahjong::piirto::laattamalli;
use crate::pelit::majakka::peli::Majakka;
use crate::pelit::majakka::piirto::majakkamalli;
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

pub struct Kortti {
    pub laji: Laji,
    pub nimi: &'static str,
    pub kuvaus: &'static str,
    pub malli: fn() -> Vec<Tahko>,
    /// Lista, jonka valikon paneeli näyttää. Voi riippua tallennetusta asetuksesta.
    pub lista: fn(&Tallennus) -> Nettipeli,
}

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

/// Tuottaa pelien luettelosta `Laji`-tyypin, valikon kortit `PELIT` ja `Kaynnissa`-muunnelmat
/// haaroineen. Jokaisella pelillä on `new(siemen)`, ja muun käynnistin hoitaa
/// `Pelinakyma`-traitin kautta. Kortin tekstit ovat `kielet.txt`:n avaimia.
macro_rules! pelit {
    ($($laji:ident($tyyppi:ty, $nimi:literal, $kuvaus:literal, $malli:expr, $lista:expr)),* $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum Laji {
            $($laji,)*
        }

        /// Valikon kortit järjestyksessä. Tallennus muistaa valinnan paikan, joten uusi peli
        /// lisätään loppuun.
        pub const PELIT: [Kortti; [$(Laji::$laji),*].len()] = [
            $(Kortti {
                laji: Laji::$laji,
                nimi: $nimi,
                kuvaus: $kuvaus,
                malli: $malli,
                lista: $lista,
            },)*
        ];

        pub enum Kaynnissa {
            $($laji(Box<$tyyppi>),)*
        }

        impl Kaynnissa {
            /// Uusi peli alkutilassaan, ilman tallennettuja asetuksia ja listoja.
            fn uusi(laji: Laji, siemen: u64) -> Self {
                match laji {
                    $(Laji::$laji => Kaynnissa::$laji(Box::new(<$tyyppi>::new(siemen))),)*
                }
            }

            pub fn nakyma(&self) -> &dyn Pelinakyma {
                match self {
                    $(Kaynnissa::$laji(peli) => &**peli,)*
                }
            }

            pub fn nakyma_mut(&mut self) -> &mut dyn Pelinakyma {
                match self {
                    $(Kaynnissa::$laji(peli) => &mut **peli,)*
                }
            }

            pub fn laji(&self) -> Laji {
                match self {
                    $(Kaynnissa::$laji(_) => Laji::$laji,)*
                }
            }
        }
    };
}

pelit! {
    Laivanupotus(Peli, "peli.laivanupotus", "valikko.laivanupotus", laivamalli, |_| Nettipeli::Laivanupotus),
    Luvut(Luvut, "peli.luvut", "valikko.luvut", lukupino, |_| Nettipeli::Luvut),
    // Viimeksi valitun lajin lista: sen pelaaja haluaa todennäköisimmin nähdä.
    Pasianssi(Pasianssi, "peli.pasianssi", "valikko.pasianssi", korttipinot, |tallennus| {
        pasianssi_peli::nettipeli(tallennus.asetukset.variantti)
    }),
    Mahjong(Mahjong, "peli.mahjong", "valikko.mahjong", laattamalli, |_| Nettipeli::Mahjong),
    Tiilirikko(Tiilirikko, "peli.tiilirikko", "valikko.tiilirikko", tiilimuuri, |_| Nettipeli::Tiilirikko),
    Asteroidit(Asteroidit, "peli.asteroidit", "valikko.asteroidit", kivimalli, |_| Nettipeli::Asteroidit),
    Silmukka(Silmukka, "peli.silmukka", "valikko.silmukka", ouroboros, |_| Nettipeli::Silmukka),
    Palikka(Palikkapeli, "peli.palikka", "valikko.palikka", jalokivi, |_| Nettipeli::Palikka),
    Miinaharava(Miinaharava, "peli.miinaharava", "valikko.miinaharava", miinamalli, |_| Nettipeli::Miinaharava),
    Blackjack(Blackjack, "peli.blackjack", "valikko.blackjack", pelimerkkipinot, |_| Nettipeli::Blackjack),
    Holdem(Holdem, "peli.holdem", "valikko.holdem", poytamalli, |_| Nettipeli::Holdem),
    Majakka(Majakka, "peli.majakka", "valikko.majakka", majakkamalli, |_| Nettipeli::Majakka),
    Kaivo(Kaivo, "peli.kaivo", "valikko.kaivo", kaivosmalli, |_| Nettipeli::Kaivo),
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
    pub fn valikon_lista(&self) -> Nettipeli {
        (PELIT[self.valittu].lista)(&self.tallennus)
    }

    /// Listan laji, kun saman pelin lajeilla on omat listansa.
    pub fn valikon_listan_laji(&self) -> Option<&'static str> {
        (PELIT[self.valittu].laji == Laji::Pasianssi)
            .then(|| lajin_nimi(self.tallennus.asetukset.variantti))
    }

    /// Valitun pelin oma paras tulos, myös listalle yltämätön. `None`, jos tulosta ei ole.
    pub fn valitun_oma_paras(&self) -> Option<u32> {
        let lista = self.valikon_lista();
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
    pub fn valitun_parhaat(&self) -> (bool, &[Tulos]) {
        let lista = self.valikon_lista();
        match &self.netti[lista as usize] {
            Some(netti) => (true, netti),
            None => (false, &self.tallennus.lista(lista).parhaat),
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

    /// Esillä oleva peli seuraa hiirtä myös ikkunan ulkopuolelta.
    pub fn hiiri_ulkonakin(&self) -> bool {
        self.peli
            .as_ref()
            .is_some_and(|peli| peli.nakyma().hiiri_ulkonakin())
    }

    /// Pohjassa olevat suunnat. Näppäintoisto olisi mailan ohjaukseen liian nykivä.
    /// Hiiren oikea nappi käy ylöspäin vain `aseta_ohjaus`-kutsussa.
    pub fn aseta_ohjaus(&mut self, vasen: bool, oikea: bool, ylos: bool, alas: bool, hiiri: bool) {
        if let Some(peli) = &mut self.peli {
            let vaaka = oikea as i8 - vasen as i8;
            let nakyma = peli.nakyma_mut();
            nakyma.aseta_ohjaus(vaaka, ylos || hiiri);
            nakyma.aseta_nuolet(vaaka, alas as i8 - ylos as i8);
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
        let mut peli = Kaynnissa::uusi(laji, siemen);
        let nakyma = peli.nakyma_mut();
        nakyma.lue_tallennuksesta(&self.tallennus);
        nakyma.aanet().paalla = self.aanet_paalla;
        for taulu in nakyma.listat() {
            let tallennettu = self.tallennus.lista(taulu.peli);
            taulu.paras = tallennettu.paras;
            taulu.parhaat = tallennettu.parhaat.clone();
            taulu.netti = self.netti[taulu.peli as usize].clone();
            // Lista voi olla muuttunut valikossa olon aikana, joten se haetaan uudelleen.
            let pyynto = Pyynto::Hae(taulu.peli);
            if !self.pyynnot.contains(&pyynto) {
                self.pyynnot.push(pyynto);
            }
            self.haettu[taulu.peli as usize] = true;
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
                    for taulu in kaynnissa.nakyma_mut().listat() {
                        if taulu.peli == peli {
                            taulu.netti = Some(lista.clone());
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
        peli.nakyma().kirjaa_tallennukseen(&mut self.tallennus);
        for taulu in peli.nakyma_mut().listat() {
            let tallennettu = self.tallennus.lista_mut(taulu.peli);
            tallennettu.paras = taulu.paras;
            tallennettu.parhaat = taulu.parhaat.clone();
            self.netti[taulu.peli as usize] = taulu.netti.clone();
            // Jono kirjataan tiedostoon, jottei tulos katoa, vaikka lähetys epäonnistuisi.
            if let Some(tulos) = taulu.lahetys.take() {
                if taulu.peli.yksi_per_nimi() {
                    // Uusi tulos korvaa saman nimen vanhan, joten jonoon jää vain uusin.
                    paivita_nimen_tulos(&mut tallennettu.jono, tulos.clone());
                } else {
                    lisaa_tulos(&mut tallennettu.jono, tulos.clone());
                }
                self.pyynnot.push(Pyynto::Laheta(taulu.peli, tulos));
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
        let lista = self.valikon_lista();
        if !self.haettu[lista as usize] {
            self.haettu[lista as usize] = true;
            self.pyynnot.push(Pyynto::Hae(lista));
        }
    }

    pub fn ota_aanet(&mut self) -> Vec<(Aani, u32)> {
        self.peli.as_mut().map_or_else(Vec::new, |peli| {
            std::mem::take(&mut peli.nakyma_mut().aanet().jono)
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
        self.aanet_paalla = nakyma.aanet().paalla;
        nakyma.kirjaa_asetukset(&mut self.tallennus.asetukset);
        if nakyma.lopeta() {
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
mod tests;
