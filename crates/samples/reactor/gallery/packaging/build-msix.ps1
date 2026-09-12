# Build ReactorWinUIGallery.msix from the self-contained gallery build.
#
# Usage:
#   powershell -File build-msix.ps1 [-TargetDir <dir>] [-OutputDir <dir>] [-Install] [-SkipSign]
#
# The package is self-contained: it carries the whole Windows App Runtime that
# window-reactor-setup staged beside the executable, and declares no framework
# dependency. Staging is an allow list (executable, DLLs, PRI files, language
# directories, the XAML resource directory) so Cargo's build directory and other
# sample binaries never enter the package.
#
# The language directories and the PRI files are not optional. A package that
# carries Microsoft.UI.Xaml.dll but drops its MUI and resource files fails at
# the first layout pass: XAML cannot load its localized resources
# (ERROR_MUI_FILE_NOT_LOADED, 0x80073B01), raises a stowed exception, and the
# process fail-fasts (0xC000027B).
param(
    [string]$TargetDir = '',
    [string]$OutputDir = '',
    [string]$CertificateName = 'ReactorGalleryDev',
    [switch]$Install,
    [switch]$SkipSign
)
$ErrorActionPreference = 'Stop'

if (-not $TargetDir) {
    # The workspace target directory lives at the repo root; walk up from the
    # script directory until it shows up.
    $dir = $PSScriptRoot
    while ($dir -and -not (Test-Path (Join-Path $dir 'target\debug'))) {
        $dir = Split-Path $dir -Parent
    }
    if (-not $dir) { throw 'target\debug not found above the script directory' }
    $TargetDir = Join-Path $dir 'target\debug'
}
if (-not (Test-Path $TargetDir)) { throw "build output not found: $TargetDir" }
if (-not $OutputDir) { $OutputDir = Join-Path $PSScriptRoot 'output' }

$staging = Join-Path $env:TEMP 'gallery-msix-stage'
if (Test-Path $staging) { Remove-Item $staging -Recurse -Force }
New-Item -ItemType Directory -Path (Join-Path $staging 'Assets') -Force | Out-Null

# --- payload -----------------------------------------------------------------
$exe = Join-Path $TargetDir 'reactor-gallery.exe'
if (-not (Test-Path $exe)) {
    throw "reactor-gallery.exe not found in $TargetDir; run cargo build -p reactor-gallery first"
}
Copy-Item $exe $staging

Get-ChildItem $TargetDir -File | Where-Object { $_.Extension -in '.dll', '.pri' } |
    Copy-Item -Destination $staging

# Language directories hold the .mui files XAML loads for localized resources.
$languageDirs = Get-ChildItem $TargetDir -Directory | Where-Object {
    $_.Name -match '^[a-zA-Z]{2}(-[a-zA-Z]{2,4})?$'
}
foreach ($d in $languageDirs) { Copy-Item $d.FullName -Destination $staging -Recurse }

# XAML theme and density resources (generic.xbf and friends).
$themeDir = Join-Path $TargetDir 'Microsoft.UI.Xaml'
if (Test-Path $themeDir) { Copy-Item $themeDir -Destination $staging -Recurse }

# --- completeness checks -----------------------------------------------------
$missing = @()
if (-not (Test-Path (Join-Path $staging 'Microsoft.UI.Xaml.dll'))) { $missing += 'Microsoft.UI.Xaml.dll' }
if (-not (Test-Path (Join-Path $staging 'Microsoft.UI.Xaml.Controls.dll'))) { $missing += 'Microsoft.UI.Xaml.Controls.dll' }
if (-not (Test-Path (Join-Path $staging 'resources.pri'))) { $missing += 'resources.pri' }
if (-not (Test-Path (Join-Path $staging 'Microsoft.UI.Xaml'))) { $missing += 'Microsoft.UI.Xaml\' }
if ($languageDirs.Count -eq 0) { $missing += 'language directories (for example en-us)' }
if ($missing.Count -gt 0) {
    throw ("self-contained layout is incomplete: " + ($missing -join ', ') +
        ". Run cargo build -p reactor-gallery so windows-reactor-setup stages the runtime.")
}
$enUs = Get-ChildItem $staging -Directory | Where-Object { $_.Name -match '^en' } | Select-Object -First 1
if ($enUs -and -not (Get-ChildItem $enUs.FullName -Filter '*.mui' | Select-Object -First 1)) {
    throw "language directory $($enUs.Name) carries no .mui files; XAML would fail to load resources"
}

# --- icons -------------------------------------------------------------------
Add-Type -AssemblyName System.Drawing
function New-Png($path, $w, $h) {
    $bmp = New-Object System.Drawing.Bitmap($w, $h)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.Clear([System.Drawing.Color]::FromArgb(255, 26, 115, 232))
    $bmp.Save($path, [System.Drawing.Imaging.ImageFormat]::Png)
    $g.Dispose(); $bmp.Dispose()
}
New-Png (Join-Path $staging 'Assets\StoreLogo.png') 50 50
New-Png (Join-Path $staging 'Assets\Square44x44Logo.png') 44 44
New-Png (Join-Path $staging 'Assets\Square150x150Logo.png') 150 150

# --- manifest ----------------------------------------------------------------
Copy-Item (Join-Path $PSScriptRoot 'Package.appxmanifest') (Join-Path $staging 'AppxManifest.xml')
[xml]$manifest = Get-Content (Join-Path $PSScriptRoot 'Package.appxmanifest')
$identityName = $manifest.Package.Identity.Name
$publisher = $manifest.Package.Identity.Publisher

# --- pack --------------------------------------------------------------------
New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
$out = Join-Path $OutputDir ($identityName + '.msix')
if (Test-Path $out) { Remove-Item $out -Force }

# MakeAppx writes [Content_Types].xml and AppxBlockMap.xml, which a hand-rolled
# zip does not.
$kitBin = Split-Path (Get-Command makeappx.exe -ErrorAction Stop).Source -Parent
& (Join-Path $kitBin 'makeappx.exe') pack /o /d $staging /p $out
if ($LASTEXITCODE -ne 0) { throw "MakeAppx failed with exit code $LASTEXITCODE" }

if (-not $SkipSign) {
    # The signing certificate subject must match Identity/@Publisher.
    & (Join-Path $kitBin 'signtool.exe') sign /fd SHA256 /a /n $CertificateName $out
    if ($LASTEXITCODE -ne 0) { throw "Signtool failed with exit code $LASTEXITCODE" }
}

if ($Install) {
    if ($SkipSign) { throw '-Install needs a signed package' }
    Get-AppxPackage -Name $identityName -ErrorAction Ignore | Remove-AppxPackage -ErrorAction Ignore
    Add-AppxPackage -Path $out -ErrorAction Stop
    Write-Output ("Installed: " + (Get-AppxPackage -Name $identityName).PackageFamilyName)
}

Write-Output ("MSIX: " + $out + " (" + [math]::Round((Get-Item $out).Length / 1MB) + " MB)")
Write-Output ("Publisher: " + $publisher + "  (signing certificate: " + $CertificateName + ")")
