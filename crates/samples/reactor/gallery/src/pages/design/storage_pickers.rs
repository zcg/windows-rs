use crate::controls::*;
use windows_reactor::*;

pub fn storage_pickers_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "Storage Pickers",
        "Open and save files through system pickers.",
        vec![sample_card("Overview", text_block("FileOpenPicker and FileSavePicker let users choose files with the system UI.").font_size(13.0), r#"FileOpenPicker.PickSingleFileAsync()"#)],
    )
}