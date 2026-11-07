//! Julkaisuversiolla ei ole konsolia, joten virheet ja kaatumiset kirjataan tiedostoon
//! tallennuksen viereen. Ilman tiedostoa virhe tulostuu vain konsoliin, kuten kehitysversiossa.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::kaynnistin::netti::VERSIO;

/// Tätä suurempi loki aloitetaan alusta, jottei toistuva virhe täytä levyä.
const SUURIN_KOKO: u64 = 64 * 1024;

static POLKU: OnceLock<PathBuf> = OnceLock::new();

/// Ottaa lokitiedoston käyttöön ja kirjaa myös kaatumiset siihen. `panic = "abort"` lopettaa
/// ohjelman heti koukun jälkeen, joten koukku on ainoa paikka, jossa kaatumisen syy näkyy.
#[cfg(not(debug_assertions))]
pub fn ota_kayttoon(polku: PathBuf) {
    if POLKU.set(polku).is_ok() {
        let edellinen = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |tieto| {
            kirjaa(&format!("kaatui: {tieto}"));
            edellinen(tieto);
        }));
    }
}

/// Tulostaa virheen konsoliin ja lisää sen lokiin, jos loki on käytössä.
pub fn kirjaa(viesti: &str) {
    eprintln!("{viesti}");
    if let Some(polku) = POLKU.get() {
        let aika = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |aika| aika.as_secs());
        // Lokin kirjoitusvirheelle ei voi tehdä mitään muuta.
        let _ = lisaa_riville(polku, &format!("{aika} v{VERSIO} {viesti}"));
    }
}

fn lisaa_riville(polku: &Path, rivi: &str) -> io::Result<()> {
    if let Some(kansio) = polku.parent() {
        fs::create_dir_all(kansio)?;
    }
    let liian_iso = fs::metadata(polku).is_ok_and(|tieto| tieto.len() > SUURIN_KOKO);
    let mut tiedosto = OpenOptions::new()
        .create(true)
        .append(!liian_iso)
        .write(true)
        .truncate(liian_iso)
        .open(polku)?;
    writeln!(tiedosto, "{rivi}")
}

#[cfg(test)]
mod tests;
