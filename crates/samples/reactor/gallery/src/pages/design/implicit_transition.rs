use crate::controls::*;
use std::time::Duration;
use windows_reactor::*;

pub fn implicit_transition_page(_: &(), cx: &mut RenderCx) -> Element {
    let (visible, set_visible) = cx.use_state(true);

    let block = if visible {
        Some(
            border(text_block("Fades in/out with implicit transitions"))
                .padding(Thickness::uniform(16.0))
                .background(ThemeRef::CardBackground)
                .corner_radius(8.0)
                .transition(
                    Some(AnimationConfig::fade_in(Duration::from_millis(400))),
                    Some(AnimationConfig::fade_out(Duration::from_millis(400))),
                )
                .into(),
        )
    } else {
        None
    };

    page_content(
        "Implicit Transitions",
        "Implicit transitions animate property changes on a target element.",
        vec![sample_card(
            "Show / Hide with Transition",
            vstack((
                button("Toggle Visibility").on_click(move || set_visible.call(!visible)),
                block.unwrap_or(Element::Empty),
            ))
            .spacing(8.0),
            r#".transition(Some(AnimationConfig::fade_in(400ms)), Some(AnimationConfig::fade_out(400ms)))"#,
        )],
    )
}
