use crate::controls::*;
use windows_reactor::*;

pub fn theme_transition_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "Theme Transitions",
        "Theme transitions animate visual changes when the theme switches.",
        vec![
            sample_card(
                "Overview",
                text_block("WinUI animates theme switches automatically; apps can add custom transition resources.").font_size(13.0),
                r#"ThemeTransition resources animate theme and visual-state changes"#,
            ),
        ],
    )
}