use crate::controls::*;
use windows_reactor::*;

pub fn app_notification_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "App Notifications",
        "Send notifications from the app through the notification platform.",
        vec![sample_card("Overview", text_block("AppNotification APIs send Windows notifications with a title, body, and actions.").font_size(13.0), r#"AppNotificationManager.Default.Show(...)"#)],
    )
}