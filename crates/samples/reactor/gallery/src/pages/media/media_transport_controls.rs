use crate::controls::*;
use windows_reactor::*;

pub struct MediaTransportControlsPage;

impl Component for MediaTransportControlsPage {
    type Message = ();
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self
    }

    fn view(&self, _: &(), _: &mut ViewContext<Self>) -> View {
        page_content(
            "MediaTransportControls",
            "The play, seek, and volume bar shown over media playback.",
            [
                KeyedView::new(
                    "basic",
                    sample_card(
                        "Transport bar",
                        Border::new()
                            .background(ThemeBrush::CardBackground)
                            .padding(Thickness::new(12.0, 8.0, 12.0, 8.0))
                            .content(MediaTransportControls::new().height(64.0)),
                        "MediaTransportControls::new()",
                    ),
                ),
                KeyedView::new(
                    "usage",
                    sample_card(
                        "Relationship to MediaPlayerElement",
                        InfoBar::new()
                            .severity(InfoBarSeverity::Informational)
                            .title("Normally created by the player")
                            .message(
                                "MediaPlayerElement creates this bar when \
                                 AreTransportControlsEnabled is set, and drives it from its own \
                                 MediaPlayer. Assign a standalone transport control only when a \
                                 custom playback surface needs the standard commands.",
                            )
                            .is_open(true),
                        "MediaPlayerElement::new().are_transport_controls_enabled(true)",
                    ),
                ),
            ],
        )
    }
}
