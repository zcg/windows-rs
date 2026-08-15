use crate::controls::*;
use std::time::Duration;
use windows_reactor::*;

pub fn animated_icon_page(_: &(), cx: &mut RenderCx) -> Element {
    let (play, set_play) = cx.use_state(false);

    let icon: Element = button("Play Animation")
        .icon(Icon::symbol(Symbol::Play))
        .accent()
        .animate(AnimationConfig {
            opacity: Some(1.0),
            scale: Some(if play { 1.15 } else { 1.0 }),
            duration: Duration::from_millis(300),
            easing: Easing::EaseOut,
        })
        .on_click(move || set_play.call(!play))
        .into();

    page_content(
        "AnimatedIcon",
        "An animated icon driven by state transitions.",
        vec![
            sample_card(
                "Animated State Icon",
                vstack((
                    icon,
                    text_block("WinUI AnimatedIcon plays Lottie animations on state changes. This sample animates a symbol icon with the reactor animation API.")
                        .font_size(12.0)
                        .opacity(0.6),
                ))
                .spacing(8.0),
                r#".animate(AnimationConfig { scale, duration, easing }) // state-driven icon animation"#,
            ),
        ],
    )
}
