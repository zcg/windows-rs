use crate::controls::*;
use windows_reactor::*;

pub fn annotated_scroll_bar_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "AnnotatedScrollBar",
        "A scroll bar with labels and value annotations.",
        vec![sample_card("Overview", text_block("AnnotatedScrollBar shows tick labels alongside a ScrollViewer scroll bar.").font_size(13.0), r#"AnnotatedScrollBar Labels + ScrollViewer"#)],
    )
}