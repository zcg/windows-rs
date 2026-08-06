use crate::controls::*;
use windows_reactor::*;

pub fn clipboard_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "Clipboard",
        "Read and write text and data to the system clipboard.",
        vec![sample_card("Overview", text_block("Clipboard APIs move text, images, and files between apps via Windows.ApplicationModel.DataTransfer.").font_size(13.0), r#"Clipboard.SetContent / GetContent"#)],
    )
}