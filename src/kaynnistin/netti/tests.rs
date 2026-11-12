use super::*;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;

/// Paikallinen palvelin, joka antaa vastaukset järjestyksessä ja palauttaa saamansa pyynnöt.
fn palvelin(vastaukset: Vec<String>) -> (String, thread::JoinHandle<Vec<String>>) {
    let kuuntelija = TcpListener::bind("127.0.0.1:0").unwrap();
    let osoite = format!("http://{}", kuuntelija.local_addr().unwrap());
    let saie = thread::spawn(move || {
        let mut pyynnot = Vec::new();
        for vastaus in vastaukset {
            let (mut yhteys, _) = kuuntelija.accept().unwrap();
            let mut lukija = BufReader::new(yhteys.try_clone().unwrap());
            let mut alku = String::new();
            lukija.read_line(&mut alku).unwrap();
            let mut pituus = 0;
            let mut allekirjoitus = String::new();
            loop {
                let mut rivi = String::new();
                lukija.read_line(&mut rivi).unwrap();
                if rivi.trim().is_empty() {
                    break;
                }
                let pieni = rivi.to_lowercase();
                if let Some(arvo) = pieni.strip_prefix("content-length:") {
                    pituus = arvo.trim().parse().unwrap();
                }
                if let Some(arvo) = pieni.strip_prefix(&format!("{ALLEKIRJOITUS}:")) {
                    allekirjoitus = format!(" allekirjoitus={}", arvo.trim());
                }
            }
            let mut runko = vec![0; pituus];
            lukija.read_exact(&mut runko).unwrap();
            pyynnot.push(format!(
                "{}{allekirjoitus} {}",
                alku.trim(),
                String::from_utf8(runko).unwrap()
            ));
            write!(yhteys, "{vastaus}").unwrap();
        }
        pyynnot
    });
    (osoite, saie)
}

const TESTIAVAIN: Avain = [7; 32];

fn http(status: &str, runko: &str) -> String {
    format!(
        "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{runko}",
        runko.len()
    )
}

fn tulos(nimi: &str, pisteet: u32) -> Tulos {
    Tulos {
        nimi: nimi.to_string(),
        pisteet,
    }
}

#[test]
fn haku_ja_lahetys_kulkevat_ja_hylatty_tulos_kuitataan() {
    let lista = http("200 OK", "120 VÄINÖ\n50 B\n");
    let hylatty = http("400 Bad Request", "kelvoton tulos\n");
    let (osoite, saie) = palvelin(vec![lista.clone(), lista, hylatty]);
    let netti = Netti::kaynnista(&osoite, Some(TESTIAVAIN)).unwrap();
    let odotettu = vec![tulos("VÄINÖ", 120), tulos("B", 50)];

    netti.laheta(Pyynto::Hae(Nettipeli::Tiilirikko));
    assert_eq!(
        netti.vastaukset.recv().unwrap(),
        Vastaus::Lista(Nettipeli::Tiilirikko, odotettu.clone())
    );

    netti.laheta(Pyynto::Laheta(Nettipeli::Luvut, tulos("VÄINÖ", 120)));
    assert_eq!(
        netti.vastaukset.recv().unwrap(),
        Vastaus::Lista(Nettipeli::Luvut, odotettu)
    );
    assert_eq!(
        netti.vastaukset.recv().unwrap(),
        Vastaus::Lahetetty(Nettipeli::Luvut, tulos("VÄINÖ", 120))
    );

    netti.laheta(Pyynto::Laheta(Nettipeli::Tiilirikko, tulos("X", 1)));
    assert_eq!(
        netti.vastaukset.recv().unwrap(),
        Vastaus::Lahetetty(Nettipeli::Tiilirikko, tulos("X", 1))
    );

    let pyynnot = saie.join().unwrap();
    assert!(pyynnot[0].starts_with("GET /top3?peli=tiilirikko "));
    assert!(pyynnot[1].starts_with("POST /tulos?peli=luvut "));
    assert!(pyynnot[1].ends_with(" 120 VÄINÖ"));
    let allekirjoitus = allekirjoita(&TESTIAVAIN, "luvut", "120 VÄINÖ");
    assert!(pyynnot[1].contains(&format!(" allekirjoitus={allekirjoitus} ")));
}

#[test]
fn ilman_avainta_tulosta_ei_laheteta_vaan_se_jaa_jonoon() {
    let kuuntelija = TcpListener::bind("127.0.0.1:0").unwrap();
    let osoite = format!("http://{}", kuuntelija.local_addr().unwrap());
    let agentti = Agent::new_with_defaults();
    let pyynto = Pyynto::Laheta(Nettipeli::Luvut, tulos("A", 1));
    assert!(kasittele(&agentti, &osoite, None, pyynto).is_empty());
    kuuntelija.set_nonblocking(true).unwrap();
    assert!(kuuntelija.accept().is_err(), "yhteyttä ei saa avata");
}

#[test]
fn vain_uudempi_kelvollinen_versio_kelpaa() {
    let polku = "https://github.com/x/y/releases/tag/";
    let uudempi = |tagi: &str| uudempi_versio(&format!("{polku}{tagi}"), "0.3.6");
    assert_eq!(uudempi("v0.3.7").as_deref(), Some("0.3.7"));
    assert_eq!(uudempi("v0.4.0").as_deref(), Some("0.4.0"));
    assert_eq!(uudempi("v1.0.0").as_deref(), Some("1.0.0"));
    // luvut vertaillaan lukuina eikä kirjaimina
    assert_eq!(uudempi("v0.3.10").as_deref(), Some("0.3.10"));
    assert_eq!(uudempi("v0.3.6"), None);
    assert_eq!(uudempi("v0.3.5"), None);
    assert_eq!(uudempi("v0.2.99"), None);
    for kelvoton in ["0.3.7", "v0.3", "v0.3.7.1", "v0.3.x", "v0.4.0-beta", "v"] {
        assert_eq!(uudempi(kelvoton), None, "{kelvoton}");
    }
    // ilman julkaisuja GitHub ohjaa julkaisujen listaan
    assert_eq!(
        uudempi_versio("https://github.com/x/y/releases", "0.3.6"),
        None
    );
    assert_eq!(uudempi_versio(&format!("{polku}v9.0.0"), "kehitys"), None);
}

#[test]
fn versio_luetaan_uudelleenohjauksesta_jota_ei_seurata() {
    let ohjaus = |tagi: &str| {
        format!(
            "HTTP/1.1 302 Found\r\nLocation: http://127.0.0.1:9/releases/tag/{tagi}\r\n\
                 Content-Length: 0\r\nConnection: close\r\n\r\n"
        )
    };
    let (osoite, saie) = palvelin(vec![
        ohjaus("v999.0.0"),
        ohjaus("v0.0.1"),
        http("200 OK", ""),
        http("404 Not Found", ""),
    ]);
    let agentti = Agent::new_with_defaults();
    let hae = || {
        kasittele(
            &agentti,
            &osoite,
            Some(&TESTIAVAIN),
            Pyynto::HaeVersio(osoite.clone()),
        )
    };
    assert_eq!(hae(), [Vastaus::UusiVersio("999.0.0".to_string())]);
    assert!(hae().is_empty());
    assert!(hae().is_empty());
    assert!(hae().is_empty());
    let pyynnot = saie.join().unwrap();
    assert!(pyynnot[0].starts_with("GET /releases/latest "));
}

#[test]
fn palvelinvirhe_ei_kuittaa_tulosta_lahetetyksi() {
    // 429 tulee palvelimen rajoittimesta: tulos kelpaa myöhemmin, joten sitä ei saa pudottaa.
    let (osoite, saie) = palvelin(vec![
        http("500 Oops", ""),
        http("429 Too Many Requests", "liian monta tulosta\n"),
    ]);
    let agentti = Agent::new_with_defaults();
    for _ in 0..2 {
        let pyynto = Pyynto::Laheta(Nettipeli::Luvut, tulos("A", 1));
        assert_eq!(
            kasittele(&agentti, &osoite, Some(&TESTIAVAIN), pyynto),
            [Vastaus::Epaonnistui]
        );
    }
    saie.join().unwrap();
}

#[test]
fn yhteysvirhe_lahetyksessa_kertoo_epaonnistumisesta() {
    let kuuntelija = TcpListener::bind("127.0.0.1:0").unwrap();
    let osoite = format!("http://{}", kuuntelija.local_addr().unwrap());
    drop(kuuntelija);
    let agentti = Agent::new_with_defaults();
    let pyynto = Pyynto::Laheta(Nettipeli::Luvut, tulos("A", 1));
    assert_eq!(
        kasittele(&agentti, &osoite, Some(&TESTIAVAIN), pyynto),
        [Vastaus::Epaonnistui]
    );
}

#[test]
fn yhteysvirhe_ei_tuota_vastausta() {
    let kuuntelija = TcpListener::bind("127.0.0.1:0").unwrap();
    let osoite = format!("http://{}", kuuntelija.local_addr().unwrap());
    drop(kuuntelija);
    let agentti = Agent::new_with_defaults();
    assert!(
        kasittele(
            &agentti,
            &osoite,
            Some(&TESTIAVAIN),
            Pyynto::Hae(Nettipeli::Luvut)
        )
        .is_empty()
    );
}
