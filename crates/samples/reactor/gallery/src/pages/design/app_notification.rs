use crate::controls::*;
use windows_reactor::*;

pub fn app_notification_page(_: &(), cx: &mut RenderCx) -> Element {
    let (status, set_status) = cx.use_state(String::new());
    let supported = app_notification_supported();

    let send = move || match send_app_notification(
        "Reactor WinUI Gallery",
        "This notification was sent from the Rust gallery.",
    ) {
        Ok(()) => set_status.call("Notification sent.".into()),
        Err(err) => set_status.call(format!("Cannot send: {err}")),
    };

    page_content(
        "App Notifications",
        "Send a Windows app notification from the current process.",
        vec![sample_card(
            "Send a Notification",
            vstack((
                button("Send notification").on_click(send),
                text_block(if supported {
                    if status.is_empty() {
                        "Ready. Click to send."
                    } else {
                        status.as_str()
                    }
                } else {
                    "App notifications are not supported in this build: unpackaged apps need a \
                     registered app identity (AUMID), which the self-contained gallery does not \
                     provide. This mirrors the WinUI Gallery behavior on unsupported systems."
                })
                .opacity(0.6),
            ))
            .spacing(8.0),
            r#"if AppNotificationManager::IsSupported()? {
    let manager = AppNotificationManager::Default()?;
    manager.Register()?;
    let notification = AppNotification::CreateInstance(payload)?;
    manager.Show(&notification)?;
}"#,
        )],
    )
}
