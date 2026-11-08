// Pelin suurin tulos. 2048:n teoreettinen huippu on 3 932 100 pistettä. Tiilirikko, Asteroidit,
// Silmukka, Palikka, Blackjack, Majakka ja Kaivo rajaavat tuloksen itse lukuun 999 999. Pasianssien huippu on heti
// voitettu vaikea peli, ja Hold'emissa voi voittaa vain neljän koneen rahat. Haasteissa tulos on
// 60 haasteen summa, joten sama pelaaja nostaa omaa tulostaan: `YKSI_PER_NIMI` pitää listan
// tasapuolisena. Miinaharavan huippu on sekunnissa voitettu vaikea kenttä (999 * 3).
// Laivanupotuksen huippu on vaikean tason voitto 17 laukauksella ((101 - 17) * 3). Mahjonggin
// raja ylittää laskennallisen huipun: jokainen pari täydellä ketjulla ja voitto heti (25 800).
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
  miinaharava: 3000,
  laivanupotus: 252,
  mahjong: 30000,
  majakka: 1000000,
  kaivo: 1000000,
};

export const YKSI_PER_NIMI = new Set(["haasteet"]);

const RIVI = /^([1-9][0-9]{0,8}) ([A-ZÄÖ0-9]{1,8})$/u;
// Kelvollinen rivi on enintään 27 tavua. Pidempää pyyntöä ei kannata lukea eikä tutkia.
export const SUURIN_RUNKO = 64;

// Lukee pyynnön rungon, mutta enintään SUURIN_RUNKO tavua. Paloina lähetetyssä pyynnössä ei ole
// content-lengthiä, joten raja tarkistetaan lukiessa. Liian pitkä runko antaa null.
export async function lue_runko(virta) {
  if (!virta) {
    return "";
  }
  const lukija = virta.getReader();
  const palat = [];
  let koko = 0;
  for (;;) {
    const { done, value } = await lukija.read();
    if (done) {
      return new Blob(palat).text();
    }
    koko += value.byteLength;
    if (koko > SUURIN_RUNKO) {
      await lukija.cancel();
      return null;
    }
    palat.push(value);
  }
}

export function lue_tulos(teksti, suurin) {
  if (teksti.length > SUURIN_RUNKO) {
    return null;
  }
  const osat = RIVI.exec(teksti.trim());
  if (!osat) {
    return null;
  }
  const pisteet = Number(osat[1]);
  return pisteet <= suurin ? { pisteet, nimi: osat[2] } : null;
}

const HEKSAT = /^[0-9a-f]{64}$/;

function tavuiksi(heksat) {
  return Uint8Array.from(heksat.match(/../g), (pari) => parseInt(pari, 16));
}

// Peli allekirjoittaa pelin tunnuksen ja rungon julkaisijan avaimella (HMAC-SHA256). Vertailu
// tehdään verify-kutsulla, joka ei paljasta ajoituksellaan, montako merkkiä täsmäsi.
export async function allekirjoitus_kelpaa(avain, peli, runko, allekirjoitus) {
  avain = avain.trim().toLowerCase();
  allekirjoitus = (allekirjoitus ?? "").trim().toLowerCase();
  if (!HEKSAT.test(avain) || !HEKSAT.test(allekirjoitus)) {
    return false;
  }
  const hmac = { name: "HMAC", hash: "SHA-256" };
  const avaimena = await crypto.subtle.importKey("raw", tavuiksi(avain), hmac, false, ["verify"]);
  const viesti = new TextEncoder().encode(`${peli}\n${runko}`);
  return crypto.subtle.verify("HMAC", avaimena, tavuiksi(allekirjoitus), viesti);
}
