// Pelin suurin tulos. 2048:n teoreettinen huippu on 3 932 100 pistettä. Tiilirikko, Asteroidit,
// Silmukka, Palikka ja Blackjack rajaavat tuloksen itse lukuun 999 999. Pasianssien huippu on heti
// voitettu vaikea peli, ja Hold'emissa voi voittaa vain neljän koneen rahat. Haasteissa tulos on
// 60 haasteen summa, joten sama pelaaja nostaa omaa tulostaan: `YKSI_PER_NIMI` pitää listan
// tasapuolisena.
export const PELIT = {
  luvut: 4000000,
  tiilirikko: 1000000,
  asteroidit: 1000000,
  silmukka: 1000000,
  palikka: 1000000,
  pasianssi: 10800,
  spider: 10800,
  pyramidi: 10800,
  haasteet: 648000,
  blackjack: 1000000,
  holdem: 4000,
};

export const YKSI_PER_NIMI = new Set(["haasteet"]);

const RIVI = /^([1-9][0-9]{0,8}) ([A-ZÄÖ0-9]{1,8})$/u;

export function lue_tulos(teksti, suurin) {
  const osat = RIVI.exec(teksti.trim());
  if (!osat) {
    return null;
  }
  const pisteet = Number(osat[1]);
  return pisteet <= suurin ? { pisteet, nimi: osat[2] } : null;
}
