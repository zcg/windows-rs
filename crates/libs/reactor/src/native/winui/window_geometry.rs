// Window geometry measured at runtime: the monitor work area, and where a window currently
// sits and how large it is.
//
// Hand-written layer. `bindings.rs` is entirely generated (the winmd generator under
// `crates/tools/reactor` rewrites it), so the Win32 entry points introduced here are declared
// in this file and survive a binding regeneration.
//
// Why Win32 instead of `Microsoft.UI.Windowing.DisplayArea`: the bindings carry no
// `DisplayArea` (`winui.toml` takes controls and a few interfaces, and
// `MoveAndResizeRelativeToDisplayArea` is still an anonymous vtable slot), while
// `MonitorFromWindow` + `GetMonitorInfoW` return the same data - `rcWork` is the work area
// with the taskbar already excluded - without regenerating the bindings.
//
// Units: every query in this file returns **physical pixels** in the virtual screen
// coordinate space (the primary monitor top-left corner is the origin), the same space Win32
// `RECT` and `AppWindow` geometry use. The declaration layer ([`WindowVisuals`]) takes DIP;
// convert with [`WorkArea::dips`], or hand the value straight to [`WindowVisuals::centered`].
//
// [`WindowVisuals`]: crate::WindowVisuals
// [`WindowVisuals::initial_position`]: crate::WindowVisuals::initial_position

use crate::WindowHandle;

/// Raw `HWND` value as [`WindowHandle::as_raw`] returns it (`*mut c_void`).
type HwndRaw = *mut std::ffi::c_void;

// `HMONITOR` / `RECT` / `MONITORINFO` are the Win32 names; the Rust spellings are used here
// because FFI cares about layout, not names. Doc comments name the original for cross-checking.
windows_core::link!("user32.dll" "system" fn GetDpiForSystem() -> u32);
windows_core::link!("user32.dll" "system" fn GetDpiForWindow(hwnd : HwndRaw) -> u32);
windows_core::link!("user32.dll" "system" fn GetWindowRect(hwnd : HwndRaw, lprect : *mut Rect) -> windows_core::BOOL);
windows_core::link!("user32.dll" "system" fn MonitorFromWindow(hwnd : HwndRaw, dwflags : u32) -> Hmonitor);
windows_core::link!("user32.dll" "system" fn GetMonitorInfoW(hmonitor : Hmonitor, lpmi : *mut MonitorInfo) -> windows_core::BOOL);
windows_core::link!("user32.dll" "system" fn SystemParametersInfoW(uiaction : u32, uiparam : u32, pvparam : *mut std::ffi::c_void, fwinini : u32) -> windows_core::BOOL);

/// `MonitorFromWindow` flag: fall back to the nearest monitor when the window is on none.
const MONITOR_DEFAULTTONEAREST: u32 = 0x0000_0002;
/// `SystemParametersInfoW` action: the primary monitor work area.
const SPI_GETWORKAREA: u32 = 0x0030;

/// Win32 `HMONITOR`: a monitor handle, opaque except to the system.
type Hmonitor = *mut std::ffi::c_void;

/// Win32 `RECT` in physical pixels. Private to this file, so the fields use Rust naming.
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

impl Rect {
    pub(super) fn width(self) -> i32 {
        self.right - self.left
    }

    pub(super) fn height(self) -> i32 {
        self.bottom - self.top
    }
}

/// Win32 `MONITORINFO`. The field order must match the C definition because
/// `GetMonitorInfoW` writes back through the memory layout, and the caller must set `cb_size`
/// to the struct size or the system rejects the call.
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct MonitorInfo {
    cb_size: u32,
    rc_monitor: Rect,
    rc_work: Rect,
    flags: u32,
}

impl MonitorInfo {
    /// A zeroed struct with `cb_size` filled in.
    fn new() -> Self {
        Self {
            cb_size: size_of::<Self>() as u32,
            ..Default::default()
        }
    }
}

/// A monitor work area: the screen rectangle minus the taskbar and other reserved regions.
///
/// Coordinates are **physical pixels** in the virtual screen space (the primary monitor
/// top-left corner is `(0, 0)`, secondary monitors can be negative), the same space Win32
/// `RECT` and `AppWindow.Position()`/`Size()` use. The declaration layer [`WindowVisuals`]
/// takes DIP; see [`WorkArea::dips`] for the conversion.
///
/// [`WindowVisuals`]: crate::WindowVisuals
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WorkArea {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl WorkArea {
    fn from_rect(rect: Rect) -> Self {
        Self {
            x: f64::from(rect.left),
            y: f64::from(rect.top),
            width: f64::from(rect.width()),
            height: f64::from(rect.height()),
        }
    }

    /// Converts the same region to DIP (`dpi` is the display scale: 96 is 100%, 144 is 150%).
    ///
    /// Feeding physical pixel values to [`WindowVisuals::initial_position`] / [`client_size`]
    /// misplaces the window on a display that is not at 100%; this is that conversion.
    ///
    /// [`WindowVisuals::initial_position`]: crate::WindowVisuals::initial_position
    /// [`client_size`]: crate::WindowVisuals::client_size
    pub fn dips(self, dpi: u32) -> Self {
        let scale = 96.0 / f64::from(dpi.max(96));
        Self {
            x: self.x * scale,
            y: self.y * scale,
            width: self.width * scale,
            height: self.height * scale,
        }
    }

    /// Top-left corner for a window of `width` by `height` centered in this work area.
    ///
    /// Arguments and result use the units of this work area (the query functions return
    /// physical pixels), and the result is whole coordinates because half a pixel means nothing.
    /// A window larger than the work area yields a negative offset: that is what centering
    /// produces, not an error.
    pub fn centered_origin(self, width: f64, height: f64) -> (f64, f64) {
        (
            (self.x + (self.width - width) / 2.0).round(),
            (self.y + (self.height - height) / 2.0).round(),
        )
    }
}

/// The primary monitor work area (taskbar excluded), in physical pixels.
///
/// The primary monitor top-left corner is the virtual screen origin, so this value feeds
/// directly into calculations such as centering on the primary display. With multiple monitors,
/// compute against the monitor the window is actually on instead, using
/// [`work_area_for_window`].
///
/// Returns `None` when the system rejects the query; callers should fall back to a conservative
/// default rather than fail.
///
/// Calls into the system API; **do not call this per rendered frame**. Query once at startup.
pub fn primary_work_area() -> Option<WorkArea> {
    let mut rect = Rect::default();
    let ok = unsafe {
        SystemParametersInfoW(SPI_GETWORKAREA, 0, std::ptr::from_mut(&mut rect).cast(), 0)
    };
    ok.as_bool().then_some(WorkArea::from_rect(rect))
}

/// The primary monitor work area size, converted to DIP with the system DPI
/// (`GetDpiForSystem`).
///
/// This is the unit [`WindowVisuals::client_size`] expects: that field takes DIP only, and
/// declaring a physical pixel size opens a larger window on a display that is not at 100%. Use
/// this value for proportional sizes such as 70 percent of the work area.
///
/// The conversion uses the **system DPI**. If the window ends up on a monitor with a different
/// scale, the declared value is reinterpreted at that monitor scale, so the size follows it. To
/// match one specific monitor exactly, use [`WorkArea::dips`] together with
/// [`work_area_for_window`] (once the window exists).
///
/// [`WindowVisuals::client_size`]: crate::WindowVisuals::client_size
pub fn primary_work_area_dip() -> Option<(f64, f64)> {
    let area = primary_work_area()?;
    let dpi = unsafe { GetDpiForSystem() };
    let area = area.dips(dpi);
    Some((area.width, area.height))
}

/// The work area of the monitor `handle` is on (taskbar excluded), in physical pixels.
///
/// Use this for multiple monitors, for a secondary display left of the primary (negative
/// coordinates), or for mixed scale factors; centering on **the monitor the window is on** is
/// [`WorkArea::centered_origin`] plus the window size.
///
/// Takes [`WindowHandle`] **by reference**: a single `run_window` often queries several values
/// in a row (the work area, the current position), and `WindowHandle` is neither `Copy` nor
/// `Clone`, so taking it by value would allow one query only.
///
/// Takes a [`WindowHandle`] rather than a raw `HWND`: when the handle is stale,
/// `MonitorFromWindow` falls back to the nearest monitor and the computed position becomes
/// arbitrary, while this reactor handle guarantees a live window.
pub fn work_area_for_window(handle: &WindowHandle<'_>) -> Option<WorkArea> {
    work_area_of(handle.as_raw().cast())
}

/// The current position of `handle`: the top-left corner of the window **frame** (border and
/// title bar included), in DIP.
///
/// Same units and anchor point as [`WindowVisuals::initial_position`], so remembering where the
/// window was means storing this value and declaring it back at startup. The DIP conversion uses
/// the DPI of the window itself (`GetDpiForWindow`), the same conversion the declaration layer
/// applies, so a round trip does not drift.
///
/// [`WindowVisuals::initial_position`]: crate::WindowVisuals::initial_position
pub fn window_placement_dip(handle: &WindowHandle<'_>) -> Option<(f64, f64)> {
    let hwnd: HwndRaw = handle.as_raw().cast();
    let rect = window_frame(hwnd)?;
    let dpi = unsafe { GetDpiForWindow(hwnd) }.max(96);
    let scale = 96.0 / f64::from(dpi);
    Some((f64::from(rect.left) * scale, f64::from(rect.top) * scale))
}

/// The current rectangle of the window **frame**, in physical pixels.
fn window_frame(hwnd: HwndRaw) -> Option<Rect> {
    let mut rect = Rect::default();
    let ok = unsafe { GetWindowRect(hwnd, &mut rect) };
    ok.as_bool().then_some(rect)
}

/// The work area of the monitor `hwnd` is on, in physical pixels. Internal form of
/// [`work_area_for_window`].
pub(super) fn work_area_of(hwnd: HwndRaw) -> Option<WorkArea> {
    let monitor = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) };
    if monitor.is_null() {
        return None;
    }
    let mut info = MonitorInfo::new();
    let ok = unsafe { GetMonitorInfoW(monitor, &mut info) };
    ok.as_bool().then_some(WorkArea::from_rect(info.rc_work))
}

#[cfg(test)]
mod window_geometry_tests {
    use super::*;

    #[test]
    fn centered_origin_rounds_to_whole_units() {
        let area = WorkArea {
            x: 0.0,
            y: 0.0,
            width: 1920.0,
            height: 1040.0,
        };
        assert_eq!(area.centered_origin(1280.0, 800.0), (320.0, 120.0));
        // Secondary display left of the primary: a negative origin still takes part in
        // centering.
        let secondary = WorkArea {
            x: -1920.0,
            y: 0.0,
            width: 1920.0,
            height: 1040.0,
        };
        assert_eq!(secondary.centered_origin(1280.0, 800.0), (-1600.0, 120.0));
        // An odd difference does not produce half a pixel.
        let odd = WorkArea {
            x: 0.0,
            y: 0.0,
            width: 1001.0,
            height: 801.0,
        };
        assert_eq!(odd.centered_origin(500.0, 400.0), (251.0, 201.0));
    }

    #[test]
    fn dips_scale_by_display_dpi() {
        let area = WorkArea {
            x: 0.0,
            y: 0.0,
            width: 2560.0,
            height: 1440.0,
        };
        assert_eq!(area.dips(96).width, 2560.0);
        assert_eq!(area.dips(192).width, 1280.0);
        assert_eq!(area.dips(120).height, 1152.0);
        // A reported value below 96 is treated as 96: no display has a negative scale, and an
        // unbounded magnification is worse than clamping.
        assert_eq!(area.dips(0).width, 2560.0);
    }

    #[test]
    fn work_area_size_matches_centered_origin_arithmetic() {
        // `primary_work_area_dip` returns the size only, and those two numbers are what a host
        // needs to center itself: size minus window size, divided by two.
        let area = WorkArea {
            x: 0.0,
            y: 0.0,
            width: 1280.0,
            height: 800.0,
        };
        let (x, y) = area.centered_origin(800.0, 600.0);
        assert_eq!((x, y), ((1280.0 - 800.0) / 2.0, (800.0 - 600.0) / 2.0));
    }
}
