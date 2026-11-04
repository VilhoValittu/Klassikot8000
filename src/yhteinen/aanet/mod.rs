//! Doom-henkiset äänet. Kaikki syntetisoidaan itse, joten paketissa ei ole äänitiedostoja
//! eikä Doomin materiaalia. Äänilaite näkyy vain tiedostossa `main.rs`.

use std::f32::consts::TAU;

use klassikot8000::Arpa;

/// Sama karkea näytetaajuus ja 8 bitin tarkkuus kuin Doomin omissa äänissä.
pub const NAYTETAAJUUS: usize = 11025;
const VOIMAKKUUS: f32 = 0.8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Aani {
    /// Raskas metallinen ovi aukeaa, kun peli alkaa.
    Ovi,
    /// Haulikon laukaus.
    Laukaus,
    /// Vastustajan tulipallo.
    Tulipallo,
    Roiske,
    Rajahdys,
    /// Iso räjähdys ja demonin kuolinkorina.
    Upotus,
    /// Ähkäisy, kun ruutuun on jo ammuttu.
    Ahkaisy,
    Voitto,
    Tappio,
    /// 2048: luvut tömähtävät reunaan.
    Liuku,
    /// 2048: kaksi lukua yhdistyy.
    Yhdistys,
    /// Tiilirikko: pallo kimpoaa seinästä tai mailasta.
    Pomppu,
    /// Tiilirikko: pallo osuu tiileen.
    Tiili,
    /// Asteroidit: laukaus.
    Ammus,
    /// Asteroidit: kivi hajoaa.
    Kivi,
    /// Asteroidit: aluksen moottori.
    Moottori,
    /// Asteroidit: lentävän lautasen ulina.
    Lautanen,
    /// Silmukka: omena rusahtaa.
    Omena,
    /// Silmukka: tyhjä silmukka haihtuu.
    Puhkus,
    /// Silmukka: muna kuoriutuu.
    Surina,
    /// Silmukka: saaliin nouseva helinä, yksi sävel saalista kohden.
    Helina1,
    Helina2,
    Helina3,
    Helina4,
    Helina5,
    Helina6,
}

const AANIA: usize = 26;

struct Soitto {
    aani: Aani,
    viive: usize,
    kohta: usize,
}

/// Soittaa päällekkäisiä ääniä. Ei tiedä äänilaitteesta mitään, joten sitä voi testata.
pub struct Mikseri {
    naytteet: [Vec<i8>; AANIA],
    soivat: Vec<Soitto>,
    paalla: bool,
}

impl Mikseri {
    pub fn new() -> Self {
        let mut arpa = Arpa::new(0x0666);
        Self {
            naytteet: [
                ovi(&mut arpa),
                laukaus(&mut arpa),
                tulipallo(&mut arpa),
                roiske(&mut arpa),
                rajahdys(&mut arpa),
                upotus(&mut arpa),
                ahkaisy(&mut arpa),
                voitto(&mut arpa),
                tappio(&mut arpa),
                liuku(&mut arpa),
                yhdistys(),
                pomppu(),
                tiili(),
                ammus(),
                kivi(&mut arpa),
                moottori(&mut arpa),
                lautanen(),
                omena(&mut arpa),
                puhkus(&mut arpa),
                surina(),
                helina(523.3),
                helina(587.3),
                helina(659.3),
                helina(784.0),
                helina(880.0),
                helina(1046.5),
            ],
            soivat: Vec::new(),
            paalla: true,
        }
    }

    /// Pois kytkeminen hiljentää myös soivat ja viiveellä odottavat äänet heti.
    pub fn aseta_paalle(&mut self, paalla: bool) {
        self.paalla = paalla;
        if !paalla {
            self.soivat.clear();
        }
    }

    pub fn soita(&mut self, aani: Aani, viive_ms: u32) {
        if self.paalla {
            self.soivat.push(Soitto {
                aani,
                viive: viive_ms as usize * NAYTETAAJUUS / 1000,
                kohta: 0,
            });
        }
    }

    /// Täyttää äänilaitteen puskurin seuraavilla näytteillä.
    pub fn tayta(&mut self, ulos: &mut [f32]) {
        ulos.fill(0.0);
        for soitto in &mut self.soivat {
            let naytteet = &self.naytteet[soitto.aani as usize];
            for naytepaikka in ulos.iter_mut() {
                if soitto.viive > 0 {
                    soitto.viive -= 1;
                } else if let Some(&nayte) = naytteet.get(soitto.kohta) {
                    *naytepaikka += nayte as f32 / 127.0 * VOIMAKKUUS;
                    soitto.kohta += 1;
                }
            }
        }
        let naytteet = &self.naytteet;
        self.soivat
            .retain(|s| s.kohta < naytteet[s.aani as usize].len());
        for nayte in ulos {
            *nayte = nayte.clamp(-1.0, 1.0);
        }
    }
}

fn kohina(arpa: &mut Arpa) -> f32 {
    arpa.alle(2001) as f32 / 1000.0 - 1.0
}

fn saha(vaihe: f32) -> f32 {
    2.0 * vaihe.fract() - 1.0
}

fn kantti(vaihe: f32) -> f32 {
    if vaihe.fract() < 0.5 { 1.0 } else { -1.0 }
}

/// Muuntaa äänen 8-bittiseksi. Yliohjaus leikkautuu, mikä tekee äänestä rouhean.
fn rakenna(kesto: f32, mut aalto: impl FnMut(f32) -> f32) -> Vec<i8> {
    let maara = (kesto * NAYTETAAJUUS as f32) as usize;
    (0..maara)
        .map(|i| {
            let t = i as f32 / NAYTETAAJUUS as f32;
            (aalto(t).clamp(-1.0, 1.0) * 127.0) as i8
        })
        .collect()
}

fn laukaus(arpa: &mut Arpa) -> Vec<i8> {
    let mut suodin = 0.0;
    rakenna(0.35, |t| {
        suodin += (kohina(arpa) - suodin) * 0.55;
        let isku = (TAU * (45.0 * t + 8.0 * (1.0 - (-t * 18.0).exp()))).sin();
        (suodin * 2.5 * (-t * 13.0).exp() + isku * 1.3 * (-t * 20.0).exp()) * 1.3
    })
}

fn roiske(arpa: &mut Arpa) -> Vec<i8> {
    let mut edellinen = 0.0;
    rakenna(0.45, |t| {
        let uusi = kohina(arpa);
        let kirkas = uusi - edellinen;
        edellinen = uusi;
        kirkas * 0.5 * (t * 30.0).min(1.0) * (-t * 8.0).exp()
    })
}

/// Räjähdyksen jyrinä: kohina pidetään yhä pidempään samassa arvossa, jolloin se madaltuu.
fn jyrina(arpa: &mut Arpa, kesto: f32, vaimeneminen: f32) -> Vec<i8> {
    let mut arvo = 0.0;
    let mut laskuri = 0.0;
    rakenna(kesto, |t| {
        laskuri -= 1.0;
        if laskuri <= 0.0 {
            arvo = kohina(arpa);
            laskuri = 1.0 + t * 14.0;
        }
        let pohja = (TAU * 48.0 * t).sin() * 0.6;
        (arvo + pohja) * 2.2 * (-t * vaimeneminen).exp()
    })
}

fn rajahdys(arpa: &mut Arpa) -> Vec<i8> {
    jyrina(arpa, 0.7, 5.0)
}

fn upotus(arpa: &mut Arpa) -> Vec<i8> {
    let pamaus = jyrina(arpa, 0.9, 3.5);
    let mut vaihe = 0.0;
    let korina = rakenna(1.3, |t| {
        vaihe += (170.0 * (-t * 1.6).exp() + 38.0) / NAYTETAAJUUS as f32;
        let tarina = 0.55 + 0.45 * kantti(t * 31.0);
        let aanne = saha(vaihe) * tarina + kohina(arpa) * 0.2;
        aanne * 1.3 * (t * 12.0).min(1.0) * (1.0 - t / 1.3)
    });
    sekoita(&pamaus, &korina, NAYTETAAJUUS / 10)
}

fn sekoita(a: &[i8], b: &[i8], b_alkaa: usize) -> Vec<i8> {
    let maara = a.len().max(b_alkaa + b.len());
    (0..maara)
        .map(|i| {
            let x = a.get(i).copied().unwrap_or(0) as i16;
            let y = i
                .checked_sub(b_alkaa)
                .and_then(|j| b.get(j).copied())
                .unwrap_or(0) as i16;
            (x + y).clamp(-127, 127) as i8
        })
        .collect()
}

fn tulipallo(arpa: &mut Arpa) -> Vec<i8> {
    let mut suodin = 0.0;
    let mut vaihe = 0.0;
    rakenna(0.4, |t| {
        // suodin avautuu ja sulkeutuu, pallo tuntuu lentävän ohi
        let auki = (t / 0.4 * TAU / 2.0).sin();
        suodin += (kohina(arpa) - suodin) * (0.08 + 0.5 * auki);
        vaihe += (90.0 + 160.0 * auki) / NAYTETAAJUUS as f32;
        (suodin * 1.8 + kantti(vaihe) * 0.2) * auki
    })
}

fn ovi(arpa: &mut Arpa) -> Vec<i8> {
    let mut vaihe = 0.0;
    rakenna(0.75, |t| {
        if t < 0.6 {
            vaihe += (62.0 + 14.0 * (t * 37.0).sin()) / NAYTETAAJUUS as f32;
            (kantti(vaihe) * 0.3 + kohina(arpa) * 0.12) * (t * 10.0).min(1.0)
        } else {
            let t = t - 0.6;
            ((TAU * 70.0 * t).sin() + kohina(arpa) * 0.5) * (-t * 28.0).exp()
        }
    })
}

fn ahkaisy(arpa: &mut Arpa) -> Vec<i8> {
    let mut vaihe = 0.0;
    rakenna(0.22, |t| {
        vaihe += (150.0 - 240.0 * t) / NAYTETAAJUUS as f32;
        let muoto = 0.6 + 0.4 * (TAU * 620.0 * t).sin();
        (saha(vaihe) * muoto + kohina(arpa) * 0.08) * (t * 60.0).min(1.0) * (1.0 - t / 0.22)
    })
}

/// Säröinen kvinttisointu, metallikitaran tapaan.
fn sointu(perus: f32, t: f32, kesto: f32) -> f32 {
    let aalto = kantti(perus * t) + kantti(perus * 1.5 * t + 0.13) + kantti(perus * 2.0 * t);
    let sammuu = (-(t / kesto) * 2.5).exp();
    (aalto * 0.9).clamp(-0.75, 0.75) * sammuu
}

/// Soittaa sointukulun, jonka jokainen pari on (perustaajuus, kesto sekunteina).
fn kulku(sointuja: &[(f32, f32)]) -> Vec<i8> {
    let mut naytteet = Vec::new();
    for &(perus, kesto) in sointuja {
        naytteet.extend(rakenna(kesto, |t| {
            if perus > 0.0 {
                sointu(perus, t, kesto)
            } else {
                0.0
            }
        }));
    }
    naytteet
}

fn voitto(_arpa: &mut Arpa) -> Vec<i8> {
    kulku(&[
        (82.4, 0.12),
        (82.4, 0.12),
        (98.0, 0.18),
        (82.4, 0.12),
        (110.0, 0.18),
        (116.5, 0.12),
        (110.0, 0.12),
        (164.8, 0.9),
    ])
}

fn tappio(arpa: &mut Arpa) -> Vec<i8> {
    let soinnut = kulku(&[(110.0, 0.4), (98.0, 0.4), (87.3, 0.4), (55.0, 1.3)]);
    let mut vaihe = 0.0;
    let huuto = rakenna(0.9, |t| {
        vaihe += (260.0 * (-t * 2.2).exp() + 60.0) / NAYTETAAJUUS as f32;
        (saha(vaihe) + kohina(arpa) * 0.25) * 0.7 * (t * 25.0).min(1.0) * (1.0 - t / 0.9)
    });
    sekoita(&soinnut, &huuto, NAYTETAAJUUS * 3 / 10)
}

fn liuku(arpa: &mut Arpa) -> Vec<i8> {
    let mut suodin = 0.0;
    rakenna(0.09, |t| {
        suodin += (kohina(arpa) - suodin) * 0.3;
        let isku = (TAU * 90.0 * t).sin();
        (suodin + isku * 0.7) * 0.6 * (-t * 40.0).exp()
    })
}

fn yhdistys() -> Vec<i8> {
    let mut vaihe = 0.0;
    rakenna(0.18, |t| {
        vaihe += (220.0 + 2200.0 * t) / NAYTETAAJUUS as f32;
        kantti(vaihe) * 0.5 * (t * 80.0).min(1.0) * (1.0 - t / 0.18)
    })
}

fn pomppu() -> Vec<i8> {
    rakenna(0.05, |t| kantti(330.0 * t) * 0.35 * (1.0 - t / 0.05))
}

fn tiili() -> Vec<i8> {
    rakenna(0.07, |t| kantti(880.0 * t) * 0.35 * (1.0 - t / 0.07))
}

fn ammus() -> Vec<i8> {
    let mut vaihe = 0.0;
    rakenna(0.12, |t| {
        vaihe += (1400.0 - 8000.0 * t) / NAYTETAAJUUS as f32;
        kantti(vaihe) * 0.3 * (1.0 - t / 0.12)
    })
}

fn kivi(arpa: &mut Arpa) -> Vec<i8> {
    jyrina(arpa, 0.2, 16.0)
}

fn moottori(arpa: &mut Arpa) -> Vec<i8> {
    let mut suodin = 0.0;
    rakenna(0.14, |_| {
        suodin += (kohina(arpa) - suodin) * 0.12;
        suodin * 0.9
    })
}

fn lautanen() -> Vec<i8> {
    let mut vaihe = 0.0;
    rakenna(0.3, |t| {
        vaihe += (620.0 + 160.0 * kantti(t * 13.0)) / NAYTETAAJUUS as f32;
        kantti(vaihe) * 0.12
    })
}

fn omena(arpa: &mut Arpa) -> Vec<i8> {
    let mut vaihe = 0.0;
    rakenna(0.09, |t| {
        vaihe += (700.0 - 3500.0 * t) / NAYTETAAJUUS as f32;
        (kantti(vaihe) * 0.3 + kohina(arpa) * 0.25) * (1.0 - t / 0.09)
    })
}

fn puhkus(arpa: &mut Arpa) -> Vec<i8> {
    let mut suodin = 0.0;
    rakenna(0.3, |t| {
        suodin += (kohina(arpa) - suodin) * (0.35 - t);
        suodin * 0.8 * (1.0 - t / 0.3)
    })
}

fn surina() -> Vec<i8> {
    let mut vaihe = 0.0;
    rakenna(0.22, |t| {
        vaihe += (420.0 + 90.0 * (TAU * 35.0 * t).sin()) / NAYTETAAJUUS as f32;
        saha(vaihe) * 0.18 * (1.0 - t / 0.22)
    })
}

/// Kirkas sävel oktaavin kanssa. Pentatoninen asteikko kuulostaa hyvältä missä tahansa järjestyksessä.
fn helina(taajuus: f32) -> Vec<i8> {
    rakenna(0.35, |t| {
        let perus = (TAU * taajuus * t).sin();
        let harmoninen = (TAU * taajuus * 2.0 * t).sin() * 0.4;
        (perus + harmoninen) * 0.32 * (-t * 9.0).exp()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn soi(mikseri: &mut Mikseri) -> bool {
        let mut puskuri = [0.0; 256];
        mikseri.tayta(&mut puskuri);
        puskuri.iter().any(|&nayte| nayte != 0.0)
    }

    #[test]
    fn jokainen_aani_soi_ja_loppuu() {
        for aani in [
            Aani::Ovi,
            Aani::Laukaus,
            Aani::Tulipallo,
            Aani::Roiske,
            Aani::Rajahdys,
            Aani::Upotus,
            Aani::Ahkaisy,
            Aani::Voitto,
            Aani::Tappio,
            Aani::Liuku,
            Aani::Yhdistys,
            Aani::Pomppu,
            Aani::Tiili,
            Aani::Ammus,
            Aani::Kivi,
            Aani::Moottori,
            Aani::Lautanen,
            Aani::Omena,
            Aani::Puhkus,
            Aani::Surina,
            Aani::Helina1,
            Aani::Helina2,
            Aani::Helina3,
            Aani::Helina4,
            Aani::Helina5,
            Aani::Helina6,
        ] {
            let mut mikseri = Mikseri::new();
            mikseri.soita(aani, 0);
            assert!(soi(&mut mikseri), "{aani:?} on hiljaa");
            let mut puskuri = vec![0.0; NAYTETAAJUUS * 5];
            mikseri.tayta(&mut puskuri);
            assert!(mikseri.soivat.is_empty(), "{aani:?} ei lopu");
            assert!(puskuri.iter().all(|nayte| nayte.abs() <= 1.0));
        }
    }

    #[test]
    fn pois_kytkeminen_hiljentaa_soivan_ja_odottavan_aanen_heti() {
        let mut mikseri = Mikseri::new();
        mikseri.soita(Aani::Tappio, 0);
        mikseri.soita(Aani::Voitto, 1000);
        assert!(soi(&mut mikseri));
        mikseri.aseta_paalle(false);
        assert!(!soi(&mut mikseri));

        mikseri.soita(Aani::Laukaus, 0);
        assert!(!soi(&mut mikseri));

        // takaisin kytkettynä vanhat äänet eivät palaa, uudet soivat
        mikseri.aseta_paalle(true);
        assert!(!soi(&mut mikseri));
        mikseri.soita(Aani::Laukaus, 0);
        assert!(soi(&mut mikseri));
    }

    #[test]
    fn viive_siirtaa_aanen_alkua() {
        let mut mikseri = Mikseri::new();
        mikseri.soita(Aani::Laukaus, 1000);
        let mut puskuri = vec![0.0; NAYTETAAJUUS - 1];
        mikseri.tayta(&mut puskuri);
        assert!(puskuri.iter().all(|&nayte| nayte == 0.0));
        assert!(soi(&mut mikseri));
    }
}
