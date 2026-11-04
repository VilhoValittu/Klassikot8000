# Rakentaa julkaisuversion ja paketoi sen tiedostoon dist\klassikot8000-<versio>-windows-x64.zip.
# Paketissa ohjelma on nimella klassikot8000-<versio>.exe.
# Ajo: powershell -ExecutionPolicy Bypass -File paketoi.ps1
# Pidetty ASCII-merkeissa, koska PowerShell 5.1 lukee BOM-merkittoman tiedoston vaarin.

$ErrorActionPreference = 'Stop'
Set-Location $PSScriptRoot

$versio = (Select-String -Path Cargo.toml -Pattern '^version = "(.+)"$' | Select-Object -First 1).Matches[0].Groups[1].Value

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
