//! WinAppSDK app notifications. Unpackaged builds have no app identity, so
//! `app_notification_supported()` reports whether notifications can be shown
//! and the send helper fails cleanly when they cannot.

use super::*;

/// Whether app notifications are supported in the current process.
pub fn app_notification_supported() -> bool {
    bindings::AppNotificationManager::IsSupported().unwrap_or(false)
}

/// Register the app and show a plain-text notification.
pub fn send_app_notification(title: &str, body: &str) -> Result<()> {
    if !app_notification_supported() {
        return Err(Error::from_hresult(windows_core::HRESULT(
            0x80070032u32 as i32,
        ))); // ERROR_NOT_SUPPORTED
    }
    let manager = bindings::AppNotificationManager::Default()?;
    manager.Register()?;
    let payload = format!(
        "<toast launch=\"\"><visual><binding template=\"ToastGeneric\">\
         <text>{title}</text><text>{body}</text></binding></visual></toast>"
    );
    let notification = bindings::AppNotification::CreateInstance(&payload)?;
    manager.Show(&notification)
}

/// Set the taskbar badge to a numeric count.
pub fn badge_set_count(count: u32) -> Result<()> {
    bindings::BadgeNotificationManager::Current()?.SetBadgeAsCount(count)
}

/// Set the taskbar badge to a glyph.
pub fn badge_set_glyph(glyph: BadgeNotificationGlyph) -> Result<()> {
    bindings::BadgeNotificationManager::Current()?.SetBadgeAsGlyph(glyph)
}

/// Clear the taskbar badge.
pub fn badge_clear() -> Result<()> {
    bindings::BadgeNotificationManager::Current()?.ClearBadge()
}
