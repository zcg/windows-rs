use super::*;
use app_shim::{create_application, install_theme_resources, install_xaml_controls_resources};
use bindings::IElementFactory;
use transient_menu::{MenuTheme, TransientMenuHost};
use windows_collections::*;
use windows_core::{
    ComObject, Event as WinEvent, HRESULT, HSTRING, IInspectable, IUnknownImpl, Interface, Ref,
    implement_decl,
};

// Global font scaling.
//
// A host that wants one font-size factor for the whole app (a settings
// "text size" slider) cannot set it per element: `TextBlock::font_size` and
// friends are declared on every call site, and WinUI exposes no host-level
// default. This multiplier is applied to every font size (and to explicit
// `TextBlock` line heights) as the property is written to the native control,
// so one dispatch point covers the library. Fixed-pixel layouts derived from a
// font size (icon boxes, wrap widths) do not scale with it; callers that need
// them to track must read `font_scale` themselves.
//
// The store is the bit pattern of an `f64` in an `AtomicU64` because std has no
// `AtomicF64`; `to_bits`/`from_bits` round-trips losslessly.
static FONT_SCALE_BITS: AtomicU64 = AtomicU64::new(1.0f64.to_bits());

/// Current global font-size scale factor (`1.0` keeps each element's own size).
///
/// Read on every font-size property write, so this is a lock-free atomic load.
#[inline]
pub fn font_scale() -> f64 {
    f64::from_bits(FONT_SCALE_BITS.load(Ordering::Relaxed))
}

/// Sets the global font-size scale factor.
///
/// Returns `false` for invalid input (the factor must be a finite positive
/// number) and leaves the previous value in place, so a caller can report the
/// problem instead of silently rendering at `1.0`.
///
/// The factor is global state, not a per-window declaration: existing controls
/// pick it up the next time the library writes their font-size properties.
pub fn set_font_scale(scale: f64) -> bool {
    if !(scale.is_finite() && scale > 0.0) {
        return false;
    }
    FONT_SCALE_BITS.store(scale.to_bits(), Ordering::Relaxed);
    true
}

#[allow(
    clippy::missing_transmute_annotations,
    clippy::upper_case_acronyms,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    // The merged filter intentionally covers more controls than this build references.
    dead_code
)]
mod bindings;

mod app;
mod app_shim;
mod bootstrap;
mod notifyicon;
mod transient_menu;
mod winui;

pub use app::*;
pub use winui::*;

struct PhysicalCoordinates(bindings::DPI_AWARENESS_CONTEXT);

impl PhysicalCoordinates {
    fn enter() -> windows_core::Result<Self> {
        let previous = unsafe {
            bindings::SetThreadDpiAwarenessContext(
                bindings::DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
            )
        };
        if previous.is_null() {
            return Err(windows_core::Error::from_thread());
        }
        Ok(Self(previous))
    }
}

impl Drop for PhysicalCoordinates {
    fn drop(&mut self) {
        unsafe {
            _ = bindings::SetThreadDpiAwarenessContext(self.0);
        }
    }
}
