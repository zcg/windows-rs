use crate::controls::*;
use windows_reactor::*;

pub struct AnimatedIconPage;

impl Component for AnimatedIconPage {
    type Message = ();
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self
    }

    fn view(&self, _: &(), _: &mut ViewContext<Self>) -> View {
        page_content(
            "AnimatedIcon",
            "Plays a Lottie animation in place of a static icon.",
            [
                KeyedView::new(
                    "basic",
                    sample_card(
                        "Icon surface",
                        Border::new()
                            .width(64.0)
                            .height(64.0)
                            .background(ThemeBrush::CardBackground)
                            .content(AnimatedIcon::new().width(40.0).height(40.0)),
                    ),
                ),
                KeyedView::new(
                    "source",
                    sample_card(
                        "Animation source",
                        InfoBar::new()
                            .severity(InfoBarSeverity::Warning)
                            .title("The icon stays empty without an animation source")
                            .message(
                                "AnimatedIcon renders an IAnimatedVisualSource2, compiled from a \
                                 Lottie file, and switches between its named states through the \
                                 AnimatedIconState attached property. With no source there is no \
                                 glyph to draw, so the surface above stays empty; use SymbolIcon \
                                 or FontIcon when no animation is available.",
                            )
                            .is_open(true),
                    ),
                ),
            ],
        )
    }
}
