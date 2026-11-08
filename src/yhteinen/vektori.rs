//! Ohjelmistopiirron alkeet konepikselin ruudukossa: viivat, monikulmiot, spritet ja kevyt 3D-malli.

use std::f32::consts::TAU;

use super::paletti::varjoon;
use super::piirto::{KORKEUS, LEVEYS};

const SUURIN_LEIKKAUS: usize = 16;
const KAMERAN_ETAISYYS: f32 = 16.0;
const KALLISTUS: f32 = 0.24;
const VALO: [f32; 3] = [0.41, 0.82, -0.41];

pub struct Kangas<'a> {
    kuva: &'a mut [u32],
    pub leveys: usize,
    pub korkeus: usize,
    /// Konepikselin sivu: viivat, monikulmiot ja taustakuvat piirtyvät tähän ruudukkoon.
    pub pikseli: i32,
}

impl<'a> Kangas<'a> {
    /// Klassinen 960x768-kuva, johon vanhat näkymät piirtävät.
    pub fn new(kuva: &'a mut [u32]) -> Self {
        Self::kokoinen(kuva, LEVEYS, KORKEUS)
    }

    /// 960 pikselin kuvassa konepikseli on 3 (320x256), täydellä ruudulla 4 (480x270).
    pub fn kokoinen(kuva: &'a mut [u32], leveys: usize, korkeus: usize) -> Self {
        Self {
            kuva,
            leveys,
            korkeus,
            pikseli: if leveys > LEVEYS { 4 } else { 3 },
        }
    }
}

impl Kangas<'_> {
    pub fn laatikko(&mut self, x: i32, y: i32, l: i32, k: i32, vari: u32) {
        let (a, b) = (
            x.max(0) as usize,
            (x + l).clamp(0, self.leveys as i32) as usize,
        );
        if a >= b {
            return;
        }
        for yy in y.max(0)..(y + k).min(self.korkeus as i32) {
            let rivi = yy as usize * self.leveys;
            self.kuva[rivi + a..rivi + b].fill(vari);
        }
    }

    /// Läpikuultava varjo: alla oleva kuva tummuu, `osuus` 0..=256.
    pub fn himmenna(&mut self, x: i32, y: i32, l: i32, k: i32, osuus: u32) {
        let (a, b) = (
            x.max(0) as usize,
            (x + l).clamp(0, self.leveys as i32) as usize,
        );
        if a >= b {
            return;
        }
        for yy in y.max(0)..(y + k).min(self.korkeus as i32) {
            let rivi = yy as usize * self.leveys;
            for pikseli in &mut self.kuva[rivi + a..rivi + b] {
                *pikseli = varjoon(*pikseli, osuus);
            }
        }
    }

    /// Kehys konepikselin paksuisena.
    pub fn kehys(&mut self, x: i32, y: i32, l: i32, k: i32, vari: u32) {
        let p = self.pikseli;
        self.laatikko(x, y, l, p, vari);
        self.laatikko(x, y + k - p, l, p, vari);
        self.laatikko(x, y, p, k, vari);
        self.laatikko(x + l - p, y, p, k, vari);
    }

    pub fn piste(&mut self, x: i32, y: i32, vari: u32) {
        let p = self.pikseli;
        self.laatikko(x.div_euclid(p) * p, y.div_euclid(p) * p, p, p, vari);
    }

    /// Bresenham konepikselien ruudukossa.
    pub fn viiva(&mut self, alku: (i32, i32), loppu: (i32, i32), vari: u32) {
        let p = self.pikseli;
        let (mut x, mut y) = (alku.0.div_euclid(p), alku.1.div_euclid(p));
        let loppu = (loppu.0.div_euclid(p), loppu.1.div_euclid(p));
        let (dx, dy) = ((loppu.0 - x).abs(), -(loppu.1 - y).abs());
        let (askel_x, askel_y) = ((loppu.0 - x).signum(), (loppu.1 - y).signum());
        let mut virhe = dx + dy;
        loop {
            self.laatikko(x * p, y * p, p, p, vari);
            if (x, y) == loppu {
                return;
            }
            let kaksi = 2 * virhe;
            if kaksi >= dy {
                virhe += dy;
                x += askel_x;
            }
            if kaksi <= dx {
                virhe += dx;
                y += askel_y;
            }
        }
    }

    /// Täyttää monikulmion vaakaviivoin, parillis-pariton-säännöllä.
    pub fn monikulmio(&mut self, pisteet: &[(i32, i32)], vari: u32) {
        self.ruudutettu_monikulmio(pisteet, vari, vari);
    }

    /// Monikulmio, jonka konepikselit vuorottelevat kahta väriä: ditteröity välisävy.
    pub fn ruudutettu_monikulmio(&mut self, pisteet: &[(i32, i32)], a: u32, b: u32) {
        let p = self.pikseli;
        let ylin = pisteet.iter().map(|p| p.1).min().unwrap_or(0).max(0) / p;
        let alin = pisteet
            .iter()
            .map(|p| p.1)
            .max()
            .unwrap_or(0)
            .min(self.korkeus as i32 - 1)
            / p;
        for rivi in ylin..=alin {
            let y = rivi * p + p / 2;
            let mut leikkaukset = [0; SUURIN_LEIKKAUS];
            let mut maara = 0;
            for (i, &alku) in pisteet.iter().enumerate() {
                let loppu = pisteet[(i + 1) % pisteet.len()];
                if (alku.1 <= y) != (loppu.1 <= y) && maara < SUURIN_LEIKKAUS {
                    leikkaukset[maara] =
                        alku.0 + (y - alku.1) * (loppu.0 - alku.0) / (loppu.1 - alku.1);
                    maara += 1;
                }
            }
            leikkaukset[..maara].sort_unstable();
            for pari in leikkaukset[..maara].chunks_exact(2) {
                let (x0, x1) = (pari[0].div_euclid(p), pari[1].div_euclid(p));
                if a == b {
                    self.laatikko(x0 * p, rivi * p, (x1 - x0 + 1) * p, p, a);
                } else {
                    let viimeinen = x1.min(self.leveys as i32 / p);
                    for sarake in x0.max(0)..=viimeinen {
                        let vari = if (rivi + sarake) % 2 == 0 { a } else { b };
                        self.laatikko(sarake * p, rivi * p, p, p, vari);
                    }
                }
            }
        }
    }

    /// Ympyrän kehä säännöllisenä monikulmiona.
    pub fn ympyra(&mut self, keski: (i32, i32), sade: f32, vari: u32) {
        const SIVUJA: i32 = 16;
        let mut edellinen = kehapiste(keski, sade, 0.0);
        for i in 1..=SIVUJA {
            let seuraava = kehapiste(keski, sade, i as f32 / SIVUJA as f32 * TAU);
            self.viiva(edellinen, seuraava, vari);
            edellinen = seuraava;
        }
    }

    /// Täytetty ympyrä konepikselin riveinä.
    pub fn kiekko(&mut self, (x, y): (i32, i32), sade: i32, vari: u32) {
        if sade <= 0 {
            return;
        }
        let p = self.pikseli;
        let nelio = sade * sade;
        for rivi in (y - sade).div_euclid(p)..=(y + sade).div_euclid(p) {
            let dy = rivi * p + p / 2 - y;
            if dy * dy > nelio {
                continue;
            }
            let puoli = ((nelio - dy * dy) as f32).sqrt() as i32;
            let (x0, x1) = ((x - puoli).div_euclid(p), (x + puoli).div_euclid(p));
            self.laatikko(x0 * p, rivi * p, (x1 - x0 + 1) * p, p, vari);
        }
    }

    /// Valaistu pallo: tumma alareuna, perusväri ja kiilto vasemmalla ylhäällä.
    pub fn kiiltopallo(&mut self, (x, y): (i32, i32), sade: i32, vari: u32) {
        let p = self.pikseli;
        self.kiekko((x, y), sade, tummenna(vari, 0.55));
        self.kiekko((x - p / 2, y - p / 2), sade - p, vari);
        self.kiekko((x - sade / 3, y - sade / 3), sade / 2, vaalenna(vari, 0.45));
        self.laatikko(x - sade / 2, y - sade / 2, p, p, 0xFFFFFF);
    }

    /// Merkkijonoista koottu kuva. `vari` kertoo merkin värin, `None` on läpinäkyvä.
    pub fn sprite(
        &mut self,
        (x, y): (i32, i32),
        kuva: &[&str],
        kerroin: i32,
        vari: impl Fn(u8) -> Option<u32>,
    ) {
        for (rivi, merkit) in kuva.iter().enumerate() {
            for (sarake, merkki) in merkit.bytes().enumerate() {
                if let Some(vari) = vari(merkki) {
                    self.laatikko(
                        x + sarake as i32 * kerroin,
                        y + rivi as i32 * kerroin,
                        kerroin,
                        kerroin,
                        vari,
                    );
                }
            }
        }
    }

    /// Yksi taustakuvan rivi konepikselin kokoisena.
    pub fn karkea_rivi(&mut self, x: i32, y: i32, lahde: &[u32]) {
        let p = self.pikseli;
        let (ylin, alin) = (y.max(0), (y + p).min(self.korkeus as i32));
        if ylin >= alin {
            return;
        }
        let ohitus = if x < 0 { (-x + p - 1) / p } else { 0 } as usize;
        let vasen = x + ohitus as i32 * p;
        let oikea = (x + lahde.len() as i32 * p).min(self.leveys as i32);
        if vasen >= oikea || ohitus >= lahde.len() {
            return;
        }
        let (vasen, oikea) = (vasen as usize, oikea as usize);
        let ensimmainen = ylin as usize * self.leveys;
        let rivi = &mut self.kuva[ensimmainen + vasen..ensimmainen + oikea];
        match p {
            // Kiinteä pituus kääntyy suoriksi kopioinneiksi: taustat ovat piirron suurin työ.
            3 => venyta_rivi::<3>(rivi, &lahde[ohitus..]),
            4 => venyta_rivi::<4>(rivi, &lahde[ohitus..]),
            _ => {
                for (kohde, &vari) in rivi.chunks_mut(p as usize).zip(&lahde[ohitus..]) {
                    kohde.fill(vari);
                }
            }
        }
        for yy in ylin + 1..alin {
            let alku = yy as usize * self.leveys;
            self.kuva
                .copy_within(ensimmainen + vasen..ensimmainen + oikea, alku + vasen);
        }
    }

    /// Täyttö pystysuuntaisella liu'ulla: välisävyt ovat ruudutusta, kuten 16-bittisillä koneilla.
    pub fn liukulaatikko(&mut self, x: i32, y: i32, l: i32, k: i32, yla: u32, ala: u32) {
        const KAISTOJA: i32 = 6;
        let p = self.pikseli;
        let riveja = (k / p).max(1);
        for rivi in 0..riveja {
            let kohta = rivi * KAISTOJA * 3 / riveja;
            let kaista = kohta / 3;
            let a = sekoita_liuku(yla, ala, kaista as f32 / KAISTOJA as f32);
            let b = sekoita_liuku(yla, ala, (kaista + 1) as f32 / KAISTOJA as f32);
            let ry = y + rivi * p;
            let korkeus = if rivi + 1 == riveja { k - rivi * p } else { p };
            match kohta % 3 {
                0 => self.laatikko(x, ry, l, korkeus, a),
                2 => self.laatikko(x, ry, l, korkeus, b),
                _ => self.ruuturivi(x, ry, l, korkeus, (a, b), rivi),
            }
        }
    }

    fn ruuturivi(&mut self, x: i32, y: i32, l: i32, k: i32, (a, b): (u32, u32), vaihe: i32) {
        let p = self.pikseli as usize;
        let vasen = x.max(0) as usize;
        let oikea = (x + l).clamp(0, self.leveys as i32) as usize;
        if vasen >= oikea {
            return;
        }
        for yy in y.max(0)..(y + k).min(self.korkeus as i32) {
            let alku = yy as usize * self.leveys;
            let rivi = &mut self.kuva[alku + vasen..alku + oikea];
            for (i, pala) in rivi.chunks_mut(p).enumerate() {
                pala.fill(if (i as i32 + vaihe) % 2 == 0 { a } else { b });
            }
        }
    }

    /// Piirtää mallin kaukaisimmasta tahkosta lähimpään, jolloin kukin peittää taakseen
    /// jäävät. Tasaiset, ditteröidyt pinnat kuin Super FX -peleissä, ilman reunaviivoja.
    ///
    /// Malli rasteroidaan ensin alinäytteinä ja tiivistetään sitten konepikseleiksi: tahkon
    /// sisällä konepikseli ja ditteröinti pysyvät ennallaan, mutta reunan konepikseli saa
    /// peittävyyden mukaisen välisävyn. Näin porrasreunat pehmenevät ruudukosta luopumatta,
    /// eivätkä tahkojen väliset saumat päästä taustaa läpi.
    pub fn malli(&mut self, tahkot: &[Tahko], kulma: f32, keski: (i32, i32), mitta: f32) {
        let tahkot = mallin_tahkot(tahkot, kulma, keski, mitta);
        piirra_naytteina(self, &tahkot);
    }
}

/// Tahko ruudun koordinaatteina ja sen kaksi ditteröinnin väriä.
struct Ruututahko {
    pisteet: [(f32, f32); 5],
    maara: usize,
    a: u32,
    b: u32,
}

/// Mallin tahkot ruudulle käännettyinä, kaukaisimmasta lähimpään.
fn mallin_tahkot(tahkot: &[Tahko], kulma: f32, keski: (i32, i32), mitta: f32) -> Vec<Ruututahko> {
    let (sin_y, cos_y) = kulma.sin_cos();
    let (sin_x, cos_x) = KALLISTUS.sin_cos();
    let kierra = |p: [f32; 3]| {
        let x = p[0] * cos_y + p[2] * sin_y;
        let z = p[2] * cos_y - p[0] * sin_y;
        [x, p[1] * cos_x + z * sin_x, z * cos_x - p[1] * sin_x]
    };

    let mut jarjestys: Vec<(f32, &Tahko, [[f32; 3]; 5])> = tahkot
        .iter()
        .map(|tahko| {
            let kierretyt = tahko.pisteet.map(kierra);
            let n = tahko.maara as f32;
            let syvyys = kierretyt[..tahko.maara].iter().map(|p| p[2]).sum::<f32>() / n;
            (syvyys, tahko, kierretyt)
        })
        .collect();
    jarjestys.sort_by(|a, b| b.0.total_cmp(&a.0));

    jarjestys
        .into_iter()
        .map(|(_, tahko, kierretyt)| {
            let mut pisteet = [(0.0, 0.0); 5];
            for (kohde, p) in pisteet.iter_mut().zip(kierretyt) {
                let syvyys = mitta * KAMERAN_ETAISYYS / (p[2] + KAMERAN_ETAISYYS);
                *kohde = (
                    keski.0 as f32 + p[0] * syvyys,
                    keski.1 as f32 - p[1] * syvyys,
                );
            }
            let (a, b) = valaistus(&kierretyt);
            Ruututahko {
                pisteet,
                maara: tahko.maara,
                a: varjosta(tahko.vari, a),
                b: varjosta(tahko.vari, b),
            }
        })
        .collect()
}

/// Alinäytteitä konepikselin sivulla.
const ALINAYTTEITA: usize = 3;
/// Näyte, johon mikään tahko ei osunut. Värit ovat 24-bittisiä, joten tämä ei ole väri.
const EI_NAYTETTA: u32 = u32::MAX;

/// Rasteroi tahkot alinäytteiksi mallin ympärille ja tiivistää ne konepikseleiksi:
/// kokonaan peitetty konepikseli saa näytteidensä värin, reunan konepikseli sekoittuu
/// taustaan peittävyyden mukaan.
fn piirra_naytteina(kangas: &mut Kangas, tahkot: &[Ruututahko]) {
    let pisteet = || tahkot.iter().flat_map(|t| &t.pisteet[..t.maara]);
    let Some(vasen) = pisteet().map(|p| p.0).reduce(f32::min) else {
        return;
    };
    let oikea = pisteet().map(|p| p.0).fold(vasen, f32::max);
    let yla = pisteet().map(|p| p.1).fold(f32::MAX, f32::min);
    let ala = pisteet().map(|p| p.1).fold(f32::MIN, f32::max);

    let p = kangas.pikseli;
    let (sarakkeita, riveja) = (kangas.leveys as i32 / p, kangas.korkeus as i32 / p);
    // Rajaus konepikseleinä, kankaan sisällä.
    let alku_x = (vasen.floor() as i32).div_euclid(p).max(0);
    let alku_y = (yla.floor() as i32).div_euclid(p).max(0);
    let loppu_x = (oikea.ceil() as i32).div_euclid(p).min(sarakkeita - 1);
    let loppu_y = (ala.ceil() as i32).div_euclid(p).min(riveja - 1);
    if alku_x > loppu_x || alku_y > loppu_y {
        return;
    }
    let s = ALINAYTTEITA;
    let leveys = (loppu_x - alku_x + 1) as usize * s;
    let korkeus = (loppu_y - alku_y + 1) as usize * s;
    let mut naytteet = vec![EI_NAYTETTA; leveys * korkeus];
    // Kunkin näyterivin täytetyt sarakkeet: tiivistys ohittaa rajauksen tyhjät kulmat.
    let mut rajat = vec![(usize::MAX, 0); korkeus];
    let koko = p as f32 / s as f32;
    let (nolla_x, nolla_y) = ((alku_x * p) as f32, (alku_y * p) as f32);

    for tahko in tahkot {
        let mut naytepisteet = [(0.0, 0.0); 5];
        for (kohde, &(x, y)) in naytepisteet.iter_mut().zip(&tahko.pisteet) {
            *kohde = ((x - nolla_x) / koko, (y - nolla_y) / koko);
        }
        tayta(
            &naytepisteet[..tahko.maara],
            leveys,
            korkeus,
            |rivi, vasen, oikea| {
                let raja = &mut rajat[rivi];
                *raja = (raja.0.min(vasen), raja.1.max(oikea));
                let rivin = &mut naytteet[rivi * leveys..(rivi + 1) * leveys];
                if tahko.a == tahko.b {
                    rivin[vasen..oikea].fill(tahko.a);
                    return;
                }
                // Ditteröinnin ruudutus kulkee konepikselin mukaan kuten ennenkin.
                let konerivi = alku_y + (rivi / s) as i32;
                let mut x = vasen;
                while x < oikea {
                    let palan_loppu = ((x / s + 1) * s).min(oikea);
                    let sarake = alku_x + (x / s) as i32;
                    let vari = if (konerivi + sarake) % 2 == 0 {
                        tahko.a
                    } else {
                        tahko.b
                    };
                    rivin[x..palan_loppu].fill(vari);
                    x = palan_loppu;
                }
            },
        );
    }

    let kaikki = (s * s) as u32;
    let pu = p as usize;
    for rivi in 0..korkeus / s {
        let ylin = rivi * s;
        let (vasen, oikea) = rajat[ylin..ylin + s]
            .iter()
            .fold((usize::MAX, 0), |a, r| (a.0.min(r.0), a.1.max(r.1)));
        if vasen >= oikea {
            continue;
        }
        let rivit: [&[u32]; ALINAYTTEITA] =
            std::array::from_fn(|i| &naytteet[(ylin + i) * leveys..(ylin + i + 1) * leveys]);
        let kuvan_y = (alku_y as usize + rivi) * pu;
        for sarake in vasen / s..=(oikea - 1) / s {
            let alku = sarake * s;
            let ensimmainen = rivit[0][alku];
            let yhtenainen = rivit
                .iter()
                .all(|r| r[alku..alku + s].iter().all(|&vari| vari == ensimmainen));
            let kuvan_x = (alku_x as usize + sarake) * pu;
            let vari = if yhtenainen {
                // Tahkon sisus tai tyhjä tausta: ei laskettavaa.
                if ensimmainen == EI_NAYTETTA {
                    continue;
                }
                ensimmainen
            } else {
                let (mut r, mut g, mut b, mut osuneet) = (0, 0, 0, 0);
                for &vari in rivit.iter().flat_map(|r| &r[alku..alku + s]) {
                    if vari != EI_NAYTETTA {
                        r += vari >> 16 & 0xFF;
                        g += vari >> 8 & 0xFF;
                        b += vari & 0xFF;
                        osuneet += 1;
                    }
                }
                // Reunalla malli peittää vain osan konepikselistä: loput on taustaa.
                let tausta = kangas.kuva[kuvan_y * kangas.leveys + kuvan_x];
                let loput = kaikki - osuneet;
                let kanava =
                    |summa: u32, siirto: u32| (summa + (tausta >> siirto & 0xFF) * loput) / kaikki;
                (kanava(r, 16) << 16) | (kanava(g, 8) << 8) | kanava(b, 0)
            };
            for y in kuvan_y..kuvan_y + pu {
                let i = y * kangas.leveys + kuvan_x;
                kangas.kuva[i..i + pu].fill(vari);
            }
        }
    }
}

/// Täyttää monikulmion näytteet, joiden keskipiste on sen sisällä (parillis-pariton-sääntö).
/// Pisteet ovat näytteinä, ja `aseta` saa rivin ja sen sarakkeet `vasen..oikea`.
fn tayta(
    pisteet: &[(f32, f32)],
    leveys: usize,
    korkeus: usize,
    mut aseta: impl FnMut(usize, usize, usize),
) {
    let ylin = pisteet.iter().map(|p| p.1).fold(f32::MAX, f32::min);
    let alin = pisteet.iter().map(|p| p.1).fold(f32::MIN, f32::max);
    let ensimmainen = (ylin - 0.5).ceil().max(0.0) as usize;
    let viimeinen = ((alin - 0.5).floor() as i64).min(korkeus as i64 - 1);
    if viimeinen < ensimmainen as i64 {
        return;
    }
    for rivi in ensimmainen..=viimeinen as usize {
        let y = rivi as f32 + 0.5;
        let mut leikkaukset = [0.0f32; SUURIN_LEIKKAUS];
        let mut maara = 0;
        for (i, &alku) in pisteet.iter().enumerate() {
            let loppu = pisteet[(i + 1) % pisteet.len()];
            if (alku.1 <= y) != (loppu.1 <= y) && maara < SUURIN_LEIKKAUS {
                leikkaukset[maara] =
                    alku.0 + (y - alku.1) * (loppu.0 - alku.0) / (loppu.1 - alku.1);
                maara += 1;
            }
        }
        leikkaukset[..maara].sort_unstable_by(f32::total_cmp);
        for pari in leikkaukset[..maara].chunks_exact(2) {
            // Sarakkeet, joiden keskipiste on välillä [vasen, oikea).
            let vasen = ((pari[0] - 0.5).ceil().max(0.0) as usize).min(leveys);
            let oikea = ((pari[1] - 0.5).ceil().max(0.0) as usize).min(leveys);
            if vasen < oikea {
                aseta(rivi, vasen, oikea);
            }
        }
    }
}

fn venyta_rivi<const P: usize>(rivi: &mut [u32], lahde: &[u32]) {
    let mut palat = rivi.chunks_exact_mut(P);
    let mut varit = lahde.iter();
    for (kohde, &vari) in (&mut palat).zip(&mut varit) {
        kohde.copy_from_slice(&[vari; P]);
    }
    if let Some(&vari) = varit.next() {
        palat.into_remainder().fill(vari);
    }
}

pub fn kehapiste(keski: (i32, i32), sade: f32, kulma: f32) -> (i32, i32) {
    let (sin, cos) = kulma.sin_cos();
    (keski.0 + (cos * sade) as i32, keski.1 + (sin * sade) as i32)
}

/// Väri kerrottuna kertoimella, jolla kukin kanava tummenee.
pub fn varjosta(vari: u32, kerroin: f32) -> u32 {
    let kanava = |siirto: u32| (((vari >> siirto & 0xFF) as f32 * kerroin) as u32).min(255);
    kanava(16) << 16 | kanava(8) << 8 | kanava(0)
}

/// Sekoitus kohti valkoista.
pub fn vaalenna(vari: u32, osuus: f32) -> u32 {
    sekoita_liuku(vari, 0xFFFFFF, osuus)
}

/// Varjo kallistuu violettiin, kuten 16-bittisten pelien väriramppien pohjalla.
pub fn tummenna(vari: u32, osuus: f32) -> u32 {
    sekoita_liuku(vari, 0x14082C, osuus)
}

pub fn sekoita_liuku(a: u32, b: u32, osuus: f32) -> u32 {
    let kanava = |siirto: u32| {
        let (x, y) = ((a >> siirto & 0xFF) as f32, (b >> siirto & 0xFF) as f32);
        ((x + (y - x) * osuus) as u32) << siirto
    };
    kanava(16) | kanava(8) | kanava(0)
}

/// Pinnan kaksi sävyä: sama väri on tasainen pinta, eri värit tarkoittavat ruudutettua välisävyä.
fn valaistus(p: &[[f32; 3]; 5]) -> (f32, f32) {
    let a = [p[1][0] - p[0][0], p[1][1] - p[0][1], p[1][2] - p[0][2]];
    let b = [p[2][0] - p[0][0], p[2][1] - p[0][1], p[2][2] - p[0][2]];
    let n = [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ];
    let pituus = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(0.001);
    let osuvuus = (n[0] * VALO[0] + n[1] * VALO[1] + n[2] * VALO[2]).abs() / pituus;
    let taso = osuvuus * 3.0;
    let (alempi, osuus) = (taso.floor(), taso.fract());
    let (ala, yla) = (0.45 + alempi * 0.2, 0.45 + (alempi + 1.0) * 0.2);
    if osuus < 0.34 {
        (ala, ala)
    } else if osuus > 0.66 {
        (yla, yla)
    } else {
        (ala, yla)
    }
}

#[derive(Clone, Copy)]
pub struct Tahko {
    pisteet: [[f32; 3]; 5],
    maara: usize,
    vari: u32,
}

pub fn tahko4(a: [f32; 3], b: [f32; 3], c: [f32; 3], d: [f32; 3], vari: u32) -> Tahko {
    Tahko {
        pisteet: [a, b, c, d, d],
        maara: 4,
        vari,
    }
}

/// Laatikon kyljet ja katto, pohjaa ei koskaan näy.
pub fn kuutio(x: [f32; 2], y: [f32; 2], z: [f32; 2], vari: u32, tahkot: &mut Vec<Tahko>) {
    let p = |i: usize, j: usize, k: usize| [x[i], y[j], z[k]];
    tahkot.push(tahko4(p(0, 1, 0), p(1, 1, 0), p(1, 1, 1), p(0, 1, 1), vari));
    tahkot.push(tahko4(p(0, 0, 0), p(1, 0, 0), p(1, 1, 0), p(0, 1, 0), vari));
    tahkot.push(tahko4(p(0, 0, 1), p(1, 0, 1), p(1, 1, 1), p(0, 1, 1), vari));
    tahkot.push(tahko4(p(0, 0, 0), p(0, 0, 1), p(0, 1, 1), p(0, 1, 0), vari));
    tahkot.push(tahko4(p(1, 0, 0), p(1, 0, 1), p(1, 1, 1), p(1, 1, 0), vari));
}

/// Alkuruudun taistelulaiva. Keula osoittaa +x-suuntaan, vesiraja on y = 0.
pub fn taistelulaiva(runko: u32, kansi: u32, torni: u32) -> Vec<Tahko> {
    let kansi_p = [
        [-5.0, 0.7, -0.8],
        [-5.0, 0.7, 0.8],
        [2.0, 0.7, 1.1],
        [5.5, 0.9, 0.0],
        [2.0, 0.7, -1.1],
    ];
    let vesi_p = [
        [-4.6, 0.0, -0.6],
        [-4.6, 0.0, 0.6],
        [2.0, 0.0, 0.9],
        [4.8, 0.0, 0.0],
        [2.0, 0.0, -0.9],
    ];
    let mut tahkot = vec![Tahko {
        pisteet: kansi_p,
        maara: 5,
        vari: kansi,
    }];
    for i in 0..5 {
        let j = (i + 1) % 5;
        tahkot.push(tahko4(kansi_p[i], kansi_p[j], vesi_p[j], vesi_p[i], runko));
    }
    // komentosilta, masto ja savupiippu
    kuutio([-0.8, 1.0], [0.7, 1.7], [-0.55, 0.55], torni, &mut tahkot);
    kuutio([-0.3, 0.7], [1.7, 2.4], [-0.35, 0.35], torni, &mut tahkot);
    kuutio([0.1, 0.25], [2.4, 3.3], [-0.07, 0.07], kansi, &mut tahkot);
    kuutio([-2.0, -1.4], [0.7, 1.6], [-0.3, 0.3], runko, &mut tahkot);
    // tykkitornit putkineen
    for (x, suunta) in [(2.2, 1.0), (-3.4, -1.0)] {
        kuutio(
            [x - 0.45, x + 0.45],
            [0.7, 1.1],
            [-0.4, 0.4],
            torni,
            &mut tahkot,
        );
        let putki_x = if suunta > 0.0 {
            [x + 0.4, x + 1.7]
        } else {
            [x - 1.7, x - 0.4]
        };
        kuutio(putki_x, [0.85, 0.97], [-0.06, 0.06], kansi, &mut tahkot);
    }
    tahkot
}

#[cfg(test)]
mod tests;
