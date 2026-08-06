use crate::controls::*;
use windows_reactor::*;

pub fn scratch_pad_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "ScratchPad",
        "A blank canvas for trying WinUI APIs.",
        vec![
            sample_card(
                "Overview",
                text_block("Use this page as a scratch area when experimenting with new controls and layouts.").font_size(13.0),
                r#"ScratchPad: throwaway experimentation area"#,
            ),
        ],
    )
}