//! Värien sävyportaat ja 16-bittisten koneiden värimatematiikka.
//!
//! Porras ei ole pelkkä tummempi ja vaaleampi versio samasta väristä: varjot kiertävät
//! sävyä kohti sinivioletteja ja kylläistyvät, valot kiertävät kohti keltaista ja
//! haalistuvat. Näin keltainen tummuu oranssiksi eikä ruskeaksi, ja valkoinen laventeliksi.

/// Varjojen sävy: sinivioletti, kuten SNES-pelien väriportaiden pohjalla.
const VARJON_SAVY: f32 = 255.0;
/// Valojen sävy: lämmin keltainen.
const VALON_SAVY: f32 = 50.0;
/// Väri, jota kohti läpikuultava varjo tummentaa.
const VARJO: u32 = 0x14082C;

/// Viisiportainen sävyporras: syvä varjo, varjo, perusväri, valo ja kiilto.
pub fn ramppi(perus: u32) -> [u32; 5] {
    [
        porras(perus, -2),
        porras(perus, -1),
        perus,
        porras(perus, 1),
        porras(perus, 2),
    ]
}

fn porras(perus: u32, askel: i32) -> u32 {
    let (savy, kyllaisyys, kirkkaus) = hsv(perus);
    let maara = askel.unsigned_abs() as f32;
    if askel < 0 {
        // Harmaalla ei ole sävyä, josta kiertää: varjo saa suoraan varjon sävyn.
        let savy = if kyllaisyys < 0.05 {
            VARJON_SAVY
        } else {
            kierra_kohti(savy, VARJON_SAVY, 14.0 * maara)
        };
        rgb(
            savy,
            (kyllaisyys + 0.1 * maara).min(1.0),
            kirkkaus * (1.0 - 0.17 * maara),
        )
    } else {
        rgb(
            kierra_kohti(savy, VALON_SAVY, 8.0 * maara),
            kyllaisyys * (1.0 - 0.3 * maara),
            kirkkaus + (1.0 - kirkkaus) * 0.45 * maara,
        )
    }
}

/// Kiertää sävyä `asteet` lyhintä tietä kohti sävyä `kohde`, mutta ei sen yli.
fn kierra_kohti(savy: f32, kohde: f32, asteet: f32) -> f32 {
    let ero = (kohde - savy + 540.0).rem_euclid(360.0) - 180.0;
    (savy + ero.clamp(-asteet, asteet)).rem_euclid(360.0)
}

fn hsv(vari: u32) -> (f32, f32, f32) {
    let [r, g, b] = [16, 8, 0].map(|siirto| (vari >> siirto & 0xFF) as f32 / 255.0);
    let suurin = r.max(g).max(b);
    let ero = suurin - r.min(g).min(b);
    let savy = if ero == 0.0 {
        0.0
    } else if suurin == r {
        60.0 * ((g - b) / ero).rem_euclid(6.0)
    } else if suurin == g {
        60.0 * ((b - r) / ero + 2.0)
    } else {
        60.0 * ((r - g) / ero + 4.0)
    };
    let kyllaisyys = if suurin == 0.0 { 0.0 } else { ero / suurin };
    (savy, kyllaisyys, suurin)
}

fn rgb(savy: f32, kyllaisyys: f32, kirkkaus: f32) -> u32 {
    let c = kirkkaus * kyllaisyys;
    let x = c * (1.0 - ((savy / 60.0).rem_euclid(2.0) - 1.0).abs());
    let (r, g, b) = match (savy / 60.0) as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = kirkkaus - c;
    let kanava = |arvo: f32| ((arvo + m) * 255.0).round().clamp(0.0, 255.0) as u32;
    kanava(r) << 16 | kanava(g) << 8 | kanava(b)
}

/// Läpikuultava varjo kuten SNES:n värimatematiikassa: pohjan väri tummuu kohti varjon
/// sävyä. `osuus` on 0..=256, jossa 256 on täysi varjo. Kokonaisluvuin, koska tätä
/// kutsutaan jokaiselle varjon pikselille.
pub fn varjoon(pohja: u32, osuus: u32) -> u32 {
    let kanava = |siirto: u32| {
        let (a, b) = (pohja >> siirto & 0xFF, VARJO >> siirto & 0xFF);
        ((a * (256 - osuus) + b * osuus) >> 8) << siirto
    };
    kanava(16) | kanava(8) | kanava(0)
}

#[cfg(test)]
mod tests;
