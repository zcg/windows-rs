use crate::controls::*;
use windows_reactor::*;
use std::time::Duration;

pub fn theme_transition_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "Theme Transitions",
        "Theme transitions animate visual changes when the theme switches.",
        vec![
            sample_card(
                "Animated Theme Accent",
                vstack((
                    border(text_block("Theme-aware surface"))
                        .padding(Thickness::uniform(20.0))
                        .background(ThemeRef::CardBackground)
                        .corner_radius(8.0)
                        .animate(AnimationConfig {
                            opacity: Some(1.0),
                            scale: Some(1.0),
                            duration: Duration::from_millis(500),
                            easing: Easing::EaseOut,
                        }),
                    text_block("Reactor animates element appearance on mount; theme switches restyle via ThemeRef resources.").font_size(12.0).opacity(0.6),
                ))
                .spacing(8.0),
                r#".animate(AnimationConfig::fade_in(500ms)) + ThemeRef resources"#,
            ),
        ],
    )
}