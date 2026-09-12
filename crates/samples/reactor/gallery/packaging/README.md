# Packaging Reactor WinUI Gallery (MSIX)

The gallery normally runs unpackaged via `cargo run -p reactor-gallery`, fully
self-contained: the Windows App Runtime 2.4 files sit beside the executable and
nothing outside the build output is touched, so a clean system needs no
Windows App SDK install.

## Taskbar badge and toasts

The Notifications page sets the badge through the Windows App SDK badge manager
(`Microsoft.Windows.BadgeNotifications.BadgeNotificationManager`) and sends
toasts through `AppNotificationManager`. Both APIs require a packaged app
identity, which a plain executable launch never has.

Identity is opt-in: `reactor-gallery --packaged` registers the build output in
place (or activates the installed MSIX) and relaunches packaged. Known issue on
Windows Insider builds (26340): every packaged launch fail-fasts in
`CoreMessagingXP` 10.0.27200 regardless of packaging style - self-contained
files, a framework-dependent package, or an in-place registration all crash
before the window appears, so on those builds the flag stays unused and the
Notifications page reports the missing identity instead.

To register the build output by hand:

1. Copy `Package.appxmanifest` to `target/debug/AppxManifest.xml` (next to the
   executable, matching its `Executable` name).
2. `Add-AppxPackage -Register <manifest>` gives the in-place process an
   identity without installing or copying anything.

The manifest `Publisher` must match a trusted code-signing cert on the machine.
Create one once and trust it (`CN=ReactorGalleryDev` is the default):

```powershell
$cert = New-SelfSignedCertificate -Type Custom -Subject 'CN=ReactorGalleryDev' `
    -KeyUsage DigitalSignature -KeyAlgorithm RSA -KeyLength 2048 `
    -CertStoreLocation 'Cert:\CurrentUser\My' -NotAfter (Get-Date).AddYears(10)
$store = New-Object System.Security.Cryptography.X509Certificates.X509Store('Root', 'CurrentUser')
$store.Open('ReadWrite'); $store.Add($cert); $store.Close()
```


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

`Executable` must match the built binary name (`reactor-gallery.exe`);
the Assets folder needs `StoreLogo.png`, `Square150x150Logo.png`,
`Square44x44Logo.png`.
