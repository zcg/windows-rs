use crate::controls::*;
use windows_reactor::*;

pub fn badge_notification_manager_page(_: &(), cx: &mut RenderCx) -> Element {
    let (count, set_count) = cx.use_state(3.0_f64);
    let (status, set_status) = cx.use_state(String::from(
        "Set a numeric count or a glyph on the taskbar icon. Badge updates apply when the app runs packaged, matching the WinUI Gallery behavior.",
    ));

    let set_badge_count = {
        let set_status = set_status.clone();
        move || match badge_set_count(count as u32) {
            Ok(()) => set_status.call(format!("Badge count set to {}.", count as u32)),
            Err(err) => set_status.call(format!("Cannot set badge: {err}")),
        }
    };

    let set_glyph = {
        let set_status = set_status.clone();
        move |glyph: BadgeNotificationGlyph, name: &'static str| {
            let result = badge_set_glyph(glyph);
            match result {
                Ok(()) => set_status.call(format!("Badge glyph set to {name}.")),
                Err(err) => set_status.call(format!("Cannot set badge: {err}")),
            }
        }
    };

    let clear_badge = move || match badge_clear() {
        Ok(()) => set_status.call("Badge cleared.".into()),
        Err(err) => set_status.call(format!("Cannot clear badge: {err}")),
    };

    page_content(
        "Badge Notifications",
        "Set numeric and glyph badges on the taskbar icon.",
        vec![
            sample_card(
                "Badge Count",
                vstack((hstack((
                    NumberBox::new(count)
                        .range(0.0, 99.0)
                        .header("Count")
                        .on_value_changed(move |value| set_count.call(value)),
                    button("Set count").on_click(set_badge_count),
                ))
                .spacing(8.0),))
                .spacing(8.0),
                r#"BadgeNotificationManager::Current()?.SetBadgeAsCount(count)?;"#,
            ),
            sample_card(
                "Badge Glyph",
                vstack((
                    hstack((
                        button("Activity").on_click({
                            let set_glyph = set_glyph.clone();
                            move || set_glyph(BadgeNotificationGlyph::Activity, "Activity")
                        }),
                        button("Alert").on_click({
                            let set_glyph = set_glyph.clone();
                            move || set_glyph(BadgeNotificationGlyph::Alert, "Alert")
                        }),
                        button("Error").on_click({
                            let set_glyph = set_glyph.clone();
                            move || set_glyph(BadgeNotificationGlyph::Error, "Error")
                        }),
                        button("New message").on_click({
                            let set_glyph = set_glyph.clone();
                            move || set_glyph(BadgeNotificationGlyph::NewMessage, "NewMessage")
                        }),
                    ))
                    .spacing(8.0),
                    hstack((
                        button("Paused").on_click({
                            let set_glyph = set_glyph.clone();
                            move || set_glyph(BadgeNotificationGlyph::Paused, "Paused")
                        }),
                        button("Playing").on_click({
                            let set_glyph = set_glyph.clone();
                            move || set_glyph(BadgeNotificationGlyph::Playing, "Playing")
                        }),
                        button("Clear badge").on_click(clear_badge),
                    ))
                    .spacing(8.0),
                    text_block(status).opacity(0.6),
                ))
                .spacing(8.0),
                r#"BadgeNotificationManager::Current()?.SetBadgeAsGlyph(BadgeNotificationGlyph::Alert)?;
BadgeNotificationManager::Current()?.ClearBadge()?;"#,
            ),
        ],
    )
}
