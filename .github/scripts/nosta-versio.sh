#!/usr/bin/env bash
# Nostaa Cargo.toml:n ja Cargo.lock:n version tagin mukaiseksi, kun pelkkä tagi viedään GitHubiin
# (git tag v1.2.3; git push origin v1.2.3). Tekee saman versionnostocommitin kuin julkaise.ps1,
# vie sen mainiin ja siirtää tagin siihen, jotta julkaisun lähdekoodi vastaa tagia.
# Jos versio on jo oikea (julkaise.ps1 teki noston), tarkistaa vain, että tagi on mainissa:
# julkaisu ei aja testejä, ja vain mainin commitit on testattu (tarkistukset.yml).
# Ajo: nosta-versio.sh v1.2.3 (julkaisu.yml, minipc:n runner)
set -euo pipefail

cd "$(dirname "$0")/../.."
tagi="$1"

if [[ ! "$tagi" =~ ^v([0-9]+\.[0-9]+\.[0-9]+)$ ]]; then
    echo "Tagi '$tagi' ei kelpaa: kirjoita se muodossa v1.2.3." >&2
    exit 1
fi
uusi="${BASH_REMATCH[1]}"
nykyinen=$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -n 1)

git fetch -q origin main
if [[ "$nykyinen" == "$uusi" ]]; then
    if ! git merge-base --is-ancestor HEAD origin/main; then
        echo "Tagi $tagi ei ole mainissa. Julkaisu tehdään vain mainin commitista." >&2
        exit 1
    fi
    echo "Versio on jo $uusi, nostoa ei tarvita."
    exit 0
fi
if [[ "$(printf '%s\n%s\n' "$nykyinen" "$uusi" | sort -V | tail -n 1)" != "$uusi" ]]; then
    echo "Tagin versio $uusi ei ole suurempi kuin nykyinen $nykyinen." >&2
    exit 1
fi

# Nosto tehdään mainin kärkeen, kuten julkaise.ps1:ssä. Muuten main ja julkaisu erkanisivat.
if [[ "$(git rev-parse HEAD)" != "$(git rev-parse origin/main)" ]]; then
    echo "Tagi $tagi ei osoita mainin uusimpaan committiin. Siirrä tagi mainin kärkeen ja vie se uudelleen." >&2
    exit 1
fi

echo "Versio $nykyinen -> $uusi"
sed -i "0,/^version = \".*\"$/s//version = \"$uusi\"/" Cargo.toml
sed -i "/^name = \"klassikot8000\"$/{n;s/^version = \".*\"$/version = \"$uusi\"/}" Cargo.lock
if ! grep -qx "version = \"$uusi\"" Cargo.toml ||
    ! grep -A1 -x 'name = "klassikot8000"' Cargo.lock | grep -qx "version = \"$uusi\""; then
    echo "Versiota ei saatu vaihdettua Cargo.toml:iin tai Cargo.lock:iin." >&2
    exit 1
fi
# --locked kaatuu, jos Cargo.lock ei vastaa Cargo.toml:ia.
cargo metadata --locked --format-version 1 > /dev/null

git -c user.name='github-actions[bot]' \
    -c user.email='41898282+github-actions[bot]@users.noreply.github.com' \
    commit -q -m "feat: Bump klassikot8000 version to $uusi" -- Cargo.toml Cargo.lock
git tag -f "$tagi" > /dev/null
# --atomic: main ja tagi päivittyvät yhdessä tai eivät ollenkaan. Main päivittyy vain
# eteenpäin, joten väliin tullut uusi commit kaataa viennin eikä katoa.
git push --atomic origin HEAD:refs/heads/main "+refs/tags/$tagi:refs/tags/$tagi"
echo "Versionnosto $(git rev-parse --short HEAD) on mainissa, ja $tagi osoittaa siihen."
