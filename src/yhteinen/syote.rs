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
