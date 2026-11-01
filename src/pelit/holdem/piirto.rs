//! Hold'emin näkymät. Alkuruutu käyttää muiden pelien kanssa samaa klassista kuvaa,
//! pöytä piirretään suoraan täysikokoiselle ruudulle.

use klassikot8000::holdem::{
    Kadenarvo, Kortti, Kulku, Luokka, PELAAJA, PELAAJIA, Pelaaja, Teko, Tila,
};

use super::peli::{Holdem, Napisto, Nappi, Vaihe};
use crate::yhteinen::kieli::{kaanna, muotoile};
use crate::yhteinen::kortit::{KORTTI_K, KORTTI_L, pelimerkki};
use crate::yhteinen::piirto::*;
use crate::yhteinen::vektori::{Kangas, Tahko, kuutio};

const POYTA_X: i32 = 24;
const SIVU_X: i32 = 1332;
const POYTA_L: i32 = SIVU_X - 12 - POYTA_X;
const SIVU_L: i32 = RUUDUN_LEVEYS as i32 - 24 - SIVU_X;
const SIVU_TEKSTI_X: i32 = SIVU_X + 18;
const SIVU_MERKKEJA: usize = 22;

// Vastustajat istuvat pöydän yläreunassa vasemmalta oikealle, eli pelaajasta myötäpäivään.
const PAIKKA_L: i32 = POYTA_L / (PELAAJIA as i32 - 1);
const PAIKKA_Y: i32 = 44;
const PAIKKA_K: i32 = 340;
const NIMI_Y: i32 = PAIKKA_Y + 6;
const RAHAT_RIVI_Y: i32 = NIMI_Y + 28;
const KORTIT_Y: i32 = RAHAT_RIVI_Y + 24;
const TEKO_Y: i32 = KORTIT_Y + KORTTI_K + 8;
const KORTTI_ASKEL: i32 = KORTTI_L + 8;
const MERKKI_L: i32 = 48;

const POTTI_Y: i32 = PAIKKA_Y + PAIKKA_K + 8;
const YHTEISET_Y: i32 = POTTI_Y + 28;
const YHTEISET_ASKEL: i32 = KORTTI_L + 12;
const OMA_Y: i32 = YHTEISET_Y + KORTTI_K + 14;
const OMA_K: i32 = KORTTI_K + 18;
const OMA_KORTTI_ASKEL: i32 = KORTTI_L + 12;
const OMA_TEKSTI_X: i32 = POYTA_X + 18;
const KASI_RIVI_Y: i32 = OMA_Y + OMA_K + 10;
const SAANTO_Y: i32 = KASI_RIVI_Y + 40;
const KOROSTUS: i32 = 3;
const TAPPIO_VARI: u32 = 0xF06060;

const RAHAT_Y: i32 = 54;
const BLINDIT_Y: i32 = RAHAT_Y + TILASTO_ASKEL;
const TILASTO_ASKEL: i32 = 66;
const VIESTI_Y: i32 = RAHAT_Y + 2 * TILASTO_ASKEL + 6;
const VIESTI_K: i32 = 4 * RIVIN_KORKEUS + 18;
const SIVU_OHJE_Y: i32 = VIESTI_Y + VIESTI_K + 18;
const NAPIT_Y: i32 = SIVU_OHJE_Y + 2 * RIVIN_KORKEUS + 18;
const NAPPI_ASKEL: i32 = NAPPI_K + 12;
// Maksun ja korotuksen teksti vaihtuu, joten kaikki napit ovat saman levyisiä.
const NAPIN_L: i32 = SIVU_L - 36;

const VUORO_NAPIT: [Nappi; 8] = [
    Nappi::Maksa,
    Nappi::Kippaa,
    Nappi::Korota,
    Nappi::Pienin,
    Nappi::PuoliPottia,
    Nappi::KokoPotti,
    Nappi::Kaikki,
    Nappi::Valikko,
];
const ODOTUS_NAPIT: [Nappi; 2] = [Nappi::Jatka, Nappi::Valikko];
const LOPPU_NAPIT: [Nappi; 2] = [Nappi::Uudestaan, Nappi::Valikko];

const ALKU_OSAT: [&str; 3] = ["holdem.tervetuloa", "holdem.ohje_1", "holdem.ohje_2"];
const ALKU_RIVIT: [&str; 5] = [
    "holdem.nappain_vuoro",
    "holdem.nappain_korotus",
    "holdem.nappain_jatka",
    "yhteinen.aanet_kytkin",
    "yhteinen.takaisin",
];

pub fn pelaajan_nimi(i: usize) -> &'static str {
    kaanna(match i {
        PELAAJA => "holdem.sina",
        1 => "holdem.kone_1",
        2 => "holdem.kone_2",
        3 => "holdem.kone_3",
        _ => "holdem.kone_4",
    })
}

pub fn kaden_nimi(arvo: &Kadenarvo) -> &'static str {
    if arvo.on_kuningasvarisuora() {
        return kaanna("holdem.kasi_kuningasvarisuora");
    }
    kaanna(match arvo.luokka {
        Luokka::Hai => "holdem.kasi_hai",
        Luokka::Pari => "holdem.kasi_pari",
        Luokka::KaksiParia => "holdem.kasi_kaksi_paria",
        Luokka::Kolmoset => "holdem.kasi_kolmoset",
        Luokka::Suora => "holdem.kasi_suora",
        Luokka::Vari => "holdem.kasi_vari",
        Luokka::Tayskasi => "holdem.kasi_tayskasi",
        Luokka::Neloset => "holdem.kasi_neloset",
        Luokka::Varisuora => "holdem.kasi_varisuora",
    })
}

fn napit(napisto: Napisto) -> &'static [Nappi] {
    match napisto {
        Napisto::Vuoro => &VUORO_NAPIT,
        Napisto::Odotus | Napisto::JakoOhi => &ODOTUS_NAPIT,
        Napisto::Loppu => &LOPPU_NAPIT,
    }
}

pub fn nappi_pisteesta(napisto: Napisto, x: i32, y: i32) -> Option<Nappi> {
    let i = (y - NAPIT_Y).div_euclid(NAPPI_ASKEL);
    let sisalla = (y - NAPIT_Y).rem_euclid(NAPPI_ASKEL) < NAPPI_K;
    let napit = napit(napisto);
    if !sisalla || !(0..napit.len() as i32).contains(&i) {
        return None;
    }
    (SIVU_TEKSTI_X..SIVU_TEKSTI_X + NAPIN_L)
        .contains(&x)
        .then_some(napit[i as usize])
}

#[cfg(test)]
pub fn napin_keskipiste(napisto: Napisto, nappi: Nappi) -> (i32, i32) {
    let i = (napit(napisto).iter()).position(|&n| n == nappi).unwrap() as i32;
    (
        SIVU_TEKSTI_X + NAPIN_L / 2,
        NAPIT_Y + i * NAPPI_ASKEL + NAPPI_K / 2,
    )
}

/// Pöydän näkymässä äänikytkin on täyden ruudun oikeassa laidassa.
pub fn aanikytkimella(vaihe: Vaihe, x: i32, y: i32) -> bool {
    let siirto = if vaihe == Vaihe::Alku {
        0
    } else {
        (RUUDUN_LEVEYS - LEVEYS) as i32
    };
    crate::yhteinen::piirto::aanikytkimella(x - siirto, y)
}

/// Napin teksti ja tieto, voiko sitä painaa.
fn napin_teksti(peli: &Holdem, nappi: Nappi) -> (String, bool) {
    let poyta = &peli.poyta;
    let vapaa = |avain: &str| (kaanna(avain).to_string(), true);
    let korottaa = poyta.voi_korottaa(PELAAJA);
    match nappi {
        Nappi::Maksa => match poyta.maksu(PELAAJA) {
            0 => vapaa("holdem.nappi_tarkista"),
            maksu => (muotoile("holdem.nappi_maksa", &[&maksu]), true),
        },
        Nappi::Kippaa => vapaa("holdem.nappi_kippaa"),
        Nappi::Korota => {
            let avain = if poyta.korkein == 0 {
                "holdem.nappi_panosta"
            } else {
                "holdem.nappi_korota"
            };
            (muotoile(avain, &[&peli.korotus]), korottaa)
        }
        Nappi::Pienin => (kaanna("holdem.nappi_pienin").to_string(), korottaa),
        Nappi::PuoliPottia => (kaanna("holdem.nappi_puoli").to_string(), korottaa),
        Nappi::KokoPotti => (kaanna("holdem.nappi_potti").to_string(), korottaa),
        Nappi::Kaikki => (kaanna("holdem.nappi_kaikki").to_string(), korottaa),
        Nappi::Jatka if poyta.kulku == Kulku::Ohi => vapaa("holdem.nappi_jako"),
        Nappi::Jatka => vapaa("holdem.nappi_jatka"),
        Nappi::Uudestaan => vapaa("yhteinen.uudestaan"),
        Nappi::Valikko => vapaa("yhteinen.valikkoon"),
    }
}

/// Pelaajan kohdan alimmat rivit: viimeisin teko tai käsi, ja panos tai voitto.
fn tekorivi(pelaaja: &Pelaaja) -> (String, u32) {
    if pelaaja.tila == Tila::Poissa {
        return (kaanna("holdem.pudonnut").to_string(), HUTI);
    }
    if let (Some((arvo, _)), true) = (pelaaja.kasi, pelaaja.naytetty) {
        let vari = if pelaaja.voitto > 0 {
            KELTAINEN
        } else {
            VALKOINEN
        };
        return (kaden_nimi(&arvo).to_uppercase(), vari);
    }
    let all_in = pelaaja.tila == Tila::Mukana && pelaaja.rahat == 0;
    if all_in {
        return (kaanna("holdem.all_in").to_string(), WB_ORANSSI);
    }
    match pelaaja.teko {
        Some(Teko::Tarkisti) => (kaanna("holdem.teko_tarkisti").to_string(), VALKOINEN),
        Some(Teko::Maksoi(_)) => (kaanna("holdem.teko_maksoi").to_string(), VALKOINEN),
        Some(Teko::Panosti(_)) => (kaanna("holdem.teko_panosti").to_string(), KELTAINEN),
        Some(Teko::Korotti(_)) => (kaanna("holdem.teko_korotti").to_string(), KELTAINEN),
        Some(Teko::Kippasi) => (kaanna("holdem.teko_kippasi").to_string(), TAPPIO_VARI),
        None => (String::new(), VALKOINEN),
    }
}

fn panosrivi(pelaaja: &Pelaaja) -> Option<(String, u32)> {
    if pelaaja.voitto > 0 {
        Some((format!("+{}", pelaaja.voitto), KELTAINEN))
    } else if pelaaja.panos > 0 {
        let teksti = muotoile("holdem.panos_rivi", &[&pelaaja.panos]);
        Some((teksti, VALKOINEN))
    } else {
        None
    }
}

/// Alkuruudun malli: kolme korttia pystyssä ja pelimerkkipino niiden edessä.
pub fn poytamalli() -> Vec<Tahko> {
    let mut tahkot = Vec::new();
    for (taso, &vari) in [OSUMA, VALKOINEN, OSUMA, VALKOINEN, MUSTA, OSUMA]
        .iter()
        .enumerate()
    {
        pelimerkki(0.0, 2.6, taso, vari, &mut tahkot);
    }
    for (taso, &vari) in [WB_SININEN, VALKOINEN, WB_SININEN].iter().enumerate() {
        pelimerkki(-2.6, 2.2, taso, vari, &mut tahkot);
    }
    // Kortit ovat eri syvyydellä, jotta ne erottuvat toisistaan.
    for (x, z, vari) in [
        (-3.7, -1.0, VALKOINEN),
        (-1.3, -0.3, OSUMA),
        (1.1, -1.0, VALKOINEN),
    ] {
        kuutio([x, x + 2.0], [0.0, 3.0], [z, z + 0.12], vari, &mut tahkot);
    }
    tahkot
}

/// Alkuruutu piirretään klassiseen kuvaan, jonka käynnistin skaalaa ruudulle.
pub fn piirra_alku(peli: &Holdem, kuva: &mut [u32]) {
    let mut p = Piirtaja {
        k: Kangas::new(kuva),
        kello: peli.kello_ms,
    };
    p.maisema(kaanna("holdem.banneri"), &poytamalli());
    p.alkuohje(&ALKU_OSAT, &ALKU_RIVIT);
    p.aloituskehote(ALOITUS);
    p.aanikytkin(peli.aanet_paalla);
}

pub fn piirra(peli: &Holdem, ruutu: &mut [u32]) {
    let mut p = Piirtaja {
        k: Kangas::kokoinen(ruutu, RUUDUN_LEVEYS, RUUDUN_KORKEUS),
        kello: peli.kello_ms,
    };
    p.he_poyta(peli);
    p.aanikytkin(peli.aanet_paalla);
}

/// Kuuluuko kortti jonkun potin voittaneen pelaajan parhaaseen käteen.
fn voittokortti(peli: &Holdem, kortti: Kortti) -> bool {
    peli.poyta.kulku == Kulku::Ohi
        && peli.poyta.pelaajat.iter().any(|pelaaja| {
            pelaaja.voitto > 0
                && pelaaja
                    .kasi
                    .is_some_and(|(_, viisi)| viisi.contains(&kortti))
        })
}

impl Piirtaja<'_> {
    fn he_poyta(&mut self, peli: &Holdem) {
        self.pelitausta();
        self.otsikkopalkki(&kaanna("peli.holdem").to_uppercase(), "");

        for i in (0..PELAAJIA).filter(|&i| i != PELAAJA) {
            self.he_vastustaja(peli, i);
        }
        self.he_yhteiset(peli);
        self.he_oma(peli);
        self.he_alarivit(peli);
        self.he_sivu(peli);
    }

    fn he_kortti(&mut self, peli: &Holdem, x: i32, y: i32, kortti: Kortti) {
        self.pelikortti(x, y, kortti);
        if voittokortti(peli, kortti) {
            for sisennys in 0..KOROSTUS {
                self.k.kehys(
                    x - sisennys,
                    y - sisennys,
                    KORTTI_L + 2 * sisennys,
                    KORTTI_K + 2 * sisennys,
                    WB_ORANSSI,
                );
            }
        }
    }

    /// Nimi vasemmalla, ja napin ja blindin merkki saman rivin oikeassa laidassa.
    fn he_nimi(&mut self, peli: &Holdem, i: usize, x: i32, y: i32, oikea: i32) {
        let pelaaja = &peli.poyta.pelaajat[i];
        let vari = if pelaaja.tila == Tila::Poissa {
            HUTI
        } else {
            KELTAINEN
        };
        let nimi = pelaajan_nimi(i).to_uppercase();
        self.suurteksti(x, y, &nimi, OTSIKKO, vari, true);
        if pelaaja.tila == Tila::Poissa || peli.poyta.jako == 0 {
            return;
        }
        let poyta = &peli.poyta;
        let merkit = [
            (poyta.iso == i, "holdem.merkki_iso", OSUMA),
            (poyta.pieni == i, "holdem.merkki_pieni", WB_SININEN),
            (poyta.nappi == i, "holdem.merkki_nappi", MUSTA),
        ];
        let mut merkki_x = oikea;
        for (paalla, avain, tausta) in merkit {
            if paalla {
                merkki_x -= MERKKI_L + 4;
                self.k.laatikko(merkki_x, y - 3, MERKKI_L, 27, tausta);
                self.k.kehys(merkki_x, y - 3, MERKKI_L, 27, VALKOINEN);
                let teksti = kaanna(avain);
                let tx = merkki_x + (MERKKI_L - teksti.chars().count() as i32 * MERKKI) / 2;
                self.teksti(tx, y, teksti, VALKOINEN);
            }
        }
    }

    fn he_rivit(&mut self, pelaaja: &Pelaaja, x: i32, y: i32) {
        let (teko, vari) = tekorivi(pelaaja);
        self.suurteksti(x, y, &teko, TULOS, vari, false);
        if let Some((rivi, vari)) = panosrivi(pelaaja) {
            self.suurteksti(x, y + 24, &rivi, TULOS, vari, false);
        }
    }

    fn he_vastustaja(&mut self, peli: &Holdem, i: usize) {
        let pelaaja = &peli.poyta.pelaajat[i];
        let x = POYTA_X + (i as i32 - 1) * PAIKKA_L;
        let teksti_x = x + 12;
        if peli.poyta.vuorossa == Some(i) {
            for sisennys in 0..KOROSTUS {
                self.k.kehys(
                    x + 4 - sisennys,
                    PAIKKA_Y - sisennys,
                    PAIKKA_L - 8 + 2 * sisennys,
                    PAIKKA_K + 2 * sisennys,
                    WB_ORANSSI,
                );
            }
        }
        self.he_nimi(peli, i, teksti_x, NIMI_Y, x + PAIKKA_L - 8);
        let rahat = pelaaja.rahat.to_string();
        self.suurteksti(teksti_x, RAHAT_RIVI_Y, &rahat, TULOS, VALKOINEN, false);

        let kortit_x = x + (PAIKKA_L - KORTTI_L - KORTTI_ASKEL) / 2;
        if pelaaja.tila == Tila::Mukana {
            for (j, &kortti) in pelaaja.kortit.iter().enumerate() {
                let kortti_x = kortit_x + j as i32 * KORTTI_ASKEL;
                if pelaaja.naytetty {
                    self.he_kortti(peli, kortti_x, KORTIT_Y, kortti);
                } else {
                    self.kortin_tausta(kortti_x, KORTIT_Y);
                }
            }
        }
        self.he_rivit(pelaaja, teksti_x, TEKO_Y);
    }

    fn he_yhteiset(&mut self, peli: &Holdem) {
        let poyta = &peli.poyta;
        let keski = POYTA_X + POYTA_L / 2;
        let mut teksti = muotoile("holdem.potti_rivi", &[&poyta.potti()]);
        if poyta.sivupotteja() > 0 {
            teksti += &muotoile("holdem.sivupotit", &[&poyta.sivupotteja()]);
        }
        if poyta.kulku != Kulku::Ohi {
            let x = keski - teksti.chars().count() as i32 * OTSIKKO.2 / 2;
            self.suurteksti(x, POTTI_Y, &teksti, OTSIKKO, KELTAINEN, true);
        }
        let leveys = 5 * YHTEISET_ASKEL - (YHTEISET_ASKEL - KORTTI_L);
        let alku = keski - leveys / 2;
        for i in 0..5 {
            let x = alku + i as i32 * YHTEISET_ASKEL;
            match poyta.yhteiset.get(i) {
                Some(&kortti) => self.he_kortti(peli, x, YHTEISET_Y, kortti),
                None => {
                    self.korttipaikka(x, YHTEISET_Y);
                }
            }
        }
    }

    fn he_oma(&mut self, peli: &Holdem) {
        let pelaaja = &peli.poyta.pelaajat[PELAAJA];
        if peli.poyta.vuorossa == Some(PELAAJA) {
            for sisennys in 0..KOROSTUS {
                self.k.kehys(
                    POYTA_X + 4 - sisennys,
                    OMA_Y - sisennys,
                    POYTA_L - 8 + 2 * sisennys,
                    OMA_K + 2 * sisennys,
                    WB_ORANSSI,
                );
            }
        }
        let leveys = 2 * KORTTI_L + (OMA_KORTTI_ASKEL - KORTTI_L);
        let x = POYTA_X + (POYTA_L - leveys) / 2;
        self.he_nimi(peli, PELAAJA, OMA_TEKSTI_X, OMA_Y + 12, x - 24);
        let rahat = pelaaja.rahat.to_string();
        self.suurteksti(OMA_TEKSTI_X, OMA_Y + 42, &rahat, OTSIKKO, VALKOINEN, true);
        self.he_rivit(pelaaja, OMA_TEKSTI_X, OMA_Y + 78);
        if pelaaja.tila == Tila::Mukana {
            for (j, &kortti) in pelaaja.kortit.iter().enumerate() {
                let kortti_x = x + j as i32 * OMA_KORTTI_ASKEL;
                self.he_kortti(peli, kortti_x, OMA_Y + 9, kortti);
            }
        }
    }

    /// Oma käsi nyt ja blinditaso. Pelin lopussa niiden tilalla on tulos.
    fn he_alarivit(&mut self, peli: &Holdem) {
        let poyta = &peli.poyta;
        let keski = POYTA_X + POYTA_L / 2;
        if poyta.pelaajat[PELAAJA].tila == Tila::Mukana
            && let Some(arvo) = poyta.nykyinen_kasi(PELAAJA)
        {
            let nimi = kaden_nimi(&arvo).to_uppercase();
            let teksti = muotoile("holdem.oma_kasi", &[&nimi]);
            let x = keski - teksti.chars().count() as i32 * OTSIKKO.2 / 2;
            self.suurteksti(x, KASI_RIVI_Y, &teksti, OTSIKKO, VALKOINEN, true);
        }
        if peli.vaihe == Vaihe::Pelissa {
            let (taso, jakoja) = (poyta.taso() + 1, poyta.jakoja_tasolla() + 1);
            let teksti = muotoile("holdem.saanto", &[&poyta.jako, &taso, &jakoja]);
            let x = keski - teksti.chars().count() as i32 * MERKKI / 2;
            self.teksti(x, SAANTO_Y, &teksti, VALKOINEN);
        } else {
            let otsikko = if peli.vaihe == Vaihe::Voitto {
                "holdem.voitit"
            } else {
                "holdem.putosit"
            };
            let teksti = kaanna(otsikko);
            let leveys = teksti.chars().count() as i32 * OTSIKKO.2;
            let x = keski - leveys / 2;
            self.ikkuna(x - 18, SAANTO_Y - 12, leveys + 36, 42);
            self.suurteksti(x, SAANTO_Y - 2, teksti, OTSIKKO, KELTAINEN, true);
        }
    }

    fn he_sivu(&mut self, peli: &Holdem) {
        let poyta = &peli.poyta;
        self.ikkuna(SIVU_X - 12, 42, SIVU_L + 24, RUUDUN_KORKEUS as i32 - 66);
        let (pieni, iso) = poyta.blindit();
        let tilastot = [
            (
                RAHAT_Y,
                "holdem.rahat",
                poyta.pelaajat[PELAAJA].rahat.to_string(),
            ),
            (BLINDIT_Y, "holdem.blindit", format!("{pieni}/{iso}")),
        ];
        for (y, otsikko, arvo) in tilastot {
            self.suurteksti(SIVU_TEKSTI_X, y, kaanna(otsikko), OTSIKKO, KELTAINEN, false);
            self.suurteksti(SIVU_TEKSTI_X, y + 30, &arvo, OTSIKKO, VALKOINEN, true);
        }

        self.ikkuna(SIVU_X, VIESTI_Y, SIVU_L, VIESTI_K);
        let mut y = VIESTI_Y + 9;
        for rivi in rivita(&peli.viesti, SIVU_MERKKEJA) {
            self.teksti(SIVU_TEKSTI_X, y, rivi, VALKOINEN);
            y += RIVIN_KORKEUS;
        }

        let napisto = peli.napisto();
        let ohjeet: &[&str] = if napisto == Napisto::Vuoro {
            &["holdem.sivu_nuolet", "holdem.sivu_aanet"]
        } else {
            &["holdem.sivu_aanet"]
        };
        let mut y = SIVU_OHJE_Y;
        for rivi in ohjeet {
            self.teksti(SIVU_TEKSTI_X, y, kaanna(rivi), HUTI);
            y += RIVIN_KORKEUS;
        }

        for (i, &nappi) in napit(napisto).iter().enumerate() {
            let y = NAPIT_Y + i as i32 * NAPPI_ASKEL;
            let (teksti, kaytossa) = napin_teksti(peli, nappi);
            let vari = if kaytossa { WB_SININEN } else { KANSI };
            self.nappitausta(SIVU_TEKSTI_X, y, NAPIN_L, NAPPI_K, VALKOINEN);
            self.teksti(SIVU_TEKSTI_X + 12, y + 3, &teksti, vari);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yhteinen::kieli::kaikilla_kielilla;
    use crate::yhteinen::syote::Syote;
    use klassikot8000::holdem::ALKURAHA;

    const KAIKKI_NAPISTOT: [Napisto; 4] = [
        Napisto::Vuoro,
        Napisto::Odotus,
        Napisto::JakoOhi,
        Napisto::Loppu,
    ];

    #[test]
    fn poyta_mahtuu_ruudulle() {
        const { assert!(TEKO_Y + 24 + TULOS.1 <= PAIKKA_Y + PAIKKA_K) };
        const { assert!(POTTI_Y + OTSIKKO.1 < YHTEISET_Y) };
        const { assert!(YHTEISET_Y + KORTTI_K < OMA_Y) };
        const { assert!(OMA_Y + OMA_K <= KASI_RIVI_Y) };
        const { assert!(SAANTO_Y + 34 < RUUDUN_KORKEUS as i32) };
        const { assert!(5 * YHTEISET_ASKEL <= POYTA_L) };
        // Vastustajan kortit eivät peitä toisiaan, ja koko arvo näkyy näytössä.
        const { assert!(KORTTI_L < KORTTI_ASKEL) };
        const { assert!(KORTTI_L + KORTTI_ASKEL + 16 <= PAIKKA_L) };
        const { assert!(NAPIT_Y + VUORO_NAPIT.len() as i32 * NAPPI_ASKEL < RUUDUN_KORKEUS as i32) };
    }

    #[test]
    fn tekstit_mahtuvat_kummallakin_kielella() {
        let tila = NAPIN_L - 24;
        assert!(SIVU_MERKKEJA as i32 * MERKKI <= SIVU_L - 36);
        kaikilla_kielilla(|| {
            let summa = 5 * ALKURAHA;
            for avain in [
                "holdem.nappi_maksa",
                "holdem.nappi_korota",
                "holdem.nappi_panosta",
            ] {
                let leveys = muotoile(avain, &[&summa]).chars().count() as i32 * MERKKI;
                assert!(leveys <= tila, "{avain}");
            }
            for avain in [
                "holdem.nappi_tarkista",
                "holdem.nappi_kippaa",
                "holdem.nappi_pienin",
                "holdem.nappi_puoli",
                "holdem.nappi_potti",
                "holdem.nappi_kaikki",
                "holdem.nappi_jako",
                "holdem.nappi_jatka",
                "yhteinen.uudestaan",
                "yhteinen.valikkoon",
            ] {
                assert!(
                    kaanna(avain).chars().count() as i32 * MERKKI <= tila,
                    "{avain}"
                );
            }
            for avain in ["holdem.sivu_nuolet", "holdem.sivu_aanet"] {
                assert!(kaanna(avain).chars().count() <= SIVU_MERKKEJA, "{avain}");
            }

            // Pisimmät viestit mahtuvat laatikkoon neljälle riville.
            let kaikki = format!(
                "{} +2500, {} +2500, {} +2500",
                pelaajan_nimi(1),
                pelaajan_nimi(2),
                pelaajan_nimi(3)
            );
            let viestit = [
                muotoile(
                    "holdem.voitot_kasi",
                    &[&kaikki, &kaanna("holdem.kasi_kuningasvarisuora")],
                ),
                muotoile(
                    "holdem.muut_kippasivat",
                    &[&format!("{} +5000", pelaajan_nimi(1))],
                ),
                muotoile("holdem.havio", &[&5]),
                muotoile("holdem.voitto", &[&summa]),
                muotoile("holdem.maksettavaa", &[&summa]),
                muotoile("holdem.miettii", &[&pelaajan_nimi(1)]),
                kaanna("holdem.ei_korotusta").to_string(),
                kaanna("holdem.sinun_vuorosi").to_string(),
                kaanna("holdem.kortit_esiin").to_string(),
                kaanna("holdem.seuraava_kortti").to_string(),
            ];
            for viesti in viestit {
                assert!(rivita(&viesti, SIVU_MERKKEJA).len() <= 4, "{viesti}");
            }

            // Vastustajan kohtaan mahtuu pisin käsi ja nimi merkkien viereen.
            let tila = PAIKKA_L / TULOS.2;
            for avain in [
                "holdem.kasi_kuningasvarisuora",
                "holdem.kasi_kaksi_paria",
                "holdem.pudonnut",
                "holdem.teko_tarkisti",
                "holdem.teko_kippasi",
            ] {
                assert!(kaanna(avain).chars().count() as i32 <= tila, "{avain}");
            }
            for i in 1..PELAAJIA {
                let nimi = pelaajan_nimi(i).chars().count() as i32 * OTSIKKO.2;
                assert!(nimi + 2 * (MERKKI_L + 4) + 24 <= PAIKKA_L, "{i}");
            }
            for avain in [
                "holdem.merkki_iso",
                "holdem.merkki_pieni",
                "holdem.merkki_nappi",
            ] {
                assert!(
                    kaanna(avain).chars().count() as i32 * MERKKI <= MERKKI_L,
                    "{avain}"
                );
            }
            let oma = muotoile(
                "holdem.oma_kasi",
                &[&kaanna("holdem.kasi_kuningasvarisuora").to_uppercase()],
            );
            assert!(oma.chars().count() as i32 * OTSIKKO.2 <= POYTA_L);
            let saanto = muotoile("holdem.saanto", &[&999, &99, &10]);
            assert!(
                saanto.chars().count() as i32 * MERKKI <= POYTA_L,
                "{saanto}"
            );
            let potti =
                muotoile("holdem.potti_rivi", &[&summa]) + &muotoile("holdem.sivupotit", &[&3]);
            assert!(potti.chars().count() as i32 * OTSIKKO.2 <= POYTA_L);
        });
        alkuohje_mahtuu(&ALKU_OSAT, &ALKU_RIVIT, ALOITUS_Y);
    }

    #[test]
    fn alkuohje_kertoo_oikeat_luvut() {
        kaikilla_kielilla(|| {
            let ohje = kaanna("holdem.ohje_2");
            assert!(ohje.contains(&ALKURAHA.to_string()), "{ohje}");
            assert!(ohje.contains("10/20"), "{ohje}");
        });
    }

    #[test]
    fn napit_loytyvat_ja_valit_eivat_ole_kohteita() {
        for napisto in KAIKKI_NAPISTOT {
            for &nappi in napit(napisto) {
                let (x, y) = napin_keskipiste(napisto, nappi);
                assert_eq!(nappi_pisteesta(napisto, x, y), Some(nappi));
            }
        }
        let vali = NAPIT_Y + NAPPI_K;
        assert_eq!(nappi_pisteesta(Napisto::Vuoro, SIVU_TEKSTI_X, vali), None);
        assert_eq!(
            nappi_pisteesta(Napisto::Vuoro, SIVU_TEKSTI_X, NAPIT_Y - 1),
            None
        );
        assert_eq!(
            nappi_pisteesta(Napisto::Vuoro, SIVU_TEKSTI_X - 1, NAPIT_Y),
            None
        );
    }

    #[test]
    fn koko_peli_piirtyy_kaatumatta() {
        let mut ruutu = vec![0; RUUDUN_LEVEYS * RUUDUN_KORKEUS];
        let mut klassinen = vec![0; LEVEYS * KORKEUS];
        for siemen in 0..3 {
            let mut peli = Holdem::new(siemen);
            piirra_alku(&peli, &mut klassinen);
            peli.kasittele(Syote::Ammu);
            for _ in 0..40 {
                piirra(&peli, &mut ruutu);
                peli.kasittele(Syote::Kirjain('R'));
                peli.edista(1000);
                piirra(&peli, &mut ruutu);
                peli.kasittele(Syote::Ammu);
                peli.edista(5000);
                if peli.vaihe != Vaihe::Pelissa {
                    piirra(&peli, &mut ruutu);
                    break;
                }
            }
        }
    }
}
