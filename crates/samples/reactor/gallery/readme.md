# Reactor gallery

This sample is the `windows-reactor` port of the WinUI control gallery. It contains 104
destinations across 12 categories:

- Basic Input
- Collections
- Date and Time
- Design Guidance
- Dialogs and Flyouts
- Layout
- Media
- Menus and Toolbars
- Navigation
- Status and Info
- System
- Text

Run it with:

```powershell
cargo run -p reactor-gallery
```

The unpackaged run reaches every control page. The taskbar badge on the Notifications page is the
one feature that needs a package identity, so it needs the MSIX install:

```powershell
cargo build -p reactor-gallery
powershell -File crates\samples\reactor\gallery\packaging\build-msix.ps1 -Install
```

`packaging/README.md` covers the layout requirements, the signing certificate, and the failure
signatures a packaged launch produces when the payload is incomplete.

The shell uses a generated `TitleBar` and `NavigationView`. The pane lists categories by default.
Select a category to open its card grid, then select a card to open the control page. Search
replaces the category links with matching control destinations. Back navigation follows the
shell's page history.

Each control destination is a component under `src/pages/`. Pages use controlled state for live
examples such as text input, sliders, selection controls, dialogs, progress indicators, and layout
changes. Page-owned state is recreated when a destination is retired and later reopened. Window
state, including the selected theme and backdrop material, belongs to the shell and persists across
page replacement.

For implementation examples, read the compiled page components in [`src/pages/`](src/pages/).
The gallery does not duplicate them as display-only source snippets, which can drift from the
working examples.

The gallery is organized as follows:

| File | Purpose |
| --- | --- |
| `src/shell.rs` | Window visuals, title bar, navigation pane, search, and history |
| `src/registry.rs` | Category and destination metadata |
| `src/router.rs` | Leaf destination to component mapping |
| `src/controls.rs` | Shared page headers, sample cards, card grids, and assets |
| `src/pages/` | Category modules and control page components |

Bundled images in `assets/` are loaded through `Image::source_file`. Sample cards use WinUI theme
brushes so their surfaces follow the selected light, dark, or system theme. The Materials page
changes the live window backdrop between Mica, Mica Alt, Acrylic, and a solid background.

The gallery uses generated `GridView` selection feedback, `BreadcrumbBar` item-click events, and
the `Viewbox` child slot. Flyout, SplitButton, TabView, and InfoBar examples use their native event
and ownership contracts rather than substituting pointer handlers.

The Camera page is the one destination that needs the MSIX install: `MediaCapture` resolves
through the package identity, and the packaged launch is what raises the per-app camera prompt.
Windows 11 builds that ship that prompt default the answer to "No", so the page cannot show frames
until it is allowed under Settings > Privacy & security > Camera.

The SwapChainPanel page hosts a Direct2D surface through the `windows-canvas` Reactor integration:
the panel publishes its element reference, the integration creates the swap chain through
`ISwapChainPanelNative::SetSwapChain`, and each composition frame redraws the surface.

Recording-runtime tests mount the full gallery, replace pages through shell navigation, mount every
registered destination, drive representative controlled input, reject events from retired pages,
and verify that shell-owned theme and backdrop state survives page replacement.
