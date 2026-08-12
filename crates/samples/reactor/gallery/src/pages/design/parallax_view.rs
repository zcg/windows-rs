use crate::controls::*;
use windows_reactor::*;

pub fn parallax_view_page(_: &(), cx: &mut RenderCx) -> Element {
    let sv_ref = cx.use_ref::<Option<windows_core::IInspectable>>(None);
    let pv_ref = cx.use_ref::<Option<windows_core::IInspectable>>(None);
    let (status, set_status) = cx.use_state(String::from("Scroll the list, then attach the parallax source"));

    let on_sv_mounted = {
        let sv_ref = sv_ref.clone();
        Callback::new(move |native: Option<windows_core::IInspectable>| {
            if let Some(n) = native { *sv_ref.borrow_mut() = Some(n); }
        })
    };
    let on_pv_mounted = {
        let pv_ref = pv_ref.clone();
        Callback::new(move |native: Option<windows_core::IInspectable>| {
            if let Some(n) = native { *pv_ref.borrow_mut() = Some(n); }
        })
    };

    let background: Element = border(
        text_block("Parallax Background").font_size(16.0).bold().opacity(0.8),
    )
    .background(Color::rgb(0, 120, 212))
    .width(320.0)
    .height(200.0)
    .on_mounted(on_pv_mounted)
    .into();

    let foreground: Element = list_view(
        (0..20).map(|i| format!("Scrollable row {i}")).collect(),
        |item, _| {
            let t: Element = text_block(item.clone()).padding(Thickness::uniform(8.0)).into();
            t
        },
    )
    .build()
    .on_mounted(on_sv_mounted);

    let attach = {
        let sv_ref = sv_ref;
        let pv_ref = pv_ref;
        let set_status = set_status;
        move || {
            let sv = sv_ref.borrow().clone();
            let pv = pv_ref.borrow().clone();
            let (Some(sv), Some(pv)) = (sv, pv) else {
                set_status.call("Elements not mounted yet".into());
                return;
            };
            let sv = sv.cast::<UIElement>().ok();
            let pv = pv.cast::<UIElement>().ok();
            let (Some(sv), Some(pv)) = (sv, pv) else {
                set_status.call("Cast failed".into());
                return;
            };
            match parallax_attach_source(&pv, &sv) {
                Ok(()) => set_status.call("Parallax source attached - scroll to see the effect".into()),
                Err(_) => set_status.call("Attach failed".into()),
            }
        }
    };

    page_content(
        "ParallaxView",
        "Parallax scrolling links a background element to a scrolling source.",
        vec![
            sample_card(
                "Scroll-linked Background",
                vstack((
                    background,
                    foreground,
                    hstack((
                        button("Attach Parallax Source").on_click(attach),
                    ))
                    .spacing(8.0),
                    text_block(status).font_size(12.0).opacity(0.6),
                ))
                .spacing(8.0),
                r#"parallax_attach_source(&parallax, &scroll_viewer) // scroll-linked background"#,
            ),
        ],
    )
}