use crate::controls::*;
use windows_reactor::*;

pub fn compact_sizing_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "Compact Sizing",
        "Compact sizing reduces the padding and spacing of controls for dense layouts.",
        vec![sample_card(
            "Default vs Compact",
            vstack((
                text_block("Default:"),
                hstack((button("Save"), button("Cancel"), ToggleSwitch::new(true))).spacing(8.0),
                text_block("Compact:"),
                hstack((
                    button("Save").padding(Thickness::uniform(4.0)),
                    button("Cancel").padding(Thickness::uniform(4.0)),
                    ToggleSwitch::new(true),
                ))
                .spacing(8.0),
            ))
            .spacing(8.0),
            r#"button("Save").padding(Thickness::uniform(4.0)) // compact control variant"#,
        )],
    )
}
