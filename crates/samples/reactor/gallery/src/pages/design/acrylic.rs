use crate::controls::*;
use windows_reactor::*;

pub fn acrylic_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "Acrylic",
        "Acrylic is a translucent material that blurs and tints the content behind a surface.",
        vec![
            sample_card(
                "Window Backdrop Acrylic",
                vstack((
                    text_block("Acrylic can be applied as the window backdrop. Use set_backdrop to switch the current window to Acrylic:"),
                    button("Apply Acrylic Backdrop")
                        .on_click(|| set_backdrop(Some(Backdrop::Acrylic))),
                    button("Reset to Mica")
                        .on_click(|| set_backdrop(Some(Backdrop::Mica))),
                ))
                .spacing(8.0),
                r#"set_backdrop(Some(Backdrop::Acrylic));"#,
            ),
            sample_card(
                "Usage Guidance",
                vstack((
                    text_block("Use Acrylic for flyouts, panels, and transient surfaces.").font_size(13.0),
                    text_block("Fall back to a solid color when the app is inactive or in high contrast.").font_size(13.0),
                ))
                .spacing(4.0),
                r#"Acrylic falls back to solid when inactive."#,
            ),
        ],
    )
}