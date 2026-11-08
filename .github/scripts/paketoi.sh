#!/usr/bin/env bash
# Rakentaa Windows-paketin Linuxilla cargo-xwinillä: sama sisältö ja nimi kuin paketoi.ps1:llä,
# dist/klassikot8000-<versio>-windows-x64.zip. Ajetaan minipc:n runnerilla (.github/runner).
# Ajo: paketoi.sh [--avain <tiedosto>]
# Ilman avainta paketti on koeversio, joka ei lähetä tuloksia nettilistalle.
set -euo pipefail

cd "$(dirname "$0")/../.."
kohde=x86_64-pc-windows-msvc
versio=$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -n 1)
kansio="${CARGO_TARGET_DIR:-target}"
avain=paketointi/tulosavain

if [[ "${1:-}" == "--avain" ]]; then
    # Avainta ei tulosteta. Kopio poistetaan lopuksi, vaikka käännös epäonnistuisi.
    trap 'rm -f "$avain"' EXIT
    install -m 600 "$2" "$avain"
    if ! tr -d '[:space:]' < "$avain" | grep -Eqx '[0-9a-fA-F]{64}'; then
        echo "Tulosavain on viallinen ($2). Pakettia ei tehty." >&2
        exit 1
    fi
elif [[ -e "$avain" ]]; then
    echo "$avain on olemassa, vaikka avainta ei annettu. Pakettia ei tehty." >&2
    exit 1
fi

cargo xwin build --release --locked --target "$kohde"

lisenssit="$kansio/THIRD-PARTY-LICENSES.txt"
python3 paketointi/lisenssit.py "$lisenssit" "$kohde"

valikansio="$kansio/paketti"
rm -rf "$valikansio"
mkdir -p "$valikansio" dist
cp "$kansio/$kohde/release/klassikot8000.exe" "$valikansio/klassikot8000-$versio.exe"
cp kielet.txt LICENSE "$lisenssit" "$valikansio/"

paketti="$PWD/dist/klassikot8000-$versio-windows-x64.zip"
rm -f "$paketti"
# Samat 7-Zipin asetukset kuin paketoi.ps1:ssä.
(cd "$valikansio" && 7z a -tzip -mx=9 -mm=Deflate -mfb=258 -mpass=15 -bso0 -bsp0 "$paketti" ./*)

koko=$(stat -c %s "$paketti")
printf 'Paketti: %s\nexe: %d kt, zip: %d kt\n' "dist/${paketti##*/}" \
    "$(( $(stat -c %s "$valikansio/klassikot8000-$versio.exe") / 1024 ))" "$(( koko / 1024 ))"
if (( koko >= 10000000 )); then
    echo "Paketti on yli 10 Mt." >&2
    exit 1
fi
