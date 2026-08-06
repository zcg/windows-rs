use crate::controls::*;
use windows_reactor::*;

pub fn sound_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "Sound",
        "Sound effects provide audio feedback for user interactions.",
        vec![
            sample_card(
                "Overview",
                vstack(vec![
                    text_block("Windows system sounds (SystemSounds) can be played from code through the Windows.Media APIs.").font_size(13.0).into(),
                    text_block("Reactor exposes system capabilities such as Clipboard; sound playback can be added once the media metadata is available in the toolchain winmd.").font_size(13.0).into(),
                ]).spacing(8.0),
                r#"Windows.Media.SystemSounds.Asterisk().Play()"#,
            ),
        ],
    )
}