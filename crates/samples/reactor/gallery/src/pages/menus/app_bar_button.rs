use crate::controls::*;
use windows_reactor::*;

pub fn app_bar_button_page(_: &(), cx: &mut RenderCx) -> Element {
    let (status, set_status) = cx.use_state(String::from("Ready"));

    page_content(
        "AppBarButton",
        "A button used in an AppBar or CommandBar, with an optional icon and label.",
        vec![
            sample_card(
                "AppBarButtons in a CommandBar",
                vstack((
                    command_bar(vec![
                        CommandBarCommandDef::Button { label: "Add".into(), icon: Some(Symbol::Add.into()) },
                        CommandBarCommandDef::Button { label: "Edit".into(), icon: Some(Symbol::Edit.into()) },
                        CommandBarCommandDef::Button { label: "Delete".into(), icon: Some(Symbol::Delete.into()) },
                    ])
                    .on_click({
                        let set_status = set_status;
                        move |label| set_status.call(format!("Last command: {label}"))
                    }),
                    text_block(status).opacity(0.6),
                ))
                .spacing(8.0),
                r#"command_bar(vec![CommandBarCommandDef::Button { label, icon }]).on_click(...)"#,
            ),
            sample_card(
                "Icon-only Buttons",
                command_bar(vec![
                    CommandBarCommandDef::Button { label: String::new(), icon: Some(Symbol::Add.into()) },
                    CommandBarCommandDef::Button { label: String::new(), icon: Some(Symbol::Edit.into()) },
                    CommandBarCommandDef::Button { label: String::new(), icon: Some(Symbol::Delete.into()) },
                ]),
                r#"CommandBarCommandDef::Button { label: String::new(), icon: Some(Symbol::Add.into()) }"#,
            ),
        ],
    )
}