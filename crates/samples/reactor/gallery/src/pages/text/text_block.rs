use crate::controls::*;
use windows_reactor::*;

pub fn text_block_page(_: &(), cx: &mut RenderCx) -> Element {
    let (selectable, set_selectable) = cx.use_state(true);

    let selectable_tb = if selectable {
        text_block("I am a selectable TextBlock with custom SelectionHighlightColor.").selectable()
    } else {
        text_block("I am a selectable TextBlock with custom SelectionHighlightColor.")
    };

    page_content(
        "TextBlock",
        "A TextBlock displays a line or paragraph of text, with support for styling, wrapping, and selection.",
        vec![
            sample_card(
                "Simple TextBlock",
                text_block("I am a TextBlock."),
                r#"text_block("I am a TextBlock.")"#,
            ),
            sample_card(
                "Styled TextBlock",
                text_block("I am a styled TextBlock.")
                    .font_family("Comic Sans MS")
                    .font_size(20.0),
                r#"text_block("I am a styled TextBlock.").font_family("Comic Sans MS").font_size(20.0)"#,
            ),
            sample_card(
                "Various Properties",
                text_block("I am super excited to be here!")
                    .font_family("Arial")
                    .font_size(24.0)
                    .foreground(Color::rgb(100, 149, 237)) // CornflowerBlue
                    .wrap(),
                r#"text_block("...").font_family("Arial").font_size(24.0).foreground(...).wrap()"#,
            ),
            sample_card(
                "Inline Text Elements",
                vstack((
                    text_block("Text in a TextBlock doesn't have to be a simple string.")
                        .font_family("Times New Roman"),
                    text_block("Text can be bold, italic, or underlined.").bold(),
                ))
                .spacing(0.0),
                r#"multiple text_block elements approximate Run / LineBreak / Span"#,
            ),
            sample_card(
                "Selectable TextBlock",
                vstack((
                    selectable_tb,
                    ToggleSwitch::new(selectable)
                        .header("IsTextSelectionEnabled")
                        .on_toggled(move |v| set_selectable.call(v)),
                ))
                .spacing(8.0),
                r#"text_block("...").selectable() + ToggleSwitch::new(selectable).on_toggled(...)"#,
            ),
        ],
    )
}
