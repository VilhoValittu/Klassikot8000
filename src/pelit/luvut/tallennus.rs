//! 2048-pelin tila, parhaat tulokset ja ääniasetus tekstitiedostoon. Rikkinäinen tiedosto ei saa estää pelaamista,
//! joten kelvottomat tiedot jätetään huomiotta. Tiedostoa käsittelee vain `main.rs`.

use std::fs;
use std::io;
use std::path::Path;
#[cfg(not(debug_assertions))]
use std::path::PathBuf;

use klassikot8000::luvut::{Lukulauta, Ruudut, SIVU};

use crate::yhteinen::kieli::Kieli;
use crate::yhteinen::tulokset::{Tulos, lisaa_tulos, tulos_tekstista};

const OTSAKE: &str = "klassikot8000 1";
// Rajat pitävät muokatun tiedoston luvut sellaisina, ettei pistelasku voi ylivuotaa.
const SUURIN_LAATTA: u32 = 1 << 20;
const SUURIN_LUKU: u32 = 1_000_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Kesken {
    pub ruudut: Ruudut,
    pub pisteet: u32,
    pub siirrot: u32,
    pub jatkettu: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tallennus {
    pub paras: u32,
    pub kesken: Option<Kesken>,
    /// Suurin tulos ensin, enintään `PARHAITA` kappaletta.
    pub parhaat: Vec<Tulos>,
    /// Oletus on epätosi, joten vanhassa tallennuksessa äänet ovat päällä.
    pub hiljaa: bool,
    /// Oletus on suomi, joten vanhassa tallennuksessa kieli säilyy entisenä.
    pub kieli: Kieli,
    /// 2048:n tulokset, jotka eivät ole vielä päässeet nettilistalle.
    pub luvut_jono: Vec<Tulos>,
    /// Tiilirikon paras tulos. Vanhassa tallennuksessa nolla.
    pub tiilirikko: u32,
    /// Tiilirikon top 3, järjestys kuten `parhaat`.
    pub tiilirikon_parhaat: Vec<Tulos>,
    /// Tiilirikon tulokset, jotka eivät ole vielä päässeet nettilistalle.
    pub tiilirikon_jono: Vec<Tulos>,
    /// Asteroidien paras tulos. Vanhassa tallennuksessa nolla.
    pub asteroidit: u32,
    /// Asteroidien top 3, järjestys kuten `parhaat`.
    pub asteroidien_parhaat: Vec<Tulos>,
    /// Asteroidien tulokset, jotka eivät ole vielä päässeet nettilistalle.
    pub asteroidien_jono: Vec<Tulos>,
    /// Silmukan paras tulos. Vanhassa tallennuksessa nolla.
    pub silmukka: u32,
    /// Silmukan top 3, järjestys kuten `parhaat`.
    pub silmukan_parhaat: Vec<Tulos>,
    /// Silmukan tulokset, jotka eivät ole vielä päässeet nettilistalle.
    pub silmukan_jono: Vec<Tulos>,
    /// Palikan paras tulos. Vanhassa tallennuksessa nolla.
    pub palikka: u32,
    /// Palikan top 3, järjestys kuten `parhaat`.
    pub palikan_parhaat: Vec<Tulos>,
    /// Palikan tulokset, jotka eivät ole vielä päässeet nettilistalle.
    pub palikan_jono: Vec<Tulos>,
    /// Pasianssin top 3, järjestys kuten `parhaat`.
    pub pasianssin_parhaat: Vec<Tulos>,
    /// Pasianssin tulokset, jotka eivät ole vielä päässeet nettilistalle.
    pub pasianssin_jono: Vec<Tulos>,
    /// Blackjackin top 3, järjestys kuten `parhaat`.
    pub blackjackin_parhaat: Vec<Tulos>,
    /// Blackjackin tulokset, jotka eivät ole vielä päässeet nettilistalle.
    pub blackjackin_jono: Vec<Tulos>,
    /// Hold'emin top 3, järjestys kuten `parhaat`.
    pub holdemin_parhaat: Vec<Tulos>,
    /// Hold'emin tulokset, jotka eivät ole vielä päässeet nettilistalle.
    pub holdemin_jono: Vec<Tulos>,
    /// Miinaharavan parhaat ajat sekunteina tasoittain. Nolla tarkoittaa, ettei tasoa ole läpäisty.
    pub miinaajat: [u32; 3],
}

fn luku(teksti: &str) -> Option<u32> {
    teksti.parse().ok().filter(|&arvo| arvo <= SUURIN_LUKU)
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

fn miinaajat(teksti: &str) -> Option<[u32; 3]> {
    let mut ajat = [0; 3];
    let mut osat = teksti.split(',');
    for aika in &mut ajat {
        *aika = luku(osat.next()?)?;
    }
    osat.next().is_none().then_some(ajat)
}

impl Tallennus {
    pub fn tekstiksi(&self) -> String {
        let mut teksti = format!(
            "{OTSAKE}\nparas {}\nhiljaa {}\nkieli {}\ntiilirikko {}\nasteroidit {}\nsilmukka {}\npalikka {}\nmiinaajat {},{},{}\n",
            self.paras,
            self.hiljaa as u8,
            self.kieli.koodi(),
            self.tiilirikko,
            self.asteroidit,
            self.silmukka,
            self.palikka,
            self.miinaajat[0],
            self.miinaajat[1],
            self.miinaajat[2],
        );
        for tulos in &self.parhaat {
            teksti += &format!("top {} {}\n", tulos.pisteet, tulos.nimi);
        }
        for tulos in &self.tiilirikon_parhaat {
            teksti += &format!("tiilitop {} {}\n", tulos.pisteet, tulos.nimi);
        }
        for tulos in &self.luvut_jono {
            teksti += &format!("lukujono {} {}\n", tulos.pisteet, tulos.nimi);
        }
        for tulos in &self.tiilirikon_jono {
            teksti += &format!("tiilijono {} {}\n", tulos.pisteet, tulos.nimi);
        }
        for tulos in &self.asteroidien_parhaat {
            teksti += &format!("asteroiditop {} {}\n", tulos.pisteet, tulos.nimi);
        }
        for tulos in &self.asteroidien_jono {
            teksti += &format!("asteroidijono {} {}\n", tulos.pisteet, tulos.nimi);
        }
        for tulos in &self.silmukan_parhaat {
            teksti += &format!("silmukkatop {} {}\n", tulos.pisteet, tulos.nimi);
        }
        for tulos in &self.silmukan_jono {
            teksti += &format!("silmukkajono {} {}\n", tulos.pisteet, tulos.nimi);
        }
        for tulos in &self.palikan_parhaat {
            teksti += &format!("palikkatop {} {}\n", tulos.pisteet, tulos.nimi);
        }
        for tulos in &self.palikan_jono {
            teksti += &format!("palikkajono {} {}\n", tulos.pisteet, tulos.nimi);
        }
        for tulos in &self.pasianssin_parhaat {
            teksti += &format!("pasianssitop {} {}\n", tulos.pisteet, tulos.nimi);
        }
        for tulos in &self.pasianssin_jono {
            teksti += &format!("pasianssijono {} {}\n", tulos.pisteet, tulos.nimi);
        }
        for tulos in &self.blackjackin_parhaat {
            teksti += &format!("blackjacktop {} {}\n", tulos.pisteet, tulos.nimi);
        }
        for tulos in &self.blackjackin_jono {
            teksti += &format!("blackjackjono {} {}\n", tulos.pisteet, tulos.nimi);
        }
        for tulos in &self.holdemin_parhaat {
            teksti += &format!("holdemtop {} {}\n", tulos.pisteet, tulos.nimi);
        }
        for tulos in &self.holdemin_jono {
            teksti += &format!("holdemjono {} {}\n", tulos.pisteet, tulos.nimi);
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
        let (mut paras, mut lauta, mut pisteet, mut siirrot) = (None, None, None, None);
        let (mut jatkettu, mut hiljaa) = (false, false);
        let mut kieli = Kieli::default();
        let (mut parhaat, mut tiilirikon_parhaat) = (Vec::new(), Vec::new());
        let (mut luvut_jono, mut tiilirikon_jono) = (Vec::new(), Vec::new());
        let (mut asteroidien_parhaat, mut asteroidien_jono) = (Vec::new(), Vec::new());
        let (mut silmukan_parhaat, mut silmukan_jono) = (Vec::new(), Vec::new());
        let (mut palikan_parhaat, mut palikan_jono) = (Vec::new(), Vec::new());
        let (mut pasianssin_parhaat, mut pasianssin_jono) = (Vec::new(), Vec::new());
        let (mut blackjackin_parhaat, mut blackjackin_jono) = (Vec::new(), Vec::new());
        let (mut holdemin_parhaat, mut holdemin_jono) = (Vec::new(), Vec::new());
        let (mut tiilirikko, mut asteroidit, mut silmukka, mut palikka) = (None, None, None, None);
        let mut ajat = None;
        for rivi in rivit {
            match rivi.split_once(' ') {
                Some(("paras", arvo)) => paras = luku(arvo),
                Some(("hiljaa", arvo)) => hiljaa = arvo == "1",
                Some(("kieli", arvo)) => kieli = Kieli::koodista(arvo).unwrap_or(kieli),
                Some(("tiilirikko", arvo)) => tiilirikko = luku(arvo),
                Some(("asteroidit", arvo)) => asteroidit = luku(arvo),
                Some(("silmukka", arvo)) => silmukka = luku(arvo),
                Some(("palikka", arvo)) => palikka = luku(arvo),
                Some(("miinaajat", arvo)) => ajat = miinaajat(arvo),
                Some(("top", arvo)) => {
                    if let Some(tulos) = tulos_tekstista(arvo) {
                        lisaa_tulos(&mut parhaat, tulos);
                    }
                }
                Some(("tiilitop", arvo)) => {
                    if let Some(tulos) = tulos_tekstista(arvo) {
                        lisaa_tulos(&mut tiilirikon_parhaat, tulos);
                    }
                }
                Some(("lukujono", arvo)) => {
                    if let Some(tulos) = tulos_tekstista(arvo) {
                        lisaa_tulos(&mut luvut_jono, tulos);
                    }
                }
                Some(("tiilijono", arvo)) => {
                    if let Some(tulos) = tulos_tekstista(arvo) {
                        lisaa_tulos(&mut tiilirikon_jono, tulos);
                    }
                }
                Some(("asteroiditop", arvo)) => {
                    if let Some(tulos) = tulos_tekstista(arvo) {
                        lisaa_tulos(&mut asteroidien_parhaat, tulos);
                    }
                }
                Some(("asteroidijono", arvo)) => {
                    if let Some(tulos) = tulos_tekstista(arvo) {
                        lisaa_tulos(&mut asteroidien_jono, tulos);
                    }
                }
                Some(("silmukkatop", arvo)) => {
                    if let Some(tulos) = tulos_tekstista(arvo) {
                        lisaa_tulos(&mut silmukan_parhaat, tulos);
                    }
                }
                Some(("silmukkajono", arvo)) => {
                    if let Some(tulos) = tulos_tekstista(arvo) {
                        lisaa_tulos(&mut silmukan_jono, tulos);
                    }
                }
                Some(("palikkatop", arvo)) => {
                    if let Some(tulos) = tulos_tekstista(arvo) {
                        lisaa_tulos(&mut palikan_parhaat, tulos);
                    }
                }
                Some(("palikkajono", arvo)) => {
                    if let Some(tulos) = tulos_tekstista(arvo) {
                        lisaa_tulos(&mut palikan_jono, tulos);
                    }
                }
                Some(("pasianssitop", arvo)) => {
                    if let Some(tulos) = tulos_tekstista(arvo) {
                        lisaa_tulos(&mut pasianssin_parhaat, tulos);
                    }
                }
                Some(("pasianssijono", arvo)) => {
                    if let Some(tulos) = tulos_tekstista(arvo) {
                        lisaa_tulos(&mut pasianssin_jono, tulos);
                    }
                }
                Some(("blackjacktop", arvo)) => {
                    if let Some(tulos) = tulos_tekstista(arvo) {
                        lisaa_tulos(&mut blackjackin_parhaat, tulos);
                    }
                }
                Some(("blackjackjono", arvo)) => {
                    if let Some(tulos) = tulos_tekstista(arvo) {
                        lisaa_tulos(&mut blackjackin_jono, tulos);
                    }
                }
                Some(("holdemtop", arvo)) => {
                    if let Some(tulos) = tulos_tekstista(arvo) {
                        lisaa_tulos(&mut holdemin_parhaat, tulos);
                    }
                }
                Some(("holdemjono", arvo)) => {
                    if let Some(tulos) = tulos_tekstista(arvo) {
                        lisaa_tulos(&mut holdemin_jono, tulos);
                    }
                }
                Some(("lauta", arvo)) => lauta = ruudut(arvo),
                Some(("pisteet", arvo)) => pisteet = luku(arvo),
                Some(("siirrot", arvo)) => siirrot = luku(arvo),
                Some(("jatkettu", arvo)) => jatkettu = arvo == "1",
                _ => {}
            }
        }
        let kesken = match (lauta, pisteet, siirrot) {
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
        Self {
            paras: paras.unwrap_or(0),
            kesken,
            parhaat,
            hiljaa,
            kieli,
            luvut_jono,
            tiilirikko: tiilirikko.unwrap_or(0),
            tiilirikon_parhaat,
            tiilirikon_jono,
            asteroidit: asteroidit.unwrap_or(0),
            asteroidien_parhaat,
            asteroidien_jono,
            silmukka: silmukka.unwrap_or(0),
            silmukan_parhaat,
            silmukan_jono,
            palikka: palikka.unwrap_or(0),
            palikan_parhaat,
            palikan_jono,
            pasianssin_parhaat,
            pasianssin_jono,
            blackjackin_parhaat,
            blackjackin_jono,
            holdemin_parhaat,
            holdemin_jono,
            miinaajat: ajat.unwrap_or_default(),
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

    fn kokonainen() -> Tallennus {
        Tallennus {
            paras: 5000,
            kesken: Some(KESKEN),
            parhaat: vec![
                Tulos {
                    nimi: "VÄINÖ".to_string(),
                    pisteet: 5000,
                },
                Tulos {
                    nimi: "A1".to_string(),
                    pisteet: 300,
                },
            ],
            hiljaa: true,
            kieli: Kieli::Englanti,
            luvut_jono: vec![Tulos {
                nimi: "A1".to_string(),
                pisteet: 300,
            }],
            tiilirikko: 1250,
            tiilirikon_parhaat: vec![Tulos {
                nimi: "TIILI".to_string(),
                pisteet: 1250,
            }],
            tiilirikon_jono: vec![Tulos {
                nimi: "NIMETÖN".to_string(),
                pisteet: 900,
            }],
            asteroidit: 4000,
            asteroidien_parhaat: vec![Tulos {
                nimi: "KIVI".to_string(),
                pisteet: 4000,
            }],
            asteroidien_jono: vec![Tulos {
                nimi: "KIVI".to_string(),
                pisteet: 4000,
            }],
            silmukka: 2500,
            silmukan_parhaat: vec![Tulos {
                nimi: "ITIKKA".to_string(),
                pisteet: 2500,
            }],
            silmukan_jono: vec![Tulos {
                nimi: "ITIKKA".to_string(),
                pisteet: 2500,
            }],
            palikka: 700,
            palikan_parhaat: vec![Tulos {
                nimi: "KIVI".to_string(),
                pisteet: 700,
            }],
            palikan_jono: vec![Tulos {
                nimi: "KIVI".to_string(),
                pisteet: 700,
            }],
            pasianssin_parhaat: vec![Tulos {
                nimi: "KORTTI".to_string(),
                pisteet: 6600,
            }],
            pasianssin_jono: vec![Tulos {
                nimi: "KORTTI".to_string(),
                pisteet: 6600,
            }],
            blackjackin_parhaat: vec![Tulos {
                nimi: "KORTTI".to_string(),
                pisteet: 120,
            }],
            blackjackin_jono: vec![Tulos {
                nimi: "KORTTI".to_string(),
                pisteet: 120,
            }],
            holdemin_parhaat: vec![Tulos {
                nimi: "KORTTI".to_string(),
                pisteet: 4000,
            }],
            holdemin_jono: vec![Tulos {
                nimi: "KORTTI".to_string(),
                pisteet: 4000,
            }],
            miinaajat: [12, 0, 345],
        }
    }

    #[test]
    fn teksti_palautuu_samaksi() {
        let t = kokonainen();
        assert_eq!(Tallennus::tekstista(&t.tekstiksi()), t);
        let t = Tallennus {
            paras: 7,
            kesken: None,
            parhaat: Vec::new(),
            hiljaa: false,
            kieli: Kieli::Suomi,
            luvut_jono: Vec::new(),
            tiilirikko: 0,
            tiilirikon_parhaat: Vec::new(),
            tiilirikon_jono: Vec::new(),
            asteroidit: 0,
            asteroidien_parhaat: Vec::new(),
            asteroidien_jono: Vec::new(),
            silmukka: 0,
            silmukan_parhaat: Vec::new(),
            silmukan_jono: Vec::new(),
            palikka: 0,
            palikan_parhaat: Vec::new(),
            palikan_jono: Vec::new(),
            pasianssin_parhaat: Vec::new(),
            pasianssin_jono: Vec::new(),
            blackjackin_parhaat: Vec::new(),
            blackjackin_jono: Vec::new(),
            holdemin_parhaat: Vec::new(),
            holdemin_jono: Vec::new(),
            miinaajat: [0; 3],
        };
        assert_eq!(Tallennus::tekstista(&t.tekstiksi()), t);
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
        let pisteet: Vec<u32> = t.parhaat.iter().map(|t| t.pisteet).collect();
        assert_eq!(pisteet, [9, 5]);
        assert_eq!(t.parhaat[0].nimi, "ISO");
    }

    #[test]
    fn vanhassa_tallennuksessa_aanet_ovat_paalla() {
        let t = Tallennus::tekstista("klassikot8000 1\nparas 7\n");
        assert_eq!(t.paras, 7);
        assert!(!t.hiljaa);
        assert_eq!(t.tiilirikko, 0);
        assert_eq!(t.asteroidit, 0);
        assert_eq!(t.silmukka, 0);
        assert_eq!(t.palikka, 0);
        assert_eq!(t.miinaajat, [0; 3]);
        assert!(t.parhaat.is_empty());
    }

    #[test]
    fn kelvottomat_miina_ajat_hylataan() {
        let hyva = "klassikot8000 1\nmiinaajat 1,2,3\n";
        assert_eq!(Tallennus::tekstista(hyva).miinaajat, [1, 2, 3]);
        for huono in ["1,2", "1,2,3,4", "a,b,c", "-1,2,3", "1,2,4000000000"] {
            let t = Tallennus::tekstista(&hyva.replace("1,2,3", huono));
            assert_eq!(t.miinaajat, [0; 3], "{huono}");
        }
    }

    #[test]
    fn kelvoton_lauta_hylataan_mutta_paras_sailyy() {
        let hyva = kokonainen().tekstiksi();
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
            assert_eq!(t.paras, 5000, "{huono}");
            assert_eq!(t.kesken, None, "{huono}");
        }
        let t = Tallennus::tekstista(&hyva.replace("paras 5000", "paras -1"));
        assert_eq!(t.paras, 0);
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
