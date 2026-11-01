//! Jokainen haaste pelataan läpi tallennetulla ratkaisulla oikeilla säännöillä. Ratkaisut ovat
//! vain täällä: ne eivät kuulu pelin mukana jaettavaan ohjelmaan.

use klassikot8000::haasteet::{HAASTEITA, SUURIN_SUMMA, klondike, pisteet, pyramidi, spider, taso};
use klassikot8000::kortit::{KUNINGAS, Kortti, MAITA};
use klassikot8000::pasianssi::{self, Kohde, Lahde, SUURIN_TULOS, Vaikeus, Variantti};
use klassikot8000::pyramidi::{Paikka, RIVEJA};
use klassikot8000::spider::{Jono, SARJOJA};

const RATKAISUT: &str = include_str!("haasteet_ratkaisut.txt");

fn ratkaisu(laji: &str, haaste: usize) -> Vec<&'static str> {
    RATKAISUT
        .lines()
        .filter_map(|rivi| rivi.strip_prefix(laji)?.strip_prefix(' '))
        .nth(haaste)
        .unwrap()
        .split(' ')
        .collect()
}

fn luku(teksti: &str) -> usize {
    teksti.parse().unwrap()
}

fn lahde(teksti: &str) -> Lahde {
    if teksti == "J" {
        Lahde::Jate
    } else if let Some(perusta) = teksti.strip_prefix('P') {
        Lahde::Perusta(luku(perusta))
    } else {
        let (sarake, kortti) = teksti[1..].split_once('.').unwrap();
        Lahde::Sarake {
            sarake: luku(sarake),
            kortti: luku(kortti),
        }
    }
}

fn kohde(teksti: &str) -> Kohde {
    match teksti.strip_prefix('P') {
        Some(perusta) => Kohde::Perusta(luku(perusta)),
        None => Kohde::Sarake(luku(&teksti[1..])),
    }
}

fn paikka(teksti: &str) -> Paikka {
    match teksti.strip_prefix('j') {
        Some(syvyys) => Paikka::Jate(luku(syvyys)),
        None => {
            let (rivi, sarake) = teksti.split_once('.').unwrap();
            Paikka::Pyramidi {
                rivi: luku(rivi),
                sarake: luku(sarake),
            }
        }
    }
}

#[test]
fn klondiken_haasteet_voi_voittaa() {
    for haaste in 0..HAASTEITA {
        let mut lauta = klondike(haaste).unwrap();
        assert_eq!(lauta.vaikeus, taso(haaste));
        assert!(!lauta.on_voitettu(), "{haaste}");
        for siirto in ratkaisu("klondike", haaste) {
            assert!(lauta.on_siirtoja(), "{haaste}: {siirto}");
            let tehty = match siirto.split_once('>') {
                None => lauta.nosta(),
                Some((mista, minne)) => lauta.siirra(lahde(mista), kohde(minne)),
            };
            assert!(tehty, "{haaste}: {siirto}");
        }
        assert!(lauta.on_voitettu(), "{haaste}");
    }
}

#[test]
fn spiderin_haasteet_voi_voittaa() {
    for haaste in 0..HAASTEITA {
        let mut lauta = spider(haaste).unwrap();
        assert_eq!(lauta.vaikeus, taso(haaste));
        assert!(!lauta.on_voitettu(), "{haaste}");
        for siirto in ratkaisu("spider", haaste) {
            assert!(lauta.on_siirtoja(), "{haaste}: {siirto}");
            let tehty = match siirto.split_once('>') {
                None => lauta.jaa(),
                Some((jono, minne)) => {
                    let (sarake, kortti) = jono.split_once('.').unwrap();
                    let jono = Jono {
                        sarake: luku(sarake),
                        kortti: luku(kortti),
                    };
                    lauta.siirra(jono, luku(minne))
                }
            };
            assert!(tehty, "{haaste}: {siirto}");
        }
        assert!(lauta.on_voitettu(), "{haaste}");
    }
}

#[test]
fn pyramidin_haasteet_voi_voittaa() {
    for haaste in 0..HAASTEITA {
        let mut lauta = pyramidi(haaste).unwrap();
        assert_eq!(lauta.vaikeus, taso(haaste));
        assert!(!lauta.on_voitettu(), "{haaste}");
        for siirto in ratkaisu("pyramidi", haaste) {
            assert!(lauta.on_siirtoja(), "{haaste}: {siirto}");
            let tehty = if let Some(kuningas) = siirto.strip_prefix("K:") {
                lauta.poista_kuningas(paikka(kuningas))
            } else if let Some(pari) = siirto.strip_prefix("P:") {
                let (a, b) = pari.split_once('+').unwrap();
                lauta.poista_pari(paikka(a), paikka(b))
            } else {
                lauta.nosta()
            };
            assert!(tehty, "{haaste}: {siirto}");
        }
        assert!(lauta.on_voitettu(), "{haaste}");
    }
}

/// Kortit laskettuina maittain ja arvoittain.
fn laskuri(kortit: impl IntoIterator<Item = Kortti>) -> [[u32; KUNINGAS as usize]; MAITA] {
    let mut maarat = [[0; KUNINGAS as usize]; MAITA];
    for kortti in kortit {
        maarat[kortti.maa as usize][kortti.arvo as usize - 1] += 1;
    }
    maarat
}

#[test]
fn klondike_ja_pyramidi_kayttavat_koko_pakkaa_ja_ovat_kesken() {
    let koko_pakka = [[1; KUNINGAS as usize]; MAITA];
    for haaste in 0..HAASTEITA {
        let tila = klondike(haaste).unwrap().tila;
        let kortit = tila
            .pakka
            .iter()
            .chain(&tila.jate)
            .chain(tila.perustat.iter().flatten())
            .chain(tila.sarakkeet.iter().flat_map(|s| &s.kortit))
            .copied();
        assert_eq!(laskuri(kortit), koko_pakka, "klondike {haaste}");
        assert!(tila.sarakkeet.iter().any(|s| s.piilossa > 0), "{haaste}");
        assert!(tila.perustat.iter().any(|p| !p.is_empty()), "{haaste}");

        let tila = pyramidi(haaste).unwrap().tila;
        let poistettuja = tila.rivit.iter().flatten().filter(|k| k.is_none()).count()
            - (RIVEJA * RIVEJA - RIVEJA * (RIVEJA + 1) / 2);
        assert!(poistettuja > 0, "pyramidi {haaste}");
    }
}

#[test]
fn spiderissa_on_oikeat_kortit_ja_peli_on_kesken() {
    for haaste in 0..HAASTEITA {
        let tila = spider(haaste).unwrap().tila;
        let poydalla = tila
            .pakka
            .iter()
            .chain(tila.sarakkeet.iter().flat_map(|s| &s.kortit))
            .count();
        let valmiissa = tila.valmiit.len() * KUNINGAS as usize;
        assert_eq!(poydalla + valmiissa, SARJOJA * KUNINGAS as usize);
        assert!(!tila.valmiit.is_empty(), "{haaste}");
        assert!(!tila.pakka.is_empty(), "{haaste}");
        assert!(tila.sarakkeet.iter().any(|s| s.piilossa > 0), "{haaste}");
    }
}

#[test]
fn haasteet_ovat_erilaisia_ja_tasot_vaikeutuvat() {
    for haaste in 0..HAASTEITA {
        let tila = klondike(haaste).unwrap().tila;
        for toinen in haaste + 1..HAASTEITA {
            assert_ne!(tila, klondike(toinen).unwrap().tila);
            assert_ne!(spider(haaste).unwrap().tila, spider(toinen).unwrap().tila);
            assert_ne!(
                pyramidi(haaste).unwrap().tila,
                pyramidi(toinen).unwrap().tila
            );
        }
    }
    assert_eq!(taso(0), Vaikeus::Helppo);
    assert_eq!(taso(HAASTEITA - 1), Vaikeus::Vaikea);
    assert!(klondike(HAASTEITA).is_none());
}

#[test]
fn peruutus_toimii_vaikeallakin_mutta_ei_ohi_alkutilanteen() {
    let mut lauta = klondike(HAASTEITA - 1).unwrap();
    let alku = lauta.tila.clone();
    assert_eq!(lauta.vaikeus, Vaikeus::Vaikea);
    assert!(!lauta.peruuta());
    assert!(lauta.nosta() || lauta.voi_kierrattaa());
    assert!(lauta.peruuta());
    assert_eq!(lauta.tila, alku);
    assert!(!lauta.peruuta());

    let mut lauta = pyramidi(HAASTEITA - 1).unwrap();
    let alku = lauta.tila.clone();
    assert!(lauta.nosta());
    assert!(lauta.peruuta());
    assert_eq!(lauta.tila, alku);

    // Tavallisessa vaikeassa pelissä sääntö ei muutu.
    let mut tavallinen = pasianssi::Lauta::new(Vaikeus::Vaikea, &mut klassikot8000::Arpa::new(1));
    assert!(tavallinen.nosta());
    assert!(!tavallinen.peruuta());
}

#[test]
fn suurin_summa_kattaa_kaikki_haasteet() {
    let summa: u32 = (0..HAASTEITA * Variantti::KAIKKI.len())
        .map(|i| pisteet(i % HAASTEITA, 0))
        .sum();
    assert!(summa <= SUURIN_SUMMA);
    assert!(pisteet(HAASTEITA - 1, 0) <= SUURIN_TULOS);
}
