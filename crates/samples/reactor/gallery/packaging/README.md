# Packaging Reactor WinUI Gallery (MSIX)

The gallery normally runs unpackaged via `cargo run -p sample_reactor_gallery`
(self-contained Windows App Runtime). To get an installed, identity-bearing
app with per-app privacy entries, package it as an MSIX.

## Development identity (no MSIX)

The taskbar badge (`Microsoft.Windows.BadgeNotifications`) requires a packaged
app identity. In development the gallery self-registers a sparse package from
its build output at startup:

1. `packaging.rs` copies `Package.appxmanifest` to `target/debug/AppxManifest.xml`
   (next to the executable, matching its `Executable` name).
2. `Add-AppxPackage -Register <manifest>` gives the in-place process an
   identity without installing or copying anything.
3. The app relaunches itself; the new process is packaged.

The manifest `Publisher` must match a trusted code-signing cert on the machine.
Create one once and trust it (`CN=ReactorGalleryDev` is the default):

```powershell
$cert = New-SelfSignedCertificate -Type Custom -Subject 'CN=ReactorGalleryDev' `
    -KeyUsage DigitalSignature -KeyAlgorithm RSA -KeyLength 2048 `
    -CertStoreLocation 'Cert:\CurrentUser\My' -NotAfter (Get-Date).AddYears(10)
$store = New-Object System.Security.Cryptography.X509Certificates.X509Store('Root', 'CurrentUser')
$store.Open('ReadWrite'); $store.Add($cert); $store.Close()
```

Then run `cargo run -p sample_reactor_gallery`; the first launch registers and
relaunches, and the badge buttons work.

## Why camera access needs packaging (or the desktop-app switch)

- The official WinUI 3 Gallery declares **only `runFullTrust`** in its
  `Package.appxmanifest` - there is no `<DeviceCapability Name="webcam"/>`.
  A `runFullTrust` desktop app is allowed to use the camera by the system
  setting "Let desktop apps access your camera" (registry
  `HKLM\...\CapabilityAccessManager\ConsentStore\webcam\NonPackaged = Allow`).
- An unpackaged Win32 app (the current gallery) has **no capability
  declaration mechanism** and depends on exactly the same system switch. If
  the switch is on but the `NonPackaged` registry value is missing, camera
  access returns `0x80070005` and no per-app permission entry appears in
  Settings - that is by design for unpackaged apps.
- Packaging as MSIX gives the app an identity: Settings shows a per-app
  camera entry, and first use can prompt for consent.

## Packaging steps (maintainer/CI)

1. `MakeAppx pack /d <this dir with assets> /p ReactorWinUIGallery.msix`
2. `signtool sign /fd SHA256 /a ReactorWinUIGallery.msix`
3. `Add-AppxPackage ReactorWinUIGallery.msix`
4. Run the gallery, then check Settings > Privacy & security > Camera for
   the app entry.

`Executable` must match the built binary name (`sample_reactor_gallery.exe`);
the Assets folder needs `StoreLogo.png`, `Square150x150Logo.png`,
`Square44x44Logo.png`.
