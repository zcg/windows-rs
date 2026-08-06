use crate::controls::*;
use windows_reactor::*;

pub fn xaml_ui_command_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "XamlUICommand",
        "A command with icon, label, and keyboard accelerator.",
        vec![sample_card("Overview", text_block("XamlUICommand pairs an action with label, icon, and optional keyboard accelerator for menus and buttons.").font_size(13.0), r#"XamlUICommand Label, Icon, KeyboardAccelerators"#)],
    )
}