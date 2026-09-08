use crate::controls::*;
use windows_reactor::*;

pub struct NotificationsPage {
    status: String,
    mode: BadgeMode,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BadgeMode {
    Dot,
    Number,
}

#[derive(Clone)]
pub enum Message {
    Toast,
    Badge,
    SetMode(BadgeMode),
    Clear,
}

/// Sends a toast through the Windows App Runtime notification manager, which
/// works for unpackaged apps once the runtime is bootstrapped (the gallery
/// bootstraps on startup). Registration is process-wide: registering twice
/// fails, so it happens only once.
fn send_toast(title: &str, body: &str) -> Result<(), windows_core::Error> {
    use crate::notifications_bindings::{AppNotification, AppNotificationManager};
    use std::sync::atomic::{AtomicBool, Ordering};

    static REGISTERED: AtomicBool = AtomicBool::new(false);

    if !AppNotificationManager::IsSupported().unwrap_or(false) {
        return Err(windows_core::Error::new(
            windows_core::HRESULT(0x80070032u32 as i32),
            "App notifications are not supported on this system.",
        ));
    }
    let manager = AppNotificationManager::Default()?;
    if !REGISTERED.swap(true, Ordering::AcqRel) {
        // Registration is process-wide: a second Register fails with
        // E_INVALIDARG ("Already Registered"). Tolerate that result so a
        // repeated send still reaches Show even if registration happened
        // through another path first.
        match manager.Register() {
            Ok(()) => {}
            Err(error) if error.code().0 == 0x80070057u32 as i32 => {}
            Err(error) => {
                REGISTERED.store(false, Ordering::Release);
                return Err(error);
            }
        }
    }
    let payload = format!(
        "<toast><visual><binding template=\"ToastGeneric\">\
         <text>{title}</text><text>{body}</text></binding></visual></toast>"
    );
    let notification = AppNotification::CreateInstance(&payload)?;
    manager.Show(&notification)
}

/// Sets the taskbar badge through the Windows App SDK badge manager
/// (`Microsoft.Windows.BadgeNotifications.BadgeNotificationManager`), the
/// same API the official WinUI Gallery uses. It requires a packaged
/// (MSIX-set) app identity: BadgeNotificationManager is unavailable in
/// unpackaged mode, and the gallery must run from an installed MSIX (not
/// merely an in-place registration) for the count to render.
fn set_badge(mode: BadgeMode, count: u32) -> windows::core::Result<()> {
    use crate::notifications_bindings::{BadgeNotificationGlyph, BadgeNotificationManager};

    let manager = BadgeNotificationManager::Current()?;
    match mode {
        BadgeMode::Dot => manager.SetBadgeAsGlyph(BadgeNotificationGlyph::Activity)?,
        BadgeMode::Number => manager.SetBadgeAsCount(count.min(99))?,
    }
    Ok(())
}

/// Removes the current taskbar badge.
fn clear_badge() -> windows::core::Result<()> {
    use crate::notifications_bindings::BadgeNotificationManager;

    BadgeNotificationManager::Current()?.ClearBadge()
}

impl Component for NotificationsPage {
    type Message = Message;
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self {
            status: String::from("Ready."),
            mode: BadgeMode::Number,
        }
    }

    fn update(&mut self, message: Message, _: &ComponentContext<Self>) {
        match message {
            Message::Toast => {
                self.status = match send_toast("Reactor Gallery", "Hello from a toast!") {
                    Ok(()) => String::from("Toast sent."),
                    Err(error) => format!("Send failed: {error}"),
                };
            }
            Message::Badge => {
                self.status = match set_badge(self.mode, 5) {
                    Ok(()) => format!("Badge set to 5 ({:?}).", self.mode),
                    Err(error) => format!("Badge failed: {error}"),
                };
            }
            Message::SetMode(mode) => {
                self.mode = mode;
                self.status = match set_badge(mode, 5) {
                    Ok(()) => format!("Badge set to 5 ({mode:?})."),
                    Err(error) => format!("Badge failed: {error}"),
                };
            }
            Message::Clear => {
                self.status = match clear_badge() {
                    Ok(()) => String::from("Badge cleared."),
                    Err(error) => format!("Badge clear failed: {error}"),
                };
            }
        }
    }

    fn view(&self, _: &(), context: &mut ViewContext<Self>) -> View {
        page_content(
            "Notifications",
            "Show toasts and update the taskbar badge.",
            [
                KeyedView::new(
                    "toast",
                    sample_card(
                        "Toast Notification",
                        StackPanel::new().spacing(8.0).children((
                            Button::new()
                                .on_click(context.message(Message::Toast))
                                .content("Send toast"),
                            TextBlock::new().text(self.status.clone()).opacity(0.6),
                        )),
                        "AppNotificationManager::Default()?.Register()?; \
                         AppNotification::CreateInstance(payload)?; \
                         manager.Show(&notification)?",
                    ),
                ),
                KeyedView::new(
                    "badge",
                    sample_card(
                        "Taskbar Badge",
                        StackPanel::new().spacing(8.0).children((
                            StackPanel::new()
                                .orientation(Orientation::Horizontal)
                                .spacing(8.0)
                                .children((
                                    RadioButton::new()
                                        .group_name("badge-mode")
                                        .is_checked(self.mode == BadgeMode::Number)
                                        .on_checked(context.callback(|_: bool| {
                                            Message::SetMode(BadgeMode::Number)
                                        }))
                                        .content("Number"),
                                    RadioButton::new()
                                        .group_name("badge-mode")
                                        .is_checked(self.mode == BadgeMode::Dot)
                                        .on_checked(
                                            context.callback(|_: bool| {
                                                Message::SetMode(BadgeMode::Dot)
                                            }),
                                        )
                                        .content("Dot"),
                                    Button::new()
                                        .on_click(context.message(Message::Badge))
                                        .content("Show"),
                                    Button::new()
                                        .on_click(context.message(Message::Clear))
                                        .content("Clear"),
                                )),
                            TextBlock::new()
                                .text(
                                    "System badge via BadgeUpdateManager (requires package identity).",
                                )
                                .opacity(0.6),
                            TextBlock::new().text(self.status.clone()).opacity(0.6),
                        )),
                        "BadgeUpdateManager::CreateBadgeUpdaterForApplication()?.Update(&badge)",
                    ),
                ),
            ],
        )
    }
}
