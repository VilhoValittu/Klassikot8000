//! Blackjackin säännöt ilman ikkunaa, piirtoa tai syötettä. Panos otetaan rahoista
//! jaon alussa, ja palautus tulee kierroksen lopussa. Kaikki rahamäärät ovat puolikkaina
//! (`YKSI`), jotta 3:2-voitto ja puolen panoksen vakuutus menevät tasan parittomallakin panoksella.

use crate::Arpa;
use crate::kortit::sekoitettu;
pub use crate::kortit::{Kortti, Maa};

/// Yksi raha puolikkaina. Panokset ovat aina kokonaisia rahoja.
pub const YKSI: u32 = 2;
pub const ALKURAHA: u32 = 300 * YKSI;
pub const PIENIN_PANOS: u32 = YKSI;
pub const SUURIN_TULOS: u32 = 999_999;
pub const ENITEN_KASIA: usize = 4;
pub const JAKAJA_JAA: u32 = 17;

const BLACKJACK: u32 = 21;
pub const PAKKOJA: usize = 6;
// Yksi kierros ei voi kuluttaa tätä enempää: pienimmätkin kortit täyttävät neljä kättä
// ja jakajan noin 60 kortilla, joten kengästä ei lopu kortit kesken kierroksen.
const SEKOITUSRAJA: usize = 78;

/// Käden summa ja tieto siitä, lasketaanko jokin ässä yhdestätoista.
pub fn summa(kortit: &[Kortti]) -> (u32, bool) {
    let kovat: u32 = kortit.iter().map(|k| k.arvo.min(10) as u32).sum();
    if kortit.iter().any(|k| k.arvo == 1) && kovat + 10 <= BLACKJACK {
        (kovat + 10, true)
    } else {
        (kovat, false)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tulos {
    Blackjack,
    Voitto,
    Tasan,
    Tappio,
}

impl Tulos {
    /// Paljonko rahaa palaa, panos mukaan lukien. Blackjack maksaa 3:2.
    pub fn palautus(self, panos: u32) -> u32 {
        match self {
            Tulos::Blackjack => panos.saturating_add(panos / 2).saturating_add(panos),
            Tulos::Voitto => panos.saturating_mul(2),
            Tulos::Tasan => panos,
            Tulos::Tappio => 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Kasi {
    pub kortit: Vec<Kortti>,
    pub panos: u32,
    pub tuplattu: bool,
    /// Käsi on syntynyt halkaisusta, joten sen 21 ei ole blackjack.
    pub halkaistu: bool,
    pub tulos: Option<Tulos>,
}

impl Kasi {
    pub fn summa(&self) -> u32 {
        summa(&self.kortit).0
    }

    pub fn on_yli(&self) -> bool {
        self.summa() > BLACKJACK
    }

    pub fn on_blackjack(&self) -> bool {
        !self.halkaistu && self.kortit.len() == 2 && self.summa() == BLACKJACK
    }

    /// Halkaistu ässä saa vain yhden lisäkortin. Muuten pelaaja pelaa, kunnes jää tai 21 täyttyy.
    fn on_valmis(&self) -> bool {
        self.summa() >= BLACKJACK || self.tuplattu || (self.halkaistu && self.kortit[0].arvo == 1)
    }

    fn tulos_jakajaa_vastaan(&self, jakaja: &[Kortti]) -> Tulos {
        let oma = self.summa();
        let vastassa = summa(jakaja).0;
        let jakajan_blackjack = jakaja.len() == 2 && vastassa == BLACKJACK;
        if self.on_yli() {
            Tulos::Tappio
        } else if self.on_blackjack() {
            if jakajan_blackjack {
                Tulos::Tasan
            } else {
                Tulos::Blackjack
            }
        } else if jakajan_blackjack {
            Tulos::Tappio
        } else if vastassa > BLACKJACK || oma > vastassa {
            Tulos::Voitto
        } else if oma == vastassa {
            Tulos::Tasan
        } else {
            Tulos::Tappio
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vuoro {
    /// Pöydällä voi yhä olla edellisen kierroksen kortit ja tulokset.
    Panostus,
    /// Jakajalla on ässä esillä: vakuutus ratkaistaan ennen kuin jakaja kurkistaa.
    Vakuutus,
    Pelaaja,
    Jakaja,
}

pub struct Poyta {
    pub rahat: u32,
    pub vuoro: Vuoro,
    pub kadet: Vec<Kasi>,
    pub vuorossa: usize,
    pub jakaja: Vec<Kortti>,
    /// Jakajan toinen kortti on nurin.
    pub piilossa: bool,
    /// Tämän kierroksen vakuutus. Nolla, jos sitä ei otettu.
    pub vakuutus: u32,
    /// Päällimmäinen kortti on viimeisenä.
    pub kenka: Vec<Kortti>,
}

impl Poyta {
    pub fn new(arpa: &mut Arpa) -> Self {
        Self {
            rahat: ALKURAHA,
            vuoro: Vuoro::Panostus,
            kadet: Vec::new(),
            vuorossa: 0,
            jakaja: Vec::new(),
            piilossa: false,
            vakuutus: 0,
            kenka: sekoitettu(PAKKOJA, arpa),
        }
    }

    fn nosta(&mut self) -> Kortti {
        self.kenka.pop().expect("kenkä riittää koko kierrokseksi")
    }

    /// Ottaa panoksen ja jakaa kortit. Epätosi, jos panos ei kelpaa.
    pub fn jaa(&mut self, panos: u32, arpa: &mut Arpa) -> bool {
        if self.vuoro != Vuoro::Panostus
            || !(PIENIN_PANOS..=self.rahat).contains(&panos)
            || !panos.is_multiple_of(YKSI)
        {
            return false;
        }
        if self.kenka.len() < SEKOITUSRAJA {
            self.kenka = sekoitettu(PAKKOJA, arpa);
        }
        self.rahat -= panos;
        let (a, b, c, d) = (self.nosta(), self.nosta(), self.nosta(), self.nosta());
        self.kadet = vec![Kasi {
            kortit: vec![a, c],
            panos,
            tuplattu: false,
            halkaistu: false,
            tulos: None,
        }];
        self.jakaja = vec![b, d];
        self.vuorossa = 0;
        self.piilossa = true;
        self.vakuutus = 0;
        if self.jakaja[0].arvo == 1 && self.vakuutuksen_hinta() <= self.rahat {
            self.vuoro = Vuoro::Vakuutus;
        } else {
            self.kurkista();
        }
        true
    }

    pub fn vakuutuksen_hinta(&self) -> u32 {
        self.kadet[0].panos / 2
    }

    /// Ottaa vakuutuksen tai jättää sen ottamatta. Blackjack-kädellä vakuutus on tasaraha:
    /// voitto on panoksen verran, oli jakajalla blackjack tai ei.
    pub fn vakuuta(&mut self, ottaa: bool) -> bool {
        if self.vuoro != Vuoro::Vakuutus {
            return false;
        }
        if ottaa {
            self.vakuutus = self.vakuutuksen_hinta();
            self.rahat -= self.vakuutus;
        }
        self.kurkista();
        true
    }

    /// Jakaja katsoo piilokorttinsa, joten pelaaja ei tuplaa tai halkaise valmiiksi hävittyä kättä.
    /// 21 kahdella kortilla vaatii ässän tai kympin esille, joten muulloin ei ole mitään löydettävää.
    fn kurkista(&mut self) {
        if self.kadet[0].on_blackjack() || self.jakajalla_blackjack() {
            self.piilossa = false;
            self.tilita();
        } else {
            self.vuoro = Vuoro::Pelaaja;
        }
    }

    pub fn jakajalla_blackjack(&self) -> bool {
        self.jakaja.len() == 2 && summa(&self.jakaja).0 == BLACKJACK
    }

    /// Siirtyy seuraavaan käteen, jolla on vielä pelattavaa. Halkaistu käsi saa toisen
    /// korttinsa vasta vuorollaan, joten sitä ei näe ennen edellisen käden ratkaisuja.
    fn etene(&mut self) {
        while self.vuorossa < self.kadet.len() {
            if self.kadet[self.vuorossa].kortit.len() == 1 {
                let kortti = self.nosta();
                self.kadet[self.vuorossa].kortit.push(kortti);
            }
            if !self.kadet[self.vuorossa].on_valmis() {
                break;
            }
            self.vuorossa += 1;
        }
        if self.vuorossa == self.kadet.len() {
            self.vuoro = Vuoro::Jakaja;
        }
    }

    pub fn ota(&mut self) -> bool {
        if self.vuoro != Vuoro::Pelaaja {
            return false;
        }
        let kortti = self.nosta();
        self.kadet[self.vuorossa].kortit.push(kortti);
        self.etene();
        true
    }

    pub fn jaa_tahan(&mut self) -> bool {
        if self.vuoro != Vuoro::Pelaaja {
            return false;
        }
        self.vuorossa += 1;
        self.etene();
        true
    }

    /// Lisäpanos otetaan kahden kortin kädestä, ja se on yhtä suuri kuin käden panos.
    pub fn voi_tuplata(&self) -> bool {
        self.vuoro == Vuoro::Pelaaja
            && self.kadet[self.vuorossa].kortit.len() == 2
            && self.kadet[self.vuorossa].panos <= self.rahat
    }

    pub fn tuplaa(&mut self) -> bool {
        if !self.voi_tuplata() {
            return false;
        }
        let kortti = self.nosta();
        let kasi = &mut self.kadet[self.vuorossa];
        self.rahat -= kasi.panos;
        kasi.panos *= 2;
        kasi.tuplattu = true;
        kasi.kortit.push(kortti);
        self.etene();
        true
    }

    /// Kymppi ja kuvakortti käyvät pariksi, koska niillä on sama arvo.
    pub fn voi_halkaista(&self) -> bool {
        self.voi_tuplata()
            && self.kadet.len() < ENITEN_KASIA
            && self.kadet[self.vuorossa].kortit[0].arvo.min(10)
                == self.kadet[self.vuorossa].kortit[1].arvo.min(10)
    }

    pub fn halkaise(&mut self) -> bool {
        if !self.voi_halkaista() {
            return false;
        }
        let uusi = self.nosta();
        let kasi = &mut self.kadet[self.vuorossa];
        self.rahat -= kasi.panos;
        kasi.halkaistu = true;
        let toinen = kasi.kortit[1];
        kasi.kortit[1] = uusi;
        let seuraava = Kasi {
            kortit: vec![toinen],
            ..kasi.clone()
        };
        self.kadet.insert(self.vuorossa + 1, seuraava);
        self.etene();
        true
    }

    fn jakaja_ottaa(&self) -> bool {
        // Jos kaikki kädet menivät yli, tulos on jo selvä.
        self.kadet.iter().any(|k| !k.on_yli()) && summa(&self.jakaja).0 < JAKAJA_JAA
    }

    /// Yksi jakajan siirto: kääntää piilokortin tai nostaa uuden. Kierros tilitetään
    /// samalla siirrolla, jonka jälkeen jakaja ei enää ota.
    pub fn jakajan_siirto(&mut self) {
        if self.vuoro != Vuoro::Jakaja {
            return;
        }
        if self.piilossa {
            self.piilossa = false;
        } else {
            let kortti = self.nosta();
            self.jakaja.push(kortti);
        }
        if !self.jakaja_ottaa() {
            self.tilita();
        }
    }

    fn tilita(&mut self) {
        for kasi in &mut self.kadet {
            let tulos = kasi.tulos_jakajaa_vastaan(&self.jakaja);
            kasi.tulos = Some(tulos);
            self.rahat = self.rahat.saturating_add(tulos.palautus(kasi.panos));
        }
        // Vakuutus maksaa 2:1.
        if self.vakuutus_voitti() {
            self.rahat = self.rahat.saturating_add(self.vakuutus.saturating_mul(3));
        }
        self.vuoro = Vuoro::Panostus;
    }

    pub fn vakuutus_voitti(&self) -> bool {
        self.vakuutus > 0 && self.jakajalla_blackjack()
    }

    /// Listan tulos: voitto alkurahoihin nähden kokonaisina rahoina. Kesken oleva panos ei kuulu rahoihin.
    pub fn pisteet(&self) -> u32 {
        (self.rahat.saturating_sub(ALKURAHA) / YKSI).min(SUURIN_TULOS)
    }

    /// Kierroksen voitto panosten ja vakuutuksen jälkeen. Tappio on negatiivinen.
    pub fn voitto(&self) -> i64 {
        let kadet: i64 = self
            .kadet
            .iter()
            .filter_map(|k| k.tulos.map(|t| t.palautus(k.panos) as i64 - k.panos as i64))
            .sum();
        let vakuutus = self.vakuutus as i64;
        if self.vakuutus_voitti() {
            kadet + 2 * vakuutus
        } else {
            kadet - vakuutus
        }
    }
}
