use crate::controls::*;
use windows_reactor::*;

pub fn connected_animation_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "Connected Animations",
        "Connected animations move an element between two views during navigation.",
        vec![
            sample_card(
                "Overview",
                text_block("ConnectedAnimationService animates a shared element across pages.").font_size(13.0),
                r#"ConnectedAnimationService.GetForCurrentView().PrepareToAnimate(...)"#,
            ),
        ],
    )
}