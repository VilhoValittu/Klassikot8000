// Reittien ja tietokantalausekkeiden testit. D1:n tilalla on Noden oma SQLite muistissa, ja
// taulu luodaan samasta schema.sql:stä kuin oikea tietokanta.

import assert from "node:assert/strict";
import { createHmac } from "node:crypto";
import { readFileSync } from "node:fs";
import { DatabaseSync } from "node:sqlite";
import { test } from "node:test";

import palvelin from "../src/index.js";
import { SUURIN_RUNKO } from "../src/tarkistus.js";

const AVAIN = "ab".repeat(32);
const SCHEMA = readFileSync(new URL("../schema.sql", import.meta.url), "utf8");

// D1:n rajapinnasta se osa, jota palvelin käyttää: prepare, bind, all ja batch.
function tietokanta() {
  const sqlite = new DatabaseSync(":memory:");
  sqlite.exec(SCHEMA);
  const lauseke = (sql, arvot = []) => ({
    bind: (...uudet) => lauseke(sql, uudet),
    all: async () => ({ results: sqlite.prepare(sql).all(...arvot) }),
    aja: () => sqlite.prepare(sql).run(...arvot),
  });
  return {
    sqlite,
    prepare: (sql) => lauseke(sql),
    // D1 ajaa erän yhtenä tapahtumana.
    batch: async (lausekkeet) => {
      sqlite.exec("BEGIN");
      try {
        lausekkeet.forEach((l) => l.aja());
        sqlite.exec("COMMIT");
      } catch (virhe) {
        sqlite.exec("ROLLBACK");
        throw virhe;
      }
    },
  };
}

function ymparisto({ avain = AVAIN, sallittuja = Infinity } = {}) {
  const DB = tietokanta();
  const avaimet = [];
  return {
    DB,
    TULOSAVAIN: avain,
    avaimet,
    RAJOITIN: {
      limit: async ({ key }) => {
        avaimet.push(key);
        return { success: avaimet.length <= sallittuja };
      },
    },
  };
}

function allekirjoita(peli, runko, avain = AVAIN) {
  return createHmac("sha256", Buffer.from(avain, "hex")).update(`${peli}\n${runko}`).digest("hex");
}

async function hae(env, polku, valinnat) {
  const vastaus = await palvelin.fetch(new Request(`https://palvelin.test${polku}`, valinnat), env);
  return { status: vastaus.status, teksti: await vastaus.text() };
}

function laheta(env, peli, runko, otsakkeet = {}) {
  return hae(env, `/tulos?peli=${peli}`, {
    method: "POST",
    body: runko,
    headers: {
      "x-allekirjoitus": allekirjoita(peli, runko),
      "cf-connecting-ip": "192.0.2.1",
      ...otsakkeet,
    },
  });
}

function rivit(env, peli) {
  return env.DB.sqlite
    .prepare("SELECT pisteet, nimi FROM tulokset WHERE peli = ? ORDER BY pisteet DESC")
    .all(peli)
    .map((r) => `${r.pisteet} ${r.nimi}`);
}

test("tyhjä lista on tyhjä vastaus", async () => {
  const vastaus = await hae(ymparisto(), "/top3?peli=luvut");
  assert.deepEqual(vastaus, { status: 200, teksti: "" });
});

test("tuntematon peli, polku ja metodi antavat 404", async () => {
  const env = ymparisto();
  for (const [polku, method] of [
    ["/top3", "GET"],
    ["/top3?peli=tetris", "GET"],
    ["/top3?peli=toString", "GET"],
    ["/muu?peli=luvut", "GET"],
    ["/tulos?peli=luvut", "GET"],
    ["/top3?peli=luvut", "POST"],
    ["/tulos?peli=luvut", "DELETE"],
  ]) {
    assert.equal((await hae(env, polku, { method })).status, 404, `${method} ${polku}`);
  }
  assert.deepEqual(env.avaimet, [], "rajoitinta ei kuluteta kelvottomiin reitteihin");
});

test("lähetetty tulos tallentuu ja vastaus on pelin oma top 3", async () => {
  const env = ymparisto();
  await laheta(env, "tiilirikko", "999 MUU");
  for (const runko of ["50 A", "70 B", "60 C"]) {
    assert.equal((await laheta(env, "luvut", runko)).status, 200);
  }
  const vastaus = await laheta(env, "luvut", "65 VÄINÖ");
  assert.deepEqual(vastaus, { status: 200, teksti: "70 B\n65 VÄINÖ\n60 C\n" });
  assert.equal((await hae(env, "/top3?peli=luvut")).teksti, "70 B\n65 VÄINÖ\n60 C\n");
  assert.equal((await hae(env, "/top3?peli=tiilirikko")).teksti, "999 MUU\n");
  // Listalta pudonnut säilyy tietokannassa.
  assert.deepEqual(rivit(env, "luvut"), ["70 B", "65 VÄINÖ", "60 C", "50 A"]);
});

test("tasapisteissä vanhempi tulos pysyy edellä", async (t) => {
  t.mock.timers.enable({ apis: ["Date"], now: 1000 });
  const env = ymparisto();
  for (const nimi of ["EKA", "TOKA", "KOLMAS", "NELJÄS"]) {
    await laheta(env, "luvut", `100 ${nimi}`);
    t.mock.timers.tick(1000);
  }
  assert.equal((await hae(env, "/top3?peli=luvut")).teksti, "100 EKA\n100 TOKA\n100 KOLMAS\n");
});

test("aika=1 lisää riveille tuloksen hetken sekunteina", async (t) => {
  t.mock.timers.enable({ apis: ["Date"], now: 1_763_000_000_999 });
  const env = ymparisto();
  await laheta(env, "luvut", "100 EKA");
  const vastaus = await hae(env, "/tulos?peli=luvut&aika=1", {
    method: "POST",
    body: "50 TOKA",
    headers: { "x-allekirjoitus": allekirjoita("luvut", "50 TOKA") },
  });
  const odotettu = "100 EKA 1763000000\n50 TOKA 1763000000\n";
  assert.deepEqual(vastaus, { status: 200, teksti: odotettu });
  assert.equal((await hae(env, "/top3?peli=luvut&aika=1")).teksti, odotettu);
  // Vanhat versiot eivät pyydä aikoja eivätkä osaisi lukea niitä.
  assert.equal((await hae(env, "/top3?peli=luvut")).teksti, "100 EKA\n50 TOKA\n");
});

test("sama tulos uudelleen lähetettynä ei tee kaksoiskappaletta", async () => {
  const env = ymparisto();
  await laheta(env, "luvut", "100 A");
  const vastaus = await laheta(env, "luvut", "100 A");
  assert.deepEqual(vastaus, { status: 200, teksti: "100 A\n" });
  assert.deepEqual(rivit(env, "luvut"), ["100 A"]);
});

test("haasteissa nimellä on yksi rivi, joka vain kasvaa", async () => {
  const env = ymparisto();
  await laheta(env, "haasteet", "100 A");
  await laheta(env, "haasteet", "300 A");
  await laheta(env, "haasteet", "200 A");
  await laheta(env, "haasteet", "150 B");
  assert.deepEqual(rivit(env, "haasteet"), ["300 A", "150 B"]);
  // Muissa peleissä saman nimen kaikki tulokset säilyvät.
  await laheta(env, "luvut", "100 A");
  await laheta(env, "luvut", "300 A");
  assert.deepEqual(rivit(env, "luvut"), ["300 A", "100 A"]);
});

test("tietokantaan jää pelin sata parasta eikä karsinta koske muita pelejä", async () => {
  const env = ymparisto();
  await laheta(env, "tiilirikko", "1 PIENI");
  const lisays = env.DB.sqlite.prepare(
    "INSERT INTO tulokset (peli, nimi, pisteet, aika) VALUES ('luvut', ?, ?, 0)",
  );
  for (let pisteet = 1; pisteet <= 100; pisteet++) {
    lisays.run(`N${pisteet}`, pisteet);
  }
  // Sadan huonoimman alle jäävä tulos karsitaan heti, parempi pudottaa huonoimman.
  await laheta(env, "luvut", "1 HUONO");
  assert.equal(rivit(env, "luvut").length, 100);
  assert.ok(!rivit(env, "luvut").includes("1 HUONO"));
  await laheta(env, "luvut", "500 HYVÄ");
  const jaljella = rivit(env, "luvut");
  assert.equal(jaljella.length, 100);
  assert.equal(jaljella[0], "500 HYVÄ");
  assert.equal(jaljella.at(-1), "2 N2");
  assert.deepEqual(rivit(env, "tiilirikko"), ["1 PIENI"]);
});

test("väärä tai puuttuva allekirjoitus hylätään 400:lla", async () => {
  const env = ymparisto();
  const runko = "100 A";
  for (const allekirjoitus of [
    "",
    "00".repeat(32),
    allekirjoita("luvut", "101 A"),
    // Toisen pelin listalle tehty allekirjoitus ei kelpaa tälle.
    allekirjoita("tiilirikko", runko),
    allekirjoita("luvut", runko, "cd".repeat(32)),
  ]) {
    const vastaus = await laheta(env, "luvut", runko, { "x-allekirjoitus": allekirjoitus });
    assert.deepEqual(vastaus, { status: 400, teksti: "kelvoton allekirjoitus\n" });
  }
  assert.deepEqual(rivit(env, "luvut"), []);
});

test("oikein allekirjoitettu kelvoton tulos hylätään 400:lla", async () => {
  const env = ymparisto();
  for (const runko of ["0 A", "100 pieni", "100 LIIANPITKA", "253 A", "100"]) {
    const vastaus = await laheta(env, "laivanupotus", runko);
    assert.deepEqual(vastaus, { status: 400, teksti: "kelvoton tulos\n" }, runko);
  }
  assert.equal((await laheta(env, "laivanupotus", "252 A")).status, 200);
  assert.deepEqual(rivit(env, "laivanupotus"), ["252 A"]);
});

test("liian pitkä runko hylätään otsakkeesta ja ilman otsaketta", async () => {
  const env = ymparisto();
  const pitka = "1 A" + " ".repeat(SUURIN_RUNKO);
  assert.equal((await laheta(env, "luvut", pitka)).status, 400);
  const virta = new ReadableStream({
    start(ohjain) {
      ohjain.enqueue(new TextEncoder().encode(pitka));
      ohjain.close();
    },
  });
  const paloina = await hae(env, "/tulos?peli=luvut", {
    method: "POST",
    body: virta,
    duplex: "half",
    headers: { "x-allekirjoitus": allekirjoita("luvut", pitka) },
  });
  assert.equal(paloina.status, 400);
  assert.deepEqual(rivit(env, "luvut"), []);
});

test("rajoitin laskee lähettäjän osoitteen mukaan ja ylitys antaa 429", async () => {
  const env = ymparisto({ sallittuja: 1 });
  assert.equal((await laheta(env, "luvut", "100 A")).status, 200);
  // 429 ei ole 400: peli pitää tuloksen jonossa ja yrittää myöhemmin uudelleen.
  const vastaus = await laheta(env, "luvut", "200 B");
  assert.deepEqual(vastaus, { status: 429, teksti: "liian monta tulosta\n" });
  assert.deepEqual(rivit(env, "luvut"), ["100 A"]);
  assert.deepEqual(env.avaimet, ["192.0.2.1", "192.0.2.1"]);
  // Listan lukemista ei rajoiteta.
  assert.equal((await hae(env, "/top3?peli=luvut")).status, 200);
});

test("ilman palvelimen avainta tulos jää pelin jonoon 503:lla", async () => {
  const env = ymparisto({ avain: "" });
  assert.equal((await laheta(env, "luvut", "100 A")).status, 503);
  assert.deepEqual(env.avaimet, []);
  assert.equal((await hae(env, "/top3?peli=luvut")).status, 200);
});
