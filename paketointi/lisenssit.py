"""Kokoaa ohjelmaan linkitettävien pakettien lisenssitekstit yhteen tiedostoon.

Ajo: python paketointi/lisenssit.py <tulostiedosto> [kohde ...]

Ilman kohdetta lista tehdään tämän koneen alustalle. Ei riippuvuuksia, jotta paketointi ei tarvitse
muuta kuin Rustin ja Pythonin. Kaatuu, jos paketilta ei löydy lisenssitekstiä, koska silloin
julkaisusta puuttuisi sen vaatima teksti.
"""

import json
import re
import subprocess
import sys
from pathlib import Path

JUURI = Path(__file__).parent.parent

LISENSSITIEDOSTO = re.compile(r"^(un)?licen[cs]e|^copying|^copyright|^notice", re.IGNORECASE)

# openssl-src on käännösaikainen riippuvuus, mutta se kääntää OpenSSL:n ohjelmaan. OpenSSL:n oma
# lisenssi on alikansiossa.
MUKAAN_KAANTYVAT = {"openssl-src"}
LISATIEDOSTOT = {"openssl-src": ["openssl/LICENSE.txt"]}

OTSIKKO = """Klassikot8000 uses the third-party software listed below. Each section gives the
license text of the packages named above it.

The Rust standard library is also part of the program. It is available under
MIT OR Apache-2.0: https://github.com/rust-lang/rust
"""
VIIVA = "=" * 80


def cargo(*argumentit):
    return subprocess.run(
        ["cargo", *argumentit, "--locked"],
        cwd=JUURI,
        check=True,
        capture_output=True,
        encoding="utf-8",
    ).stdout


def linkitetyt(kohde):
    """Palauttaa (nimi, versio) -parit. Vain käännösaikana käytetyt paketit jäävät pois."""
    argumentit = ["--prefix", "none", "-f", "{p}"]
    if kohde:
        argumentit += ["--target", kohde]
    parit = puun_paketit("normal", argumentit)
    parit |= {p for p in puun_paketit("normal,build", argumentit) if p[0] in MUKAAN_KAANTYVAT}
    return parit


def puun_paketit(riippuvuudet, argumentit):
    parit = set()
    for rivi in cargo("tree", "-e", riippuvuudet, *argumentit).splitlines():
        if rivi.strip():
            nimi, versio = rivi.split()[:2]
            parit.add((nimi, versio.removeprefix("v")))
    return parit


def lisenssitekstit(paketti):
    kansio = Path(paketti["manifest_path"]).parent
    polut = [p for p in kansio.iterdir() if p.is_file() and LISENSSITIEDOSTO.match(p.name)]
    polut += [kansio / nimi for nimi in LISATIEDOSTOT.get(paketti["name"], [])]
    if paketti.get("license_file"):
        polut.append(kansio / paketti["license_file"])
    # Rivinvaihdot yhtenäistetään, jotta sama teksti tunnistetaan samaksi.
    return {
        p.read_text(encoding="utf-8", errors="replace").replace("\r\n", "\n").strip()
        for p in set(polut)
    }


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    tuloste = Path(sys.argv[1])
    kohteet = sys.argv[2:] or [None]

    tarvittavat = set().union(*(linkitetyt(kohde) for kohde in kohteet))
    metatiedot = json.loads(cargo("metadata", "--format-version", "1"))

    ryhmat = {}
    puuttuvat = []
    for paketti in metatiedot["packages"]:
        # Omalla paketilla ei ole lähdettä, ja sen lisenssi on mukana omana tiedostonaan.
        if paketti["source"] is None or (paketti["name"], paketti["version"]) not in tarvittavat:
            continue
        tekstit = lisenssitekstit(paketti)
        if not tekstit:
            puuttuvat.append(paketti["name"])
        kuvaus = f"{paketti['name']} {paketti['version']} ({paketti['license']})"
        for teksti in tekstit:
            ryhmat.setdefault(teksti, []).append(kuvaus)

    if puuttuvat:
        sys.exit("Lisenssiteksti puuttuu paketeilta: " + ", ".join(sorted(puuttuvat)))

    osat = [OTSIKKO]
    for teksti, kuvaukset in sorted(ryhmat.items(), key=lambda ryhma: sorted(ryhma[1])):
        osat.append(f"{VIIVA}\n" + "\n".join(sorted(kuvaukset)) + f"\n{VIIVA}\n\n{teksti}\n")

    tuloste.parent.mkdir(parents=True, exist_ok=True)
    tuloste.write_text("\n".join(osat), encoding="utf-8", newline="\n")
    print(f"{tuloste}: {len(tarvittavat)} pakettia, {len(ryhmat)} eri tekstiä")


if __name__ == "__main__":
    main()
