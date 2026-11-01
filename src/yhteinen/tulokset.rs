//! Parhaiden tulosten lista ja nimimerkin kirjoitus.

use super::kieli::kaanna;
use super::syote::Syote;

pub const PARHAITA: usize = 3;
pub const NIMEN_PITUUS: usize = 8;
const OLETUSNIMI: &str = "yhteinen.oletusnimi";
// Raja pitää muokatun tiedoston tai vieraan vastauksen luvut sellaisina, ettei mikään voi ylivuotaa.
const SUURIN_PISTEMAARA: u32 = 1_000_000_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tulos {
    pub nimi: String,
    pub pisteet: u32,
}

/// Lukee rivin muodossa "pisteet NIMI". Tallennustiedosto ja nettilista käyttävät samaa muotoa.
pub fn tulos_tekstista(teksti: &str) -> Option<Tulos> {
    let (pisteet, nimi) = teksti.split_once(' ')?;
    let pituus = nimi.chars().count();
    let kelpaa = (1..=NIMEN_PITUUS).contains(&pituus)
        && nimi.chars().all(|m| m.is_uppercase() || m.is_ascii_digit());
    let pisteet = pisteet
        .parse()
        .ok()
        .filter(|&p| p > 0 && p <= SUURIN_PISTEMAARA)?;
    kelpaa.then(|| Tulos {
        nimi: nimi.to_string(),
        pisteet,
    })
}

pub fn lista_tekstista(teksti: &str) -> Vec<Tulos> {
    let mut lista = Vec::new();
    for tulos in teksti.lines().filter_map(tulos_tekstista) {
        lisaa_tulos(&mut lista, tulos);
    }
    lista
}

/// Tasapisteissä vanhempi tulos säilyttää paikkansa, joten sen ohi ei pääse.
pub fn mahtuu_parhaisiin(parhaat: &[Tulos], pisteet: u32) -> bool {
    pisteet > 0 && (parhaat.len() < PARHAITA || parhaat.iter().any(|t| t.pisteet < pisteet))
}

pub fn lisaa_tulos(parhaat: &mut Vec<Tulos>, tulos: Tulos) {
    let paikka = parhaat.partition_point(|t| t.pisteet >= tulos.pisteet);
    parhaat.insert(paikka, tulos);
    parhaat.truncate(PARHAITA);
}

/// Listalle, jossa kullakin nimellä on yksi rivi: uusi tulos korvaa saman nimen vanhan.
pub fn paivita_nimen_tulos(parhaat: &mut Vec<Tulos>, tulos: Tulos) {
    parhaat.retain(|t| t.nimi != tulos.nimi);
    lisaa_tulos(parhaat, tulos);
}

/// Ottaa kirjoitetun nimen. Ikkunan sulkeminen ei saa hävittää tulosta, joten tyhjä
/// nimi korvataan oletuksella.
pub fn ota_nimi(nimi: &mut String) -> String {
    if nimi.is_empty() {
        kaanna(OLETUSNIMI).to_string()
    } else {
        std::mem::take(nimi)
    }
}

pub fn muokkaa_nimea(nimi: &mut String, syote: Syote) {
    let merkki = match syote {
        Syote::Kirjain(kirjain) => kirjain,
        // M on äänikytkimen pikanäppäin, mutta nimessä se on kirjain.
        Syote::VaihdaAanet => 'M',
        Syote::Numero(numero) => (b'0' + numero) as char,
        Syote::Pyyhi => {
            nimi.pop();
            return;
        }
        _ => return,
    };
    if nimi.chars().count() < NIMEN_PITUUS {
        nimi.push(merkki);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parhaat_pysyvat_jarjestyksessa_ja_tasapisteilla_vanhempi_sailyy() {
        let tulos = |nimi: &str, pisteet| Tulos {
            nimi: nimi.to_string(),
            pisteet,
        };
        let mut parhaat = Vec::new();
        assert!(!mahtuu_parhaisiin(&parhaat, 0));
        for (nimi, pisteet) in [("B", 200), ("C", 100), ("A", 300)] {
            assert!(mahtuu_parhaisiin(&parhaat, pisteet));
            lisaa_tulos(&mut parhaat, tulos(nimi, pisteet));
        }
        assert!(!mahtuu_parhaisiin(&parhaat, 100));
        assert!(mahtuu_parhaisiin(&parhaat, 101));
        lisaa_tulos(&mut parhaat, tulos("D", 200));
        assert_eq!(parhaat, [tulos("A", 300), tulos("B", 200), tulos("D", 200)]);
    }

    #[test]
    fn lista_luetaan_tekstista_ja_kelvottomat_rivit_ohitetaan() {
        let lista = lista_tekstista("5 PIENI\n9 VÄINÖ\n0 NOLLA\nkissa\n7 pieni\n2000000000 ISO\n");
        let pisteet: Vec<u32> = lista.iter().map(|t| t.pisteet).collect();
        assert_eq!(pisteet, [9, 5]);
        assert_eq!(lista[0].nimi, "VÄINÖ");
    }

    #[test]
    fn saman_nimen_tulos_korvataan() {
        let tulos = |nimi: &str, pisteet| Tulos {
            nimi: nimi.to_string(),
            pisteet,
        };
        let mut lista = vec![tulos("A", 300), tulos("B", 200), tulos("C", 100)];
        paivita_nimen_tulos(&mut lista, tulos("C", 400));
        assert_eq!(lista, [tulos("C", 400), tulos("A", 300), tulos("B", 200)]);
    }

    #[test]
    fn tyhja_nimi_saa_oletuksen() {
        let mut nimi = String::new();
        assert_eq!(ota_nimi(&mut nimi), kaanna(OLETUSNIMI));
        muokkaa_nimea(&mut nimi, Syote::Kirjain('A'));
        assert_eq!(ota_nimi(&mut nimi), "A");
        assert!(nimi.is_empty());
    }
}
