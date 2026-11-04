/// Ikkunakirjastosta riippumaton syöte. Hiiren koordinaatit ovat kuvapuskurin pikseleitä.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Syote {
    Hiiri(i32, i32),
    Klikkaus(i32, i32),
    /// Hiiren oikea nappi painui alas.
    OikeaKlikkaus(i32, i32),
    /// Hiiren nappi nousi: vedetty kortti putoaa.
    Vapautus(i32, i32),
    Nuoli(i8, i8),
    /// Hiiren rulla: -1 ylös, 1 alas.
    Rulla(i8),
    /// Sarakekirjain A-J, uusinnan K/E tai nimimerkin kirjain.
    Kirjain(char),
    /// Rivinäppäin, jossa 0 tarkoittaa riviä 10.
    Numero(u8),
    /// Nimimerkin viimeisen merkin poisto.
    Pyyhi,
    Ammu,
    Peru,
    VaihdaAanet,
}

/// Syötteen osa, jonka kaikki pelit käsittelevät samoin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Yleinen {
    /// Nimeä kysytään, ja syöte kuuluu nimikentälle.
    Nimi,
    /// Esc. Peli päättää itse, perutaanko valinta, pysähdytäänkö vai poistutaanko.
    Peru,
    /// Äänet vaihtuivat.
    Aanet,
    /// Nimeä kysyttäessä klikkaus muualle kuin äänikytkimeen ei tee mitään.
    Ohita,
    /// Pelin oma syöte.
    Peli,
}

/// Lajittelee syötteen ja vaihtaa äänet, kun syöte on M tai klikkaus äänikytkimeen.
/// Klikkaus ohittaa nimikentän, jotta äänikytkin toimii nimeä kirjoittaessakin.
pub fn yleinen(
    syote: Syote,
    nimi: bool,
    aanet_paalla: &mut bool,
    kytkimella: impl FnOnce(i32, i32) -> bool,
) -> Yleinen {
    if nimi && !matches!(syote, Syote::Klikkaus(..)) {
        return Yleinen::Nimi;
    }
    if syote == Syote::Peru {
        return Yleinen::Peru;
    }
    if syote == Syote::VaihdaAanet || matches!(syote, Syote::Klikkaus(x, y) if kytkimella(x, y)) {
        *aanet_paalla = !*aanet_paalla;
        return Yleinen::Aanet;
    }
    if nimi { Yleinen::Ohita } else { Yleinen::Peli }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yleinen_syote_lajitellaan_nimen_aanten_ja_pelin_kesken() {
        let kytkin = |x: i32, y: i32| (x, y) == (1, 1);
        let mut aanet = true;
        let lajittele = |syote, nimi, aanet: &mut bool| yleinen(syote, nimi, aanet, kytkin);

        // Nimeä kirjoittaessa kirjaimet, M ja Esc kuuluvat nimikentälle.
        for syote in [
            Syote::Kirjain('M'),
            Syote::VaihdaAanet,
            Syote::Peru,
            Syote::Ammu,
        ] {
            assert_eq!(lajittele(syote, true, &mut aanet), Yleinen::Nimi);
        }
        assert!(aanet);
        // Klikkaus ohittaa nimikentän: kytkin vaihtaa äänet, muu kohta ei tee mitään.
        assert_eq!(
            lajittele(Syote::Klikkaus(1, 1), true, &mut aanet),
            Yleinen::Aanet
        );
        assert!(!aanet);
        assert_eq!(
            lajittele(Syote::Klikkaus(5, 5), true, &mut aanet),
            Yleinen::Ohita
        );

        assert_eq!(lajittele(Syote::Peru, false, &mut aanet), Yleinen::Peru);
        assert_eq!(
            lajittele(Syote::VaihdaAanet, false, &mut aanet),
            Yleinen::Aanet
        );
        assert!(aanet);
        assert_eq!(
            lajittele(Syote::Klikkaus(5, 5), false, &mut aanet),
            Yleinen::Peli
        );
        assert_eq!(lajittele(Syote::Ammu, false, &mut aanet), Yleinen::Peli);
        assert!(aanet);
    }
}
