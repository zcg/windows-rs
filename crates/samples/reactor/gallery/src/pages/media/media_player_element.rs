use crate::controls::*;
use windows_reactor::*;

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
            [
                KeyedView::new(
                    "basic",
                    sample_card(
                        "Basic MediaPlayerElement",
                        MediaPlayerElement::new()
                            .are_transport_controls_enabled(true)
                            .auto_play(false)
                            .width(480.0)
                            .height(270.0),
                    ),
                ),
                KeyedView::new(
                    "stretch",
                    sample_card(
                        "Stretch",
                        MediaPlayerElement::new()
                            .are_transport_controls_enabled(true)
                            .stretch(Stretch::Uniform)
                            .width(320.0)
                            .height(180.0),
                    ),
                ),
            ],
        )
    }
}
