use crate::controls::*;
use windows_reactor::*;

pub fn templates_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "Templates",
        "Control templates define the visual structure of controls.",
        vec![
            sample_card(
                "Styled Controls (Template-like)",
                vstack((
                    button("Primary").accent().width(140.0),
                    button("Secondary").subtle().width(140.0),
                    button("Disabled").enabled(false).width(140.0),
                ))
                .spacing(8.0),
                r#"button("Primary").accent() // style variants emulate template styling"#,
            ),
            sample_card(
                "Reusable Card Layout",
                hstack((
                    sample_card("Card 1", text_block("Reusable layout"), r#"sample_card(...)"#),
                    sample_card("Card 2", text_block("Composed from primitives"), r#"sample_card(...)"#),
                ))
                .spacing(8.0),
                r#"sample_card(title, sample, source) // a reusable template in code"#,
            ),
        ],
    )
}