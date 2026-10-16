use crate::controls::*;
use windows_reactor::*;

pub struct VariableSizedWrapGridPage;

impl Component for VariableSizedWrapGridPage {
    type Message = ();
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self
    }

    fn view(&self, _: &(), _: &mut ViewContext<Self>) -> View {
        let tile = |text: &str| -> View {
            Border::new()
                .background(ThemeBrush::CardBackground)
                .padding(12.0)
                .content(TextBlock::new().text(text))
                .into()
        };
        let tiles: Vec<View> = ["1", "2", "3", "4", "5"].into_iter().map(tile).collect();
        page_content(
            "VariableSizedWrapGrid",
            "Arranges items in a grid with a fixed item size, wrapping into rows.",
            [KeyedView::new(
                "basic",
                sample_card(
                    "Wrapping Tiles",
                    VariableSizedWrapGrid::new()
                        .item_height(64.0)
                        .item_width(64.0)
                        .children(tiles),
                ),
            )],
        )
    }
}
