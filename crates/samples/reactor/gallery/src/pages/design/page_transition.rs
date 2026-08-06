use crate::controls::*;
use windows_reactor::*;
use std::time::Duration;

pub fn page_transition_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "Page Transitions",
        "Page transitions animate navigation between pages.",
        vec![
            sample_card(
                "Animated Page Content",
                vstack((
                    text_block("Page content animates in with a fade + scale transition.").font_size(14.0),
                    hstack((
                        border(text_block("Section A")).padding(Thickness::uniform(16.0)).background(ThemeRef::CardBackground).corner_radius(8.0)
                            .animate(AnimationConfig { opacity: Some(1.0), scale: Some(1.0), duration: Duration::from_millis(300), easing: Easing::EaseOut }),
                        border(text_block("Section B")).padding(Thickness::uniform(16.0)).background(ThemeRef::CardBackground).corner_radius(8.0)
                            .animate(AnimationConfig { opacity: Some(1.0), scale: Some(1.0), duration: Duration::from_millis(500), easing: Easing::EaseInOut }),
                    ))
                    .spacing(8.0),
                    text_block("Staggered durations approximate navigation page transitions.").font_size(12.0).opacity(0.6),
                ))
                .spacing(8.0),
                r#".animate(AnimationConfig { duration, easing }) // staggered"#,
            ),
        ],
    )
}