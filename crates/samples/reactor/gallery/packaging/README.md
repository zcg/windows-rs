# Packaging Reactor WinUI Gallery (MSIX)

The gallery runs packaged: `cargo run -p reactor-gallery` packs the build
output into this MSIX, installs it (first run only - an installed package with
the current build is activated as-is) and relaunches through
`shell:AppsFolder\<PFN>!App` so the process carries a package identity. When
the executable is newer than the packed MSIX (any code change), the package is
built and installed again; a plain `cargo build` without changes keeps the
fast activate-only path.

## Taskbar badge and toasts

The Notifications page uses the official Windows App SDK APIs - the badge
through `Microsoft.Windows.BadgeNotifications.BadgeNotificationManager` and
toasts through `AppNotificationManager` - the same ones the official WinUI
Gallery uses. Both require the packaged identity that the flow above
provides; there is no self-drawn fallback.

The MSIX is framework-dependent: `build-msix.ps1` stages no Windows App
Runtime files, and the manifest declares the `Microsoft.WindowsAppRuntime.2`
dependency. Shipping self-contained runtime files in a packaged process
fail-fasts in CoreMessagingXP.

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

`packaging/build-msix.ps1` does everything `cargo run` triggers on demand:
it stages the manifest, executable, and non-framework files, generates the
placeholder logo assets, packs with MakeAppx, and signs with the
`ReactorGalleryDev` certificate:

```powershell
powershell -ExecutionPolicy Bypass -File packaging/build-msix.ps1
Add-AppxPackage -Path packaging/output/ReactorWinUIGallery.msix
```

`Executable` must match the built binary name (`reactor-gallery.exe`); the
generated Assets folder provides `StoreLogo.png`, `Square150x150Logo.png`,
`Square44x44Logo.png`.
