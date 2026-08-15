use crate::controls::*;
use windows_reactor::*;

pub fn theme_shadow_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "Theme Shadows",
        "Theme shadows give elements depth with ambient and key shadows.",
        vec![sample_card(
            "Overview",
            vstack(vec![
                text_block("ThemeShadow casts a shadow from an element onto surfaces behind it.")
                    .font_size(13.0)
                    .into(),
                text_block("Configure receivers for the shadow to appear.")
                    .font_size(13.0)
                    .into(),
            ])
            .spacing(8.0),
            r#"ThemeShadow Receivers = { Grid }"#,
        )],
    )
}
