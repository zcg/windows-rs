use crate::controls::*;
use windows_reactor::*;

pub fn variable_sized_wrap_grid_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "VariableSizedWrapGrid",
        "A grid layout where items can span rows and columns.",
        vec![sample_card("Overview", text_block("VariableSizedWrapGrid arranges items in a grid with per-item RowSpan/ColumnSpan.").font_size(13.0), r#"VariableSizedWrapGrid.RowSpan / ColumnSpan"#)],
    )
}