use crate::controls::*;
use windows_reactor::*;

pub fn popup_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "Popup",
        "A temporary overlay above the page content.",
        vec![sample_card("Overview", text_block("Popup shows content in an overlay; reactor exposes it via the Popup binding, with flyouts as the recommended alternative.").font_size(13.0), r#"Popup IsOpen + Child"#)],
    )
}