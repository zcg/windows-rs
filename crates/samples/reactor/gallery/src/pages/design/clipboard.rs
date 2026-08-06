use crate::controls::*;
use windows_reactor::*;

pub fn clipboard_page(_: &(), cx: &mut RenderCx) -> Element {
    let (status, set_status) = cx.use_state(String::from("Clipboard ready"));
    let (text, set_text) = cx.use_state(String::from("Hello from WinUI Gallery!"));

    page_content(
        "Clipboard",
        "Read and write text to the system clipboard.",
        vec![
            sample_card(
                "Copy and Paste",
                vstack((
                    text_box(text.clone())
                        .on_text_changed({
                            let set_text = set_text.clone();
                            move |v| set_text.call(v)
                        })
                        .width(360.0),
                    hstack((
                        button("Copy to Clipboard")
                            .on_click({
                                let text = text.clone();
                                let set_status = set_status.clone();
                                move || {
                                    let ok = clipboard_set_text(&text).is_ok();
                                    set_status.call(if ok { "Copied!".into() } else { "Copy failed".into() });
                                }
                            }),
                        button("Paste from Clipboard")
                            .on_click({
                                let set_text = set_text.clone();
                                let set_status = set_status.clone();
                                move || {
                                    match clipboard_get_text() {
                                        Some(v) => { set_text.call(v); set_status.call("Pasted!".into()); }
                                        None => set_status.call("Clipboard has no text".into()),
                                    }
                                }
                            }),
                    ))
                    .spacing(8.0),
                    text_block(status).opacity(0.6),
                ))
                .spacing(8.0),
                r#"clipboard_set_text(text);
clipboard_get_text();"#,
            ),
        ],
    )
}