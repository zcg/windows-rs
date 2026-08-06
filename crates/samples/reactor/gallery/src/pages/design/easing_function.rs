use crate::controls::*;
use windows_reactor::*;
use std::time::Duration;

pub fn easing_function_page(_: &(), _cx: &mut RenderCx) -> Element {
    let block = |color: (u8, u8, u8), easing: Easing| -> Element {
        border(Element::Empty)
            .background(Color::rgb(color.0, color.1, color.2))
            .width(120.0)
            .height(48.0)
            .corner_radius(6.0)
            .animate(AnimationConfig {
                opacity: Some(1.0),
                scale: Some(1.0),
                duration: Duration::from_millis(700),
                easing,
            })
            .into()
    };

    page_content(
        "Easing Functions",
        "Easing functions control how animations accelerate and decelerate.",
        vec![
            sample_card(
                "Easing Comparison",
                vstack((
                    text_block("Linear").font_size(12.0).opacity(0.6),
                    block((0, 120, 215), Easing::Linear),
                    text_block("EaseOut").font_size(12.0).opacity(0.6),
                    block((16, 124, 16), Easing::EaseOut),
                    text_block("EaseIn").font_size(12.0).opacity(0.6),
                    block((157, 93, 0), Easing::EaseIn),
                    text_block("EaseInOut").font_size(12.0).opacity(0.6),
                    block((196, 43, 28), Easing::EaseInOut),
                ))
                .spacing(6.0),
                r#".animate(AnimationConfig { scale: Some(1.0), duration, easing })"#,
            ),
        ],
    )
}