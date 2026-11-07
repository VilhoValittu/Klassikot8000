//! Käynnistimen ja pelin välinen sopimus. Käynnistin hoitaa jokaista peliä tämän kautta,
//! joten uusi peli tarvitsee käynnistimeen vain rivin `pelit!`-luetteloon.

use super::aanet::Aanet;
use super::piirto::Kehys;
use super::syote::Syote;
use super::tulokset::Tulostaulu;
use crate::kaynnistin::tallennus::{Asetukset, Tallennus};

pub trait Pelinakyma {
    fn kasittele(&mut self, syote: Syote);
    fn edista(&mut self, ms: u32);
    fn piirra(&self, kehys: &mut Kehys);
    /// Pelin äänikytkin ja soittojono. Käynnistin asettaa kytkimen pelin alkaessa ja lukee
    /// sen jokaisen syötteen jälkeen, koska peli vaihtaa sitä itse M-näppäimestä.
    fn aanet(&mut self) -> &mut Aanet;
    /// Peli on valmis palaamaan valikkoon.
    fn lopeta(&self) -> bool;

    /// Tosi kerran, kun syntyi jotain tallennettavaa.
    fn ota_uusi_tulos(&mut self) -> bool {
        false
    }

    /// Ikkuna suljetaan kesken pelin: nimeä odottava tulos kirjataan, ettei se katoa.
    fn viimeistele(&mut self) {}

    /// Pelin top-listat. Käynnistin lähettää jokaisen taulun `lahetys`-tuloksen.
    fn listat(&mut self) -> Vec<&mut Tulostaulu> {
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

    /// Pohjassa olevat nuolet tai WASD ilman hiirtä: vaaka -1 vasen, pysty -1 ylös.
    fn aseta_nuolet(&mut self, _vaaka: i8, _pysty: i8) {}

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
