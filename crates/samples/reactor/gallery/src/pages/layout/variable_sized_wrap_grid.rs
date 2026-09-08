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
        let tile = |text: &str, row_span: i32, column_span: i32| {
            Border::new()
                .background(ThemeBrush::CardBackground)
                .padding(12.0)
                .wrap_grid_row_span(row_span)
                .wrap_grid_column_span(column_span)
                .content(TextBlock::new().text(text))
        };
        page_content(
            "VariableSizedWrapGrid",
            "Arranges items in a grid where items can span rows and columns.",
            [KeyedView::new(
                "basic",
                sample_card(
                    "Spanning Tiles",
                    VariableSizedWrapGrid::new()
                        .item_height(64.0)
                        .item_width(64.0)
                        .maximum_rows_or_columns(4)
                        .children([
                            tile("1x1", 1, 1),
                            tile("2x1", 2, 1),
                            tile("1x2", 1, 2),
                            tile("1x1", 1, 1),
                            tile("2x2", 2, 2),
                        ]),
                    "VariableSizedWrapGrid::new().children([...wrap_grid_row_span(2)...])",
                ),
            )],
        )
    }
}
