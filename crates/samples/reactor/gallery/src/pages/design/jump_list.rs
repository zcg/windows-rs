use crate::controls::*;
use windows_reactor::*;

pub fn jump_list_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "Jump List",
        "Jump Lists surface recent and frequent items on the taskbar.",
        vec![sample_card("Overview", text_block("Configure JumpList items and system groups for the app taskbar entry.").font_size(13.0), r#"JumpList.LoadCurrentAsync()"#)],
    )
}