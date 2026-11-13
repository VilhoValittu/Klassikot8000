//! Jatsin näkymät. Alkuruutu käyttää muiden pelien kanssa samaa klassista kuvaa, pöytä
//! piirretään suoraan täysikokoiselle ruudulle: pöytäkirja vasemmalla, nopat ja viestit
//! oikean laidan paneelissa.

use klassikot8000::jatsi::{BONUS, BONUSRAJA, HEITTOJA, NOPPIA, PELAAJIA, Rivi, Vaikeus, pisteet};

use super::peli::{HEITTO_MS, Istuja, JATSI_MS, Jatsi, Vaihe, rivin_nimi};
use crate::yhteinen::kieli::{kaanna, muotoile};
use crate::yhteinen::loppunapit::Loppunappi;
use crate::yhteinen::piirto::*;
use crate::yhteinen::vektori::{Kangas, Tahko, kuutio};

// Pöytäkirja: otsikkorivi, yläosa, välisumma ja bonus, alaosa ja summa.
/// Pöytäkirjan vasen reuna vähintään. Muuten kirja on keskellä paneelin vasemmalla puolella.
const KIRJA_X: i32 = 36;
const KIRJA_Y: i32 = 60;
const OTSIKKO_K: i32 = 54;
const RIVI_K: i32 = 48;
/// Rivien nimien sarake: 12 merkkiä ja välit.
const NIMI_L: i32 = 12 * MERKKI + 24;
/// Pelaajan sarake: kahdeksan merkin nimi ja välit.
const SARAKE_L: i32 = 8 * MERKKI + 24;
const KIRJAN_K: i32 = OTSIKKO_K + 18 * RIVI_K;
/// Teksti rivin keskelle.
const RIVIN_TEKSTI: i32 = (RIVI_K - 24) / 2;

// Oikean laidan paneeli.
const SIVU_X: i32 = 1272;
const SIVU_L: i32 = RUUDUN_LEVEYS as i32 - 24 - SIVU_X;
const SIVU_TEKSTI_X: i32 = SIVU_X + 18;
const SIVU_MERKKEJA: usize = 24;
const VUORO_Y: i32 = 60;
const NOPPA: i32 = 96;
const NOPAN_ASKEL: i32 = NOPPA + 24;
const NOPAT_X: i32 = SIVU_X + (SIVU_L - 5 * NOPAN_ASKEL + 24) / 2;
const NOPAT_Y: i32 = 216;
/// Lukittu noppa nousee tämän verran.
const NOUSU: i32 = 16;
const ALUSTA_Y: i32 = NOPAT_Y - 54;
const ALUSTA_K: i32 = NOPPA + 84;
const HEITTO_X: i32 = SIVU_TEKSTI_X;
const HEITTO_Y: i32 = ALUSTA_Y + ALUSTA_K + 18;
const HEITTO_L: i32 = SIVU_L - 36;
const HEITTO_K: i32 = 48;
const VIESTI_Y: i32 = HEITTO_Y + HEITTO_K + 24;
const VIESTIN_RIVEJA: usize = 5;
const VIESTI_K: i32 = VIESTIN_RIVEJA as i32 * RIVIN_KORKEUS + 18;
const OHJE_Y: i32 = VIESTI_Y + VIESTI_K + 18;
const NAPIT_Y: i32 = RUUDUN_KORKEUS as i32 - 36 - 2 * NAPPI_ASKEL;
const LOPPUNAPIT: [(Loppunappi, &str); 2] = [
    (Loppunappi::Uudestaan, "yhteinen.uudestaan"),
    (Loppunappi::Valikko, "yhteinen.valikkoon"),
];

const NOPPA_VARI: u32 = 0xF4F0E8;
const NOPPA_HIMMEA: u32 = 0x9890A8;
const SILMA: u32 = 0x201838;
const SILMA_YKSI: u32 = 0xC8382A;
const ALUSTA: u32 = 0x0E3A24;
const SARAKE_VUORO: u32 = 0x3858C8;
const VIIVA: u32 = 0x7080C8;
const EHDOTUS: u32 = 0x8088C0;

// Alkuruutu: ohje, paikat allekkain, tasovalinta ja aloituskehote.
const ALKU_MERKIT: usize = 36;
const PAIKAT_Y: i32 = 420;
const PAIKKA_Y: i32 = PAIKAT_Y + 36;
const PAIKAN_ASKEL: i32 = 36;
const ISTUJA_X: i32 = 84;
const ISTUJA_L: i32 = 8 * MERKKI + 24;
const PAIKAN_NIMI_X: i32 = ISTUJA_X + ISTUJA_L + 18;
const PAIKAN_NIMI_L: i32 = 10 * MERKKI;
const PAIKAN_OHJE_Y: i32 = PAIKKA_Y + PELAAJIA as i32 * PAIKAN_ASKEL + 6;
const TASO_Y: i32 = ALOITUS_Y - NAPPI_K - 6;
const TASO_X: i32 = 48 + 6 * MERKKI;
const TASO_VALI: i32 = 12;
const TASOT: [(Vaikeus, &str, i32); 3] = [
    (Vaikeus::Helppo, "yhteinen.taso_1", TASO_X),
    (
        Vaikeus::Normaali,
        "yhteinen.taso_2",
        TASO_X + 8 * MERKKI + 24 + TASO_VALI,
    ),
    (
        Vaikeus::Vaikea,
        "yhteinen.taso_3",
        TASO_X + 18 * MERKKI + 48 + 2 * TASO_VALI,
    ),
];
const UUSI_PELI_NAPPI: &str = "jatsi.uusi_nappi";
const UUSI_PELI_Y: i32 = ALOITUS_Y - NAPPI_K - 6;

/// Silmien paikat nopan keskeltä silmien väleinä.
const SILMAT: [&[(i32, i32)]; 6] = [
    &[(0, 0)],
    &[(-1, -1), (1, 1)],
    &[(-1, -1), (0, 0), (1, 1)],
    &[(-1, -1), (1, -1), (-1, 1), (1, 1)],
    &[(-1, -1), (1, -1), (0, 0), (-1, 1), (1, 1)],
    &[(-1, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (1, 1)],
];

fn kirjan_leveys(pelaajia: usize) -> i32 {
    NIMI_L + pelaajia as i32 * SARAKE_L
}

/// Rivin yläreuna. Välisumma ja bonus ovat yläosan alla.
fn rivin_y(rivi: Rivi) -> i32 {
    let i = rivi.indeksi() as i32;
    let paikka = if i < 6 { i } else { i + 2 };
    KIRJA_Y + OTSIKKO_K + paikka * RIVI_K
}

/// Kirjan vasen reuna konepikselin ruudukossa.
fn kirja_x(pelaajia: usize) -> i32 {
    let keski = (SIVU_X - 12 - kirjan_leveys(pelaajia)) / 2;
    (keski / 4 * 4).max(KIRJA_X)
}

fn sarakkeen_x(pelaajia: usize, pelaaja: usize) -> i32 {
    kirja_x(pelaajia) + NIMI_L + pelaaja as i32 * SARAKE_L
}

pub fn rivi_pisteesta(pelaajia: usize, x: i32, y: i32) -> Option<Rivi> {
    let vasen = kirja_x(pelaajia);
    if !(vasen..vasen + kirjan_leveys(pelaajia)).contains(&x) {
        return None;
    }
    Rivi::KAIKKI
        .into_iter()
        .find(|&rivi| (rivin_y(rivi)..rivin_y(rivi) + RIVI_K).contains(&y))
}

fn nopan_x(noppa: usize) -> i32 {
    NOPAT_X + noppa as i32 * NOPAN_ASKEL
}

/// Noppa ja sen yläpuolinen tila, johon lukittu noppa nousee.
pub fn noppa_pisteesta(x: i32, y: i32) -> Option<usize> {
    if !(NOPAT_Y - NOUSU..NOPAT_Y + NOPPA).contains(&y) {
        return None;
    }
    (0..NOPPIA).find(|&i| (nopan_x(i)..nopan_x(i) + NOPPA).contains(&x))
}

pub fn heitto_pisteessa(x: i32, y: i32) -> bool {
    (HEITTO_X..HEITTO_X + HEITTO_L).contains(&x) && (HEITTO_Y..HEITTO_Y + HEITTO_K).contains(&y)
}

pub fn loppunappi_pisteesta(x: i32, y: i32) -> Option<Loppunappi> {
    sarakkeen_nappi(&LOPPUNAPIT, (SIVU_TEKSTI_X, NAPIT_Y), x, y)
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

pub fn vaikeus_pisteesta(x: i32, y: i32) -> Option<Vaikeus> {
    rivin_nappi(&TASOT, TASO_Y, x, y)
}

fn paikan_y(paikka: usize) -> i32 {
    PAIKKA_Y + paikka as i32 * PAIKAN_ASKEL
}

fn paikka_pisteesta(x: i32, y: i32, vasen: i32, leveys: i32) -> Option<usize> {
    if !(vasen..vasen + leveys).contains(&x) {
        return None;
    }
    (0..PELAAJIA).find(|&i| (paikan_y(i)..paikan_y(i) + NAPPI_K).contains(&y))
}

pub fn istuja_pisteesta(x: i32, y: i32) -> Option<usize> {
    paikka_pisteesta(x, y, ISTUJA_X, ISTUJA_L)
}

pub fn nimi_pisteesta(x: i32, y: i32) -> Option<usize> {
    paikka_pisteesta(x, y, PAIKAN_NIMI_X, PAIKAN_NIMI_L)
}

fn uuden_pelin_x() -> i32 {
    LEVEYS as i32 - 48 - napin_leveys(UUSI_PELI_NAPPI)
}

pub fn uusi_peli_pisteessa(x: i32, y: i32) -> bool {
    (uuden_pelin_x()..uuden_pelin_x() + napin_leveys(UUSI_PELI_NAPPI)).contains(&x)
        && (UUSI_PELI_Y..UUSI_PELI_Y + NAPPI_K).contains(&y)
}

fn istujan_nimi(istuja: Istuja) -> &'static str {
    kaanna(match istuja {
        Istuja::Ihminen => "jatsi.ihminen",
        Istuja::Kone => "jatsi.kone_paikka",
        Istuja::Tyhja => "jatsi.tyhja",
    })
}

/// Alkuruudun malli: kaksi noppaa, joiden silmät erottuvat pyöriessä.
pub fn noppamalli() -> Vec<Tahko> {
    let mut tahkot = Vec::new();
    // Keskikohta (x, z) ja sivujen silmäluvut: ylä, +x, -x, +z, -z.
    for (kx, kz, silmat) in [(-1.7, -0.4, [5, 3, 4, 1, 6]), (1.6, 0.6, [6, 2, 5, 3, 4])] {
        let h = 1.3;
        kuutio(
            [kx - h, kx + h],
            [0.0, 2.0 * h],
            [kz - h, kz + h],
            NOPPA_VARI,
            &mut tahkot,
        );
        let (ky, s, paksu, vali) = (h, 0.2, 0.06, 0.7);
        let [yla, px, mx, pz, mz] = silmat;
        for &(u, v) in SILMAT[yla - 1] {
            let (x, z) = (kx + u as f32 * vali, kz + v as f32 * vali);
            let y = 2.0 * h;
            kuutio(
                [x - s, x + s],
                [y, y + paksu],
                [z - s, z + s],
                SILMA,
                &mut tahkot,
            );
        }
        for (maara, suunta) in [(px, 1.0), (mx, -1.0)] {
            let x = kx + suunta * h;
            let xs = if suunta > 0.0 {
                [x, x + paksu]
            } else {
                [x - paksu, x]
            };
            for &(u, v) in SILMAT[maara - 1] {
                let (y, z) = (ky + u as f32 * vali, kz + v as f32 * vali);
                kuutio(xs, [y - s, y + s], [z - s, z + s], SILMA, &mut tahkot);
            }
        }
        for (maara, suunta) in [(pz, 1.0), (mz, -1.0)] {
            let z = kz + suunta * h;
            let zs = if suunta > 0.0 {
                [z, z + paksu]
            } else {
                [z - paksu, z]
            };
            for &(u, v) in SILMAT[maara - 1] {
                let (x, y) = (kx + u as f32 * vali, ky + v as f32 * vali);
                kuutio([x - s, x + s], [y - s, y + s], zs, SILMA, &mut tahkot);
            }
        }
    }
    tahkot
}

pub fn piirra_alku(peli: &Jatsi, kuva: &mut [u32]) {
    let mut p = Piirtaja {
        k: Kangas::new(kuva),
        kello: peli.kello_ms,
    };
    p.jatsin_alkuruutu(peli);
    p.aanikytkin(peli.aanet.paalla);
}

pub fn piirra(peli: &Jatsi, ruutu: &mut [u32]) {
    let mut p = Piirtaja {
        k: Kangas::kokoinen(ruutu, RUUDUN_LEVEYS, RUUDUN_KORKEUS),
        kello: peli.kello_ms,
    };
    p.jatsin_poyta(peli);
    p.aanikytkin(peli.aanet.paalla);
}

impl Piirtaja<'_> {
    fn jatsin_alkuruutu(&mut self, peli: &Jatsi) {
        self.maisema(kaanna("jatsi.banneri"), &noppamalli());
        let otsikko = listan_otsikko(peli.taulu.netti.is_some());
        self.parhaat_alkuruudulla(otsikko, peli.taulu.nakyva());
        let mut y = crate::yhteinen::piirto::OHJE_Y;
        for rivi in rivita(kaanna("jatsi.ohje_1"), ALKU_MERKIT) {
            self.teksti(48, y, rivi, LAVENTELI);
            y += RIVIN_KORKEUS;
        }
        if peli.jatkuu {
            self.kesken_jaanyt(peli);
            return;
        }
        self.teksti(48, PAIKAT_Y, kaanna("jatsi.pelaajat"), KELTAINEN);
        for i in 0..PELAAJIA {
            self.paikka(peli, i);
        }
        self.teksti(48, PAIKAN_OHJE_Y, kaanna("jatsi.paikka_ohje"), VALKOINEN);

        self.teksti(48, TASO_Y + 3, kaanna("yhteinen.taso"), KELTAINEN);
        for (taso, teksti, x) in TASOT {
            if taso == peli.vaikeus {
                self.nappi(x, TASO_Y, teksti, WB_ORANSSI, MUSTA);
            } else {
                self.nappi(x, TASO_Y, teksti, VALKOINEN, WB_SININEN);
            }
        }
        self.aloituskehote(ALOITUS);
    }

    fn paikka(&mut self, peli: &Jatsi, i: usize) {
        let y = paikan_y(i);
        let paikka = &peli.paikat[i];
        self.teksti(48, y + 3, &(i + 1).to_string(), KELTAINEN);
        let nimi = istujan_nimi(paikka.istuja);
        let tausta = match paikka.istuja {
            Istuja::Ihminen => VALKOINEN,
            Istuja::Kone => HUTI,
            Istuja::Tyhja => KANSI,
        };
        // Ensimmäinen paikka on aina ihmisen, joten se ei ole nappi.
        if i == 0 {
            self.kupera(ISTUJA_X, y, ISTUJA_L, NAPPI_K, tausta);
        } else {
            self.nappitausta(ISTUJA_X, y, ISTUJA_L, NAPPI_K, tausta);
        }
        let tx = ISTUJA_X + (ISTUJA_L - nimi.chars().count() as i32 * MERKKI) / 2;
        self.teksti(tx, y + 3, nimi, MUSTA);

        let kirjoitetaan = peli.nimetaan && peli.valittu_paikka == i;
        match paikka.istuja {
            Istuja::Ihminen if kirjoitetaan => {
                let kentta = format!("> {}", paikka.nimi);
                self.teksti(PAIKAN_NIMI_X, y + 3, &kentta, VALKOINEN);
                if self.vilkku() {
                    let x = PAIKAN_NIMI_X + kentta.chars().count() as i32 * MERKKI;
                    self.k.laatikko(x, y + 3, MERKKI, MERKKI, REUNA);
                }
            }
            Istuja::Ihminen if paikka.nimi.is_empty() => {
                self.teksti(PAIKAN_NIMI_X, y + 3, &Jatsi::oletusnimi(i), EHDOTUS);
            }
            Istuja::Ihminen => self.teksti(PAIKAN_NIMI_X, y + 3, &paikka.nimi, VALKOINEN),
            Istuja::Kone | Istuja::Tyhja => {}
        }
        if peli.valittu_paikka == i && !kirjoitetaan {
            let l = PAIKAN_NIMI_X + PAIKAN_NIMI_L - 42;
            self.valintakehys(42, y - 3, l, NAPPI_K + 6, KELTAINEN);
        }
    }

    fn kesken_jaanyt(&mut self, peli: &Jatsi) {
        self.teksti(48, PAIKAT_Y, kaanna("jatsi.kesken"), KELTAINEN);
        let summat = peli.poyta.summat();
        for (i, summa) in summat.iter().enumerate() {
            let y = PAIKKA_Y + i as i32 * 30;
            self.teksti(48, y, &peli.nimi(i), VALKOINEN);
            let teksti = summa.to_string();
            let x = 48 + 14 * MERKKI - teksti.chars().count() as i32 * MERKKI;
            self.teksti(x, y, &teksti, KELTAINEN);
        }
        let kierros = peli.poyta.kirja().kirjattuja() + 1;
        let teksti = muotoile("jatsi.kierros", &[&kierros]);
        self.teksti(48, PAIKAN_OHJE_Y, &teksti, VALKOINEN);
        self.teksti(48, UUSI_PELI_Y + 3, kaanna("jatsi.uusi_peli"), KELTAINEN);
        self.nappi(
            uuden_pelin_x(),
            UUSI_PELI_Y,
            UUSI_PELI_NAPPI,
            VALKOINEN,
            WB_SININEN,
        );
        self.aloituskehote("jatsi.jatka");
    }

    fn jatsin_poyta(&mut self, peli: &Jatsi) {
        self.pelitausta();
        let otsikko = kaanna("peli.jatsi").to_uppercase();
        let koneita = peli.poyta.koneet.iter().any(|&k| k);
        let taso = kaanna(match peli.vaikeus {
            Vaikeus::Helppo => "yhteinen.taso_1",
            Vaikeus::Normaali => "yhteinen.taso_2",
            Vaikeus::Vaikea => "yhteinen.taso_3",
        });
        self.otsikkopalkki(&otsikko, if koneita { taso } else { "" });
        self.poytakirja(peli);
        self.jatsin_sivu(peli);
        if let Some(alku) = peli.jatsi_ms {
            let kulunut = self.kello.saturating_sub(alku);
            if kulunut < JATSI_MS {
                self.jatsi_juhla(kulunut, peli.poyta.pelaajia());
            }
        }
    }

    fn poytakirja(&mut self, peli: &Jatsi) {
        let poyta = &peli.poyta;
        let n = poyta.pelaajia();
        let leveys = kirjan_leveys(n);
        let kx = kirja_x(n);
        self.ikkuna(kx - 18, KIRJA_Y - 12, leveys + 36, KIRJAN_K + 24);
        let pelissa = peli.vaihe == Vaihe::Pelissa;
        // Vuorossa olevan sarake on vaaleampi koko matkalta.
        if pelissa {
            let x = sarakkeen_x(n, poyta.vuorossa);
            self.k
                .laatikko(x, KIRJA_Y, SARAKE_L, KIRJAN_K, SARAKE_VUORO);
        }
        // Joka toinen rivi tummempi, jotta rivi on helppo seurata sarakkeiden yli.
        for i in (1..18).step_by(2) {
            let y = KIRJA_Y + OTSIKKO_K + i * RIVI_K;
            self.k.himmenna(kx, y, leveys, RIVI_K, 48);
        }
        for i in [0, 6, 8, 17] {
            let y = KIRJA_Y + OTSIKKO_K + i * RIVI_K - 4;
            self.k.laatikko(kx, y, leveys, 4, VIIVA);
        }

        for pelaaja in 0..n {
            let nimi = peli.nimi(pelaaja);
            let x = sarakkeen_x(n, pelaaja) + (SARAKE_L - nimi.chars().count() as i32 * MERKKI) / 2;
            let vari = if pelissa && pelaaja == poyta.vuorossa {
                KELTAINEN
            } else {
                VALKOINEN
            };
            self.teksti(x, KIRJA_Y + (OTSIKKO_K - 24) / 2, &nimi, vari);
        }

        let ihmisen_vuoro = pelissa && !poyta.kone_vuorossa();
        let ehdotukset = ihmisen_vuoro && poyta.heitetty > 0 && !peli.animoi();
        for rivi in Rivi::KAIKKI {
            let y = rivin_y(rivi);
            let vapaa = poyta.kirja().vapaa(rivi);
            let nimen_vari = if ihmisen_vuoro && peli.hiiren_rivi == Some(rivi) && vapaa {
                KELTAINEN
            } else {
                LAVENTELI
            };
            self.teksti(kx + 12, y + RIVIN_TEKSTI, rivin_nimi(rivi), nimen_vari);
            for pelaaja in 0..n {
                let kirja = &poyta.kirjat[pelaaja];
                if let Some(arvo) = kirja.arvo(rivi) {
                    let (teksti, vari) = if arvo > 0 {
                        (arvo.to_string(), VALKOINEN)
                    } else {
                        ("-".to_string(), KANSI)
                    };
                    self.solu(n, pelaaja, y, &teksti, vari);
                } else if ehdotukset && pelaaja == poyta.vuorossa {
                    let arvo = pisteet(rivi, &poyta.nopat);
                    let vari = if peli.viivaus == Some(rivi) {
                        WB_ORANSSI
                    } else {
                        EHDOTUS
                    };
                    self.solu(n, pelaaja, y, &arvo.to_string(), vari);
                }
            }
        }
        if ehdotukset && poyta.kirja().vapaa(peli.kursori) {
            let x = sarakkeen_x(n, poyta.vuorossa);
            let y = rivin_y(peli.kursori);
            self.valintakehys(x + 12, y + 4, SARAKE_L - 24, RIVI_K - 8, KELTAINEN);
        }

        for (paikka, avain) in [(6, "jatsi.ylaosa"), (7, "jatsi.bonus"), (17, "jatsi.summa")] {
            let y = KIRJA_Y + OTSIKKO_K + paikka * RIVI_K;
            self.teksti(kx + 12, y + RIVIN_TEKSTI, kaanna(avain), KELTAINEN);
            for pelaaja in 0..n {
                let kirja = &poyta.kirjat[pelaaja];
                let ylaosa_taynna = Rivi::KAIKKI[..6].iter().all(|&r| !kirja.vapaa(r));
                let (teksti, vari) = match paikka {
                    6 if kirja.ylaosa() >= BONUSRAJA => (kirja.ylaosa().to_string(), KELTAINEN),
                    6 => (kirja.ylaosa().to_string(), VALKOINEN),
                    7 if kirja.bonus() > 0 => (BONUS.to_string(), KELTAINEN),
                    7 if ylaosa_taynna => ("-".to_string(), KANSI),
                    // Puuttuvat pisteet bonukseen.
                    7 => (format!("-{}", BONUSRAJA - kirja.ylaosa()), EHDOTUS),
                    _ => (kirja.summa().to_string(), KELTAINEN),
                };
                self.solu(n, pelaaja, y, &teksti, vari);
            }
        }
    }

    /// Luku sarakkeen oikeaan laitaan.
    fn solu(&mut self, pelaajia: usize, pelaaja: usize, y: i32, teksti: &str, vari: u32) {
        let oikea = sarakkeen_x(pelaajia, pelaaja) + SARAKE_L - 24;
        let x = oikea - teksti.chars().count() as i32 * MERKKI;
        self.teksti(x, y + RIVIN_TEKSTI, teksti, vari);
    }

    fn jatsin_sivu(&mut self, peli: &Jatsi) {
        let poyta = &peli.poyta;
        self.ikkuna(SIVU_X - 12, 42, SIVU_L + 24, RUUDUN_KORKEUS as i32 - 66);
        let pelissa = peli.vaihe == Vaihe::Pelissa;
        let ihmisen_vuoro = pelissa && !poyta.kone_vuorossa();

        if pelissa {
            let otsikko = kaanna("jatsi.vuorossa_otsikko");
            self.suurteksti(SIVU_TEKSTI_X, VUORO_Y, otsikko, OTSIKKO, KELTAINEN, false);
            let nimi = peli.nimi(poyta.vuorossa);
            self.suurteksti(SIVU_TEKSTI_X, VUORO_Y + 30, &nimi, OTSIKKO, VALKOINEN, true);
        }
        self.nopat(peli, ihmisen_vuoro);

        let jaljella = HEITTOJA - poyta.heitetty;
        let voi = ihmisen_vuoro && poyta.voi_heittaa();
        let teksti = muotoile("jatsi.heita_nappi", &[&jaljella]);
        let tausta = if voi { WB_ORANSSI } else { KANSI };
        if voi {
            self.nappitausta(HEITTO_X, HEITTO_Y, HEITTO_L, HEITTO_K, tausta);
        } else {
            self.kupera(HEITTO_X, HEITTO_Y, HEITTO_L, HEITTO_K, tausta);
        }
        let x = HEITTO_X + (HEITTO_L - teksti.chars().count() as i32 * MERKKI) / 2;
        self.teksti(x, HEITTO_Y + (HEITTO_K - 24) / 2, &teksti, MUSTA);

        self.ikkuna(SIVU_X, VIESTI_Y, SIVU_L, VIESTI_K);
        if peli.vaihe == Vaihe::Loppu {
            self.jatsin_loppu(peli);
            return;
        }
        let vari = if peli.viivaus.is_some() {
            WB_ORANSSI
        } else {
            VALKOINEN
        };
        let mut y = VIESTI_Y + 15;
        for rivi in rivita(&peli.viesti, SIVU_MERKKEJA)
            .into_iter()
            .take(VIESTIN_RIVEJA)
        {
            self.teksti(SIVU_TEKSTI_X, y, rivi, vari);
            y += RIVIN_KORKEUS;
        }
        let ohjeet: &[&str] = if ihmisen_vuoro {
            &["jatsi.ohje_heitto", "jatsi.ohje_kirjaus"]
        } else {
            &["jatsi.hoputa"]
        };
        let mut y = OHJE_Y;
        for avain in ohjeet {
            for rivi in rivita(kaanna(avain), SIVU_MERKKEJA) {
                self.teksti(SIVU_TEKSTI_X, y, rivi, HUTI);
                y += RIVIN_KORKEUS;
            }
            y += RIVIN_KORKEUS / 2;
        }
        self.teksti(SIVU_TEKSTI_X, y, kaanna("yhteinen.takaisin"), HUTI);
    }

    fn jatsin_loppu(&mut self, peli: &Jatsi) {
        let mut y = VIESTI_Y + 12;
        let otsikko = peli.loppuotsikko();
        self.suurteksti(SIVU_TEKSTI_X, y, &otsikko, OTSIKKO, KELTAINEN, true);
        y += 36;
        self.teksti(SIVU_TEKSTI_X, y, kaanna("jatsi.listalle"), HUTI);
        y += RIVIN_KORKEUS;
        for &(pelaaja, pisteet) in peli.tulokset.iter().take(VIESTIN_RIVEJA - 2) {
            self.teksti(SIVU_TEKSTI_X, y, &peli.nimet[pelaaja], VALKOINEN);
            let teksti = pisteet.to_string();
            let x = SIVU_X + SIVU_L - 18 - teksti.chars().count() as i32 * MERKKI;
            self.teksti(x, y, &teksti, KELTAINEN);
            y += RIVIN_KORKEUS;
        }
        let parhaat_y = OHJE_Y;
        let otsikko = listan_otsikko(peli.taulu.netti.is_some());
        self.parhaat_lista(
            SIVU_TEKSTI_X,
            parhaat_y,
            SIVU_L - 36,
            TULOS,
            Some(PVM),
            otsikko,
            peli.taulu.nakyva(),
        );
        for (i, (_, avain)) in LOPPUNAPIT.iter().enumerate() {
            let y = NAPIT_Y + i as i32 * NAPPI_ASKEL;
            self.nappi(SIVU_TEKSTI_X, y, avain, VALKOINEN, WB_SININEN);
        }
    }

    fn nopat(&mut self, peli: &Jatsi, ihmisen_vuoro: bool) {
        let poyta = &peli.poyta;
        let alusta_x = NOPAT_X - 18;
        let alusta_l = 5 * NOPAN_ASKEL + 12;
        self.k
            .laatikko(alusta_x, ALUSTA_Y, alusta_l, ALUSTA_K, ALUSTA);
        self.upotus(alusta_x, ALUSTA_Y, alusta_l, ALUSTA_K);
        let pyorii = peli
            .heitto_ms
            .map(|alku| self.kello.saturating_sub(alku))
            .filter(|&kulunut| kulunut < HEITTO_MS);
        let painike = ihmisen_vuoro && poyta.voi_lukita();
        for i in 0..NOPPIA {
            let x = nopan_x(i);
            let lukittu = poyta.lukitut[i] && poyta.heitetty > 0;
            let numero = (i + 1).to_string();
            self.teksti(x + NOPPA / 2 - 12, ALUSTA_Y + 9, &numero, KELTAINEN);
            let (arvo, hyppy) = match pyorii {
                Some(kulunut) if !lukittu => {
                    let vaihe = kulunut / 70 + i as u32 * 3;
                    let hyppy = ((kulunut / 50 + i as u32) % 3) as i32 - 1;
                    ((vaihe % 6) as u8 + 1, hyppy * 8)
                }
                _ => (poyta.nopat[i], 0),
            };
            let y = NOPAT_Y + hyppy - if lukittu { NOUSU } else { 0 };
            self.noppa(x, y, arvo, poyta.heitetty == 0, painike);
            if lukittu {
                self.valintakehys(x, y, NOPPA, NOPPA, KELTAINEN);
            }
        }
    }

    /// Kupera noppa varjoineen. Ennen vuoron ensimmäistä heittoa edellinen heitto näkyy himmeänä.
    fn noppa(&mut self, x: i32, y: i32, arvo: u8, himmea: bool, painike: bool) {
        let vari = if himmea { NOPPA_HIMMEA } else { NOPPA_VARI };
        self.k.himmenna(x + 8, y + 8, NOPPA, NOPPA, 120);
        if painike {
            self.nappitausta(x, y, NOPPA, NOPPA, vari);
        } else {
            self.kupera(x, y, NOPPA, NOPPA, vari);
        }
        let silma = if arvo == 1 { SILMA_YKSI } else { SILMA };
        let sade = if arvo == 1 { 12 } else { 8 };
        let (kx, ky) = (x + NOPPA / 2, y + NOPPA / 2);
        for &(u, v) in SILMAT[(arvo.clamp(1, 6) - 1) as usize] {
            self.k.kiekko((kx + u * 26, ky + v * 26), sade, silma);
        }
    }

    /// JATSI! kasvaa esiin pöytäkirjan päälle ja säkenöi.
    fn jatsi_juhla(&mut self, kulunut: u32, pelaajia: usize) {
        let t = kulunut as f32 / JATSI_MS as f32;
        let teksti = kaanna("jatsi.juhla");
        let kasvu = (t * 5.0).min(1.0) * 2.0;
        let koko = (
            (SUURI.0 as f32 * kasvu) as i32,
            (SUURI.1 as f32 * kasvu) as i32,
            (SUURI.2 as f32 * kasvu) as i32,
        );
        if koko.0 < 4 {
            return;
        }
        let leveys = teksti.chars().count() as i32 * koko.2;
        let keski = (
            kirja_x(pelaajia) + kirjan_leveys(pelaajia) / 2,
            KIRJA_Y + KIRJAN_K / 2,
        );
        self.rajahdys(keski, t, 20, 640.0);
        let vari = if (kulunut / 120).is_multiple_of(2) {
            KELTAINEN
        } else {
            WB_ORANSSI
        };
        self.suurteksti(
            keski.0 - leveys / 2,
            keski.1 - koko.1 / 2,
            teksti,
            koko,
            vari,
            true,
        );
    }
}

#[cfg(test)]
mod tests;
