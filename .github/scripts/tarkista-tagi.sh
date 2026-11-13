#!/usr/bin/env bash
# Tarkistaa julkaisun tagin ennen paketin rakentamista: muoto v1.2.3, Cargo.toml:n versio on sama ja
# tagi on mainissa (vain mainin commitit on testattu, tarkistukset.yml). Tarvitsee mainin historian
# (actions/checkout fetch-depth: 0). Kirjoittaa VERSION=<versio> tiedostoon $GITHUB_ENV.
# Ajo: tarkista-tagi.sh v1.2.3 (release-*.yml)
set -euo pipefail

cd "$(dirname "$0")/../.."
tagi="$1"

if [[ ! "$tagi" =~ ^v([0-9]+\.[0-9]+\.[0-9]+)$ ]]; then
    echo "Tagin on oltava muotoa v1.2.3, nyt: $tagi" >&2
    exit 1
fi
versio="${BASH_REMATCH[1]}"
cargo_versio=$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -n 1)
if [[ "$cargo_versio" != "$versio" ]]; then
    echo "Tagin $tagi Cargo.toml:ssa versio on $cargo_versio. Nosta versio julkaise.bat:llä." >&2
    exit 1
fi
if ! git merge-base --is-ancestor HEAD origin/main; then
    echo "Tagi $tagi ei ole mainissa. Julkaisu tehdään vain mainin commitista." >&2
    exit 1
fi
if [[ -n "${GITHUB_ENV:-}" ]]; then
    echo "VERSION=$versio" >> "$GITHUB_ENV"
fi
echo "Tagi $tagi kelpaa (versio $versio)."
