use super::*;
use klassikot8000::holdem::{Kortti, Maa, Tila, paras_kasi};

/// Peli, jossa on pelaajan vuoro. Pelaajalla on oikea pino.
fn omalla_vuorolla(siemen: u64) -> Holdem {
    let mut peli = Holdem::new(siemen);
    peli.kasittele(Syote::Ammu);
    peli.edista(1_000_000);
    // Kierrokset, joilla pelaaja ei pääse toimimaan, jatkuvat uudella jaolla.
    while peli.napisto() != Napisto::Vuoro {
        assert_eq!(peli.vaihe, Vaihe::Pelissa, "siemen {siemen}");
        peli.kasittele(Syote::Ammu);
        peli.edista(1_000_000);
    }
    peli
}

#[test]
fn alkuruutu_aloittaa_pelin_ja_kaikilla_on_tuhat() {
    let mut peli = Holdem::new(1);
    assert_eq!(peli.vaihe, Vaihe::Alku);
    assert_eq!(peli.kuvan_askel(), None);
    assert!(peli.poyta.pelaajat.iter().all(|p| p.rahat == ALKURAHA));
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
    assert_eq!(peli.poyta.jako, 1);
    assert_eq!(
        peli.poyta
            .pelaajat
            .iter()
            .filter(|p| p.kortit.len() == 2)
            .count(),
        5
    );
}

#[test]
fn koneet_pelaavat_ajan_kuluessa_ja_valilyonti_hoputtaa() {
    let mut peli = Holdem::new(5);
    peli.kasittele(Syote::Ammu);
    // Etsitään jako, jossa kone on ensin vuorossa.
    while peli.napisto() != Napisto::Odotus {
        peli.kasittele(Syote::Kirjain('K'));
        peli.edista(1_000_000);
        peli.kasittele(Syote::Ammu);
    }
    let ennen = peli.kuvan_askel();
    peli.edista(ASKEL_MS - 1);
    assert_eq!(peli.kuvan_askel(), ennen);
    peli.edista(1);
    assert_ne!(peli.kuvan_askel(), ennen);

    if peli.napisto() == Napisto::Odotus {
        let ennen = peli.kuvan_askel();
        peli.kasittele(Syote::Ammu);
        assert_ne!(peli.kuvan_askel(), ennen);
    }
}

#[test]
fn levossa_kuva_ei_muutu_omalla_vuorolla() {
    let mut peli = omalla_vuorolla(1);
    let lepo = peli.kuvan_askel();
    peli.edista(600_000);
    assert_eq!(peli.kuvan_askel(), lepo);
    assert_eq!(peli.napisto(), Napisto::Vuoro);
}

#[test]
fn valilyonti_ei_koskaan_maksa_omalla_vuorolla() {
    let mut peli = omalla_vuorolla(1);
    let rahat = peli.poyta.pelaajat[PELAAJA].rahat;
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.poyta.pelaajat[PELAAJA].rahat, rahat);
    assert_eq!(peli.napisto(), Napisto::Vuoro);
}

#[test]
fn korotus_pysyy_rajoissa_ja_napit_asettavat_sen() {
    let mut peli = omalla_vuorolla(1);
    let (pienin, suurin) = (
        peli.poyta.pienin_taso(PELAAJA),
        peli.poyta.suurin_taso(PELAAJA),
    );
    let iso = peli.poyta.blindit().1;
    assert_eq!(peli.korotus, pienin);
    peli.kasittele(Syote::Nuoli(0, -1));
    assert_eq!(peli.korotus, pienin);
    peli.kasittele(Syote::Nuoli(0, 1));
    assert_eq!(peli.korotus, pienin + iso);
    peli.kasittele(Syote::Nuoli(-1, 0));
    assert_eq!(peli.korotus, pienin + 6 * iso);
    peli.kasittele(Syote::Nuoli(1, 0));
    assert_eq!(peli.korotus, pienin + iso);
    peli.kasittele(Syote::Numero(4));
    assert_eq!(peli.korotus, suurin);
    peli.kasittele(Syote::Nuoli(0, 1));
    assert_eq!(peli.korotus, suurin);
    peli.kasittele(Syote::Numero(1));
    assert_eq!(peli.korotus, pienin);
    peli.kasittele(Syote::Numero(3));
    let potti = peli.poyta.korkein + peli.poyta.potti() + peli.poyta.maksu(PELAAJA);
    assert_eq!(peli.korotus, potti.clamp(pienin, suurin));
    let (x, y) = piirto::napin_keskipiste(Napisto::Vuoro, Nappi::Kaikki);
    peli.kasittele(Syote::Klikkaus(x, y));
    assert_eq!(peli.korotus, suurin);
    // asettaminen ei vielä korota
    assert_eq!(peli.napisto(), Napisto::Vuoro);
}

#[test]
fn kippaus_maksu_ja_korotus_toimivat_nappaimilla() {
    let mut peli = omalla_vuorolla(1);
    peli.kasittele(Syote::Kirjain('K'));
    assert_eq!(peli.poyta.pelaajat[PELAAJA].tila, Tila::Kippasi);

    let mut peli = omalla_vuorolla(1);
    let rahat = peli.poyta.pelaajat[PELAAJA].rahat;
    let maksu = peli.poyta.maksu(PELAAJA);
    peli.kasittele(Syote::Kirjain('V'));
    assert_eq!(peli.poyta.pelaajat[PELAAJA].rahat, rahat - maksu);

    let mut peli = omalla_vuorolla(1);
    let taso = peli.korotus;
    peli.kasittele(Syote::Kirjain('R'));
    assert_eq!(peli.poyta.korkein, taso);
    assert_eq!(peli.poyta.viime_hyokkaaja, Some(PELAAJA));
}

#[test]
fn jakojen_valissa_vali_jakaa_ja_tulos_nakyy_viestissa() {
    let mut peli = omalla_vuorolla(2);
    peli.kasittele(Syote::Kirjain('K'));
    peli.edista(1_000_000);
    assert_eq!(peli.napisto(), Napisto::JakoOhi);
    assert!(peli.viesti.contains('+'), "{}", peli.viesti);
    let jako = peli.poyta.jako;
    peli.edista(1_000_000);
    assert_eq!(peli.poyta.jako, jako);
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.poyta.jako, jako + 1);
}

/// Asettaa voitot (voitto, josta pääpotista) ja kaikille saman näytetyn käden.
fn tulos(voitot: [(u32, u32); PELAAJIA]) -> String {
    let mut peli = Holdem::new(1);
    let kortit = [2, 3, 4, 5, 6].map(|arvo| Kortti {
        maa: Maa::Pata,
        arvo,
    });
    for (pelaaja, (voitto, paapotista)) in peli.poyta.pelaajat.iter_mut().zip(voitot) {
        pelaaja.voitto = voitto;
        pelaaja.paapotista = paapotista;
        pelaaja.kasi = paras_kasi(&kortit);
    }
    peli.tulosviesti()
}

#[test]
fn tulos_kertoo_yhden_voittajan_tasakaden_ja_sivupotin_eri_tavoin() {
    let kasi = kaanna("holdem.kasi_varisuora");
    let nimi = pelaajan_nimi;
    assert_eq!(
        tulos([(50, 50), (0, 0), (0, 0), (0, 0), (0, 0)]),
        muotoile("holdem.voitot_kasi", &[&format!("{} +50", nimi(0)), &kasi])
    );
    // Sama voittaja kummassakin potissa on yhä yksi voittaja.
    assert_eq!(
        tulos([(70, 50), (0, 0), (0, 0), (0, 0), (0, 0)]),
        muotoile("holdem.voitot_kasi", &[&format!("{} +70", nimi(0)), &kasi])
    );
    assert_eq!(
        tulos([(25, 25), (0, 0), (25, 25), (0, 0), (0, 0)]),
        muotoile(
            "holdem.tasan",
            &[&format!("{}, {}", nimi(0), nimi(2)), &kasi]
        )
    );
    assert_eq!(
        tulos([(330, 330), (0, 0), (400, 0), (0, 0), (0, 0)]),
        muotoile(
            "holdem.sivupotti_tulos",
            &[&format!("{} +330", nimi(0)), &format!("{} +400", nimi(2))]
        )
    );
}

#[test]
fn rahojen_loppuessa_peli_paattyy_ja_alkaa_alusta() {
    let mut peli = omalla_vuorolla(3);
    peli.poyta.pelaajat[PELAAJA].rahat = 0;
    peli.poyta.pelaajat[1].rahat = 0;
    peli.jako_paattyi();
    assert_eq!(peli.vaihe, Vaihe::Havio);
    assert!(peli.aanet.jono.contains(&(Aani::Tappio, LOPPU_VIIVE_MS)));
    // Kolme on yhä rahoissa, joten pelaajan sija on neljäs.
    assert_eq!(peli.viesti, muotoile("holdem.havio", &[&4]));
    peli.kasittele(Syote::Ammu);
    peli.kasittele(Syote::Kirjain('V'));
    assert_eq!(peli.vaihe, Vaihe::Havio);
    peli.kasittele(Syote::Kirjain('K'));
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
    assert_eq!(peli.poyta.jako, 1);
    assert!(peli.poyta.pelaajat[PELAAJA].rahat > 0);
}

#[test]
fn viimeisena_rahoissa_voittaa_pelin() {
    let mut peli = omalla_vuorolla(4);
    for i in 1..PELAAJIA {
        peli.poyta.pelaajat[i].rahat = 0;
    }
    peli.poyta.pelaajat[PELAAJA].rahat = ALKURAHA * PELAAJIA as u32;
    peli.jako_paattyi();
    assert_eq!(peli.vaihe, Vaihe::Nimi);
    assert!(peli.aanet.jono.contains(&(Aani::Voitto, LOPPU_VIIVE_MS)));
    assert_eq!(peli.viesti, muotoile("holdem.voitto", &[&5000]));
    peli.kasittele(Syote::Kirjain('K'));
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.vaihe, Vaihe::Voitto);
    assert!(!peli.lopeta);
    assert_eq!(peli.taulu.parhaat[0].pisteet, 4000);
    // Kertaalleen kirjattu voitto ei kirjaudu toista kertaa poistuttaessa.
    peli.taulu.lahetys = None;
    peli.kasittele(Syote::Peru);
    assert!(peli.lopeta);
    assert_eq!(peli.taulu.lahetys, None);

    peli.lopeta = false;
    peli.kasittele(Syote::Kirjain('K'));
    assert_eq!(peli.vaihe, Vaihe::Pelissa);
}

#[test]
fn voitolla_poistuva_kirjoittaa_nimen_ja_palaa_valikkoon() {
    let mut peli = omalla_vuorolla(1);
    peli.poyta.pelaajat[PELAAJA].rahat = ALKURAHA + 300;
    peli.kasittele(Syote::Peru);
    assert_eq!(peli.vaihe, Vaihe::Nimi);
    // Koneet eivät pelaa nimen aikana, eikä K kippaa.
    let siirrot = peli.siirtoja;
    peli.edista(100_000);
    assert_eq!(peli.siirtoja, siirrot);
    peli.kasittele(Syote::Kirjain('K'));
    assert_eq!(peli.poyta.pelaajat[PELAAJA].tila, Tila::Mukana);
    peli.kasittele(Syote::Ammu);
    assert!(peli.lopeta);
    assert_eq!(peli.taulu.parhaat[0].nimi, "K");
    assert_eq!(peli.taulu.parhaat[0].pisteet, 300);
}

#[test]
fn napit_ja_esc_palaavat_valikkoon() {
    let mut peli = omalla_vuorolla(1);
    let (x, y) = piirto::napin_keskipiste(Napisto::Vuoro, Nappi::Kippaa);
    peli.kasittele(Syote::Klikkaus(x, y));
    assert_eq!(peli.poyta.pelaajat[PELAAJA].tila, Tila::Kippasi);
    peli.kasittele(Syote::Peru);
    assert!(peli.lopeta);

    let mut peli = omalla_vuorolla(1);
    peli.kasittele(Syote::Kirjain('E'));
    assert!(peli.lopeta);
}

#[test]
fn kokonainen_peli_paattyy_hoputtamalla_ja_kippaamalla() {
    let mut peli = Holdem::new(7);
    peli.kasittele(Syote::Ammu);
    for _ in 0..3000 {
        if peli.vaihe != Vaihe::Pelissa {
            return;
        }
        peli.kasittele(Syote::Kirjain('V'));
        peli.edista(10_000);
        peli.kasittele(Syote::Ammu);
    }
    panic!("peli ei päättynyt");
}
