use crate::controls::*;
use windows_reactor::*;

pub fn animated_icon_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "AnimatedIcon",
        "An animated icon driven by state transitions.",
        vec![sample_card("Overview", text_block("AnimatedIcon plays Lottie-style animations on state changes (hover, press, checked).").font_size(13.0), r#"AnimatedIcon + AnimatedIconSource"#)],
    )
}