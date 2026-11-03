//! Miinaharavan säännöt ilman piirtoa. Miinat arvotaan vasta ensimmäisellä avauksella,
//! jotta ensimmäinen ruutu ja sen naapurit ovat aina turvallisia.

use crate::Arpa;

/// (rivi, sarake)
pub type Paikka = (usize, usize);

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

    pub fn korkeus(self) -> usize {
        match self {
            Vaikeus::Helppo => 9,
            Vaikeus::Normaali | Vaikeus::Vaikea => 16,
        }
    }

    pub fn leveys(self) -> usize {
        match self {
            Vaikeus::Helppo => 9,
            Vaikeus::Normaali => 16,
            Vaikeus::Vaikea => 30,
        }
    }

    pub fn miinoja(self) -> usize {
        match self {
            Vaikeus::Helppo => 10,
            Vaikeus::Normaali => 40,
            Vaikeus::Vaikea => 99,
        }
    }

    /// Pisteiden kerroin: isompi kenttä vie pidempään, joten sen aika on arvokkaampaa.
    pub fn kerroin(self) -> u32 {
        match self {
            Vaikeus::Helppo => 1,
            Vaikeus::Normaali => 2,
            Vaikeus::Vaikea => 3,
        }
    }
}

/// Kello pysähtyy tähän, kuten klassisessa miinaharavassa.
pub const SUURIN_AIKA: u32 = 999;

/// Voiton pisteet top-listalle: mitä nopeammin, sitä enemmän. Voitto antaa aina jotain.
pub fn aikapisteet(vaikeus: Vaikeus, sekunnit: u32) -> u32 {
    (SUURIN_AIKA + 1).saturating_sub(sekunnit).max(1) * vaikeus.kerroin()
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Ruutu {
    pub miina: bool,
    pub auki: bool,
    pub lippu: bool,
    pub naapureita: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tila {
    /// Miinoja ei ole vielä arvottu.
    Alussa,
    Kesken,
    Voitto,
    Tappio,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Avaus {
    /// Mitään ei tapahtunut: ruutu oli jo auki tai liputettu, tai peli on ohi.
    Ei,
    /// Näin monta ruutua aukesi.
    Auki(usize),
    Miina,
    Voitto,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Miinakentta {
    pub vaikeus: Vaikeus,
    pub tila: Tila,
    /// Ruutu, jonka miina räjähti.
    pub rajahtanyt: Option<Paikka>,
    ruudut: Vec<Ruutu>,
    miinoja: usize,
    avattuja: usize,
}

impl Miinakentta {
    pub fn new(vaikeus: Vaikeus) -> Self {
        Self {
            vaikeus,
            tila: Tila::Alussa,
            rajahtanyt: None,
            ruudut: vec![Ruutu::default(); vaikeus.korkeus() * vaikeus.leveys()],
            miinoja: vaikeus.miinoja(),
            avattuja: 0,
        }
    }

    /// Kenttä valmiilla miinoilla, kokeita varten.
    pub fn miinoista(vaikeus: Vaikeus, miinat: &[Paikka]) -> Self {
        let mut kentta = Self::new(vaikeus);
        kentta.aseta_miinat(miinat);
        kentta
    }

    pub fn ruutu(&self, p: Paikka) -> Ruutu {
        self.ruudut[self.indeksi(p)]
    }

    pub fn liput(&self) -> usize {
        self.ruudut.iter().filter(|r| r.lippu).count()
    }

    /// Negatiivinen, jos lippuja on enemmän kuin miinoja.
    pub fn miinoja_jaljella(&self) -> i32 {
        self.miinoja as i32 - self.liput() as i32
    }

    pub fn paattynyt(&self) -> bool {
        matches!(self.tila, Tila::Voitto | Tila::Tappio)
    }

    fn indeksi(&self, (rivi, sarake): Paikka) -> usize {
        rivi * self.vaikeus.leveys() + sarake
    }

    pub fn naapurit(&self, (rivi, sarake): Paikka) -> impl Iterator<Item = Paikka> + use<> {
        let (korkeus, leveys) = (self.vaikeus.korkeus(), self.vaikeus.leveys());
        (rivi.saturating_sub(1)..(rivi + 2).min(korkeus))
            .flat_map(move |r| {
                (sarake.saturating_sub(1)..(sarake + 2).min(leveys)).map(move |s| (r, s))
            })
            .filter(move |&q| q != (rivi, sarake))
    }

    fn aseta_miinat(&mut self, miinat: &[Paikka]) {
        self.miinoja = miinat.len();
        for &p in miinat {
            let i = self.indeksi(p);
            self.ruudut[i].miina = true;
        }
        for &p in miinat {
            for q in self.naapurit(p) {
                let i = self.indeksi(q);
                self.ruudut[i].naapureita += 1;
            }
        }
        self.tila = Tila::Kesken;
    }

    fn arvo_miinat(&mut self, ensimmainen: Paikka, arpa: &mut Arpa) {
        let (korkeus, leveys) = (self.vaikeus.korkeus(), self.vaikeus.leveys());
        let mut vapaat: Vec<Paikka> = (0..korkeus)
            .flat_map(|r| (0..leveys).map(move |s| (r, s)))
            .filter(|p| p.0.abs_diff(ensimmainen.0) > 1 || p.1.abs_diff(ensimmainen.1) > 1)
            .collect();
        let miinat: Vec<Paikka> = (0..self.vaikeus.miinoja())
            .map(|_| vapaat.swap_remove(arpa.alle(vapaat.len())))
            .collect();
        self.aseta_miinat(&miinat);
    }

    /// Avaa ruudun. Tyhjä ruutu avaa myös naapurinsa, joten alue aukeaa kerralla.
    pub fn avaa(&mut self, p: Paikka, arpa: &mut Arpa) -> Avaus {
        if self.paattynyt() || self.ruutu(p).lippu {
            return Avaus::Ei;
        }
        if self.tila == Tila::Alussa {
            self.arvo_miinat(p, arpa);
        }
        if self.ruutu(p).auki {
            return Avaus::Ei;
        }
        if self.ruutu(p).miina {
            self.tila = Tila::Tappio;
            self.rajahtanyt = Some(p);
            return Avaus::Miina;
        }
        let mut avatut = 0;
        let mut jono = vec![p];
        while let Some(q) = jono.pop() {
            let i = self.indeksi(q);
            let ruutu = self.ruudut[i];
            if ruutu.auki || ruutu.lippu {
                continue;
            }
            self.ruudut[i].auki = true;
            avatut += 1;
            if ruutu.naapureita == 0 {
                jono.extend(self.naapurit(q));
            }
        }
        self.avattuja += avatut;
        if self.avattuja == self.ruudut.len() - self.miinoja {
            self.tila = Tila::Voitto;
            // Avaamatta jääneet ovat varmasti miinoja, joten laskuri menee nollaan.
            for ruutu in &mut self.ruudut {
                ruutu.lippu = ruutu.miina;
            }
            return Avaus::Voitto;
        }
        Avaus::Auki(avatut)
    }

    /// Vaihtaa lipun. Palauttaa tiedon, muuttuiko mitään.
    pub fn liputa(&mut self, p: Paikka) -> bool {
        let i = self.indeksi(p);
        if self.paattynyt() || self.ruudut[i].auki {
            return false;
        }
        self.ruudut[i].lippu = !self.ruudut[i].lippu;
        true
    }

    /// Avatun numeron ympäriltä avataan loput, kun lippuja on yhtä monta kuin numero sanoo.
    /// Väärä lippu räjäyttää miinan aivan kuten suora avaus.
    pub fn avaa_ymparilta(&mut self, p: Paikka, arpa: &mut Arpa) -> Avaus {
        let ruutu = self.ruutu(p);
        let lippuja = self.naapurit(p).filter(|&q| self.ruutu(q).lippu).count();
        if self.paattynyt()
            || !ruutu.auki
            || ruutu.naapureita == 0
            || lippuja != ruutu.naapureita as usize
        {
            return Avaus::Ei;
        }
        let mut avatut = 0;
        for q in self.naapurit(p) {
            match self.avaa(q, arpa) {
                Avaus::Ei => {}
                Avaus::Auki(maara) => avatut += maara,
                loppu => return loppu,
            }
        }
        if avatut == 0 {
            Avaus::Ei
        } else {
            Avaus::Auki(avatut)
        }
    }
}
