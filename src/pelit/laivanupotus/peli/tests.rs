use super::*;
use crate::yhteinen::piirto::aanikytkimen_keskipiste;
use crate::yhteinen::tulokset::Tulos;

fn sijoituksessa(siemen: u64) -> Peli {
    let mut peli = Peli::new(siemen);
    assert!(peli.kasittele(Syote::Ammu));
    assert_eq!(peli.vaihe, Vaihe::Sijoitus);
    peli
}

fn pelissa(siemen: u64) -> Peli {
    let mut peli = sijoituksessa(siemen);
    assert!(peli.kasittele(Syote::Ammu));
    assert_eq!(peli.vaihe, Vaihe::Ammunta);
    peli
}

/// Ruutu, jossa ei ole laivaa eikä laivan vieressä.
fn vapaa_ruutu(peli: &Peli) -> Ruutu {
    let varatut = peli
        .oma
        .laivat()
        .iter()
        .fold(Ruudut::default(), |kaikki, l| {
            kaikki.yhdiste(l.ruudut.ymparys())
        });
    (0..KOKO * KOKO)
        .filter_map(|i| Ruutu::new(i / KOKO, i % KOKO))
        .find(|&r| !varatut.sisaltaa(r))
        .unwrap()
}

#[test]
fn laiva_nostetaan_ja_lasketaan_hiirella_vain_vapaaseen_paikkaan() {
    let mut peli = sijoituksessa(1);
    let laiva = peli.oma.laivat()[4];
    let alku = laiva.alku().unwrap();
    let (x, y) = piirto::oman_ruudun_keskipiste(alku);
    assert!(peli.kasittele(Syote::Klikkaus(x, y)));
    assert_eq!(peli.nostettu.map(|n| n.laiva), Some(4));

    // toisen laivan päälle ei voi laskea
    let toinen = peli.oma.laivat()[0].alku().unwrap();
    let (x, y) = piirto::oman_ruudun_keskipiste(toinen);
    peli.kasittele(Syote::Hiiri(x, y));
    assert!(!peli.haamu().unwrap().2);
    peli.aanet.jono.clear();
    peli.kasittele(Syote::Klikkaus(x, y));
    assert_eq!(peli.aanet.jono, [(Aani::Virhe, 0)]);
    assert!(peli.nostettu.is_some());
    assert_eq!(peli.oma.laivat()[4].ruudut, laiva.ruudut);

    // Vapaa paikka löytyy aina kahden ruudun laivalle, kun se on nostettu pois.
    let ruudut = (0..KOKO * KOKO)
        .filter_map(|i| Ruutu::new(i / KOKO, i % KOKO))
        .find(|&r| {
            peli.kursori = r;
            peli.haamu()
                .is_some_and(|(_, ruudut, kelpaa)| kelpaa && ruudut != laiva.ruudut)
        })
        .unwrap();
    let (x, y) = piirto::oman_ruudun_keskipiste(ruudut);
    peli.kasittele(Syote::Klikkaus(x, y));
    assert!(peli.nostettu.is_none());
    assert_ne!(peli.oma.laivat()[4].ruudut, laiva.ruudut);
    assert!(peli.oma.mahtuu(4, peli.oma.laivat()[4].ruudut));
}

#[test]
fn nappaimisto_nostaa_kaantaa_ja_laskee_laivan() {
    let mut peli = sijoituksessa(2);
    assert!(peli.kasittele(Syote::Numero(5)));
    let ennen = peli.oma.laivat()[4];
    assert_eq!(peli.kursori, ennen.alku().unwrap());
    peli.kasittele(Syote::Kirjain('R'));
    assert_eq!(peli.nostettu.map(|n| n.vaaka), Some(!ennen.vaaka()));
    peli.kasittele(Syote::Kirjain('R'));
    // Nuolet vievät haamun vapaaseen kohtaan, ja välilyönti laskee sen.
    let kohde = vapaa_ruutu(&peli);
    while peli.kursori != kohde {
        let d = |a: u8, b: u8| (b as i8 - a as i8).signum();
        peli.kasittele(Syote::Nuoli(
            d(peli.kursori.rivi, kohde.rivi),
            d(peli.kursori.sarake, kohde.sarake),
        ));
    }
    if peli.haamu().unwrap().2 {
        peli.kasittele(Syote::Ammu);
        assert!(peli.nostettu.is_none());
        assert_eq!(peli.vaihe, Vaihe::Sijoitus);
    }
    // Välilyönti ilman nostettua laivaa aloittaa taistelun.
    peli.nostettu = None;
    peli.aanet.jono.clear();
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.vaihe, Vaihe::Ammunta);
    assert_eq!(peli.aanet.jono, [(Aani::Alku, 0)]);
}

#[test]
fn laudalla_oleva_laiva_kaantyy_paikallaan_jos_mahtuu() {
    let mut kaantyi = 0;
    for siemen in 0..50 {
        let mut peli = sijoituksessa(siemen);
        let ennen = peli.oma.laivat()[4];
        peli.kursori = ennen.alku().unwrap();
        peli.kasittele(Syote::Kirjain('R'));
        assert!(peli.nostettu.is_none());
        let jalkeen = peli.oma.laivat()[4];
        if jalkeen.ruudut != ennen.ruudut {
            kaantyi += 1;
            assert_eq!(jalkeen.vaaka(), !ennen.vaaka());
            assert!(jalkeen.ruudut.sisaltaa(ennen.alku().unwrap()));
        } else {
            assert_eq!(peli.aanet.jono.last(), Some(&(Aani::Virhe, 0)));
        }
    }
    assert!(kaantyi > 0);
}

#[test]
fn arvonta_ja_taistelunappi_toimivat_ja_nosto_peruuntuu() {
    let mut peli = sijoituksessa(1);
    let ennen: Vec<_> = peli.oma.laivat().iter().map(|l| l.ruudut).collect();
    let (x, y) = piirto::napin_keskipiste(piirto::Nappi::Arvo);
    assert!(peli.kasittele(Syote::Klikkaus(x, y)));
    let jalkeen: Vec<_> = peli.oma.laivat().iter().map(|l| l.ruudut).collect();
    assert_ne!(ennen, jalkeen);

    peli.kasittele(Syote::Numero(1));
    let (x, y) = piirto::napin_keskipiste(piirto::Nappi::Taisteluun);
    peli.kasittele(Syote::Klikkaus(x, y));
    assert_eq!(peli.vaihe, Vaihe::Ammunta);
    assert!(peli.nostettu.is_none());
    let lopuksi: Vec<_> = peli.oma.laivat().iter().map(|l| l.ruudut).collect();
    assert_eq!(lopuksi, jalkeen);
}

#[test]
fn tilasto_kirjaa_voitot_tappiot_ja_vahimmat_laukaukset() {
    let mut voitot = 0;
    let mut tappiot = 0;
    for siemen in 0..60 {
        let mut peli = pelissa(siemen);
        peli.vaikeus = Vaikeus::Helppo;
        for i in 0..KOKO * KOKO {
            peli.ammu(Ruutu::new(i / KOKO, i % KOKO).unwrap());
            if peli.vaihe != Vaihe::Ammunta {
                break;
            }
        }
        assert!(peli.uusi_tulos);
        let tilasto = peli.tilastot[0];
        assert_eq!(peli.tilastot[2], Tilasto::default());
        // Tyhjälle listalle jokainen voitto yltää, joten se odottaa nimeä.
        if peli.vaihe == Vaihe::Nimi {
            voitot += 1;
            assert_eq!(tilasto.voitot, 1);
            assert_eq!(tilasto.vahiten_laukauksia, peli.laukauksia());
        } else {
            tappiot += 1;
            assert_eq!(tilasto.tappiot, 1);
            assert_eq!(tilasto.vahiten_laukauksia, 0);
        }
    }
    assert!(voitot > 0 && tappiot > 0);

    let mut tilasto = Tilasto::default();
    tilasto.kirjaa(true, 50);
    tilasto.kirjaa(true, 40);
    tilasto.kirjaa(true, 60);
    tilasto.kirjaa(false, 30);
    assert_eq!(
        tilasto,
        Tilasto {
            voitot: 3,
            tappiot: 1,
            vahiten_laukauksia: 40
        }
    );
}

#[test]
fn nappaimisto_siirtaa_kursoria_rajojen_sisalla() {
    let mut peli = pelissa(1);
    peli.kasittele(Syote::Nuoli(-1, -1));
    assert_eq!(peli.kursori, Ruutu { rivi: 0, sarake: 0 });
    peli.kasittele(Syote::Kirjain('J'));
    peli.kasittele(Syote::Numero(0));
    assert_eq!(peli.kursori.to_string(), "J10");
    peli.kasittele(Syote::Nuoli(1, 1));
    assert_eq!(peli.kursori.to_string(), "J10");
    peli.kasittele(Syote::Numero(3));
    peli.kasittele(Syote::Nuoli(0, -1));
    assert_eq!(peli.kursori.to_string(), "I3");
}

#[test]
fn klikkaus_ampuu_ja_vastustaja_vastaa() {
    let mut peli = pelissa(1);
    let (x, y) = piirto::ruudun_keskipiste(Ruutu { rivi: 2, sarake: 3 });
    assert!(peli.kasittele(Syote::Klikkaus(x, y)));
    assert_eq!(peli.vastustaja.ammutut().maara(), 1);
    assert_eq!(peli.oma.ammutut().maara(), 1);
    assert!(peli.viestit[0].starts_with("Sinä ammuit D3: "));
    assert!(peli.viestit[1].starts_with("Vastustaja ampui "));
}

#[test]
fn klikkaus_laudan_ulkopuolelle_ei_tee_mitaan() {
    let mut peli = pelissa(1);
    assert!(!peli.kasittele(Syote::Klikkaus(0, 0)));
    assert_eq!(peli.vastustaja.ammutut().maara(), 0);
}

#[test]
fn sama_ruutu_antaa_huomautuksen_eika_kierrosta() {
    let mut peli = pelissa(1);
    peli.kasittele(Syote::Ammu);
    peli.edista(VASTAUS_VIIVE_MS + TULOS_VIIVE_MS);
    peli.kasittele(Syote::Ammu);
    assert!(peli.ammuttu_jo);
    assert_eq!(peli.oma.ammutut().maara(), 1);
}

#[test]
fn uusi_laukaus_odottaa_vastustajan_vuoroa() {
    let mut peli = pelissa(1);
    peli.kasittele(Syote::Ammu);
    peli.kasittele(Syote::Nuoli(0, 1));
    assert!(!peli.kasittele(Syote::Ammu));
    let (x, y) = piirto::ruudun_keskipiste(Ruutu { rivi: 5, sarake: 5 });
    peli.kasittele(Syote::Klikkaus(x, y));
    assert_eq!(peli.vastustaja.ammutut().maara(), 1);
    assert_eq!(peli.oma.ammutut().maara(), 1);

    peli.edista(VASTAUS_VIIVE_MS + TULOS_VIIVE_MS);
    assert!(peli.kasittele(Syote::Ammu));
    assert_eq!(peli.vastustaja.ammutut().maara(), 2);
}

#[test]
fn uponneen_laivan_ymparys_merkitaan_vasta_osuman_jalkeen() {
    let mut peli = pelissa(1);
    assert_eq!(peli.varmasti_tyhjat().maara(), 0);
    let laiva = peli.vastustaja.laivat()[4];
    let ruudut: Vec<_> = (0..KOKO * KOKO)
        .filter_map(|i| Ruutu::new(i / KOKO, i % KOKO))
        .filter(|&r| laiva.ruudut.sisaltaa(r))
        .collect();
    peli.ammu(ruudut[0]);
    peli.ammu(ruudut[1]);
    assert_eq!(peli.varmasti_tyhjat().maara(), 0);

    peli.edista(TULOS_VIIVE_MS);
    let tyhjat = peli.varmasti_tyhjat();
    assert!(tyhjat.maara() > 0);
    assert!(!tyhjat.leikkaako(laiva.ruudut));
    assert!(tyhjat.on_osajoukko(laiva.ruudut.ymparys()));
    for muu in peli.vastustaja.laivat() {
        assert!(!tyhjat.leikkaako(muu.ruudut));
    }
}

#[test]
fn voiton_jalkeen_vastustaja_ei_ammu_ja_tappio_tunnistetaan() {
    let mut voittoja = 0;
    let mut tappioita = 0;
    for siemen in 0..200 {
        let mut peli = pelissa(siemen);
        for i in 0..KOKO * KOKO {
            peli.ammu(Ruutu::new(i / KOKO, i % KOKO).unwrap());
            if peli.vaihe != Vaihe::Ammunta {
                break;
            }
        }
        let omat = peli.oma.ammutut().maara();
        let vastustajan = peli.vastustaja.ammutut().maara();
        match peli.vaihe {
            Vaihe::Nimi => {
                voittoja += 1;
                assert!(peli.viestit[1].contains(&peli.pisteet().to_string()));
                assert_eq!(omat + 1, vastustajan);
            }
            Vaihe::Tappio => {
                tappioita += 1;
                assert_eq!(omat, vastustajan);
            }
            _ => panic!("peli ei päättynyt"),
        }
    }
    assert!(voittoja > 0 && tappioita > 0);
}

#[test]
fn lopussa_k_aloittaa_alusta_ja_e_lopettaa() {
    let mut peli = pelissa(1);
    peli.ammu(Ruutu { rivi: 0, sarake: 0 });
    peli.vaihe = Vaihe::Tappio;
    assert!(peli.kasittele(Syote::Kirjain('K')));
    assert_eq!(peli.vaihe, Vaihe::Alku);
    assert_eq!(peli.oma.ammutut().maara(), 0);
    assert!(peli.viestit[0].is_empty());

    let mut peli = pelissa(1);
    peli.vaihe = Vaihe::Voitto;
    peli.kasittele(Syote::Kirjain('E'));
    assert!(peli.lopeta);
}

#[test]
fn lopussa_napit_toimivat_hiirella() {
    let mut peli = pelissa(1);
    peli.vaihe = Vaihe::Voitto;
    let (x, y) = piirto::napin_keskipiste(piirto::Nappi::Uudestaan);
    assert!(peli.kasittele(Syote::Klikkaus(x, y)));
    assert_eq!(peli.vaihe, Vaihe::Alku);

    let mut peli = pelissa(1);
    peli.vaihe = Vaihe::Voitto;
    let (x, y) = piirto::napin_keskipiste(piirto::Nappi::Lopeta);
    peli.kasittele(Syote::Klikkaus(x, y));
    assert!(peli.lopeta);
}

#[test]
fn aanet_kytketaan_napilla_ja_kytkimella_kesken_pelin() {
    let mut peli = pelissa(1);
    assert!(peli.aanet.paalla);
    assert!(peli.kasittele(Syote::VaihdaAanet));
    assert!(!peli.aanet.paalla);
    let (x, y) = aanikytkimen_keskipiste();
    assert!(peli.kasittele(Syote::Klikkaus(x, y)));
    assert!(peli.aanet.paalla);
    assert_eq!(peli.vastustaja.ammutut().maara(), 0);
}

#[test]
fn kytkimen_klikkaus_ei_aloita_pelia_ja_valinta_sailyy_uusinnassa() {
    let mut peli = Peli::new(1);
    let (x, y) = aanikytkimen_keskipiste();
    peli.kasittele(Syote::Klikkaus(x, y));
    assert_eq!(peli.vaihe, Vaihe::Alku);
    assert!(!peli.aanet.paalla);

    peli.kasittele(Syote::Ammu);
    peli.vaihe = Vaihe::Voitto;
    peli.kasittele(Syote::Kirjain('K'));
    assert!(!peli.aanet.paalla);
}

#[test]
fn kierros_soittaa_laukauksen_ja_vastustajan_tulipallon() {
    let mut peli = pelissa(1);
    peli.aanet.jono.clear();
    peli.ammu(Ruutu { rivi: 0, sarake: 0 });
    assert_eq!(peli.aanet.jono[0], (Aani::Laukaus, 0));
    assert!(
        peli.aanet
            .jono
            .contains(&(Aani::Vastalaukaus, VASTAUS_VIIVE_MS))
    );

    peli.aanet.jono.clear();
    peli.ammu(Ruutu { rivi: 0, sarake: 0 });
    assert_eq!(peli.aanet.jono, [(Aani::Virhe, 0)]);
}

#[test]
fn tulos_paljastuu_vasta_osumahetkella_ja_tehoste_poistuu() {
    let mut peli = pelissa(1);
    let ruutu = Ruutu { rivi: 0, sarake: 0 };
    peli.ammu(ruutu);
    assert!(peli.odottaa_osumaa(false, ruutu));
    assert!(peli.vastaus_kesken());

    peli.edista(TULOS_VIIVE_MS);
    assert!(!peli.odottaa_osumaa(false, ruutu));
    assert!(peli.vastaus_kesken());

    peli.edista(VASTAUS_VIIVE_MS);
    assert!(!peli.vastaus_kesken());
    assert!(!peli.tehosteet.is_empty());

    peli.edista(TEHOSTE_KESTO_MS);
    assert!(peli.tehosteet.is_empty());
}

#[test]
fn taso_valitaan_alkuruudulla_ja_sailyy_uusinnassa() {
    let mut peli = Peli::new(1);
    assert_eq!(peli.vaikeus, Vaikeus::Normaali);
    peli.kasittele(Syote::Numero(3));
    assert_eq!(peli.vaikeus, Vaikeus::Vaikea);
    peli.kasittele(Syote::Nuoli(0, 1));
    assert_eq!(peli.vaikeus, Vaikeus::Vaikea);
    peli.kasittele(Syote::Nuoli(0, -1));
    assert_eq!(peli.vaikeus, Vaikeus::Normaali);
    peli.kasittele(Syote::Numero(0));
    peli.kasittele(Syote::Numero(4));
    assert_eq!(peli.vaikeus, Vaikeus::Normaali);

    let (x, y) = piirto::vaikeuden_keskipiste(Vaikeus::Helppo);
    peli.kasittele(Syote::Klikkaus(x, y));
    assert_eq!(peli.vaikeus, Vaikeus::Helppo);
    assert_eq!(peli.vaihe, Vaihe::Alku);

    peli.kasittele(Syote::Klikkaus(0, 0));
    assert_eq!(peli.vaihe, Vaihe::Sijoitus);
    peli.vaihe = Vaihe::Voitto;
    peli.kasittele(Syote::Kirjain('K'));
    assert_eq!(peli.vaikeus, Vaikeus::Helppo);
}

#[test]
fn numerot_eivat_vaihda_tasoa_kesken_pelin() {
    let mut peli = pelissa(1);
    peli.kasittele(Syote::Numero(1));
    assert_eq!(peli.vaikeus, Vaikeus::Normaali);
    assert_eq!(peli.kursori.rivi, 0);
}

/// Ampuu ruudut järjestyksessä, kunnes peli päättyy. `None`, jos se päättyi tappioon.
fn voitettu(siemen: u64) -> Option<Peli> {
    let mut peli = pelissa(siemen);
    for i in 0..KOKO * KOKO {
        peli.ammu(Ruutu::new(i / KOKO, i % KOKO).unwrap());
        if peli.vaihe != Vaihe::Ammunta {
            break;
        }
    }
    (peli.vaihe != Vaihe::Tappio).then_some(peli)
}

#[test]
fn listalle_yltava_voitto_kysyy_nimen_ja_kirjaa_pisteet() {
    let mut peli = (0..).find_map(voitettu).unwrap();
    assert_eq!(peli.vaihe, Vaihe::Nimi);
    let pisteet = pisteet(peli.vaikeus, peli.laukauksia());
    assert_eq!(peli.pisteet(), pisteet);
    // Välilyönti ei kuittaa tyhjää nimeä, eivätkä kirjaimet ammu tai lopeta peliä.
    assert!(!peli.kasittele(Syote::Ammu));
    peli.kasittele(Syote::Kirjain('A'));
    peli.kasittele(Syote::VaihdaAanet);
    peli.kasittele(Syote::Kirjain('E'));
    assert!(peli.aanet.paalla);
    assert!(!peli.lopeta);
    peli.kasittele(Syote::Ammu);
    assert_eq!(peli.vaihe, Vaihe::Voitto);
    let tulos = Tulos {
        nimi: "AME".to_string(),
        pisteet,
    };
    assert_eq!(peli.taulu.parhaat, std::slice::from_ref(&tulos));
    assert_eq!(peli.taulu.lahetys, vec![tulos]);
    assert!(peli.uusi_tulos);
    assert_eq!(peli.pisteet(), pisteet);
}

#[test]
fn listalle_yltamaton_voitto_ja_tappio_eivat_kysy_nimea() {
    let taysi = vec![
        Tulos {
            nimi: "X".to_string(),
            pisteet: 999,
        };
        3
    ];
    let mut peli = pelissa(0);
    peli.taulu.netti = Some(taysi);
    for i in 0..KOKO * KOKO {
        peli.ammu(Ruutu::new(i / KOKO, i % KOKO).unwrap());
        if peli.vaihe != Vaihe::Ammunta {
            break;
        }
    }
    assert!(matches!(peli.vaihe, Vaihe::Voitto | Vaihe::Tappio));
    assert_eq!(peli.taulu.lahetys, Vec::new());
    if peli.vaihe == Vaihe::Tappio {
        assert_eq!(peli.pisteet(), 0);
    }
}

#[test]
fn ikkunan_sulkeminen_kirjaa_nimea_odottavan_voiton() {
    let mut peli = (0..).find_map(voitettu).unwrap();
    Pelinakyma::viimeistele(&mut peli);
    assert_eq!(peli.taulu.parhaat[0].nimi, "NIMETÖN");
    let listat = peli.listat();
    assert_eq!(listat[0].peli, Nettipeli::Laivanupotus);
    assert!(!listat[0].lahetys.is_empty());
}

#[test]
fn tilaston_paras_voitto_muuttuu_pisteiksi() {
    let tilasto = Tilasto {
        voitot: 2,
        tappiot: 1,
        vahiten_laukauksia: 60,
    };
    assert_eq!(tilasto.parhaat_pisteet(Vaikeus::Vaikea), 123);
    assert_eq!(Tilasto::default().parhaat_pisteet(Vaikeus::Vaikea), 0);
}

#[test]
fn esc_lopettaa_missa_tahansa_vaiheessa() {
    let mut peli = Peli::new(1);
    peli.kasittele(Syote::Peru);
    assert!(peli.lopeta);
}
