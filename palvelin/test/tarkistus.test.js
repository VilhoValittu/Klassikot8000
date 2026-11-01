import assert from "node:assert/strict";
import { test } from "node:test";

import { lue_tulos, PELIT, YKSI_PER_NIMI } from "../src/tarkistus.js";

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
