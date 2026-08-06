use crate::controls::*;
use windows_reactor::*;

pub fn standard_ui_command_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "StandardUICommand",
        "Standard commands with built-in icons and keyboard shortcuts.",
        vec![sample_card("Overview", text_block("StandardUICommand provides standard commands (Copy, Paste, Save) with icons, labels, and shortcuts.").font_size(13.0), r#"StandardUICommandKind.Copy / Paste / Save"#)],
    )
}