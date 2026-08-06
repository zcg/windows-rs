use crate::controls::*;
use windows_reactor::*;

pub fn page_transition_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "Page Transitions",
        "Page transitions animate navigation between pages.",
        vec![
            sample_card(
                "Overview",
                text_block("NavigationView and Frame provide built-in page transition animations.").font_size(13.0),
                r#"Frame.Navigate with transition animations"#,
            ),
        ],
    )
}