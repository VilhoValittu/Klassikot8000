/// Pieni xorshift64*-generaattori. Oma toteutus, koska pelissä ei tarvita enempää
/// ja jokainen riippuvuus kasvattaa pakettia.
#[derive(Clone)]
pub struct Arpa(u64);

impl Arpa {
    pub fn new(siemen: u64) -> Self {
        // SplitMix64-sekoitus: xorshift ei saa alkaa nollasta, ja lähekkäiset siemenet
        // antaisivat muuten samankaltaiset alut.
        let mut z = siemen.wrapping_add(0x9E37_79B9_7F4A_7C15);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        Self((z ^ (z >> 31)) | 1)
    }

    fn seuraava(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Luku väliltä `0..raja`. Raja on pelissä enintään 100, joten vinouma on merkityksetön.
    pub fn alle(&mut self, raja: usize) -> usize {
        (((self.seuraava() >> 32) * raja as u64) >> 32) as usize
    }
}

#[cfg(test)]
mod tests {
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
}
