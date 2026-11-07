use super::*;

#[test]
fn kaikki_fraasien_aanteet_tunnetaan() {
    for (fraasi, _) in super::super::FRAASIT.iter().flatten() {
        for merkki in fraasi.split_whitespace() {
            let merkki = merkki.trim_end_matches([':', '!']);
            assert!(
                merkki == "_" || aanne(merkki).is_some() || klusiili(merkki).is_some(),
                "{merkki}"
            );
        }
    }
}

#[test]
fn puhe_pysyy_rajoissa_ja_kestaa_kohtuullisesti() {
    for (fraasi, savel) in super::super::FRAASIT.iter().flatten() {
        let aani = puhu(fraasi, *savel, &mut Arpa::new(1));
        assert!(
            aani.iter().all(|x| x.is_finite() && x.abs() <= 1.0),
            "{fraasi}"
        );
        let kesto = aani.len() as f32 / FS;
        assert!((0.4..2.5).contains(&kesto), "{fraasi}: {kesto} s");
    }
}
