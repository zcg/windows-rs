use crate::controls::*;
use windows_reactor::*;

pub fn media_player_element_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "MediaPlayerElement",
        "Play audio and video content.",
        vec![sample_card("Overview", text_block("MediaPlayerElement hosts a MediaPlayer for audio/video playback with transport controls.").font_size(13.0), r#"MediaPlayerElement Source + AreTransportControlsEnabled"#)],
    )
}