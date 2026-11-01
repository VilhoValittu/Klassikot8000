//! Tiilirikon näkymät.

use std::sync::LazyLock;

use klassikot8000::tiilirikko::{
    self, KENTTIA, MAILA_K, MAILA_Y, PALLON_SADE, RIVEJA, SARAKKEITA, TIILET_Y, TIILI_K, TIILI_L,
    Tila,
};

use super::peli::{KIPINA_MS, Nappi, Tiilirikko, Vaihe};
use crate::yhteinen::kieli::kaanna;
use crate::yhteinen::piirto::*;
use crate::yhteinen::rasteri::{Rasteri, liukuvari};
use crate::yhteinen::vektori::{Kangas, Tahko, kuutio, sekoita_liuku, tummenna, vaalenna};

pub const KENTTA_X: i32 = 30;
const KENTTA_Y: i32 = 48;
const KENTTA_L: i32 = tiilirikko::LEVEYS as i32;
const KENTTA_K: i32 = tiilirikko::KORKEUS as i32;
const SARAKE_X: i32 = KENTTA_X + KENTTA_L + 30;
const TILASTON_ASKEL: i32 = 87;
const PARHAAT_Y: i32 = KENTTA_Y + 4 * TILASTON_ASKEL + 66;
const PARHAAT_L: i32 = LEVEYS as i32 - 12 - SARAKE_X;
// Kapeaan sarakkeeseen mahtuu sijoitus, kahdeksan merkin nimi ja kuusinumeroiset pisteet.
const PARHAAT_KOKO: (i32, i32, i32) = (10, 15, 13);

const RIVIN_VARIT: [u32; RIVEJA] = [
    OSUMA, WB_ORANSSI, KELTAINEN, 0x70B050, HUTI, 0x4FA8E0, REUNA, 0x8B3FC0,
];

const OHJE: &str = "tiilirikko.ohje";
const LAUKAISU: &str = "tiilirikko.laukaisu";
const TAUKO: &str = "yhteinen.tauko";
const TAUON_OHJE: &str = "yhteinen.tauon_ohje";
const VOITTO: &str = "tiilirikko.voitto";
const TAPPIO: &str = "yhteinen.peli_paattyi";

const NAPIT: [(Nappi, &str, i32); 2] = [
    (Nappi::Uudestaan, "yhteinen.uudestaan", NAPPI_X),
    (
        Nappi::Valikko,
        "yhteinen.valikkoon",
        NAPPI_X + 13 * MERKKI + 48,
    ),
];

const SUPERTIILI: u32 = 0x302850;

/// Kentän tausta: tumma metallilevy, jonka saumat ja niitit erottuvat vain vähän.
static KENTTA_KUVA: LazyLock<Rasteri> = LazyLock::new(|| {
    const LEVY: i32 = 20;
    let (l, k) = (KENTTA_L / 3, KENTTA_K / 3);
    Rasteri::luo(l as usize, k as usize, move |x, y| {
        let vari = liukuvari(&[0x101C60, 0x0A1040, 0x060828], y as f32 / k as f32, x, y);
        let (lx, ly) = (x % LEVY, y % LEVY);
        if lx == 0 || ly == 0 {
            sekoita_liuku(vari, 0x000000, 0.45)
        } else if lx == 1 || ly == 1 {
            sekoita_liuku(vari, 0x4060C0, 0.18)
        } else if (lx == 3 || lx == LEVY - 3) && (ly == 3 || ly == LEVY - 3) {
            sekoita_liuku(vari, 0x6080E0, 0.5)
        } else {
            vari
        }
    })
});
const SUPERPALLO: u32 = KELTAINEN;

const ALKU_OSAT: [&str; 8] = [
    "tiilirikko.tervetuloa",
    "yhteinen.ohjeet",
    "tiilirikko.ohje_1",
    "tiilirikko.ohje_2",
    "tiilirikko.ohje_3",
    "tiilirikko.ohje_4",
    "tiilirikko.ohje_5",
    "tiilirikko.ohje_6",
];
const ALKU_RIVIT: [&str; 4] = [
    "tiilirikko.nappain_hiiri",
    "tiilirikko.nappain_laukaisu",
    "yhteinen.tauko_aanet",
    "yhteinen.takaisin",
];

pub fn nappi_pisteesta(x: i32, y: i32) -> Option<Nappi> {
    NAPIT
        .iter()
        .find(|&&(_, teksti, nx)| {
            (nx..nx + napin_leveys(teksti)).contains(&x)
                && (NAPPI_Y..NAPPI_Y + NAPPI_K).contains(&y)
        })
        .map(|&(nappi, ..)| nappi)
}

#[cfg(test)]
pub fn napin_keskipiste(nappi: Nappi) -> (i32, i32) {
    let (_, teksti, x) = NAPIT.iter().find(|n| n.0 == nappi).copied().unwrap();
    (x + napin_leveys(teksti) / 2, NAPPI_Y + NAPPI_K / 2)
}

fn tiilen_paikka(rivi: usize, sarake: usize) -> (i32, i32) {
    (
        KENTTA_X + sarake as i32 * TIILI_L as i32,
        KENTTA_Y + TIILET_Y as i32 + rivi as i32 * TIILI_K as i32,
    )
}

/// Alkuruudun malli: tiilimuuri ja sen edessä maila.
pub fn tiilimuuri() -> Vec<Tahko> {
    let mut tahkot = Vec::new();
    for (rivi, &vari) in RIVIN_VARIT[..3].iter().enumerate() {
        for sarake in 0..4 {
            let x = -4.4 + sarake as f32 * 2.2;
            let y = 0.9 + rivi as f32 * 0.75;
            kuutio([x, x + 2.0], [y, y + 0.65], [-0.6, 0.6], vari, &mut tahkot);
        }
    }
    kuutio([-1.6, 1.6], [0.0, 0.3], [-0.6, 0.6], TORNI, &mut tahkot);
    tahkot
}

pub fn piirra(peli: &Tiilirikko, kuva: &mut [u32]) {
    let mut p = Piirtaja {
        k: Kangas::new(kuva),
        kello: peli.kello_ms,
    };
    if peli.vaihe == Vaihe::Alku {
        p.tiili_alkuruutu(peli);
    } else {
        p.tiili_nakyma(peli);
    }
    p.aanikytkin(peli.aanet_paalla);
}

impl Piirtaja<'_> {
    fn tiili_alkuruutu(&mut self, peli: &Tiilirikko) {
        self.maisema(kaanna("tiilirikko.banneri"), &tiilimuuri());
        let otsikko = listan_otsikko(peli.netti_parhaat.is_some());
        self.parhaat_alkuruudulla(otsikko, peli.nakyva_lista());
        self.alkuohje(&ALKU_OSAT, &ALKU_RIVIT);
        self.aloituskehote(ALOITUS);
    }

    fn tiili_nakyma(&mut self, peli: &Tiilirikko) {
        self.tausta_kentan_ympari(KENTTA_X, KENTTA_Y, KENTTA_L, KENTTA_K);
        self.otsikkopalkki(&kaanna("peli.tiilirikko").to_uppercase(), "");

        KENTTA_KUVA.piirra(&mut self.k, KENTTA_X, KENTTA_Y);
        self.upotus(KENTTA_X, KENTTA_Y, KENTTA_L, KENTTA_K);
        self.tiilet(peli);
        self.maila_ja_pallo(peli);
        self.tiili_tilasto(peli);
        self.tiili_paneeli(peli);
    }

    fn tiilet(&mut self, peli: &Tiilirikko) {
        let (l, k) = (TIILI_L as i32, TIILI_K as i32);
        for (rivi, &vari) in RIVIN_VARIT.iter().enumerate() {
            for sarake in 0..SARAKKEITA {
                let kesto = peli.lauta.tiilet[rivi][sarake];
                if kesto == 0 {
                    continue;
                }
                let (x, y) = tiilen_paikka(rivi, sarake);
                let pallotiili = peli.lauta.pallotiilet[rivi][sarake];
                let supertiili = peli.lauta.supertiilet[rivi][sarake];
                let vari = if pallotiili {
                    VALKOINEN
                } else if supertiili {
                    SUPERTIILI
                } else {
                    vari
                };
                // Viisto reuna ja kiilto: valo tulee vasemmalta ylhäältä.
                self.k.laatikko(x, y, l, k, tummenna(vari, 0.7));
                self.k.laatikko(x, y, l - 3, k - 3, vari);
                self.k.laatikko(x, y, l - 3, 3, vaalenna(vari, 0.6));
                self.k.laatikko(x, y, 3, k - 3, vaalenna(vari, 0.35));
                self.k
                    .laatikko(x + 3, y + k - 6, l - 6, 3, tummenna(vari, 0.3));
                self.k.laatikko(x + 6, y + 6, 12, 3, vaalenna(vari, 0.75));
                // Palkintotiilen keskellä on pallon kokoinen merkki sen tuomaa palloa vastaavin värein.
                if pallotiili || supertiili {
                    let sivu = 2 * PALLON_SADE as i32;
                    let merkki = if supertiili { SUPERPALLO } else { TORNI };
                    let (mx, my) = (x + l / 2 - sivu / 2, y + k / 2 - sivu / 2 - 1);
                    self.k.laatikko(mx, my, sivu, sivu, MUSTA);
                    self.k.laatikko(mx + 3, my + 3, sivu - 6, sivu - 6, merkki);
                }
                // Kestävän tiilen keskellä näkyy yksi niitti jokaista lisäosumaa kohti.
                for niitti in 1..kesto as i32 {
                    let nx = x + l / 2 + (2 * niitti - kesto as i32) * 9 - 6;
                    self.k
                        .laatikko(nx, y + k / 2 - 6, 9, 9, tummenna(vari, 0.8));
                    self.k.laatikko(nx, y + k / 2 - 6, 6, 6, TORNI);
                    self.k.laatikko(nx, y + k / 2 - 6, 3, 3, VALKOINEN);
                }
            }
        }
        for &(rivi, sarake, alku) in &peli.kipinat {
            let (x, y) = tiilen_paikka(rivi, sarake);
            let ika = self.kello.saturating_sub(alku) as f32 / KIPINA_MS as f32;
            self.rajahdys((x + l / 2, y + k / 2), ika, 8, 45.0);
        }
    }

    fn maila_ja_pallo(&mut self, peli: &Tiilirikko) {
        let lauta = &peli.lauta;
        let (l, k) = (lauta.maila_l() as i32, MAILA_K as i32);
        let x = KENTTA_X + (lauta.maila_x - lauta.maila_l() / 2.0) as i32;
        let y = KENTTA_Y + MAILA_Y as i32;
        // Metallinen kapseli, jonka päissä hehkuvat moottorit.
        self.k.laatikko(x + 3, y + 3, l, k, 0x04061C);
        self.k.liukulaatikko(x + 3, y, l - 6, k, TORNI, 0x384058);
        self.k.laatikko(x + 12, y + 3, l - 24, 3, VALKOINEN);
        for (reuna, kiilto) in [(x, 6), (x + l - 15, 9)] {
            self.k.liukulaatikko(reuna, y, 15, k, KELTAINEN, OSUMA);
            self.k.laatikko(reuna + kiilto - 3, y + 3, 3, 3, VALKOINEN);
        }

        for pallo in &lauta.pallot {
            let px = KENTTA_X + pallo.paikka.0 as i32;
            let py = KENTTA_Y + pallo.paikka.1 as i32;
            // Pallo katoaa kentän alareunassa, joten paneelin päälle se ei piirry.
            if py + PALLON_SADE as i32 <= KENTTA_Y + KENTTA_K {
                let vari = if pallo.superpallo {
                    SUPERPALLO
                } else {
                    VALKOINEN
                };
                self.k.kiiltopallo((px, py), PALLON_SADE as i32 + 2, vari);
            }
        }
    }

    fn tiili_tilasto(&mut self, peli: &Tiilirikko) {
        let lauta = &peli.lauta;
        let kentta = format!("{}/{KENTTIA}", lauta.kentta + 1);
        for (i, (otsikko, arvo)) in [
            (kaanna("yhteinen.pisteet"), lauta.pisteet.to_string()),
            (kaanna("yhteinen.paras"), peli.paras.to_string()),
            (kaanna("tiilirikko.kentta"), kentta),
        ]
        .into_iter()
        .enumerate()
        {
            let y = KENTTA_Y + i as i32 * TILASTON_ASKEL;
            self.suurteksti(SARAKE_X, y, otsikko, OTSIKKO, KELTAINEN, false);
            self.suurteksti(SARAKE_X, y + 30, &arvo, OTSIKKO, VALKOINEN, true);
        }

        let y = KENTTA_Y + 3 * TILASTON_ASKEL;
        self.suurteksti(
            SARAKE_X,
            y,
            kaanna("tiilirikko.pallot"),
            OTSIKKO,
            KELTAINEN,
            false,
        );
        for i in 0..lauta.elamat as i32 {
            self.k
                .kiiltopallo((SARAKE_X + 12 + i * 27, y + 42), 11, TORNI);
        }

        let y = KENTTA_Y + 4 * TILASTON_ASKEL;
        let bonus = lauta.aikabonus().to_string();
        self.suurteksti(
            SARAKE_X,
            y,
            kaanna("tiilirikko.aikabonus"),
            TULOS,
            KELTAINEN,
            false,
        );
        self.suurteksti(SARAKE_X, y + 30, &bonus, OTSIKKO, VALKOINEN, true);

        self.parhaat_lista(
            SARAKE_X,
            PARHAAT_Y,
            PARHAAT_L,
            PARHAAT_KOKO,
            listan_otsikko(peli.netti_parhaat.is_some()),
            peli.nakyva_lista(),
        );
    }

    fn tiili_paneeli(&mut self, peli: &Tiilirikko) {
        self.paneelin_tausta();
        self.teksti(TEKSTI_X, PANEELI_RIVI_Y, &peli.viesti, VALKOINEN);

        let alarivi = PANEELI_RIVI_Y + (PANEELI_RIVEJA as i32 - 1) * RIVIN_KORKEUS;
        let otsikko_y = alarivi - RIVIN_KORKEUS - 12;
        match peli.vaihe {
            Vaihe::Pelissa if peli.lauta.tila == Tila::Kiinni => {
                self.teksti(TEKSTI_X, alarivi, kaanna(LAUKAISU), KELTAINEN);
            }
            Vaihe::Pelissa => self.teksti(TEKSTI_X, alarivi, kaanna(OHJE), HUTI),
            Vaihe::Tauko => {
                self.suurteksti(TEKSTI_X, otsikko_y, kaanna(TAUKO), TULOS, KELTAINEN, true);
                self.teksti(TEKSTI_X, alarivi, kaanna(TAUON_OHJE), HUTI);
            }
            Vaihe::Voitto | Vaihe::Tappio => {
                let otsikko = if peli.vaihe == Vaihe::Voitto {
                    VOITTO
                } else {
                    TAPPIO
                };
                self.suurteksti(TEKSTI_X, otsikko_y, kaanna(otsikko), TULOS, KELTAINEN, true);
                for (_, teksti, x) in NAPIT {
                    self.nappi(x, NAPPI_Y, teksti, VALKOINEN, WB_SININEN);
                }
            }
            Vaihe::Nimi => self.nimikysely(peli.lauta.pisteet, &peli.nimi),
            Vaihe::Alku => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yhteinen::kieli::{kaikilla_kielilla, muotoile};

    #[test]
    fn napit_loytyvat_eivatka_mene_paallekkain() {
        for (nappi, ..) in NAPIT {
            let (x, y) = napin_keskipiste(nappi);
            assert_eq!(nappi_pisteesta(x, y), Some(nappi));
        }
        assert_eq!(nappi_pisteesta(0, 0), None);
        kaikilla_kielilla(|| assert!(NAPIT[0].2 + napin_leveys(NAPIT[0].1) < NAPIT[1].2));
    }

    #[test]
    fn tekstit_ja_kentta_mahtuvat_paikoilleen() {
        kaikilla_kielilla(|| {
            for teksti in [
                kaanna(OHJE).to_string(),
                kaanna(LAUKAISU).to_string(),
                kaanna(TAUON_OHJE).to_string(),
                kaanna("tiilirikko.uusi_pallo").to_string(),
                kaanna("tiilirikko.superpallo").to_string(),
                kaanna("tiilirikko.toinen_meni").to_string(),
                muotoile("tiilirikko.pallo_meni", &[&3]),
                muotoile("tiilirikko.kentta_lapi", &[&5000]),
                muotoile("tiilirikko.loppubonus", &[&5000, &999999]),
                muotoile("yhteinen.tulos", &[&999999]),
            ] {
                assert!(teksti.chars().count() <= RIVILLE_MERKKEJA, "{teksti}");
            }
            let voitto = kaanna(VOITTO).chars().count() as i32;
            assert!(voitto * TULOS.2 < LEVEYS as i32 - TEKSTI_X);
            for avain in ["tiilirikko.kentta", "tiilirikko.pallot"] {
                let leveys = kaanna(avain).chars().count() as i32 * OTSIKKO.2;
                assert!(SARAKE_X + leveys < LEVEYS as i32, "{avain}");
            }
            let bonus = kaanna("tiilirikko.aikabonus").chars().count() as i32 * TULOS.2;
            assert!(SARAKE_X + bonus < LEVEYS as i32);
        });
        alkuohje_mahtuu(&ALKU_OSAT, &ALKU_RIVIT, ALOITUS_Y);
        const { assert!((2 + 8 + 1 + 6) * PARHAAT_KOKO.2 <= PARHAAT_L) };
        const { assert!(PARHAAT_Y + 30 + 3 * 26 < PANEELI_Y) };
        const { assert!(KENTTA_Y + KENTTA_K < PANEELI_Y) };
        assert!(SARAKE_X + "PISTEET".len() as i32 * OTSIKKO.2 < LEVEYS as i32);
    }

    #[test]
    fn koko_peli_piirtyy_kaatumatta() {
        let mut kuva = vec![0; LEVEYS * KORKEUS];
        let mut peli = Tiilirikko::new(3);
        piirra(&peli, &mut kuva);
        peli.kasittele(crate::yhteinen::syote::Syote::Ammu);
        peli.kasittele(crate::yhteinen::syote::Syote::Ammu);
        // Mailaa ei liikuteta, joten pallot putoavat kentän ali ja peli päättyy.
        for _ in 0..3000 {
            peli.edista(16);
            piirra(&peli, &mut kuva);
            if peli.lauta.tila == Tila::Kiinni {
                peli.kasittele(crate::yhteinen::syote::Syote::Ammu);
            }
        }
        peli.vaihe = Vaihe::Tauko;
        piirra(&peli, &mut kuva);
        peli.vaihe = Vaihe::Voitto;
        piirra(&peli, &mut kuva);
        peli.vaihe = Vaihe::Nimi;
        piirra(&peli, &mut kuva);
    }
}
