//! Valmiiksi lasketut taustakuvat konepikseleinä. Tausta kuten 16-bittisten koneiden taustataso:
//! se lasketaan kerran, ja piirto on pelkkää rivien kopiointia, joten hieno tausta ei maksa mitään.

use super::vektori::{Kangas, sekoita_liuku};

/// Järjestetty ditteröinti 4x4-kuviolla.
const BAYER: [[u8; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];

pub fn ditteroi(x: i32, y: i32, a: u32, b: u32, osuus: f32) -> u32 {
    let raja = BAYER[y.rem_euclid(4) as usize][x.rem_euclid(4) as usize] as f32 + 0.5;
    if osuus * 16.0 > raja { b } else { a }
}

/// Liuku useiden värien läpi. Sävyt kvantisoidaan kaistoiksi, joiden väliin tulee ditteröinti,
/// joten liuku näyttää rajatun väripaletin liu'ulta eikä nykyaikaiselta tasaiselta.
pub fn liukuvari(varit: &[u32], t: f32, x: i32, y: i32) -> u32 {
    const KAISTOJA: f32 = 6.0;
    let osia = (varit.len() - 1) as f32;
    let kohta = t.clamp(0.0, 0.9999) * osia;
    let osa = kohta as usize;
    let sisalla = kohta.fract() * KAISTOJA;
    let kaista = sisalla.floor();
    let a = sekoita_liuku(varit[osa], varit[osa + 1], kaista / KAISTOJA);
    let b = sekoita_liuku(varit[osa], varit[osa + 1], (kaista + 1.0) / KAISTOJA);
    ditteroi(x, y, a, b, sisalla - kaista)
}

/// Sama siemen antaa aina saman luvun väliltä 0..1: taustojen tähdet ja kohina.
pub fn kohina(x: i32, y: i32) -> f32 {
    let siemen = (x as u32)
        .wrapping_mul(0x9E37_79B1)
        .wrapping_add((y as u32).wrapping_mul(0x85EB_CA6B));
    let h = (siemen ^ (siemen >> 15)).wrapping_mul(0xC2B2_AE35);
    ((h ^ (h >> 13)) & 0xFFFF) as f32 / 65536.0
}

pub struct Rasteri {
    leveys: usize,
    pikselit: Vec<u32>,
}

impl Rasteri {
    pub fn luo(leveys: usize, korkeus: usize, vari: impl Fn(i32, i32) -> u32) -> Self {
        let mut pikselit = Vec::with_capacity(leveys * korkeus);
        for y in 0..korkeus as i32 {
            for x in 0..leveys as i32 {
                pikselit.push(vari(x, y));
            }
        }
        Self { leveys, pikselit }
    }

    /// Konepikselin kokoiseksi piirretystä kuvasta takaisin konepikseleiksi.
    pub fn kuvasta(kuva: &[u32], kuvan_leveys: usize, pikseli: usize) -> Self {
        let korkeus = kuva.len() / kuvan_leveys / pikseli;
        let leveys = kuvan_leveys / pikseli;
        Self::luo(leveys, korkeus, |x, y| {
            kuva[y as usize * pikseli * kuvan_leveys + x as usize * pikseli]
        })
    }

    pub fn korkeus(&self) -> usize {
        self.pikselit.len() / self.leveys
    }

    pub fn rivi(&self, y: usize) -> &[u32] {
        &self.pikselit[y * self.leveys..(y + 1) * self.leveys]
    }

    pub fn piirra(&self, k: &mut Kangas, x: i32, y: i32) {
        self.piirra_alue(k, x, y, (0, 0, self.leveys, self.korkeus()));
    }

    /// Piirtää kuvan osan `(x, y, l, k)`, konepikseleinä, paikkaan `(x, y)`.
    pub fn piirra_alue(
        &self,
        kangas: &mut Kangas,
        x: i32,
        y: i32,
        (ax, ay, al, ak): (usize, usize, usize, usize),
    ) {
        let p = kangas.pikseli;
        for rivi in 0..ak {
            let lahde = &self.rivi(ay + rivi)[ax..ax + al];
            kangas.karkea_rivi(x, y + rivi as i32 * p, lahde);
        }
    }

    /// Kuvan rivit ennen riviä `raja` paitsi alue `(x, y, l, k)`, konepikseleinä. Kentän yli
    /// piirtyvät kappaleet peittyvät näin, eikä kenttää tarvitse piirtää kahdesti.
    pub fn piirra_paitsi(
        &self,
        kangas: &mut Kangas,
        (ax, ay, al, ak): (usize, usize, usize, usize),
        raja: usize,
    ) {
        let (l, k) = (self.leveys, raja);
        let p = kangas.pikseli;
        let kohta = |n: usize| n as i32 * p;
        self.piirra_alue(kangas, 0, 0, (0, 0, l, ay));
        self.piirra_alue(kangas, 0, kohta(ay), (0, ay, ax, ak));
        self.piirra_alue(
            kangas,
            kohta(ax + al),
            kohta(ay),
            (ax + al, ay, l - ax - al, ak),
        );
        self.piirra_alue(kangas, 0, kohta(ay + ak), (0, ay + ak, l, k - ay - ak));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yhteinen::piirto::{KORKEUS, LEVEYS};

    #[test]
    fn ditterointi_antaa_paat_tarkasti() {
        for y in 0..4 {
            for x in 0..4 {
                assert_eq!(ditteroi(x, y, 1, 2, 0.0), 1);
                assert_eq!(ditteroi(x, y, 1, 2, 1.0), 2);
            }
        }
        let puolet = (0..16)
            .filter(|i| ditteroi(i % 4, i / 4, 1, 2, 0.5) == 2)
            .count();
        assert_eq!(puolet, 8);
    }

    #[test]
    fn liuku_kestaa_paat_ja_kohina_pysyy_valilla() {
        for t in [-1.0, 0.0, 0.5, 1.0, 2.0] {
            liukuvari(&[1, 0xFFFFFF, 3], t, 0, 0);
        }
        assert_eq!(liukuvari(&[0x101010, 0xFFFFFF], 0.0, 0, 0), 0x101010);
        for i in 0..1000 {
            assert!((0.0..1.0).contains(&kohina(i, i * 3)));
        }
    }

    #[test]
    fn piirto_paitsi_alue_jattaa_alueen_koskematta() {
        let mut kuva = vec![0; LEVEYS * KORKEUS];
        let rasteri = Rasteri::luo(320, 256, |_, _| 7);
        let mut kangas = Kangas::new(&mut kuva);
        rasteri.piirra_paitsi(&mut kangas, (10, 10, 100, 100), 256);
        let pikseli = |x: usize, y: usize| kuva[y * LEVEYS + x];
        assert_eq!(pikseli(0, 0), 7);
        assert_eq!(pikseli(29, 150), 7);
        assert_eq!(pikseli(30, 30), 0);
        assert_eq!(pikseli(329, 329), 0);
        assert_eq!(pikseli(330, 150), 7);
        assert_eq!(pikseli(150, 330), 7);
        assert_eq!(pikseli(LEVEYS - 1, KORKEUS - 1), 7);
    }
}
