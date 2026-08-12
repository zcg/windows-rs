use crate::controls::*;
use windows_reactor::*;


pub fn connected_animation_page(_: &(), cx: &mut RenderCx) -> Element {
    let (status, set_status) = cx.use_state(String::from("Press to run a connected animation"));
    let element_ref = cx.use_ref::<Option<windows_core::IInspectable>>(None);

    let on_mounted = {
        let element_ref = element_ref.clone();
        Callback::new(move |native: Option<windows_core::IInspectable>| {
            if let Some(n) = native {
                *element_ref.borrow_mut() = Some(n);
            }
        })
    };

    let badge: Element = border(text_block("A"))
        .background(Color::rgb(0, 120, 212))
        .corner_radius(16.0)
        .width(56.0)
        .height(56.0)
        .on_mounted(on_mounted)
        .into();

    let run_animation = {
        let element_ref = element_ref;
        let set_status = set_status;
        move || {
            let borrowed = element_ref.borrow();
            let Some(element) = borrowed.as_ref() else {
                set_status.call("Element not mounted yet".into());
                return;
            };
            let view = element.cast::<UIElement>().ok();
            let Some(view) = view else {
                set_status.call("Cast failed".into());
                return;
            };
            match run_connected_animation("gallery-anim", &view) {
                Ok(true) => set_status.call("Connected animation started".into()),
                Ok(false) => set_status.call("Animation could not start".into()),
                Err(_) => set_status.call("ConnectedAnimationService failed".into()),
            }
        }
    };

    page_content(
        "Connected Animations",
        "Connected animations move an element between two views during navigation.",
        vec![
            sample_card(
                "ConnectedAnimationService",
                vstack((
                    badge,
                    button("Run Connected Animation").on_click(run_animation),
                    text_block(status).opacity(0.6),
                ))
                .spacing(12.0),
                r#"ConnectedAnimationService::GetForCurrentView()
    .PrepareToAnimate(key, element)?
    .TryStart(&target)"#,
            ),
        ],
    )
}