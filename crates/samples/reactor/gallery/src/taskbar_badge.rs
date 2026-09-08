//! Taskbar badge through `ITaskbarList3::SetOverlayIcon`.
//!
//! Unlike the Windows App SDK badge manager, the overlay icon needs no
//! packaged app identity: any desktop process can attach an icon to its own
//! taskbar button. The gallery therefore shows badges unpackaged, without
//! registering a package or relaunching.
//!
//! The icon is drawn with GDI into a 32bpp DIB: a filled circle plus an
//! optional count, with the alpha channel filled in afterwards.

use std::ffi::c_void;
use std::mem::size_of;

use windows::Win32::combaseapi::{CLSCTX_ALL, CoCreateInstance};
use windows::Win32::minwindef::{LPARAM, TRUE};
use windows::Win32::processthreadsapi::GetCurrentProcessId;
use windows::Win32::shobjidl_core::ITaskbarList3;
use windows::Win32::windef::{COLORREF, HBITMAP, HFONT, HGDIOBJ, HICON, HWND, RECT};
use windows::Win32::wingdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CLEARTYPE_QUALITY, CreateCompatibleDC, CreateDIBSection,
    CreateFontIndirectW, CreateSolidBrush, DEFAULT_CHARSET, DEFAULT_PITCH, DIB_RGB_COLORS,
    DeleteDC, DeleteObject, Ellipse, FW_SEMIBOLD, GetStockObject, LOGFONTW, NULL_PEN,
    OUT_DEFAULT_PRECIS, SelectObject, SetBkMode, SetTextColor, TRANSPARENT,
};
use windows::Win32::winuser::{
    CreateIconIndirect, DT_CENTER, DT_NOCLIP, DT_SINGLELINE, DT_VCENTER, DestroyIcon, DrawTextW,
    EnumWindows, GetDC, GetDpiForWindow, GetWindowTextW, GetWindowThreadProcessId, ICONINFO,
    IsWindowVisible, ReleaseDC,
};
use windows::core::{BOOL, GUID, PCWSTR, PWSTR, Result};

/// `CLSID_TaskbarList` (not projected by this windows crate build).
const CLSID_TASKBAR_LIST: GUID = GUID::from_u128(0x56fdf344_fd6d_11d0_958a_006097c9a090);

/// The badge presented on the taskbar button.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Badge {
    /// A plain dot without a number.
    Dot,
    /// The count shown inside the badge; `0` renders as a dot too.
    Number(u32),
    /// Removes the overlay.
    Clear,
}

/// Applies `badge` to the current taskbar button.
pub fn set_badge(badge: Badge) -> Result<()> {
    let hwnd = find_window()?;
    let taskbar = taskbar()?;
    match badge {
        Badge::Clear => {
            unsafe { taskbar.SetOverlayIcon(hwnd, HICON::default(), PCWSTR::null()) }.ok()
        }
        Badge::Dot | Badge::Number(0) => show_icon(&taskbar, hwnd, None),
        Badge::Number(count) => show_icon(&taskbar, hwnd, Some(count)),
    }
}

fn show_icon(taskbar: &ITaskbarList3, hwnd: HWND, count: Option<u32>) -> Result<()> {
    let icon = render_icon(hwnd, count)?;
    let result = unsafe { taskbar.SetOverlayIcon(hwnd, icon, PCWSTR::null()) };
    unsafe {
        let _ = DestroyIcon(icon);
    };
    result.ok()
}

fn taskbar() -> Result<ITaskbarList3> {
    let taskbar: ITaskbarList3 = unsafe {
        CoCreateInstance::<_, ITaskbarList3>(&CLSID_TASKBAR_LIST, None, CLSCTX_ALL as u32)?
    };
    unsafe { taskbar.HrInit() }.ok()?;
    Ok(taskbar)
}

/// Finds the first visible top-level window owned by this process.
fn find_window() -> Result<HWND> {
    let mut found = None;
    unsafe extern "system" fn callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let found = unsafe { &mut *(lparam.0 as *mut Option<HWND>) };
        let mut window_pid = 0u32;
        unsafe {
            GetWindowThreadProcessId(hwnd, Some(&raw mut window_pid));
        }
        let current_pid = unsafe { GetCurrentProcessId() };
        if window_pid != current_pid || !unsafe { IsWindowVisible(hwnd) }.as_bool() {
            return BOOL(TRUE);
        }
        let mut title = [0u16; 64];
        let len = unsafe { GetWindowTextW(hwnd, PWSTR(title.as_mut_ptr()), title.len() as i32) };
        if len == 0 {
            return BOOL(TRUE);
        }
        *found = Some(hwnd);
        BOOL(TRUE)
    }
    unsafe {
        let _ = EnumWindows(Some(callback), LPARAM(&raw mut found as isize));
    }
    found.ok_or_else(|| {
        windows::core::Error::new(
            windows::core::HRESULT(0x80070057u32 as i32),
            "no visible window for this process",
        )
    })
}

fn colorref(r: u32, g: u32, b: u32) -> COLORREF {
    COLORREF(r | (g << 8) | (b << 16))
}

/// Renders the badge icon: a red circle (scaled by DPI) with the count.
fn render_icon(hwnd: HWND, count: Option<u32>) -> Result<HICON> {
    unsafe {
        let dpi = GetDpiForWindow(hwnd);
        let size = (32 * dpi as i32 / 96).max(16);
        let header = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: size,
                biHeight: -size,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB as u32,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bits: *mut c_void = core::ptr::null_mut();
        let screen_dc = GetDC(None);
        let dib = CreateDIBSection(
            Some(screen_dc),
            &header,
            DIB_RGB_COLORS as u32,
            &raw mut bits,
            None,
            0,
        );
        let dib_dc = CreateCompatibleDC(Some(screen_dc));
        let old_dib = SelectObject(dib_dc, HGDIOBJ(dib.0));

        // Draw the circle, then the count. GDI leaves the DIB alpha channel
        // zero, so it is set to opaque for every painted pixel below.
        let brush = CreateSolidBrush(colorref(0xC4, 0x2B, 0x1C));
        let old_brush = SelectObject(dib_dc, HGDIOBJ(brush.0));
        let old_pen = SelectObject(dib_dc, GetStockObject(NULL_PEN));
        SetBkMode(dib_dc, TRANSPARENT);
        let margin = dpi as i32 * 2 / 96;
        let _ = Ellipse(dib_dc, margin, margin, size - margin, size - margin);

        if let Some(count_value) = count {
            let font = create_font(dpi as i32);
            let old_font = SelectObject(dib_dc, HGDIOBJ(font.0));
            SetTextColor(dib_dc, colorref(0xFF, 0xFF, 0xFF));
            let mut text: Vec<u16> = count_value.to_string().encode_utf16().collect();
            text.push(0);
            let mut rect = RECT {
                left: 0,
                top: 0,
                right: size,
                bottom: size,
            };
            DrawTextW(
                dib_dc,
                PCWSTR::from_raw(text.as_ptr()),
                -1,
                &raw mut rect,
                (DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOCLIP) as u32,
            );
            SelectObject(dib_dc, old_font);
            let _ = DeleteObject(HGDIOBJ(font.0));
        }

        // Fill the alpha channel for every painted (non-zero) pixel.
        stride_fill_alpha(bits, size);

        SelectObject(dib_dc, old_dib);
        SelectObject(dib_dc, old_brush);
        SelectObject(dib_dc, old_pen);
        let _ = DeleteDC(dib_dc);
        let _ = DeleteObject(HGDIOBJ(brush.0));
        let _ = ReleaseDC(None, screen_dc);

        let info = ICONINFO {
            fIcon: BOOL(TRUE),
            xHotspot: 0,
            yHotspot: 0,
            hbmMask: HBITMAP::default(),
            hbmColor: dib,
        };
        Ok(CreateIconIndirect(&info))
    }
}

fn stride_fill_alpha(bits: *mut c_void, size: i32) {
    let bytes = size as usize * 4;
    for y in 0..size as usize {
        let row = unsafe { (bits as *mut u8).add(y * bytes) };
        for x in 0..size as usize {
            let pixel = unsafe { row.add(x * 4) };
            let (b, g, r) = unsafe { (*pixel, *pixel.add(1), *pixel.add(2)) };
            if b | g | r != 0 {
                unsafe { *pixel.add(3) = 0xFF };
            }
        }
    }
}

fn create_font(dpi: i32) -> HFONT {
    let mut font = LOGFONTW {
        lfHeight: -(11 * dpi / 96) + 1,
        lfWeight: FW_SEMIBOLD,
        lfCharSet: DEFAULT_CHARSET as u8,
        lfOutPrecision: OUT_DEFAULT_PRECIS as u8,
        lfQuality: CLEARTYPE_QUALITY as u8,
        lfPitchAndFamily: DEFAULT_PITCH as u8,
        ..Default::default()
    };
    let name = "Segoe UI";
    for (slot, unit) in font
        .lfFaceName
        .iter_mut()
        .zip(name.encode_utf16().chain(std::iter::once(0)))
    {
        *slot = unit;
    }
    unsafe { CreateFontIndirectW(&font) }
}
