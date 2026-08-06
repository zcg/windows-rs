use crate::controls::*;
use windows_reactor::*;

pub fn items_view_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "ItemsView",
        "A data-driven list with flexible layout and selection.",
        vec![sample_card("Overview", text_block("ItemsView combines a data source with layout and selection support.").font_size(13.0), r#"ItemsView ItemsSource + Layout"#)],
    )
}