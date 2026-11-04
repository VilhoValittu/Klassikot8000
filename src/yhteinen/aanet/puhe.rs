//! Pieni formanttisyntetisaattori kuuluttajan fraaseille. Äänihuulten pulssi tai kohina kulkee
//! kolmen resonaattorin läpi kuten vanhojen pelien puhepiireissä, joten exeen ei tarvita
//! äänitteitä. Suomi lausutaan kuten kirjoitetaan, joten fraasit ovat lähes kirjaimia.

use klassikot8000::Arpa;

use super::{Kaista, NAYTETAAJUUS, Resonaattori, kohina};

const FS: f32 = NAYTETAAJUUS as f32;
/// Suodinten kertoimet lasketaan uudelleen tämän näytemäärän välein, ei jokaisella näytteellä.
const PAIVITYSVALI: usize = 16;

/// Yhden äänteen tavoitearvot. Äänteestä toiseen arvot liukuvat, jolloin syntyvät siirtymät.
#[derive(Clone, Copy)]
struct Kohde {
    formantit: [f32; 3],
    kaistat: [f32; 3],
    /// Äänihuulten osuus.
    aani: f32,
    /// h: kohina kulkee ääntöväylän läpi.
    henkays: f32,
    /// Frikatiivin tai klusiilin purkauksen kohina, joka ohittaa ääntöväylän.
    suhina: f32,
    suhinan_taajuus: f32,
    suhinan_kaista: f32,
    /// Suomen r värisee.
    tarina: bool,
}

const VOKAALIN_KAISTAT: [f32; 3] = [70.0, 100.0, 150.0];

const fn vokaali(f1: f32, f2: f32, f3: f32) -> Kohde {
    Kohde {
        formantit: [f1, f2, f3],
        kaistat: VOKAALIN_KAISTAT,
        aani: 1.0,
        henkays: 0.0,
        suhina: 0.0,
        suhinan_taajuus: 5000.0,
        suhinan_kaista: 2000.0,
        tarina: false,
    }
}

const fn soinnillinen(f: [f32; 3], kaistat: [f32; 3], aani: f32) -> Kohde {
    Kohde {
        formantit: f,
        kaistat,
        aani,
        ..vokaali(0.0, 0.0, 0.0)
    }
}

const fn suhu(f: [f32; 3], aani: f32, suhina: f32, taajuus: f32, kaista: f32) -> Kohde {
    Kohde {
        formantit: f,
        kaistat: VOKAALIN_KAISTAT,
        aani,
        henkays: 0.0,
        suhina,
        suhinan_taajuus: taajuus,
        suhinan_kaista: kaista,
        tarina: false,
    }
}

/// Äänne ja sen kesto millisekunteina. Klusiilit puretaan sulkeumaksi ja purkaukseksi erikseen.
fn aanne(merkki: &str) -> Option<(Kohde, f32)> {
    let vokaali_ms = 80.0;
    Some(match merkki {
        // suomen vokaalit
        "a" => (vokaali(700.0, 1150.0, 2500.0), vokaali_ms),
        "ae" => (vokaali(650.0, 1700.0, 2450.0), vokaali_ms),
        "e" => (vokaali(430.0, 1950.0, 2550.0), vokaali_ms),
        "i" => (vokaali(280.0, 2250.0, 2950.0), vokaali_ms),
        "o" => (vokaali(460.0, 800.0, 2450.0), vokaali_ms),
        "u" => (vokaali(310.0, 850.0, 2250.0), vokaali_ms),
        // englannin vokaalit
        "I" => (vokaali(400.0, 1900.0, 2550.0), vokaali_ms),
        "E" => (vokaali(540.0, 1800.0, 2500.0), vokaali_ms),
        "O" => (vokaali(560.0, 860.0, 2400.0), vokaali_ms),
        "U" => (vokaali(420.0, 1050.0, 2300.0), vokaali_ms),
        "3" => (vokaali(480.0, 1350.0, 1700.0), vokaali_ms),
        // puolivokaalit, likvidat ja nasaalit
        "j" => (
            soinnillinen([260.0, 2100.0, 3000.0], [60.0, 120.0, 200.0], 0.7),
            50.0,
        ),
        "w" => (
            soinnillinen([290.0, 650.0, 2200.0], [60.0, 100.0, 200.0], 0.7),
            55.0,
        ),
        "l" => (
            soinnillinen([360.0, 1300.0, 2700.0], [80.0, 150.0, 200.0], 0.7),
            55.0,
        ),
        "r" => (
            soinnillinen([320.0, 1100.0, 1450.0], [80.0, 120.0, 150.0], 0.75),
            55.0,
        ),
        "R" => (
            Kohde {
                tarina: true,
                ..soinnillinen([450.0, 1350.0, 2400.0], [90.0, 150.0, 200.0], 0.8)
            },
            65.0,
        ),
        "m" => (
            soinnillinen([250.0, 1100.0, 2300.0], [100.0, 200.0, 300.0], 0.45),
            65.0,
        ),
        "n" => (
            soinnillinen([250.0, 1700.0, 2600.0], [100.0, 200.0, 300.0], 0.45),
            65.0,
        ),
        // frikatiivit
        "s" => (
            suhu([300.0, 1700.0, 2600.0], 0.0, 0.7, 5200.0, 2500.0),
            95.0,
        ),
        "z" => (
            suhu([260.0, 1700.0, 2600.0], 0.5, 0.35, 5000.0, 2500.0),
            75.0,
        ),
        "f" => (
            suhu([300.0, 1100.0, 2300.0], 0.0, 0.15, 5000.0, 6000.0),
            85.0,
        ),
        "v" => (
            suhu([300.0, 1200.0, 2300.0], 0.6, 0.06, 4000.0, 4000.0),
            55.0,
        ),
        // h saa formanttinsa seuraavalta äänteeltä fraasia koottaessa
        "h" => (
            Kohde {
                henkays: 0.5,
                aani: 0.0,
                ..vokaali(500.0, 1500.0, 2500.0)
            },
            60.0,
        ),
        _ => return None,
    })
}

/// Klusiilin sulkeuman formantit (kohta, josta seuraava vokaali liukuu), soinnillisuus
/// sekä purkauksen voima, taajuus, kaista ja kesto.
fn klusiili(merkki: &str) -> Option<([f32; 3], bool, f32, f32, f32, f32)> {
    Some(match merkki {
        "p" => ([250.0, 900.0, 2200.0], false, 0.3, 1200.0, 2000.0, 10.0),
        "t" => ([300.0, 1750.0, 2700.0], false, 0.8, 4200.0, 2500.0, 12.0),
        "k" => ([300.0, 2000.0, 2600.0], false, 0.7, 2300.0, 1200.0, 14.0),
        "g" => ([250.0, 2000.0, 2600.0], true, 0.6, 2300.0, 1200.0, 12.0),
        _ => return None,
    })
}

struct Jakso {
    kohde: Kohde,
    kesto: f32,
    korostus: bool,
}

/// Kokoaa fraasin äänteistä. Merkinnät: `:` pidentää, `!` korostaa ja `_` on lyhyt tauko.
fn jaksot(fraasi: &str) -> Vec<Jakso> {
    let mut jaksot: Vec<Jakso> = Vec::new();
    let hiljaa = Kohde {
        aani: 0.0,
        ..vokaali(500.0, 1500.0, 2500.0)
    };
    for merkki in fraasi.split_whitespace() {
        let pitka = merkki.contains(':');
        let korostus = merkki.contains('!');
        let merkki = merkki.trim_end_matches([':', '!']);
        let kerroin = if pitka { 1.8 } else { 1.0 };
        if merkki == "_" {
            // tauko pitää edellisen äänteen formantit, jottei mitään liu'u hiljaisuuteen
            let edellinen = jaksot.last().map_or(hiljaa, |j| j.kohde);
            jaksot.push(Jakso {
                kohde: Kohde {
                    aani: 0.0,
                    henkays: 0.0,
                    suhina: 0.0,
                    ..edellinen
                },
                kesto: 0.035,
                korostus: false,
            });
        } else if let Some((formantit, soi, voima, taajuus, kaista, purkaus_ms)) = klusiili(merkki)
        {
            let sulku = Kohde {
                formantit,
                aani: if soi { 0.2 } else { 0.0 },
                ..hiljaa
            };
            jaksot.push(Jakso {
                kohde: sulku,
                kesto: if soi { 0.045 } else { 0.055 } * kerroin,
                korostus: false,
            });
            jaksot.push(Jakso {
                kohde: Kohde {
                    suhina: voima,
                    suhinan_taajuus: taajuus,
                    suhinan_kaista: kaista,
                    aani: if soi { 0.3 } else { 0.0 },
                    ..sulku
                },
                kesto: purkaus_ms / 1000.0,
                korostus: false,
            });
        } else if let Some((kohde, kesto_ms)) = aanne(merkki) {
            jaksot.push(Jakso {
                kohde,
                kesto: kesto_ms / 1000.0 * kerroin,
                korostus,
            });
        } else {
            debug_assert!(false, "tuntematon äänne {merkki}");
        }
    }
    // h ääntyy seuraavan vokaalin suussa
    for i in 0..jaksot.len().saturating_sub(1) {
        if jaksot[i].kohde.henkays > 0.0 {
            jaksot[i].kohde.formantit = jaksot[i + 1].kohde.formantit;
        }
    }
    jaksot
}

/// Rosenbergin äänihuulipulssi yhden jakson vaiheessa 0..1.
fn aanihuulet(vaihe: f32) -> f32 {
    const AUKI: f32 = 0.4;
    const SULKEUTUU: f32 = 0.16;
    if vaihe < AUKI {
        0.5 * (1.0 - (std::f32::consts::PI * vaihe / AUKI).cos())
    } else if vaihe < AUKI + SULKEUTUU {
        (std::f32::consts::FRAC_PI_2 * (vaihe - AUKI) / SULKEUTUU).cos()
    } else {
        0.0
    }
}

fn liu_uta(arvo: &mut f32, kohde: f32, kerroin: f32) {
    *arvo += (kohde - *arvo) * kerroin;
}

/// Puhuu fraasin. `perussavel` on (alku, loppu) hertseinä: lause laskee loppua kohti.
pub fn puhu(fraasi: &str, perussavel: (f32, f32), arpa: &mut Arpa) -> Vec<f32> {
    let jaksot = jaksot(fraasi);
    let kokonais: f32 = jaksot.iter().map(|j| j.kesto).sum::<f32>() + 0.06;
    let naytteita = (kokonais * FS) as usize;

    // Formantit liukuvat 15 ms:n ja voimakkuudet 3 ms:n aikavakiolla.
    let formanttien_liuku = 1.0 - (-(PAIVITYSVALI as f32) / (0.015 * FS)).exp();
    let voiman_liuku = 1.0 - (-1.0 / (0.003 * FS)).exp();
    let savelen_liuku = 1.0 - (-1.0 / (0.04 * FS)).exp();

    let mut nyt = jaksot
        .first()
        .map_or(vokaali(500.0, 1500.0, 2500.0), |j| j.kohde);
    nyt.aani = 0.0;
    nyt.suhina = 0.0;
    nyt.henkays = 0.0;
    let mut vaye = [Resonaattori::default(); 4];
    vaye[3].aseta(3400.0, 250.0);
    // Kaksi suodinta peräkkäin, jotta suhina erottuu terävästi vokaaleista.
    let mut suhinasuotimet = [Kaista::default(); 2];
    let mut edellinen_vayla = 0.0;
    let mut vaihe = 0.0;
    let mut edellinen_pulssi = 0.0;
    let mut korostus = 0.0;
    let mut tasavirta = 0.0;

    let mut ulos = Vec::with_capacity(naytteita);
    let mut jakso = 0;
    let mut jakson_alku = 0.0;
    for n in 0..naytteita {
        let t = n as f32 / FS;
        while jakso < jaksot.len() && t >= jakson_alku + jaksot[jakso].kesto {
            jakson_alku += jaksot[jakso].kesto;
            jakso += 1;
        }
        let (kohde, korostettu) = match jaksot.get(jakso) {
            Some(j) => (j.kohde, j.korostus),
            None => (
                Kohde {
                    aani: 0.0,
                    suhina: 0.0,
                    henkays: 0.0,
                    ..nyt
                },
                false,
            ),
        };

        if n.is_multiple_of(PAIVITYSVALI) {
            for (i, resonaattori) in vaye.iter_mut().take(3).enumerate() {
                liu_uta(&mut nyt.formantit[i], kohde.formantit[i], formanttien_liuku);
                liu_uta(&mut nyt.kaistat[i], kohde.kaistat[i], formanttien_liuku);
                resonaattori.aseta(nyt.formantit[i], nyt.kaistat[i]);
            }
            nyt.suhinan_taajuus = kohde.suhinan_taajuus;
            nyt.suhinan_kaista = kohde.suhinan_kaista;
            for suodin in &mut suhinasuotimet {
                suodin.aseta(nyt.suhinan_taajuus, nyt.suhinan_kaista);
            }
        }
        liu_uta(&mut nyt.aani, kohde.aani, voiman_liuku);
        liu_uta(&mut nyt.henkays, kohde.henkays, voiman_liuku);
        liu_uta(&mut nyt.suhina, kohde.suhina, voiman_liuku);
        liu_uta(
            &mut korostus,
            if korostettu { 1.0 } else { 0.0 },
            savelen_liuku,
        );

        let osuus = t / kokonais;
        let savel = (perussavel.0 + (perussavel.1 - perussavel.0) * osuus) * (1.0 + 0.2 * korostus);
        vaihe += savel / FS;
        if vaihe >= 1.0 {
            vaihe -= 1.0;
        }
        let pulssi = aanihuulet(vaihe);
        // Derivaatta vastaa suun säteilyä ja kirkastaa äänen. Kerroin pitää voiman sävelestä riippumattomana.
        let lahde = (pulssi - edellinen_pulssi) * FS / savel * 0.25;
        edellinen_pulssi = pulssi;

        let mut aani = nyt.aani;
        if kohde.tarina {
            let paikka = t - jakson_alku;
            aani *= 0.35 + 0.65 * (0.5 + 0.5 * (std::f32::consts::TAU * 28.0 * paikka).cos());
        }
        let mut x = lahde * aani + kohina(arpa) * nyt.henkays;
        for resonaattori in &mut vaye {
            x = resonaattori.kasittele(x);
        }
        // Esikorostus nostaa ylempiä formantteja kuten kuulutuskaiuttimessa, mikä selkeyttää puhetta.
        let vayla = (x - 0.7 * edellinen_vayla) * 2.5;
        edellinen_vayla = x;
        let suhina = suhinasuotimet
            .iter_mut()
            .fold(kohina(arpa), |x, suodin| suodin.kasittele(x));
        let nayte = vayla + suhina * nyt.suhina * 3.0;
        // tasavirta pois
        tasavirta += (nayte - tasavirta) * 0.002;
        ulos.push(nayte - tasavirta);
    }

    // Puhe pidetään puhtaana: voimakkuus asetetaan huipun mukaan ilman ylivetoa.
    let huippu = ulos.iter().fold(0.0f32, |m, &x| m.max(x.abs())).max(1e-6);
    ulos.iter().map(|&x| x / huippu * 0.9).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kaikki_fraasien_aanteet_tunnetaan() {
        for (fraasi, _) in super::super::FRAASIT.iter().flatten() {
            for merkki in fraasi.split_whitespace() {
                let merkki = merkki.trim_end_matches([':', '!']);
                assert!(
                    merkki == "_" || aanne(merkki).is_some() || klusiili(merkki).is_some(),
                    "{merkki}"
                );
            }
        }
    }

    #[test]
    fn puhe_pysyy_rajoissa_ja_kestaa_kohtuullisesti() {
        for (fraasi, savel) in super::super::FRAASIT.iter().flatten() {
            let aani = puhu(fraasi, *savel, &mut Arpa::new(1));
            assert!(
                aani.iter().all(|x| x.is_finite() && x.abs() <= 1.0),
                "{fraasi}"
            );
            let kesto = aani.len() as f32 / FS;
            assert!((0.4..2.5).contains(&kesto), "{fraasi}: {kesto} s");
        }
    }
}
