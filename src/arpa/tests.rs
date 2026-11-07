use super::*;

#[test]
fn pysyy_rajoissa_ja_kattaa_kaikki_arvot() {
    let mut arpa = Arpa::new(7);
    let mut nahty = [false; 10];
    for _ in 0..1000 {
        nahty[arpa.alle(10)] = true;
    }
    assert!(nahty.iter().all(|&n| n));
}

#[test]
fn sama_siemen_antaa_saman_sarjan() {
    let (mut a, mut b) = (Arpa::new(1), Arpa::new(1));
    assert!((0..50).all(|_| a.alle(100) == b.alle(100)));
}
