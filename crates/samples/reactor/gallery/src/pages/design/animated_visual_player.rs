use crate::controls::*;
use windows_reactor::*;

pub fn animated_visual_player_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "AnimatedVisualPlayer",
        "Plays animated visuals (Lottie, etc.) in a control.",
        vec![sample_card("Overview", text_block("AnimatedVisualPlayer renders AnimatedVisuals from JSON/Lottie or code.").font_size(13.0), r#"AnimatedVisualPlayer.Source = AnimatedVisual"#)],
    )
}