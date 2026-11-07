use super::*;
use crate::yhteinen::piirto::{LEVEYS, SUURI};
use crate::yhteinen::tulokset::{NIMEN_PITUUS, tulos_tekstista};
use std::path::Path;

/// Ajaa tarkistuksen kummallakin kielellä ja palauttaa suomen.
pub fn kaikilla_kielilla(mut tarkistus: impl FnMut()) {
    for kieli in Kieli::KAIKKI {
        aseta(kieli);
        tarkistus();
    }
    aseta(Kieli::Suomi);
}

fn upotetut() -> [Taulu; 2] {
    rakenna(None)
}

fn lahdekoodi(kansio: &Path, koodi: &mut String) {
    for tiedosto in fs::read_dir(kansio).unwrap() {
        let polku = tiedosto.unwrap().path();
        if polku.is_dir() {
            lahdekoodi(&polku, koodi);
        } else if polku.extension().is_some_and(|e| e == "rs") {
            *koodi += &fs::read_to_string(&polku).unwrap();
        }
    }
}

#[test]
fn kielilla_on_samat_avaimet_ja_arvokohdat() {
    let [suomi, englanti] = upotetut();
    assert!(!suomi.is_empty());
    for (avain, teksti) in &suomi {
        let toinen = englanti.get(avain).unwrap_or_else(|| panic!("en: {avain}"));
        assert_eq!(
            teksti.matches("{}").count(),
            toinen.matches("{}").count(),
            "{avain}"
        );
    }
    for avain in englanti.keys() {
        assert!(suomi.contains_key(avain), "fi: {avain}");
    }
}

#[test]
fn koodin_avaimet_loytyvat_ja_jokaista_tekstia_kaytetaan() {
    let [suomi, _] = upotetut();
    let mut koodi = String::new();
    lahdekoodi(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut koodi,
    );
    let etuliite = |avain: &str| avain.split('.').next().unwrap_or("").to_string();
    let etuliitteet: Vec<String> = suomi.keys().map(|a| etuliite(a)).collect();
    for pala in koodi.split('"') {
        let avaimen_muoto = pala.contains('.')
            && pala
                .chars()
                .all(|m| m == '.' || m == '_' || m.is_ascii_lowercase() || m.is_ascii_digit());
        if avaimen_muoto && etuliitteet.contains(&etuliite(pala)) {
            assert!(suomi.contains_key(pala), "kielet.txt: {pala} puuttuu");
        }
    }
    for avain in suomi.keys() {
        assert!(
            koodi.contains(&format!("\"{avain}\"")),
            "kielet.txt: {avain} on turha"
        );
    }
}

#[test]
fn tekstit_piirtyvat_fonteilla() {
    for taulu in upotetut() {
        for (avain, teksti) in &taulu {
            let teksti = teksti.replace("{}", "");
            for merkki in teksti.chars() {
                let tunnettu = matches!(merkki, ' '..='z' | 'Ä' | 'Ö' | 'ä' | 'ö');
                assert!(tunnettu, "{avain}: {merkki}");
            }
        }
    }
}

#[test]
fn bannerit_mahtuvat_kuvaan() {
    for taulu in upotetut() {
        for (avain, teksti) in taulu.iter().filter(|(a, _)| a.ends_with(".banneri")) {
            let leveys = teksti.chars().count() as i32 * SUURI.2;
            assert!(leveys <= LEVEYS as i32 - 48, "{avain}");
        }
    }
}

#[test]
fn oletusnimi_kelpaa_tulokseksi() {
    kaikilla_kielilla(|| {
        let nimi = kaanna("yhteinen.oletusnimi");
        assert!(nimi.chars().count() <= NIMEN_PITUUS);
        assert!(tulos_tekstista(&format!("1 {nimi}")).is_some());
    });
}

#[test]
fn tiedosto_korvaa_upotetun_ja_puuttuva_teksti_sailyy() {
    let teksti = "\u{feff}# kommentti\r\n[en]\r\npeli.luvut = Numbers = 2\r\nroskaa\r\n[xx]\r\npeli.pasianssi = X\r\n";
    let [suomi, englanti] = rakenna(Some(teksti));
    let [_, alkuperainen] = upotetut();
    assert_eq!(englanti["peli.luvut"], "Numbers = 2");
    assert_eq!(englanti["peli.pasianssi"], alkuperainen["peli.pasianssi"]);
    assert_eq!(suomi["peli.luvut"], "2048");
}

#[test]
fn kielen_vaihto_ja_arvojen_sijoitus() {
    assert_eq!(nykyinen(), Kieli::Suomi);
    assert_eq!(kaanna("peli.pasianssi"), "Pasianssi");
    assert_eq!(muotoile("laivanupotus.upotettu", &[&1, &5]), "Upotettu 1/5");
    aseta(nykyinen().toinen());
    assert_eq!(kaanna("peli.pasianssi"), "Solitaire");
    assert_eq!(muotoile("laivanupotus.upotettu", &[&1, &5]), "Sunk 1/5");
    aseta(Kieli::Suomi);
    assert_eq!(
        Kieli::koodista(Kieli::Englanti.koodi()),
        Some(Kieli::Englanti)
    );
    assert_eq!(Kieli::koodista("sv"), None);
}
