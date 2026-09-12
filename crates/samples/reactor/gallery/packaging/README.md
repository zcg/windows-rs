# Packaging Reactor WinUI Gallery (MSIX)

Development runs unpackaged: `cargo run -p reactor-gallery` starts
self-contained (Windows App Runtime files beside the executable) and the
startup does no packaging work. The MSIX flow is a separate, manual step for
when the packaged identity features are being tested.

## Taskbar badge and toasts

The Notifications page uses the official Windows App SDK APIs - the badge
through `Microsoft.Windows.BadgeNotifications.BadgeNotificationManager` and
toasts through `AppNotificationManager` - the same ones the official WinUI
Gallery uses. Toasts work for any desktop process. The badge manager is a
packaged-only API: without the install it reports an error, and the page
shows a banner pointing at the flow below.

## Test the packaged flow

One command packs the current build, signs it, replaces any previous
registration, and installs:

```powershell
powershell -ExecutionPolicy Bypass -File `
    crates/samples/reactor/gallery/packaging/build-msix.ps1 -Install
```

The script signs with the `CN=ReactorGalleryDev` certificate, which must
exist in the user certificate store and in Trusted Root (created once with
`New-SelfSignedCertificate`; it already exists on the development machine).

Then launch **Reactor WinUI Gallery** from the Start Menu (a direct
executable launch stays unpackaged - only the Start Menu entry carries the
package identity). The MSIX is self-contained: the script stages every
Windows App Runtime file the build ships, and the manifest declares no
framework dependency, so the packaged process runs from the files inside the
package on any system, with or without a Windows App SDK install. Do not add
a `<PackageDependency>`: a framework in the package graph alongside the
packaged files makes the process load two runtime copies, which fail-fasts
in CoreMessagingXP.


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
