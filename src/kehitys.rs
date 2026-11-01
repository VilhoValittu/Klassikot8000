//! Kehityslippujen tuki. Mukana vain debug-käännöksessä, ei julkaistavassa exessä.

use std::fs::File;
use std::io::{self, Write};

use klassikot8000::laivanupotus::{Ruutu, Vaikeus};
use klassikot8000::luvut::{Ruudut, SIVU, Suunta};

use crate::kaynnistin::{Laji, PELIT};
use crate::yhteinen::kieli::Kieli;

pub struct Liput {
    pub siemen: Option<u64>,
    pub kuvakaappaus: Option<String>,
    pub siirrot: Vec<Ruutu>,
    pub vaikeus: Option<Vaikeus>,
    /// Kieli koodina `fi` tai `en`. Menee tallennuksen asetuksen edelle.
    pub kieli: Option<Kieli>,
    /// Ohittaa käynnistimen. Kunkin pelin omat liput tekevät saman.
    peli: Option<Laji>,
    /// Käynnistimen valittu kortti nollasta alkaen.
    pub valittu: Option<usize>,
    /// 2048: luvut laudalla riveittäin, kuvakaappausta varten.
    pub lauta: Option<Ruudut>,
    /// 2048: siirrot kirjaimina U, D, L ja R.
    pub liu_ut: Vec<Suunta>,
    /// Tallennustiedosto. Ilman tätä kehitysajo ei lue eikä kirjoita tallennusta.
    pub tallennus: Option<String>,
    /// Nettilistan osoite. Ilman tätä kehitysajo ei ota yhteyttä verkkoon.
    pub netti: Option<String>,
    /// Julkaisujen osoite uuden version tarkistukseen. Tarvitsee rinnalleen `--net`-lipun.
    pub julkaisut: Option<String>,
    /// Näyttää ilmoituksen tästä versiosta ilman verkkoa.
    pub uusi_versio: Option<String>,
    /// Pasianssi: aloittaa pelin ja nostaa pakasta näin monta kertaa.
    pub nostot: Option<u32>,
    /// Pasianssi: aloittaa pelin voitettuna, loppujuhlan tarkistukseen.
    pub voitto: bool,
    /// Pasianssi, Blackjack ja Hold'em: näyttää nimikyselyn. Pasianssi tarvitsee rinnalleen `--win`-lipun.
    pub nimi: bool,
    /// Pasianssi: pelaa pelin tilanteeseen, jossa siirrot loppuvat.
    pub jumissa: bool,
    /// Tiilirikko: aloittaa tästä kentästä (nollasta alkaen) pallo lennossa.
    pub kentta: Option<usize>,
    /// Tiilirikko: pallo lähtee `--stage`-kentän alimman pallotiilen alta.
    pub toinen_pallo: bool,
    /// Tiilirikko: `--stage`-kentän pallo on superpallo.
    pub superpallo: bool,
    /// Asteroidit: aloittaa tästä aallosta (nollasta alkaen) ja lentää `aika_ms` paikallaan.
    pub aalto: Option<usize>,
    /// Silmukka: aloittaa valmiista renkaasta, jonka sisällä on näin monta itikkaa.
    pub rengas: Option<i32>,
    /// Palikka ja miinaharava: aloittaa pelin ja pelaa näin monta siirtoa valmiiksi.
    pub pelatut: Option<u32>,
    /// Miinaharava: aloittaa pelin ja astuu miinaan.
    pub pamaus: bool,
    /// Blackjack ja Hold'em: aloittaa pelin ja antaa nämä näppäimet: kirjaimet ja numerot
    /// sellaisinaan, välilyönti jakaa tai jatkaa. Koneet ehtivät pelata joka näppäimen jälkeen.
    pub napit: Option<String>,
    /// Blackjack: aloittaa pelin ja asettaa pöydälle näin monta kättä.
    pub kadet: Option<usize>,
    /// Blackjack ja Hold'em: aloittaa pelin, jossa rahat ovat juuri loppuneet.
    pub konkurssi: bool,
    /// Kuvakaappauksen hetki viimeisen laukauksen jälkeen, animaatioiden tarkistukseen.
    pub aika_ms: u32,
}

impl Liput {
    pub fn laji(&self) -> Option<Laji> {
        if self.peli.is_some() {
            self.peli
        } else if self.vaikeus.is_some() || !self.siirrot.is_empty() {
            Some(Laji::Laivanupotus)
        } else if self.lauta.is_some() || !self.liu_ut.is_empty() {
            Some(Laji::Luvut)
        } else if self.nostot.is_some() || self.voitto || self.jumissa {
            Some(Laji::Pasianssi)
        } else if self.kentta.is_some() {
            Some(Laji::Tiilirikko)
        } else if self.aalto.is_some() {
            Some(Laji::Asteroidit)
        } else if self.rengas.is_some() {
            Some(Laji::Silmukka)
        } else if self.napit.is_some() || self.kadet.is_some() || self.konkurssi || self.nimi {
            Some(Laji::Blackjack)
        } else {
            None
        }
    }
}

// Riittää siihen, että kaikki tehosteet ovat päättyneet.
const OLETUSAIKA_MS: u32 = 3000;

pub fn lue_liput() -> Result<Liput, String> {
    let mut liput = Liput {
        siemen: None,
        kuvakaappaus: None,
        siirrot: Vec::new(),
        vaikeus: None,
        kieli: None,
        peli: None,
        valittu: None,
        lauta: None,
        liu_ut: Vec::new(),
        tallennus: None,
        netti: None,
        julkaisut: None,
        uusi_versio: None,
        nostot: None,
        voitto: false,
        nimi: false,
        jumissa: false,
        kentta: None,
        toinen_pallo: false,
        superpallo: false,
        aalto: None,
        rengas: None,
        pelatut: None,
        pamaus: false,
        napit: None,
        kadet: None,
        konkurssi: false,
        aika_ms: OLETUSAIKA_MS,
    };
    let mut args = std::env::args().skip(1);
    while let Some(lippu) = args.next() {
        let arvo = args.next().ok_or(format!("{lippu}: arvo puuttuu"))?;
        match lippu.as_str() {
            "--lang" => {
                liput.kieli = Some(Kieli::koodista(&arvo).ok_or(format!("--lang: {arvo}"))?);
            }
            "--seed" => liput.siemen = Some(arvo.parse().map_err(|_| "--seed: ei luku")?),
            "--screenshot" => liput.kuvakaappaus = Some(arvo),
            "--save" => liput.tallennus = Some(arvo),
            "--net" => liput.netti = Some(arvo),
            "--releases" => liput.julkaisut = Some(arvo),
            "--new-version" => liput.uusi_versio = Some(arvo),
            "--time" => liput.aika_ms = arvo.parse().map_err(|_| "--time: ei luku")?,
            "--game" => {
                liput.peli = Some(match arvo.as_str() {
                    "laivanupotus" => Laji::Laivanupotus,
                    "2048" => Laji::Luvut,
                    "pasianssi" => Laji::Pasianssi,
                    "tiilirikko" => Laji::Tiilirikko,
                    "asteroidit" => Laji::Asteroidit,
                    "silmukka" => Laji::Silmukka,
                    "palikka" => Laji::Palikka,
                    "miinaharava" => Laji::Miinaharava,
                    "blackjack" => Laji::Blackjack,
                    "holdem" => Laji::Holdem,
                    _ => return Err(format!("--game: {arvo}")),
                });
            }
            "--select" => {
                let i = arvo.parse().map_err(|_| "--select: ei luku")?;
                if i >= PELIT.len() {
                    return Err(format!("--select: {arvo}"));
                }
                liput.valittu = Some(i);
            }
            "--draws" => liput.nostot = Some(arvo.parse().map_err(|_| "--draws: ei luku")?),
            "--win" => liput.voitto = arvo == "1",
            "--name" => liput.nimi = arvo == "1",
            "--stuck" => liput.jumissa = arvo == "1",
            "--stage" => liput.kentta = Some(arvo.parse().map_err(|_| "--stage: ei luku")?),
            "--second-ball" => liput.toinen_pallo = arvo == "1",
            "--super-ball" => liput.superpallo = arvo == "1",
            "--wave" => liput.aalto = Some(arvo.parse().map_err(|_| "--wave: ei luku")?),
            "--ring" => liput.rengas = Some(arvo.parse().map_err(|_| "--ring: ei luku")?),
            "--plays" => liput.pelatut = Some(arvo.parse().map_err(|_| "--plays: ei luku")?),
            "--boom" => liput.pamaus = arvo == "1",
            "--keys" => liput.napit = Some(arvo),
            "--hands" => liput.kadet = Some(arvo.parse().map_err(|_| "--hands: ei luku")?),
            "--broke" => liput.konkurssi = arvo == "1",
            "--board" => {
                let luvut = arvo
                    .split(',')
                    .map(|osa| osa.parse::<u32>())
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| format!("--board: {arvo}"))?;
                if luvut.len() != SIVU * SIVU {
                    return Err(format!("--board: tarvitaan {} lukua", SIVU * SIVU));
                }
                let mut ruudut = [[0; SIVU]; SIVU];
                for (i, luku) in luvut.into_iter().enumerate() {
                    ruudut[i / SIVU][i % SIVU] = luku;
                }
                liput.lauta = Some(ruudut);
            }
            "--slides" => {
                for kirjain in arvo.chars() {
                    liput.liu_ut.push(match kirjain {
                        'U' => Suunta::Ylos,
                        'D' => Suunta::Alas,
                        'L' => Suunta::Vasen,
                        'R' => Suunta::Oikea,
                        _ => return Err(format!("--slides: {kirjain}")),
                    });
                }
            }
            "--level" => {
                let taso = Vaikeus::KAIKKI
                    .into_iter()
                    .find(|taso| taso.nimi().eq_ignore_ascii_case(&arvo));
                liput.vaikeus = Some(taso.ok_or(format!("--level: {arvo}"))?);
            }
            "--moves" => {
                for osa in arvo.split(',') {
                    let ruutu = osa.parse().map_err(|_| format!("--moves: {osa}"))?;
                    liput.siirrot.push(ruutu);
                }
            }
            _ => return Err(format!("tuntematon lippu {lippu}")),
        }
    }
    Ok(liput)
}

/// Tallentaa kuvan pakkaamattomana PNG:nä, jotta pakkaaja ei tarvitse riippuvuutta.
pub fn tallenna_png(polku: &str, kuva: &[u32], leveys: usize, korkeus: usize) -> io::Result<()> {
    let mut raaka = Vec::with_capacity((leveys * 3 + 1) * korkeus);
    for rivi in kuva.chunks(leveys) {
        raaka.push(0);
        for &pikseli in rivi {
            raaka.extend_from_slice(&pikseli.to_be_bytes()[1..]);
        }
    }

    let mut zlib = vec![0x78, 0x01];
    let mut palat = raaka.chunks(0xFFFF).peekable();
    while let Some(pala) = palat.next() {
        zlib.push(palat.peek().is_none() as u8);
        zlib.extend_from_slice(&(pala.len() as u16).to_le_bytes());
        zlib.extend_from_slice(&(!(pala.len() as u16)).to_le_bytes());
        zlib.extend_from_slice(pala);
    }
    zlib.extend_from_slice(&adler32(&raaka).to_be_bytes());

    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&(leveys as u32).to_be_bytes());
    ihdr.extend_from_slice(&(korkeus as u32).to_be_bytes());
    ihdr.extend_from_slice(&[8, 2, 0, 0, 0]);

    let mut tiedosto = File::create(polku)?;
    tiedosto.write_all(b"\x89PNG\r\n\x1a\n")?;
    kirjoita_pala(&mut tiedosto, b"IHDR", &ihdr)?;
    kirjoita_pala(&mut tiedosto, b"IDAT", &zlib)?;
    kirjoita_pala(&mut tiedosto, b"IEND", &[])
}

fn kirjoita_pala(ulos: &mut impl Write, tyyppi: &[u8; 4], data: &[u8]) -> io::Result<()> {
    ulos.write_all(&(data.len() as u32).to_be_bytes())?;
    ulos.write_all(tyyppi)?;
    ulos.write_all(data)?;
    ulos.write_all(&crc32(tyyppi, data).to_be_bytes())
}

fn crc32(tyyppi: &[u8], data: &[u8]) -> u32 {
    let mut crc = !0u32;
    for &tavu in tyyppi.iter().chain(data) {
        crc ^= tavu as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                0xEDB8_8320 ^ (crc >> 1)
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for &tavu in data {
        a = (a + tavu as u32) % 65521;
        b = (b + a) % 65521;
    }
    b << 16 | a
}
