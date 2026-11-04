use klassikot8000::Arpa;
use klassikot8000::luvut::{Lukulauta, Ruudut, Siirto, Suunta, TAVOITE};

use super::piirto;
use crate::yhteinen::aanet::{Aani, PUHE_VIIVE_MS};
use crate::yhteinen::kieli::{kaanna, muotoile};
use crate::yhteinen::pelinakyma::{Lista, Pelinakyma};
use crate::yhteinen::piirto::{Kehys, aanikytkimella};
use crate::yhteinen::syote::Syote;
use crate::yhteinen::tulokset::{
    Nettipeli, Tulos, lisaa_tulos, mahtuu_parhaisiin, muokkaa_nimea, ota_nimi,
};

pub const LIUKU_MS: u32 = 110;
pub const KIPINA_MS: u32 = 450;
const YHDISTYS_VIIVE_MS: u32 = 60;
const LOPPU_VIIVE_MS: u32 = 250;

/// Kesken jäänyt peli, joka jatkuu seuraavalla kerralla.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Kesken {
    pub ruudut: Ruudut,
    pub pisteet: u32,
    pub siirrot: u32,
    pub jatkettu: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vaihe {
    Alku,
    Pelissa,
    Voitto,
    Tappio,
    /// Tulos pääsi kolmen parhaan joukkoon ja odottaa nimimerkkiä.
    Nimi,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nappi {
    Uudestaan,
    Valikko,
    Jatka,
}

pub struct Luvut {
    pub vaihe: Vaihe,
    pub lauta: Lukulauta,
    pub paras: u32,
    pub parhaat: Vec<Tulos>,
    /// Nettilista. `None`, kunnes palvelin on vastannut.
    pub netti_parhaat: Option<Vec<Tulos>>,
    /// Kirjattu tulos, joka odottaa lähetystä nettilistalle.
    pub lahetys: Option<Tulos>,
    /// Kirjoitettavana oleva nimimerkki.
    pub nimi: String,
    /// Tosi, kun uusi tulos odottaa tiedostoon kirjoitusta.
    pub uusi_tulos: bool,
    pub siirtoja: u32,
    pub viesti: String,
    pub ei_liiku: bool,
    /// Suunta, johon hiiren klikkaus siirtäisi. Piirretään nuolena laudan reunaan.
    pub hiiren_suunta: Option<Suunta>,
    /// Viimeisin siirto ja sen alkuhetki animaatiota varten.
    pub viime_siirto: Option<(Siirto, u32)>,
    /// Pyyntö palata käynnistimeen.
    pub lopeta: bool,
    pub aanet_paalla: bool,
    pub aanijono: Vec<(Aani, u32)>,
    pub kello_ms: u32,
    /// Alkuruudulla odottaa tallennettu peli, jota voi jatkaa.
    pub jatkuu: bool,
    jatkettu: bool,
    /// Nimimerkin jälkeen alkaa uusi peli eikä näytetä tappiota.
    uusi_alkaa: bool,
    arpa: Arpa,
}

impl Luvut {
    pub fn new(siemen: u64) -> Self {
        let mut arpa = Arpa::new(siemen);
        Self {
            vaihe: Vaihe::Alku,
            lauta: Lukulauta::new(&mut arpa),
            paras: 0,
            parhaat: Vec::new(),
            netti_parhaat: None,
            lahetys: None,
            nimi: String::new(),
            uusi_tulos: false,
            siirtoja: 0,
            viesti: String::new(),
            ei_liiku: false,
            hiiren_suunta: None,
            viime_siirto: None,
            lopeta: false,
            aanet_paalla: true,
            aanijono: Vec::new(),
            kello_ms: 0,
            jatkuu: false,
            jatkettu: false,
            uusi_alkaa: false,
            arpa,
        }
    }

    /// Esillä oleva lista: nettilista, tai oma lista, jos verkkoon ei ole saatu yhteyttä.
    pub fn nakyva_lista(&self) -> &[Tulos] {
        self.netti_parhaat.as_deref().unwrap_or(&self.parhaat)
    }

    pub fn palauta(&mut self, kesken: Kesken) {
        self.lauta = Lukulauta::ruuduista(kesken.ruudut);
        self.lauta.pisteet = kesken.pisteet;
        self.siirtoja = kesken.siirrot;
        self.jatkettu = kesken.jatkettu;
        self.jatkuu = true;
        self.paras = self.paras.max(kesken.pisteet);
    }

    /// Tallennettava tilanne. Päättynyttä tai aloittamatonta peliä ei tallenneta.
    pub fn kesken(&self) -> Option<Kesken> {
        let tallennetaan = match self.vaihe {
            Vaihe::Alku => self.jatkuu,
            Vaihe::Pelissa | Vaihe::Voitto => true,
            Vaihe::Tappio => false,
            Vaihe::Nimi => self.uusi_alkaa,
        };
        tallennetaan.then(|| Kesken {
            ruudut: self.lauta.ruudut(),
            pisteet: self.lauta.pisteet,
            siirrot: self.siirtoja,
            // Voitto on jo juhlittu, joten sitä ei näytetä jatkettaessa uudelleen.
            jatkettu: self.jatkettu || self.vaihe == Vaihe::Voitto,
        })
    }

    pub fn edista(&mut self, ms: u32) {
        self.kello_ms = self.kello_ms.saturating_add(ms);
        let kello = self.kello_ms;
        if let Some((_, alku)) = self.viime_siirto
            && kello >= alku + LIUKU_MS + KIPINA_MS
        {
            self.viime_siirto = None;
        }
    }

    fn aloita_alusta(&mut self) {
        self.vaihe = Vaihe::Alku;
        self.lauta = Lukulauta::new(&mut self.arpa);
        self.siirtoja = 0;
        self.viesti.clear();
        self.ei_liiku = false;
        self.hiiren_suunta = None;
        self.viime_siirto = None;
        self.jatkuu = false;
        self.jatkettu = false;
    }

    /// Uusi peli hylkää nykyisen, joten hyvä tulos kirjataan ensin listalle.
    fn hylkaa_ja_aloita(&mut self) {
        if mahtuu_parhaisiin(self.nakyva_lista(), self.lauta.pisteet) {
            self.uusi_alkaa = true;
            self.kysy_nimi();
        } else {
            self.aloita_alusta();
        }
    }

    fn kysy_nimi(&mut self) {
        self.nimi.clear();
        self.vaihe = Vaihe::Nimi;
    }

    fn paata_peli(&mut self) {
        if mahtuu_parhaisiin(self.nakyva_lista(), self.lauta.pisteet) {
            self.kysy_nimi();
        } else {
            self.vaihe = Vaihe::Tappio;
        }
    }

    /// Kirjaa odottavan tuloksen.
    pub fn vahvista_nimi(&mut self) {
        if self.vaihe != Vaihe::Nimi {
            return;
        }
        let nimi = ota_nimi(&mut self.nimi);
        let pisteet = self.lauta.pisteet;
        let tulos = Tulos { nimi, pisteet };
        lisaa_tulos(&mut self.parhaat, tulos.clone());
        if let Some(lista) = &mut self.netti_parhaat {
            lisaa_tulos(lista, tulos.clone());
        }
        self.lahetys = Some(tulos);
        self.uusi_tulos = true;
        if std::mem::take(&mut self.uusi_alkaa) {
            self.aloita_alusta();
        } else {
            self.vaihe = Vaihe::Tappio;
        }
    }

    fn kasittele_nimi(&mut self, syote: Syote) {
        match syote {
            Syote::Ammu | Syote::Peru => self.vahvista_nimi(),
            _ => muokkaa_nimea(&mut self.nimi, syote),
        }
    }

    pub fn kasittele(&mut self, syote: Syote) {
        // Klikkaus jatkaa eteenpäin, jotta äänikytkin toimii nimeä kirjoittaessakin.
        if self.vaihe == Vaihe::Nimi && !matches!(syote, Syote::Klikkaus(..)) {
            self.kasittele_nimi(syote);
            return;
        }
        if syote == Syote::Peru {
            self.lopeta = true;
            return;
        }
        let kytkimella = matches!(syote, Syote::Klikkaus(x, y) if aanikytkimella(x, y));
        if syote == Syote::VaihdaAanet || kytkimella {
            self.aanet_paalla = !self.aanet_paalla;
            return;
        }
        match self.vaihe {
            Vaihe::Alku => {
                let uusi = syote == Syote::Kirjain('K')
                    || matches!(syote, Syote::Klikkaus(x, y) if piirto::uusi_peli_pisteessa(x, y));
                if self.jatkuu && uusi {
                    self.hylkaa_ja_aloita();
                } else if matches!(syote, Syote::Ammu | Syote::Klikkaus(..)) {
                    self.vaihe = Vaihe::Pelissa;
                    self.aanijono.push((Aani::Alku, 0));
                    if self.jatkuu {
                        self.jatkuu = false;
                        self.viesti = kaanna("luvut.jatketaan_tallennettua").to_string();
                    }
                }
            }
            Vaihe::Pelissa => self.kasittele_pelissa(syote),
            Vaihe::Voitto | Vaihe::Tappio => self.kasittele_loppu(syote),
            Vaihe::Nimi => {}
        }
    }

    fn kasittele_pelissa(&mut self, syote: Syote) {
        match syote {
            Syote::Nuoli(-1, _) => self.siirra(Suunta::Ylos),
            Syote::Nuoli(1, _) => self.siirra(Suunta::Alas),
            Syote::Nuoli(_, -1) => self.siirra(Suunta::Vasen),
            Syote::Nuoli(_, 1) => self.siirra(Suunta::Oikea),
            Syote::Hiiri(x, y) => self.hiiren_suunta = piirto::suunta_pisteesta(x, y),
            Syote::Klikkaus(x, y) => {
                if let Some(suunta) = piirto::suunta_pisteesta(x, y) {
                    self.siirra(suunta);
                }
            }
            _ => {}
        }
    }

    fn kasittele_loppu(&mut self, syote: Syote) {
        let nappi = match syote {
            Syote::Kirjain('K') => Nappi::Uudestaan,
            Syote::Kirjain('E') => Nappi::Valikko,
            Syote::Kirjain('J') => Nappi::Jatka,
            Syote::Klikkaus(x, y) => match piirto::nappi_pisteesta(x, y) {
                Some(nappi) => nappi,
                None => return,
            },
            _ => return,
        };
        match nappi {
            Nappi::Uudestaan if self.vaihe == Vaihe::Voitto => {
                // Voitto on jo juhlittu, jos nimen kirjoitus keskeytyy tallennukseen.
                self.jatkettu = true;
                self.hylkaa_ja_aloita();
            }
            Nappi::Uudestaan => self.aloita_alusta(),
            Nappi::Valikko => self.lopeta = true,
            Nappi::Jatka if self.vaihe == Vaihe::Voitto => {
                self.jatkettu = true;
                self.viesti = kaanna("luvut.jatketaan").to_string();
                if self.lauta.on_siirtoja() {
                    self.vaihe = Vaihe::Pelissa;
                } else {
                    self.paata_peli();
                }
            }
            Nappi::Jatka => {}
        }
    }

    pub fn siirra(&mut self, suunta: Suunta) {
        let Some(siirto) = self.lauta.siirra(suunta, &mut self.arpa) else {
            self.ei_liiku = true;
            self.aanijono.push((Aani::Virhe, 0));
            return;
        };
        self.ei_liiku = false;
        self.siirtoja += 1;
        self.paras = self.paras.max(self.lauta.pisteet);
        self.viesti = if siirto.pisteet > 0 {
            muotoile("luvut.siirto_pisteet", &[&self.siirtoja, &siirto.pisteet])
        } else {
            muotoile("luvut.siirto", &[&self.siirtoja])
        };
        self.aanijono.push((Aani::Liuku, 0));
        if siirto.pisteet > 0 {
            self.aanijono.push((Aani::Yhdistys, YHDISTYS_VIIVE_MS));
        }
        self.viime_siirto = Some((siirto, self.kello_ms));

        if !self.jatkettu && self.lauta.suurin() >= TAVOITE {
            self.vaihe = Vaihe::Voitto;
            self.aanijono.push((Aani::Voitto, LOPPU_VIIVE_MS));
            self.aanijono
                .push((Aani::Voitit, LOPPU_VIIVE_MS + PUHE_VIIVE_MS));
        } else if !self.lauta.on_siirtoja() {
            self.paata_peli();
            self.aanijono.push((Aani::Tappio, LOPPU_VIIVE_MS));
            self.aanijono
                .push((Aani::EiSiirtoja, LOPPU_VIIVE_MS + PUHE_VIIVE_MS));
        }
    }
}

impl Pelinakyma for Luvut {
    fn kasittele(&mut self, syote: Syote) {
        Luvut::kasittele(self, syote);
    }

    fn edista(&mut self, ms: u32) {
        Luvut::edista(self, ms);
    }

    fn piirra(&self, kehys: &mut Kehys) {
        piirto::piirra(self, &mut kehys.kuva);
        kehys.venyta();
    }

    fn aanijono(&mut self) -> &mut Vec<(Aani, u32)> {
        &mut self.aanijono
    }

    fn aanet_paalla(&self) -> bool {
        self.aanet_paalla
    }

    fn aseta_aanet(&mut self, paalla: bool) {
        self.aanet_paalla = paalla;
    }

    fn lopeta(&self) -> bool {
        self.lopeta
    }

    fn ota_uusi_tulos(&mut self) -> bool {
        std::mem::take(&mut self.uusi_tulos)
    }

    fn viimeistele(&mut self) {
        self.vahvista_nimi();
    }

    fn listat(&mut self) -> Vec<Lista<'_>> {
        vec![Lista {
            peli: Nettipeli::Luvut,
            paras: Some(&mut self.paras),
            parhaat: &mut self.parhaat,
            netti: &mut self.netti_parhaat,
            lahetys: Some(&mut self.lahetys),
        }]
    }

    fn nuolet_toistuvat(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pelissa(ruudut: [[u32; 4]; 4]) -> Luvut {
        let mut luvut = Luvut::new(1);
        luvut.kasittele(Syote::Ammu);
        assert_eq!(luvut.vaihe, Vaihe::Pelissa);
        luvut.lauta = Lukulauta::ruuduista(ruudut);
        luvut
    }

    const LAHES_VOITTO: [[u32; 4]; 4] = [[1024, 1024, 0, 0], [0; 4], [0; 4], [0; 4]];
    // Ainoa mahdollinen siirto täyttää laudan ilman pareja.
    const LAHES_LUKOSSA: [[u32; 4]; 4] = [
        [16, 8, 16, 0],
        [16, 8, 16, 8],
        [8, 16, 8, 16],
        [16, 8, 16, 8],
    ];
    // Vasemmalle tuottaa 4 pistettä ja lukitsee laudan, oli uusi luku 2 tai 4.
    const LUKKOON_PISTEILLA: [[u32; 4]; 4] = [
        [2, 2, 16, 8],
        [16, 32, 64, 16],
        [4, 16, 8, 32],
        [16, 32, 64, 128],
    ];

    #[test]
    fn tallennettu_peli_jatkuu_samasta_tilanteesta() {
        let mut luvut = pelissa([[2, 2, 0, 0], [0; 4], [0; 4], [0; 4]]);
        luvut.kasittele(Syote::Nuoli(0, -1));
        let kesken = luvut.kesken().unwrap();

        let mut uusi = Luvut::new(2);
        uusi.palauta(kesken);
        assert_eq!(uusi.vaihe, Vaihe::Alku);
        assert_eq!(uusi.paras, 4);
        uusi.kasittele(Syote::Ammu);
        assert_eq!(uusi.vaihe, Vaihe::Pelissa);
        assert_eq!(uusi.lauta, luvut.lauta);
        assert_eq!(uusi.siirtoja, 1);
        assert!(!uusi.jatkuu);
        assert!(uusi.viesti.contains("tallennettua"));
    }

    #[test]
    fn k_hylkaa_tallennetun_mutta_parhaan_ei_ja_ilman_tallennusta_k_ei_tee_mitaan() {
        let mut luvut = Luvut::new(1);
        let alku = luvut.lauta.clone();
        luvut.kasittele(Syote::Kirjain('K'));
        assert_eq!(luvut.lauta, alku);

        luvut.palauta(Kesken {
            ruudut: [[64, 0, 0, 0], [0; 4], [0; 4], [0; 4]],
            pisteet: 500,
            siirrot: 9,
            jatkettu: false,
        });
        assert!(luvut.kesken().is_some());
        luvut.kasittele(Syote::Kirjain('K'));
        assert_eq!(luvut.vaihe, Vaihe::Nimi);
        assert!(luvut.kesken().is_some());
        luvut.kasittele(Syote::Ammu);
        assert_eq!(luvut.parhaat[0].pisteet, 500);
        assert_eq!(luvut.vaihe, Vaihe::Alku);
        assert!(!luvut.jatkuu);
        assert_eq!(luvut.lauta.pisteet, 0);
        assert_eq!(luvut.siirtoja, 0);
        assert_eq!(luvut.paras, 500);
        assert_eq!(luvut.kesken(), None);
    }

    #[test]
    fn uuden_pelin_nappi_hylkaa_tallennetun_ja_muu_klikkaus_jatkaa() {
        let kesken = Kesken {
            ruudut: [[64, 0, 0, 0], [0; 4], [0; 4], [0; 4]],
            pisteet: 0,
            siirrot: 9,
            jatkettu: false,
        };
        let (x, y) = piirto::uuden_pelin_keskipiste();

        let mut luvut = Luvut::new(1);
        luvut.palauta(kesken);
        luvut.kasittele(Syote::Klikkaus(x, y));
        assert_eq!(luvut.vaihe, Vaihe::Alku);
        assert!(!luvut.jatkuu);
        assert_eq!(luvut.siirtoja, 0);
        // Ilman tallennusta nappia ei ole, joten sama klikkaus aloittaa pelin.
        luvut.kasittele(Syote::Klikkaus(x, y));
        assert_eq!(luvut.vaihe, Vaihe::Pelissa);

        let mut luvut = Luvut::new(1);
        luvut.palauta(kesken);
        luvut.kasittele(Syote::Klikkaus(0, 0));
        assert_eq!(luvut.vaihe, Vaihe::Pelissa);
        assert_eq!(luvut.siirtoja, 9);
    }

    #[test]
    fn voitto_tallentuu_jatkettuna_ja_tappio_ei_tallennu() {
        let mut luvut = pelissa(LAHES_VOITTO);
        luvut.kasittele(Syote::Nuoli(0, -1));
        assert_eq!(luvut.vaihe, Vaihe::Voitto);
        assert!(luvut.kesken().unwrap().jatkettu);

        let mut luvut = pelissa(LAHES_LUKOSSA);
        luvut.kasittele(Syote::Nuoli(0, 1));
        assert_eq!(luvut.vaihe, Vaihe::Tappio);
        assert_eq!(luvut.kesken(), None);
    }

    #[test]
    fn peli_alkaa_valilyonnilla_ja_nuoli_ei_siirra_alkuruudulla() {
        let mut luvut = Luvut::new(1);
        luvut.kasittele(Syote::Nuoli(0, -1));
        assert_eq!(luvut.vaihe, Vaihe::Alku);
        assert_eq!(luvut.siirtoja, 0);
        luvut.kasittele(Syote::Ammu);
        assert_eq!(luvut.vaihe, Vaihe::Pelissa);
    }

    #[test]
    fn nuolet_siirtavat_ja_pisteet_kertyvat_parhaaseen() {
        let mut luvut = pelissa([[2, 2, 0, 0], [0; 4], [0; 4], [0; 4]]);
        luvut.kasittele(Syote::Nuoli(0, -1));
        assert_eq!(luvut.lauta.arvo((0, 0)), 4);
        assert_eq!(luvut.lauta.pisteet, 4);
        assert_eq!(luvut.paras, 4);
        assert_eq!(luvut.siirtoja, 1);
        assert!(luvut.viesti.contains("+4"));
        assert!(
            luvut
                .aanijono
                .contains(&(Aani::Yhdistys, YHDISTYS_VIIVE_MS))
        );
    }

    #[test]
    fn liikkumaton_siirto_ahkaisee_eika_kuluta_vuoroa() {
        let mut luvut = pelissa([[2, 0, 0, 0], [0; 4], [0; 4], [0; 4]]);
        luvut.aanijono.clear();
        luvut.kasittele(Syote::Nuoli(0, -1));
        assert!(luvut.ei_liiku);
        assert_eq!(luvut.siirtoja, 0);
        assert_eq!(luvut.aanijono, [(Aani::Virhe, 0)]);
    }

    #[test]
    fn voitto_tulee_kerran_ja_jatkaminen_toimii() {
        let mut luvut = pelissa(LAHES_VOITTO);
        luvut.kasittele(Syote::Nuoli(0, -1));
        assert_eq!(luvut.vaihe, Vaihe::Voitto);
        luvut.kasittele(Syote::Kirjain('J'));
        assert_eq!(luvut.vaihe, Vaihe::Pelissa);

        // jo yli tavoitteen: seuraava siirto ei voita uudelleen
        luvut.kasittele(Syote::Nuoli(1, 0));
        assert_eq!(luvut.vaihe, Vaihe::Pelissa);
    }

    #[test]
    fn lukkoon_ajautuminen_on_tappio_eika_tappiosta_voi_jatkaa() {
        let mut luvut = pelissa(LAHES_LUKOSSA);
        // uusi luku voi olla 2 tai 4, ei kumpikaan yhdisty naapureihinsa
        luvut.kasittele(Syote::Nuoli(0, 1));
        assert_eq!(luvut.vaihe, Vaihe::Tappio);
        luvut.kasittele(Syote::Kirjain('J'));
        assert_eq!(luvut.vaihe, Vaihe::Tappio);
    }

    #[test]
    fn uudestaan_nollaa_pelin_mutta_parhaan_ei() {
        let mut luvut = pelissa(LAHES_VOITTO);
        luvut.kasittele(Syote::Nuoli(0, -1));
        let paras = luvut.paras;
        assert!(paras >= 2048);
        luvut.kasittele(Syote::Kirjain('K'));
        assert_eq!(luvut.vaihe, Vaihe::Nimi);
        luvut.kasittele(Syote::Ammu);
        assert_eq!(luvut.vaihe, Vaihe::Alku);
        assert_eq!(luvut.lauta.pisteet, 0);
        assert_eq!(luvut.siirtoja, 0);
        assert_eq!(luvut.paras, paras);
    }

    #[test]
    fn tappio_kysyy_nimen_ja_kirjaa_tuloksen_kerran() {
        let mut luvut = pelissa(LUKKOON_PISTEILLA);
        luvut.kasittele(Syote::Nuoli(0, -1));
        assert_eq!(luvut.vaihe, Vaihe::Nimi);
        assert_eq!(luvut.kesken(), None);

        // M on kirjain eikä äänikytkin, ja nimen pituudella on raja
        luvut.kasittele(Syote::VaihdaAanet);
        assert!(luvut.aanet_paalla);
        for kirjain in "ÄXXXXXXXXX".chars() {
            luvut.kasittele(Syote::Kirjain(kirjain));
        }
        assert_eq!(luvut.nimi, "MÄXXXXXX");
        luvut.kasittele(Syote::Pyyhi);
        luvut.kasittele(Syote::Numero(7));
        luvut.kasittele(Syote::Ammu);

        assert_eq!(luvut.vaihe, Vaihe::Tappio);
        assert!(luvut.uusi_tulos);
        assert_eq!(luvut.parhaat.len(), 1);
        assert_eq!(luvut.parhaat[0].nimi, "MÄXXXXX7");
        assert_eq!(luvut.parhaat[0].pisteet, 4);

        // tappion jälkeinen uusi peli ei kirjaa samaa tulosta toiseen kertaan
        luvut.kasittele(Syote::Kirjain('K'));
        assert_eq!(luvut.vaihe, Vaihe::Alku);
        assert_eq!(luvut.parhaat.len(), 1);
    }

    #[test]
    fn nettilista_ratkaisee_nimen_kysymisen_kun_se_on_tiedossa() {
        let taysi = |pisteet| {
            (0..3)
                .map(|_| Tulos {
                    nimi: "A".to_string(),
                    pisteet,
                })
                .collect::<Vec<_>>()
        };

        let mut luvut = pelissa(LUKKOON_PISTEILLA);
        luvut.netti_parhaat = Some(taysi(1000));
        luvut.kasittele(Syote::Nuoli(0, -1));
        assert_eq!(luvut.vaihe, Vaihe::Tappio);
        assert_eq!(luvut.lahetys, None);

        let mut luvut = pelissa(LUKKOON_PISTEILLA);
        luvut.parhaat = taysi(1000);
        luvut.netti_parhaat = Some(taysi(2));
        luvut.kasittele(Syote::Nuoli(0, -1));
        assert_eq!(luvut.vaihe, Vaihe::Nimi);
        luvut.kasittele(Syote::Kirjain('B'));
        luvut.kasittele(Syote::Ammu);
        assert_eq!(luvut.netti_parhaat.as_ref().unwrap()[0].nimi, "B");
        assert_eq!(luvut.lahetys.as_ref().unwrap().pisteet, 4);
    }

    #[test]
    fn ilman_verkkoa_tulos_kirjataan_omalle_listalle_ja_lahetettavaksi() {
        let mut luvut = pelissa(LUKKOON_PISTEILLA);
        luvut.kasittele(Syote::Nuoli(0, -1));
        luvut.kasittele(Syote::Kirjain('B'));
        luvut.kasittele(Syote::Ammu);
        assert_eq!(luvut.nakyva_lista()[0].nimi, "B");
        assert_eq!(luvut.netti_parhaat, None);
        assert_eq!(luvut.lahetys.as_ref().unwrap().nimi, "B");
    }

    #[test]
    fn listalle_yltamaton_tappio_ei_kysy_nimea() {
        let mut luvut = pelissa(LUKKOON_PISTEILLA);
        luvut.parhaat = (0..3)
            .map(|_| Tulos {
                nimi: "A".to_string(),
                pisteet: 4,
            })
            .collect();
        luvut.kasittele(Syote::Nuoli(0, -1));
        assert_eq!(luvut.vaihe, Vaihe::Tappio);
    }

    #[test]
    fn esc_ja_ikkunan_sulkeminen_kirjaavat_tuloksen_oletusnimella() {
        let mut luvut = pelissa(LUKKOON_PISTEILLA);
        luvut.kasittele(Syote::Nuoli(0, -1));
        luvut.kasittele(Syote::Peru);
        assert!(!luvut.lopeta);
        assert_eq!(luvut.parhaat[0].nimi, "NIMETÖN");

        let mut luvut = pelissa(LUKKOON_PISTEILLA);
        luvut.kasittele(Syote::Nuoli(0, -1));
        luvut.kasittele(Syote::Kirjain('A'));
        luvut.vahvista_nimi();
        assert_eq!(luvut.parhaat[0].nimi, "A");
        assert_eq!(luvut.vaihe, Vaihe::Tappio);
    }

    #[test]
    fn napit_toimivat_hiirella() {
        let mut luvut = pelissa(LAHES_VOITTO);
        luvut.kasittele(Syote::Nuoli(0, -1));
        let (x, y) = piirto::napin_keskipiste(Nappi::Jatka);
        luvut.kasittele(Syote::Klikkaus(x, y));
        assert_eq!(luvut.vaihe, Vaihe::Pelissa);

        let mut luvut = pelissa(LAHES_VOITTO);
        luvut.kasittele(Syote::Nuoli(0, -1));
        let (x, y) = piirto::napin_keskipiste(Nappi::Valikko);
        luvut.kasittele(Syote::Klikkaus(x, y));
        assert!(luvut.lopeta);
    }

    #[test]
    fn hiiri_nayttaa_suunnan_ja_klikkaus_siirtaa_sinne() {
        let mut luvut = pelissa([[0, 0, 0, 0], [0, 8, 0, 0], [0; 4], [0; 4]]);
        let (x, y) = piirto::laudan_reuna(Suunta::Oikea);
        luvut.kasittele(Syote::Hiiri(x, y));
        assert_eq!(luvut.hiiren_suunta, Some(Suunta::Oikea));
        luvut.kasittele(Syote::Klikkaus(x, y));
        assert_eq!(luvut.lauta.arvo((1, 3)), 8);

        luvut.kasittele(Syote::Klikkaus(0, 0));
        assert_eq!(luvut.siirtoja, 1);
    }

    #[test]
    fn animaatio_paattyy_ja_esc_pyytaa_paluuta() {
        let mut luvut = pelissa([[2, 2, 0, 0], [0; 4], [0; 4], [0; 4]]);
        luvut.kasittele(Syote::Nuoli(0, -1));
        assert!(luvut.viime_siirto.is_some());
        luvut.edista(LIUKU_MS + KIPINA_MS);
        assert!(luvut.viime_siirto.is_none());
        luvut.kasittele(Syote::Peru);
        assert!(luvut.lopeta);
    }
}
