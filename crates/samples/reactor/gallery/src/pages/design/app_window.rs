use crate::controls::*;
use windows_reactor::*;

fn counter_window_view(cx: &mut RenderCx, heading: &str) -> Element {
    let (count, set_count) = cx.use_state(0_i32);
    vstack((
        text_block(heading).bold().font_size(20.0),
        text_block(format!("Count: {count}")).font_size(28.0),
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
    .spacing(12.0)
    .margin(Thickness::uniform(24.0))
    .into()
}

pub fn app_window_page(_: &(), cx: &mut RenderCx) -> Element {
    let (opened, set_opened) = cx.use_state(0_u32);

    let open = {
        let set_opened = set_opened;
        move || {
            let n = opened + 1;
            match ReactorWindow::new()
                .title(format!("AppWindow #{n}"))
                .inner_size(320.0, 220.0)
                .render(move |cx| counter_window_view(cx, "Independent window"))
            {
                Ok(_handle) => set_opened.call(n),
                Err(err) => eprintln!("failed to open window: {err}"),
            }
        }
    };

    page_content(
        "AppWindow",
        "Open independent windows with their own content.",
        vec![
            sample_card(
                "Open a Secondary Window",
                vstack((
                    button("Open New Window").on_click(open),
                    text_block(format!("Windows opened: {opened}")).opacity(0.6),
                ))
                .spacing(8.0),
                r#"ReactorWindow::new().title("...").inner_size(320.0, 220.0).render(move |cx| view)"#,
            ),
        ],
    )
}