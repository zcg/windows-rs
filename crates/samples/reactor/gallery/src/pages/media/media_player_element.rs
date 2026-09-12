use crate::controls::*;
use windows::Foundation::Uri;
use windows::Media::Core::MediaSource;
use windows_core::{HSTRING, IUnknown, Interface};
use windows_reactor::*;

const SAMPLE: &str =
    "https://test-videos.co.uk/vids/bigbuckbunny/mp4/h264/360/Big_Buck_Bunny_360_10s_1MB.mp4";

pub struct MediaPlayerElementPage;

impl Component for MediaPlayerElementPage {
    type Message = ();
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self
    }

    fn view(&self, _: &(), _: &mut ViewContext<Self>) -> View {
        let player: View = match build_player() {
            Ok(player) => player.into(),
            Err(error) => TextBlock::new()
                .text(format!("Playback unavailable: {error}"))
                .into(),
        };

        page_content(
            "MediaPlayerElement",
            "Hosts a MediaPlayer for audio/video playback with optional transport controls.",
            [KeyedView::new(
                "basic",
                sample_card(
                    "Basic MediaPlayerElement",
                    player,
                    "MediaSource::CreateFromUri(&uri)?; MediaPlayerElement::new().source(source)",
                ),
            )],
        )
    }
}

/// Builds the sample player. `MediaPlayerElement::source` takes a media
/// playback source object, so the URI is wrapped in a `MediaSource` first.
fn build_player() -> windows::core::Result<MediaPlayerElement> {
    let uri = Uri::CreateUri(&HSTRING::from(SAMPLE))?;
    let source = MediaSource::CreateFromUri(&uri)?;

    Ok(MediaPlayerElement::new()
        .source(source.cast::<IUnknown>()?)
        .are_transport_controls_enabled(true)
        .auto_play(false)
        .width(480.0)
        .height(270.0))
}
