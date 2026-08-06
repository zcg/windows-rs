use crate::controls::*;
use windows_reactor::*;

pub fn easing_function_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "Easing Functions",
        "Easing functions control how animations accelerate and decelerate.",
        vec![
            sample_card(
                "Overview",
                vstack(vec![text_block("Reactor exposes composition easing through the animation system.").font_size(13.0).into(), text_block("Common easings: linear, ease-out, ease-in, and back.").font_size(13.0).into()]).spacing(8.0),
                r#"compositor.create_linear_easing_function() // and other easings"#,
            ),
        ],
    )
}