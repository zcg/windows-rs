use crate::controls::*;
use windows_reactor::*;

pub struct AnimatedVisualPlayerPage;

impl Component for AnimatedVisualPlayerPage {
    type Message = ();
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self
    }

    fn view(&self, _: &(), _: &mut ViewContext<Self>) -> View {
        page_content(
            "AnimatedVisualPlayer",
            "Plays a Lottie animation inside the XAML tree.",
            [
                KeyedView::new(
                    "basic",
                    sample_card(
                        "Player surface",
                        Border::new()
                            .height(240.0)
                            .background(ThemeBrush::CardBackground)
                            .content(AnimatedVisualPlayer::new().auto_play(true).height(240.0)),
                        "AnimatedVisualPlayer::new().auto_play(true)",
                    ),
                ),
                KeyedView::new(
                    "source",
                    sample_card(
                        "Animation source",
                        InfoBar::new()
                            .severity(InfoBarSeverity::Warning)
                            .title("The player stays empty without an animation source")
                            .message(
                                "AnimatedVisualPlayer renders an IAnimatedVisualSource2, which is \
                                 compiled from a Lottie JSON file into a class that implements the \
                                 interface. The gallery ships no Lottie assets, so the surface \
                                 above stays blank until a source is assigned.",
                            )
                            .is_open(true),
                        "player.source(animated_visual_source)",
                    ),
                ),
            ],
        )
    }
}
