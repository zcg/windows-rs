use crate::controls::*;
use windows_reactor::*;

pub fn capture_element_preview_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "Capture Element Preview",
        "Preview camera capture in a MediaCaptureElement.",
        vec![sample_card("Overview", text_block("CaptureElementPreview shows a live camera preview using MediaCapture.").font_size(13.0), r#"MediaCapture.StartPreviewAsync()"#)],
    )
}