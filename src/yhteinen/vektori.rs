//! Ohjelmistopiirron alkeet: viivat, monikulmiot, spritet ja kevyt 3D-malli. Muodot ovat
//! konepikselin mittaisia, mutta niiden reunat pehmennetään ruudun pikseleihin asti.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::paletti::varjoon;
use super::pehmennys::{Peitto, pehmenna};
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

    /// Sekoittaa värin pikseliin, `alfa` 0..=255. Kuvan ulkopuoli ohitetaan.
    pub fn sekoita(&mut self, x: i32, y: i32, vari: u32, alfa: u32) {
        if x < 0 || y < 0 || x >= self.leveys as i32 || y >= self.korkeus as i32 || alfa == 0 {
            return;
        }
        let pikseli = &mut self.kuva[y as usize * self.leveys + x as usize];
        *pikseli = sekoita_alfa(*pikseli, vari, alfa);
    }

    /// Käy läpi peittokuvan näkyvät pikselit kohdassa `(x, y)`: `kasittele` saa kuvan
    /// pikselin, peiton värin ja peittävyyden sekä peiton rivin.
    fn peiton_pikselit(
        &mut self,
        (x, y): (i32, i32),
        peitto: &Peitto,
        mut kasittele: impl FnMut(&mut u32, u32, u32, usize),
    ) {
        let vasen = (-x).max(0) as usize;
        let oikea = (self.leveys as i32 - x).clamp(0, peitto.leveys as i32) as usize;
        if vasen >= oikea {
            return;
        }
        for rivi in
            (-y).max(0) as usize..(self.korkeus as i32 - y).clamp(0, peitto.korkeus as i32) as usize
        {
            let (nakyva_alku, nakyva_loppu) = peitto.valit[rivi];
            let (vasen, oikea) = (vasen.max(nakyva_alku), oikea.min(nakyva_loppu));
            if vasen >= oikea {
                continue;
            }
            let lahde = &peitto.pikselit[rivi * peitto.leveys..(rivi + 1) * peitto.leveys];
            let alku = (y + rivi as i32) as usize * self.leveys + (x + vasen as i32) as usize;
            let kohde = &mut self.kuva[alku..alku + oikea - vasen];
            for (pikseli, &(vari, alfa)) in kohde.iter_mut().zip(&lahde[vasen..oikea]) {
                if alfa != 0 {
                    kasittele(pikseli, vari, alfa as u32, rivi);
                }
            }
        }
    }

    /// Peittokuva omin värein.
    pub fn peitto(&mut self, kohta: (i32, i32), peitto: &Peitto) {
        self.peiton_pikselit(kohta, peitto, |pikseli, vari, alfa, _| {
            *pikseli = sekoita_alfa(*pikseli, vari, alfa);
        });
    }

    /// Peittokuvan muotoinen läpikuultava varjo, `osuus` 0..=256 kuten `himmenna`.
    pub fn peiton_varjo(&mut self, kohta: (i32, i32), peitto: &Peitto, osuus: u32) {
        self.peiton_pikselit(kohta, peitto, |pikseli, _, alfa, _| {
            *pikseli = varjoon(*pikseli, osuus * alfa / 255);
        });
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

    /// Konepikselin paksuinen viiva pyöreine päineen. Reunat sekoittuvat taustaan
    /// peittävyyden mukaan, joten vinokaan viiva ei porrastu.
    pub fn viiva(&mut self, alku: (i32, i32), loppu: (i32, i32), vari: u32) {
        let puoli = self.pikseli as f32 * 0.5;
        // Pisteet ovat pikselien keskipisteitä.
        let (ax, ay) = (alku.0 as f32 + 0.5, alku.1 as f32 + 0.5);
        let (bx, by) = (loppu.0 as f32 + 0.5, loppu.1 as f32 + 0.5);
        let (dx, dy) = (bx - ax, by - ay);
        let pituus2 = dx * dx + dy * dy;
        let ulottuma = puoli + 0.5;
        let ulottuma2 = ulottuma * ulottuma;
        let sisa2 = (ulottuma - 1.0).max(0.0).powi(2);
        let ylin = (ay.min(by) - ulottuma).floor().max(0.0) as i32;
        let alin = ((ay.max(by) + ulottuma).ceil() as i32).min(self.korkeus as i32 - 1);
        let vasen = (ax.min(bx) - ulottuma).floor().max(0.0) as i32;
        let oikea = ((ax.max(bx) + ulottuma).ceil() as i32).min(self.leveys as i32 - 1);
        for y in ylin..=alin {
            let py = y as f32 + 0.5;
            // Rivin osuus, jolla viivan kaista kulkee: loivalla viivalla koko rajaus.
            let (mut x0, mut x1) = (vasen, oikea);
            if dy.abs() > 0.001 {
                let keski = ax + (py - ay) * dx / dy;
                let leveys = ulottuma * pituus2.sqrt() / dy.abs() + 1.0;
                x0 = x0.max((keski - leveys).floor() as i32);
                x1 = x1.min((keski + leveys).ceil() as i32);
            }
            let rivi = y as usize * self.leveys;
            for x in x0.max(0)..=x1.min(self.leveys as i32 - 1) {
                let px = x as f32 + 0.5;
                let t = if pituus2 > 0.0 {
                    (((px - ax) * dx + (py - ay) * dy) / pituus2).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                let (ex, ey) = (px - ax - t * dx, py - ay - t * dy);
                let etaisyys2 = ex * ex + ey * ey;
                // Neliöjuuri vain reunapikseleille: sisä- ja ulkopuoli tunnistetaan neliöistä.
                let pikseli = &mut self.kuva[rivi + x as usize];
                if etaisyys2 <= sisa2 {
                    *pikseli = vari;
                } else if etaisyys2 < ulottuma2 {
                    let peitto = (ulottuma - etaisyys2.sqrt()).clamp(0.0, 1.0);
                    *pikseli = sekoita_alfa(*pikseli, vari, (peitto * 255.0 + 0.5) as u32);
                }
            }
        }
    }

    /// Täyttää monikulmion parillis-pariton-säännöllä. Reunapikselit saavat
    /// peittävyytensä mukaisen välisävyn, joten vinot reunat eivät porrastu. Kukin rivi
    /// näytteistetään neljänä alirivinä; pikselit, jotka kaikki alirivit peittävät
    /// kokonaan, täytetään suoraan, ja vain reunat lasketaan.
    pub fn monikulmio(&mut self, pisteet: &[(i32, i32)], vari: u32) {
        const ALIRIVEJA: usize = 4;
        if pisteet.len() < 3 {
            return;
        }
        let ylin = pisteet.iter().map(|p| p.1).min().unwrap_or(0).max(0);
        let alin = pisteet
            .iter()
            .map(|p| p.1)
            .max()
            .unwrap_or(0)
            .min(self.korkeus as i32 - 1);
        let leveys = self.leveys as f32;
        // Vaakasuorat särmät eivät leikkaa yhtään alirivin keskiviivaa, joten ne jäävät pois.
        let mut sarmat = [(0.0f32, 0.0f32, 0.0f32, 0.0f32); SUURIN_LEIKKAUS * 2];
        let mut sarmia = 0;
        for (i, &alku) in pisteet.iter().enumerate() {
            let loppu = pisteet[(i + 1) % pisteet.len()];
            if alku.1 == loppu.1 || sarmia == sarmat.len() {
                continue;
            }
            let (ay, by) = (alku.1 as f32, loppu.1 as f32);
            let kerroin = (loppu.0 - alku.0) as f32 / (by - ay);
            sarmat[sarmia] = (ay, by, alku.0 as f32, kerroin);
            sarmia += 1;
        }
        let sarmat = &sarmat[..sarmia];
        let mut aktiiviset = [(0.0f32, 0.0f32, 0.0f32, 0.0f32); SUURIN_LEIKKAUS * 2];
        for y in ylin..=alin {
            // Rivin aktiiviset särmät: alirivit testaavat vain nämä.
            let (yla, ala) = (y as f32, y as f32 + 1.0);
            let mut n_aktiivisia = 0;
            for &sarma in sarmat {
                if sarma.0.min(sarma.1) < ala && sarma.0.max(sarma.1) > yla {
                    aktiiviset[n_aktiivisia] = sarma;
                    n_aktiivisia += 1;
                }
            }
            if n_aktiivisia == 0 {
                continue;
            }
            let aktiiviset = &aktiiviset[..n_aktiivisia];
            // Kunkin alirivin välit, kuvan leveyteen rajattuina.
            let mut valit = [[(0.0f32, 0.0f32); SUURIN_LEIKKAUS / 2]; ALIRIVEJA];
            let mut maarat = [0; ALIRIVEJA];
            for (ali, (alin_valit, maara)) in valit.iter_mut().zip(&mut maarat).enumerate() {
                let ry = y as f32 + (ali as f32 + 0.5) / ALIRIVEJA as f32;
                let mut leikkaukset = [0.0f32; SUURIN_LEIKKAUS];
                let mut n = 0;
                for &(ay, by, ax, kerroin) in aktiiviset {
                    if (ay <= ry) != (by <= ry) && n < SUURIN_LEIKKAUS {
                        leikkaukset[n] = ax + (ry - ay) * kerroin;
                        n += 1;
                    }
                }
                leikkaukset[..n].sort_unstable_by(f32::total_cmp);
                for pari in leikkaukset[..n].chunks_exact(2) {
                    let (a, b) = (pari[0].max(0.0), pari[1].min(leveys));
                    if a < b {
                        alin_valit[*maara] = (a, b);
                        *maara += 1;
                    }
                }
            }
            if maarat.iter().all(|&m| m == 0) {
                continue;
            }
            let rivi = y as usize * self.leveys;
            // Tavallinen kupera tapaus: yksi väli joka alirivillä. Sisäosa täytetään suoraan.
            if maarat.iter().all(|&m| m == 1) {
                let sisa_a = valit.iter().map(|v| v[0].0).fold(f32::MIN, f32::max).ceil();
                let sisa_b = valit
                    .iter()
                    .map(|v| v[0].1)
                    .fold(f32::MAX, f32::min)
                    .floor();
                let ulko_a = valit
                    .iter()
                    .map(|v| v[0].0)
                    .fold(f32::MAX, f32::min)
                    .floor();
                let ulko_b = valit.iter().map(|v| v[0].1).fold(f32::MIN, f32::max).ceil();
                if sisa_a < sisa_b {
                    let (sa, sb) = (sisa_a as usize, sisa_b as usize);
                    self.kuva[rivi + sa..rivi + sb].fill(vari);
                    self.reunapikselit(rivi, (ulko_a as usize, sa), &valit, vari);
                    self.reunapikselit(rivi, (sb, ulko_b as usize), &valit, vari);
                } else {
                    self.reunapikselit(rivi, (ulko_a as usize, ulko_b as usize), &valit, vari);
                }
                continue;
            }
            let ulko_a = valit
                .iter()
                .zip(&maarat)
                .flat_map(|(v, &m)| &v[..m])
                .map(|v| v.0)
                .fold(f32::MAX, f32::min)
                .floor();
            let ulko_b = valit
                .iter()
                .zip(&maarat)
                .flat_map(|(v, &m)| &v[..m])
                .map(|v| v.1)
                .fold(f32::MIN, f32::max)
                .ceil();
            for x in ulko_a as usize..(ulko_b as usize).min(self.leveys) {
                let (pa, pb) = (x as f32, x as f32 + 1.0);
                let peitto: f32 = valit
                    .iter()
                    .zip(&maarat)
                    .flat_map(|(v, &m)| &v[..m])
                    .map(|&(a, b)| (b.min(pb) - a.max(pa)).max(0.0))
                    .sum();
                let alfa = (peitto / ALIRIVEJA as f32 * 255.0 + 0.5) as u32;
                if alfa > 0 {
                    let pikseli = &mut self.kuva[rivi + x];
                    *pikseli = sekoita_alfa(*pikseli, vari, alfa.min(255));
                }
            }
        }
    }

    /// Kuperan monikulmion rivin reunapikselit `alku..loppu`: kunkin alirivin yksi väli.
    fn reunapikselit(
        &mut self,
        rivi: usize,
        (alku, loppu): (usize, usize),
        valit: &[[(f32, f32); SUURIN_LEIKKAUS / 2]],
        vari: u32,
    ) {
        for x in alku..loppu.min(self.leveys) {
            let (pa, pb) = (x as f32, x as f32 + 1.0);
            let peitto: f32 = valit
                .iter()
                .map(|v| (v[0].1.min(pb) - v[0].0.max(pa)).max(0.0))
                .sum();
            let alfa = (peitto / valit.len() as f32 * 255.0 + 0.5) as u32;
            if alfa > 0 {
                let pikseli = &mut self.kuva[rivi + x];
                *pikseli = sekoita_alfa(*pikseli, vari, alfa.min(255));
            }
        }
    }

    /// Ympyrän kehä konepikselin paksuisena, reunat pehmennettyinä.
    pub fn ympyra(&mut self, keski: (i32, i32), sade: f32, vari: u32) {
        let puoli = self.pikseli as f32 * 0.5;
        let peitto = |etaisyys: f32| puoli + 0.5 - (etaisyys - sade).abs();
        self.ympyran_alue(keski, sade + puoli, None, peitto, vari);
    }

    /// Täytetty ympyrä, reunat pehmennettyinä.
    pub fn kiekko(&mut self, keski: (i32, i32), sade: i32, vari: u32) {
        if sade <= 0 {
            return;
        }
        let sade = sade as f32;
        let taysi = Some(sade - 0.5);
        self.ympyran_alue(keski, sade, taysi, |etaisyys| sade + 0.5 - etaisyys, vari);
    }

    /// Käy läpi ympyrän `(keski, ulottuma)` pikselit: `peitto(etäisyys)` antaa kunkin
    /// pikselin peittävyyden 0..1 sen keskipisteen etäisyydestä. Pikselit, joiden keskipiste
    /// on lähempänä kuin `taysi`, täytetään laskematta.
    fn ympyran_alue(
        &mut self,
        (x, y): (i32, i32),
        ulottuma: f32,
        taysi: Option<f32>,
        peitto: impl Fn(f32) -> f32,
        vari: u32,
    ) {
        let (kx, ky) = (x as f32, y as f32);
        let r = ulottuma + 1.0;
        let ylin = ((ky - r).floor() as i32).max(0);
        let alin = ((ky + r).ceil() as i32).min(self.korkeus as i32 - 1);
        for py in ylin..=alin {
            let dy = py as f32 + 0.5 - ky;
            if dy.abs() > r {
                continue;
            }
            let puoli = (r * r - dy * dy).sqrt();
            let x0 = ((kx - puoli).floor() as i32).max(0);
            let x1 = ((kx + puoli).ceil() as i32).min(self.leveys as i32 - 1);
            // Täysin peitetty keskiosa: keskipisteet välillä kx ± sisa.
            let sisa = taysi
                .filter(|&t| t > dy.abs())
                .map(|t| (t * t - dy * dy).sqrt());
            let (s0, s1) = match sisa {
                Some(sisa) => (
                    ((kx - sisa - 0.5).ceil() as i32).max(x0),
                    ((kx + sisa - 0.5).floor() as i32 + 1).min(x1 + 1),
                ),
                None => (x1 + 1, x1 + 1),
            };
            if s0 < s1 {
                let rivi = py as usize * self.leveys;
                self.kuva[rivi + s0 as usize..rivi + s1 as usize].fill(vari);
            }
            for px in (x0..s0.min(x1 + 1)).chain(s1.max(x0)..=x1) {
                let dx = px as f32 + 0.5 - kx;
                let osuus = peitto((dx * dx + dy * dy).sqrt()).clamp(0.0, 1.0);
                if osuus > 0.0 {
                    self.sekoita(px, py, vari, (osuus * 255.0 + 0.5) as u32);
                }
            }
        }
    }

    /// Valaistu pallo: tumma alareuna, perusväri ja kiilto vasemmalla ylhäällä.
    pub fn kiiltopallo(&mut self, (x, y): (i32, i32), sade: i32, vari: u32) {
        let p = self.pikseli;
        self.kiekko((x, y), sade, tummenna(vari, 0.55));
        self.kiekko((x - p / 2, y - p / 2), sade - p, vari);
        self.kiekko((x - sade / 3, y - sade / 3), sade / 2, vaalenna(vari, 0.45));
        let kiilto = (x - sade / 2 + p / 2, y - sade / 2 + p / 2);
        self.kiekko(kiilto, (p + 1) / 2, 0xFFFFFF);
    }

    /// Merkkijonoista koottu kuva. `vari` kertoo merkin värin, `None` on läpinäkyvä.
    pub fn sprite(
        &mut self,
        kohta: (i32, i32),
        kuva: &[&str],
        kerroin: i32,
        vari: impl Fn(u8) -> Option<u32>,
    ) {
        let sarakkeita = kuva.iter().map(|rivi| rivi.len()).max().unwrap_or(0);
        let solut: Vec<Option<u32>> = kuva
            .iter()
            .flat_map(|rivi| {
                let tavut = rivi.as_bytes();
                (0..sarakkeita).map(move |i| tavut.get(i).copied())
            })
            .map(|merkki| merkki.and_then(&vari))
            .collect();
        let peitto = spriten_peitto(solut, sarakkeita, kerroin);
        self.peitto(kohta, &peitto);
    }

    /// Valaistu kenttä kohdassa `(x, y)`, `sarakkeita` konepikseliä leveä, rivit `rivit`.
    /// `varit(sarake, rivi)` antaa konepikselin pimeän ja valaistun värin, ja ne sekoitetaan
    /// valon määrän `osuus(x, y)` mukaan; piste on konepikseleinä. Osuus lasketaan konepikselin keskeltä. Jos se poikkeaa
    /// naapureistaan, konepikselin ruudun pikselit saavat osuutensa naapurien keskipisteistä
    /// lineaarisesti: tekstuuri pysyy konepikseleinä, mutta valon reuna on sileä, eikä
    /// `osuus`-funktiota kutsuta enempää kuin ennen.
    pub fn valaistu_alue(
        &mut self,
        (x, y): (i32, i32),
        sarakkeita: usize,
        rivit: std::ops::Range<usize>,
        osuus: impl Fn(f32, f32) -> f32,
        varit: impl Fn(usize, usize) -> (u32, u32),
    ) {
        let vari = |(pimea, valo): (u32, u32), osuus: f32| {
            if osuus <= 0.0 {
                pimea
            } else {
                sekoita_alfa(pimea, valo, (osuus.min(1.0) * 255.0 + 0.5) as u32)
            }
        };
        // Loivassa liu'ussa konepikselin porras jää muutamaan sävyaskeleeseen, joten vain
        // jyrkät reunat lasketaan pikseleittäin.
        const RAJA: f32 = 1.0 / 16.0;
        if sarakkeita == 0 {
            return;
        }
        let p = self.pikseli as usize;
        let laske = |rivi: usize, kohde: &mut Vec<f32>| {
            kohde.clear();
            kohde.extend((0..sarakkeita).map(|s| osuus(s as f32 + 0.5, rivi as f32 + 0.5)));
        };
        let (mut yla, mut oma, mut ala) = (Vec::new(), Vec::new(), Vec::new());
        laske(rivit.start.saturating_sub(1), &mut yla);
        laske(rivit.start, &mut oma);
        let mut konerivi = vec![0; sarakkeita];
        let mut reunat = Vec::new();
        // Ruudun pikselin etäisyys konepikselin keskeltä (-1/3, 0, 1/3, kun p = 3).
        let siirrot: Vec<f32> = (0..p).map(|i| (i as f32 + 0.5) / p as f32 - 0.5).collect();
        for rivi in rivit {
            laske(rivi + 1, &mut ala);
            // Ensin rivi konepikseleinä kuten karkea taustakuva. Rivin ja sen naapurien
            // valaistun välin ulkopuolella pikseli on suoraan pimeä, eikä siellä ole reunaa.
            reunat.clear();
            let valaistu = |rivi: &[f32]| {
                let alku = rivi.iter().position(|&o| o != 0.0)?;
                Some((alku, rivi.iter().rposition(|&o| o != 0.0).unwrap_or(alku)))
            };
            let vali = [&yla, &oma, &ala]
                .iter()
                .filter_map(|r| valaistu(r))
                .reduce(|a, b| (a.0.min(b.0), a.1.max(b.1)))
                .map(|(a, b)| (a.saturating_sub(1), (b + 2).min(sarakkeita)));
            let (alku, loppu) = vali.unwrap_or((0, 0));
            for (s, kohde) in konerivi.iter_mut().enumerate() {
                if s < alku || s >= loppu {
                    *kohde = varit(s, rivi).0;
                    continue;
                }
                let o = oma[s];
                *kohde = vari(varit(s, rivi), o);
                let naapurit = [
                    yla[s],
                    ala[s],
                    oma[s.saturating_sub(1)],
                    oma[(s + 1).min(sarakkeita - 1)],
                ];
                if naapurit.iter().any(|n| (n - o).abs() > RAJA) {
                    reunat.push(s);
                }
            }
            let ylin = y + (rivi * p) as i32;
            self.karkea_rivi(x, ylin, &konerivi);
            // Sitten valon reunan konepikselit pikseleittäin.
            for &s in &reunat {
                let parin = varit(s, rivi);
                for (sy, &fy) in siirrot.iter().enumerate() {
                    let pysty = if fy < 0.0 { &yla } else { &ala };
                    let py = ylin + sy as i32;
                    for (sx, &fx) in siirrot.iter().enumerate() {
                        let n = if fx < 0.0 {
                            s.saturating_sub(1)
                        } else {
                            (s + 1).min(sarakkeita - 1)
                        };
                        let (ax, ay) = (fx.abs(), fy.abs());
                        let tama = oma[s] + (oma[n] - oma[s]) * ax;
                        let toinen = pysty[s] + (pysty[n] - pysty[s]) * ax;
                        let px = x + (s * p + sx) as i32;
                        self.aseta(px, py, vari(parin, tama + (toinen - tama) * ay));
                    }
                }
            }
            std::mem::swap(&mut yla, &mut oma);
            std::mem::swap(&mut oma, &mut ala);
        }
    }

    /// Yksi pikseli, kuvan ulkopuoli ohitetaan.
    fn aseta(&mut self, x: i32, y: i32, vari: u32) {
        if x >= 0 && y >= 0 && (x as usize) < self.leveys && (y as usize) < self.korkeus {
            self.kuva[y as usize * self.leveys + x as usize] = vari;
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

    /// Valmiin rivin kopio kohtaan `(x, y)`, kuvan ulkopuoli leikataan pois.
    pub fn tarkka_rivi(&mut self, x: i32, y: i32, lahde: &[u32]) {
        if y < 0 || y >= self.korkeus as i32 {
            return;
        }
        let ohitus = (-x).max(0) as usize;
        let vasen = x.max(0) as usize;
        let oikea = (x + lahde.len() as i32).min(self.leveys as i32);
        if ohitus >= lahde.len() || vasen as i32 >= oikea {
            return;
        }
        let oikea = oikea as usize;
        let alku = y as usize * self.leveys;
        self.kuva[alku + vasen..alku + oikea]
            .copy_from_slice(&lahde[ohitus..ohitus + oikea - vasen]);
    }

    /// Täyttö pystysuuntaisella liu'ulla, joka vaihtuu tasaisesti rivi riviltä.
    pub fn liukulaatikko(&mut self, x: i32, y: i32, l: i32, k: i32, yla: u32, ala: u32) {
        let jakaja = (k - 1).max(1) as f32;
        for rivi in 0..k {
            let vari = sekoita_liuku(yla, ala, rivi as f32 / jakaja);
            self.laatikko(x, y + rivi, l, 1, vari);
        }
    }

    /// Piirtää mallin kaukaisimmasta tahkosta lähimpään, jolloin kukin peittää taakseen
    /// jäävät. Tasaiset pinnat kuin Super FX -peleissä, ilman reunaviivoja.
    ///
    /// Malli rasteroidaan ensin alinäytteinä ja tiivistetään sitten pikseleiksi: tahkon
    /// sisäpikseli saa tahkon värin, mutta reunapikseli saa peittävyyden mukaisen välisävyn.
    /// Näin reunat eivät porrastu, eivätkä tahkojen väliset saumat päästä taustaa läpi.
    pub fn malli(&mut self, tahkot: &[Tahko], kulma: f32, keski: (i32, i32), mitta: f32) {
        let tahkot = mallin_tahkot(tahkot, kulma, keski, mitta);
        piirra_naytteina(self, &tahkot);
    }
}

/// Spriten tunniste: solujen värit, sarakkeiden määrä ja suurennos.
type SpritenAvain = (Vec<Option<u32>>, usize, i32);

thread_local! {
    /// Pehmennetyt spritet värisolujen ja koon mukaan.
    static SPRITET: RefCell<HashMap<SpritenAvain, Rc<Peitto>>> = RefCell::new(HashMap::new());
}

/// Välimuistin yläraja: vaihtuvavärinen sprite tuottaa joka värille oman kuvansa.
const SPRITEJA_ENINTAAN: usize = 1024;

/// Spriten pehmennetty kuva. Sama sprite samoin värein lasketaan vain kerran.
fn spriten_peitto(solut: Vec<Option<u32>>, sarakkeita: usize, kerroin: i32) -> Rc<Peitto> {
    SPRITET.with_borrow_mut(|spritet| {
        let avain = (solut, sarakkeita, kerroin);
        if let Some(peitto) = spritet.get(&avain) {
            return peitto.clone();
        }
        if spritet.len() >= SPRITEJA_ENINTAAN {
            spritet.clear();
        }
        let (solut, sarakkeita, kerroin) = &avain;
        let riveja = solut.len().checked_div(*sarakkeita).unwrap_or(0);
        let peitto = Rc::new(pehmenna(
            *sarakkeita as i32,
            riveja as i32,
            (*kerroin, *kerroin),
            |rivi, sarake| solut[rivi as usize * sarakkeita + sarake as usize],
        ));
        spritet.insert(avain, peitto.clone());
        peitto
    })
}

/// Tahko ruudun koordinaatteina ja sen kaksi ditteröinnin väriä.
struct Ruututahko {
    pisteet: [(f32, f32); 5],
    maara: usize,
    vari: u32,
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
            Ruututahko {
                pisteet,
                maara: tahko.maara,
                vari: varjosta(tahko.vari, valaistus(&kierretyt)),
            }
        })
        .collect()
}

/// Alinäytteitä pikselin sivulla.
const ALINAYTTEITA: usize = 2;
/// Näyte, johon mikään tahko ei osunut. Värit ovat 24-bittisiä, joten tämä ei ole väri.
const EI_NAYTETTA: u32 = u32::MAX;

/// Rasteroi tahkot alinäytteiksi mallin ympärille ja tiivistää ne pikseleiksi:
/// kokonaan peitetty pikseli saa näytteidensä värin, reunapikseli sekoittuu
/// taustaan peittävyyden mukaan.
fn piirra_naytteina(kangas: &mut Kangas, tahkot: &[Ruututahko]) {
    let pisteet = || tahkot.iter().flat_map(|t| &t.pisteet[..t.maara]);
    let Some(vasen) = pisteet().map(|p| p.0).reduce(f32::min) else {
        return;
    };
    let oikea = pisteet().map(|p| p.0).fold(vasen, f32::max);
    let yla = pisteet().map(|p| p.1).fold(f32::MAX, f32::min);
    let ala = pisteet().map(|p| p.1).fold(f32::MIN, f32::max);

    // Rajaus pikseleinä, kankaan sisällä.
    let alku_x = (vasen.floor() as i32).max(0);
    let alku_y = (yla.floor() as i32).max(0);
    let loppu_x = (oikea.ceil() as i32).min(kangas.leveys as i32 - 1);
    let loppu_y = (ala.ceil() as i32).min(kangas.korkeus as i32 - 1);
    if alku_x > loppu_x || alku_y > loppu_y {
        return;
    }
    let s = ALINAYTTEITA;
    let leveys = (loppu_x - alku_x + 1) as usize * s;
    let korkeus = (loppu_y - alku_y + 1) as usize * s;
    let mut naytteet = vec![EI_NAYTETTA; leveys * korkeus];
    // Kunkin näyterivin täytetyt sarakkeet: tiivistys ohittaa rajauksen tyhjät kulmat.
    let mut rajat = vec![(usize::MAX, 0); korkeus];
    let koko = 1.0 / s as f32;
    let (nolla_x, nolla_y) = (alku_x as f32, alku_y as f32);

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
                naytteet[rivi * leveys + vasen..rivi * leveys + oikea].fill(tahko.vari);
            },
        );
    }

    let kaikki = (s * s) as u32;
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
        let kuvan_y = alku_y as usize + rivi;
        for sarake in vasen / s..=(oikea - 1) / s {
            let alku = sarake * s;
            let ensimmainen = rivit[0][alku];
            let yhtenainen = rivit
                .iter()
                .all(|r| r[alku..alku + s].iter().all(|&vari| vari == ensimmainen));
            let kuvan_x = alku_x as usize + sarake;
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
                // Reunalla malli peittää vain osan pikselistä: loput on taustaa.
                let tausta = kangas.kuva[kuvan_y * kangas.leveys + kuvan_x];
                let loput = kaikki - osuneet;
                let kanava =
                    |summa: u32, siirto: u32| (summa + (tausta >> siirto & 0xFF) * loput) / kaikki;
                (kanava(r, 16) << 16) | (kanava(g, 8) << 8) | kanava(b, 0)
            };
            kangas.kuva[kuvan_y * kangas.leveys + kuvan_x] = vari;
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

/// Väri `vari` pohjan päälle peittävyydellä `alfa` 0..=255, kokonaisluvuin.
pub fn sekoita_alfa(pohja: u32, vari: u32, alfa: u32) -> u32 {
    if alfa >= 255 {
        return vari;
    }
    // 255 vastaa 256:ta, jolloin jako on siirto.
    let a = alfa + (alfa >> 7);
    let b = 256 - a;
    let pr = ((vari & 0xFF00FF) * a + (pohja & 0xFF00FF) * b) >> 8 & 0xFF00FF;
    let g = ((vari & 0x00FF00) * a + (pohja & 0x00FF00) * b) >> 8 & 0x00FF00;
    pr | g
}

pub fn sekoita_liuku(a: u32, b: u32, osuus: f32) -> u32 {
    let kanava = |siirto: u32| {
        let (x, y) = ((a >> siirto & 0xFF) as f32, (b >> siirto & 0xFF) as f32);
        ((x + (y - x) * osuus) as u32) << siirto
    };
    kanava(16) | kanava(8) | kanava(0)
}

/// Pinnan valoisuus: sitä enemmän, mitä suoremmin valo osuu. Sävy vaihtuu portaatta,
/// joten pyörivän mallin pinnat tummuvat ja vaalenevat pehmeästi.
fn valaistus(p: &[[f32; 3]; 5]) -> f32 {
    let a = [p[1][0] - p[0][0], p[1][1] - p[0][1], p[1][2] - p[0][2]];
    let b = [p[2][0] - p[0][0], p[2][1] - p[0][1], p[2][2] - p[0][2]];
    let n = [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ];
    let pituus = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(0.001);
    let osuvuus = (n[0] * VALO[0] + n[1] * VALO[1] + n[2] * VALO[2]).abs() / pituus;
    0.45 + osuvuus * 0.6
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
