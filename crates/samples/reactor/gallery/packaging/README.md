# Gallery MSIX packaging

The gallery runs unpackaged with `cargo run -p reactor-gallery`. Packaging exists for the one
feature that needs a package identity: the taskbar badge on the Notifications page. Toasts work
either way.

## Build and install

```powershell
cargo build -p reactor-gallery
powershell -File crates\samples\reactor\gallery\packaging\build-msix.ps1 -Install
```

`build-msix.ps1` stages `cargo`'s profile output, generates placeholder icons, packs with
`MakeAppx`, signs with the `ReactorGalleryDev` certificate, and optionally installs the result.
Drop `-Install` to only produce `packaging\output\ReactorWinUIGallery.msix`.

The signing certificate subject must match `Identity/@Publisher` in `Package.appxmanifest`. The
default publisher `CN=ReactorGalleryDev` matches a development certificate of that name; create a
replacement with `winapp cert generate` or `New-SelfSignedCertificate` and trust it in
`Cert:\CurrentUser\Root` before installing.

## Package layout

The package is self-contained: it carries the whole Windows App Runtime and declares no framework
dependency. Staging is an allow list taken from the executable's directory:

| Copied | Why |
| --- | --- |
| `reactor-gallery.exe` | Application entry point |
| `*.dll` | Windows App Runtime, WebView2 projection, VC runtime |
| `*.pri` | Resource index used by MRT and XAML |
| `<language>\` directories | `.mui` files XAML loads for localized resources |
| `Microsoft.UI.Xaml\` | XAML theme and density resources (`generic.xbf`) |

Cargo's `build`, `deps`, `.fingerprint`, and `incremental` directories, other sample binaries, and
debug symbols are never copied.

## Why the layout is strict

Omitting the language directories or the resource directories produces a package that installs
cleanly, shows a black window, and then dies within a few seconds. The process reaches the first
layout pass, XAML cannot load its localized resources, and the failure surfaces as:

| Signal | Value |
| --- | --- |
| Underlying error | `ERROR_MUI_FILE_NOT_LOADED`, `0x80073B01` |
| Exception | `0xC000027B` (stowed exception) |
| Faulting module | `Microsoft.UI.Xaml.dll`, `FailFastWithStowedExceptions` |

`build-msix.ps1` refuses to pack when `Microsoft.UI.Xaml.dll`, `Microsoft.UI.Xaml.Controls.dll`,
`resources.pri`, the `Microsoft.UI.Xaml` directory, or the language directories are missing, so this
failure cannot be reintroduced silently.

A framework-dependent package (runtime files left out, `PackageDependency` on
`Microsoft.WindowsAppRuntime.2` declared) fails the same way on the tested Insider build: the runtime
loads from the framework, but the packaged process resolves its MUI resources against the
application package directory. Ship the runtime in the package.

## Toast activation

`AppNotificationManager::Register()` needs a COM server in the package manifest. Without it,
registration fails with `No COM servers are registered for this app` (`0x80004005`) and no toast is
shown. `Package.appxmanifest` declares both halves:

| Manifest element | Purpose |
| --- | --- |
| `desktop:Extension Category="windows.toastNotificationActivation"` | Binds the toast to the activator CLSID |
| `com:Extension Category="windows.comServer"` with `com:ExeServer` | Registers the CLSID that Windows activates |

Both `ToastActivatorCLSID` and `com:Class/@Id` take a bare GUID. Adding braces fails manifest
validation with `C00CE169`.

## Taskbar badge rendering

`BadgeNotificationManager::SetBadgeAsCount` takes a value of 1 through 99, but whether Windows 11
paints that number depends on the taskbar icon size. The taskbar sizes the badge from the button's
internal icon height:

| Taskbar icon size | Icon height | Badge rendering |
| --- | --- | --- |
| Small | 16 | A dot, whatever the badge value |
| Default or Large | 24 | The numeric badge or the chosen glyph |

The count reaches the system in both cases; only the rendering changes. `SetBadgeAsCount(5)` and
`SetBadgeAsGlyph(Activity)` are pixel-identical while the taskbar runs small icons, which makes the
badge look broken even though every API call succeeded. Setting the taskbar icon size back to
Default restores the number.

The Notifications page reports whether the API call succeeded, which is the only signal the API
itself provides.
