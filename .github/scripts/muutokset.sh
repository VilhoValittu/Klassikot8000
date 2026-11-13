#!/usr/bin/env bash
# Tekee julkaisun muutoslistan commit-viesteistä: edellisestä versiotagista annettuun tagiin.
# Ajo: muutokset.sh v0.3.4, tai muutokset.sh HEAD ennen tagia (julkaise.py:n esikatselu).
set -euo pipefail

tagi="$1"
# Versiotagin oma tagi ohitetaan; muu viite (HEAD) verrataan sen omaan viimeisimpään tagiin.
lahto="$tagi"
if [[ "$tagi" == v[0-9]* ]]; then
    lahto="$tagi^"
fi
edellinen=$(git describe --tags --abbrev=0 --match 'v[0-9]*' "$lahto" 2>/dev/null || true)
alue="${edellinen:+$edellinen..}$tagi"

uutta=""
korjaukset=""
muuta=""
kaava='^([a-z]+)(\([^)]*\))?!?: *(.*)$'
while IFS=$'\t' read -r tiiviste viesti; do
    # Versionnostot eivät kerro pelaajalle mitään.
    if [[ "$viesti" == *klassikot8000\ version* ]]; then
        continue
    fi
    laji=""
    teksti="$viesti"
    if [[ "$viesti" =~ $kaava ]]; then
        laji="${BASH_REMATCH[1]}"
        teksti="${BASH_REMATCH[3]}"
    fi
    rivi="- ${teksti} (${tiiviste})"$'\n'
    case "$laji" in
        feat) uutta+="$rivi" ;;
        fix) korjaukset+="$rivi" ;;
        *) muuta+="$rivi" ;;
    esac
done < <(git log --no-merges --format='%h%x09%s' "$alue")

osio() {
    if [[ -n "$2" ]]; then
        printf '## %s\n\n%s\n' "$1" "$2"
    fi
}

osio "Uutta" "$uutta"
osio "Korjaukset" "$korjaukset"
osio "Muuta" "$muuta"

if [[ -n "$edellinen" && -n "${GITHUB_REPOSITORY:-}" ]]; then
    printf '**Kaikki muutokset**: https://github.com/%s/compare/%s...%s\n' "$GITHUB_REPOSITORY" "$edellinen" "$tagi"
fi
