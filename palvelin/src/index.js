// Pelien nettilista. Vastaukset ovat tekstiä, rivi muodossa "pisteet NIMI", jotta peli
// ei tarvitse JSON-kirjastoa.

import { lue_tulos, PELIT, YKSI_PER_NIMI } from "./tarkistus.js";

const NAYTETTAVIA = 3;
// Poistettu huijaustulos paljastaa alemmat, joten tietokantaan jää enemmän kuin näytetään.
const SAILYTETTAVIA = 100;

const LISTA = `SELECT pisteet, nimi FROM tulokset WHERE peli = ?1
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

async function lista(env, peli) {
  const { results } = await env.DB.prepare(LISTA).bind(peli).all();
  return vastaus(results.map((r) => `${r.pisteet} ${r.nimi}\n`).join(""));
}

async function lisaa(pyynto, env, peli) {
  const osoite = pyynto.headers.get("cf-connecting-ip") ?? "tuntematon";
  const { success } = await env.RAJOITIN.limit({ key: osoite });
  if (!success) {
    return vastaus("liian monta tulosta\n", 429);
  }
  const tulos = lue_tulos(await pyynto.text(), PELIT[peli]);
  if (!tulos) {
    return vastaus("kelvoton tulos\n", 400);
  }
  const lausekkeet = [env.DB.prepare(LISAYS).bind(peli, tulos.nimi, tulos.pisteet, Date.now())];
  if (YKSI_PER_NIMI.has(peli)) {
    lausekkeet.push(env.DB.prepare(NIMEN_VANHAT).bind(peli, tulos.nimi));
  }
  lausekkeet.push(env.DB.prepare(KARSINTA).bind(peli));
  await env.DB.batch(lausekkeet);
  return lista(env, peli);
}

export default {
  async fetch(pyynto, env) {
    const url = new URL(pyynto.url);
    const peli = url.searchParams.get("peli");
    if (!Object.hasOwn(PELIT, peli ?? "")) {
      return vastaus("tuntematon peli\n", 404);
    }
    if (pyynto.method === "GET" && url.pathname === "/top3") {
      return lista(env, peli);
    }
    if (pyynto.method === "POST" && url.pathname === "/tulos") {
      return lisaa(pyynto, env, peli);
    }
    return vastaus("ei löydy\n", 404);
  },
};
