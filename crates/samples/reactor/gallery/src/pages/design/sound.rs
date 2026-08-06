use crate::controls::*;
use windows_reactor::*;

pub fn sound_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "Sound",
        "Sound effects provide audio feedback for user interactions.",
        vec![
            sample_card(
                "Overview",
                vstack(vec![text_block("Windows includes system sounds that can be played from code; reactor apps can trigger them through platform APIs.").font_size(13.0).into(), text_block("Common uses: confirmations, errors, and notifications.").font_size(13.0).into()]).spacing(8.0),
                r#"// Sound APIs are invoked through Windows platform APIs"#,
            ),
        ],
    )
}