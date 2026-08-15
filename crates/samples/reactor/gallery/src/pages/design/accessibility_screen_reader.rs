use crate::controls::*;
use windows_reactor::*;

pub fn accessibility_screen_reader_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "Accessibility: Screen Reader",
        "Screen reader support guidance.",
        vec![sample_card("Guidance", text_block("Provide labels and descriptive text so screen readers can announce controls.").font_size(13.0), r#"AutomationProperties.Name for screen readers"#), sample_card("Reactor", text_block("Use accessibility modifiers to set automation name, help text, and live settings.").font_size(13.0), r#"Modifiers::accessibility(...)"#)],
    )
}
