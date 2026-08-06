use crate::controls::*;
use windows_reactor::*;

pub fn app_bar_toggle_button_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "AppBarToggleButton",
        "A toggle button used in an AppBar or CommandBar, showing a checked state.",
        vec![
            sample_card(
                "Toggle Buttons in a CommandBar",
                command_bar(vec![
                    CommandBarCommandDef::Toggle { label: "Bold".into(), icon: Some(Symbol::Font.into()) },
                    CommandBarCommandDef::Toggle { label: "Italic".into(), icon: Some(Symbol::Italic.into()) },
                    CommandBarCommandDef::Toggle { label: "Underline".into(), icon: Some(Symbol::Underline.into()) },
                ]),
                r#"CommandBarCommandDef::Toggle { label, icon }"#,
            ),
        ],
    )
}