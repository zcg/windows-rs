//! Active-window access for title bar customization. The `AppWindowTitleBar`
//! of the first window is obtained through the reactor host, then the public
//! helpers apply the settings the gallery demonstrates.

use super::*;

/// The `AppWindow` of the first window opened on the current UI thread.
pub fn active_app_window() -> Result<AppWindow> {
    with_active_host(|host| {
        let window = host.window().clone();
        window.cast::<bindings::IWindow2>().ok()?.AppWindow().ok()
    })
    .flatten()
    .ok_or_else(Error::empty)
}

/// The title bar of the active window.
pub fn active_title_bar() -> Result<AppWindowTitleBar> {
    active_app_window()?.TitleBar()
}

/// Extend app content into the title bar area.
pub fn title_bar_set_extended(title_bar: &AppWindowTitleBar, on: bool) -> Result<()> {
    title_bar.SetExtendsContentIntoTitleBar(on)
}

/// Select the title bar height option (standard or tall).
pub fn title_bar_set_height(
    title_bar: &AppWindowTitleBar,
    option: TitleBarHeightOption,
) -> Result<()> {
    title_bar
        .cast::<bindings::IAppWindowTitleBar2>()?
        .SetPreferredHeightOption(option)
}

/// Select the title bar theme.
pub fn title_bar_set_theme(title_bar: &AppWindowTitleBar, theme: TitleBarTheme) -> Result<()> {
    title_bar
        .cast::<bindings::IAppWindowTitleBar3>()?
        .SetPreferredTheme(theme)
}

/// Apply (or clear) an accent color pair to the title bar buttons.
pub fn title_bar_set_accent(title_bar: &AppWindowTitleBar, on: bool) -> Result<()> {
    if on {
        title_bar.SetButtonBackgroundColor(Some(Color {
            a: 255,
            r: 0,
            g: 120,
            b: 212,
        }))?;
        title_bar.SetButtonForegroundColor(Some(Color {
            a: 255,
            r: 255,
            g: 255,
            b: 255,
        }))?;
    } else {
        title_bar.SetButtonBackgroundColor(None)?;
        title_bar.SetButtonForegroundColor(None)?;
    }
    Ok(())
}

/// Restore the system default title bar.
pub fn title_bar_reset(title_bar: &AppWindowTitleBar) -> Result<()> {
    title_bar.ResetToDefault()
}
