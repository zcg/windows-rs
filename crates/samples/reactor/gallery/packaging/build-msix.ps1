# 从已构建的 self-contained 产物生成 ReactorWinUIGallery.msix。
# 用法：powershell -File build-msix.ps1 [-Install]
#   -Install 打包签名之后卸旧装新，一条命令完成整套打包安装流程。
param(
    [string]$TargetDir = '',
    [string]$OutputDir = (Join-Path $PSScriptRoot 'output'),
    [switch]$Install
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
[xml]$manifestXml = Get-Content (Join-Path $PSScriptRoot 'Package.appxmanifest')
$identityName = $manifestXml.Package.Identity.Name
$staging = Join-Path $env:TEMP 'gallery-msix-stage'
if (Test-Path $staging) { Remove-Item $staging -Recurse -Force }
New-Item -ItemType Directory -Path (Join-Path $staging 'Assets') -Force | Out-Null

Add-Type -AssemblyName System.Drawing
function New-Png($path, $w, $h) {
    $bmp = New-Object System.Drawing.Bitmap($w, $h)
    $g2 = [System.Drawing.Graphics]::FromImage($bmp)
    $g2.Clear([System.Drawing.Color]::FromArgb(255, 26, 115, 232))
    $bmp.Save($path, [System.Drawing.Imaging.ImageFormat]::Png)
    $g2.Dispose(); $bmp.Dispose()
}
New-Png (Join-Path $staging 'Assets\StoreLogo.png') 50 50
New-Png (Join-Path $staging 'Assets\Square44x44Logo.png') 44 44
New-Png (Join-Path $staging 'Assets\Square150x150Logo.png') 150 150

Copy-Item (Join-Path $PSScriptRoot 'Package.appxmanifest') (Join-Path $staging 'AppxManifest.xml')
Copy-Item (Join-Path $TargetDir 'reactor-gallery.exe') $staging

# The MSIX is framework-dependent: the Windows App Runtime files listed in
# reactor-setup's assets/runtime.txt are provided by the installed
# Microsoft.WindowsAppRuntime.2 framework package (declared as a
# <PackageDependency> in the manifest). Shipping them beside the executable
# too makes a packaged process load two copies of the runtime, which
# fail-fasts in CoreMessagingXP. Everything else (VC runtime, WebView2) is
# carried in the package.
$frameworkFiles = @{}
$runtimeList = Join-Path $PSScriptRoot '..\..\..\..\libs\reactor-setup\assets\runtime.txt'
if (Test-Path $runtimeList) {
    Get-Content $runtimeList | ForEach-Object { $frameworkFiles[$_.Trim().ToLowerInvariant()] = $true }
} else {
    throw 'reactor-setup assets/runtime.txt not found'
}
Get-ChildItem $TargetDir -Filter '*.dll' | Where-Object {
    -not $frameworkFiles.ContainsKey($_.Name.ToLowerInvariant())
} | Copy-Item -Destination $staging
Get-ChildItem $TargetDir -Filter '*.pri' | Where-Object {
    -not $frameworkFiles.ContainsKey($_.Name.ToLowerInvariant())
} | Copy-Item -Destination $staging

New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
$out = Join-Path $OutputDir ($identityName + '.msix')
if (Test-Path $out) { Remove-Item $out -Force }

# Pack and sign with the Windows SDK tools: MakeAppx writes the
# [Content_Types].xml and AppxBlockMap.xml a valid package needs, which a
# hand-rolled zip cannot.
$kitBin = Split-Path (Get-Command makeappx.exe -ErrorAction Stop).Source -Parent
$makeappx = Join-Path $kitBin 'makeappx.exe'
$signtool = Join-Path $kitBin 'signtool.exe'
& $makeappx pack /o /d $staging /p $out
if ($LASTEXITCODE -ne 0) { throw "MakeAppx failed with exit code $LASTEXITCODE" }
& $signtool sign /fd SHA256 /a /n 'ReactorGalleryDev' $out
if ($LASTEXITCODE -ne 0) { throw "Signtool failed with exit code $LASTEXITCODE" }

if ($Install) {
    Get-AppxPackage -Name $identityName | Remove-AppxPackage -ErrorAction SilentlyContinue
    Add-AppxPackage -Path $out -ErrorAction Stop
    Write-Output ("Installed: " + (Get-AppxPackage -Name $identityName).PackageFamilyName)
}

Write-Output ("MSIX: " + $out + " (" + [math]::Round((Get-Item $out).Length / 1MB) + " MB)")
