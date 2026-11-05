//! Nettilistalle lähtevän tuloksen allekirjoitus (HMAC-SHA256). Palvelin hyväksyy vain tuloksen,
//! jonka allekirjoitus täsmää sen omaan avaimeen, joten pelkkä lähdekoodi ei riitä tuloksen
//! keksimiseen. Oma toteutus, koska yhden tiivisteen takia ei kannata ottaa riippuvuutta.

const LOHKO: usize = 64;

const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let mut viesti = data.to_vec();
    let bitteja = (data.len() as u64).wrapping_mul(8);
    // Täyte: 0x80, nollia ja viestin pituus bitteinä lohkon viimeisissä kahdeksassa tavussa.
    viesti.push(0x80);
    while viesti.len() % LOHKO != LOHKO - 8 {
        viesti.push(0);
    }
    viesti.extend_from_slice(&bitteja.to_be_bytes());
    for lohko in viesti.chunks_exact(LOHKO) {
        let mut w = [0u32; 64];
        for (sana, tavut) in w.iter_mut().zip(lohko.chunks_exact(4)) {
            *sana = u32::from_be_bytes([tavut[0], tavut[1], tavut[2], tavut[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for (&k, &sana) in K.iter().zip(&w) {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(k)
                .wrapping_add(sana);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (arvo, lisa) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
            *arvo = arvo.wrapping_add(lisa);
        }
    }
    let mut tiiviste = [0; 32];
    for (tavut, arvo) in tiiviste.chunks_exact_mut(4).zip(h) {
        tavut.copy_from_slice(&arvo.to_be_bytes());
    }
    tiiviste
}

pub fn hmac(avain: &[u8], viesti: &[u8]) -> [u8; 32] {
    let tiivistetty;
    let avain = if avain.len() > LOHKO {
        tiivistetty = sha256(avain);
        &tiivistetty[..]
    } else {
        avain
    };
    let mut sisa = vec![0x36; LOHKO];
    let mut ulko = vec![0x5c; LOHKO];
    for (i, &tavu) in avain.iter().enumerate() {
        sisa[i] ^= tavu;
        ulko[i] ^= tavu;
    }
    sisa.extend_from_slice(viesti);
    ulko.extend_from_slice(&sha256(&sisa));
    sha256(&ulko)
}

/// Tuloksen allekirjoitus heksoina. Viesti on pelin tunnus ja pyynnön runko rivinvaihdolla
/// erotettuina, joten samaa allekirjoitusta ei voi siirtää toisen pelin listalle.
pub fn allekirjoita(avain: &[u8], tunnus: &str, runko: &str) -> String {
    let viesti = format!("{tunnus}\n{runko}");
    hmac(avain, viesti.as_bytes())
        .iter()
        .map(|tavu| format!("{tavu:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn heksoina(tavut: &[u8]) -> String {
        tavut.iter().map(|tavu| format!("{tavu:02x}")).collect()
    }

    #[test]
    fn sha256_tasmaa_tunnettuihin_arvoihin() {
        for (syote, odotettu) in [
            (
                &b""[..],
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            ),
            (
                b"abc",
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            ),
            // 56 tavua: täyte ei mahdu samaan lohkoon pituuden kanssa.
            (
                b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
                "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
            ),
            (
                &[b'a'; 1000],
                "41edece42d63e8d9bf515a9ba6932e1c20cbc9f5a5d134645adb5db1b9737ea3",
            ),
        ] {
            assert_eq!(heksoina(&sha256(syote)), odotettu);
        }
    }

    #[test]
    fn hmac_tasmaa_rfc_4231_arvoihin() {
        assert_eq!(
            heksoina(&hmac(&[0x0b; 20], b"Hi There")),
            "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
        );
        assert_eq!(
            heksoina(&hmac(b"Jefe", b"what do ya want for nothing?")),
            "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
        );
        assert_eq!(
            heksoina(&hmac(
                &[0xaa; 131],
                b"Test Using Larger Than Block-Size Key - Hash Key First"
            )),
            "60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54"
        );
    }

    /// Sama arvo on palvelimen testissä, joten peli ja palvelin laskevat samoin.
    #[test]
    fn allekirjoitus_tasmaa_palvelimen_kanssa() {
        let avain: Vec<u8> = (0..32).collect();
        assert_eq!(
            allekirjoita(&avain, "luvut", "120 VÄINÖ7"),
            "b749393e7f9c87cb24073eef4b9abf4bb361b5f6e1a3b4ad4a21d57823dde500"
        );
    }
}
