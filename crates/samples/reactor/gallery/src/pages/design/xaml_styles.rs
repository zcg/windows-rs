use crate::controls::*;
use windows_reactor::*;

pub fn xaml_styles_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "XAML Styles",
        "Styles group property values applied to controls.",
        vec![
            sample_card(
                "Button Style Variants",
                vstack((
                    text_block("Accent").font_size(12.0).opacity(0.6),
                    button("Save").accent().width(140.0),
                    text_block("Subtle").font_size(12.0).opacity(0.6),
                    button("Cancel").subtle().width(140.0),
                    text_block("Text link").font_size(12.0).opacity(0.6),
                    button("Learn more").text_link().width(140.0),
                ))
                .spacing(6.0),
                r#"button("Save").accent() / .subtle() / .text_link()"#,
            ),
            sample_card(
                "Shared Appearance via Helpers",
                hstack((
                    border(text_block("Card A")).corner_radius(8.0).padding(Thickness::uniform(12.0)).background(ThemeRef::CardBackground),
                    border(text_block("Card B")).corner_radius(8.0).padding(Thickness::uniform(12.0)).background(ThemeRef::CardBackground),
                ))
                .spacing(8.0),
                r#"border(...).corner_radius(8.0).background(ThemeRef::CardBackground) // consistent style"#,
            ),
        ],
    )
}