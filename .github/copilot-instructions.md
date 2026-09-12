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
# Regenerate codegen (after editing winui.toml or tool-reactor source)
cargo run -p tool-reactor --quiet

# Regenerate bindings (after editing filter .txt files)
cargo run -p tool-bindings --quiet

# Verify reactor compiles
cargo check -p windows-reactor --quiet

# Unit tests (headless, fast)
cargo test -p test_reactor --quiet

# Integration tests (launches WinUI window)
cargo run -p test-reactor-selftest
cargo run -p test-reactor-selftest -- --headless    # CI mode
cargo run -p test-reactor-selftest -- --filter Name  # single fixture

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

1. **`tool-reactor`** - reads `crates/tools/reactor/src/winui.toml` + WinUI `.winmd` metadata ->
   generates `generated.rs`, `generated_set_prop.rs`, `generated_attach_event.rs`, and
   `generated.txt` filter entries.

2. **`tool-bindings`** - reads filter `.txt` files from `crates/tools/bindings/src/` -> runs
   `windows-bindgen` -> generates `bindings.rs` in each crate:
   - `crates/libs/canvas/src/bindings.rs` (from `canvas.txt`)
   - `crates/libs/time/src/bindings.rs`, `numerics`, `reference`, etc.

3. **`tool-package`** - generates the published `windows` and `windows-sys` package crates using
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

- WinUI backend is in `crates/libs/reactor/src/backend/winui/`.
- The TOML config (`winui.toml`) declares ~52 WinUI controls. Keys are WinUI metadata names; the
  tool infers types, setter patterns, and event handlers from `.winmd` files.
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
- **No `thread_local!` in app code** - use reactor hooks (`use_state`, `use_ref`) instead.
  `thread_local!` is reserved for framework plumbing.
- **Test naming**: Unit tests in `test_reactor`, integration tests in `test-reactor-selftest`.
  Canvas tests use WARP software rendering.

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

### windows-clang: editorializing beyond the header

Context: issue [#4720](https://github.com/microsoft/windows-rs/issues/4720) and discussion
[#4725](https://github.com/microsoft/windows-rs/issues/4725) (the `SW_NORMAL` should-be-signed
report). The scrape aims to match the SDK headers, not a theoretical C-standard purity,
yet a few rules still rewrite or drop what the header literally declares. The fundamental-type
canonicalization (`DWORD`/`CHAR` -> primitives, `BOOLEAN` -> `bool`,
`LARGE_INTEGER`/`ULARGE_INTEGER` -> `i64`/`u64`, the D2D numerics) is settled and intended; the
items below are the ones still worth revisiting. Sources:
`crates/libs/clang/src/{const,canon,annotation,interface,lib}.rs`,
`crates/tools/win32/src/main.rs`, `docs/crates/windows-clang.md`.

**Active type rewrites - close to the header, but restate the declared type:**

- **Scalar collapse is a curated allowlist** (`canon.rs`: `fundamental_scalar`, `semantic_scalar`,
  `pointer_sized_abi`, `floating_typedef`, `guid_alias`, `void_pointer_alias`, `d2d_compat_alias`).
  `DWORD` -> `u32` - and now `BOOLEAN` -> `bool`, `LARGE_INTEGER` -> `i64` - erase the named alias,
  while `COLORREF`/`ATOM`/`HFILE` (byte-identical underlying types) stay named. The split is
  editorial. Fix: emit every typedef as a named alias in the winmd and move the collapse to the
  Layer-B `windows-bindgen` projection, where it is an ergonomics choice, not a metadata fact.
- **String-alias normalization + SAL const-flip** (`canon.rs`: `normalize_string_alias`,
  `apply_sal_constness`, `promote_null_terminated_string`). `LPCWSTR` -> `PCWSTR`, and `_In_ LPWSTR`
  -> `PCWSTR` flips a mutable pointer to const from the `_In_` bit. Declared type is mutable; the
  emitted type is const. Fix: keep the header's pointer/const-ness; expose SAL direction as a
  separate attribute.
- **`[iid_is]` inferred from parameter name** (`annotation.rs` `infer_iid_is`,
  `IID_SELECTOR_PARAM_NAMES = ["riid","iid","riidltf"]`). An un-annotated `_COM_Outptr_ IUnknown**`
  becomes `void**` + `[iid_is]` because a sibling parameter is named `riid` (5 methods across 4
  functions). The header never expressed the linkage. Fix: a "source-annotations-only" mode that
  honors only an explicit MIDL `[iid_is]` comment.
- **`D2D1_*` -> `D2D_*` compat collapse** (`canon.rs` `d2d_compat_alias`) - curated erasure of the
  `D2D1_`-spelled alias the header declares. Same remedy as the scalar collapse.

**Dropped / reshaped header content - coverage opinions:**

- **Redundant-constant dropping** (`lib.rs` final pass) - drops a top-level constant whose name and
  value match an enumerator elsewhere. Only needed because of the flat namespace below.
- **Single flat `Windows.Win32` namespace** - lossy for genuine name collisions the reference
  disambiguated by sub-namespace (`PID_SECURITY`, the `E_NOTFOUND` HRESULT-vs-`#define` class). Fix:
  on a true USR/value collision, keep both under a disambiguating suffix instead of dropping.
- **`UNICODE`/`_UNICODE` not defined** - the TU is built ANSI-default. Higher-coverage (defining
  `UNICODE` drops 71 bare-ANSI exports) but the generic-text typedefs follow the ANSI branch.
  Fix (large, what the reference does): scrape ANSI and Unicode in two passes and merge.
- **Orphan named-type dropping** - a type an in-scope header declares but no emitted signature
  references is dropped (`PROCESSOR_POWER_INFORMATION`, `FIRMWARE_TABLE_PROVIDER`,
  `PROCESSOR_FEATURE_ID`). Fix: emit all named types defined in a `HEADERS` file, not only the
  reachability closure.
- **`intsafe.h` exclusion, `drop_lib_less`, `vertdll` ordering** (`tool-win32`) - pragmatic drops
  and relinks of content the headers/libs provide. Low priority, defensible.

**Correct as-is (do not "fix"):** overloaded-virtual vtable reversal (`interface.rs`, reproduces the
true MSVC vtable slot order - ABI-critical); the `_HRESULT_TYPEDEF_`/`_NDIS_ERROR_TYPEDEF_`
cast-wrapper map (`const.rs` `cast_wrapper_macro`, honors an explicit author type annotation).

Suggested order: `UNICODE` two-pass (largest fidelity win), then the `[iid_is]` name-gate,
orphan-type retention, the scalar/string/D2D collapses (move to Layer B), then flat-namespace
disambiguation.

### windows-clang: suppress-definition coupling

`lib.rs` (`StructDecl`/`UnionDecl`) and `typedef.rs` consult the `canon.rs` collapse views
(`numerics_alias`, `semantic_scalar`, the scalar/guid/void views) to skip emitting a collapsed
type's definition, mirroring the reference-site collapses in `alias_collapse`. The name lists are
not duplicated (both sides read the shared table), but nothing guarantees a collapse row and its
suppression site stay paired. A suppress-definition column on `Collapse` carrying the cursor kind
(typedef/struct/union) each row suppresses would make the pairing a compiler-checked fact.

Output-neutrality check for any clang change: regenerate the two scrape consumers - `tool-win32`
(flat, `write_by_header`, both the um and km scrapes) and `tool-webview` (namespaced, `write`) - and
confirm `git diff` shows no generated-file changes. `tool-bindings`/`tool-package`/`tool-features`
derive from the winmds, so an unchanged winmd proves them unchanged too.

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
