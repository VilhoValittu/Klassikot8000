//! 16-bittisen konsolin henkiset äänet: FM-torvet ja -kellot, PSG-kohina, kanttiaallot ja
//! SNES-tyylinen kaiku, sekä pieni puhesyntetisaattori. Kaikki syntetisoidaan käynnistyksessä,
//! joten paketissa ei ole äänitiedostoja. Äänilaite näkyy vain tiedostossa `main.rs`.

mod puhe;

use std::f32::consts::{PI, TAU};

use klassikot8000::Arpa;

use crate::yhteinen::kieli::{self, Kieli};

/// SNESin oma näytetaajuus. Näytteet ovat 16-bittisiä.
pub const NAYTETAAJUUS: usize = 32000;
const FS: f32 = NAYTETAAJUUS as f32;
const VOIMAKKUUS: f32 = 0.8;
/// Yhtä aikaa soivien ja odottavien äänten enimmäismäärä. Ilman rajaa nopea räjähdyssarja
/// summautuu yli täyden voimakkuuden ja säröytyy leikkurissa.
const ENINTAAN_SOIVIA: usize = 12;
/// Puhe alkaa näin paljon voitto- tai tappiosävelen jälkeen, kun sävel jo sammuu.
pub const PUHE_VIIVE_MS: u32 = 700;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Aani {
    /// Nouseva FM-kellojen arpeggio, kun peli tai uusi taso alkaa.
    Alku,
    /// Oma tykinlaukaus: matala isku ja kohinapamaus.
    Laukaus,
    /// Vastustajan laukaus viheltää tuloaan.
    Vastalaukaus,
    Roiske,
    Rajahdys,
    /// Iso räjähdys, laskeva FM-jyminä ja kaiku.
    Upotus,
    /// Kaksi matalaa piippausta, kun siirto ei käy.
    Virhe,
    Voitto,
    Tappio,
    /// Pehmeä napsahdus: luvut liukuvat, kortti laskeutuu.
    Liuku,
    /// Kirkas kaksisävel: luvut yhdistyvät, pari löytyy.
    Yhdistys,
    /// Tiilirikko: pallo kimpoaa seinästä tai mailasta.
    Pomppu,
    /// Tiilirikko: pallo osuu tiileen.
    Tiili,
    /// Asteroidit: laserlaukaus.
    Ammus,
    /// Asteroidit: kivi hajoaa.
    Kivi,
    /// Asteroidit: aluksen moottori, soitetaan toistuvasti.
    Moottori,
    /// Asteroidit: lentävän lautasen ulina, soitetaan toistuvasti.
    Lautanen,
    /// Silmukka: omena haukataan.
    Omena,
    /// Silmukka: tyhjä silmukka haihtuu.
    Puhkus,
    /// Silmukka: muna kuoriutuu.
    Surina,
    /// Nouseva helinä, yksi sävel ketjun askelta kohden.
    Helina1,
    Helina2,
    Helina3,
    Helina4,
    Helina5,
    Helina6,
    /// Majakka: matala kaksiääninen sumutorvi.
    Sumutorvi,
    /// Majakka: laivakello kilahtaa kahdesti, kun laiva pääsee satamaan.
    Laivakello,
    /// Majakka: runko rusahtaa karikkoon.
    Karille,
    /// Majakka: tuulenpuuska humahtaa.
    Tuuli,
    /// Kaivo: kuokka iskee maahan.
    Kuokka,
    /// Kaivo: tukematon katto narisee ja rapisee.
    Ritina,
    /// Kaivo: maa sortuu jyristen.
    Sortuma,
    /// Kuuluttaja: "Ei siirtoja jäljellä" / "No more moves left".
    EiSiirtoja,
    /// Kuuluttaja: "Peli ohi" / "Game over".
    PeliOhi,
    /// Kuuluttaja: "Voitit!" / "You win!".
    Voitit,
}

const TEHOSTEITA: usize = 33;
const PUHEITA: usize = 3;

pub const KAIKKI: [Aani; TEHOSTEITA + PUHEITA] = [
    Aani::Alku,
    Aani::Laukaus,
    Aani::Vastalaukaus,
    Aani::Roiske,
    Aani::Rajahdys,
    Aani::Upotus,
    Aani::Virhe,
    Aani::Voitto,
    Aani::Tappio,
    Aani::Liuku,
    Aani::Yhdistys,
    Aani::Pomppu,
    Aani::Tiili,
    Aani::Ammus,
    Aani::Kivi,
    Aani::Moottori,
    Aani::Lautanen,
    Aani::Omena,
    Aani::Puhkus,
    Aani::Surina,
    Aani::Helina1,
    Aani::Helina2,
    Aani::Helina3,
    Aani::Helina4,
    Aani::Helina5,
    Aani::Helina6,
    Aani::Sumutorvi,
    Aani::Laivakello,
    Aani::Karille,
    Aani::Tuuli,
    Aani::Kuokka,
    Aani::Ritina,
    Aani::Sortuma,
    Aani::EiSiirtoja,
    Aani::PeliOhi,
    Aani::Voitit,
];

/// Fraasit kielittäin äänteinä (merkinnät tiedostossa `puhe.rs`) ja perussävel alussa ja lopussa.
const FRAASIT: [[(&str, (f32, f32)); PUHEITA]; 2] = [
    [
        ("e! i _ s i!: R t o j a _ j ae! l j e l l ae", (125.0, 88.0)),
        ("p e! l i _ o! h i", (125.0, 90.0)),
        ("v o! i t i t", (140.0, 105.0)),
    ],
    [
        ("n o! U _ m O!: r _ m u!: v z _ l E! f t", (125.0, 88.0)),
        ("g e! I m _ o! U v 3", (125.0, 90.0)),
        ("j u: _ w I! n", (135.0, 110.0)),
    ],
];

impl Aani {
    #[cfg(debug_assertions)]
    pub fn on_puhe(self) -> bool {
        self.puhe().is_some()
    }

    fn puhe(self) -> Option<usize> {
        match self {
            Aani::EiSiirtoja => Some(0),
            Aani::PeliOhi => Some(1),
            Aani::Voitit => Some(2),
            _ => None,
        }
    }

    /// Paikka näytetaulussa: tehosteet ensin, sitten kunkin kielen puheet.
    fn indeksi(self, kieli: Kieli) -> usize {
        match self.puhe() {
            Some(i) => TEHOSTEITA + kieli as usize * PUHEITA + i,
            None => self as usize,
        }
    }
}

struct Soitto {
    indeksi: usize,
    viive: usize,
    kohta: usize,
}

/// Pelin äänikytkin ja soittojono. Käynnistin vie jonon mikserille joka kierroksella.
#[derive(Debug)]
pub struct Aanet {
    pub paalla: bool,
    /// Ääni ja viive millisekunteina.
    pub jono: Vec<(Aani, u32)>,
}

impl Default for Aanet {
    fn default() -> Self {
        Self {
            paalla: true,
            jono: Vec::new(),
        }
    }
}

/// Soittaa päällekkäisiä ääniä. Ei tiedä äänilaitteesta mitään, joten sitä voi testata.
pub struct Mikseri {
    naytteet: Vec<Vec<i16>>,
    soivat: Vec<Soitto>,
    paalla: bool,
}

impl Mikseri {
    pub fn new() -> Self {
        let mut arpa = Arpa::new(0x16B1);
        let mut naytteet: Vec<Vec<i16>> = KAIKKI
            .iter()
            .filter(|aani| aani.puhe().is_none())
            .map(|&aani| valmis(tehoste(aani, &mut arpa)))
            .collect();
        // Molemmat kielet valmiiksi, jotta kielen vaihto ei odota.
        for fraasit in &FRAASIT {
            for &(fraasi, savel) in fraasit {
                naytteet.push(valmis(puhe::puhu(fraasi, savel, &mut arpa)));
            }
        }
        Self {
            naytteet,
            soivat: Vec::new(),
            paalla: true,
        }
    }

    /// Pois kytkeminen hiljentää myös soivat ja viiveellä odottavat äänet heti.
    pub fn aseta_paalle(&mut self, paalla: bool) {
        self.paalla = paalla;
        if !paalla {
            self.soivat.clear();
        }
    }

    /// Puhe soi sillä kielellä, joka on valittuna soittohetkellä.
    pub fn soita(&mut self, aani: Aani, viive_ms: u32) {
        if self.paalla {
            if self.soivat.len() >= ENINTAAN_SOIVIA {
                // Pisimmälle soinut on jo enimmäkseen kuultu. Odottavat äänet, kuten
                // tuloksen puhe, säilyvät, koska niiden kohta on vielä alussa.
                let vanhin = (0..self.soivat.len()).max_by_key(|&i| self.soivat[i].kohta);
                if let Some(i) = vanhin {
                    self.soivat.swap_remove(i);
                }
            }
            self.soivat.push(Soitto {
                indeksi: aani.indeksi(kieli::nykyinen()),
                viive: viive_ms as usize * NAYTETAAJUUS / 1000,
                kohta: 0,
            });
        }
    }

    /// Täyttää äänilaitteen puskurin seuraavilla näytteillä.
    pub fn tayta(&mut self, ulos: &mut [f32]) {
        ulos.fill(0.0);
        for soitto in &mut self.soivat {
            // Viive ohitetaan kerralla eikä näyte kerrallaan: äänisäie ehtii heikollakin koneella.
            let alku = soitto.viive.min(ulos.len());
            soitto.viive -= alku;
            let jaljella = self.naytteet[soitto.indeksi]
                .get(soitto.kohta..)
                .unwrap_or_default();
            let maara = (ulos.len() - alku).min(jaljella.len());
            for (paikka, &nayte) in ulos[alku..alku + maara].iter_mut().zip(jaljella) {
                *paikka += nayte as f32 * (VOIMAKKUUS / 32768.0);
            }
            soitto.kohta += maara;
        }
        let naytteet = &self.naytteet;
        self.soivat.retain(|s| s.kohta < naytteet[s.indeksi].len());
        for nayte in ulos {
            *nayte = nayte.clamp(-1.0, 1.0);
        }
    }

    /// Äänen näytteet tiedostoon kirjoittamista varten.
    #[cfg(debug_assertions)]
    pub fn naytteet(&self, aani: Aani, kieli: Kieli) -> &[i16] {
        &self.naytteet[aani.indeksi(kieli)]
    }
}

fn tehoste(aani: Aani, arpa: &mut Arpa) -> Vec<f32> {
    match aani {
        Aani::Alku => alku(),
        Aani::Laukaus => laukaus(arpa),
        Aani::Vastalaukaus => vastalaukaus(arpa),
        Aani::Roiske => roiske(arpa),
        Aani::Rajahdys => pamaus(arpa, 0.45, 6.5),
        Aani::Upotus => upotus(arpa),
        Aani::Virhe => virhe(),
        Aani::Voitto => voitto(arpa),
        Aani::Tappio => tappio(),
        Aani::Liuku => liuku(arpa),
        Aani::Yhdistys => yhdistys(),
        Aani::Pomppu => pomppu(),
        Aani::Tiili => tiili(),
        Aani::Ammus => ammus(),
        Aani::Kivi => kivi(arpa),
        Aani::Moottori => moottori(arpa),
        Aani::Lautanen => lautanen(),
        Aani::Omena => omena(),
        Aani::Puhkus => puhkus(arpa),
        Aani::Surina => surina(),
        Aani::Helina1 => helina(523.25),
        Aani::Helina2 => helina(587.33),
        Aani::Helina3 => helina(659.25),
        Aani::Helina4 => helina(783.99),
        Aani::Helina5 => helina(880.0),
        Aani::Helina6 => helina(1046.5),
        Aani::Sumutorvi => sumutorvi(),
        Aani::Laivakello => laivakello(),
        Aani::Karille => karille(arpa),
        Aani::Tuuli => tuuli(arpa),
        Aani::Kuokka => kuokka(arpa),
        Aani::Ritina => ritina(arpa),
        Aani::Sortuma => sortuma(arpa),
        Aani::EiSiirtoja | Aani::PeliOhi | Aani::Voitit => Vec::new(),
    }
}

// ---- rakennuspalat ----

fn kohina(arpa: &mut Arpa) -> f32 {
    arpa.alle(65536) as f32 / 32768.0 - 1.0
}

/// Oskillaattorin vaihe 0..1. Taajuus voi vaihtua näytteittäin ilman napsahduksia.
#[derive(Clone, Copy, Default)]
struct Vaihe(f32);

impl Vaihe {
    fn askel(&mut self, taajuus: f32) -> f32 {
        let vaihe = self.0;
        self.0 = (self.0 + taajuus / FS).fract();
        vaihe
    }
}

fn sini(vaihe: f32) -> f32 {
    (TAU * vaihe).sin()
}

fn pulssi(vaihe: f32, leveys: f32) -> f32 {
    if vaihe < leveys { 1.0 } else { -1.0 }
}

fn kolmio(vaihe: f32) -> f32 {
    1.0 - 4.0 * (vaihe - 0.5).abs()
}

/// Mega Driven tapainen kahden operaattorin FM: moduloija heiluttaa kantajan vaihetta.
fn fm(kantaja: f32, moduloija: f32, syvyys: f32) -> f32 {
    (TAU * kantaja + syvyys * sini(moduloija)).sin()
}

/// Nopea nousu ja eksponentiaalinen sammuminen.
fn verho(t: f32, nousu: f32, lasku: f32) -> f32 {
    (t / nousu).min(1.0) * (-t * lasku).exp()
}

/// Lyhyen piippauksen verho: nousee millisekunnissa ja laskee suoraan nollaan.
fn lyhyt(t: f32, kesto: f32) -> f32 {
    (t / 0.001).min(1.0) * (1.0 - t / kesto).max(0.0)
}

/// PSG-piirin tapainen kohina: arvo vaihtuu taajuuden tahdissa, joten matala taajuus rouhii.
#[derive(Default)]
struct Rouhe {
    arvo: f32,
    laskuri: f32,
}

impl Rouhe {
    fn askel(&mut self, arpa: &mut Arpa, taajuus: f32) -> f32 {
        self.laskuri += taajuus / FS;
        if self.laskuri >= 1.0 {
            self.laskuri = self.laskuri.fract();
            self.arvo = if arpa.alle(2) == 0 { 1.0 } else { -1.0 };
        }
        self.arvo
    }
}

/// Klattin resonaattori: yksi formantti, vahvistus 1 tasavirralla.
#[derive(Clone, Copy, Default)]
struct Resonaattori {
    a: f32,
    b: f32,
    c: f32,
    y1: f32,
    y2: f32,
}

impl Resonaattori {
    fn aseta(&mut self, taajuus: f32, kaista: f32) {
        let r = (-PI * kaista / FS).exp();
        self.c = -r * r;
        self.b = 2.0 * r * (TAU * taajuus / FS).cos();
        self.a = 1.0 - self.b - self.c;
    }

    fn kasittele(&mut self, x: f32) -> f32 {
        let y = self.a * x + self.b * self.y1 + self.c * self.y2;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}

/// Kaistanpäästösuodin, jonka vahvistus huipussa on 1.
#[derive(Clone, Copy, Default)]
struct Kaista {
    b0: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl Kaista {
    fn aseta(&mut self, taajuus: f32, kaista: f32) {
        let taajuus = taajuus.clamp(20.0, FS * 0.45);
        let w = TAU * taajuus / FS;
        let q = (taajuus / kaista).max(0.3);
        let alfa = w.sin() / (2.0 * q);
        let a0 = 1.0 + alfa;
        self.b0 = alfa / a0;
        self.a1 = -2.0 * w.cos() / a0;
        self.a2 = (1.0 - alfa) / a0;
    }

    fn kasittele(&mut self, x: f32) -> f32 {
        let y = self.b0 * (x - self.x2) - self.a1 * self.y1 - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}

fn rakenna(kesto: f32, mut aalto: impl FnMut(f32) -> f32) -> Vec<f32> {
    (0..(kesto * FS) as usize)
        .map(|i| aalto(i as f32 / FS))
        .collect()
}

fn skaalaa(mut aani: Vec<f32>, kerroin: f32) -> Vec<f32> {
    aani.iter_mut().for_each(|x| *x *= kerroin);
    aani
}

/// Lisää äänen `b` äänen `a` päälle hetkestä `alkaa` (sekunteina).
fn sekoita(mut a: Vec<f32>, b: &[f32], alkaa: f32) -> Vec<f32> {
    let alku = (alkaa * FS) as usize;
    if a.len() < alku + b.len() {
        a.resize(alku + b.len(), 0.0);
    }
    for (paikka, &x) in a[alku..].iter_mut().zip(b) {
        *paikka += x;
    }
    a
}

/// Soittaa sävelet soittimella: (alku sekunteina, taajuus, kesto).
fn savelkulku(savelet: &[(f32, f32, f32)], soitin: impl Fn(f32, f32) -> Vec<f32>) -> Vec<f32> {
    savelet
        .iter()
        .fold(Vec::new(), |kaikki, &(alku, taajuus, kesto)| {
            sekoita(kaikki, &soitin(taajuus, kesto), alku)
        })
}

/// SNESin kaiun tapainen toisto, jossa jokainen kierros tummuu.
fn kaiku(aani: Vec<f32>, viive: f32, palaute: f32, maara: f32) -> Vec<f32> {
    let d = (viive * FS) as usize;
    let kierroksia = (0.02f32.ln() / palaute.ln()).ceil() as usize;
    let pituus = aani.len() + d * kierroksia;
    let mut silmukka = vec![0.0; pituus];
    let mut ulos = vec![0.0; pituus];
    let mut tumma = 0.0;
    for n in 0..pituus {
        let x = aani.get(n).copied().unwrap_or(0.0);
        let takaisin = if n >= d { silmukka[n - d] } else { 0.0 };
        tumma += (takaisin - tumma) * 0.35;
        silmukka[n] = x + tumma * palaute;
        ulos[n] = x + tumma * maara;
    }
    ulos
}

/// Leikkaa hiljaisen hännän, häivyttää lopun napsahduksetta ja muuntaa 16-bittiseksi.
fn valmis(mut aani: Vec<f32>) -> Vec<i16> {
    let kuuluva = aani
        .iter()
        .rposition(|x| x.abs() > 0.002)
        .map_or(0, |i| i + 1);
    aani.truncate(kuuluva);
    let reuna = ((0.004 * FS) as usize).min(aani.len());
    let pituus = aani.len();
    for i in 0..reuna {
        aani[pituus - 1 - i] *= i as f32 / reuna as f32;
    }
    aani.iter()
        .map(|x| (x.clamp(-1.0, 1.0) * 32767.0) as i16)
        .collect()
}

// ---- soittimet ----

/// FM-kello: kolminkertainen moduloija antaa lasimaisen kilahduksen, joka pehmenee nopeasti.
fn kello(taajuus: f32, kesto: f32) -> Vec<f32> {
    let (mut kantaja, mut moduloija) = (Vaihe::default(), Vaihe::default());
    rakenna(kesto, |t| {
        let syvyys = 1.8 * (-t * 18.0).exp() + 0.25;
        fm(
            kantaja.askel(taajuus),
            moduloija.askel(taajuus * 3.0),
            syvyys,
        ) * verho(t, 0.001, 4.0 / kesto)
    })
}

/// FM-torvi: kirkas puhallus alussa ja vibrato pidetyssä sävelessä.
fn torvi(taajuus: f32, kesto: f32, kirkkaus: f32) -> Vec<f32> {
    let (mut kantaja, mut moduloija) = (Vaihe::default(), Vaihe::default());
    let haivytys = 0.06;
    rakenna(kesto + haivytys, |t| {
        let vibrato = 1.0 + 0.007 * (TAU * 6.0 * t).sin() * ((t - 0.08) / 0.1).clamp(0.0, 1.0);
        let f = taajuus * vibrato;
        let syvyys = kirkkaus * (0.6 * (t / 0.02).min(1.0) + 0.4 * (-t * 4.0).exp());
        let loppu = if t > kesto {
            1.0 - (t - kesto) / haivytys
        } else {
            1.0
        };
        let voima = (t / 0.006).min(1.0) * (1.0 - 0.25 * (t / kesto).min(1.0)) * loppu;
        fm(kantaja.askel(f), moduloija.askel(f), syvyys) * voima
    })
}

/// FM-basso: näpäkkä alku, jonka kirkkaus sammuu heti.
fn basso(taajuus: f32, kesto: f32) -> Vec<f32> {
    let (mut kantaja, mut moduloija) = (Vaihe::default(), Vaihe::default());
    rakenna(kesto, |t| {
        let syvyys = 2.2 * (-t * 10.0).exp() + 0.6;
        fm(kantaja.askel(taajuus), moduloija.askel(taajuus), syvyys) * verho(t, 0.002, 3.0 / kesto)
    })
}

fn virveli(arpa: &mut Arpa) -> Vec<f32> {
    let mut rouhe = Rouhe::default();
    let mut runko = Vaihe::default();
    rakenna(0.15, |t| {
        rouhe.askel(arpa, 12000.0) * verho(t, 0.0005, 25.0)
            + sini(runko.askel(190.0)) * verho(t, 0.0005, 40.0) * 0.6
    })
}

/// Kanttiaaltopiippaus, kuten konsolin PSG-kanavassa.
fn piippaus(taajuus: f32, kesto: f32, leveys: f32) -> Vec<f32> {
    let mut vaihe = Vaihe::default();
    rakenna(kesto, |t| {
        pulssi(vaihe.askel(taajuus), leveys) * lyhyt(t, kesto)
    })
}

// ---- äänet ----

fn alku() -> Vec<f32> {
    let kulku = savelkulku(
        &[
            (0.0, 523.25, 0.12),
            (0.05, 659.25, 0.12),
            (0.10, 783.99, 0.12),
            (0.15, 1046.5, 0.3),
        ],
        kello,
    );
    let kulku = sekoita(
        skaalaa(kulku, 0.3),
        &skaalaa(basso(130.81, 0.25), 0.3),
        0.15,
    );
    kaiku(kulku, 0.11, 0.35, 0.4)
}

fn laukaus(arpa: &mut Arpa) -> Vec<f32> {
    let mut runko = Vaihe::default();
    let mut rouhe = Rouhe::default();
    let mut suodin = 0.0;
    rakenna(0.3, |t| {
        let isku = sini(runko.askel(150.0 * (-t * 25.0).exp() + 42.0)) * verho(t, 0.001, 12.0);
        let r = rouhe.askel(arpa, 9000.0 * (-t * 10.0).exp() + 400.0);
        suodin += (r - suodin) * (0.6 * (-t * 8.0).exp() + 0.05);
        let pamaus = suodin * verho(t, 0.0005, 16.0);
        (2.0 * (isku * 0.9 + pamaus * 0.7)).tanh() * 0.8
    })
}

fn vastalaukaus(arpa: &mut Arpa) -> Vec<f32> {
    let kesto = 0.25;
    let mut vaihe = Vaihe::default();
    let mut suhina = Kaista::default();
    suhina.aseta(2000.0, 1500.0);
    rakenna(kesto, |t| {
        let f = (500.0 + 900.0 * (-t * 9.0).exp()) * (1.0 + 0.03 * (TAU * 30.0 * t).sin());
        let voima = (t / 0.02).min(1.0) * (1.0 - t / kesto);
        (kolmio(vaihe.askel(f)) * 0.4 + suhina.kasittele(kohina(arpa)) * 0.35) * voima
    })
}

fn roiske(arpa: &mut Arpa) -> Vec<f32> {
    let mut suhina = Kaista::default();
    let mut plop = Vaihe::default();
    rakenna(0.32, |t| {
        let keski = 2500.0 * (-t * 6.0).exp() + 500.0;
        suhina.aseta(keski, keski * 0.8);
        let kohisee = suhina.kasittele(kohina(arpa)) * verho(t, 0.004, 9.0) * 1.1;
        let plop =
            sini(plop.askel(600.0 * (-t * 30.0).exp() + 180.0)) * verho(t, 0.001, 25.0) * 0.4;
        (kohisee + plop) * 0.8
    })
}

/// Räjähdys: PSG-kohinan tahti laskee, joten pamaus madaltuu jyrinäksi.
fn pamaus(arpa: &mut Arpa, kesto: f32, lasku: f32) -> Vec<f32> {
    let mut rouhe = Rouhe::default();
    let mut suodin = 0.0;
    let mut runko = Vaihe::default();
    rakenna(kesto, |t| {
        let r = rouhe.askel(arpa, 7000.0 * (-t * 7.0).exp() + 300.0);
        suodin += (r - suodin) * (0.5 * (-t * 5.0).exp() + 0.08);
        let jyly = sini(runko.askel(110.0 * (-t * 15.0).exp() + 38.0)) * verho(t, 0.001, 14.0);
        (1.8 * (suodin * verho(t, 0.001, lasku) + jyly * 0.8)).tanh() * 0.8
    })
}

fn upotus(arpa: &mut Arpa) -> Vec<f32> {
    let iso = pamaus(arpa, 0.75, 4.5);
    let (mut kantaja, mut moduloija) = (Vaihe::default(), Vaihe::default());
    let jymina = rakenna(0.8, |t| {
        let f = 180.0 * (-t * 3.0).exp() + 45.0;
        let syvyys = 3.0 * (-t * 3.0).exp() + 0.5;
        fm(kantaja.askel(f), moduloija.askel(f), syvyys) * verho(t, 0.01, 3.2) * 0.55
    });
    let rasahdys = skaalaa(pamaus(arpa, 0.2, 18.0), 0.6);
    let kaikki = sekoita(sekoita(iso, &jymina, 0.0), &rasahdys, 0.13);
    let kaikki = kaikki.into_iter().map(|x| x.tanh() * 0.8).collect();
    kaiku(kaikki, 0.09, 0.3, 0.3)
}

fn virhe() -> Vec<f32> {
    let kulku = savelkulku(&[(0.0, 196.0, 0.05), (0.065, 196.0, 0.06)], |f, k| {
        piippaus(f, k, 0.25)
    });
    skaalaa(kulku, 0.35)
}

fn voitto(arpa: &mut Arpa) -> Vec<f32> {
    let melodia = savelkulku(
        &[
            (0.0, 392.0, 0.07),
            (0.08, 523.25, 0.07),
            (0.16, 659.25, 0.07),
            (0.24, 783.99, 0.45),
        ],
        |f, k| torvi(f, k, 2.4),
    );
    let sointu = savelkulku(&[(0.24, 523.25, 0.45), (0.24, 659.25, 0.45)], |f, k| {
        torvi(f, k, 1.6)
    });
    let pohja = savelkulku(&[(0.0, 196.0, 0.2), (0.24, 130.81, 0.45)], basso);
    let kaikki = sekoita(skaalaa(melodia, 0.3), &skaalaa(sointu, 0.15), 0.0);
    let kaikki = sekoita(kaikki, &skaalaa(pohja, 0.3), 0.0);
    let kaikki = sekoita(kaikki, &skaalaa(virveli(arpa), 0.3), 0.24);
    kaiku(skaalaa(kaikki, 0.85), 0.12, 0.3, 0.35)
}

fn tappio() -> Vec<f32> {
    let melodia = savelkulku(
        &[
            (0.0, 392.0, 0.12),
            (0.14, 369.99, 0.12),
            (0.28, 349.23, 0.12),
            (0.42, 329.63, 0.3),
        ],
        |f, k| torvi(f, k, 1.3),
    );
    let pohja = savelkulku(&[(0.0, 98.0, 0.4), (0.42, 82.41, 0.35)], basso);
    let kaikki = sekoita(skaalaa(melodia, 0.32), &skaalaa(pohja, 0.3), 0.0);
    kaiku(kaikki, 0.12, 0.3, 0.3)
}

fn liuku(arpa: &mut Arpa) -> Vec<f32> {
    let mut runko = Vaihe::default();
    let mut suodin = 0.0;
    rakenna(0.06, |t| {
        suodin += (kohina(arpa) - suodin) * 0.3;
        sini(runko.askel(330.0 * (-t * 40.0).exp() + 140.0)) * verho(t, 0.0005, 60.0) * 0.45
            + suodin * verho(t, 0.0003, 250.0) * 0.5
    })
}

fn yhdistys() -> Vec<f32> {
    skaalaa(
        savelkulku(&[(0.0, 987.77, 0.1), (0.05, 1318.5, 0.16)], kello),
        0.28,
    )
}

fn pomppu() -> Vec<f32> {
    let kesto = 0.045;
    let mut vaihe = Vaihe::default();
    rakenna(kesto, |t| {
        pulssi(vaihe.askel(523.25 * (1.0 - t * 3.0)), 0.125) * lyhyt(t, kesto) * 0.3
    })
}

fn tiili() -> Vec<f32> {
    let kesto = 0.07;
    let (mut kantti, mut kirkas) = (Vaihe::default(), Vaihe::default());
    rakenna(kesto, |t| {
        let aalto = pulssi(kantti.askel(1046.5), 0.25) * 0.7 + sini(kirkas.askel(2093.0)) * 0.3;
        aalto * verho(t, 0.0005, 30.0) * (1.0 - t / kesto) * 0.35
    })
}

fn ammus() -> Vec<f32> {
    let kesto = 0.13;
    let (mut kantaja, mut moduloija) = (Vaihe::default(), Vaihe::default());
    rakenna(kesto, |t| {
        let f = 2000.0 * (-t * 22.0).exp() + 150.0;
        fm(kantaja.askel(f), moduloija.askel(f * 0.5), 2.5) * (1.0 - t / kesto) * 0.3
    })
}

fn kivi(arpa: &mut Arpa) -> Vec<f32> {
    let mut rouhe = Rouhe::default();
    let mut suodin = 0.0;
    rakenna(0.2, |t| {
        suodin += (rouhe.askel(arpa, 4000.0 * (-t * 12.0).exp() + 250.0) - suodin) * 0.45;
        (1.5 * suodin * verho(t, 0.0005, 16.0)).tanh() * 0.75
    })
}

/// Soitetaan 120 ms:n välein, joten 130 ms:n pätkät menevät hieman limittäin.
fn moottori(arpa: &mut Arpa) -> Vec<f32> {
    let mut rouhe = Rouhe::default();
    let mut suodin = 0.0;
    rakenna(0.13, |t| {
        suodin += (rouhe.askel(arpa, 1100.0) - suodin) * 0.25;
        suodin * (t / 0.005).min(1.0) * 0.25
    })
}

/// Soitetaan 300 ms:n välein. Ulina heilahtaa tasan kolme kertaa, joten pätkät jatkuvat saumatta.
fn lautanen() -> Vec<f32> {
    let mut vaihe = Vaihe::default();
    rakenna(0.3, |t| {
        pulssi(vaihe.askel(900.0 + 250.0 * (TAU * 10.0 * t).sin()), 0.5)
            * (t / 0.004).min(1.0)
            * 0.1
    })
}

fn omena() -> Vec<f32> {
    let kulku = savelkulku(&[(0.0, 783.99, 0.035), (0.035, 1174.66, 0.045)], |f, k| {
        piippaus(f, k, 0.25)
    });
    skaalaa(kulku, 0.28)
}

fn puhkus(arpa: &mut Arpa) -> Vec<f32> {
    let kesto = 0.25;
    let mut suhina = Kaista::default();
    rakenna(kesto, |t| {
        let auki = (PI * t / kesto).sin();
        suhina.aseta(400.0 + 2600.0 * auki, 1500.0);
        suhina.kasittele(kohina(arpa)) * auki.powf(1.5) * 1.2
    })
}

fn surina() -> Vec<f32> {
    let kesto = 0.22;
    let mut vaihe = Vaihe::default();
    rakenna(kesto, |t| {
        let f = if ((t / 0.025) as usize).is_multiple_of(2) {
            659.25
        } else {
            880.0
        };
        pulssi(vaihe.askel(f * (1.0 + t)), 0.25) * (t / 0.003).min(1.0) * (1.0 - t / kesto) * 0.22
    })
}

/// Pentatoninen asteikko kuulostaa hyvältä missä tahansa järjestyksessä.
fn helina(taajuus: f32) -> Vec<f32> {
    kaiku(skaalaa(kello(taajuus, 0.35), 0.3), 0.08, 0.25, 0.25)
}

/// Kaksi matalaa torvea kvintin päässä toisistaan, kuten majakan diafoni.
fn sumutorvi() -> Vec<f32> {
    let kesto = 1.05;
    let ala = torvi(73.4, kesto, 1.1);
    let yla = skaalaa(torvi(110.0, kesto, 0.8), 0.6);
    let yhdessa = sekoita(ala, &yla, 0.0);
    skaalaa(kaiku(yhdessa, 0.12, 0.3, 0.2), 0.45)
}

fn laivakello() -> Vec<f32> {
    let kulku = savelkulku(&[(0.0, 1318.5, 0.5), (0.2, 1318.5, 0.5)], kello);
    skaalaa(kaiku(kulku, 0.09, 0.25, 0.2), 0.3)
}

/// Rouhiva rusahdus ja matala tömähdys.
fn karille(arpa: &mut Arpa) -> Vec<f32> {
    let kesto = 0.7;
    let mut rouhe = Rouhe::default();
    let mut suodin = 0.0;
    let rusahdus = rakenna(kesto, |t| {
        let taajuus = 900.0 * (-t * 5.0).exp() + 90.0;
        suodin += (rouhe.askel(arpa, taajuus) - suodin) * 0.3;
        suodin * verho(t, 0.002, 5.0) * 0.8
    });
    let tomahdys = skaalaa(basso(55.0, 0.5), 0.6);
    sekoita(rusahdus, &tomahdys, 0.0)
}

/// Suodatettu kohina, jonka kaista nousee ja laskee puuskan mukana.
fn tuuli(arpa: &mut Arpa) -> Vec<f32> {
    let kesto = 1.4;
    let mut suhina = Kaista::default();
    rakenna(kesto, |t| {
        let auki = (PI * t / kesto).sin();
        suhina.aseta(300.0 + 700.0 * auki, 400.0);
        suhina.kasittele(kohina(arpa)) * auki * auki * 0.9
    })
}

/// Metallinen kilahdus ja soran rapina.
fn kuokka(arpa: &mut Arpa) -> Vec<f32> {
    let (mut kantaja, mut moduloija) = (Vaihe::default(), Vaihe::default());
    let mut rouhe = Rouhe::default();
    rakenna(0.1, |t| {
        let kilahdus =
            fm(kantaja.askel(1850.0), moduloija.askel(2590.0), 1.6) * verho(t, 0.0005, 45.0);
        let rapina = rouhe.askel(arpa, 6000.0) * verho(t, 0.0005, 60.0);
        (kilahdus * 0.5 + rapina * 0.35) * 0.55
    })
}

/// Huojuva puun narina ja sen päällä satunnaisia naksahduksia.
fn ritina(arpa: &mut Arpa) -> Vec<f32> {
    let kesto = 0.5;
    let mut runko = Vaihe::default();
    let mut naksu = 0.0f32;
    rakenna(kesto, |t| {
        let taajuus = 120.0 + 35.0 * (TAU * 6.0 * t).sin();
        let narina = (runko.askel(taajuus) * 2.0 - 1.0) * (PI * t / kesto).sin() * 0.22;
        if arpa.alle(700) == 0 {
            naksu = 1.0;
        }
        naksu *= 0.993;
        (narina + naksu * kohina(arpa) * 0.5) * 0.5
    })
}

/// Matala jyrinä ja perään valuvaa soraa.
fn sortuma(arpa: &mut Arpa) -> Vec<f32> {
    let jyly = pamaus(arpa, 0.5, 5.0);
    let mut rouhe = Rouhe::default();
    let mut suodin = 0.0;
    let sora = rakenna(0.7, |t| {
        suodin += (rouhe.askel(arpa, 900.0 * (-t * 3.0).exp() + 150.0) - suodin) * 0.2;
        suodin * verho(t, 0.02, 4.0) * 0.7
    });
    skaalaa(sekoita(jyly, &sora, 0.05), 0.7)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn soi(mikseri: &mut Mikseri) -> bool {
        // Puhe voi alkaa klusiilin hiljaisella sulkeumalla, joten katsotaan 0,1 s.
        let mut puskuri = [0.0; NAYTETAAJUUS / 10];
        mikseri.tayta(&mut puskuri);
        puskuri.iter().any(|&nayte| nayte != 0.0)
    }

    #[test]
    fn jokainen_aani_soi_ja_loppuu() {
        let mut mikseri = Mikseri::new();
        kieli::kaikilla_kielilla(|| {
            for aani in KAIKKI {
                mikseri.soita(aani, 0);
                assert!(soi(&mut mikseri), "{aani:?} on hiljaa");
                let mut puskuri = vec![0.0; NAYTETAAJUUS * 3];
                mikseri.tayta(&mut puskuri);
                assert!(mikseri.soivat.is_empty(), "{aani:?} ei lopu");
                assert!(puskuri.iter().all(|nayte| nayte.abs() <= 1.0));
            }
        });
    }

    #[test]
    fn tehosteet_ovat_lyhyita_ja_puhe_omissa_paikoissaan() {
        let mikseri = Mikseri::new();
        assert_eq!(mikseri.naytteet.len(), TEHOSTEITA + 2 * PUHEITA);
        for aani in KAIKKI {
            if aani.puhe().is_none() {
                assert!(aani.indeksi(Kieli::Suomi) < TEHOSTEITA, "{aani:?}");
                let kesto = mikseri.naytteet(aani, Kieli::Suomi).len() as f32 / FS;
                assert!(kesto <= 1.6, "{aani:?}: {kesto} s");
            } else {
                assert_ne!(
                    mikseri.naytteet(aani, Kieli::Suomi),
                    mikseri.naytteet(aani, Kieli::Englanti),
                );
            }
        }
    }

    #[test]
    fn puhe_soi_valitulla_kielella() {
        let mut mikseri = Mikseri::new();
        let pituus = |kieli| mikseri.naytteet(Aani::EiSiirtoja, kieli).len();
        let (suomi, englanti) = (pituus(Kieli::Suomi), pituus(Kieli::Englanti));
        kieli::aseta(Kieli::Englanti);
        mikseri.soita(Aani::EiSiirtoja, 0);
        kieli::aseta(Kieli::Suomi);
        assert_eq!(mikseri.naytteet[mikseri.soivat[0].indeksi].len(), englanti);
        assert_ne!(suomi, englanti);
    }

    #[test]
    fn pois_kytkeminen_hiljentaa_soivan_ja_odottavan_aanen_heti() {
        let mut mikseri = Mikseri::new();
        mikseri.soita(Aani::Tappio, 0);
        mikseri.soita(Aani::Voitto, 1000);
        assert!(soi(&mut mikseri));
        mikseri.aseta_paalle(false);
        assert!(!soi(&mut mikseri));

        mikseri.soita(Aani::Laukaus, 0);
        assert!(!soi(&mut mikseri));

        // takaisin kytkettynä vanhat äänet eivät palaa, uudet soivat
        mikseri.aseta_paalle(true);
        assert!(!soi(&mut mikseri));
        mikseri.soita(Aani::Laukaus, 0);
        assert!(soi(&mut mikseri));
    }

    #[test]
    fn viive_siirtaa_aanen_alkua() {
        let mut mikseri = Mikseri::new();
        mikseri.soita(Aani::Laukaus, 1000);
        let mut puskuri = vec![0.0; NAYTETAAJUUS - 1];
        mikseri.tayta(&mut puskuri);
        assert!(puskuri.iter().all(|&nayte| nayte == 0.0));
        assert!(soi(&mut mikseri));
    }

    #[test]
    fn puskurin_koko_ei_muuta_aanta() {
        let soita = |mikseri: &mut Mikseri| {
            mikseri.soita(Aani::Laukaus, 0);
            mikseri.soita(Aani::Roiske, 7);
            mikseri.soita(Aani::Voitto, 300);
        };
        let mut kerralla = Mikseri::new();
        soita(&mut kerralla);
        let mut odotettu = vec![0.0; NAYTETAAJUUS];
        kerralla.tayta(&mut odotettu);

        let mut paloina = Mikseri::new();
        soita(&mut paloina);
        let mut saatu = Vec::new();
        for koko in [1, 3, 100, 223, 1000].into_iter().cycle() {
            let koko = koko.min(NAYTETAAJUUS - saatu.len());
            if koko == 0 {
                break;
            }
            let mut pala = vec![0.0; koko];
            paloina.tayta(&mut pala);
            saatu.extend(pala);
        }
        assert_eq!(saatu, odotettu);
    }

    #[test]
    fn soivia_on_rajattu_maara_ja_odottava_puhe_sailyy() {
        let mut mikseri = Mikseri::new();
        mikseri.soita(Aani::Voitit, PUHE_VIIVE_MS);
        for _ in 0..3 * ENINTAAN_SOIVIA {
            mikseri.soita(Aani::Kivi, 0);
            let mut puskuri = vec![0.0; 64];
            mikseri.tayta(&mut puskuri);
        }
        assert_eq!(mikseri.soivat.len(), ENINTAAN_SOIVIA);
        let puhe = Aani::Voitit.indeksi(kieli::nykyinen());
        assert!(mikseri.soivat.iter().any(|s| s.indeksi == puhe));
    }
}
