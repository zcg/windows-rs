use crate::controls::*;
use windows_reactor::*;
use std::time::Duration;

pub fn animated_visual_player_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "AnimatedVisualPlayer",
        "Plays animated visuals (Lottie, etc.) in a control.",
        vec![
            sample_card(
                "Animated Visual Playback",
                vstack((
                    border(Element::Empty)
                        .background(Color::rgb(0, 120, 212))
                        .width(120.0)
                        .height(120.0)
                        .corner_radius(12.0)
                        .animate(AnimationConfig {
                            opacity: Some(1.0),
                            scale: Some(1.0),
                            duration: Duration::from_millis(600),
                            easing: Easing::EaseInOut,
                        }),
                    text_block("AnimatedVisualPlayer renders Lottie/JSON visuals. This sample plays a composition animation through the reactor animation API.")
                        .font_size(12.0)
                        .opacity(0.6),
                ))
                .spacing(8.0),
                r#".animate(AnimationConfig { duration, easing }) // plays on mount"#,
            ),
        ],
    )
}