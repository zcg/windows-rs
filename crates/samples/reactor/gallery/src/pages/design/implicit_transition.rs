use crate::controls::*;
use windows_reactor::*;

pub fn implicit_transition_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "Implicit Transitions",
        "Implicit transitions animate property changes on a target element.",
        vec![
            sample_card(
                "Overview",
                vstack(vec![text_block("ElementCompositionPreview SetImplicitShowAnimation/SetImplicitHideAnimation drive implicit transitions.").font_size(13.0).into(), text_block("Reactor supports them through the composition layer.").font_size(13.0).into()]).spacing(8.0),
                r#"ElementCompositionPreview::SetImplicitShowAnimation(...)"#,
            ),
        ],
    )
}