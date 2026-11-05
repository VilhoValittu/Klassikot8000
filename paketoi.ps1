# Rakentaa julkaisuversion ja paketoi sen tiedostoon dist\klassikot8000-<versio>-windows-x64.zip.
# Paketissa ohjelma on nimella klassikot8000-<versio>.exe.
# Ajo: powershell -ExecutionPolicy Bypass -File paketoi.ps1
# Julkaisu: tee ensin tagi ja vie se GitHubiin (git tag v<versio>; git push origin v<versio>),
# sitten sama ajo valinnalla -Julkaise. Muutoslista tehdaan commit-viesteista edellisesta tagista.
# Pidetty ASCII-merkeissa, koska PowerShell 5.1 lukee BOM-merkittoman tiedoston vaarin.

param([switch]$Julkaise)

$ErrorActionPreference = 'Stop'
Set-Location $PSScriptRoot

$versio = (Select-String -Path Cargo.toml -Pattern '^version = "(.+)"$' | Select-Object -First 1).Matches[0].Groups[1].Value
$tagi = "v$versio"

# Avain on vain julkaisijan koneella. Ilman sita paketti ei voisi lahettaa tuloksia nettilistalle.
$avain = 'paketointi\tulosavain'
if (-not (Test-Path $avain) -or ((Get-Content $avain -Raw).Trim() -notmatch '^[0-9a-fA-F]{64}$')) {
    Write-Error "Tulosavain puuttuu tai on viallinen ($avain). Pakettia ei tehty, ks. netti.md."
}

if ($Julkaise) {
    git rev-parse -q --verify "refs/tags/$tagi" | Out-Null
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Tagia $tagi ei ole. Tee se ja vie GitHubiin ennen julkaisua."
    }
}

cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Error 'Kaannos ei onnistunut, pakettia ei tehty.'
}

$lisenssit = 'target\THIRD-PARTY-LICENSES.txt'
python paketointi\lisenssit.py $lisenssit
if ($LASTEXITCODE -ne 0) {
    Write-Error 'Lisenssilista ei onnistunut, pakettia ei tehty.'
}

$paketti = "dist\klassikot8000-$versio-windows-x64.zip"

# Compress-Archive ei osaa nimeta tiedostoa uudelleen, joten ohjelma kopioidaan versioidulla nimella valikansioon.
$valikansio = 'target\paketti'
if (Test-Path $valikansio) {
    Remove-Item -Recurse -Force $valikansio
}
New-Item -ItemType Directory -Force $valikansio | Out-Null
$exe = "$valikansio\klassikot8000-$versio.exe"
Copy-Item 'target\release\klassikot8000.exe' $exe

New-Item -ItemType Directory -Force dist | Out-Null
Compress-Archive -Path $exe, 'kielet.txt', 'LICENSE', $lisenssit -DestinationPath $paketti -CompressionLevel Optimal -Force

Write-Host ''
Write-Host "Paketti: $paketti"
Write-Host ('exe: {0:N0} kt, zip: {1:N0} kt' -f ((Get-Item $exe).Length / 1KB), ((Get-Item $paketti).Length / 1KB))

# Paketin on pysyttava alle 10 Mt:n.
if ((Get-Item $paketti).Length -ge 10000000) {
    Write-Error 'Paketti on yli 10 Mt.'
}

if ($Julkaise) {
    # Gitin oma bash: polun ensimmainen bash voi olla WSL:n, joka ei nae tata kansiota samoin.
    $bash = Join-Path (Split-Path (git var GIT_SHELL_PATH)) 'bash.exe'
    $muutokset = 'target\muutokset.md'
    $teksti = & $bash .github/scripts/muutokset.sh $tagi
    if ($LASTEXITCODE -ne 0) {
        Write-Error 'Muutoslista ei onnistunut, julkaisua ei tehty.'
    }
    [IO.File]::WriteAllText((Join-Path $PSScriptRoot $muutokset), ($teksti -join "`n") + "`n", (New-Object Text.UTF8Encoding($false)))
    $valinnat = @($tagi, $paketti, '--verify-tag', '--title', "Klassikot8000 $tagi", '--notes-file', $muutokset)
    if ($tagi.Contains('-')) {
        $valinnat += '--prerelease'
    }
    gh release create @valinnat
    if ($LASTEXITCODE -ne 0) {
        Write-Error 'Julkaisu ei onnistunut.'
    }
}
