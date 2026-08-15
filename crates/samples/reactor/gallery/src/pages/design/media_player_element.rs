use crate::controls::*;
use windows_reactor::*;

pub fn media_player_element_page(_: &(), cx: &mut RenderCx) -> Element {
    let (controls_on, set_controls_on) = cx.use_state(true);

    page_content(
        "MediaPlayerElement",
        "Hosts a MediaPlayer for audio and video playback with transport controls.",
        vec![
            sample_card(
                "Video Playback",
                vstack((
                    media_player_element("https://www.w3schools.com/html/mov_bbb.mp4")
                        .transport_controls(controls_on)
                        .auto_play(false)
                        .height(240.0)
                        .width(420.0),
                    ToggleSwitch::new(controls_on)
                        .header("AreTransportControlsEnabled")
                        .on_toggled(move |v| set_controls_on.call(v)),
                    text_block("Plays a remote MP4 via Windows.Media.Core.MediaSource. Requires network access for the sample clip.")
                        .font_size(12.0)
                        .opacity(0.6),
                ))
                .spacing(8.0),
                r#"media_player_element("https://.../mov_bbb.mp4").transport_controls(true).auto_play(false)"#,
            ),
        ],
    )
}
