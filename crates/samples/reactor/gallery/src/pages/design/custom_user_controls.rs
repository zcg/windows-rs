use crate::controls::*;
use windows_reactor::*;

pub fn custom_user_controls_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "Custom User Controls",
        "Compose reusable custom controls.",
        vec![sample_card("Custom User Controls", text_block("Reactor apps compose controls from built-in widgets; custom components wrap state and render logic into reusable pieces.").font_size(13.0), r#"component(custom_component, props)"#)],
    )
}