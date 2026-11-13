// Pelien nettilista. Vastaukset ovat tekstiä, rivi muodossa "pisteet NIMI", jotta peli
// ei tarvitse JSON-kirjastoa. Parametrilla aika=1 rivin perään tulee tuloksen hetki
// Unix-sekunteina; vanhat versiot eivät osaa lukea sitä, joten ne saavat vanhan muodon.

import {
  allekirjoitus_kelpaa,
  lue_runko,
  lue_tulos,
  PELIT,
  SUURIN_RUNKO,
  YKSI_PER_NIMI,
} from "./tarkistus.js";

const NAYTETTAVIA = 3;
// Poistettu huijaustulos paljastaa alemmat, joten tietokantaan jää enemmän kuin näytetään.
const SAILYTETTAVIA = 100;

const LISTA = `SELECT pisteet, nimi, aika FROM tulokset WHERE peli = ?1
               ORDER BY pisteet DESC, aika ASC, rowid ASC LIMIT ${NAYTETTAVIA}`;
// Sama nimi ja pisteet tallentuvat kerran, joten pelin uudelleenlähetys ei tee kaksoiskappaletta.
const LISAYS = `INSERT OR IGNORE INTO tulokset (peli, nimi, pisteet, aika) VALUES (?1, ?2, ?3, ?4)`;
const NIMEN_VANHAT = `DELETE FROM tulokset WHERE peli = ?1 AND nimi = ?2 AND pisteet < (
                        SELECT MAX(pisteet) FROM tulokset WHERE peli = ?1 AND nimi = ?2)`;
const KARSINTA = `DELETE FROM tulokset WHERE peli = ?1 AND rowid NOT IN (
                    SELECT rowid FROM tulokset WHERE peli = ?1
                    ORDER BY pisteet DESC, aika ASC, rowid ASC LIMIT ${SAILYTETTAVIA})`;

function vastaus(teksti, status = 200) {
  return new Response(teksti, {
    status,
    headers: {
      "content-type": "text/plain; charset=utf-8",
      "cache-control": "no-store",
    },
  });
}

function rivi(r, ajat) {
  return ajat ? `${r.pisteet} ${r.nimi} ${Math.floor(r.aika / 1000)}\n` : `${r.pisteet} ${r.nimi}\n`;
}

async function lista(env, peli, ajat) {
  const { results } = await env.DB.prepare(LISTA).bind(peli).all();
  return vastaus(results.map((r) => rivi(r, ajat)).join(""));
}

async function lisaa(pyynto, env, peli, ajat) {
  // Ilman avainta mikään tulos ei kelpaisi. 503 jättää tuloksen pelin jonoon, joten se lähtee,
  // kun salaisuus on asetettu (wrangler secret put TULOSAVAIN).
  if (!env.TULOSAVAIN) {
    return vastaus("tulosavain puuttuu palvelimelta\n", 503);
  }
  const osoite = pyynto.headers.get("cf-connecting-ip") ?? "tuntematon";
  const { success } = await env.RAJOITIN.limit({ key: osoite });
  if (!success) {
    return vastaus("liian monta tulosta\n", 429);
  }
  // Peli hylkää tuloksen 400:lla eikä yritä uudelleen, joten liian pitkä runko saa saman.
  // Otsake hylkää liian pitkän heti, ja lue_runko myös ne, joista otsake puuttuu.
  if (Number(pyynto.headers.get("content-length") ?? 0) > SUURIN_RUNKO) {
    return vastaus("kelvoton tulos\n", 400);
  }
  const runko = await lue_runko(pyynto.body);
  if (runko === null) {
    return vastaus("kelvoton tulos\n", 400);
  }
  const allekirjoitus = pyynto.headers.get("x-allekirjoitus");
  if (!(await allekirjoitus_kelpaa(env.TULOSAVAIN, peli, runko, allekirjoitus))) {
    return vastaus("kelvoton allekirjoitus\n", 400);
  }
  const tulos = lue_tulos(runko, PELIT[peli]);
  if (!tulos) {
    return vastaus("kelvoton tulos\n", 400);
  }
  const lausekkeet = [env.DB.prepare(LISAYS).bind(peli, tulos.nimi, tulos.pisteet, Date.now())];
  if (YKSI_PER_NIMI.has(peli)) {
    lausekkeet.push(env.DB.prepare(NIMEN_VANHAT).bind(peli, tulos.nimi));
  }
  lausekkeet.push(env.DB.prepare(KARSINTA).bind(peli));
  await env.DB.batch(lausekkeet);
  return lista(env, peli, ajat);
}

export default {
  async fetch(pyynto, env) {
    const url = new URL(pyynto.url);
    const peli = url.searchParams.get("peli");
    const ajat = url.searchParams.get("aika") === "1";
    if (!Object.hasOwn(PELIT, peli ?? "")) {
      return vastaus("tuntematon peli\n", 404);
    }
    if (pyynto.method === "GET" && url.pathname === "/top3") {
      return lista(env, peli, ajat);
    }
    if (pyynto.method === "POST" && url.pathname === "/tulos") {
      return lisaa(pyynto, env, peli, ajat);
    }
    return vastaus("ei löydy\n", 404);
  },
};
