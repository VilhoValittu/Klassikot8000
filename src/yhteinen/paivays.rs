//! Tulosten päivämäärät. Aika kulkee Unix-sekunteina, ja näkyvä päivä lasketaan Suomen ajassa,
//! koska Windowsin aikavyöhykettä ei saa luettua ilman uutta riippuvuutta tai `unsafe`a.

use std::time::{SystemTime, UNIX_EPOCH};

const PAIVA: i64 = 86_400;
const TUNTI: i64 = 3_600;
const VAKIOAIKA: i64 = 2 * TUNTI;
/// 31.12.9999 klo 23.59.59 UTC. Raja pitää muokatun tiedoston ajan laskuissa ilman ylivuotoa.
pub const SUURIN_AIKA: u64 = 253_402_300_799;

/// Nykyhetki Unix-sekunteina. Ennen vuotta 1970 oleva kello antaa nollan.
pub fn nyt() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |kesto| kesto.as_secs())
}

/// Päivämäärä muodossa pp.kk.vvvv Suomen ajassa.
pub fn pvm(aika: u64) -> String {
    let (vuosi, kuukausi, paiva) = suomen_paiva(aika);
    format!("{paiva:02}.{kuukausi:02}.{vuosi}")
}

/// (vuosi, kuukausi, päivä) Suomen ajassa: UTC+2, kesäaikana UTC+3.
fn suomen_paiva(aika: u64) -> (i64, u32, u32) {
    let utc = aika.min(SUURIN_AIKA) as i64;
    let siirto = if kesaaikaa(utc) { TUNTI } else { 0 };
    paiva_numerosta((utc + VAKIOAIKA + siirto).div_euclid(PAIVA))
}

/// EU:n kesäaika alkaa maaliskuun ja päättyy lokakuun viimeisenä sunnuntaina klo 1.00 UTC.
fn kesaaikaa(utc: i64) -> bool {
    let (vuosi, _, _) = paiva_numerosta(utc.div_euclid(PAIVA));
    let alku = viimeinen_sunnuntai(vuosi, 3) * PAIVA + TUNTI;
    let loppu = viimeinen_sunnuntai(vuosi, 10) * PAIVA + TUNTI;
    (alku..loppu).contains(&utc)
}

/// Kuukauden viimeisen sunnuntain päivänumero (päiviä vuoden 1970 alusta).
fn viimeinen_sunnuntai(vuosi: i64, kuukausi: u32) -> i64 {
    let viimeinen = numero_paivasta(vuosi, kuukausi, 31);
    // 1.1.1970 oli torstai, joten (päivä + 4) mod 7 on viikonpäivä sunnuntaista alkaen.
    viimeinen - (viimeinen + 4).rem_euclid(7)
}

/// Päivänumerosta (vuosi, kuukausi, päivä) gregoriaanisessa kalenterissa (Howard Hinnantin kaava).
fn paiva_numerosta(numero: i64) -> (i64, u32, u32) {
    let z = numero + 719_468;
    let aikakausi = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let paiva = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let kuukausi = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let vuosi = yoe + aikakausi * 400 + i64::from(kuukausi <= 2);
    (vuosi, kuukausi, paiva)
}

/// Päivänumero (vuosi, kuukausi, päivä) -kolmikosta, edellisen käänteinen.
fn numero_paivasta(vuosi: i64, kuukausi: u32, paiva: u32) -> i64 {
    let vuosi = vuosi - i64::from(kuukausi <= 2);
    let aikakausi = vuosi.div_euclid(400);
    let yoe = vuosi.rem_euclid(400);
    let mp = i64::from((kuukausi + 9) % 12);
    let doy = (153 * mp + 2) / 5 + i64::from(paiva) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    aikakausi * 146_097 + doe - 719_468
}

#[cfg(test)]
mod tests;
