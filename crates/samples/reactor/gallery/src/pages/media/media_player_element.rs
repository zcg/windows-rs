use crate::controls::*;
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
        page_content(
            "MediaPlayerElement",
            "Hosts a MediaPlayer for audio/video playback with optional transport controls.",
            [KeyedView::new(
                "basic",
                sample_card(
                    "Basic MediaPlayerElement",
                    MediaPlayerElement::new()
                        .source(SAMPLE)
                        .unwrap()
                        .are_transport_controls_enabled(true)
                        .auto_play(false)
                        .width(480.0)
                        .height(270.0),
                    "MediaPlayerElement::new().source(url).unwrap()",
                ),
            )],
        )
    }
}
