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
