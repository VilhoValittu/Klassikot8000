//! Kehityslippujen tuki. Mukana vain debug-käännöksessä, ei julkaistavassa exessä.

use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use klassikot8000::Arpa;
use klassikot8000::asteroidit::Avaruus;
use klassikot8000::blackjack::{self, Vuoro};
use klassikot8000::jatsi::{self, Toimi, koneen_toimi};
use klassikot8000::kaivo::Kaivos;
use klassikot8000::laivanupotus::{Ruutu, Vaikeus};
use klassikot8000::luvut::{Lukulauta, Ruudut, SIVU, Suunta};
use klassikot8000::mahjong;
use klassikot8000::majakka::{Laiva, MAJAKKA, Meri, etaisyys, suunta_majakalta};
use klassikot8000::miinaharava;
use klassikot8000::pasianssi::{self, KUNINGAS, Kortti, Maa, Variantti};
use klassikot8000::silmukka::{self, Otus};
use klassikot8000::tiilirikko::{self, Tiilipeli};

use crate::kaynnistin::netti::{Netti, Pyynto, Vastaus, tulosavain};
use crate::kaynnistin::{Kaynnissa, Kaynnistin, Laji, PELIT, piirto, tallennus};
use crate::pelit::asteroidit::peli::Asteroidit;
use crate::pelit::blackjack::peli::Blackjack;
use crate::pelit::holdem::peli::{self as holdem_peli, Holdem};
use crate::pelit::jatsi::peli::{self as jatsi_peli, HEITTO_MS, Jatsi, KONE_ASKEL_MS};
use crate::pelit::kaivo::peli::Kaivo;
use crate::pelit::laivanupotus::peli::{Peli, Vaihe};
use crate::pelit::luvut::peli::{self as luvut_peli, Luvut};
use crate::pelit::mahjong::peli::Mahjong;
use crate::pelit::majakka::peli::Majakka;
use crate::pelit::majakka::piirto::Merikuva;
use crate::pelit::miinaharava::peli::Miinaharava;
use crate::pelit::palikka::peli::{self as palikka_peli, Palikkapeli};
use crate::pelit::pasianssi::peli::{self as pasianssi_peli, Pasianssi};
use crate::pelit::silmukka::peli::Silmukka;
use crate::pelit::tiilirikko::peli::Tiilirikko;
use crate::yhteinen::aanet::{self, Mikseri, NAYTETAAJUUS};
use crate::yhteinen::kieli::{self, Kieli};
use crate::yhteinen::osoitin;
use crate::yhteinen::piirto::{Kehys, RUUDUN_KORKEUS, RUUDUN_LEVEYS};
use crate::yhteinen::syote::Syote;

#[derive(Default)]
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
    /// Pasianssi: laji, ilman tätä Klondike.
    pub variantti: Option<Variantti>,
    /// Pasianssi: avaa haastelistan.
    pub haasteet: bool,
    /// Pasianssi: aloittaa tämän haasteen nollasta alkaen.
    pub haaste: Option<usize>,
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
    /// Majakka: aloittaa tästä yöstä (nollasta alkaen), ja keila seuraa `aika_ms` ajan lähintä laivaa.
    pub yo: Option<u32>,
    /// Kaivo: aloittaa tältä tasolta (nollasta alkaen) ja painaa `--keys`-näppäimet: `^v<>`
    /// kaivavat ja kulkevat, välilyönti pystyttää tuen ja R heittää köyden.
    pub kaivos: Option<u32>,
    /// Palikka ja miinaharava: aloittaa pelin ja pelaa näin monta siirtoa valmiiksi.
    pub pelatut: Option<u32>,
    /// Miinaharava: aloittaa pelin ja astuu miinaan.
    pub pamaus: bool,
    /// Blackjack ja Hold'em: aloittaa pelin ja antaa nämä näppäimet: kirjaimet ja numerot
    /// sellaisinaan, välilyönti jakaa tai jatkaa. Koneet ehtivät pelata joka näppäimen jälkeen.
    /// Laivanupotus (`--game laivanupotus`): näppäimet alkuruudulta alkaen, `^v<>` ovat nuolet.
    pub napit: Option<String>,
    /// Blackjack: aloittaa pelin ja asettaa pöydälle näin monta kättä.
    pub kadet: Option<usize>,
    /// Blackjack ja Hold'em: aloittaa pelin, jossa rahat ovat juuri loppuneet.
    pub konkurssi: bool,
    /// Toimintapelit ja Palikka: painaa lopuksi Esc, joten kuvassa on tauko. Valikon
    /// kuvakaappauksessa Esc kysyy lopetusta.
    pub tauko: bool,
    /// Kuvakaappauksen hetki viimeisen laukauksen jälkeen, animaatioiden tarkistukseen.
    pub aika_ms: u32,
    /// Kirjoittaa kaikki äänet tähän kansioon WAV-tiedostoina kuunneltaviksi ja lopettaa.
    pub aanet: Option<String>,
    /// Hiiren paikka näkymän koordinaateissa kuvakaappausta varten: napit korostuvat.
    pub hiiri: Option<(i32, i32)>,
    /// Hiiren nappi on pohjassa `--mouse`-kohdassa, ilman napin toimintoa.
    pub painettu: bool,
}

impl Liput {
    pub fn laji(&self) -> Option<Laji> {
        if self.peli.is_some() {
            self.peli
        } else if self.vaikeus.is_some() || !self.siirrot.is_empty() {
            Some(Laji::Laivanupotus)
        } else if self.lauta.is_some() || !self.liu_ut.is_empty() {
            Some(Laji::Luvut)
        } else if self.nostot.is_some()
            || self.voitto
            || self.jumissa
            || self.variantti.is_some()
            || self.haasteet
            || self.haaste.is_some()
        {
            Some(Laji::Pasianssi)
        } else if self.kentta.is_some() {
            Some(Laji::Tiilirikko)
        } else if self.aalto.is_some() {
            Some(Laji::Asteroidit)
        } else if self.rengas.is_some() {
            Some(Laji::Silmukka)
        } else if self.yo.is_some() {
            Some(Laji::Majakka)
        } else if self.kaivos.is_some() {
            Some(Laji::Kaivo)
        } else if self.napit.is_some() || self.kadet.is_some() || self.konkurssi || self.nimi {
            Some(Laji::Blackjack)
        } else {
            None
        }
    }
}

// Riittää siihen, että kaikki tehosteet ovat päättyneet.
const OLETUSAIKA_MS: u32 = 3000;

/// Lipun arvo lukuna.
fn luku<T: std::str::FromStr>(lippu: &str, arvo: &str) -> Result<T, String> {
    arvo.parse().map_err(|_| format!("{lippu}: ei luku"))
}

pub fn lue_liput() -> Result<Liput, String> {
    let mut liput = Liput {
        aika_ms: OLETUSAIKA_MS,
        ..Liput::default()
    };
    let mut args = std::env::args().skip(1);
    while let Some(lippu) = args.next() {
        let arvo = args.next().ok_or(format!("{lippu}: arvo puuttuu"))?;
        match lippu.as_str() {
            "--lang" => {
                liput.kieli = Some(Kieli::koodista(&arvo).ok_or(format!("--lang: {arvo}"))?);
            }
            "--seed" => liput.siemen = Some(luku(&lippu, &arvo)?),
            "--screenshot" => liput.kuvakaappaus = Some(arvo),
            "--sounds" => liput.aanet = Some(arvo),
            "--save" => liput.tallennus = Some(arvo),
            "--net" => liput.netti = Some(arvo),
            "--releases" => liput.julkaisut = Some(arvo),
            "--new-version" => liput.uusi_versio = Some(arvo),
            "--time" => liput.aika_ms = luku(&lippu, &arvo)?,
            "--mouse" => {
                let (x, y) = arvo.split_once(',').ok_or("--mouse: x,y")?;
                liput.hiiri = Some((luku(&lippu, x)?, luku(&lippu, y)?));
            }
            "--press" => liput.painettu = arvo == "1",
            "--game" => {
                liput.peli = Some(match arvo.as_str() {
                    "laivanupotus" => Laji::Laivanupotus,
                    "2048" => Laji::Luvut,
                    "pasianssi" => Laji::Pasianssi,
                    "mahjong" => Laji::Mahjong,
                    "tiilirikko" => Laji::Tiilirikko,
                    "asteroidit" => Laji::Asteroidit,
                    "silmukka" => Laji::Silmukka,
                    "palikka" => Laji::Palikka,
                    "miinaharava" => Laji::Miinaharava,
                    "blackjack" => Laji::Blackjack,
                    "holdem" => Laji::Holdem,
                    "majakka" => Laji::Majakka,
                    "kaivo" => Laji::Kaivo,
                    "jatsi" => Laji::Jatsi,
                    _ => return Err(format!("--game: {arvo}")),
                });
            }
            "--select" => {
                let i = luku(&lippu, &arvo)?;
                if i >= PELIT.len() {
                    return Err(format!("--select: {arvo}"));
                }
                liput.valittu = Some(i);
            }
            "--variant" => {
                liput.variantti = Some(match arvo.as_str() {
                    "klondike" => Variantti::Klondike,
                    "spider" => Variantti::Spider,
                    "pyramidi" => Variantti::Pyramidi,
                    _ => return Err(format!("--variant: {arvo}")),
                });
            }
            "--challenges" => liput.haasteet = arvo == "1",
            "--challenge" => {
                liput.haaste = Some(luku(&lippu, &arvo)?);
            }
            "--draws" => liput.nostot = Some(luku(&lippu, &arvo)?),
            "--win" => liput.voitto = arvo == "1",
            "--name" => liput.nimi = arvo == "1",
            "--stuck" => liput.jumissa = arvo == "1",
            "--stage" => liput.kentta = Some(luku(&lippu, &arvo)?),
            "--second-ball" => liput.toinen_pallo = arvo == "1",
            "--super-ball" => liput.superpallo = arvo == "1",
            "--wave" => liput.aalto = Some(luku(&lippu, &arvo)?),
            "--ring" => liput.rengas = Some(luku(&lippu, &arvo)?),
            "--night" => liput.yo = Some(luku(&lippu, &arvo)?),
            "--mine" => liput.kaivos = Some(luku(&lippu, &arvo)?),
            "--plays" => liput.pelatut = Some(luku(&lippu, &arvo)?),
            "--boom" => liput.pamaus = arvo == "1",
            "--keys" => liput.napit = Some(arvo),
            "--hands" => liput.kadet = Some(luku(&lippu, &arvo)?),
            "--broke" => liput.konkurssi = arvo == "1",
            "--pause" => liput.tauko = arvo == "1",
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

/// Kirjoittaa jokaisen äänen omaksi WAV-tiedostokseen, puheet kummallakin kielellä.
/// Kehitysajon alku: peli ikkunaan, tai lippujen pyytämä työ on jo tehty.
pub enum Alku {
    Ikkuna(Box<Kaynnistin>, Option<PathBuf>, Option<Netti>),
    Valmis(ExitCode),
}

fn lopetus(tulos: io::Result<()>, kohde: &str) -> Alku {
    Alku::Valmis(match tulos {
        Ok(()) => ExitCode::SUCCESS,
        Err(virhe) => {
            eprintln!("{kohde}: {virhe}");
            ExitCode::FAILURE
        }
    })
}

/// Lukee liput ja asettaa käynnistimen ja pelin niiden mukaiseen tilaan. Äänet ja
/// kuvakaappaus tallennetaan heti, eikä ikkunaa silloin avata.
pub fn aloita(kehys: &mut Kehys) -> Alku {
    let liput = match lue_liput() {
        Ok(liput) => liput,
        Err(virhe) => {
            eprintln!("{virhe}");
            return Alku::Valmis(ExitCode::FAILURE);
        }
    };
    if let Some(kansio) = &liput.aanet {
        return lopetus(tallenna_aanet(kansio), kansio);
    }
    let mut kaynnistin = Kaynnistin::new(liput.siemen.unwrap_or_else(crate::aikasiemen));
    // Kehitysajo ei koske pelaajan tallennukseen ilman lippua.
    let polku = liput.tallennus.as_ref().map(PathBuf::from);
    if let Some(polku) = &polku {
        match tallennus::lue_tiedosto(polku) {
            Ok(tallennus) => kaynnistin.lataa(tallennus),
            Err(virhe) => return lopetus(Err(virhe), &polku.display().to_string()),
        }
    }
    if let Some(kieli) = liput.kieli {
        kieli::aseta(kieli);
    }
    if let Some(i) = liput.valittu {
        kaynnistin.valitse(i);
    }
    if let Some(laji) = liput.laji() {
        kaynnistin.kaynnista(laji);
    }
    if let Some(peli) = &mut kaynnistin.peli {
        valmistele(peli, &liput);
        if liput.tauko {
            peli.nakyma_mut().kasittele(Syote::Peru);
        }
    }
    kaynnistin.ota_aanet();
    if let Some(versio) = &liput.uusi_versio {
        kaynnistin.vastaanota(Vastaus::UusiVersio(versio.clone()));
    }
    if let Some(polku) = &liput.kuvakaappaus {
        // Renkaan tehosteet on ajettu jo, eikä niitä saa hypätä yli.
        if liput.rengas.is_none() {
            kaynnistin.edista(liput.aika_ms);
        }
        // Valikon lopetuskysymys raukeaa ajan kuluessa, joten Esc painetaan vasta nyt.
        if liput.tauko && kaynnistin.peli.is_none() {
            kaynnistin.kasittele(Syote::Peru);
        }
        if let Some((x, y)) = liput.hiiri {
            kaynnistin.kasittele(Syote::Hiiri(x, y));
            if liput.painettu {
                // Vain hiiren tila: klikkaus laukaisisi napin.
                osoitin::kirjaa(Syote::Klikkaus(x, y));
            }
        }
        piirto::piirra(&kaynnistin, kehys);
        let tulos = tallenna_png(polku, &kehys.ruutu, RUUDUN_LEVEYS, RUUDUN_KORKEUS);
        return lopetus(tulos, polku);
    }
    // Kehitysajo ei koske oikeaan nettilistaan ilman lippua.
    let netti = liput
        .netti
        .as_deref()
        .and_then(|osoite| Netti::kaynnista(osoite, tulosavain()));
    if let (Some(netti), Some(julkaisut)) = (&netti, &liput.julkaisut) {
        netti.laheta(Pyynto::HaeVersio(julkaisut.clone()));
    }
    Alku::Ikkuna(Box::new(kaynnistin), polku, netti)
}

/// Vie juuri käynnistetyn pelin lippujen kuvaamaan tilanteeseen.
fn valmistele(peli: &mut Kaynnissa, liput: &Liput) {
    match peli {
        Kaynnissa::Laivanupotus(peli) => valmistele_laivanupotus(peli, liput),
        Kaynnissa::Luvut(peli) => valmistele_luvut(peli, liput),
        Kaynnissa::Pasianssi(peli) => valmistele_pasianssi(peli, liput),
        Kaynnissa::Tiilirikko(peli) => valmistele_tiilirikko(peli, liput),
        Kaynnissa::Asteroidit(peli) => valmistele_asteroidit(peli, liput),
        Kaynnissa::Silmukka(peli) => valmistele_silmukka(peli, liput),
        Kaynnissa::Palikka(peli) => valmistele_palikka(peli, liput),
        Kaynnissa::Miinaharava(peli) => valmistele_miinaharava(peli, liput),
        Kaynnissa::Mahjong(peli) => valmistele_mahjong(peli, liput),
        Kaynnissa::Blackjack(peli) => valmistele_blackjack(peli, liput),
        Kaynnissa::Holdem(peli) => valmistele_holdem(peli, liput),
        Kaynnissa::Majakka(peli) => valmistele_majakka(peli, liput),
        Kaynnissa::Kaivo(peli) => valmistele_kaivo(peli, liput),
        Kaynnissa::Jatsi(peli) => valmistele_jatsi(peli, liput),
    }
}

fn valmistele_laivanupotus(peli: &mut Peli, liput: &Liput) {
    if let Some(vaikeus) = liput.vaikeus {
        peli.vaikeus = vaikeus;
    }
    // Alkuruutu ja arvottujen laivojen hyväksyntä.
    if !liput.siirrot.is_empty() {
        peli.kasittele(Syote::Ammu);
        peli.kasittele(Syote::Ammu);
    }
    // Välilyönti, numerot ja kirjaimet sellaisinaan, ^v<> ovat nuolet.
    for merkki in liput.napit.as_deref().unwrap_or_default().chars() {
        peli.kasittele(match merkki {
            ' ' => Syote::Ammu,
            '^' => Syote::Nuoli(-1, 0),
            'v' => Syote::Nuoli(1, 0),
            '<' => Syote::Nuoli(0, -1),
            '>' => Syote::Nuoli(0, 1),
            '0'..='9' => Syote::Numero(merkki as u8 - b'0'),
            _ => Syote::Kirjain(merkki),
        });
    }
    for &ruutu in &liput.siirrot {
        if peli.vaihe == Vaihe::Ammunta {
            peli.ammu(ruutu);
        }
    }
}

fn valmistele_luvut(peli: &mut Luvut, liput: &Liput) {
    if liput.lauta.is_some() || !liput.liu_ut.is_empty() {
        peli.kasittele(Syote::Ammu);
    }
    if let Some(ruudut) = liput.lauta {
        peli.lauta = Lukulauta::ruuduista(ruudut);
    }
    for &suunta in &liput.liu_ut {
        if peli.vaihe == luvut_peli::Vaihe::Pelissa {
            peli.siirra(suunta);
        }
    }
}

fn valmistele_pasianssi(peli: &mut Pasianssi, liput: &Liput) {
    if let Some(vaikeus) = liput.vaikeus {
        let taso = pasianssi::Vaikeus::KAIKKI
            .into_iter()
            .find(|taso| taso.nimi() == vaikeus.nimi());
        peli.vaikeus = taso.unwrap_or(peli.vaikeus);
    }
    if let Some(variantti) = liput.variantti {
        peli.variantti = variantti;
    }
    if liput.haasteet {
        peli.kasittele(Syote::Kirjain('H'));
    }
    if let Some(haaste) = liput.haaste {
        peli.aloita_haaste(haaste);
    } else if liput.nostot.is_some() || liput.voitto || liput.jumissa {
        peli.kasittele(Syote::Ammu);
    }
    for _ in 0..liput.nostot.unwrap_or(0) {
        peli.kasittele(Syote::Kirjain('N'));
    }
    if liput.jumissa && peli.variantti == Variantti::Klondike {
        // Viimeinen siirto kerää ässän, ja peli toteaa lopun itse.
        let tila = &mut peli.lauta.tila;
        tila.pakka.clear();
        tila.jate.clear();
        tila.sarakkeet = Default::default();
        tila.sarakkeet[0].kortit = vec![Kortti {
            maa: Maa::Pata,
            arvo: 9,
        }];
        tila.sarakkeet[1].kortit = vec![
            Kortti {
                maa: Maa::Risti,
                arvo: 5,
            },
            Kortti {
                maa: Maa::Hertta,
                arvo: 1,
            },
        ];
        peli.kasittele(Syote::Kirjain('A'));
    }
    if liput.voitto {
        for (pino, &maa) in peli.lauta.tila.perustat.iter_mut().zip(&Maa::KAIKKI) {
            *pino = (1..=KUNINGAS).map(|arvo| Kortti { maa, arvo }).collect();
        }
        peli.lauta.tila.sarakkeet = Default::default();
        peli.lauta.tila.pakka.clear();
        peli.lauta.tila.jate.clear();
        peli.spider.tila.valmiit = vec![Maa::Pata; klassikot8000::spider::SARJOJA];
        peli.spider.tila.sarakkeet = Default::default();
        peli.spider.tila.pakka.clear();
        peli.pyramidi.tila.rivit = Default::default();
        peli.vaihe = pasianssi_peli::Vaihe::Voitto;
        if liput.nimi {
            peli.vaihe = pasianssi_peli::Vaihe::Nimi;
        }
    }
}

fn valmistele_tiilirikko(peli: &mut Tiilirikko, liput: &Liput) {
    if let Some(kentta) = liput.kentta {
        peli.kasittele(Syote::Ammu);
        peli.lauta = Tiilipeli::new(kentta, &mut Arpa::new(liput.siemen.unwrap_or(1)));
        peli.kasittele(Syote::Ammu);
        if liput.toinen_pallo {
            let lauta = &mut peli.lauta;
            let rivi = (0..tiilirikko::RIVEJA)
                .rfind(|&r| lauta.pallotiilet[r].contains(&true))
                .unwrap_or(0);
            let sarake = (0..tiilirikko::SARAKKEITA)
                .find(|&s| lauta.pallotiilet[rivi][s])
                .unwrap_or(0);
            // Alemmat tiilet pois, ettei pallo osu niihin ensin.
            for alempi in &mut lauta.tiilet[rivi + 1..] {
                alempi[sarake] = 0;
            }
            lauta.pallot[0].paikka = (
                (sarake as f32 + 0.5) * tiilirikko::TIILI_L,
                tiilirikko::TIILET_Y
                    + (rivi + 1) as f32 * tiilirikko::TIILI_K
                    + tiilirikko::PALLON_SADE
                    + 0.5,
            );
        }
        if liput.superpallo {
            let pallo = &mut peli.lauta.pallot[0];
            pallo.superpallo = true;
            pallo.nopeus.0 *= tiilirikko::SUPERVAUHTI;
            pallo.nopeus.1 *= tiilirikko::SUPERVAUHTI;
        }
        // Aikahyppy on rajattu, joten kuvakaappauksen hetki ajetaan pieninä askelina.
        for _ in 0..liput.aika_ms / 16 {
            peli.edista(16);
            peli.kasittele(Syote::Ammu);
        }
    }
}

fn valmistele_asteroidit(peli: &mut Asteroidit, liput: &Liput) {
    if let Some(aalto) = liput.aalto {
        peli.kasittele(Syote::Ammu);
        peli.lauta = Avaruus::new(aalto, &mut Arpa::new(liput.siemen.unwrap_or(1)));
        // Aikahyppy on rajattu, joten kuvakaappauksen hetki ajetaan pieninä askelina.
        for _ in 0..liput.aika_ms / 16 {
            peli.edista(16);
        }
    }
}

fn valmistele_silmukka(peli: &mut Silmukka, liput: &Liput) {
    if let Some(itikoita) = liput.rengas {
        peli.kasittele(Syote::Ammu);
        // Pää sulkee renkaan kolmen askeleen päästä, ja häntä jää sen ulkopuolelle.
        let mut ruudut: Vec<(i32, i32)> = (1..=18).map(|x| (x, 4)).collect();
        ruudut.extend((5..=16).map(|y| (18, y)));
        ruudut.extend((5..=17).rev().map(|x| (x, 16)));
        ruudut.extend((7..=15).rev().map(|y| (5, y)));
        peli.lauta.jalki = ruudut[0];
        peli.lauta.mato = ruudut.into_iter().rev().collect();
        peli.lauta.suunta = silmukka::Suunta::Ylos;
        let mato = peli.lauta.mato.clone();
        peli.lauta.herkut.retain(|h| !mato.contains(&h.paikka));
        peli.lauta.otukset = (0..itikoita)
            .map(|i| {
                Otus::uusi(
                    silmukka::Laji::Itikka,
                    (7 + i * 2, 7 + i % 3 * 3),
                    (1, 1),
                    0,
                )
            })
            .collect();
        // Aikahyppy on rajattu, joten kuvakaappauksen hetki ajetaan pieninä askelina.
        for _ in 0..liput.aika_ms / 16 {
            peli.edista(16);
        }
    }
}

fn valmistele_majakka(peli: &mut Majakka, liput: &Liput) {
    let Some(yo) = liput.yo else {
        return;
    };
    peli.kasittele(Syote::Ammu);
    peli.meri = Meri::new(peli.vaikeus, yo, &mut Arpa::new(liput.siemen.unwrap_or(1)));
    peli.kuva = Some(Merikuva::new(&peli.meri));
    // Keila seuraa majakkaa lähintä laivaa, jotta kuvassa on valaistu laiva.
    for _ in 0..liput.aika_ms / 16 {
        let lahin = peli
            .meri
            .laivat
            .iter()
            .min_by(|a, b| {
                let d = |l: &&Laiva| etaisyys(l.paikka, MAJAKKA);
                d(a).total_cmp(&d(b))
            })
            .map(|laiva| suunta_majakalta(laiva.paikka));
        peli.ohjaus.tahtain = lahin;
        peli.edista(16);
    }
}

fn valmistele_kaivo(peli: &mut Kaivo, liput: &Liput) {
    let Some(taso) = liput.kaivos else {
        return;
    };
    peli.kasittele(Syote::Ammu);
    peli.kaivos = Kaivos::new(
        peli.vaikeus,
        taso,
        &mut Arpa::new(liput.siemen.unwrap_or(1)),
    );
    peli.paivita_kuva();
    // Joka näppäimelle aikaa valmistua: kaivu, kävely tai putoaminen.
    for merkki in liput.napit.as_deref().unwrap_or_default().chars() {
        peli.kasittele(match merkki {
            ' ' => Syote::Ammu,
            '^' => Syote::Nuoli(-1, 0),
            'v' => Syote::Nuoli(1, 0),
            '<' => Syote::Nuoli(0, -1),
            '>' => Syote::Nuoli(0, 1),
            _ => Syote::Kirjain(merkki.to_ascii_uppercase()),
        });
        for _ in 0..40 {
            peli.edista(16);
        }
    }
}

/// `--keys` alkuruudulla tai pelissä, ja `--plays <n>` pelaa n kirjausta: ihmisen vuorot
/// vaikean koneen päätöksin, koneet omassa tahdissaan.
fn valmistele_jatsi(peli: &mut Jatsi, liput: &Liput) {
    if let Some(vaikeus) = liput.vaikeus {
        let taso = jatsi::Vaikeus::KAIKKI
            .into_iter()
            .find(|taso| taso.nimi() == vaikeus.nimi());
        peli.vaikeus = taso.unwrap_or(peli.vaikeus);
    }
    for merkki in liput.napit.as_deref().unwrap_or_default().chars() {
        peli.kasittele(match merkki {
            ' ' => Syote::Ammu,
            '^' => Syote::Nuoli(-1, 0),
            'v' => Syote::Nuoli(1, 0),
            '<' => Syote::Nuoli(0, -1),
            '>' => Syote::Nuoli(0, 1),
            '0'..='9' => Syote::Numero(merkki as u8 - b'0'),
            _ => Syote::Kirjain(merkki.to_ascii_uppercase()),
        });
        peli.edista(HEITTO_MS);
    }
    let Some(kirjauksia) = liput.pelatut else {
        return;
    };
    if peli.vaihe == jatsi_peli::Vaihe::Alku {
        peli.kasittele(Syote::Ammu);
    }
    let kirjattu = |peli: &Jatsi| -> u32 {
        peli.poyta
            .kirjat
            .iter()
            .map(|k| k.kirjattuja() as u32)
            .sum()
    };
    let alku = kirjattu(peli);
    while kirjattu(peli) - alku < kirjauksia && peli.vaihe == jatsi_peli::Vaihe::Pelissa {
        if peli.poyta.kone_vuorossa() {
            peli.edista(KONE_ASKEL_MS + HEITTO_MS);
        } else {
            match koneen_toimi(jatsi::Vaikeus::Vaikea, &peli.poyta) {
                Toimi::Heita => peli.heita(),
                Toimi::Lukitse(lukitut) => {
                    peli.poyta.lukitut = lukitut;
                    peli.heita();
                }
                Toimi::Kirjaa(rivi) => {
                    // Nolla vaatii vahvistuksen: toinen painallus viivaa.
                    peli.kirjaa(rivi);
                    if peli.viivaus == Some(rivi) {
                        peli.kirjaa(rivi);
                    }
                }
            }
            peli.edista(HEITTO_MS);
        }
    }
}

fn valmistele_palikka(peli: &mut Palikkapeli, liput: &Liput) {
    if let Some(siirtoja) = liput.pelatut {
        peli.kasittele(Syote::Ammu);
        for i in 0..siirtoja {
            // Edellinen toisto loppuu ensin, mutta viimeinen jää kuvakaappauksen hetkelle.
            if i > 0 {
                peli.edista(60_000);
            }
            match peli.lauta.vihje() {
                Some((a, b)) if peli.vaihe == palikka_peli::Vaihe::Pelissa => {
                    peli.vaihda(a, b);
                }
                _ => break,
            }
        }
    }
}

fn valmistele_miinaharava(peli: &mut Miinaharava, liput: &Liput) {
    if let Some(vaikeus) = liput.vaikeus {
        let taso = miinaharava::Vaikeus::KAIKKI
            .into_iter()
            .find(|taso| taso.nimi() == vaikeus.nimi());
        peli.vaikeus = taso.unwrap_or(peli.vaikeus);
    }
    if liput.pelatut.is_some() || liput.pamaus {
        peli.kasittele(Syote::Ammu);
        let (korkeus, leveys) = (peli.vaikeus.korkeus(), peli.vaikeus.leveys());
        peli.avaa((korkeus / 2, leveys / 2));
        let kaikki: Vec<(usize, usize)> = (0..korkeus)
            .flat_map(|r| (0..leveys).map(move |s| (r, s)))
            .collect();
        let avattavat: Vec<(usize, usize)> = kaikki
            .iter()
            .copied()
            .filter(|&p| !peli.lauta.ruutu(p).miina && !peli.lauta.ruutu(p).auki)
            .take(liput.pelatut.unwrap_or(0) as usize)
            .collect();
        for p in avattavat {
            peli.avaa(p);
        }
        let miinat: Vec<(usize, usize)> = kaikki
            .iter()
            .copied()
            .filter(|&p| peli.lauta.ruutu(p).miina)
            .collect();
        // Oikea ja väärä lippu, jotta kaikki merkit näkyvät kuvassa.
        peli.liputa(miinat[0]);
        let vaara = kaikki
            .iter()
            .copied()
            .find(|&p| !peli.lauta.ruutu(p).miina && !peli.lauta.ruutu(p).auki);
        if let Some(vaara) = vaara {
            peli.liputa(vaara);
        }
        if liput.pamaus {
            peli.avaa(miinat[1]);
        }
    }
}

fn valmistele_mahjong(peli: &mut Mahjong, liput: &Liput) {
    if let Some(vaikeus) = liput.vaikeus {
        let taso = mahjong::Vaikeus::KAIKKI
            .into_iter()
            .find(|taso| taso.nimi() == vaikeus.nimi());
        peli.vaikeus = taso.unwrap_or(peli.vaikeus);
    }
    if liput.pelatut.is_some() || liput.voitto {
        peli.kasittele(Syote::Ammu);
        // Pareja otetaan jaon omassa ratkaisujärjestyksessä, joten peli ei jumitu.
        let ratkaisu = peli.lauta.ratkaisu.clone();
        let maara = if liput.voitto {
            ratkaisu.len()
        } else {
            liput.pelatut.unwrap_or(0) as usize
        };
        for &(a, b) in ratkaisu.iter().take(maara) {
            peli.edista(1500);
            peli.valitse(a);
            peli.valitse(b);
        }
        // Viimeinen pari jää valituksi, jotta korostus näkyy kuvassa.
        if let Some(&(a, _)) = ratkaisu.get(maara) {
            peli.valitse(a);
        }
        peli.edista(liput.aika_ms);
    }
}

fn valmistele_blackjack(peli: &mut Blackjack, liput: &Liput) {
    if liput.napit.is_some() || liput.kadet.is_some() || liput.konkurssi || liput.nimi {
        peli.kasittele(Syote::Ammu);
    }
    for merkki in liput.napit.as_deref().unwrap_or_default().chars() {
        peli.kasittele(match merkki {
            ' ' => Syote::Ammu,
            '0'..='9' => Syote::Numero(merkki as u8 - b'0'),
            _ => Syote::Kirjain(merkki),
        });
        peli.edista(10_000);
    }
    if let Some(kasia) = liput.kadet {
        // Pisin käsi näyttää, miten kortit mahtuvat kapeaankin kaistaan.
        let kasi = |arvot: &[u8], panos| blackjack::Kasi {
            kortit: arvot
                .iter()
                .map(|&arvo| Kortti {
                    maa: Maa::KAIKKI[arvo as usize % Maa::KAIKKI.len()],
                    arvo,
                })
                .collect(),
            panos,
            tuplattu: false,
            halkaistu: true,
            tulos: None,
        };
        let mallit: [&[u8]; 4] = [&[1, 1, 2, 2, 3, 3, 1, 1], &[10, 6], &[1, 5], &[13, 9]];
        peli.poyta.kadet = mallit[..kasia.clamp(1, 4)]
            .iter()
            .map(|arvot| kasi(arvot, 25 * blackjack::YKSI))
            .collect();
        peli.poyta.jakaja = kasi(&[10, 7], 0).kortit;
        peli.poyta.piilossa = true;
        peli.poyta.vuoro = Vuoro::Pelaaja;
    }
    if liput.konkurssi {
        // Kaikki rahat peliin: pelaajalle 17 ja jakajalle 20.
        peli.kasittele(Syote::Kirjain('K'));
        peli.poyta.kenka.extend([10, 7, 10, 10].map(|arvo| Kortti {
            maa: Maa::Pata,
            arvo,
        }));
        peli.kasittele(Syote::Ammu);
        peli.kasittele(Syote::Kirjain('J'));
        peli.edista(10_000);
    }
    if liput.nimi {
        peli.poyta.rahat = blackjack::ALKURAHA + 120 * blackjack::YKSI;
        peli.poistu();
    }
}

fn valmistele_holdem(peli: &mut Holdem, liput: &Liput) {
    if liput.napit.is_some() || liput.konkurssi || liput.voitto || liput.nimi {
        peli.kasittele(Syote::Ammu);
        peli.edista(100_000);
    }
    for merkki in liput.napit.as_deref().unwrap_or_default().chars() {
        peli.kasittele(match merkki {
            ' ' => Syote::Ammu,
            '0'..='9' => Syote::Numero(merkki as u8 - b'0'),
            _ => Syote::Kirjain(merkki),
        });
        peli.edista(100_000);
    }
    if liput.konkurssi {
        peli.vaihe = holdem_peli::Vaihe::Havio;
        peli.viesti = kieli::muotoile("holdem.havio", &[&5]);
    }
    if liput.voitto {
        peli.vaihe = holdem_peli::Vaihe::Voitto;
        peli.viesti = kieli::muotoile("holdem.voitto", &[&5000]);
    }
    if liput.nimi {
        peli.poyta.pelaajat[0].rahat = 1300;
        peli.poistu();
    }
}

pub fn tallenna_aanet(kansio: &str) -> io::Result<()> {
    std::fs::create_dir_all(kansio)?;
    let mikseri = Mikseri::new();
    for aani in aanet::KAIKKI {
        let kielet: &[Kieli] = if aani.on_puhe() {
            &Kieli::KAIKKI
        } else {
            &[Kieli::Suomi]
        };
        for &kieli in kielet {
            let nimi = if aani.on_puhe() {
                format!("{aani:?}_{}.wav", kieli.koodi())
            } else {
                format!("{aani:?}.wav")
            };
            tallenna_wav(&Path::new(kansio).join(nimi), mikseri.naytteet(aani, kieli))?;
        }
    }
    Ok(())
}

fn tallenna_wav(polku: &Path, naytteet: &[i16]) -> io::Result<()> {
    let tavuja = naytteet.len() as u32 * 2;
    let taajuus = NAYTETAAJUUS as u32;
    let mut ulos = Vec::with_capacity(44 + tavuja as usize);
    ulos.extend_from_slice(b"RIFF");
    ulos.extend_from_slice(&(36 + tavuja).to_le_bytes());
    ulos.extend_from_slice(b"WAVEfmt ");
    ulos.extend_from_slice(&16u32.to_le_bytes());
    // PCM, yksi kanava, 16 bittiä
    ulos.extend_from_slice(&1u16.to_le_bytes());
    ulos.extend_from_slice(&1u16.to_le_bytes());
    ulos.extend_from_slice(&taajuus.to_le_bytes());
    ulos.extend_from_slice(&(taajuus * 2).to_le_bytes());
    ulos.extend_from_slice(&2u16.to_le_bytes());
    ulos.extend_from_slice(&16u16.to_le_bytes());
    ulos.extend_from_slice(b"data");
    ulos.extend_from_slice(&tavuja.to_le_bytes());
    for nayte in naytteet {
        ulos.extend_from_slice(&nayte.to_le_bytes());
    }
    std::fs::write(polku, ulos)
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
