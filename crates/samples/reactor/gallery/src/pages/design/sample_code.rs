use crate::controls::*;
use windows_reactor::*;

pub fn sample_code_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "Sample Code",
        "A collection of runnable samples and patterns.",
        vec![
            sample_card(
                "Overview",
                text_block("This Gallery is itself a sample: every page shows its source snippet in the card footer.").font_size(13.0),
                r#"Each sample_card embeds the source snippet"#,
            ),
        ],
    )
}