use super::*;

#[test]
fn kuvat_ovat_tasalevyisia() {
    for kuva in [&HERTTA, &RUUTU, &PATA, &RISTI] {
        assert!(kuva.iter().all(|rivi| rivi.len() == 9));
    }
    for kuva in [&KUNINGAS_KUVA, &KUNINGATAR_KUVA, &JATKA_KUVA] {
        for rivi in kuva {
            assert!(rivi.len() as i32 <= KUVAN_LEVEYS, "{rivi}");
        }
    }
}

#[test]
fn hahmo_mahtuu_kehykseen() {
    const { assert!(KUVAN_LEVEYS * KUVAN_KERROIN <= KORTTI_L - 42) };
    const { assert!(2 * 13 * KUVAN_KERROIN <= KORTTI_K - 57) };
}
