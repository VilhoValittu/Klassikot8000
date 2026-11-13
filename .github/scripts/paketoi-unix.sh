#!/usr/bin/env bash
# Rakentaa Linux- tai macOS-paketin tämän koneen arkkitehtuurille natiivikäännöksenä:
#   Linux: dist/klassikot8000-<versio>-linux-<x64|arm64>.tar.gz (kansio, jossa ohjelma, ikoni,
#          kielet.txt ja lisenssit)
#   macOS: dist/klassikot8000-<versio>-macos-<arm64|x64>.zip (Klassikot8000-<versio>.app, ad hoc
#          -allekirjoitus, koska Apple Silicon ei aja allekirjoittamatonta ohjelmaa lainkaan)
# Windows-paketin tekee paketoi.sh. Ajetaan release-linux-*.yml- ja release-macos-*.yml-työnkuluissa.
# Ajo: paketoi-unix.sh [--avain <tiedosto>]
# Ilman avainta paketti on koeversio, joka ei lähetä tuloksia nettilistalle.
set -euo pipefail

cd "$(dirname "$0")/../.."
versio=$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -n 1)
kansio="${CARGO_TARGET_DIR:-target}"
avain=paketointi/tulosavain

case "$(uname -s)" in
    Linux) alusta=linux ;;
    Darwin) alusta=macos ;;
    *) echo "Tuntematon käyttöjärjestelmä $(uname -s). Pakettia ei tehty." >&2; exit 1 ;;
esac
case "$(uname -m)" in
    x86_64 | amd64) arkki=x64 ;;
    aarch64 | arm64) arkki=arm64 ;;
    *) echo "Tuntematon arkkitehtuuri $(uname -m). Pakettia ei tehty." >&2; exit 1 ;;
esac

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

cargo build --release --locked
exe="$kansio/release/klassikot8000"

lisenssit="$kansio/THIRD-PARTY-LICENSES.txt"
python3 paketointi/lisenssit.py "$lisenssit"

valikansio="$kansio/paketti"
rm -rf "$valikansio"
mkdir -p "$valikansio" dist

if [[ "$alusta" == linux ]]; then
    nimi="klassikot8000-$versio-linux-$arkki"
    sisalto="$valikansio/$nimi"
    mkdir -p "$sisalto"
    cp "$exe" "$sisalto/klassikot8000-$versio"
    cp kielet.txt LICENSE "$lisenssit" "$sisalto/"
    # Linuxin ohjelmatiedostossa ei voi olla ikonia, joten se tulee mukaan pikakuvaketta varten.
    cp paketointi/ikoni.png "$sisalto/klassikot8000.png"
    paketti="$PWD/dist/$nimi.tar.gz"
    rm -f "$paketti"
    tar -C "$valikansio" -czf "$paketti" "$nimi"
else
    sovellus="$valikansio/Klassikot8000-$versio.app"
    sisalto="$sovellus/Contents"
    mkdir -p "$sisalto/MacOS" "$sisalto/Resources"
    cp "$exe" "$sisalto/MacOS/klassikot8000"
    # kielet.txt jää pois: peli käyttää upotettua kopiota, ja ylimääräinen tiedosto MacOS-kansiossa
    # rikkoisi allekirjoituksen.
    cp paketointi/ikoni.icns LICENSE "$lisenssit" "$sisalto/Resources/"
    sed "s/VERSIO/$versio/g" paketointi/Info.plist > "$sisalto/Info.plist"
    codesign --force --sign - "$sovellus"
    paketti="$PWD/dist/klassikot8000-$versio-macos-$arkki.zip"
    rm -f "$paketti"
    ditto -c -k --keepParent "$sovellus" "$paketti"
fi

koko=$(wc -c < "$paketti" | tr -d ' ')
printf 'Paketti: %s\nohjelma: %d kt, paketti: %d kt\n' "dist/${paketti##*/}" \
    "$(( $(wc -c < "$exe" | tr -d ' ') / 1024 ))" "$(( koko / 1024 ))"
if (( koko >= 10000000 )); then
    echo "Paketti on yli 10 Mt." >&2
    exit 1
fi
