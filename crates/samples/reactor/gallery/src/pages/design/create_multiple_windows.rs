use crate::controls::*;
use windows_reactor::*;

fn mini_view(cx: &mut RenderCx, label: &str) -> Element {
    let (count, set_count) = cx.use_state(0_i32);
    vstack((
        text_block(format!("{label} - {count}")).bold().font_size(18.0),
        hstack((
            button("-").on_click({
                let s = set_count.clone();
                move || s.call(count - 1)
            }),
            button("+").on_click({
                let s = set_count.clone();
                move || s.call(count + 1)
            }),
        ))
        .spacing(8.0),
    ))
    .spacing(8.0)
    .margin(Thickness::uniform(16.0))
    .into()
}

pub fn create_multiple_windows_page(_: &(), cx: &mut RenderCx) -> Element {
    let (count, set_count) = cx.use_state(0_u32);

    let open = {
        let set_count = set_count;
        move || {
            let n = count + 1;
            match ReactorWindow::new()
                .title(format!("Window {n}"))
                .inner_size(260.0, 160.0)
                .render(move |cx| mini_view(cx, "Window"))
            {
                Ok(_handle) => set_count.call(n),
                Err(err) => eprintln!("failed to open window: {err}"),
            }
        }
    };

    page_content(
        "Create Multiple Windows",
        "Open more than one window for an app.",
        vec![
            sample_card(
                "Multiple Independent Windows",
                vstack((
                    button("Open Another Window").on_click(open),
                    text_block(format!("Windows opened: {count}")).opacity(0.6),
                ))
                .spacing(8.0),
                r#"ReactorWindow::new().render(move |cx| view) // independent per-window state"#,
            ),
        ],
    )
}