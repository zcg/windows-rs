//! 窗口几何实测：显示器工作区，以及窗口当前落在哪儿、有多大。
//!
//! 手写层。`bindings.rs` 整体是生成产物（`crates/tools/reactor` 里的 winmd 生成器重写它），
//! 所以这里新用到的 Win32 入口定义在本文件里，重新生成绑定不会把它们冲掉。
//!
//! 为什么用 Win32 而不是 `Microsoft.UI.Windowing.DisplayArea`：绑定里没有 `DisplayArea`
//! （`winui.toml` 只收控件与少量接口，`MoveAndResizeRelativeToDisplayArea` 在 vtable 里还是
//! 匿名占位），而 `MonitorFromWindow` + `GetMonitorInfoW` 给的是同一份数据——`rcWork` 就是
//! 工作区，系统已经在里面排除了任务栏——且不必重新生成绑定。
//!
//! 单位约定：本文件的查询一律返回**物理像素**（虚拟屏幕坐标系，主显示器左上角是原点），
//! 与 Win32 `RECT`、`AppWindow` 的几何信息同一套。声明层（[`WindowVisuals`]）收的是 DIP，
//! 两者之间用 [`WorkArea::dips`] 换算，或者直接交给 [`WindowVisuals::centered`]。
//!
//! [`WindowVisuals`]: crate::WindowVisuals
//! [`WindowVisuals::centered`]: crate::WindowVisuals::centered

use super::*;
use crate::WindowHandle;

// `HMONITOR` / `RECT` / `MONITORINFO` 是 Win32 的名字，这里按 Rust 命名写（FFI 只认
// 布局不认名字），文档里标回原名以便对照。
windows_core::link!("user32.dll" "system" fn GetDpiForSystem() -> u32);
windows_core::link!("user32.dll" "system" fn GetWindowRect(hwnd : HWND, lprect : *mut Rect) -> windows_core::BOOL);
windows_core::link!("user32.dll" "system" fn MonitorFromWindow(hwnd : HWND, dwflags : u32) -> Hmonitor);
windows_core::link!("user32.dll" "system" fn GetMonitorInfoW(hmonitor : Hmonitor, lpmi : *mut MonitorInfo) -> windows_core::BOOL);
windows_core::link!("user32.dll" "system" fn SystemParametersInfoW(uiaction : u32, uiparam : u32, pvparam : *mut std::ffi::c_void, fwinini : u32) -> windows_core::BOOL);

/// `MonitorFromWindow` 的取项：窗口没有落在任何显示器上时退回最近的那一个。
const MONITOR_DEFAULTTONEAREST: u32 = 0x0000_0002;
/// `SystemParametersInfoW` 的取项：主显示器的工作区。
const SPI_GETWORKAREA: u32 = 0x0030;

/// Win32 `HMONITOR`：显示器句柄，只有系统认识的不透明指针。
type Hmonitor = *mut std::ffi::c_void;

/// Win32 `RECT`（物理像素）。只在本文件内部用，所以字段名按 Rust 习惯写。
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

/// Win32 `MONITORINFO`。字段顺序必须与 C 定义一致（`GetMonitorInfoW` 按内存布局写回），
/// `cb_size` 也必须由调用方填成结构体大小，否则系统直接拒绝。
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct MonitorInfo {
    cb_size: u32,
    rc_monitor: Rect,
    rc_work: Rect,
    flags: u32,
}

impl MonitorInfo {
    /// 填好 `cb_size` 的空结构体。
    fn new() -> Self {
        Self {
            cb_size: size_of::<Self>() as u32,
            ..Default::default()
        }
    }
}

/// 显示器工作区：屏幕区域减去任务栏等系统保留区。
///
/// 坐标是**物理像素**，坐标系是虚拟屏幕（主显示器左上角为 `(0, 0)`，副显示器可以是负数），
/// 与 Win32 `RECT`、`AppWindow.Position()`/`Size()` 同一套。声明层 [`WindowVisuals`] 收的是
/// DIP，换算见 [`WorkArea::dips`]。
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

    /// 把同一块区域换算成 DIP（`dpi` 是显示器缩放：96 是 100%，144 是 150%）。
    ///
    /// 直接把物理像素值喂给 [`WindowVisuals::client_position`] / [`client_size`] 会在非 100%
    /// 缩放的屏幕上落错位置，这一步就是那条换算。
    ///
    /// [`WindowVisuals::client_position`]: crate::WindowVisuals::client_position
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

    /// 宽 `width`、高 `height` 的窗口在本工作区里居中时的左上角坐标。
    ///
    /// 入参与返回值都使用本工作区的单位（查询函数给的是物理像素），结果是整数坐标——
    /// 半个像素没有意义。窗口比工作区还大时结果会是负偏移：那是居中本身的结果，不是错误。
    pub fn centered_origin(self, width: f64, height: f64) -> (f64, f64) {
        (
            (self.x + (self.width - width) / 2.0).round(),
            (self.y + (self.height - height) / 2.0).round(),
        )
    }
}

/// 主显示器的工作区（已排除任务栏），物理像素。
///
/// 主显示器左上角就是虚拟屏幕原点，所以它是"居中到主屏"这类算式的直接输入；多显示器下
/// 要按窗口实际所在的显示器算，用 [`work_area_for_window`]。
///
/// 失败（系统拒绝查询）返回 `None`，调用方应退回一个保守默认值而不是就此失败。
///
/// 会调用系统 API，**不要在每帧渲染里调**——启动时查一次即可。
pub fn primary_work_area() -> Option<WorkArea> {
    let mut rect = Rect::default();
    let ok = unsafe {
        SystemParametersInfoW(SPI_GETWORKAREA, 0, std::ptr::from_mut(&mut rect).cast(), 0)
    };
    ok.as_bool().then_some(WorkArea::from_rect(rect))
}

/// 主显示器工作区尺寸，按系统 DPI（`GetDpiForSystem`）换算成 DIP。
///
/// 这是 [`WindowVisuals::client_size`] 想要的单位：`client_size` 只收 DIP，拿物理像素尺寸
/// 去声明会在非 100% 缩放的屏幕上开出一个更大的窗口。用它实现"工作区的 70%"这类比例尺寸。
///
/// 换算按**系统 DPI**：窗口若最终落在缩放比例不同的显示器上，声明值会按那块显示器的比例
/// 重新解释，尺寸随该显示器的比例缩放。要严格贴合某块显示器，请用 [`WorkArea::dips`] 配合
/// [`work_area_for_window`]（在窗口存在之后）。
///
/// [`WindowVisuals::client_size`]: crate::WindowVisuals::client_size
pub fn primary_work_area_dip() -> Option<(f64, f64)> {
    let area = primary_work_area()?;
    let dpi = unsafe { GetDpiForSystem() };
    let area = area.dips(dpi);
    Some((area.width, area.height))
}

/// `handle` 所在显示器的工作区（已排除任务栏），物理像素。
///
/// 多显示器、副屏在主屏左边（负坐标）或缩放比例不同的场景用它；居中到**自己所在的那块屏**
/// 就是 [`WorkArea::centered_origin`] 加窗口尺寸。
///
/// 按**借用**收 [`WindowHandle`]：一次 `run_window` 里往往要连着查好几项（工作区、当前
/// 落点），而 `WindowHandle` 既不是 `Copy` 也不是 `Clone`，按值收就只能查一次。
///
/// 只收 [`WindowHandle`] 而不是裸 `HWND`：句柄失效时 `MonitorFromWindow` 会退回最近显示器，
/// 算出来的位置就会莫名其妙，而 reactor 的这个句柄保证指向一个活着的窗口。
pub fn work_area_for_window(handle: &WindowHandle<'_>) -> Option<WorkArea> {
    work_area_of(handle.as_raw() as HWND)
}

/// `handle` 当前的位置：窗口**外框**（含边框与标题栏）左上角，单位 DIP。
///
/// 与 [`WindowVisuals::client_position`] 是同一套单位与落点，所以"记住窗口上次在哪"就是
/// 把这里的返回值存下来、启动时声明回去。DIP 换算用窗口自己的 DPI（`GetDpiForWindow`），
/// 与声明层下发时的换算完全一致，来回一趟不会漂移。
///
/// [`WindowVisuals::client_position`]: crate::WindowVisuals::client_position
pub fn window_placement_dip(handle: &WindowHandle<'_>) -> Option<(f64, f64)> {
    let hwnd = handle.as_raw() as HWND;
    let rect = window_frame(hwnd)?;
    let dpi = unsafe { GetDpiForWindow(hwnd) }.max(96);
    let scale = 96.0 / f64::from(dpi);
    Some((f64::from(rect.left) * scale, f64::from(rect.top) * scale))
}

/// 窗口**外框**当前尺寸（宽、高，物理像素）。
///
/// 内部用：`.centered()` 要按窗口此刻真实的外框尺寸算居中——客户区尺寸声明完只是
/// `ResizeClient` 的结果，外框还要加上边框和标题栏，自己算容易漏。
pub(super) fn window_frame_size(hwnd: HWND) -> Option<(f64, f64)> {
    let rect = window_frame(hwnd)?;
    Some((f64::from(rect.width()), f64::from(rect.height())))
}

/// 窗口**外框**当前的矩形（物理像素）。
fn window_frame(hwnd: HWND) -> Option<Rect> {
    let mut rect = Rect::default();
    let ok = unsafe { GetWindowRect(hwnd, &mut rect) };
    ok.as_bool().then_some(rect)
}

/// `hwnd` 所在显示器的工作区（物理像素）。[`work_area_for_window`] 的内部版本。
pub(super) fn work_area_of(hwnd: HWND) -> Option<WorkArea> {
    let monitor = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) };
    if monitor.is_null() {
        return None;
    }
    let mut info = MonitorInfo::new();
    let ok = unsafe { GetMonitorInfoW(monitor, &mut info) };
    ok.as_bool().then_some(WorkArea::from_rect(info.rc_work))
}

#[cfg(test)]
mod tests {
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
        // 副屏在主屏左边：负原点同样参与居中。
        let secondary = WorkArea {
            x: -1920.0,
            y: 0.0,
            width: 1920.0,
            height: 1040.0,
        };
        assert_eq!(secondary.centered_origin(1280.0, 800.0), (-1600.0, 120.0));
        // 奇数差值不产生半个像素。
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
        // 低于 96 的回报值当 96 处理：没有"负缩放"的显示器，除零式的放大只会更糟。
        assert_eq!(area.dips(0).width, 2560.0);
    }

    #[test]
    fn work_area_size_matches_centered_origin_arithmetic() {
        // `primary_work_area_dip` 只返回尺寸，宿主自己居中时要的也是这两个数：
        // 尺寸减去窗口尺寸再除以二。
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
