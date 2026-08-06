use crate::controls::*;
use windows_reactor::*;

pub fn accessibility_color_contrast_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "Accessibility: Color Contrast",
        "Color contrast guidance for accessible apps.",
        vec![sample_card("Guidance", text_block("Ensure color contrast meets WCAG AA (4.5:1 for normal text).").font_size(13.0), r#"WCAG AA contrast ratio 4.5:1"#), sample_card("Tools", text_block("Windows includes a built-in high-contrast theme and contrast checker.").font_size(13.0), r#"High-contrast themes"#)],
    )
}