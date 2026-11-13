//! Pelien yhteinen tallennus tekstitiedostoon: asetukset, ennätykset, top-listat, lähtemättömät
//! nettitulokset sekä kesken jääneet 2048 ja Jatsi. Rikkinäinen tiedosto ei saa estää pelaamista, joten
//! kelvottomat tiedot jätetään huomiotta. Tiedostoa käsittelee vain `main.rs`.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use klassikot8000::haasteet::HAASTEITA;
use klassikot8000::jatsi;
use klassikot8000::kaivo;
use klassikot8000::laivanupotus;
use klassikot8000::luvut::{Lukulauta, Ruudut, SIVU};
use klassikot8000::mahjong;
use klassikot8000::majakka;
use klassikot8000::miinaharava;
use klassikot8000::pasianssi::{self, SUURIN_TULOS, Variantti};
use klassikot8000::suora;

use crate::pelit::jatsi::peli::{self as jatsi_peli, Istuin, Kesken as JatsiKesken};
use crate::pelit::laivanupotus::peli::Tilasto;
use crate::pelit::luvut::peli::Kesken;
use crate::pelit::pasianssi::peli::Haastepisteet;
use crate::virheloki;
use crate::yhteinen::kieli::Kieli;
use crate::yhteinen::tulokset::{
    NETTIPELEJA, Nettipeli, Tulos, lisaa_tulos, nimi_kelpaa, tulos_tekstiksi, tulos_tekstista,
};

const OTSAKE: &str = "klassikot8000 1";
// Rajat pitävät muokatun tiedoston luvut sellaisina, ettei pistelasku voi ylivuotaa.
const SUURIN_LAATTA: u32 = 1 << 20;
const SUURIN_LUKU: u32 = 1_000_000_000;

/// Yhden top-listan tiedot.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Listatallennus {
    /// Pelin oma ennätys, jos peli pitää sen listasta erillään. Vanhassa tallennuksessa nolla.
    pub paras: u32,
    /// Suurin tulos ensin, enintään `PARHAITA` kappaletta.
    pub parhaat: Vec<Tulos>,
    /// Tulokset, jotka eivät ole vielä päässeet nettilistalle.
    pub jono: Vec<Tulos>,
}

/// Valinnat, jotka säilyvät käynnistyksestä toiseen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Asetukset {
    /// Valikossa viimeksi valittu peli.
    pub valittu: usize,
    pub laivanupotus: laivanupotus::Vaikeus,
    pub pasianssi: pasianssi::Vaikeus,
    pub variantti: Variantti,
    pub miinaharava: miinaharava::Vaikeus,
    pub mahjong: mahjong::Vaikeus,
    pub majakka: majakka::Vaikeus,
    pub kaivo: kaivo::Vaikeus,
    pub jatsi: jatsi::Vaikeus,
    pub suora: suora::Vaikeus,
}

impl Default for Asetukset {
    fn default() -> Self {
        Self {
            valittu: 0,
            laivanupotus: laivanupotus::Vaikeus::Normaali,
            pasianssi: pasianssi::Vaikeus::Normaali,
            variantti: Variantti::Klondike,
            miinaharava: miinaharava::Vaikeus::Helppo,
            mahjong: mahjong::Vaikeus::Helppo,
            majakka: majakka::Vaikeus::Normaali,
            kaivo: kaivo::Vaikeus::Normaali,
            jatsi: jatsi::Vaikeus::Normaali,
            suora: suora::Vaikeus::Normaali,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tallennus {
    /// Oletus on epätosi, joten vanhassa tallennuksessa äänet ovat päällä.
    pub hiljaa: bool,
    /// Oletus on englanti. Tallennus kirjoittaa kielen aina, joten valittu kieli säilyy.
    pub kieli: Kieli,
    pub asetukset: Asetukset,
    /// Järjestys kuten `Nettipeli::KAIKKI`.
    listat: [Listatallennus; NETTIPELEJA],
    /// 2048:n kesken jäänyt peli.
    pub kesken: Option<Kesken>,
    /// Kerran kysytty nimi: summa kasvaa haaste kerrallaan, joten sitä ei kysytä joka voitosta.
    pub haastenimi: String,
    pub haastepisteet: Haastepisteet,
    /// Miinaharavan parhaat ajat sekunteina tasoittain. Nolla tarkoittaa, ettei tasoa ole läpäisty.
    pub miinaajat: [u32; 3],
    /// Laivanupotuksen voitot, tappiot ja ennätys tasoittain.
    pub laivatilasto: [Tilasto; 3],
    /// Jatsin alkuruudun paikat. Tyhjä, kun niitä ei ole tallennettu.
    pub jatsipaikat: Vec<Istuin>,
    pub jatsikesken: Option<JatsiKesken>,
}

/// Listan rivien avaimet tiedostossa: erillinen ennätys, top 3 ja jono. Vanhat nimet
/// säilyvät, jotta aiempi tallennus kelpaa. Ennätys on oma paras tulos silloinkin, kun se ei
/// yltänyt listalle.
fn avaimet(peli: Nettipeli) -> (Option<&'static str>, &'static str, &'static str) {
    match peli {
        Nettipeli::Luvut => (Some("paras"), "top", "lukujono"),
        Nettipeli::Tiilirikko => (Some("tiilirikko"), "tiilitop", "tiilijono"),
        Nettipeli::Asteroidit => (Some("asteroidit"), "asteroiditop", "asteroidijono"),
        Nettipeli::Silmukka => (Some("silmukka"), "silmukkatop", "silmukkajono"),
        Nettipeli::Palikka => (Some("palikka"), "palikkatop", "palikkajono"),
        Nettipeli::Pasianssi => (Some("pasianssiparas"), "pasianssitop", "pasianssijono"),
        Nettipeli::Spider => (Some("spiderparas"), "spidertop", "spiderjono"),
        Nettipeli::Pyramidi => (Some("pyramidiparas"), "pyramiditop", "pyramidijono"),
        Nettipeli::Haasteet => (None, "haastetop", "haastejono"),
        Nettipeli::Blackjack => (Some("blackjackparas"), "blackjacktop", "blackjackjono"),
        Nettipeli::Holdem => (Some("holdemparas"), "holdemtop", "holdemjono"),
        // Miinaharavan ja laivanupotuksen ennätys lasketaan niiden omista tilastoista.
        Nettipeli::Miinaharava => (None, "miinatop", "miinajono"),
        Nettipeli::Laivanupotus => (None, "laivatop", "laivajono"),
        Nettipeli::Mahjong => (Some("mahjongparas"), "mahjongtop", "mahjongjono"),
        Nettipeli::Majakka => (Some("majakka"), "majakkatop", "majakkajono"),
        Nettipeli::Kaivo => (Some("kaivo"), "kaivotop", "kaivojono"),
        Nettipeli::Jatsi => (Some("jatsiparas"), "jatsitop", "jatsijono"),
        Nettipeli::Suora => (Some("suoraparas"), "suoratop", "suorajono"),
    }
}

fn luku(teksti: &str) -> Option<u32> {
    teksti.parse().ok().filter(|&arvo| arvo <= SUURIN_LUKU)
}

/// Valinta tallennetaan järjestysnumerona, joka ei riipu kielestä.
fn valinta<T: Copy>(kaikki: &[T], teksti: &str) -> Option<T> {
    kaikki.get(teksti.parse::<usize>().ok()?).copied()
}

fn numero<T: PartialEq>(kaikki: &[T], arvo: &T) -> usize {
    kaikki.iter().position(|t| t == arvo).unwrap_or(0)
}

fn ruudut(teksti: &str) -> Option<Ruudut> {
    let mut ruudut = [[0; SIVU]; SIVU];
    let mut osat = teksti.split(',');
    for arvo in ruudut.iter_mut().flatten() {
        *arvo = osat.next()?.parse().ok()?;
        if *arvo != 0 && (*arvo < 2 || *arvo > SUURIN_LAATTA || !arvo.is_power_of_two()) {
            return None;
        }
    }
    osat.next().is_none().then_some(ruudut)
}

/// Pilkuin erotetut luvut, joita on tasan `N`.
fn luvut<const N: usize>(teksti: &str) -> Option<[u32; N]> {
    let mut luvut = [0; N];
    let mut osat = teksti.split(',');
    for arvo in &mut luvut {
        *arvo = luku(osat.next()?)?;
    }
    osat.next().is_none().then_some(luvut)
}

/// Rivi muodossa "laji p1,p2,...". Kelvoton rivi jätetään huomiotta.
fn haasterivi(teksti: &str) -> Option<(usize, [u32; HAASTEITA])> {
    let (laji, arvot) = teksti.split_once(' ')?;
    let laji: usize = laji.parse().ok().filter(|&i| i < Variantti::KAIKKI.len())?;
    let mut pisteet = [0; HAASTEITA];
    let mut osat = arvot.split(',');
    for arvo in &mut pisteet {
        *arvo = osat.next()?.parse().ok().filter(|&p| p <= SUURIN_TULOS)?;
    }
    osat.next().is_none().then_some((laji, pisteet))
}

/// Rivi muodossa "taso voitot,tappiot,vähiten_laukauksia".
fn tilastorivi(teksti: &str) -> Option<(usize, Tilasto)> {
    let (taso, arvot) = teksti.split_once(' ')?;
    let taso = taso
        .parse()
        .ok()
        .filter(|&i| i < laivanupotus::Vaikeus::KAIKKI.len())?;
    let [voitot, tappiot, vahiten_laukauksia] = luvut(arvot)?;
    Some((
        taso,
        Tilasto {
            voitot,
            tappiot,
            vahiten_laukauksia,
        },
    ))
}

impl Tallennus {
    pub fn lista(&self, peli: Nettipeli) -> &Listatallennus {
        &self.listat[peli as usize]
    }

    pub fn lista_mut(&mut self, peli: Nettipeli) -> &mut Listatallennus {
        &mut self.listat[peli as usize]
    }

    pub fn tekstiksi(&self) -> String {
        let a = &self.asetukset;
        let mut teksti = format!(
            "{OTSAKE}\nhiljaa {}\nkieli {}\nvalittu {}\nlaivataso {}\npasianssitaso {}\n\
             pasianssilaji {}\nmiinataso {}\nmiinaajat {},{},{}\nmahjongtaso {}\nmajakkataso {}\nkaivotaso {}\njatsitaso {}\nsuorataso {}\n",
            self.hiljaa as u8,
            self.kieli.koodi(),
            a.valittu,
            numero(&laivanupotus::Vaikeus::KAIKKI, &a.laivanupotus),
            numero(&pasianssi::Vaikeus::KAIKKI, &a.pasianssi),
            numero(&Variantti::KAIKKI, &a.variantti),
            numero(&miinaharava::Vaikeus::KAIKKI, &a.miinaharava),
            self.miinaajat[0],
            self.miinaajat[1],
            self.miinaajat[2],
            numero(&mahjong::Vaikeus::KAIKKI, &a.mahjong),
            numero(&majakka::Vaikeus::KAIKKI, &a.majakka),
            numero(&kaivo::Vaikeus::KAIKKI, &a.kaivo),
            numero(&jatsi::Vaikeus::KAIKKI, &a.jatsi),
            numero(&suora::Vaikeus::KAIKKI, &a.suora),
        );
        for peli in Nettipeli::KAIKKI {
            let (paras, top, jono) = avaimet(peli);
            let lista = self.lista(peli);
            if let Some(avain) = paras {
                teksti += &format!("{avain} {}\n", lista.paras);
            }
            for tulos in &lista.parhaat {
                teksti += &format!("{top} {}\n", tulos_tekstiksi(tulos));
            }
            for tulos in &lista.jono {
                teksti += &format!("{jono} {}\n", tulos_tekstiksi(tulos));
            }
        }
        for (taso, tilasto) in self.laivatilasto.iter().enumerate() {
            teksti += &format!(
                "laivatilasto {taso} {},{},{}\n",
                tilasto.voitot, tilasto.tappiot, tilasto.vahiten_laukauksia
            );
        }
        if !self.haastenimi.is_empty() {
            teksti += &format!("haastenimi {}\n", self.haastenimi);
        }
        for (laji, pisteet) in self.haastepisteet.iter().enumerate() {
            let luvut: Vec<String> = pisteet.iter().map(u32::to_string).collect();
            teksti += &format!("haastepisteet {laji} {}\n", luvut.join(","));
        }
        for rivi in jatsi_peli::paikat_riveiksi(&self.jatsipaikat) {
            teksti += &format!("jatsipaikka {rivi}\n");
        }
        if let Some(kesken) = &self.jatsikesken {
            teksti += &format!("jatsivuoro {}\n", kesken.vuororivi());
            for rivi in kesken.kirjarivit() {
                teksti += &format!("jatsikirja {rivi}\n");
            }
        }
        if let Some(kesken) = &self.kesken {
            let luvut: Vec<String> = kesken.ruudut.iter().flatten().map(u32::to_string).collect();
            teksti += &format!(
                "lauta {}\npisteet {}\nsiirrot {}\njatkettu {}\n",
                luvut.join(","),
                kesken.pisteet,
                kesken.siirrot,
                kesken.jatkettu as u8,
            );
        }
        teksti
    }

    /// `None`, kun teksti ei ala tämän ohjelman otsakkeella: tiedosto on jotain muuta tai
    /// uudemman version kirjoittama, eikä sitä saa kirjoittaa tyhjällä päälle.
    pub fn tekstista(teksti: &str) -> Option<Self> {
        // Muistio voi lisätä tiedoston alkuun BOM-merkin.
        let mut rivit = teksti.trim_start_matches('\u{feff}').lines();
        if rivit.next() != Some(OTSAKE) {
            return None;
        }
        let mut t = Self::default();
        let (mut lauta, mut pisteet, mut siirrot, mut jatkettu) = (None, None, None, false);
        let (mut jatsipaikat, mut jatsivuoro, mut jatsikirjat) = (Vec::new(), None, Vec::new());
        for rivi in rivit {
            let Some((avain, arvo)) = rivi.split_once(' ') else {
                continue;
            };
            let a = &mut t.asetukset;
            match avain {
                "hiljaa" => t.hiljaa = arvo == "1",
                "kieli" => t.kieli = Kieli::koodista(arvo).unwrap_or(t.kieli),
                "valittu" => a.valittu = arvo.parse().unwrap_or(a.valittu),
                "laivataso" => {
                    a.laivanupotus =
                        valinta(&laivanupotus::Vaikeus::KAIKKI, arvo).unwrap_or(a.laivanupotus);
                }
                "pasianssitaso" => {
                    a.pasianssi = valinta(&pasianssi::Vaikeus::KAIKKI, arvo).unwrap_or(a.pasianssi);
                }
                "pasianssilaji" => {
                    a.variantti = valinta(&Variantti::KAIKKI, arvo).unwrap_or(a.variantti);
                }
                "miinataso" => {
                    a.miinaharava =
                        valinta(&miinaharava::Vaikeus::KAIKKI, arvo).unwrap_or(a.miinaharava);
                }
                "mahjongtaso" => {
                    a.mahjong = valinta(&mahjong::Vaikeus::KAIKKI, arvo).unwrap_or(a.mahjong);
                }
                "majakkataso" => {
                    a.majakka = valinta(&majakka::Vaikeus::KAIKKI, arvo).unwrap_or(a.majakka);
                }
                "kaivotaso" => {
                    a.kaivo = valinta(&kaivo::Vaikeus::KAIKKI, arvo).unwrap_or(a.kaivo);
                }
                "jatsitaso" => {
                    a.jatsi = valinta(&jatsi::Vaikeus::KAIKKI, arvo).unwrap_or(a.jatsi);
                }
                "suorataso" => {
                    a.suora = valinta(&suora::Vaikeus::KAIKKI, arvo).unwrap_or(a.suora);
                }
                "jatsipaikka" => jatsipaikat.push(arvo),
                "jatsivuoro" => jatsivuoro = Some(arvo),
                "jatsikirja" => jatsikirjat.push(arvo),
                "miinaajat" => t.miinaajat = luvut(arvo).unwrap_or_default(),
                "laivatilasto" => {
                    if let Some((taso, tilasto)) = tilastorivi(arvo) {
                        t.laivatilasto[taso] = tilasto;
                    }
                }
                "haastenimi" => {
                    if nimi_kelpaa(arvo) {
                        t.haastenimi = arvo.to_string();
                    }
                }
                "haastepisteet" => {
                    if let Some((laji, pisteet)) = haasterivi(arvo) {
                        t.haastepisteet[laji] = pisteet;
                    }
                }
                "lauta" => lauta = ruudut(arvo),
                "pisteet" => pisteet = luku(arvo),
                "siirrot" => siirrot = luku(arvo),
                "jatkettu" => jatkettu = arvo == "1",
                _ => t.lue_listan_rivi(avain, arvo),
            }
        }
        // Kelvoton paikka hylkää kaikki paikat, jottei pöytä jää puolittain vanhaksi.
        if !jatsipaikat.is_empty() {
            t.jatsipaikat = jatsi_peli::paikat_riveista(&jatsipaikat).unwrap_or_default();
        }
        t.jatsikesken = jatsivuoro.and_then(|vuoro| JatsiKesken::riveista(vuoro, &jatsikirjat));
        t.kesken = match (lauta, pisteet, siirrot) {
            (Some(ruudut), Some(pisteet), Some(siirrot))
                if Lukulauta::ruuduista(ruudut).on_siirtoja() =>
            {
                Some(Kesken {
                    ruudut,
                    pisteet,
                    siirrot,
                    jatkettu,
                })
            }
            _ => None,
        };
        Some(t)
    }

    /// Ennätys-, top- tai jonorivi. Tuntematon avain ohitetaan.
    fn lue_listan_rivi(&mut self, avain: &str, arvo: &str) {
        for peli in Nettipeli::KAIKKI {
            let (paras, top, jono) = avaimet(peli);
            let lista = self.lista_mut(peli);
            if paras == Some(avain) {
                lista.paras = luku(arvo).unwrap_or(0);
            } else if avain == top || avain == jono {
                let kohde = if avain == top {
                    &mut lista.parhaat
                } else {
                    &mut lista.jono
                };
                if let Some(tulos) = tulos_tekstista(arvo) {
                    lisaa_tulos(kohde, tulos);
                }
            } else {
                continue;
            }
            return;
        }
    }
}

#[cfg(not(debug_assertions))]
fn tallennuskansio() -> Option<PathBuf> {
    let muuttuja = |nimi| std::env::var_os(nimi).filter(|arvo| !arvo.is_empty());
    if cfg!(windows) {
        return muuttuja("APPDATA").map(PathBuf::from);
    }
    if cfg!(target_os = "macos") {
        return Some(PathBuf::from(muuttuja("HOME")?).join("Library/Application Support"));
    }
    muuttuja("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| Some(PathBuf::from(muuttuja("HOME")?).join(".local/share")))
}

#[cfg(not(debug_assertions))]
pub fn oletuspolku() -> Option<PathBuf> {
    Some(
        tallennuskansio()?
            .join("klassikot8000")
            .join("tallennus.txt"),
    )
}

/// Tallennuksen lukko. Jokainen ilmentymä lukee tallennuksen käynnistyessään ja kirjoittaa koko
/// tilansa, joten kaksi yhtäaikaista hävittäisi toistensa ennätykset.
#[cfg(any(test, not(debug_assertions)))]
#[derive(Debug)]
pub enum Lukitus {
    /// Lukko pysyy niin kauan kuin tiedosto on auki, ja käyttöjärjestelmä vapauttaa sen myös
    /// kaatuneelta ohjelmalta.
    Oma(fs::File),
    /// Toinen ilmentymä on jo käynnissä.
    Varattu,
    /// Lukkoa ei saatu tehtyä, esimerkiksi koska kansioon ei voi kirjoittaa.
    Puuttuu(io::Error),
}

/// Lukitsee tallennuksen tälle ilmentymälle. Lukko on oma tiedostonsa tallennuksen vieressä,
/// koska tallennus itse korvataan jokaisella kirjoituksella uudella tiedostolla.
#[cfg(any(test, not(debug_assertions)))]
pub fn lukitse(polku: &Path) -> Lukitus {
    let avaa = || {
        if let Some(kansio) = polku.parent() {
            fs::create_dir_all(kansio)?;
        }
        fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(polku.with_extension("lukko"))
    };
    match avaa() {
        Ok(tiedosto) => match tiedosto.try_lock() {
            Ok(()) => Lukitus::Oma(tiedosto),
            Err(fs::TryLockError::WouldBlock) => Lukitus::Varattu,
            Err(fs::TryLockError::Error(virhe)) => Lukitus::Puuttuu(virhe),
        },
        Err(virhe) => Lukitus::Puuttuu(virhe),
    }
}

/// Lukee tallennuksen. Puuttuva tiedosto antaa tyhjän tallennuksen. Tyhjä tallennus kirjoitetaan
/// myöhemmin tiedoston päälle, joten kelvoton tai lukukelvoton tiedosto kopioidaan ensin talteen
/// omalla nimellään, jottei ennätyksiä menetetä. Virhe kertoo, ettei kopiokaan onnistunut:
/// silloin tiedostoon ei saa kirjoittaa.
pub fn lue_tiedosto(polku: &Path) -> io::Result<Tallennus> {
    let syy = match fs::read_to_string(polku) {
        Ok(teksti) => match Tallennus::tekstista(&teksti) {
            Some(tallennus) => return Ok(tallennus),
            None => "tuntematon muoto".to_string(),
        },
        Err(virhe) if virhe.kind() == io::ErrorKind::NotFound => return Ok(Tallennus::default()),
        Err(virhe) => virhe.to_string(),
    };
    let kopio = talteen_polku(polku);
    fs::copy(polku, &kopio)?;
    virheloki::kirjaa(&format!(
        "{}: {syy}, kopio talteen: {}",
        polku.display(),
        kopio.display()
    ));
    Ok(Tallennus::default())
}

/// Kopion nimessä on aika, joten myöhempi kopio ei korvaa aiempaa.
fn talteen_polku(polku: &Path) -> PathBuf {
    let aika = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |aika| aika.as_secs());
    polku.with_extension(format!("{aika}.bak"))
}

/// Kirjoittaa ensin väliaikaiseen tiedostoon, jottei keskeytynyt kirjoitus tuhoa tallennusta.
pub fn kirjoita_tiedosto(polku: &Path, tallennus: &Tallennus) -> io::Result<()> {
    if let Some(kansio) = polku.parent() {
        fs::create_dir_all(kansio)?;
    }
    let valiaikainen = polku.with_extension("tmp");
    fs::write(&valiaikainen, tallennus.tekstiksi())?;
    fs::rename(&valiaikainen, polku)
}

#[cfg(test)]
mod tests;
