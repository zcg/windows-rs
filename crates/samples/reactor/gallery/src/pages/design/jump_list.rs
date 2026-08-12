use crate::controls::*;
use windows_reactor::*;

pub fn jump_list_page(_: &(), cx: &mut RenderCx) -> Element {
    let (status, set_status) = cx.use_state(String::from(
        "Add tasks to the taskbar jump list. Requires a packaged app identity (AUMID).",
    ));

    let add_tasks = {
        let set_status = set_status.clone();
        move || {
            let set_status_done = set_status.clone();
            if let Err(err) = jumplist_add_tasks(move |result| {
                let text = match result {
                    Ok(message) => message,
                    Err(err) => format!("Cannot update jump list: {err}"),
                };
                set_status_done.call(text);
            }) {
                set_status.call(format!("Cannot start: {err}"));
            }
        }
    };

    let clear_tasks = move || {
        let set_status_done = set_status.clone();
        if let Err(err) = jumplist_clear(move |result| {
            let text = match result {
                Ok(message) => message,
                Err(err) => format!("Cannot update jump list: {err}"),
            };
            set_status_done.call(text);
        }) {
            set_status.call(format!("Cannot start: {err}"));
        }
    };

    page_content(
        "Jump List",
        "Add tasks and custom groups to the taskbar jump list.",
        vec![sample_card(
            "Jump List Tasks",
            vstack((
                hstack((
                    button("Add tasks").on_click(add_tasks),
                    button("Clear tasks").on_click(clear_tasks),
                ))
                .spacing(8.0),
                text_block(status).opacity(0.6),
            ))
            .spacing(8.0),
            r#"let jump_list = JumpList::LoadCurrentAsync()?.GetResults()?;
jump_list.Items()?.Append(JumpListItem::CreateWithArguments("/compose", "New Message")?)?;
jump_list.SaveAsync()?;"#,
        )],
    )
}
