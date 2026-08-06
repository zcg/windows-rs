use crate::controls::*;
use windows_reactor::*;
use std::time::Duration;

pub fn connected_animation_page(_: &(), cx: &mut RenderCx) -> Element {
    let (moved, set_moved) = cx.use_state(false);

    let badge: Element = border(text_block("A"))
        .background(Color::rgb(0, 120, 212))
        .corner_radius(16.0)
        .width(48.0)
        .height(48.0)
        .transition(
            Some(AnimationConfig {
                opacity: Some(1.0),
                scale: Some(1.0),
                duration: Duration::from_millis(400),
                easing: Easing::EaseOut,
            }),
            None,
        )
        .margin(if moved {
            Thickness { left: 240.0, top: 0.0, right: 0.0, bottom: 0.0 }
        } else {
            Thickness::uniform(0.0)
        })
        .into();

    page_content(
        "Connected Animations",
        "Connected animations move an element between two views during navigation.",
        vec![
            sample_card(
                "Animated Element Movement",
                vstack((
                    badge,
                    button("Move Element").on_click(move || set_moved.call(!moved)),
                    text_block("ConnectedAnimationService animates a shared element across pages; this sample animates an element position with a transition.")
                        .font_size(12.0)
                        .opacity(0.6),
                ))
                .spacing(8.0),
                r#".transition(Some(AnimationConfig::fade_in(400ms))) + margin change"#,
            ),
        ],
    )
}