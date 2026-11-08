#!/usr/bin/env bash
# Rekisteröi runnerin ensimmäisellä käynnistyksellä ja käynnistää sen. Rekisteröinti säilyy
# asetukset-levyllä, joten kontin voi rakentaa uudelleen ilman uutta rekisteröintiä.
set -euo pipefail

cd /home/runner/actions-runner
asetukset=/home/runner/asetukset
tiedostot=(.runner .credentials .credentials_rsaparams)

if [[ -f "$asetukset/.runner" ]]; then
    for f in "${tiedostot[@]}"; do cp "$asetukset/$f" .; done
else
    if [[ -z "${REKISTEROINTI:-}" ]]; then
        echo "Runneria ei ole rekisteröity. Käynnistä kerran: REKISTEROINTI=<token> docker compose up -d" >&2
        exit 1
    fi
    ./config.sh --unattended --url "$REPO_URL" --token "$REKISTEROINTI" \
        --name minipc --labels minipc --work /home/runner/tyo --replace
    for f in "${tiedostot[@]}"; do cp "$f" "$asetukset/"; done
fi

exec ./run.sh
