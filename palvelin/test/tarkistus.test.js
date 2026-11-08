import assert from "node:assert/strict";
import { test } from "node:test";

import {
  allekirjoitus_kelpaa,
  lue_runko,
  lue_tulos,
  PELIT,
  SUURIN_RUNKO,
  YKSI_PER_NIMI,
} from "../src/tarkistus.js";

test("kelvollinen tulos luetaan", () => {
  assert.deepEqual(lue_tulos("120 VÄINÖ7\n", 1000), { pisteet: 120, nimi: "VÄINÖ7" });
  assert.deepEqual(lue_tulos("1000 A", 1000), { pisteet: 1000, nimi: "A" });
});

test("kelvoton tulos hylätään", () => {
  for (const teksti of [
    "",
    "0 A",
    "-1 A",
    "1001 A",
    "12",
    "12 ",
    "12 pieni",
    "12 LIIANPITKA",
    "12 KAKSI OSAA",
    "12 A\n13 B",
    "1e2 A",
  ]) {
    assert.equal(lue_tulos(teksti, 1000), null, teksti);
  }
});

test("haasteiden lista kattaa 60 haasteen summan", () => {
  assert.equal(PELIT.haasteet, 60 * PELIT.pasianssi);
  assert.ok(YKSI_PER_NIMI.has("haasteet"));
  assert.ok(!YKSI_PER_NIMI.has("pasianssi"));
});

test("miinaharavan lista hyväksyy vaikeimman kentän parhaan tuloksen", () => {
  assert.deepEqual(lue_tulos("2997 MIINA", PELIT.miinaharava), { pisteet: 2997, nimi: "MIINA" });
  assert.equal(lue_tulos("3001 MIINA", PELIT.miinaharava), null);
});

test("laivanupotuksen lista hyväksyy nopeimman vaikean voiton", () => {
  assert.deepEqual(lue_tulos("252 AMIRAALI", PELIT.laivanupotus), { pisteet: 252, nimi: "AMIRAALI" });
  assert.equal(lue_tulos("253 A", PELIT.laivanupotus), null);
});

test("liian pitkä runko hylätään, mutta pisin kelvollinen rivi mahtuu", () => {
  const pisin = "999999999 ÄÄÄÄÄÄÄÄ";
  assert.ok(new TextEncoder().encode(pisin).length <= SUURIN_RUNKO);
  assert.deepEqual(lue_tulos(pisin, 1e9), { pisteet: 999999999, nimi: "ÄÄÄÄÄÄÄÄ" });
  assert.equal(lue_tulos(`12 A${" ".repeat(SUURIN_RUNKO)}`, 1000), null);
});

/// Runko paloina, kuten ilman content-lengthiä lähetetyssä pyynnössä.
function paloina(...palat) {
  return new ReadableStream({
    start(ohjain) {
      for (const pala of palat) {
        ohjain.enqueue(new TextEncoder().encode(pala));
      }
      ohjain.close();
    },
  });
}

test("runko luetaan paloista rajaan asti", async () => {
  assert.equal(await lue_runko(paloina("120 ", "VÄINÖ7")), "120 VÄINÖ7");
  assert.equal(await lue_runko(paloina("1".repeat(SUURIN_RUNKO))), "1".repeat(SUURIN_RUNKO));
  assert.equal(await lue_runko(null), "");
});

test("rajan ylittävä runko hylätään lukematta loppuun", async () => {
  assert.equal(await lue_runko(paloina("1".repeat(SUURIN_RUNKO), "1")), null);
  // Loputon virta: lukeminen pysähtyy rajaan eikä jää odottamaan loppua.
  const loputon = new ReadableStream({
    pull(ohjain) {
      ohjain.enqueue(new Uint8Array(16));
    },
  });
  assert.equal(await lue_runko(loputon), null);
});

// Sama avain ja allekirjoitus ovat pelin testissä (kaynnistin/allekirjoitus.rs).
const AVAIN = Array.from({ length: 32 }, (_, i) => i.toString(16).padStart(2, "0")).join("");
const ALLEKIRJOITUS = "b749393e7f9c87cb24073eef4b9abf4bb361b5f6e1a3b4ad4a21d57823dde500";

test("pelin allekirjoitus kelpaa", async () => {
  assert.ok(await allekirjoitus_kelpaa(AVAIN, "luvut", "120 VÄINÖ7", ALLEKIRJOITUS));
  assert.ok(
    await allekirjoitus_kelpaa(`${AVAIN}\n`, "luvut", "120 VÄINÖ7", ALLEKIRJOITUS.toUpperCase()),
  );
});

test("muutettu tulos, toinen peli tai puuttuva allekirjoitus hylätään", async () => {
  assert.ok(!(await allekirjoitus_kelpaa(AVAIN, "luvut", "121 VÄINÖ7", ALLEKIRJOITUS)));
  assert.ok(!(await allekirjoitus_kelpaa(AVAIN, "tiilirikko", "120 VÄINÖ7", ALLEKIRJOITUS)));
  assert.ok(!(await allekirjoitus_kelpaa(AVAIN, "luvut", "120 VÄINÖ7", null)));
  assert.ok(!(await allekirjoitus_kelpaa(AVAIN, "luvut", "120 VÄINÖ7", "zz")));
  const toinen = AVAIN.replace(/^00/, "ff");
  assert.ok(!(await allekirjoitus_kelpaa(toinen, "luvut", "120 VÄINÖ7", ALLEKIRJOITUS)));
  assert.ok(!(await allekirjoitus_kelpaa("", "luvut", "120 VÄINÖ7", ALLEKIRJOITUS)));
});
