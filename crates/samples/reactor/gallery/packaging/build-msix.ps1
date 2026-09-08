# 从已构建的 self-contained 产物生成 ReactorWinUIGallery.msix。
# 用法：pwsh -File build-msix.ps1 [-TargetDir <repo>\target\debug]
param(
    [string]$TargetDir = (Join-Path (Split-Path $PSScriptRoot -Parent) 'target\debug'),
    [string]$OutputDir = (Join-Path $PSScriptRoot 'output')
)
$ErrorActionPreference = 'Stop'
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
Copy-Item (Join-Path $TargetDir 'sample_reactor_gallery.exe') $staging
Get-ChildItem $TargetDir -Filter '*.dll' | Copy-Item -Destination $staging
Get-ChildItem $TargetDir -Filter '*.pri' | Copy-Item -Destination $staging

Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem
New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
$out = Join-Path $OutputDir 'ReactorWinUIGallery.msix'
if (Test-Path $out) { Remove-Item $out -Force }

# 1. Zip all staged files (BlockMap is added last, so it is not in the map).
$zip = [System.IO.Compression.ZipFile]::Open($out, [System.IO.Compression.ZipArchiveMode]::Create)
foreach ($f in (Get-ChildItem $staging -Recurse -File)) {
    $rel = $f.FullName.Substring($staging.Length + 1).Replace('\', '/')
    [System.IO.Compression.ZipFileExtensions]::CreateEntryFromFile($zip, $f.FullName, $rel, [System.IO.Compression.CompressionLevel]::Optimal) | Out-Null
}
$zip.Dispose()

# 2. BlockMap with Size + LZSize (read from the written zip) + SHA-256.
$sha = [System.Security.Cryptography.SHA256]::Create()
$reader = [System.IO.Compression.ZipFile]::OpenRead($out)
$lines = foreach ($entry in $reader.Entries) {
    $file = Join-Path $staging ($entry.FullName.Replace('/', '\'))
    $hash = [Convert]::ToBase64String($sha.ComputeHash([System.IO.File]::ReadAllBytes($file)))
    '  <File Name="' + $entry.FullName + '" Size="' + $entry.Length + '" LZSize="' + $entry.CompressedLength + '" Hash="' + $hash + '" />'
}
$reader.Dispose()
$blockMap = '<?xml version="1.0" encoding="UTF-8"?>' + "`n" + '<BlockMap xmlns="http://schemas.microsoft.com/appx/2010/blockmap" HashMethod="http://www.w3.org/2001/04/xmlenc#sha256">' + "`n" + ($lines -join "`n") + "`n</BlockMap>`n"

# 3. Append AppxBlockMap.xml stored (uncompressed).
$zip = [System.IO.Compression.ZipFile]::Open($out, [System.IO.Compression.ZipArchiveMode]::Update)
$entry = $zip.CreateEntry('AppxBlockMap.xml', [System.IO.Compression.CompressionLevel]::NoCompression)
$stream = $entry.Open()
$writer = New-Object System.IO.StreamWriter($stream, (New-Object System.Text.UTF8Encoding($false)))
$writer.Write($blockMap)
$writer.Flush(); $writer.Dispose(); $stream.Dispose()
$zip.Dispose()

Write-Output ("MSIX: " + $out + " (" + [math]::Round((Get-Item $out).Length / 1MB) + " MB)")
Write-Output "Install (administrator PowerShell):"
Write-Output "  Add-AppxPackage -Path $out"
