#![doc = include_str!("../readme.md")]
#![allow(missing_docs)]

#[allow(
    non_snake_case,
    non_upper_case_globals,
    non_camel_case_types,
    dead_code,
    clippy::upper_case_acronyms,
    clippy::missing_transmute_annotations
)]
mod bindings;

mod app;
mod app_shim;
mod backend;
mod bootstrap;
#[cfg(feature = "canvas")]
mod canvas_bridge;
mod diagnostics;
mod drag;
mod element;
mod engine;
mod fault;
mod generated;
mod hooks;
mod host;
mod interaction;
mod reconciler;
mod style;
mod widget;
mod widgets;

pub use app::*;
pub use backend::*;
pub use bindings::AutomationHeadingLevel;
pub use bindings::AutomationLiveSetting;
pub use bindings::Color;
pub use bindings::CommandBarDefaultLabelPosition;
pub use bindings::DispatcherQueuePriority;
pub use bindings::FlyoutPlacementMode;
pub use bindings::HorizontalAlignment;
pub use bindings::InfoBarSeverity;
pub use bindings::NavigationViewDisplayMode;
pub use bindings::NavigationViewPaneDisplayMode;
pub use bindings::Orientation;
pub use bindings::PasswordRevealMode;
pub use bindings::RefreshPullDirection;
pub use bindings::ScrollBarVisibility;
pub use bindings::ScrollingScrollBarVisibility;
pub use bindings::Stretch;
pub use bindings::Symbol;
pub use bindings::TeachingTipPlacementMode;
pub use bindings::TextTrimming;
pub use bindings::TextWrapping;
pub use bindings::Thickness;
pub use bindings::TreeViewSelectionMode;
pub use bindings::VerticalAlignment;
pub use bindings::VirtualKey;
pub use bindings::VirtualKeyModifiers;
pub use bootstrap::*;
#[cfg(feature = "canvas")]
pub use canvas_bridge::{
    CanvasImageSource, CanvasSwapChain, DrawContext, Invalidator, animated_canvas,
    animated_canvas_with_device, canvas, canvas_invalidated,
};
pub use drag::*;
pub use element::*;
pub use engine::*;
pub use fault::Fault;
pub use hooks::*;
pub use host::*;
pub use interaction::*;
pub use reconciler::*;
pub use style::*;
pub use widget::*;
pub use widgets::*;
pub use windows_core::{Error, Interface, Result};
pub use windows_time::{DateTime, TimeSpan};

/// Copy text to the system clipboard. Returns an error if the clipboard
/// is unavailable or the operation fails.
pub fn clipboard_set_text(text: &str) -> Result<()> {
    let package = bindings::DataPackage::new()?;
    package.SetText(text)?;
    bindings::Clipboard::SetContent(&package)?;
    bindings::Clipboard::Flush()
}

/// Read the current text from the system clipboard, if it contains text.
pub fn clipboard_get_text() -> Option<String> {
    let view = bindings::Clipboard::GetContent().ok()?;
    let operation = view.GetTextAsync().ok()?;
    operation.GetResults().ok().map(|h| h.to_string_lossy())
}
