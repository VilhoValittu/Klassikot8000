//! Pelien yhteinen tallennus tekstitiedostoon: asetukset, ennätykset, top-listat, lähtemättömät
//! nettitulokset ja kesken jäänyt 2048. Rikkinäinen tiedosto ei saa estää pelaamista, joten
//! kelvottomat tiedot jätetään huomiotta. Tiedostoa käsittelee vain `main.rs`.

use std::fs;
use std::io;
use std::path::Path;
#[cfg(not(debug_assertions))]
use std::path::PathBuf;

use klassikot8000::haasteet::HAASTEITA;
use klassikot8000::laivanupotus;
use klassikot8000::luvut::{Lukulauta, Ruudut, SIVU};
use klassikot8000::miinaharava;
use klassikot8000::pasianssi::{self, SUURIN_TULOS, Variantti};

use crate::pelit::laivanupotus::peli::Tilasto;
use crate::pelit::luvut::peli::Kesken;
use crate::pelit::pasianssi::peli::Haastepisteet;
use crate::yhteinen::kieli::Kieli;
use crate::yhteinen::tulokset::{NETTIPELEJA, Nettipeli, Tulos, lisaa_tulos, tulos_tekstista};

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
}

impl Default for Asetukset {
    fn default() -> Self {
        Self {
            valittu: 0,
            laivanupotus: laivanupotus::Vaikeus::Normaali,
            pasianssi: pasianssi::Vaikeus::Normaali,
            variantti: Variantti::Klondike,
            miinaharava: miinaharava::Vaikeus::Helppo,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tallennus {
    /// Oletus on epätosi, joten vanhassa tallennuksessa äänet ovat päällä.
    pub hiljaa: bool,
    /// Oletus on suomi, joten vanhassa tallennuksessa kieli säilyy entisenä.
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
             pasianssilaji {}\nmiinataso {}\nmiinaajat {},{},{}\n",
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
        );
        for peli in Nettipeli::KAIKKI {
            let (paras, top, jono) = avaimet(peli);
            let lista = self.lista(peli);
            if let Some(avain) = paras {
                teksti += &format!("{avain} {}\n", lista.paras);
            }
            for tulos in &lista.parhaat {
                teksti += &format!("{top} {} {}\n", tulos.pisteet, tulos.nimi);
            }
            for tulos in &lista.jono {
                teksti += &format!("{jono} {} {}\n", tulos.pisteet, tulos.nimi);
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

    pub fn tekstista(teksti: &str) -> Self {
        let mut rivit = teksti.lines();
        if rivit.next() != Some(OTSAKE) {
            return Self::default();
        }
        let mut t = Self::default();
        let (mut lauta, mut pisteet, mut siirrot, mut jatkettu) = (None, None, None, false);
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
                "miinaajat" => t.miinaajat = luvut(arvo).unwrap_or_default(),
                "laivatilasto" => {
                    if let Some((taso, tilasto)) = tilastorivi(arvo) {
                        t.laivatilasto[taso] = tilasto;
                    }
                }
                "haastenimi" => {
                    // Nimi kelpaa samoin ehdoin kuin tulosrivillä.
                    if let Some(tulos) = tulos_tekstista(&format!("1 {arvo}")) {
                        t.haastenimi = tulos.nimi;
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
        t
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

pub fn lue_tiedosto(polku: &Path) -> Tallennus {
    fs::read_to_string(polku).map_or_else(|_| Tallennus::default(), |t| Tallennus::tekstista(&t))
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
mod tests {
    use super::*;

    const KESKEN: Kesken = Kesken {
        ruudut: [[2, 4, 8, 16], [0; 4], [0, 1024, 0, 0], [0; 4]],
        pisteet: 1234,
        siirrot: 56,
        jatkettu: true,
    };

    fn tulos(nimi: &str, pisteet: u32) -> Tulos {
        Tulos {
            nimi: nimi.to_string(),
            pisteet,
        }
    }

    fn kokonainen() -> Tallennus {
        let mut t = Tallennus {
            hiljaa: true,
            kieli: Kieli::Englanti,
            asetukset: Asetukset {
                valittu: 7,
                laivanupotus: laivanupotus::Vaikeus::Vaikea,
                pasianssi: pasianssi::Vaikeus::Helppo,
                variantti: Variantti::Pyramidi,
                miinaharava: miinaharava::Vaikeus::Normaali,
            },
            kesken: Some(KESKEN),
            haastenimi: "HAASTAJA".to_string(),
            miinaajat: [12, 0, 345],
            ..Tallennus::default()
        };
        t.haastepisteet[1][3] = 3500;
        t.haastepisteet[2][19] = 3500;
        t.laivatilasto[2] = Tilasto {
            voitot: 4,
            tappiot: 9,
            vahiten_laukauksia: 41,
        };
        // Jokaiselle listalle omat arvot, jotta sekaantuneet avaimet huomataan.
        for (i, peli) in Nettipeli::KAIKKI.into_iter().enumerate() {
            let pisteet = 100 * (i as u32 + 1);
            let lista = t.lista_mut(peli);
            if avaimet(peli).0.is_some() {
                lista.paras = pisteet;
            }
            lista.parhaat = vec![tulos("VÄINÖ", pisteet), tulos("A1", 3)];
            lista.jono = vec![tulos("NIMETÖN", pisteet + 1)];
        }
        t
    }

    #[test]
    fn teksti_palautuu_samaksi() {
        let t = kokonainen();
        assert_eq!(Tallennus::tekstista(&t.tekstiksi()), t);
        let mut t = Tallennus::default();
        t.lista_mut(Nettipeli::Luvut).paras = 7;
        assert_eq!(Tallennus::tekstista(&t.tekstiksi()), t);
    }

    #[test]
    fn vanhan_version_avaimet_luetaan_oikeille_listoille() {
        let t = Tallennus::tekstista(
            "klassikot8000 1\nparas 7\ntop 9 A\nlukujono 8 B\ntiilirikko 6\ntiilitop 5 C\n\
             tiilijono 4 D\nasteroidit 3\nsilmukkatop 2 E\npalikkajono 1 F\nspidertop 11 G\n\
             haastejono 12 H\nholdemtop 13 I\n",
        );
        let luvut = t.lista(Nettipeli::Luvut);
        assert_eq!(luvut.paras, 7);
        assert_eq!(luvut.parhaat, [tulos("A", 9)]);
        assert_eq!(luvut.jono, [tulos("B", 8)]);
        let tiili = t.lista(Nettipeli::Tiilirikko);
        assert_eq!(tiili.paras, 6);
        assert_eq!(tiili.parhaat, [tulos("C", 5)]);
        assert_eq!(tiili.jono, [tulos("D", 4)]);
        assert_eq!(t.lista(Nettipeli::Asteroidit).paras, 3);
        assert_eq!(t.lista(Nettipeli::Silmukka).parhaat, [tulos("E", 2)]);
        assert_eq!(t.lista(Nettipeli::Palikka).jono, [tulos("F", 1)]);
        assert_eq!(t.lista(Nettipeli::Spider).parhaat, [tulos("G", 11)]);
        assert_eq!(t.lista(Nettipeli::Haasteet).jono, [tulos("H", 12)]);
        assert_eq!(t.lista(Nettipeli::Holdem).parhaat, [tulos("I", 13)]);
        assert!(t.lista(Nettipeli::Pasianssi).parhaat.is_empty());
        // asetukset puuttuvat vanhasta tiedostosta
        assert_eq!(t.asetukset, Asetukset::default());
    }

    #[test]
    fn vieras_tai_tyhja_tiedosto_antaa_tyhjan_tallennuksen() {
        assert_eq!(Tallennus::tekstista(""), Tallennus::default());
        assert_eq!(Tallennus::tekstista("paras 99\n"), Tallennus::default());
    }

    #[test]
    fn kelvoton_tulos_hylataan_ja_muokattu_lista_jarjestetaan() {
        let t = Tallennus::tekstista(
            "klassikot8000 1\ntop 5 PIENI\ntop 9 ISO\ntop 0 NOLLA\ntop x KISSA\n\
             top 7 pieni\ntop 7 LIIANPITKA\ntop 7\ntop 7 KAKSI OSAA\n",
        );
        let parhaat = &t.lista(Nettipeli::Luvut).parhaat;
        let pisteet: Vec<u32> = parhaat.iter().map(|t| t.pisteet).collect();
        assert_eq!(pisteet, [9, 5]);
        assert_eq!(parhaat[0].nimi, "ISO");
    }

    #[test]
    fn vanhassa_tallennuksessa_aanet_ovat_paalla() {
        let t = Tallennus::tekstista("klassikot8000 1\nparas 7\n");
        assert_eq!(t.lista(Nettipeli::Luvut).paras, 7);
        assert!(!t.hiljaa);
        for peli in [
            Nettipeli::Tiilirikko,
            Nettipeli::Asteroidit,
            Nettipeli::Silmukka,
            Nettipeli::Palikka,
        ] {
            assert_eq!(t.lista(peli).paras, 0);
        }
        assert_eq!(t.miinaajat, [0; 3]);
        assert!(t.lista(Nettipeli::Luvut).parhaat.is_empty());
    }

    #[test]
    fn kelvottomat_asetukset_hylataan() {
        let t = Tallennus::tekstista(
            "klassikot8000 1\nlaivataso 3\npasianssitaso -1\npasianssilaji x\nmiinataso 9\n\
             valittu kissa\n",
        );
        assert_eq!(t.asetukset, Asetukset::default());
        let t = Tallennus::tekstista("klassikot8000 1\nlaivataso 0\nmiinataso 2\nvalittu 4\n");
        assert_eq!(t.asetukset.laivanupotus, laivanupotus::Vaikeus::Helppo);
        assert_eq!(t.asetukset.miinaharava, miinaharava::Vaikeus::Vaikea);
        assert_eq!(t.asetukset.valittu, 4);
    }

    #[test]
    fn kelvottomat_haastetiedot_hylataan() {
        let nollia = vec!["0"; HAASTEITA].join(",");
        let hyva = format!("klassikot8000 1\nhaastepisteet 2 5,{}\n", &nollia[2..]);
        assert_eq!(Tallennus::tekstista(&hyva).haastepisteet[2][0], 5);
        for huono in [
            format!("haastepisteet 3 {nollia}"),
            format!("haastepisteet 0 {nollia},0"),
            "haastepisteet 0 1,2".to_string(),
            format!("haastepisteet 0 99999999,{}", &nollia[2..]),
        ] {
            let t = Tallennus::tekstista(&format!("klassikot8000 1\n{huono}\n"));
            assert_eq!(t.haastepisteet, Haastepisteet::default(), "{huono}");
        }
        let t = Tallennus::tekstista("klassikot8000 1\nhaastenimi pieni\n");
        assert!(t.haastenimi.is_empty());
    }

    #[test]
    fn kelvottomat_miina_ajat_ja_tilastot_hylataan() {
        let hyva = "klassikot8000 1\nmiinaajat 1,2,3\n";
        assert_eq!(Tallennus::tekstista(hyva).miinaajat, [1, 2, 3]);
        for huono in ["1,2", "1,2,3,4", "a,b,c", "-1,2,3", "1,2,4000000000"] {
            let t = Tallennus::tekstista(&hyva.replace("1,2,3", huono));
            assert_eq!(t.miinaajat, [0; 3], "{huono}");
        }
        for huono in ["3 1,2,3", "0 1,2", "x 1,2,3", "1 1,2,3,4", "1 -1,2,3"] {
            let t = Tallennus::tekstista(&format!("klassikot8000 1\nlaivatilasto {huono}\n"));
            assert_eq!(t.laivatilasto, [Tilasto::default(); 3], "{huono}");
        }
    }

    #[test]
    fn kelvoton_lauta_hylataan_mutta_paras_sailyy() {
        let mut alku = kokonainen();
        alku.lista_mut(Nettipeli::Luvut).paras = 5000;
        let hyva = alku.tekstiksi();
        let lauta = "2,4,8,16,0,0,0,0,0,1024,0,0,0,0,0,0";
        for huono in [
            "3,4,8,16,0,0,0,0,0,1024,0,0,0,0,0,0",
            "1,4,8,16,0,0,0,0,0,1024,0,0,0,0,0,0",
            "4294967296,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0",
            "2147483648,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0",
            "2,4,8",
            "2,4,8,16,0,0,0,0,0,1024,0,0,0,0,0,0,0",
            "kissa",
            // täysi lauta ilman siirtoja
            "2,4,2,4,4,2,4,2,2,4,2,4,4,2,4,2",
        ] {
            let t = Tallennus::tekstista(&hyva.replace(lauta, huono));
            assert_eq!(t.lista(Nettipeli::Luvut).paras, 5000, "{huono}");
            assert_eq!(t.kesken, None, "{huono}");
        }
        let t = Tallennus::tekstista(&hyva.replace("paras 5000", "paras -1"));
        assert_eq!(t.lista(Nettipeli::Luvut).paras, 0);
        assert!(t.kesken.is_some());
    }

    #[test]
    fn tiedosto_luodaan_ja_luetaan_ja_puuttuva_on_tyhja() {
        let kansio = std::env::temp_dir().join(format!("k8000_{}", std::process::id()));
        let polku = kansio.join("ala").join("tallennus.txt");
        assert_eq!(lue_tiedosto(&polku), Tallennus::default());
        kirjoita_tiedosto(&polku, &kokonainen()).unwrap();
        kirjoita_tiedosto(&polku, &kokonainen()).unwrap();
        assert_eq!(lue_tiedosto(&polku), kokonainen());
        fs::remove_dir_all(&kansio).unwrap();
    }
}
