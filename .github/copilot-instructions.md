# Copilot Instructions for windows-rs

Read this file at the start of every session. It contains the essential commands, conventions, and
architecture knowledge needed to work on this repository.

## Workflow

Do not create git commits automatically. The maintainer reviews all changes first and handles
commits manually. Make changes locally, run fmt/clippy/tests to verify, and report back.

## Repository Layout

Cargo workspace (`resolver = "3"`). Members are globbed from:

- `crates/libs/*` - the published/library crates (`windows`, `windows-sys`, `windows-core`, plus
  `windows-bindgen`, `metadata`, `rdl`, and the newer `reactor`/`canvas`/`webview`/`window` crates).
  See `docs/readme.md` for the full categorized crate index, and `docs/crates/<crate>.md` per crate.
- `crates/tools/*` - code generators and CI helpers, run via `cargo run -p tool-*`.
- `crates/tests/*/*` - test crates; `crates/tests/libs/<crate>` mirrors each library crate (e.g.
  `test_reactor`, `test-webview`). Crate names are `test_<dir>`.
- `crates/samples/*/*` - runnable examples.

The crates fall into rough groups (see `docs/readme.md` for the authoritative list): core & errors
(`windows-core`, `windows-result`, `windows-strings`); values & collections (`numerics`,
`collections`, `reference`, `time`); async & threading (`future`, `threading`); system services
(`registry`, `services`, `version`); COM macros & linking (`implement`, `interface`, `link`); UI &
graphics (`reactor`, `canvas`, `webview`, `window`, `animation`, `reactor-setup`); codegen &
metadata tooling (`bindgen`, `metadata`, `rdl`, `cppwinrt`); and the full API projection (`windows`,
`windows-sys`).

## Before Finalizing Changes

```sh
cargo fmt --all
```

CI enforces rustfmt. Always format before finalizing changes. On Windows, the workspace-wide
command may exceed the process command-line limit; use `cargo fmt -p <crate>` for each affected
crate instead.

## Build & Test Commands

### Core crates

```sh
cargo check -p windows-core --quiet
cargo check -p windows --quiet
cargo clippy -p <crate> --all-targets
cargo test -p <crate>

# Run a single test by name filter
cargo test -p <crate> <test_name_substring>
```

CI sets `RUSTFLAGS: -D warnings`, so any warning fails the build. Workspace-wide lints are
configured in the root `Cargo.toml` (`[workspace.lints]`) - clippy lints like
`uninlined_format_args`, `redundant_clone`, and `semicolon_if_nothing_returned` are promoted to
warnings and therefore enforced.

### Reactor

```sh
# Refresh WinUI metadata after changing Windows App SDK or WebView2 pins
cargo run -p tool-reactor-metadata --quiet

# Regenerate Reactor code after editing schema.toml or tool-reactor source
cargo run -p tool-reactor --quiet

# Regenerate bindings (after editing filter .txt files)
cargo run -p tool-bindings --quiet

# Verify reactor compiles
cargo check -p windows-reactor --quiet

# Unit tests (headless, fast)
cargo test -p windows-reactor --all-features --quiet

# Integration tests (launches WinUI window)
cargo run -p test-reactor-selftest
cargo run -p test-reactor-selftest -- --headless    # CI mode
cargo run -p test-reactor-integration --bin application_menu
cargo run -p test-reactor-integration --bin canvas_integration
cargo run -p test-reactor-integration --bin webview_integration
cargo run -p test-reactor-integration --bin window_state

# Clippy
cargo clippy -p windows-reactor --all-targets
```

### Canvas

```sh
cargo check -p windows-canvas --quiet
cargo test -p windows-canvas --quiet
cargo clippy -p windows-canvas --all-targets
```

### Full workspace

```sh
cargo run -p tool-clippy-all    # runs clippy across all crates
```

## Code Generation Pipeline

**Never hand-edit generated files.** Generated outputs are committed, and CI fails if regenerating
produces a diff (the `gen` workflow runs each `cargo run -p tool-*` and rejects any change; the
`test` workflow likewise fails if tests modify tracked files). After editing generators or filters,
re-run the tool and commit the result.

The core `windows` / `windows-sys` crates are generated from Windows metadata (`.winmd`) via
`windows-bindgen` (driven by `tool-package`). `windows-metadata` and `windows-rdl` support
reading/authoring that metadata. The reactor / canvas / webview pipelines layer on top:

1. **`tool-reactor-metadata`** - refreshes committed WinUI / Windows App SDK / WebView2 `.winmd`
   metadata and generates `extras.winmd`.

2. **`tool-reactor`** - reads `crates/tools/reactor/src/schema.toml` plus the shared WinUI
   metadata -> generates Reactor declarations, native projection code, the Reactor native
   bindings, and the canvas Reactor bridge bindings.

3. **`tool-bindings`** - reads filter `.txt` files from `crates/tools/bindings/src/` -> runs
   `windows-bindgen` -> generates `bindings.rs` in each crate:
   - `crates/libs/canvas/src/bindings.rs` (from `canvas.txt`)
   - `crates/libs/time/src/bindings.rs`, `numerics`, `reference`, etc.

4. **`tool-package`** - generates the published `windows` and `windows-sys` package crates using
   `--package` mode (per-namespace files + Cargo.toml features).

4. After regenerating, always verify: `cargo check -p <affected-crate> --quiet`

## Key Architecture Facts

### Crate relationships

- `windows-core` is the foundation - almost everything depends on it.
- `windows` is the umbrella crate that re-exports from `windows-core`, `windows-numerics`,
  `windows-time`, `windows-collections`, `windows-reference`, etc.
- `windows-reactor` depends on `windows-core` (not `windows`) and uses minimal bindings generated
  with `--minimal --flat` mode.
- `windows-canvas` similarly uses minimal bindings for D2D/DXGI/DWrite/WIC.
- `windows-animation` wraps Win32 UIAnimation Manager COM APIs.

### Reactor architecture

- WinUI backend ownership is split across `crates/libs/reactor/src/native/winui.rs` and
  `crates/libs/reactor/src/native/winui/`.
- The TOML config (`crates/tools/reactor/src/schema.toml`) declares 79 WinUI controls. Keys are
  WinUI metadata names; the tool infers types, setter patterns, and event handlers from `.winmd`
  files.
- COM casts: classes Deref to their default interface (zero-cost). Only cast to non-default parent
  interfaces. The `Param` trait handles parent-class conversions automatically.

### Canvas architecture

- Canvas wraps D2D/DXGI behind safe Rust types (`GpuDevice`, `SwapChain`, `DrawingSession`,
  `PathBuilder`).
- `animated_canvas()` (reactor feature) renders on UI thread via `CompositionTarget::Rendering`.
- Device-lost recovery is automatic.

## Conventions

- **Panics**: Use `panic!` only for invariant violations. Use `diag::` helpers for missing features
  (warn in debug, no-op in release).
- **`.unwrap()` over `.expect("...")`** - the panic hook provides full context.
- **No `thread_local!` in app code** - keep state in components and pass shared values through typed
  contexts. `thread_local!` is reserved for framework plumbing.
- **Test naming**: Reactor unit tests live in `windows-reactor`; live WinUI coverage lives in
  `test-reactor-selftest`. Public application and companion-crate fixtures live in
  `test-reactor-integration` without the Reactor `test` feature. Build that package separately
  from privileged tests to avoid Cargo feature unification. Canvas tests use WARP rendering.

## Documentation

The `docs/` folder has one page per crate:

- **`docs/crates/<crate>.md`** - a single page per crate covering both usage and internals (how the
  crate is built and maintained: codegen pipeline, generated files, conventions). It links to the
  crate's own `readme.md` for the user-facing intro and quick example.
- **`crates/libs/<crate>/readme.md`** - the user-facing introduction with a quick example (also the
  crates.io / docs.rs landing).
- **`docs/readme.md`** - the documentation hub and crate index.

`docs/` also holds `contributing.md`, `code_of_conduct.md`, and `security.md`.

When making changes to a crate, update its `docs/crates/<crate>.md` page and its `readme.md` as
needed. For example, `windows-reactor` changes touch `docs/crates/windows-reactor.md` (codegen,
TOML, threading, COM pitfalls, plus the conceptual overview) and `crates/libs/reactor/readme.md`
(getting started and the quick example).

## Writing Style for Docs and Comments

These rules were established while cleaning up the docs and code comments and apply to all Markdown
(`.md`) and Rust comments/doc-comments across the repo. Keep new writing consistent with them.

- **ASCII punctuation only.** Use `-` for dashes (never em/en-dashes), `...` for ellipsis, `->` for
  arrows, `<=`/`>=`/`!=` for comparisons, and straight quotes. Drop the section sign from standard
  references (write `ECMA-335 II.22`, `C11 6.4.4.1`, not `\u00a7...`). The only non-ASCII that stays
  is genuine test data (e.g. a Greek-letter string literal exercising UTF-8 handling).
- **100-column wrap.** Hard-wrap Markdown prose and long comment blocks at 100 columns. Keep the
  wrap consistent within a file - do not mix wrapped and unwrapped paragraphs.
- **No LLM tells.** Avoid the vocabulary and tics that mark agent-written text: `faithful`,
  `corpus`, `ledger`, `crucially`, `notably`, `essentially`, `robust`, `seamless`, `comprehensive`,
  `deliberately`, `simply`, `under the hood`, `that said`, `importantly`, `conceptually`,
  `effectively`, `single source of truth`, `industry-standard`, `first-class`, `leverage`,
  `utilize`, `Note that ...`. Say the concrete thing instead. `ergonomic` and rustdoc `**bold**` are
  fine.
- **No decorative formatting.** No box-drawing banner comments (`// -- Title ------`), no duplicated
  doc paragraphs (a real copy-paste tell), no filler that restates the code.
- **Prefer tables over wordy paragraphs** where the content is a set of parallel cases (rules,
  mappings, options).
- **Describe the code as it is,** not its history. Avoid churn narration (`used to`, `previously`,
  `an earlier version`, `no longer`) unless it describes real runtime behavior, not codebase edits.
- **Do not edit generated files** to satisfy these rules; only hand-written sources.

## Open Investigations / TODO

Enduring record of known issues so they are not lost between sessions. Add findings here; remove or
mark done as they are addressed. Cite code by file + symbol name, not line number - line numbers go
stale fast.

### Repo-wide dead-code / quality audit (2026-07)

Open items across the hand-written crates (reactor, bindgen, rdl, clang, canvas, metadata, webview,
core) that need a design decision or a larger change. Any bindgen source change must be proven
output-neutral by running the `tool-*` generators and confirming `git diff` shows no generated-file
changes (the `gen` workflow enforces this).

#### Duplication / refactor candidates

| Location | Issue |
| --- | --- |
| `metadata` `merge/mod.rs` `write_type` vs `merge/remap.rs` `write_type` | Two ~60-line structurally identical functions; the remap copy's comment says it mirrors `merge::write_type`. Any new ECMA table must be added to both, with no compiler guard. |
| `canvas` `session.rs` gradient-stop + bitmap-properties builders | Duplicated ABI-stop and bitmap-properties construction across the brush paths. |
| `bindgen` `types/interface.rs` + `cpp_interface.rs` local `fn combine` | Duplicate local helper. |
| `canvas` `color.rs` `DARK_SLATE_BLUE` | `rgb(0.05, 0.05, 0.1)` does not match the CSS color of that name (public API used by samples). |

#### Coverage gaps

| Area | Gap |
| --- | --- |
| `metadata` `Remapper` (`merge/remap.rs`) | No tests anywhere; routing/`split_apis` logic is exercised only in the live build, so a regression yields a malformed namespaced winmd with no failing test. |
| webview | `process-failed`, download, and deferral paths untested. |
| `reactor` `items_view` widget + Windows App SDK 2.3.1 | Setting `ItemsView.ItemTemplate` to any `IElementFactory` (reproduced with an official `DataTemplate`) crashes the runtime during layout (`0xC000027B`). The widget intentionally leaves `ItemTemplate` unset; the gallery demonstrates the data-driven model through `ItemsRepeater`. Revisit when the runtime is fixed or upgraded. |

### gallery: packaged launch renders a black window on Insider builds (2026-09)

Resolved. On a Windows Insider build 26340 host, a packaged gallery launch
showed a black window and died while the identical unpackaged executable ran
the whole gallery. Provisioning changes (removing the Windhawk DLL injection,
adding the `Main.2`/`DDLM.2` satellite packages, re-running the official 2.4
runtime installer elevated) moved the failure from an instant `CoreMessagingXP`
fail-fast (`0xC0000602`, caused by shipping a second copy of the runtime beside
a framework-dependent executable) to a XAML stowed exception at the first
layout pass (`0xC000027B`, `FailFastWithStowedExceptions`, source
`dxaml/xcp/components/base/ErrorContext.cpp`).

The remaining cause was an incomplete package payload, not activation
registration. Two gaps mattered: `resources.pri` copied from the runtime package
carries the resource map name `Microsoft.WindowsAppRuntime.2` instead of the
application package name, and the language directories holding
`Microsoft.ui.xaml.dll.mui` were not staged at all. A packaged process resolves
localized resources against the application package, so either gap ends in
`ERROR_MUI_FILE_NOT_LOADED` (`0x80073B01`), a stowed exception, and a fail-fast.
The earlier `CLASS_E_NOAGGREGATION` reading came from the framework-dependent
layout, where both the module path and the error context pointed at the
application package; it was not a missing `windows.activatableClass`
declaration.

Packaging the full self-contained layout works: every runtime DLL, every `*.pri`,
all language directories, and `Microsoft.UI.Xaml\` beside the executable.
Verified on build 26340 with a package that launches from the Start menu,
renders, navigates, sends a toast, and sets a taskbar badge.
`crates/samples/reactor/gallery/packaging/` holds the manifest, `build-msix.ps1`
(which refuses to pack an incomplete layout), and a README recording the failure
signatures. Removing the language or resource directories from that payload
reproduces the crash, which is the regression check for this fix.

Two platform facts recorded while verifying. A packaged app needs
`windows.toastNotificationActivation` plus a `windows.comServer` declaration
before `AppNotificationManager::Register()` succeeds; without them registration
fails with "No COM servers are registered for this app" (`0x80004005`).

A taskbar badge looks like a dot rather than a number when the taskbar icon size
is set to Small. Windows sizes the badge from the button's internal icon height:
16 (Small) paints a dot for any badge value, 24 (Default or Large) paints the
numeric badge. `SetBadgeAsCount(5)` and `SetBadgeAsGlyph(Activity)` are
pixel-identical under Small, and the value is then only visible through UI
Automation, whose HelpText reports a status item count. The Windhawk
`taskbar-icon-size` mod masks this by forcing icon height 24 while it hooks
`TaskListButton::UpdateBadge`, so numbers reappear whenever Windhawk is
injected; its source comments the 16-versus-24 behaviour. Set the taskbar icon
size back to Default to get numbers without third-party injection.

### gallery: camera preview appears black on a packaged launch (2026-09)

Resolved as a camera-side condition, not a rendering bug. The Camera page runs
`MediaCapture` in shared-readonly video mode, wraps the preview frame source with
`MediaSource::CreateFromMediaFrameSource`, and hands the result to a declarative
`MediaPlayerElement`. The preview does update: two frames captured 2.5 seconds
apart differ across 1.86 percent of the preview pixels, and multiplying that
region by 7 exposes live sensor noise and scene content around a single bright
pixel. The snapshot path agrees, writing a 1280x720 JPEG whose contents match the
preview, and the shell renders it through `Image::source_file`.

The frames are black because the device emits black frames on that host. The
Windows Camera app shows the same black preview with the same single bright pixel
while the gallery is closed and the device is free, and the enclosure resolves as
`USB Camera` in the `Camera` device class with status `OK`.

Two platform facts came out of the check. Build 26340 raises a per-application
camera prompt the first time a packaged app touches the device, and the prompt
defaults to No; answering No leaves `MediaCapture` initialized with no frames
until the answer changes under Settings > Privacy & security > Camera. Reinstalling
the package raises the prompt again. The taskbar badge aside, this is the second
gallery feature whose behaviour depends on package identity.

`MediaPlayerElement::source` accepts a `windows_core::IUnknown` rather than a URI
string, so a caller passes either `MediaSource::CreateFromUri` or a camera frame
source and the WinUI backend assigns `MediaPlayerElement.Source` directly. The
`media_source` adapter in `winui.toml` therefore declares no filter entries of its
own; adding `MediaSource::CreateFromUri` back to the generated filter reintroduces
an unused-function warning, because nothing in the library calls it.

### Upstream third-preview merge: local feature ports left for follow-up (2026-10)

The local branch's reactor work was merged onto the upstream reactor rewrite. The widget
coverage carries over (21 widgets are now declared in `crates/tools/reactor/src/schema.toml`),
but the items below did not fit the rewritten architecture in the same pass and are
intentionally absent. Old implementations live in history (`git show HEAD~1:<path>` on the
`backup/pre-merge-20261009` branch). Each needs a proper port rather than a copy:

| Area | What is missing | Old home |
| --- | --- | --- |
| Text | `font_family` property (TextBlock/RichTextBlock/NavigationView), per-name FontFamily cache, global `font_scale` | `native/winui/mod.rs`, `generate_winui.rs` |
| Image | `image_decode_width` downsample decode | `generate.rs` |
| Border | `implicit_translation` / `implicit_translation_transition` | `generate_winui.rs` |
| Events | `value_field` projection (Slider.SizeChanged width) and `FieldAsF64` conversion | `schema.rs`, `generate_winui.rs` |
| Widgets | `media_source` (MediaPlayerElement.source), `items_view_layout` (ItemsView.layout), `map_center` (MapControl.center), `refresh_deferral` (RefreshContainer.on_refresh_requested) | `generate.rs`, `generate_winui.rs` |
| Tree/Time | `tree_node_content` (TreeView), `clock_identifier` (TimePicker) | `schema.rs` |
| Window | Work-area queries (`WorkArea`, `primary_work_area`, `primary_work_area_dip`, `window_placement_dip`, `work_area_for_window`); the file `native/winui/window_geometry.rs` is kept but not wired into the module tree | `window_geometry.rs` |
| App | `install_theme_resources` exists in `native/app_shim.rs` but the new App startup has no application-resource hook | `app/mod.rs` |
| Backdrop | `WindowBackdrop::{AcrylicThin, Solid}` and `WindowBackdropTuning` (composition-controller path); the merged enum has `None/Mica/MicaAlt/Acrylic` | `native/winui/mod.rs` |
| Reference | `ScrollOffsets`, `ElementRef::observe_scroll_offsets`, `ScrollViewer::on_view_changed` | `reference.rs` |
| Menus | `MenuItem::{radio, toggle}` (RadioMenuFlyoutItem/ToggleMenuFlyoutItem) | `schema.rs` |
| Swipe | `SwipeControl` swipe items (`SwipeItem`/`SwipeItems`) | `schema.rs`, `generate.rs` |

The resolver extensions those adapters relied on already live in
`crates/tools/reactor-metadata/src/lib.rs` (`value_fields`, `class_methods`,
`ReadValueConversion::FieldAsF64`, `resolve_event_args_property_field`,
`resolve_event_args_method`, and the `EventHandler<T>` generic-arity name fix), so each
port is mostly generator + schema work.
