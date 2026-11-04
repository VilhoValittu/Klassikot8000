//! Käynnistimen ja pelin välinen sopimus. Käynnistin hoitaa jokaista peliä tämän kautta,
//! joten uusi peli tarvitsee käynnistimeen vain korttinsa ja rivin `pelit!`-luetteloon.

use super::aanet::Aani;
use super::piirto::Kehys;
use super::syote::Syote;
use super::tulokset::{Nettipeli, Tulos};
use crate::kaynnistin::tallennus::{Asetukset, Tallennus};

/// Pelin yksi top-lista käynnistimen silmin: käynnistin täyttää sen tallennuksesta ja
/// nettilistasta ja kirjaa siitä tallennukseen ja jonoon.
pub struct Lista<'a> {
    pub peli: Nettipeli,
    /// Pelin oma ennätys, jos se pidetään listasta erillään.
    pub paras: Option<&'a mut u32>,
    pub parhaat: &'a mut Vec<Tulos>,
    pub netti: &'a mut Option<Vec<Tulos>>,
    /// Lähtevä tulos. Vain listalla, jolle peli juuri nyt kirjaa tuloksia.
    pub lahetys: Option<&'a mut Option<Tulos>>,
}

pub trait Pelinakyma {
    fn kasittele(&mut self, syote: Syote);
    fn edista(&mut self, ms: u32);
    fn piirra(&self, kehys: &mut Kehys);
    fn aanijono(&mut self) -> &mut Vec<(Aani, u32)>;
    fn aanet_paalla(&self) -> bool;
    fn aseta_aanet(&mut self, paalla: bool);
    /// Peli on valmis palaamaan valikkoon.
    fn lopeta(&self) -> bool;

    /// Tosi kerran, kun syntyi jotain tallennettavaa.
    fn ota_uusi_tulos(&mut self) -> bool {
        false
    }

    /// Ikkuna suljetaan kesken pelin: nimeä odottava tulos kirjataan, ettei se katoa.
    fn viimeistele(&mut self) {}

    fn listat(&mut self) -> Vec<Lista<'_>> {
        Vec::new()
    }

    /// Uusi peli ottaa tallennuksesta asetuksensa ja omat tietonsa. Top-listat käynnistin
    /// täyttää itse `listat`-kutsun kautta.
    fn lue_tallennuksesta(&mut self, _tallennus: &Tallennus) {}

    /// Pelin sisällä vaihdettu taso tai laji on seuraavallakin kerralla valmiina. Kutsutaan
    /// jokaisen syötteen jälkeen.
    fn kirjaa_asetukset(&self, _asetukset: &mut Asetukset) {}

    /// Pelin omat tallennettavat tiedot, kuten tilastot tai kesken jäänyt peli.
    fn kirjaa_tallennukseen(&self, _tallennus: &mut Tallennus) {}

    /// Kaikki, mistä paikallaan pysyvä kuva riippuu syötteen lisäksi, tai `None`, kun kuva
    /// elää joka ruudussa.
    fn kuvan_avain(&self) -> Option<(u32, u32)> {
        None
    }

    /// Reaaliaikainen peli tarvitsee tiheämmän päivityksen kuin vuoropelit.
    fn toimintapeli(&self) -> bool {
        false
    }

    /// Pohjassa olevat suunnat: -1 vasen, 1 oikea, ja ylös.
    fn aseta_ohjaus(&mut self, _suunta: i8, _ylos: bool) {}

    /// Ikkuna ei ole aktiivinen.
    fn keskeyta(&mut self) {}

    /// Hiiri ohjaa peliä myös ikkunan ulkopuolelta, kuvan reunaan rajattuna. Nopea veto vie
    /// kursorin helposti ikkunan reunan yli, eikä ohjaus saa silloin jäädä paikalleen.
    fn hiiri_ulkonakin(&self) -> bool {
        false
    }

    /// Pohjassa pidetty nuoli toistuu. Siirtopeleissä toisto tekisi siitä siirtosarjan.
    fn nuolet_toistuvat(&self) -> bool {
        true
    }

    /// Näkymä piirtyy koko ruudun kokoisena eikä klassisen kuvan kautta.
    fn suoraan_ruudulle(&self) -> bool {
        false
    }
}
