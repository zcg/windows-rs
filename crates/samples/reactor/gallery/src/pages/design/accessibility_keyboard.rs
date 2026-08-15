use crate::controls::*;
use windows_reactor::*;

pub fn accessibility_keyboard_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "Accessibility: Keyboard",
        "Keyboard accessibility guidance.",
        vec![sample_card("Guidance", text_block("Make every action reachable by keyboard: Tab order, shortcuts, and arrow navigation.").font_size(13.0), r#"Keyboard accessibility: Tab, arrows, shortcuts"#), sample_card("Examples", text_block("Buttons, toggles, and sliders all support keyboard input by default in WinUI.").font_size(13.0), r#"Default WinUI keyboard support"#)],
    )
}
