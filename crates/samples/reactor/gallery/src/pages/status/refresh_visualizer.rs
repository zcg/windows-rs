use crate::controls::*;
use windows_reactor::*;

pub struct RefreshVisualizerPage;

impl Component for RefreshVisualizerPage {
    type Message = ();
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self
    }

    fn view(&self, _: &(), _: &mut ViewContext<Self>) -> View {
        page_content(
            "RefreshVisualizer",
            "Draws the pull-to-refresh spinner for a RefreshContainer.",
            [
                KeyedView::new(
                    "basic",
                    sample_card(
                        "Visualizer",
                        StackPanel::new().spacing(8.0).children((
                            RefreshVisualizer::new().height(120.0),
                            TextBlock::new()
                                .text("The visualizer shows its spinner while a refresh runs.")
                                .opacity(0.6),
                        )),
                        "RefreshVisualizer::new().orientation(RefreshVisualizerOrientation::Normal)",
                    ),
                ),
                KeyedView::new(
                    "orientation",
                    sample_card(
                        "Orientation",
                        RefreshVisualizer::new()
                            .orientation(RefreshVisualizerOrientation::Normal)
                            .height(120.0),
                        "RefreshVisualizerOrientation::Auto | Normal | Rotate90DegreesCounterclockwise",
                    ),
                ),
            ],
        )
    }
}
