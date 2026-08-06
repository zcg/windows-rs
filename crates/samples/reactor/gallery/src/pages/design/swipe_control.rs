use crate::controls::*;
use windows_reactor::*;

pub fn swipe_control_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "SwipeControl",
        "Reveal contextual actions by swiping an item.",
        vec![sample_card("Overview", text_block("SwipeControl wraps content and reveals SwipeItems on swipe gestures (delete, archive, etc.).").font_size(13.0), r#"SwipeControl LeftItems/RightItems + SwipeItem"#)],
    )
}