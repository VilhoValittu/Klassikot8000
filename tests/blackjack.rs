use klassikot8000::Arpa;
use klassikot8000::blackjack::{
    ALKURAHA, ENITEN_KASIA, JAKAJA_JAA, Kortti, Maa, PAKKOJA, PIENIN_PANOS, Poyta, Tulos, Vuoro,
    YKSI, summa,
};

fn k(arvo: u8) -> Kortti {
    Kortti {
        maa: Maa::Pata,
        arvo,
    }
}

/// Kokonaiset rahat logiikan puolikkaiksi.
fn r(rahaa: u32) -> u32 {
    rahaa * YKSI
}

/// Pöytä, josta kortit nousevat annetussa järjestyksessä. Jako: pelaaja, jakaja, pelaaja, jakaja.
fn poyta(arvot: &[u8]) -> (Poyta, Arpa) {
    let mut arpa = Arpa::new(1);
    let mut poyta = Poyta::new(&mut arpa);
    let mut kenka = vec![k(2); 100];
    kenka.extend(arvot.iter().rev().map(|&arvo| k(arvo)));
    poyta.kenka = kenka;
    (poyta, arpa)
}

fn pelaa_jakaja(poyta: &mut Poyta) {
    while poyta.vuoro == Vuoro::Jakaja {
        poyta.jakajan_siirto();
    }
}

#[test]
fn assa_on_yksitoista_kunnes_kasi_menisi_yli_ja_kuvakortit_ovat_kymppeja() {
    assert_eq!(summa(&[k(1), k(6)]), (17, true));
    assert_eq!(summa(&[k(1), k(6), k(10)]), (17, false));
    assert_eq!(summa(&[k(1), k(1)]), (12, true));
    assert_eq!(summa(&[k(1), k(1), k(9)]), (21, true));
    assert_eq!(summa(&[k(11), k(12), k(13)]), (30, false));
    assert_eq!(summa(&[]), (0, false));
}

#[test]
fn alussa_on_300_rahaa_ja_kuusi_pakkaa() {
    let poyta = Poyta::new(&mut Arpa::new(1));
    assert_eq!(poyta.rahat, r(300));
    assert_eq!(ALKURAHA, r(300));
    assert_eq!(PIENIN_PANOS, r(1));
    assert_eq!(poyta.kenka.len(), PAKKOJA * 52);
}

#[test]
fn panos_rajataan_pienimmasta_rahoihin_ja_on_kokonaisia_rahoja() {
    let (mut poyta, mut arpa) = poyta(&[10, 10, 8, 7]);
    assert!(!poyta.jaa(0, &mut arpa));
    assert!(!poyta.jaa(1, &mut arpa));
    assert!(!poyta.jaa(r(5) + 1, &mut arpa));
    assert!(!poyta.jaa(ALKURAHA + YKSI, &mut arpa));
    assert_eq!(poyta.vuoro, Vuoro::Panostus);
    assert!(poyta.jaa(PIENIN_PANOS, &mut arpa));
    assert_eq!(poyta.rahat, ALKURAHA - r(1));
    assert!(!poyta.jaa(r(1), &mut arpa));
}

#[test]
fn blackjack_maksaa_tasan_kolme_kahteen() {
    // puolikkaina: panos 1 voittaa 1,5 ja panos 3 voittaa 4,5
    for (panos, voitto) in [(r(10), r(15)), (r(1), 3), (r(3), 9), (r(5), 15)] {
        let (mut poyta, mut arpa) = poyta(&[1, 9, 13, 8]);
        assert!(poyta.jaa(panos, &mut arpa));
        assert_eq!(poyta.vuoro, Vuoro::Panostus);
        assert!(!poyta.piilossa);
        assert_eq!(poyta.kadet[0].tulos, Some(Tulos::Blackjack));
        assert_eq!(poyta.rahat, ALKURAHA + voitto);
        assert_eq!(poyta.voitto(), voitto as i64);
    }
}

#[test]
fn jakajan_blackjack_kympilla_esilla_lopettaa_kierroksen_heti_ilman_vakuutusta() {
    let (mut p, mut arpa) = poyta(&[10, 13, 9, 1]);
    assert!(p.jaa(r(10), &mut arpa));
    assert_eq!(p.vuoro, Vuoro::Panostus);
    assert!(!p.piilossa);
    assert_eq!(p.kadet[0].tulos, Some(Tulos::Tappio));
    assert_eq!(p.rahat, ALKURAHA - r(10));

    // molemmilla blackjack: panos palaa
    let (mut p, mut arpa) = poyta(&[1, 12, 10, 1]);
    assert!(p.jaa(r(10), &mut arpa));
    assert_eq!(p.kadet[0].tulos, Some(Tulos::Tasan));
    assert_eq!(p.rahat, ALKURAHA);
}

#[test]
fn vakuutus_tarjotaan_vain_assasta_ja_ennen_kurkistusta() {
    let (mut p, mut arpa) = poyta(&[10, 1, 9, 13]);
    assert!(p.jaa(r(10), &mut arpa));
    assert_eq!(p.vuoro, Vuoro::Vakuutus);
    assert!(p.piilossa);
    assert_eq!(p.vakuutuksen_hinta(), r(5));
    assert!(!p.ota());
    assert!(!p.jaa_tahan());
    assert!(!p.voi_tuplata());
    assert!(!p.halkaise());
    assert!(!p.jaa(r(1), &mut arpa));

    let (mut p, mut arpa) = poyta(&[10, 9, 9, 1]);
    assert!(p.jaa(r(10), &mut arpa));
    assert_eq!(p.vuoro, Vuoro::Pelaaja);
    assert!(!p.vakuuta(true));
    assert_eq!(p.vakuutus, 0);
}

#[test]
fn vakuutus_maksaa_kaksi_yhteen_jakajan_blackjackista() {
    let (mut p, mut arpa) = poyta(&[10, 1, 9, 13]);
    assert!(p.jaa(r(10), &mut arpa));
    assert!(p.vakuuta(true));
    assert_eq!(p.vuoro, Vuoro::Panostus);
    assert!(!p.piilossa);
    assert_eq!(p.kadet[0].tulos, Some(Tulos::Tappio));
    assert!(p.vakuutus_voitti());
    assert_eq!(p.rahat, ALKURAHA);
    assert_eq!(p.voitto(), 0);

    // ilman vakuutusta panos menee
    let (mut p, mut arpa) = poyta(&[10, 1, 9, 13]);
    assert!(p.jaa(r(10), &mut arpa));
    assert!(p.vakuuta(false));
    assert!(!p.vakuutus_voitti());
    assert_eq!(p.rahat, ALKURAHA - r(10));
    assert_eq!(p.voitto(), -(r(10) as i64));
}

#[test]
fn havitty_vakuutus_jaa_jakajalle_ja_peli_jatkuu() {
    // pelaaja 19, jakaja pehmeä 17
    let (mut p, mut arpa) = poyta(&[10, 1, 9, 6]);
    assert!(p.jaa(r(10), &mut arpa));
    assert!(p.vakuuta(true));
    assert_eq!(p.vuoro, Vuoro::Pelaaja);
    assert!(p.piilossa);
    assert_eq!(p.rahat, ALKURAHA - r(15));
    assert!(p.jaa_tahan());
    pelaa_jakaja(&mut p);
    assert_eq!(p.kadet[0].tulos, Some(Tulos::Voitto));
    assert!(!p.vakuutus_voitti());
    assert_eq!(p.rahat, ALKURAHA + r(5));
    assert_eq!(p.voitto(), r(5) as i64);

    // seuraava jako alkaa ilman vakuutusta
    assert!(p.jaa(r(1), &mut arpa));
    assert_eq!(p.vakuutus, 0);
}

#[test]
fn tasaraha_antaa_saman_voiton_kortista_riippumatta() {
    for piilokortti in [13, 6] {
        let (mut p, mut arpa) = poyta(&[1, 1, 13, piilokortti]);
        assert!(p.jaa(r(10), &mut arpa));
        assert_eq!(p.vuoro, Vuoro::Vakuutus);
        assert!(p.vakuuta(true));
        assert_eq!(p.vuoro, Vuoro::Panostus);
        assert_eq!(p.rahat, ALKURAHA + r(10));
        assert_eq!(p.voitto(), r(10) as i64);
    }
    // ilman tasarahaa blackjack maksaa 3:2, jos jakajalla ei ole blackjackia
    let (mut p, mut arpa) = poyta(&[1, 1, 13, 6]);
    assert!(p.jaa(r(10), &mut arpa));
    assert!(p.vakuuta(false));
    assert_eq!(p.rahat, ALKURAHA + r(15));
}

#[test]
fn vakuutus_puolesta_panoksesta_menee_tasan_ja_vaatii_rahaa() {
    let (mut p, mut arpa) = poyta(&[10, 1, 9, 13]);
    assert!(p.jaa(r(1), &mut arpa));
    assert_eq!(p.vakuutuksen_hinta(), 1);
    assert!(p.vakuuta(true));
    assert_eq!(p.rahat, ALKURAHA);

    // kaikki rahat panoksena: vakuutusta ei voi maksaa, joten jakaja kurkistaa heti
    let (mut p, mut arpa) = poyta(&[10, 1, 9, 6]);
    assert!(p.jaa(ALKURAHA, &mut arpa));
    assert_eq!(p.vuoro, Vuoro::Pelaaja);
}

#[test]
fn yli_menevasta_kadesta_havitaan_eika_jakaja_nosta() {
    let (mut p, mut arpa) = poyta(&[10, 5, 6, 6, 9]);
    assert!(p.jaa(r(10), &mut arpa));
    assert!(p.ota());
    assert_eq!(p.vuoro, Vuoro::Jakaja);
    pelaa_jakaja(&mut p);
    assert_eq!(p.kadet[0].tulos, Some(Tulos::Tappio));
    assert_eq!(p.jakaja.len(), 2);
    assert!(!p.piilossa);
    assert_eq!(p.rahat, ALKURAHA - r(10));
}

#[test]
fn jakaja_nostaa_kuuteentoista_asti_ja_jaa_pehmeaan_17_seen() {
    // pelaaja 20, jakaja 5 + 6, nostaa 4 ja 2 = 17
    let (mut p, mut arpa) = poyta(&[10, 5, 10, 6, 4, 2, 9]);
    assert!(p.jaa(r(10), &mut arpa));
    assert!(p.jaa_tahan());
    assert!(p.piilossa);
    assert_eq!(p.rahat, ALKURAHA - r(10));
    // kortit kääntyvät yksi kerrallaan, ja tulos selviää viimeisellä
    p.jakajan_siirto();
    assert!(!p.piilossa);
    assert_eq!(p.jakaja.len(), 2);
    assert_eq!(p.vuoro, Vuoro::Jakaja);
    p.jakajan_siirto();
    assert_eq!(p.vuoro, Vuoro::Jakaja);
    p.jakajan_siirto();
    assert_eq!(p.vuoro, Vuoro::Panostus);
    assert_eq!(summa(&p.jakaja).0, JAKAJA_JAA);
    assert_eq!(p.kadet[0].tulos, Some(Tulos::Voitto));
    assert_eq!(p.rahat, ALKURAHA + r(10));

    // kuutonen esillä ja ässä piilossa on pehmeä 17
    let (mut p, mut arpa) = poyta(&[10, 6, 7, 1, 9]);
    assert!(p.jaa(r(10), &mut arpa));
    assert!(p.jaa_tahan());
    pelaa_jakaja(&mut p);
    assert_eq!(p.jakaja.len(), 2);
    assert_eq!(p.kadet[0].tulos, Some(Tulos::Tasan));
    assert_eq!(p.rahat, ALKURAHA);
}

#[test]
fn jakajan_yli_meneminen_voittaa_ja_suurempi_summa_ratkaisee() {
    let (mut p, mut arpa) = poyta(&[10, 10, 2, 6, 10]);
    assert!(p.jaa(r(20), &mut arpa));
    assert!(p.jaa_tahan());
    pelaa_jakaja(&mut p);
    assert_eq!(p.kadet[0].tulos, Some(Tulos::Voitto));
    assert_eq!(p.voitto(), r(20) as i64);

    let (mut p, mut arpa) = poyta(&[10, 10, 7, 8]);
    assert!(p.jaa(r(20), &mut arpa));
    assert!(p.jaa_tahan());
    pelaa_jakaja(&mut p);
    assert_eq!(p.kadet[0].tulos, Some(Tulos::Tappio));
    assert_eq!(p.voitto(), -(r(20) as i64));
}

#[test]
fn kaksikymmentayksi_paattaa_vuoron_itsestaan() {
    let (mut p, mut arpa) = poyta(&[10, 10, 5, 8, 6]);
    assert!(p.jaa(r(10), &mut arpa));
    assert!(p.ota());
    assert_eq!(p.vuoro, Vuoro::Jakaja);
    pelaa_jakaja(&mut p);
    assert_eq!(p.kadet[0].tulos, Some(Tulos::Voitto));
    // kolmen kortin 21 ei ole blackjack
    assert_eq!(p.rahat, ALKURAHA + r(10));
}

#[test]
fn tuplaus_kaksinkertaistaa_panoksen_ja_antaa_yhden_kortin() {
    let (mut p, mut arpa) = poyta(&[5, 10, 6, 9, 10]);
    assert!(p.jaa(r(10), &mut arpa));
    assert!(p.voi_tuplata());
    assert!(p.tuplaa());
    assert_eq!(p.kadet[0].panos, r(20));
    assert_eq!(p.kadet[0].kortit.len(), 3);
    assert_eq!(p.rahat, ALKURAHA - r(20));
    assert_eq!(p.vuoro, Vuoro::Jakaja);
    pelaa_jakaja(&mut p);
    assert_eq!(p.rahat, ALKURAHA + r(20));
}

#[test]
fn tuplaus_vaatii_rahaa_ja_kaksi_korttia() {
    let (mut p, mut arpa) = poyta(&[5, 10, 6, 9, 2]);
    assert!(p.jaa(ALKURAHA, &mut arpa));
    assert!(!p.voi_tuplata());
    assert!(!p.tuplaa());
    assert_eq!(p.rahat, 0);

    let (mut p, mut arpa) = poyta(&[5, 10, 6, 9, 2]);
    assert!(p.jaa(r(10), &mut arpa));
    assert!(p.ota());
    assert!(!p.voi_tuplata());
}

#[test]
fn halkaisu_tekee_kaksi_erikseen_tilitettavaa_katta() {
    // kädet 8 + 3 ja 8 + 2, jakaja 10 + 9 = 19
    let (mut p, mut arpa) = poyta(&[8, 10, 8, 9, 3, 10, 2]);
    assert!(p.jaa(r(10), &mut arpa));
    assert!(p.voi_halkaista());
    assert!(p.halkaise());
    assert_eq!(p.kadet.len(), 2);
    assert_eq!(p.rahat, ALKURAHA - r(20));
    assert_eq!(p.kadet[0].summa(), 11);
    assert_eq!(p.vuorossa, 0);

    // 11 + 10 = 21 voittaa, seuraava käsi jää kymppiin ja häviää
    assert!(p.ota());
    assert_eq!(p.vuorossa, 1);
    assert_eq!(p.kadet[1].summa(), 10);
    assert!(p.jaa_tahan());
    pelaa_jakaja(&mut p);
    assert_eq!(p.kadet[0].tulos, Some(Tulos::Voitto));
    assert_eq!(p.kadet[1].tulos, Some(Tulos::Tappio));
    assert_eq!(p.rahat, ALKURAHA);
    assert_eq!(p.voitto(), 0);
}

#[test]
fn halkaistu_kasi_saa_toisen_korttinsa_vasta_vuorollaan() {
    let (mut p, mut arpa) = poyta(&[8, 10, 8, 9, 3, 5, 7]);
    assert!(p.jaa(r(10), &mut arpa));
    assert!(p.halkaise());
    assert_eq!(p.kadet[0].kortit, [k(8), k(3)]);
    assert_eq!(p.kadet[1].kortit, [k(8)]);
    // edellinen käsi nostaa ensin, joten seuraava kortti menee sille
    assert!(p.ota());
    assert_eq!(p.kadet[0].kortit, [k(8), k(3), k(5)]);
    assert_eq!(p.kadet[1].kortit, [k(8)]);
    assert!(p.jaa_tahan());
    assert_eq!(p.vuorossa, 1);
    assert_eq!(p.kadet[1].kortit, [k(8), k(7)]);
    // kortin saatuaan käden voi tuplata
    assert!(p.voi_tuplata());
}

#[test]
fn halkaistut_assat_saavat_yhden_kortin_eika_21_ole_blackjack() {
    let (mut p, mut arpa) = poyta(&[1, 10, 1, 8, 13, 9]);
    assert!(p.jaa(r(10), &mut arpa));
    assert!(p.halkaise());
    // molemmat kädet valmiita heti
    assert_eq!(p.vuoro, Vuoro::Jakaja);
    assert_eq!(p.kadet[1].kortit.len(), 2);
    pelaa_jakaja(&mut p);
    assert_eq!(p.kadet[0].summa(), 21);
    assert_eq!(p.kadet[0].tulos, Some(Tulos::Voitto));
    assert_eq!(p.kadet[1].tulos, Some(Tulos::Voitto));
    assert_eq!(p.rahat, ALKURAHA + r(20));
}

#[test]
fn kuvakortit_kayvat_pariksi_ja_kasia_on_enintaan_nelja() {
    let (mut p, mut arpa) = poyta(&[10, 5, 13, 9, 12, 11, 10]);
    assert!(p.jaa(r(1), &mut arpa));
    for maara in 2..=ENITEN_KASIA {
        assert!(p.voi_halkaista());
        assert!(p.halkaise());
        assert_eq!(p.kadet.len(), maara);
    }
    assert!(!p.voi_halkaista());
    assert!(!p.halkaise());
    assert_eq!(p.rahat, ALKURAHA - r(ENITEN_KASIA as u32));
}

#[test]
fn eri_arvoista_ei_voi_halkaista_eika_ilman_rahaa() {
    let (mut p, mut arpa) = poyta(&[9, 10, 8, 9]);
    assert!(p.jaa(r(1), &mut arpa));
    assert!(!p.voi_halkaista());

    let (mut p, mut arpa) = poyta(&[8, 10, 8, 9]);
    assert!(p.jaa(ALKURAHA, &mut arpa));
    assert!(!p.voi_halkaista());
}

#[test]
fn siirrot_toimivat_vain_omalla_vuorolla() {
    let (mut p, _arpa) = poyta(&[10, 10, 8, 7]);
    assert!(!p.ota());
    assert!(!p.jaa_tahan());
    assert!(!p.tuplaa());
    assert!(!p.halkaise());
    assert!(!p.vakuuta(true));
    p.jakajan_siirto();
    assert_eq!(p.vuoro, Vuoro::Panostus);
    assert!(p.jakaja.is_empty());
}

/// Rahaa ei synny eikä katoa: pelataan satunnaisia kierroksia ja kirjanpidon on täsmättävä.
#[test]
fn rahat_tasmaavat_ja_kenka_riittaa_tuhansilla_kierroksilla() {
    for siemen in 1..=5 {
        let mut arpa = Arpa::new(siemen);
        let mut p = Poyta::new(&mut arpa);
        p.rahat = r(100_000);
        for _ in 0..3000 {
            let ennen = p.rahat;
            let panos = r(1 + arpa.alle(5) as u32);
            assert!(p.jaa(panos, &mut arpa));
            if p.vuoro == Vuoro::Vakuutus {
                assert!(p.vakuuta(arpa.alle(2) == 0));
            }
            while p.vuoro == Vuoro::Pelaaja {
                match arpa.alle(4) {
                    0 => {
                        p.ota();
                    }
                    1 => {
                        p.tuplaa();
                    }
                    2 => {
                        p.halkaise();
                    }
                    _ => {
                        p.jaa_tahan();
                    }
                }
            }
            pelaa_jakaja(&mut p);
            assert!(p.kadet.len() <= ENITEN_KASIA);
            assert!(
                p.kadet
                    .iter()
                    .all(|k| k.tulos.is_some() && k.kortit.len() >= 2)
            );
            assert_eq!(p.rahat as i64, ennen as i64 + p.voitto());
            // Jakaja jää alle rajan vain, jos kierros ratkesi ilman hänen korttejaan.
            let ratkesi = p.kadet[0].on_blackjack() || p.kadet.iter().all(|k| k.on_yli());
            assert!(ratkesi || summa(&p.jakaja).0 >= JAKAJA_JAA);
        }
    }
}

#[test]
fn tulos_on_voitto_kokonaisina_rahoina() {
    use klassikot8000::blackjack::{ALKURAHA, SUURIN_TULOS, YKSI};
    let mut poyta = klassikot8000::blackjack::Poyta::new(&mut klassikot8000::Arpa::new(1));
    assert_eq!(poyta.pisteet(), 0);
    poyta.rahat = ALKURAHA - YKSI;
    assert_eq!(poyta.pisteet(), 0);
    poyta.rahat = ALKURAHA + 1;
    assert_eq!(poyta.pisteet(), 0);
    poyta.rahat = ALKURAHA + 25 * YKSI + 1;
    assert_eq!(poyta.pisteet(), 25);
    poyta.rahat = u32::MAX;
    assert_eq!(poyta.pisteet(), SUURIN_TULOS);
}
