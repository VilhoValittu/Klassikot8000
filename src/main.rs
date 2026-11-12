#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod kaynnistin;
#[cfg(debug_assertions)]
mod kehitys;
mod pelit;
mod virheloki;
mod yhteinen;

use std::mem::{ManuallyDrop, take};
use std::path::Path;
use std::process::{Command, ExitCode};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Scale, Window, WindowOptions};
use tinyaudio::{OutputDevice, OutputDeviceParameters, run_output_device};

#[cfg(not(debug_assertions))]
use kaynnistin::netti::Pyynto;
use kaynnistin::netti::{JULKAISUSIVU, Netti};
use kaynnistin::tallennus;
#[cfg(not(debug_assertions))]
use kaynnistin::tallennus::Lukitus;
use kaynnistin::{Kaynnistin, piirto};
use yhteinen::aanet::{Mikseri, NAYTETAAJUUS};
use yhteinen::kieli;
use yhteinen::piirto::{Ikkunakuva, Kehys, Sovitus};
use yhteinen::syote::Syote;

const PAIVITYKSIA_SEKUNNISSA: usize = 30;
const TOIMINTAPELIN_PAIVITYKSIA: usize = 60;
// Noin 32 ms: kytkin hiljentää äänet heti, mutta puskuri ei tyhjene kesken.
const AANIPUSKURI: usize = NAYTETAAJUUS / 32;
// Scale::FitScreen kertoo tämän suurimmalla kertoimella, joka mahtuu näytölle, ja vähentää
// yhden askeleen: ikkuna on 16:9 ja näytön reunaan jää tilaa otsikkopalkille ja tehtäväpalkille.
const IKKUNAN_POHJA: (usize, usize) = (160, 90);

/// Exen ikoni ei siirry ikkunaan itsestään, ja minifb lataa ikkunan ikonin vain tiedostosta.
/// Polku annetaan suoraan, koska minifb:n `Icon::from_str` vapauttaa sen ennen käyttöä.
#[cfg(windows)]
fn aseta_ikoni(ikkuna: &mut Window) {
    use std::os::windows::ffi::OsStrExt;

    let polku = std::env::temp_dir().join("klassikot8000.ico");
    if std::fs::write(&polku, include_bytes!("../paketointi/ikoni.ico")).is_ok() {
        let leveat: Vec<u16> = polku.as_os_str().encode_wide().chain(Some(0)).collect();
        ikkuna.set_icon(minifb::Icon::Path(leveat.as_ptr()));
    }
}

/// X11 haluaa ikonin lukuina: leveys, korkeus ja pikselit, kukin omassa 64-bittisessä luvussaan.
#[cfg(target_os = "linux")]
fn aseta_ikoni(ikkuna: &mut Window) {
    const SIVU: u64 = 48;
    let pikselit = include_bytes!("../paketointi/ikoni48.argb")
        .chunks_exact(4)
        .map(|p| u64::from(u32::from_le_bytes([p[0], p[1], p[2], p[3]])));
    let luvut: Vec<u64> = [SIVU, SIVU].into_iter().chain(pikselit).collect();
    if let Ok(ikoni) = minifb::Icon::try_from(luvut.as_slice()) {
        ikkuna.set_icon(ikoni);
    }
}

// macOS ottaa ikonin paketin Info.plististä, ja minifb:n set_icon kaatuisi siellä.
#[cfg(target_os = "macos")]
fn aseta_ikoni(_ikkuna: &mut Window) {}

fn aikasiemen() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(1, |aika| aika.as_nanos() as u64)
}

fn nappain_syotteeksi(nappain: Key) -> Option<Syote> {
    Some(match nappain {
        Key::Up => Syote::Nuoli(-1, 0),
        Key::Down => Syote::Nuoli(1, 0),
        Key::Left => Syote::Nuoli(0, -1),
        Key::Right => Syote::Nuoli(0, 1),
        Key::Space | Key::Enter | Key::NumPadEnter => Syote::Ammu,
        Key::Escape => Syote::Peru,
        Key::M => Syote::VaihdaAanet,
        Key::A => Syote::Kirjain('A'),
        Key::B => Syote::Kirjain('B'),
        Key::C => Syote::Kirjain('C'),
        Key::D => Syote::Kirjain('D'),
        Key::E => Syote::Kirjain('E'),
        Key::F => Syote::Kirjain('F'),
        Key::G => Syote::Kirjain('G'),
        Key::H => Syote::Kirjain('H'),
        Key::I => Syote::Kirjain('I'),
        Key::J => Syote::Kirjain('J'),
        Key::K => Syote::Kirjain('K'),
        Key::L => Syote::Kirjain('L'),
        Key::N => Syote::Kirjain('N'),
        Key::O => Syote::Kirjain('O'),
        Key::P => Syote::Kirjain('P'),
        Key::Q => Syote::Kirjain('Q'),
        Key::R => Syote::Kirjain('R'),
        Key::S => Syote::Kirjain('S'),
        Key::T => Syote::Kirjain('T'),
        Key::U => Syote::Kirjain('U'),
        Key::V => Syote::Kirjain('V'),
        Key::W => Syote::Kirjain('W'),
        Key::X => Syote::Kirjain('X'),
        Key::Y => Syote::Kirjain('Y'),
        Key::Z => Syote::Kirjain('Z'),
        // Suomalaisen näppäimistön paikat: minifb lukee näppäimen sijainnin.
        Key::Semicolon => Syote::Kirjain('Ö'),
        Key::Apostrophe => Syote::Kirjain('Ä'),
        Key::Backspace => Syote::Pyyhi,
        Key::Key0 | Key::NumPad0 => Syote::Numero(0),
        Key::Key1 | Key::NumPad1 => Syote::Numero(1),
        Key::Key2 | Key::NumPad2 => Syote::Numero(2),
        Key::Key3 | Key::NumPad3 => Syote::Numero(3),
        Key::Key4 | Key::NumPad4 => Syote::Numero(4),
        Key::Key5 | Key::NumPad5 => Syote::Numero(5),
        Key::Key6 | Key::NumPad6 => Syote::Numero(6),
        Key::Key7 | Key::NumPad7 => Syote::Numero(7),
        Key::Key8 | Key::NumPad8 => Syote::Numero(8),
        Key::Key9 | Key::NumPad9 => Syote::Numero(9),
        _ => return None,
    })
}

fn paivitystaajuus(kaynnistin: &Kaynnistin) -> usize {
    if kaynnistin.toimintapeli() {
        TOIMINTAPELIN_PAIVITYKSIA
    } else {
        PAIVITYKSIA_SEKUNNISSA
    }
}

/// Avaa äänilaitteen. Jos sitä ei ole, peli toimii hiljaa.
fn avaa_aanilaite(mikseri: Arc<Mutex<Mikseri>>) -> Option<OutputDevice> {
    let parametrit = OutputDeviceParameters {
        sample_rate: NAYTETAAJUUS,
        channels_count: 1,
        channel_sample_count: AANIPUSKURI,
    };
    run_output_device(parametrit, move |ulos| match mikseri.lock() {
        Ok(mut mikseri) => mikseri.tayta(ulos),
        Err(_) => ulos.fill(0.0),
    })
    .ok()
}

/// Epäonnistunut tallennus ei saa kaataa peliä.
fn tallenna(polku: Option<&Path>, kaynnistin: &Kaynnistin) {
    if let Some(polku) = polku
        && let Err(virhe) = tallennus::kirjoita_tiedosto(polku, &kaynnistin.tallennus)
    {
        virheloki::kirjaa(&format!("{}: {virhe}", polku.display()));
    }
}

/// Avaa osoitteen oletusselaimeen. Epäonnistuminen ei haittaa peliä, joten siitä vain kerrotaan.
fn avaa_selaimessa(osoite: &str) {
    let ohjelma = if cfg!(windows) {
        "explorer"
    } else if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    match Command::new(ohjelma).arg(osoite).spawn() {
        // Odotus omassa säikeessään, jottei päättynyt prosessi jää roikkumaan eikä peli nyi.
        Ok(mut lapsi) => {
            thread::spawn(move || lapsi.wait());
        }
        Err(virhe) => virheloki::kirjaa(&format!("{ohjelma}: {virhe}")),
    }
}

/// Välittää pyynnöt verkkosäikeelle ja ottaa sen vastaukset. Ilman verkkoa pyynnöt vain unohtuvat.
fn vaihda_netin_kanssa(kaynnistin: &mut Kaynnistin, netti: Option<&Netti>) {
    let pyynnot = take(&mut kaynnistin.pyynnot);
    if let Some(netti) = netti {
        for pyynto in pyynnot {
            netti.laheta(pyynto);
        }
        while let Ok(vastaus) = netti.vastaukset.try_recv() {
            kaynnistin.vastaanota(vastaus);
        }
    }
}

/// Painetut näppäimet syötteiksi. Pohjassa pidetty näppäin toistuu vain nuolissa, ettei
/// pohjassa pidetty välilyönti ammu sarjaa.
fn lue_nappaimet(ikkuna: &Window, kaynnistin: &mut Kaynnistin) {
    for nappain in ikkuna.get_keys_pressed(KeyRepeat::Yes) {
        if let Some(syote) = nappain_syotteeksi(nappain) {
            let toistuva = matches!(syote, Syote::Nuoli(..)) && kaynnistin.nuolet_toistuvat();
            if toistuva || ikkuna.is_key_pressed(nappain, KeyRepeat::No) {
                kaynnistin.kasittele(syote);
            }
        }
    }
}

/// Hiiren tila edelliseltä kierrokselta, jotta liike, painallus ja vapautus erottuvat.
#[derive(Default)]
struct Hiiri {
    paikka: Option<(i32, i32)>,
    vasen_alhaalla: bool,
    oikea_alhaalla: bool,
}

impl Hiiri {
    fn lue(&mut self, ikkuna: &Window, sovitus: Sovitus, kaynnistin: &mut Kaynnistin) {
        // Kursori luetaan myös ikkunan ulkopuolelta: ruudun reunassa se on ikkunan
        // koonmuutosreunan päällä, ja oikealla ikkunan vieressä on muita ohjelmia.
        let kursori = ikkuna
            .get_unscaled_mouse_pos(MouseMode::Pass)
            .map(|(x, y)| (x as i32, y as i32));
        let hiiren_paikka = kursori.and_then(|(x, y)| sovitus.ruudulle(x, y));
        let liike = match (hiiren_paikka, kursori) {
            (None, Some((x, y))) if ikkuna.is_active() && kaynnistin.hiiri_ulkonakin() => {
                Some(sovitus.ruudulle_rajattu(x, y))
            }
            _ => hiiren_paikka,
        };
        if let Some(piste) = liike
            && liike != self.paikka
        {
            self.paikka = liike;
            let nakyma = kaynnistin.nakyman_piste(piste.0, piste.1);
            kaynnistin.kasittele(Syote::Hiiri(nakyma.0, nakyma.1));
        }

        // Napit vain ikkunan sisältä: muualla klikkaus kuuluu toiselle ohjelmalle.
        let Some(piste) = hiiren_paikka else {
            self.vasen_alhaalla = false;
            self.oikea_alhaalla = false;
            return;
        };
        let (x, y) = kaynnistin.nakyman_piste(piste.0, piste.1);
        let vasen = ikkuna.get_mouse_down(MouseButton::Left);
        if vasen && !self.vasen_alhaalla {
            kaynnistin.kasittele(Syote::Klikkaus(x, y));
        } else if !vasen && self.vasen_alhaalla {
            kaynnistin.kasittele(Syote::Vapautus(x, y));
        }
        self.vasen_alhaalla = vasen;
        let oikea = ikkuna.get_mouse_down(MouseButton::Right);
        if oikea && !self.oikea_alhaalla {
            kaynnistin.kasittele(Syote::OikeaKlikkaus(x, y));
        }
        self.oikea_alhaalla = oikea;
        // Rulla ylös vie listassa ylöspäin.
        if let Some((_, rulla)) = ikkuna.get_scroll_wheel()
            && rulla != 0.0
        {
            kaynnistin.kasittele(Syote::Rulla(if rulla > 0.0 { -1 } else { 1 }));
        }
    }
}

/// Pohjassa olevat suunnat toimintapeleille. WASD käy nuolten tavoin, kuten Silmukassa.
fn lue_ohjaus(ikkuna: &Window, kaynnistin: &mut Kaynnistin) {
    let pohjassa = |nappaimet: [Key; 2]| nappaimet.iter().any(|&n| ikkuna.is_key_down(n));
    kaynnistin.aseta_ohjaus(
        pohjassa([Key::Left, Key::A]),
        pohjassa([Key::Right, Key::D]),
        pohjassa([Key::Up, Key::W]),
        pohjassa([Key::Down, Key::S]),
        ikkuna.get_mouse_down(MouseButton::Right),
    );
    if !ikkuna.is_active() {
        kaynnistin.keskeyta();
    }
}

/// Vie äänikytkimen tilan ja soittojonon mikserille.
fn soita_aanet(mikseri: &Mutex<Mikseri>, kaynnistin: &mut Kaynnistin) {
    if let Ok(mut mikseri) = mikseri.lock() {
        if take(&mut kaynnistin.hiljenna) {
            mikseri.aseta_paalle(false);
        }
        mikseri.aseta_paalle(kaynnistin.aanet_paalla);
        for (aani, viive_ms) in kaynnistin.ota_aanet() {
            mikseri.soita(aani, viive_ms);
        }
    }
}

/// Hoitaa käynnistimen pyynnöt pelin ulkopuolelle: verkon, tallennuksen ja selaimen.
fn hoida_pyynnot(kaynnistin: &mut Kaynnistin, polku: Option<&Path>, netti: Option<&Netti>) {
    vaihda_netin_kanssa(kaynnistin, netti);
    if take(&mut kaynnistin.tallenna) {
        tallenna(polku, kaynnistin);
    }
    if take(&mut kaynnistin.avaa_julkaisut) {
        avaa_selaimessa(JULKAISUSIVU);
    }
}

fn pelisilmukka(
    mut kaynnistin: Kaynnistin,
    kehys: &mut Kehys,
    polku: Option<&Path>,
    netti: Option<Netti>,
) -> Result<(), minifb::Error> {
    let asetukset = WindowOptions {
        resize: true,
        scale: Scale::FitScreen,
        ..WindowOptions::default()
    };
    let mut otsikko = kaynnistin.otsikko();
    let mut ikkuna = Window::new(otsikko, IKKUNAN_POHJA.0, IKKUNAN_POHJA.1, asetukset)?;
    aseta_ikoni(&mut ikkuna);
    // Oletuspaikassa ikkuna voisi jatkua ruudun yli, ja sulkemisnappi jäisi piiloon.
    ikkuna.set_position(0, 0);
    let mut taajuus = paivitystaajuus(&kaynnistin);
    ikkuna.set_target_fps(taajuus);
    let mut ikkunakuva = Ikkunakuva::default();

    let mikseri = Arc::new(Mutex::new(Mikseri::new()));
    // Laitetta ei suljeta itse: tinyaudion Drop voi kaataa pelin lopetuksessa, ja
    // käyttöjärjestelmä vapauttaa laitteen prosessin päättyessä joka tapauksessa.
    let _aanilaite = ManuallyDrop::new(avaa_aanilaite(Arc::clone(&mikseri)));

    let mut edellinen = Instant::now();
    let mut hiiri = Hiiri::default();
    let mut kuvan_tila = None;

    while ikkuna.is_open() && !kaynnistin.lopeta {
        // Pienennetyssä ikkunassa koko voi olla nolla.
        let (ikkuna_l, ikkuna_k) = ikkuna.get_size();
        let (ikkuna_l, ikkuna_k) = (ikkuna_l.max(1), ikkuna_k.max(1));
        let sovitus = Sovitus::new(ikkuna_l, ikkuna_k);

        lue_nappaimet(&ikkuna, &mut kaynnistin);
        hiiri.lue(&ikkuna, sovitus, &mut kaynnistin);
        lue_ohjaus(&ikkuna, &mut kaynnistin);
        soita_aanet(&mikseri, &mut kaynnistin);
        hoida_pyynnot(&mut kaynnistin, polku, netti.as_ref());

        if kaynnistin.otsikko() != otsikko {
            otsikko = kaynnistin.otsikko();
            ikkuna.set_title(otsikko);
        }
        if paivitystaajuus(&kaynnistin) != taajuus {
            taajuus = paivitystaajuus(&kaynnistin);
            ikkuna.set_target_fps(taajuus);
        }

        // Millisekunnin osat jäävät seuraavaan kierrokseen, jottei peli hidastu 60 Hz:llä.
        let kulunut = edellinen.elapsed().as_millis() as u32;
        kaynnistin.edista(kulunut);
        edellinen += Duration::from_millis(kulunut as u64);

        // Näkymää ei piirretä, kun sen tila ja ikkunan koko ovat ennallaan.
        let tila = kaynnistin
            .kuvan_avain(ikkuna.is_active())
            .map(|avain| (avain, ikkuna_l, ikkuna_k));
        let ennallaan = tila.is_some() && tila == kuvan_tila;
        kuvan_tila = tila;
        if !ennallaan {
            piirto::piirra(&kaynnistin, kehys);
        }
        // Windows piirtää peitosta paljastuneen ikkunan itse edellisestä puskurista, joten
        // muuttumatonta kuvaa ei tarvitse lähettää uudelleen.
        if !ennallaan
            && ikkunakuva.sovita(&kehys.ruutu, &kehys.muuttuneet, ikkuna_l, ikkuna_k, sovitus)
        {
            ikkuna.update_with_buffer(&ikkunakuva.pikselit, ikkuna_l, ikkuna_k)?;
        } else {
            ikkuna.update();
        }
    }
    // Ikkunan voi sulkea kesken pelin.
    kaynnistin.viimeistele();
    tallenna(polku, &kaynnistin);
    // Lähetys ei välttämättä ehdi perille, mutta tulos on jonossa seuraavaa käynnistystä varten.
    vaihda_netin_kanssa(&mut kaynnistin, netti.as_ref());
    Ok(())
}

fn main() -> ExitCode {
    let mut kehys = Kehys::default();
    if let Some(polku) = kieli::oletuspolku() {
        kieli::lue_tiedosto(&polku);
    }

    #[cfg(debug_assertions)]
    let (kaynnistin, polku, netti) = match kehitys::aloita(&mut kehys) {
        kehitys::Alku::Ikkuna(kaynnistin, polku, netti) => (*kaynnistin, polku, netti),
        kehitys::Alku::Valmis(koodi) => return koodi,
    };
    #[cfg(not(debug_assertions))]
    let (kaynnistin, polku, netti, _lukko) = {
        let polku = tallennus::oletuspolku();
        if let Some(polku) = &polku {
            virheloki::ota_kayttoon(polku.with_file_name("virheet.txt"));
        }
        // Lukko pidetään ohjelman loppuun asti.
        let lukko = match polku.as_deref().map(tallennus::lukitse) {
            // Peli on jo auki toisessa ikkunassa, ja kaksi ilmentymää kirjoittaisi toistensa
            // tallennuksen yli.
            Some(Lukitus::Varattu) => return ExitCode::SUCCESS,
            Some(Lukitus::Oma(lukko)) => Some(lukko),
            // Ilman lukkoa pelataan kuten ennenkin.
            Some(Lukitus::Puuttuu(virhe)) => {
                virheloki::kirjaa(&format!("tallennuksen lukko: {virhe}"));
                None
            }
            None => None,
        };
        let mut kaynnistin = Kaynnistin::new(aikasiemen());
        let polku = polku.and_then(|polku| {
            match tallennus::lue_tiedosto(&polku) {
                Ok(tallennus) => {
                    kaynnistin.lataa(tallennus);
                    Some(polku)
                }
                // Tiedostoa ei saatu luettua eikä talteen, joten sen päälle ei kirjoiteta.
                Err(virhe) => {
                    virheloki::kirjaa(&format!("{}: {virhe}, ei tallenneta", polku.display()));
                    None
                }
            }
        });
        let netti = Netti::kaynnista(kaynnistin::netti::OSOITE, kaynnistin::netti::tulosavain());
        if let Some(netti) = &netti {
            netti.laheta(Pyynto::HaeVersio(kaynnistin::netti::JULKAISUT.to_string()));
        }
        (kaynnistin, polku, netti, lukko)
    };

    match pelisilmukka(kaynnistin, &mut kehys, polku.as_deref(), netti) {
        Ok(()) => ExitCode::SUCCESS,
        Err(virhe) => {
            virheloki::kirjaa(&virhe.to_string());
            ExitCode::FAILURE
        }
    }
}
