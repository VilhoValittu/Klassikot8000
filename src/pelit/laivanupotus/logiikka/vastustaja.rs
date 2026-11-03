use super::{KOKO, Ruudut, Ruutu};
use crate::Arpa;

const SUUNNAT: [(isize, isize); 4] = [(0, 1), (1, 0), (0, -1), (-1, 0)];

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

    /// Voiton pisteiden kerroin: vaikeampi vastustaja on vaikeampi voittaa.
    pub const fn kerroin(self) -> u32 {
        match self {
            Vaikeus::Helppo => 1,
            Vaikeus::Normaali => 2,
            Vaikeus::Vaikea => 3,
        }
    }
}

/// Se, minkä ampuja näkee laudasta. Kelluvien laivojen paikkoja ei ole mukana,
/// joten tekoäly ei voi huijata.
#[derive(Clone, Debug, Default)]
pub struct Tiedot {
    pub ammutut: Ruudut,
    /// Osumat laivoihin, jotka eivät ole vielä uponneet.
    pub osumat: Ruudut,
    pub upotetut: Ruudut,
    /// Uppoamattomien laivojen pituudet.
    pub jaljella: Vec<usize>,
}

/// Valitsee ruudun, johon ei ole ammuttu. `None`, jos lauta on täynnä.
pub fn valitse(vaikeus: Vaikeus, tiedot: &Tiedot, arpa: &mut Arpa) -> Option<Ruutu> {
    let valinta = match vaikeus {
        Vaikeus::Helppo => None,
        Vaikeus::Normaali => jahtaa(tiedot, arpa),
        Vaikeus::Vaikea => todennakoisin(tiedot, arpa),
    };
    valinta.or_else(|| satunnainen(tiedot.ammutut, arpa))
}

fn kaikki_ruudut() -> impl Iterator<Item = Ruutu> {
    (0..KOKO * KOKO).filter_map(|i| Ruutu::new(i / KOKO, i % KOKO))
}

fn arvo(ruudut: &[Ruutu], arpa: &mut Arpa) -> Option<Ruutu> {
    (!ruudut.is_empty()).then(|| ruudut[arpa.alle(ruudut.len())])
}

fn satunnainen(ammutut: Ruudut, arpa: &mut Arpa) -> Option<Ruutu> {
    let vapaat: Vec<_> = kaikki_ruudut()
        .filter(|&ruutu| !ammutut.sisaltaa(ruutu))
        .collect();
    arvo(&vapaat, arpa)
}

fn naapuri(ruutu: Ruutu, drivi: isize, dsarake: isize) -> Option<Ruutu> {
    Ruutu::new(
        (ruutu.rivi as usize).checked_add_signed(drivi)?,
        (ruutu.sarake as usize).checked_add_signed(dsarake)?,
    )
}

/// Normaali: osuman jälkeen ammutaan viereen. Kaksi osumaa kertoo laivan suunnan,
/// joten silloin jatketaan vain linjan päistä.
fn jahtaa(tiedot: &Tiedot, arpa: &mut Arpa) -> Option<Ruutu> {
    let mut viereiset = Vec::new();
    let mut linjassa = Vec::new();
    for osuma in kaikki_ruudut().filter(|&r| tiedot.osumat.sisaltaa(r)) {
        for (drivi, dsarake) in SUUNNAT {
            let Some(kohde) = naapuri(osuma, drivi, dsarake) else {
                continue;
            };
            if tiedot.ammutut.sisaltaa(kohde) {
                continue;
            }
            let takana = naapuri(osuma, -drivi, -dsarake);
            if takana.is_some_and(|r| tiedot.osumat.sisaltaa(r)) {
                linjassa.push(kohde);
            } else {
                viereiset.push(kohde);
            }
        }
    }
    if linjassa.is_empty() {
        arvo(&viereiset, arpa)
    } else {
        arvo(&linjassa, arpa)
    }
}

/// Vaikea: lasketaan jokaiselle ruudulle, moniko jäljellä olevien laivojen mahdollinen
/// paikka sen peittää, ja ammutaan suurimpaan. Paikat rajaavat hudit, uponneet laivat
/// ja sääntö, etteivät laivat kosketa toisiaan.
fn todennakoisin(tiedot: &Tiedot, arpa: &mut Arpa) -> Option<Ruutu> {
    let kielletyt = tiedot
        .upotetut
        .ymparys()
        .yhdiste(tiedot.ammutut.erotus(tiedot.osumat));
    let mut pisteet = [0u32; KOKO * KOKO];
    for &pituus in &tiedot.jaljella {
        for (drivi, dsarake) in [(0, 1), (1, 0)] {
            for alku in kaikki_ruudut() {
                let Some(paikka) = laivan_paikka(alku, pituus, drivi, dsarake) else {
                    continue;
                };
                if paikka.leikkaako(kielletyt) {
                    continue;
                }
                // Viereinen osuma kuuluisi toiseen laivaan, joka koskettaisi tätä.
                if paikka.ymparys().erotus(paikka).leikkaako(tiedot.osumat) {
                    continue;
                }
                // Avoimen osuman aikana kelpaavat vain paikat, jotka selittävät osuman.
                let peitetyt = paikka.leikkaus(tiedot.osumat).maara() as u32;
                let paino = if tiedot.osumat.maara() == 0 {
                    1
                } else {
                    peitetyt
                };
                for ruutu in kaikki_ruudut().filter(|&r| paikka.sisaltaa(r)) {
                    pisteet[ruutu.rivi as usize * KOKO + ruutu.sarake as usize] += paino;
                }
            }
        }
    }

    let vapaat = || kaikki_ruudut().filter(|&r| !tiedot.ammutut.sisaltaa(r));
    let pisteet_ruudulle = |r: Ruutu| pisteet[r.rivi as usize * KOKO + r.sarake as usize];
    let paras = vapaat().map(pisteet_ruudulle).max().filter(|&p| p > 0)?;
    let parhaat: Vec<_> = vapaat().filter(|&r| pisteet_ruudulle(r) == paras).collect();
    arvo(&parhaat, arpa)
}

fn laivan_paikka(alku: Ruutu, pituus: usize, drivi: isize, dsarake: isize) -> Option<Ruudut> {
    let mut paikka = Ruudut::default();
    for i in 0..pituus as isize {
        paikka.lisaa(naapuri(alku, drivi * i, dsarake * i)?);
    }
    Some(paikka)
}
