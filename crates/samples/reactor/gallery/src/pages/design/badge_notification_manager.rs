use crate::controls::*;
use windows_reactor::*;

pub fn badge_notification_manager_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "Badge Notifications",
        "Set numeric and glyph badges on the taskbar icon.",
        vec![sample_card("Overview", text_block("BadgeNotificationManager updates the taskbar badge with a number or glyph.").font_size(13.0), r#"BadgeNotificationManager.CreateBadgeUpdaterForApplication()"#)],
    )
}