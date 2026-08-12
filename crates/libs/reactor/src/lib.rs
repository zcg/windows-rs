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
mod app_window;
mod backend;
mod bootstrap;
#[cfg(feature = "canvas")]
mod canvas_bridge;
mod capture;
mod composition;
mod diagnostics;
mod drag;
mod element;
mod engine;
mod fault;
mod generated;
mod hooks;
mod host;
mod interaction;
mod islands;
mod jumplist;
mod notifications;
mod pickers;
mod reconciler;
mod sound;
mod style;
mod widget;
mod widgets;

pub use app::*;
pub use app_window::*;
pub use backend::*;
pub use bindings::AppWindow;
pub use bindings::AppWindowTitleBar;
pub use bindings::AutomationHeadingLevel;
pub use bindings::AutomationLiveSetting;
pub use bindings::BadgeNotificationGlyph;
pub use bindings::Color;
pub use bindings::CommandBarDefaultLabelPosition;
pub use bindings::DispatcherQueuePriority;
pub use bindings::ElementSoundKind;
pub use bindings::ElementSoundPlayerState;
pub use bindings::ElementSpatialAudioMode;
pub use bindings::FlyoutPlacementMode;
pub use bindings::HorizontalAlignment;
pub use bindings::InfoBarSeverity;
pub use bindings::ItemsViewSelectionMode;
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
pub use bindings::TitleBarHeightOption;
pub use bindings::TitleBarTheme;
pub use bindings::TreeViewSelectionMode;
pub use bindings::UIElement;
pub use bindings::VerticalAlignment;
pub use bindings::VirtualKey;
pub use bindings::VirtualKeyModifiers;
pub use bindings::WindowId;
pub use bootstrap::*;
#[cfg(feature = "canvas")]
pub use canvas_bridge::{
    CanvasImageSource, CanvasSwapChain, DrawContext, Invalidator, animated_canvas,
    animated_canvas_with_device, canvas, canvas_invalidated,
};
pub use capture::*;
pub use composition::*;
pub use drag::*;
pub use element::*;
pub use engine::*;
pub use fault::Fault;
pub use hooks::*;
pub use host::*;
pub use interaction::*;
pub use islands::*;
pub use jumplist::*;
pub use notifications::*;
pub use pickers::*;
pub use reconciler::*;
pub use sound::*;
pub use style::*;
pub use widget::*;
pub use widgets::*;
pub use windows_core;
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

/// Run a connected animation between the given element and itself, using the
/// ConnectedAnimationService for the current view. Returns whether the
/// animation started. `key` identifies the animation pair.
pub fn run_connected_animation(key: &str, element: &UIElement) -> Result<bool> {
    let service = bindings::ConnectedAnimationService::GetForCurrentView()?;
    let animation = service.PrepareToAnimate(key, element)?;
    animation.TryStart(element)
}

/// Attach a scrolling source (e.g. a ScrollViewer) to a ParallaxView so its
/// background shifts while the source scrolls.
pub fn parallax_attach_source(parallax: &UIElement, source: &UIElement) -> Result<()> {
    let pv: bindings::ParallaxView = parallax.cast()?;
    pv.SetSource(source)
}
