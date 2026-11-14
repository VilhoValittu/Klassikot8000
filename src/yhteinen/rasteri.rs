//! Valmiiksi lasketut taustakuvat konepikseleinä. Tausta kuten 16-bittisten koneiden taustataso:
//! se lasketaan kerran, ja piirto on pelkkää rivien kopiointia, joten hieno tausta ei maksa mitään.

use super::vektori::{Kangas, sekoita_alfa, sekoita_liuku};

/// Välisävy värien `a` ja `b` väliltä, `osuus` 0..1. Tämä korvasi järjestetyn
/// ditteröinnin, jonka ruudutus näytti rakeiselta; kohinan kanssa kutsuttuna tulos on
/// silti elävä pintakuvio.
pub fn valisavy(a: u32, b: u32, osuus: f32) -> u32 {
    sekoita_alfa(a, b, (osuus.clamp(0.0, 1.0) * 255.0 + 0.5) as u32)
}

/// Liuku useiden värien läpi: sävy vaihtuu portaatta.
pub fn liukuvari(varit: &[u32], t: f32) -> u32 {
    let osia = (varit.len() - 1) as f32;
    let kohta = t.clamp(0.0, 0.9999) * osia;
    let osa = kohta as usize;
    sekoita_liuku(varit[osa], varit[osa + 1], kohta.fract())
}

/// Onko piste `(x, y)` konepikselin keskelle piirretyn pyöreän täplän sisällä? Tarkka
/// kuva pehmentää täplän reunan, joten tähdet ja niitit ovat pyöreitä eivätkä neliöitä.
pub fn taplassa(x: f32, y: f32, sade: f32) -> bool {
    let (dx, dy) = (x.rem_euclid(1.0) - 0.5, y.rem_euclid(1.0) - 0.5);
    dx * dx + dy * dy < sade * sade
}

/// Sama siemen antaa aina saman luvun väliltä 0..1: taustojen tähdet ja kohina.
pub fn kohina(x: i32, y: i32) -> f32 {
    let siemen = (x as u32)
        .wrapping_mul(0x9E37_79B1)
        .wrapping_add((y as u32).wrapping_mul(0x85EB_CA6B));
    let h = (siemen ^ (siemen >> 15)).wrapping_mul(0xC2B2_AE35);
    ((h ^ (h >> 13)) & 0xFFFF) as f32 / 65536.0
}

/// Taustakuva. Karkea kuva on konepikseleinä ja venytetään piirrettäessä; tarkka kuva on
/// laskettu valmiiksi ruudun pikseleinä (`tiheys` pikseliä konepikselin sivulla), joten sen
/// reunat ovat pehmeät ja piirto on pelkkää kopiointia. Alueet ja korkeus ovat aina
/// konepikseleinä, joten kutsujan ei tarvitse tietää, kumpi kuva on kyseessä.
pub struct Rasteri {
    leveys: usize,
    pikselit: Vec<u32>,
    tiheys: usize,
}

/// Väriero, jonka kohdalla tarkka kuva näytteistetään uudelleen reunanpehmennystä varten.
const REUNAN_ERO: u32 = 40;
/// Reunapikselin alinäytteet sivua kohti.
const REUNAN_NAYTTEET: usize = 3;

impl Rasteri {
    pub fn luo(leveys: usize, korkeus: usize, vari: impl Fn(i32, i32) -> u32) -> Self {
        let mut pikselit = Vec::with_capacity(leveys * korkeus);
        for y in 0..korkeus as i32 {
            for x in 0..leveys as i32 {
                pikselit.push(vari(x, y));
            }
        }
        Self {
            leveys,
            pikselit,
            tiheys: 1,
        }
    }

    /// Tarkka kuva, `leveys` x `korkeus` konepikseliä. `vari(x, y)` saa pisteen
    /// konepikseleinä liukulukuna. Kukin pikseli lasketaan keskipisteestään, ja pikselit,
    /// jotka eroavat naapuristaan jyrkästi, lasketaan uudelleen alinäytteiden
    /// keskiarvona: vain reunat maksavat lisää, ja ne pehmenevät.
    pub fn tarkka(
        leveys: usize,
        korkeus: usize,
        tiheys: usize,
        vari: impl Fn(f32, f32) -> u32 + Sync,
    ) -> Self {
        let (l, k) = (leveys * tiheys, korkeus * tiheys);
        let mitta = 1.0 / tiheys as f32;
        // Täysi ruutu on kaksi miljoonaa pikseliä: rivit jaetaan kaikille ytimille, jotta
        // ensimmäinen kuva ei viivästy.
        let saikeita = std::thread::available_parallelism().map_or(1, |n| n.get());
        let rivit_osaan = k.div_ceil(saikeita).max(1);
        let mut pikselit = vec![0; l * k];
        std::thread::scope(|s| {
            for (osa, alue) in pikselit.chunks_mut(rivit_osaan * l).enumerate() {
                let vari = &vari;
                s.spawn(move || {
                    for (i, pikseli) in alue.iter_mut().enumerate() {
                        let (x, y) = (i % l, osa * rivit_osaan + i / l);
                        *pikseli = vari((x as f32 + 0.5) * mitta, (y as f32 + 0.5) * mitta);
                    }
                });
            }
        });
        let reunalla = |i: usize| {
            let (x, y, oma) = (i % l, i / l, pikselit[i]);
            (x + 1 < l && varieroa(oma, pikselit[i + 1]))
                || (y + 1 < k && varieroa(oma, pikselit[i + l]))
                || (x > 0 && varieroa(oma, pikselit[i - 1]))
                || (y > 0 && varieroa(oma, pikselit[i - l]))
        };
        let n = REUNAN_NAYTTEET;
        let tasoita = |i: usize| {
            let (x, y) = ((i % l) as f32, (i / l) as f32);
            let (mut r, mut g, mut b) = (0, 0, 0);
            for ny in 0..n {
                for nx in 0..n {
                    let vari = vari(
                        (x + (nx as f32 + 0.5) / n as f32) * mitta,
                        (y + (ny as f32 + 0.5) / n as f32) * mitta,
                    );
                    r += vari >> 16 & 0xFF;
                    g += vari >> 8 & 0xFF;
                    b += vari & 0xFF;
                }
            }
            let maara = (n * n) as u32;
            ((r / maara) << 16) | ((g / maara) << 8) | (b / maara)
        };
        let korjaukset: Vec<(usize, u32)> = std::thread::scope(|s| {
            let osat: Vec<_> = (0..saikeita)
                .map(|osa| {
                    let (reunalla, tasoita) = (&reunalla, &tasoita);
                    s.spawn(move || {
                        let alue = (osa * rivit_osaan * l).min(l * k)
                            ..((osa + 1) * rivit_osaan * l).min(l * k);
                        alue.filter(|&i| reunalla(i))
                            .map(|i| (i, tasoita(i)))
                            .collect::<Vec<_>>()
                    })
                })
                .collect();
            osat.into_iter()
                .flat_map(|osa| osa.join().expect("taustan laskenta ei kaadu"))
                .collect()
        });
        for (i, vari) in korjaukset {
            pikselit[i] = vari;
        }
        Self {
            leveys: l,
            pikselit,
            tiheys,
        }
    }

    /// Piirretystä kuvasta: `tiheys` on kuvan pikseleitä konepikselin sivulla.
    pub fn kuvasta(kuva: &[u32], kuvan_leveys: usize, tiheys: usize) -> Self {
        Self {
            leveys: kuvan_leveys,
            pikselit: kuva.to_vec(),
            tiheys,
        }
    }

    /// Sama kuva, jonka jokainen pikseli on kulkenut `muunnos`-funktion läpi.
    pub fn muunna(&self, muunnos: impl Fn(u32) -> u32) -> Self {
        Self {
            leveys: self.leveys,
            pikselit: self.pikselit.iter().map(|&vari| muunnos(vari)).collect(),
            tiheys: self.tiheys,
        }
    }

    /// Korkeus konepikseleinä.
    pub fn korkeus(&self) -> usize {
        self.pikselit.len() / self.leveys / self.tiheys
    }

    /// Kuvan pikseleitä konepikselin sivulla: karkealla kuvalla 1.
    pub fn tiheys(&self) -> usize {
        self.tiheys
    }

    /// Tallennettu rivi: tarkalla kuvalla ruudun pikseleinä.
    pub fn rivi(&self, y: usize) -> &[u32] {
        &self.pikselit[y * self.leveys..(y + 1) * self.leveys]
    }

    pub fn piirra(&self, k: &mut Kangas, x: i32, y: i32) {
        let leveys = self.leveys / self.tiheys;
        self.piirra_alue(k, x, y, (0, 0, leveys, self.korkeus()));
    }

    /// Piirtää kuvan osan `(x, y, l, k)`, konepikseleinä, paikkaan `(x, y)`.
    pub fn piirra_alue(
        &self,
        kangas: &mut Kangas,
        x: i32,
        y: i32,
        (ax, ay, al, ak): (usize, usize, usize, usize),
    ) {
        let t = self.tiheys;
        if t == 1 {
            let p = kangas.pikseli;
            for rivi in 0..ak {
                let lahde = &self.rivi(ay + rivi)[ax..ax + al];
                kangas.karkea_rivi(x, y + rivi as i32 * p, lahde);
            }
            return;
        }
        debug_assert_eq!(t as i32, kangas.pikseli, "tarkka kuva väärälle kankaalle");
        for rivi in 0..ak * t {
            let lahde = &self.rivi(ay * t + rivi)[ax * t..(ax + al) * t];
            kangas.tarkka_rivi(x, y + rivi as i32, lahde);
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
        let (l, k) = (self.leveys / self.tiheys, raja);
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

/// Eroavatko värit jossakin kanavassa enemmän kuin `REUNAN_ERO`.
fn varieroa(a: u32, b: u32) -> bool {
    [16, 8, 0]
        .iter()
        .any(|&s| (a >> s & 0xFF).abs_diff(b >> s & 0xFF) > REUNAN_ERO)
}

#[cfg(test)]
mod tests;
