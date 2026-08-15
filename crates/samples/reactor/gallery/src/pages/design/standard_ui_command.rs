use crate::controls::*;
use windows_reactor::*;

pub fn standard_ui_command_page(_: &(), cx: &mut RenderCx) -> Element {
    let (status, set_status) = cx.use_state(String::from("Choose a command"));

    page_content(
        "StandardUICommand",
        "Standard commands with built-in icons and labels.",
        vec![sample_card(
            "Standard Commands in a CommandBar",
            vstack((
                command_bar(vec![
                    CommandBarCommandDef::Button {
                        label: "Copy".into(),
                        icon: Some(Symbol::Copy.into()),
                    },
                    CommandBarCommandDef::Button {
                        label: "Paste".into(),
                        icon: Some(Symbol::Paste.into()),
                    },
                    CommandBarCommandDef::Button {
                        label: "Save".into(),
                        icon: Some(Symbol::Save.into()),
                    },
                    CommandBarCommandDef::Separator,
                    CommandBarCommandDef::Button {
                        label: "Delete".into(),
                        icon: Some(Symbol::Delete.into()),
                    },
                ])
                .on_click({
                    let set_status = set_status;
                    move |label| set_status.call(format!("Last command: {label}"))
                }),
                text_block(status).opacity(0.6),
            ))
            .spacing(8.0),
            r#"StandardUICommandKind.Copy / Paste / Save rendered as CommandBar buttons"#,
        )],
    )
}
