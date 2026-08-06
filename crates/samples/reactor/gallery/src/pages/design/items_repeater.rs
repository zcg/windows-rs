use crate::controls::*;
use windows_reactor::*;

pub fn items_repeater_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "ItemsRepeater",
        "A data-driven control that repeats a template per item.",
        vec![sample_card("Overview", text_block("ItemsRepeater is a layout-driven item repeater; reactor provides templated ListView/GridView equivalents.").font_size(13.0), r#"ItemsRepeater with ItemsSource + ItemTemplate"#)],
    )
}