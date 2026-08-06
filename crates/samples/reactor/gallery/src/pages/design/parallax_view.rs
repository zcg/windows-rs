use crate::controls::*;
use windows_reactor::*;

pub fn parallax_view_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "ParallaxView",
        "Parallax scrolling links an element to a scrolling source.",
        vec![sample_card("Overview", text_block("ParallaxView moves a background element at a different rate than the scroll source.").font_size(13.0), r#"ParallaxView Source + HorizontalShift"#)],
    )
}