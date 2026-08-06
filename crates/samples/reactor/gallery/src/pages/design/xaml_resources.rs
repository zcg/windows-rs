use crate::controls::*;
use windows_reactor::*;

pub fn xaml_resources_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "XAML Resources",
        "Resources are reusable objects such as colors, brushes, and styles.",
        vec![
            sample_card(
                "Reusable Values",
                vstack((
                    text_block("Accent").foreground(Color::rgb(0, 120, 212)).font_size(18.0).bold(),
                    text_block("Success").foreground(Color::rgb(16, 124, 16)).font_size(18.0).bold(),
                    text_block("Warning").foreground(Color::rgb(157, 93, 0)).font_size(18.0).bold(),
                    text_block("Danger").foreground(Color::rgb(196, 43, 28)).font_size(18.0).bold(),
                ))
                .spacing(8.0),
                r#"text_block("Accent").foreground(Color::rgb(0, 120, 212))"#,
            ),
            sample_card(
                "Theme-aware Colors",
                vstack((
                    text_block("Primary text uses ThemeRef::PrimaryText").foreground(ThemeRef::PrimaryText),
                    text_block("Secondary text uses ThemeRef::SecondaryText").foreground(ThemeRef::SecondaryText),
                ))
                .spacing(4.0),
                r#"foreground(ThemeRef::PrimaryText) // theme-aware resource"#,
            ),
        ],
    )
}