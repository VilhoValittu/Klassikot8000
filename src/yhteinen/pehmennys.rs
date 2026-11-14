//! Karkean ruudukon kuvat (kirjasin, ASCII-spritet) pehmeiksi peittokuviksi. Porrasreunat
//! oikenevat EPX-säännöllä viistoiksi: solun kulmasta tulee kolmio, kun kaksi sen viereistä
//! naapuria ovat keskenään samaa väriä. Kulmat näytteistetään tiheämmin kuin pikselit,
//! joten reunapikseli saa peittävyytensä mukaisen läpikuultavuuden eikä porrasta jää.

/// Alinäytteitä pikselin sivulla: 16 näytettä antaa 17 peittävyyden porrasta.
const NAYTTEITA: i32 = 4;

/// Peittokuva: kunkin pikselin väri (osuneiden näytteiden keskiarvo) ja peittävyys 0..=255.
pub struct Peitto {
    pub leveys: usize,
    pub korkeus: usize,
    pub pikselit: Vec<(u32, u8)>,
    /// Kunkin rivin näkyvät sarakkeet `alku..loppu`: piirto ohittaa läpinäkyvät reunat.
    pub valit: Vec<(usize, usize)>,
}

/// Pehmentää `sarakkeita` x `riveja` solun ruudukon, jonka solu on `(pl, pk)` pikseliä.
/// `solu(rivi, sarake)` on solun väri tai `None` läpinäkyvälle; sitä kysytään vain
/// ruudukon sisältä, ja ulkopuoli on läpinäkyvä.
pub fn pehmenna(
    sarakkeita: i32,
    riveja: i32,
    (pl, pk): (i32, i32),
    solu: impl Fn(i32, i32) -> Option<u32>,
) -> Peitto {
    // Solut reunuksineen, jotta naapurit voi lukea tarkistamatta rajoja.
    let reunus_l = sarakkeita + 4;
    let mut solut = vec![None; (reunus_l * (riveja + 4)) as usize];
    for rivi in 0..riveja {
        for sarake in 0..sarakkeita {
            solut[((rivi + 2) * reunus_l + sarake + 2) as usize] = solu(rivi, sarake);
        }
    }
    let lue = |rivi: i32, sarake: i32| solut[((rivi + 2) * reunus_l + sarake + 2) as usize];

    let (leveys, korkeus) = ((sarakkeita * pl) as usize, (riveja * pk) as usize);
    let mut pikselit = vec![(0, 0); leveys * korkeus];
    let kaikki = (NAYTTEITA * NAYTTEITA) as u32;
    // Näytteen paikka solun sisällä: pikselin alinäytteen keskipiste soluina.
    let (askel_x, askel_y) = (1.0 / (NAYTTEITA * pl) as f32, 1.0 / (NAYTTEITA * pk) as f32);
    for py in 0..korkeus {
        let rivi = py as i32 / pk;
        for px in 0..leveys {
            let sarake = px as i32 / pl;
            let oma = lue(rivi, sarake);
            // Solun keskiosa on aina solun väriä: kolmiot ulottuvat vain kulmista
            // puoliväliin, joten keskimmäiset pikselit ohittavat näytteistyksen.
            let (sx, sy) = (px as i32 % pl, py as i32 % pk);
            let kulmasta = |s: i32, koko: i32| s.min(koko - 1 - s) as f32 / koko as f32;
            if kulmasta(sx, pl) + kulmasta(sy, pk) >= 0.5 {
                if let Some(vari) = oma {
                    pikselit[py * leveys + px] = (vari, 255);
                }
                continue;
            }
            let (mut r, mut g, mut b, mut osuneet) = (0, 0, 0, 0);
            for ny in 0..NAYTTEITA {
                let fy = ((sy * NAYTTEITA + ny) as f32 + 0.5) * askel_y;
                for nx in 0..NAYTTEITA {
                    let fx = ((sx * NAYTTEITA + nx) as f32 + 0.5) * askel_x;
                    if let Some(vari) = naytteen_vari(&lue, rivi, sarake, oma, fx, fy) {
                        r += vari >> 16 & 0xFF;
                        g += vari >> 8 & 0xFF;
                        b += vari & 0xFF;
                        osuneet += 1;
                    }
                }
            }
            if let (Some(r), Some(g), Some(b)) = (
                r.checked_div(osuneet),
                g.checked_div(osuneet),
                b.checked_div(osuneet),
            ) {
                let vari = (r << 16) | (g << 8) | b;
                pikselit[py * leveys + px] = (vari, (osuneet * 255 / kaikki) as u8);
            }
        }
    }
    Peitto::uusi(leveys, korkeus, pikselit)
}

impl Peitto {
    pub fn uusi(leveys: usize, korkeus: usize, pikselit: Vec<(u32, u8)>) -> Self {
        let valit = pikselit
            .chunks_exact(leveys.max(1))
            .map(|rivi| {
                let alku = rivi.iter().position(|p| p.1 != 0).unwrap_or(0);
                let loppu = rivi.iter().rposition(|p| p.1 != 0).map_or(0, |i| i + 1);
                (alku, loppu)
            })
            .collect();
        Self {
            leveys,
            korkeus,
            pikselit,
            valit,
        }
    }

    /// Piirtää `peiton` tämän päälle kohtaan `(x, y)` värillä `vari(rivi)`, kuten
    /// läpikuultava maali: peittävyydet ja värit yhdistyvät.
    pub fn maalaa(&mut self, (x, y): (usize, usize), peitto: &Peitto, vari: impl Fn(usize) -> u32) {
        for rivi in 0..peitto.korkeus {
            let (alku, loppu) = peitto.valit[rivi];
            for sarake in alku..loppu {
                let ylla = peitto.pikselit[rivi * peitto.leveys + sarake].1 as f32 / 255.0;
                if ylla <= 0.0 {
                    continue;
                }
                let kohde = &mut self.pikselit[(y + rivi) * self.leveys + x + sarake];
                let alla = kohde.1 as f32 / 255.0;
                let yhteensa = ylla + alla * (1.0 - ylla);
                let osuus = ylla / yhteensa;
                let kanava = |s: u32| {
                    let (a, b) = (
                        (kohde.0 >> s & 0xFF) as f32,
                        (vari(rivi) >> s & 0xFF) as f32,
                    );
                    ((a + (b - a) * osuus).round() as u32) << s
                };
                *kohde = (
                    kanava(16) | kanava(8) | kanava(0),
                    (yhteensa * 255.0).round() as u8,
                );
            }
        }
    }

    /// Laskee rivien näkyvät välit uudelleen maalauksen jälkeen.
    pub fn viimeistele(self) -> Self {
        Self::uusi(self.leveys, self.korkeus, self.pikselit)
    }
}

/// Näytteen väri kohdassa `(fx, fy)` solun sisällä (0..1). EPX: vasemman yläkulman
/// kolmio saa vasemman ja ylemmän naapurin värin, jos ne ovat samat ja kumpikin eroaa
/// vastakkaisesta naapuristaan (vasen alemmasta, ylempi oikeasta). Kolmio on kulman
/// puolikas, joten 45 asteen porras oikenee suoraksi viivaksi.
fn naytteen_vari(
    lue: &impl Fn(i32, i32) -> Option<u32>,
    rivi: i32,
    sarake: i32,
    oma: Option<u32>,
    fx: f32,
    fy: f32,
) -> Option<u32> {
    let (vaaka, dx) = if fx < 0.5 { (-1, fx) } else { (1, 1.0 - fx) };
    let (pysty, dy) = if fy < 0.5 { (-1, fy) } else { (1, 1.0 - fy) };
    if dx + dy >= 0.5 {
        return oma;
    }
    let sivu = lue(rivi, sarake + vaaka);
    let yla_ala = lue(rivi + pysty, sarake);
    if sivu == yla_ala && sivu != lue(rivi - pysty, sarake) && yla_ala != lue(rivi, sarake - vaaka)
    {
        sivu
    } else {
        oma
    }
}

#[cfg(test)]
mod tests;
