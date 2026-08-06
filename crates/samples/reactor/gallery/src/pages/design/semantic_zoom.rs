use crate::controls::*;
use windows_reactor::*;

pub fn semantic_zoom_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "SemanticZoom",
        "Zoom between two views of the same data.",
        vec![sample_card("Overview", text_block("SemanticZoom switches between a zoomed-in view and a zoomed-out view of a collection.").font_size(13.0), r#"SemanticZoom ZoomedInView + ZoomedOutView"#)],
    )
}