use crate::controls::*;
use windows_reactor::*;

pub fn xaml_ui_command_page(_: &(), cx: &mut RenderCx) -> Element {
    let (status, set_status) = cx.use_state(String::from("Ready"));

    page_content(
        "XamlUICommand",
        "A command with label, icon, and optional keyboard accelerator.",
        vec![
            sample_card(
                "Commands on Buttons",
                vstack((
                    hstack((
                        button("Copy").icon(Icon::symbol(Symbol::Copy)).on_click({
                            let set_status = set_status.clone();
                            move || set_status.call("Copy command invoked".into())
                        }),
                        button("Paste").icon(Icon::symbol(Symbol::Paste)).on_click({
                            let set_status = set_status.clone();
                            move || set_status.call("Paste command invoked".into())
                        }),
                        button("Save").icon(Icon::symbol(Symbol::Save)).on_click({
                            let set_status = set_status.clone();
                            move || set_status.call("Save command invoked".into())
                        }),
                    ))
                    .spacing(8.0),
                    text_block(status).opacity(0.6),
                ))
                .spacing(8.0),
                r#"button("Copy").icon(Icon::symbol(Symbol::Copy)).on_click(...)"#,
            ),
        ],
    )
}