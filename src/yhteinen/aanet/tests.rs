use super::*;

fn soi(mikseri: &mut Mikseri) -> bool {
    // Puhe voi alkaa klusiilin hiljaisella sulkeumalla, joten katsotaan 0,1 s.
    let mut puskuri = [0.0; NAYTETAAJUUS / 10];
    mikseri.tayta(&mut puskuri);
    puskuri.iter().any(|&nayte| nayte != 0.0)
}

#[test]
fn jokainen_aani_soi_ja_loppuu() {
    let mut mikseri = Mikseri::new();
    kieli::kaikilla_kielilla(|| {
        for aani in KAIKKI {
            mikseri.soita(aani, 0);
            assert!(soi(&mut mikseri), "{aani:?} on hiljaa");
            let mut puskuri = vec![0.0; NAYTETAAJUUS * 3];
            mikseri.tayta(&mut puskuri);
            assert!(mikseri.soivat.is_empty(), "{aani:?} ei lopu");
            assert!(puskuri.iter().all(|nayte| nayte.abs() <= 1.0));
        }
    });
}

#[test]
fn tehosteet_ovat_lyhyita_ja_puhe_omissa_paikoissaan() {
    let mikseri = Mikseri::new();
    assert_eq!(mikseri.naytteet.len(), TEHOSTEITA + 2 * PUHEITA);
    for aani in KAIKKI {
        if aani.puhe().is_none() {
            assert!(aani.indeksi(Kieli::Suomi) < TEHOSTEITA, "{aani:?}");
            let kesto = mikseri.naytteet(aani, Kieli::Suomi).len() as f32 / FS;
            assert!(kesto <= 1.6, "{aani:?}: {kesto} s");
        } else {
            assert_ne!(
                mikseri.naytteet(aani, Kieli::Suomi),
                mikseri.naytteet(aani, Kieli::Englanti),
            );
        }
    }
}

#[test]
fn puhe_soi_valitulla_kielella() {
    let mut mikseri = Mikseri::new();
    let pituus = |kieli| mikseri.naytteet(Aani::EiSiirtoja, kieli).len();
    let (suomi, englanti) = (pituus(Kieli::Suomi), pituus(Kieli::Englanti));
    kieli::aseta(Kieli::Englanti);
    mikseri.soita(Aani::EiSiirtoja, 0);
    kieli::aseta(Kieli::Suomi);
    assert_eq!(mikseri.naytteet[mikseri.soivat[0].indeksi].len(), englanti);
    assert_ne!(suomi, englanti);
}

#[test]
fn pois_kytkeminen_hiljentaa_soivan_ja_odottavan_aanen_heti() {
    let mut mikseri = Mikseri::new();
    mikseri.soita(Aani::Tappio, 0);
    mikseri.soita(Aani::Voitto, 1000);
    assert!(soi(&mut mikseri));
    mikseri.aseta_paalle(false);
    assert!(!soi(&mut mikseri));

    mikseri.soita(Aani::Laukaus, 0);
    assert!(!soi(&mut mikseri));

    // takaisin kytkettynä vanhat äänet eivät palaa, uudet soivat
    mikseri.aseta_paalle(true);
    assert!(!soi(&mut mikseri));
    mikseri.soita(Aani::Laukaus, 0);
    assert!(soi(&mut mikseri));
}

#[test]
fn viive_siirtaa_aanen_alkua() {
    let mut mikseri = Mikseri::new();
    mikseri.soita(Aani::Laukaus, 1000);
    let mut puskuri = vec![0.0; NAYTETAAJUUS - 1];
    mikseri.tayta(&mut puskuri);
    assert!(puskuri.iter().all(|&nayte| nayte == 0.0));
    assert!(soi(&mut mikseri));
}

#[test]
fn puskurin_koko_ei_muuta_aanta() {
    let soita = |mikseri: &mut Mikseri| {
        mikseri.soita(Aani::Laukaus, 0);
        mikseri.soita(Aani::Roiske, 7);
        mikseri.soita(Aani::Voitto, 300);
    };
    let mut kerralla = Mikseri::new();
    soita(&mut kerralla);
    let mut odotettu = vec![0.0; NAYTETAAJUUS];
    kerralla.tayta(&mut odotettu);

    let mut paloina = Mikseri::new();
    soita(&mut paloina);
    let mut saatu = Vec::new();
    for koko in [1, 3, 100, 223, 1000].into_iter().cycle() {
        let koko = koko.min(NAYTETAAJUUS - saatu.len());
        if koko == 0 {
            break;
        }
        let mut pala = vec![0.0; koko];
        paloina.tayta(&mut pala);
        saatu.extend(pala);
    }
    assert_eq!(saatu, odotettu);
}

#[test]
fn soivia_on_rajattu_maara_ja_odottava_puhe_sailyy() {
    let mut mikseri = Mikseri::new();
    mikseri.soita(Aani::Voitit, PUHE_VIIVE_MS);
    for _ in 0..3 * ENINTAAN_SOIVIA {
        mikseri.soita(Aani::Kivi, 0);
        let mut puskuri = vec![0.0; 64];
        mikseri.tayta(&mut puskuri);
    }
    assert_eq!(mikseri.soivat.len(), ENINTAAN_SOIVIA);
    let puhe = Aani::Voitit.indeksi(kieli::nykyinen());
    assert!(mikseri.soivat.iter().any(|s| s.indeksi == puhe));
}
