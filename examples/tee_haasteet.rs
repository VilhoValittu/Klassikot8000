//! Luo pasianssien haasteet ja niiden ratkaisut: `cargo run --release --example tee_haasteet`.
//! Klondike ja Pyramidi ratkaistaan oikeasta jaosta, ja haaste on ratkaisun puolivälin tilanne.
//! Spiderin neljää maata ei tällainen haku ratkaise, joten se pelataan voitosta taaksepäin.
//! Jokainen ratkaisu pelataan lopuksi läpi oikeilla säännöillä, ja `tests/haasteet.rs` tekee saman.

use std::collections::HashSet;
use std::collections::hash_map::DefaultHasher;
use std::fmt::Write as _;
use std::hash::{Hash, Hasher};

use klassikot8000::Arpa;
use klassikot8000::haasteet::{HAASTEITA, taso};
use klassikot8000::kortit::{KUNINGAS, Kortti, MAITA, Maa};
use klassikot8000::pasianssi::{self, Kohde, Lahde, SARAKKEITA, Sarake, Vaikeus};
use klassikot8000::pyramidi::{self, JATTEEN_NAKYVAT, Paikka, RIVEJA};
use klassikot8000::spider::{self, Jono};

const HAUN_RAJA: usize = 400_000;
const SUMMA: u8 = 13;
const KLONDIKEN_PIILOKORTIT: usize = 21;
const KLONDIKEN_TYO: usize = pasianssi::KORTTEJA + KLONDIKEN_PIILOKORTIT;

fn kortti_teksti(kortti: Kortti) -> String {
    let arvo = "A23456789TJQK".as_bytes()[kortti.arvo as usize - 1] as char;
    format!("{arvo}{}", maa_teksti(kortti.maa))
}

fn maa_teksti(maa: Maa) -> char {
    "HRIP".as_bytes()[maa as usize] as char
}

fn kortit_teksti(kortit: &[Kortti]) -> String {
    if kortit.is_empty() {
        return "-".to_string();
    }
    kortit.iter().map(|&k| kortti_teksti(k)).collect()
}

fn sarake_teksti(sarake: &Sarake) -> String {
    format!("{}:{}", sarake.piilossa, kortit_teksti(&sarake.kortit))
}

fn sarakkeet_teksti(sarakkeet: &[Sarake]) -> String {
    let osat: Vec<String> = sarakkeet.iter().map(sarake_teksti).collect();
    osat.join(" ")
}

fn sekoita_kortit(hasher: &mut DefaultHasher, kortit: &[Kortti]) {
    kortit.len().hash(hasher);
    for kortti in kortit {
        (kortti.maa as u8, kortti.arvo).hash(hasher);
    }
}

/// Kuinka pitkälle ratkaisua pelataan ennen kuin pelaaja liittyy: helppo haaste on melkein
/// valmis, ja tason sisällä pelattavaa jää koko ajan enemmän.
fn osuus_tasolla(haaste: usize) -> f32 {
    let (alku, maara) = tason_alue(haaste);
    0.60 - 0.35 * (haaste - alku) as f32 / (maara - 1) as f32
}

fn tason_alue(haaste: usize) -> (usize, usize) {
    let alku = (0..haaste)
        .rev()
        .find(|&i| taso(i) != taso(haaste))
        .map_or(0, |i| i + 1);
    let maara = (alku..HAASTEITA)
        .take_while(|&i| taso(i) == taso(haaste))
        .count();
    (alku, maara)
}

/// Syvyyshaku, jonka siirrot tulevat `siirrot`-funktiolta tärkein viimeisenä.
fn etsi<T: Clone, S: Copy>(
    alku: T,
    siirrot: impl Fn(&T) -> Vec<S>,
    tee: impl Fn(&T, S) -> Option<T>,
    voitettu: impl Fn(&T) -> bool,
    tunniste: impl Fn(&T) -> u64,
) -> Option<Vec<S>> {
    let mut nahdyt = HashSet::new();
    nahdyt.insert(tunniste(&alku));
    let ensimmaiset = siirrot(&alku);
    let mut pino: Vec<(T, Vec<S>, Option<S>)> = vec![(alku, ensimmaiset, None)];
    let mut solmuja = 0;
    while let Some(paalla) = pino.last_mut() {
        let Some(siirto) = paalla.1.pop() else {
            pino.pop();
            continue;
        };
        let Some(uusi) = tee(&paalla.0, siirto) else {
            continue;
        };
        if !nahdyt.insert(tunniste(&uusi)) {
            continue;
        }
        solmuja += 1;
        if solmuja > HAUN_RAJA {
            return None;
        }
        if voitettu(&uusi) {
            let mut polku: Vec<S> = pino.iter().filter_map(|taso| taso.2).collect();
            polku.push(siirto);
            return Some(polku);
        }
        let seuraavat = siirrot(&uusi);
        pino.push((uusi, seuraavat, Some(siirto)));
    }
    None
}

// --- Klondike ---

#[derive(Clone, Copy)]
enum KSiirto {
    Nosto,
    Siirto(Lahde, Kohde),
}

fn klondike_siirrot(lauta: &pasianssi::Lauta) -> Vec<KSiirto> {
    let tila = &lauta.tila;
    let mut siirrot = Vec::new();
    // Tärkein viimeisenä: haku ottaa siirrot pinon päältä.
    for perusta in 0..MAITA {
        for sarake in 0..SARAKKEITA {
            let lahde = Lahde::Perusta(perusta);
            if lauta.voi_siirtaa(lahde, Kohde::Sarake(sarake)) {
                siirrot.push(KSiirto::Siirto(lahde, Kohde::Sarake(sarake)));
            }
        }
    }
    for sarake in 0..SARAKKEITA {
        let tiedot = &tila.sarakkeet[sarake];
        for kortti in tiedot.piilossa + 1..tiedot.kortit.len() {
            let lahde = Lahde::Sarake { sarake, kortti };
            for kohde in 0..SARAKKEITA {
                if lauta.voi_siirtaa(lahde, Kohde::Sarake(kohde)) {
                    siirrot.push(KSiirto::Siirto(lahde, Kohde::Sarake(kohde)));
                }
            }
        }
    }
    if !tila.pakka.is_empty() || lauta.voi_kierrattaa() {
        siirrot.push(KSiirto::Nosto);
    }
    for kohde in 0..SARAKKEITA {
        if lauta.voi_siirtaa(Lahde::Jate, Kohde::Sarake(kohde)) {
            siirrot.push(KSiirto::Siirto(Lahde::Jate, Kohde::Sarake(kohde)));
        }
    }
    for sarake in 0..SARAKKEITA {
        let tiedot = &tila.sarakkeet[sarake];
        if tiedot.kortit.is_empty() {
            continue;
        }
        let lahde = Lahde::Sarake {
            sarake,
            kortti: tiedot.piilossa,
        };
        for kohde in 0..SARAKKEITA {
            let turha = tiedot.piilossa == 0 && tila.sarakkeet[kohde].kortit.is_empty();
            if !turha && lauta.voi_siirtaa(lahde, Kohde::Sarake(kohde)) {
                siirrot.push(KSiirto::Siirto(lahde, Kohde::Sarake(kohde)));
            }
        }
    }
    let mut lahteet = vec![Lahde::Jate];
    for sarake in 0..SARAKKEITA {
        if let Some(kortti) = tila.sarakkeet[sarake].kortit.len().checked_sub(1) {
            lahteet.push(Lahde::Sarake { sarake, kortti });
        }
    }
    for lahde in lahteet {
        for perusta in 0..MAITA {
            if lauta.voi_siirtaa(lahde, Kohde::Perusta(perusta)) {
                siirrot.push(KSiirto::Siirto(lahde, Kohde::Perusta(perusta)));
            }
        }
    }
    siirrot
}

fn klondike_tee(lauta: &mut pasianssi::Lauta, siirto: KSiirto) -> bool {
    match siirto {
        KSiirto::Nosto => lauta.nosta(),
        KSiirto::Siirto(lahde, kohde) => lauta.siirra(lahde, kohde),
    }
}

fn klondike_ratkaisu(vaikeus: Vaikeus, tila: pasianssi::Tila) -> Option<Vec<KSiirto>> {
    let rajattu = vaikeus.kierroksia().is_some();
    etsi(
        tila,
        |t: &pasianssi::Tila| klondike_siirrot(&pasianssi::Lauta::kesken(vaikeus, t.clone())),
        |t, siirto| {
            let mut lauta = pasianssi::Lauta::kesken(vaikeus, t.clone());
            klondike_tee(&mut lauta, siirto).then_some(lauta.tila)
        },
        |t| t.perustat.iter().all(|p| p.len() == KUNINGAS as usize),
        |t| {
            let mut h = DefaultHasher::new();
            sekoita_kortit(&mut h, &t.pakka);
            sekoita_kortit(&mut h, &t.jate);
            for pino in &t.perustat {
                pino.len().hash(&mut h);
            }
            for sarake in &t.sarakkeet {
                sarake.piilossa.hash(&mut h);
                sekoita_kortit(&mut h, &sarake.kortit);
            }
            if rajattu {
                t.kierratykset.hash(&mut h);
            }
            h.finish()
        },
    )
}

fn klondike_siirto_teksti(siirto: KSiirto) -> String {
    match siirto {
        KSiirto::Nosto => "N".to_string(),
        KSiirto::Siirto(lahde, kohde) => {
            let lahde = match lahde {
                Lahde::Jate => "J".to_string(),
                Lahde::Perusta(p) => format!("P{p}"),
                Lahde::Sarake { sarake, kortti } => format!("S{sarake}.{kortti}"),
            };
            let kohde = match kohde {
                Kohde::Perusta(p) => format!("P{p}"),
                Kohde::Sarake(s) => format!("S{s}"),
            };
            format!("{lahde}>{kohde}")
        }
    }
}

/// Montako korttia on perustoilla ja montako piilokorttia on käännetty.
fn klondiken_edistys(tila: &pasianssi::Tila) -> usize {
    let piilossa: usize = tila.sarakkeet.iter().map(|s| s.piilossa).sum();
    let perustoilla: usize = tila.perustat.iter().map(Vec::len).sum();
    perustoilla + KLONDIKEN_PIILOKORTIT - piilossa
}

fn klondike_haaste(haaste: usize, siemen: &mut u64) -> (String, String) {
    let vaikeus = taso(haaste);
    loop {
        *siemen += 1;
        let alku = pasianssi::Lauta::new(vaikeus, &mut Arpa::new(*siemen)).tila;
        let Some(polku) = klondike_ratkaisu(vaikeus, alku.clone()) else {
            continue;
        };
        // Suuri osa siirroista on nostoja, joten katkaisukohta luetaan pelin etenemisestä.
        let tavoite = (KLONDIKEN_TYO as f32 * osuus_tasolla(haaste)) as usize;
        let mut lauta = pasianssi::Lauta::kesken(vaikeus, alku);
        let mut katkaisu = 0;
        while klondiken_edistys(&lauta.tila) < tavoite {
            assert!(klondike_tee(&mut lauta, polku[katkaisu]));
            katkaisu += 1;
        }
        let piilossa: usize = lauta.tila.sarakkeet.iter().map(|s| s.piilossa).sum();
        let perustoilla: usize = lauta.tila.perustat.iter().map(Vec::len).sum();
        // Kesken jääneen näköinen: osa korteista on perustoilla, ja työtä on jäljellä.
        if piilossa == 0 || perustoilla == 0 || perustoilla > 40 {
            continue;
        }
        let tila = &lauta.tila;
        let perustat: Vec<String> = tila.perustat.iter().map(|p| p.len().to_string()).collect();
        let rivi = format!(
            "klondike {} {} {} {} {}",
            tila.kierratykset,
            kortit_teksti(&tila.pakka),
            kortit_teksti(&tila.jate),
            perustat.join(","),
            sarakkeet_teksti(&tila.sarakkeet),
        );
        let ratkaisu: Vec<String> = polku[katkaisu..]
            .iter()
            .map(|&s| klondike_siirto_teksti(s))
            .collect();
        return (rivi, format!("klondike {}", ratkaisu.join(" ")));
    }
}

// --- Pyramidi ---

#[derive(Clone, Copy)]
enum PSiirto {
    Nosto,
    Kuningas(Paikka),
    Pari(Paikka, Paikka),
}

fn pyramidi_siirrot(lauta: &pyramidi::Lauta) -> Vec<PSiirto> {
    let mut paikat: Vec<Paikka> = Vec::new();
    for syvyys in 0..JATTEEN_NAKYVAT {
        paikat.push(Paikka::Jate(syvyys));
    }
    for rivi in 0..RIVEJA {
        for sarake in 0..=rivi {
            paikat.push(Paikka::Pyramidi { rivi, sarake });
        }
    }
    let pelattavat: Vec<(Paikka, Kortti)> = paikat
        .into_iter()
        .filter_map(|p| lauta.pelattava(p).map(|k| (p, k)))
        .collect();
    let mut siirrot = Vec::new();
    if !lauta.tila.pakka.is_empty() || lauta.voi_kierrattaa() {
        siirrot.push(PSiirto::Nosto);
    }
    for (i, &(a, kortti_a)) in pelattavat.iter().enumerate() {
        if kortti_a.arvo == KUNINGAS {
            siirrot.push(PSiirto::Kuningas(a));
        }
        for &(b, kortti_b) in &pelattavat[i + 1..] {
            if kortti_a.arvo + kortti_b.arvo == SUMMA {
                siirrot.push(PSiirto::Pari(a, b));
            }
        }
    }
    siirrot
}

fn pyramidi_tee(lauta: &mut pyramidi::Lauta, siirto: PSiirto) -> bool {
    match siirto {
        PSiirto::Nosto => lauta.nosta(),
        PSiirto::Kuningas(paikka) => lauta.poista_kuningas(paikka),
        PSiirto::Pari(a, b) => lauta.poista_pari(a, b),
    }
}

fn pyramidi_ratkaisu(vaikeus: Vaikeus, tila: pyramidi::Tila) -> Option<Vec<PSiirto>> {
    let rajattu = pyramidi::kierroksia(vaikeus).is_some();
    etsi(
        tila,
        |t: &pyramidi::Tila| pyramidi_siirrot(&pyramidi::Lauta::kesken(vaikeus, t.clone())),
        |t, siirto| {
            let mut lauta = pyramidi::Lauta::kesken(vaikeus, t.clone());
            pyramidi_tee(&mut lauta, siirto).then_some(lauta.tila)
        },
        |t| t.rivit.iter().flatten().all(Option::is_none),
        |t| {
            let mut h = DefaultHasher::new();
            sekoita_kortit(&mut h, &t.pakka);
            sekoita_kortit(&mut h, &t.jate);
            for kortti in t.rivit.iter().flatten() {
                kortti.map(|k| (k.maa as u8, k.arvo)).hash(&mut h);
            }
            if rajattu {
                t.kierratykset.hash(&mut h);
            }
            h.finish()
        },
    )
}

fn paikka_teksti(paikka: Paikka) -> String {
    match paikka {
        Paikka::Pyramidi { rivi, sarake } => format!("{rivi}.{sarake}"),
        Paikka::Jate(syvyys) => format!("j{syvyys}"),
    }
}

fn pyramidi_siirto_teksti(siirto: PSiirto) -> String {
    match siirto {
        PSiirto::Nosto => "N".to_string(),
        PSiirto::Kuningas(paikka) => format!("K:{}", paikka_teksti(paikka)),
        PSiirto::Pari(a, b) => format!("P:{}+{}", paikka_teksti(a), paikka_teksti(b)),
    }
}

fn pyramidi_haaste(haaste: usize, siemen: &mut u64) -> (String, String) {
    let vaikeus = taso(haaste);
    loop {
        *siemen += 1;
        let alku = pyramidi::Lauta::new(vaikeus, &mut Arpa::new(*siemen)).tila;
        let Some(polku) = pyramidi_ratkaisu(vaikeus, alku.clone()) else {
            continue;
        };
        let katkaisu = (polku.len() as f32 * osuus_tasolla(haaste)) as usize;
        let mut lauta = pyramidi::Lauta::kesken(vaikeus, alku);
        for &siirto in &polku[..katkaisu] {
            assert!(pyramidi_tee(&mut lauta, siirto));
        }
        let tila = &lauta.tila;
        let jaljella = tila.rivit.iter().flatten().flatten().count();
        if !(6..=24).contains(&jaljella) {
            continue;
        }
        let mut paikat = Vec::new();
        for (r, rivi) in tila.rivit.iter().enumerate() {
            for paikka in &rivi[..=r] {
                paikat.push(paikka.map_or("-".to_string(), kortti_teksti));
            }
        }
        let rivi = format!(
            "pyramidi {} {} {} {}",
            tila.kierratykset,
            kortit_teksti(&tila.pakka),
            kortit_teksti(&tila.jate),
            paikat.join(","),
        );
        let ratkaisu: Vec<String> = polku[katkaisu..]
            .iter()
            .map(|&s| pyramidi_siirto_teksti(s))
            .collect();
        return (rivi, format!("pyramidi {}", ratkaisu.join(" ")));
    }
}

// --- Spider ---

#[derive(Clone, Copy)]
enum SSiirto {
    Jako,
    Siirto(Jono, usize),
}

const SARJAN_PITUUS: usize = KUNINGAS as usize;
// Piilokortteja kertyy muuten yhteen sarakkeeseen, joka ei näyttäisi oikealta peliltä.
const PIILOSSA_ENINTAAN: usize = 8;
const KAANTEISIA_ASKELIA: usize = 1500;
const ALKUSEKOITUS: usize = 20;

struct SpiderKohde {
    valmiit: usize,
    jakoja: usize,
    piilossa: usize,
    askelia: usize,
}

fn spider_kohde(haaste: usize) -> SpiderKohde {
    let (alku, maara) = tason_alue(haaste);
    let osuus = (haaste - alku) as f32 / (maara - 1) as f32;
    // Yhdellä maalla jokainen purettu sarja vie kaksi saraketta, joten kaikkea ei voi purkaa.
    let vahiten_valmiita = match taso(haaste) {
        Vaikeus::Helppo => 3,
        Vaikeus::Normaali => 2,
        Vaikeus::Vaikea => 1,
    };
    let valmiit = 5 - ((5 - vahiten_valmiita) as f32 * osuus).round() as usize;
    let jakoja = 1 + (3.0 * osuus).round() as usize;
    let poydalla = spider::KORTTEJA - valmiit * SARJAN_PITUUS - jakoja * spider::SARAKKEITA;
    SpiderKohde {
        valmiit,
        jakoja,
        // Osan korteista on jäätävä näkyviin, muuten tilanne näyttäisi tuoreelta jaolta.
        piilossa: (poydalla as f32 * (0.30 + 0.20 * osuus)) as usize,
        askelia: 60 + (120.0 * osuus).round() as usize,
    }
}

fn hanta_on_sarja(sarake: &Sarake) -> bool {
    let Some(alku) = sarake.kortit.len().checked_sub(SARJAN_PITUUS) else {
        return false;
    };
    let sarja = &sarake.kortit[alku..];
    alku >= sarake.piilossa
        && sarja[0].arvo == KUNINGAS
        && sarja
            .windows(2)
            .all(|p| p[1].maa == p[0].maa && p[1].arvo + 1 == p[0].arvo)
}

/// Asettaa sarakkeen päälle kortteja. Yksin näkyvä kortti voi kääntyä nurin, koska eteenpäin
/// pelattaessa se kääntyy esiin, kun päältä lähtevät viimeiset kortit.
fn lisaa_sarakkeeseen(sarake: &mut Sarake, kortit: &[Kortti], piilota: bool) {
    let yksin = !sarake.kortit.is_empty() && sarake.piilossa + 1 == sarake.kortit.len();
    if piilota && yksin && sarake.piilossa < PIILOSSA_ENINTAAN {
        sarake.piilossa += 1;
    }
    sarake.kortit.extend_from_slice(kortit);
}

/// Sarakkeen hännän jonot, joita voi siirtää taaksepäin: niiden alla on tyhjä tai yhtä suurempi
/// näkyvä kortti, joten eteenpäin pelattaessa ne voivat laskeutua tähän. Lyhin ensin.
fn palautettavat(sarake: &Sarake) -> Vec<usize> {
    let mut alut = Vec::new();
    for alku in (sarake.piilossa..sarake.kortit.len()).rev() {
        let kortit = &sarake.kortit[alku..];
        if !kortit
            .windows(2)
            .all(|p| p[1].maa == p[0].maa && p[1].arvo + 1 == p[0].arvo)
        {
            break;
        }
        let pohjalla = alku == 0;
        let alla_sopiva =
            alku > sarake.piilossa && sarake.kortit[alku - 1].arvo == kortit[0].arvo + 1;
        if pohjalla || alla_sopiva {
            alut.push(alku);
        }
    }
    alut
}

fn vie_jono(
    tila: &mut spider::Tila,
    siirrot: &mut Vec<SSiirto>,
    mista: usize,
    alku: usize,
    minne: usize,
    piilota: bool,
) {
    let jono = tila.sarakkeet[mista].kortit.split_off(alku);
    let vanha = tila.sarakkeet[minne].clone();
    lisaa_sarakkeeseen(&mut tila.sarakkeet[minne], &jono, piilota);
    if hanta_on_sarja(&tila.sarakkeet[minne]) {
        tila.sarakkeet[minne] = vanha;
        tila.sarakkeet[mista].kortit.extend(jono);
        return;
    }
    siirrot.push(SSiirto::Siirto(
        Jono {
            sarake: minne,
            kortti: vanha.kortit.len(),
        },
        mista,
    ));
}

/// Purkaa yhden valmiin sarjan kahteen jonoon, jotka eteenpäin pelattaessa yhdistyvät ja poistuvat.
fn pura_sarja(tila: &mut spider::Tila, siirrot: &mut Vec<SSiirto>, arpa: &mut Arpa) -> bool {
    let tyhjat: Vec<usize> = (0..spider::SARAKKEITA)
        .filter(|&i| tila.sarakkeet[i].kortit.is_empty())
        .collect();
    if tyhjat.is_empty() {
        return false;
    }
    let a = tyhjat[arpa.alle(tyhjat.len())];
    // Loppuosa lasketaan yhtä suuremman päälle, jotta tyhjät sarakkeet riittävät kuninkaille.
    let sopivat: Vec<usize> = (0..spider::SARAKKEITA)
        .filter(|&i| i != a && tila.sarakkeet[i].kortit.last().is_some_and(|k| k.arvo >= 2))
        .collect();
    let (b, raja) = if sopivat.is_empty() || (tyhjat.len() > 3 && arpa.alle(2) == 0) {
        let Some(&b) = tyhjat.iter().find(|&&i| i != a) else {
            return false;
        };
        (b, 2 + arpa.alle(KUNINGAS as usize - 1) as u8)
    } else {
        let b = sopivat[arpa.alle(sopivat.len())];
        (
            b,
            tila.sarakkeet[b].kortit[tila.sarakkeet[b].kortit.len() - 1].arvo,
        )
    };
    let Some(maa) = tila.valmiit.pop() else {
        return false;
    };
    let yla: Vec<Kortti> = (raja..=KUNINGAS)
        .rev()
        .map(|arvo| Kortti { maa, arvo })
        .collect();
    let ala: Vec<Kortti> = (1..raja).rev().map(|arvo| Kortti { maa, arvo }).collect();
    let kortti = tila.sarakkeet[b].kortit.len();
    let vanha = tila.sarakkeet[b].clone();
    lisaa_sarakkeeseen(&mut tila.sarakkeet[b], &ala, false);
    // Samaa maata oleva yläosa kokoaisi sarjan jo väärässä kohdassa.
    if hanta_on_sarja(&tila.sarakkeet[b]) {
        tila.sarakkeet[b] = vanha;
        tila.valmiit.push(maa);
        sekoita_jono(tila, siirrot, arpa);
        return true;
    }
    lisaa_sarakkeeseen(&mut tila.sarakkeet[a], &yla, false);
    siirrot.push(SSiirto::Siirto(Jono { sarake: b, kortti }, a));
    true
}

/// Siirtää satunnaisen jonon tyhjään sarakkeeseen tai yhtä suuremman päälle, jolloin se pysyy liikuteltavana.
fn sekoita_jono(tila: &mut spider::Tila, siirrot: &mut Vec<SSiirto>, arpa: &mut Arpa) {
    let mista = arpa.alle(spider::SARAKKEITA);
    let alut = palautettavat(&tila.sarakkeet[mista]);
    if alut.is_empty() {
        return;
    }
    let alku = alut[arpa.alle(alut.len())];
    let arvo = tila.sarakkeet[mista].kortit[alku].arvo;
    let kohteet: Vec<usize> = (0..spider::SARAKKEITA)
        .filter(|&i| {
            i != mista
                && tila.sarakkeet[i]
                    .kortit
                    .last()
                    .is_none_or(|k| k.arvo == arvo + 1)
        })
        .collect();
    if !kohteet.is_empty() {
        let minne = kohteet[arpa.alle(kohteet.len())];
        vie_jono(tila, siirrot, mista, alku, minne, false);
    }
}

/// Täyttää liian lyhyen sarakkeen pidemmän hännästä niin, että molempiin jää `tarve` korttia.
fn levita(tila: &mut spider::Tila, siirrot: &mut Vec<SSiirto>, tarve: usize) -> bool {
    let Some(minne) = (0..spider::SARAKKEITA).find(|&i| tila.sarakkeet[i].kortit.len() < tarve)
    else {
        return false;
    };
    let vajaus = tarve - tila.sarakkeet[minne].kortit.len();
    let paalla = tila.sarakkeet[minne].kortit.last().map(|k| k.arvo);
    for mista in (0..spider::SARAKKEITA).filter(|&i| i != minne) {
        let kortit = &tila.sarakkeet[mista].kortit;
        let sopiva = palautettavat(&tila.sarakkeet[mista])
            .into_iter()
            .find(|&alku| {
                alku >= tarve
                    && kortit.len() - alku >= vajaus
                    && paalla.is_none_or(|arvo| arvo == kortit[alku].arvo + 1)
            });
        if let Some(alku) = sopiva {
            vie_jono(tila, siirrot, mista, alku, minne, false);
            return true;
        }
    }
    false
}

/// Peittää yksin näkyvän kortin toisen sarakkeen hännällä. Palauttaa, löytyikö sellainen.
fn piilota_kortti(tila: &mut spider::Tila, siirrot: &mut Vec<SSiirto>, arpa: &mut Arpa) -> bool {
    let kohteet: Vec<usize> = (0..spider::SARAKKEITA)
        .filter(|&i| {
            let s = &tila.sarakkeet[i];
            !s.kortit.is_empty()
                && s.piilossa + 1 == s.kortit.len()
                && s.piilossa < PIILOSSA_ENINTAAN
        })
        .collect();
    let lahteet: Vec<usize> = (0..spider::SARAKKEITA)
        .filter(|&i| !palautettavat(&tila.sarakkeet[i]).is_empty())
        .collect();
    if lahteet.is_empty() {
        return false;
    }
    if kohteet.is_empty() {
        // Yksin näkyvä kortti syntyy lyhentämällä lyhintä saraketta.
        let lyhin = lahteet
            .iter()
            .copied()
            .min_by_key(|&i| tila.sarakkeet[i].kortit.len());
        if let Some(mista) = lyhin {
            let alut = palautettavat(&tila.sarakkeet[mista]);
            let alku = alut[alut.len() - 1];
            let minne = (mista + 1 + arpa.alle(spider::SARAKKEITA - 1)) % spider::SARAKKEITA;
            vie_jono(tila, siirrot, mista, alku, minne, false);
        }
        return true;
    }
    let minne = kohteet[arpa.alle(kohteet.len())];
    let mista = lahteet[arpa.alle(lahteet.len())];
    if minne == mista {
        return true;
    }
    let alku = palautettavat(&tila.sarakkeet[mista])[0];
    vie_jono(tila, siirrot, mista, alku, minne, true);
    true
}

fn spider_kaanteinen(haaste: usize, arpa: &mut Arpa) -> Option<(spider::Tila, Vec<SSiirto>)> {
    let kohde = spider_kohde(haaste);
    let maat = spider::maat(taso(haaste));
    let mut valmiit: Vec<Maa> = (0..spider::SARJOJA).map(|i| maat[i % maat.len()]).collect();
    for i in (1..valmiit.len()).rev() {
        valmiit.swap(i, arpa.alle(i + 1));
    }
    let mut tila = spider::Tila {
        pakka: Vec::new(),
        sarakkeet: Default::default(),
        valmiit,
        siirrot: 0,
    };
    let mut siirrot = Vec::new();
    for _ in 0..KAANTEISIA_ASKELIA {
        if tila.valmiit.len() > kohde.valmiit {
            if !pura_sarja(&mut tila, &mut siirrot, arpa) {
                return None;
            }
            continue;
        }
        let jakoja = tila.pakka.len() / spider::SARAKKEITA;
        if jakoja < kohde.jakoja {
            // Jokaisessa sarakkeessa on oltava jokaisen jaon jälkeen näkyvä kortti.
            let tarve = kohde.jakoja - jakoja + 1;
            if siirrot.len() < ALKUSEKOITUS {
                sekoita_jono(&mut tila, &mut siirrot, arpa);
            } else if tila.sarakkeet.iter().all(|s| s.kortit.len() >= tarve) {
                // Eteenpäin jako alkaa ensimmäisestä sarakkeesta, joten sen kortti tulee pakan päälle.
                for sarake in (0..spider::SARAKKEITA).rev() {
                    if let Some(kortti) = tila.sarakkeet[sarake].kortit.pop() {
                        tila.pakka.push(kortti);
                    }
                }
                siirrot.push(SSiirto::Jako);
            } else if !levita(&mut tila, &mut siirrot, tarve) {
                sekoita_jono(&mut tila, &mut siirrot, arpa);
            }
            continue;
        }
        if siirrot.len() < kohde.askelia {
            sekoita_jono(&mut tila, &mut siirrot, arpa);
            continue;
        }
        let piilossa: usize = tila.sarakkeet.iter().map(|s| s.piilossa).sum();
        if piilossa >= kohde.piilossa {
            return Some((tila, siirrot));
        }
        if !piilota_kortti(&mut tila, &mut siirrot, arpa) {
            return None;
        }
    }
    None
}

fn spider_haaste(haaste: usize, siemen: &mut u64) -> (String, String) {
    let vaikeus = taso(haaste);
    let mut yrityksia = 0;
    loop {
        *siemen += 1;
        yrityksia += 1;
        assert!(yrityksia < 30_000, "spider {haaste}: ei löydy");
        let Some((tila, mut siirrot)) = spider_kaanteinen(haaste, &mut Arpa::new(*siemen)) else {
            continue;
        };
        siirrot.reverse();
        let mut lauta = spider::Lauta::kesken(vaikeus, tila.clone());
        let kelpaa = siirrot.iter().all(|&siirto| match siirto {
            SSiirto::Jako => lauta.jaa(),
            SSiirto::Siirto(jono, kohde) => lauta.siirra(jono, kohde),
        });
        if !kelpaa || !lauta.on_voitettu() {
            continue;
        }
        let valmiit: String = tila.valmiit.iter().map(|&m| maa_teksti(m)).collect();
        let rivi = format!(
            "spider {} {} {}",
            kortit_teksti(&tila.pakka),
            if valmiit.is_empty() { "-" } else { &valmiit },
            sarakkeet_teksti(&tila.sarakkeet),
        );
        let ratkaisu: Vec<String> = siirrot
            .iter()
            .map(|&siirto| match siirto {
                SSiirto::Jako => "N".to_string(),
                SSiirto::Siirto(jono, kohde) => format!("{}.{}>{kohde}", jono.sarake, jono.kortti),
            })
            .collect();
        return (rivi, format!("spider {}", ratkaisu.join(" ")));
    }
}

type Luoja = fn(usize, &mut u64) -> (String, String);

fn main() {
    let (mut data, mut ratkaisut) = (String::new(), String::new());
    let lajit: [(&str, Luoja); 3] = [
        ("klondike", klondike_haaste),
        ("spider", spider_haaste),
        ("pyramidi", pyramidi_haaste),
    ];
    for (nimi, luo) in lajit {
        let mut siemen = 1000;
        for haaste in 0..HAASTEITA {
            let (rivi, ratkaisu) = luo(haaste, &mut siemen);
            let siirtoja = ratkaisu.split(' ').count() - 1;
            println!("{nimi} {haaste}: siemen {siemen}, ratkaisussa {siirtoja} siirtoa");
            writeln!(data, "{rivi}").unwrap();
            writeln!(ratkaisut, "{ratkaisu}").unwrap();
        }
    }
    std::fs::write("src/pelit/pasianssi/haasteet.txt", data).unwrap();
    std::fs::write("tests/haasteet_ratkaisut.txt", ratkaisut).unwrap();
}
