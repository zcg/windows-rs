use crate::controls::*;
use windows_reactor::*;

pub fn items_view_page(_: &(), _cx: &mut RenderCx) -> Element {
    let items: Vec<String> = (0..15).map(|i| format!("Item {i}")).collect();

    page_content(
        "ItemsView",
        "A data-driven list with flexible layout and selection.",
        vec![
            sample_card(
                "Data-driven Items (ItemsRepeater-based)",
                items_repeater(items, |item, _idx| {
                    let el: Element = border(text_block(item.clone()))
                        .padding(Thickness::uniform(10.0))
                        .corner_radius(4.0)
                        .background(ThemeRef::CardBackground)
                        .margin(Thickness::uniform(2.0))
                        .into();
                    el
                })
                .height(280.0)
                .width(320.0),
                r#"items_repeater(items, |item, _| ...) // ItemsView builds on ItemsRepeater"#,
            ),
            sample_card(
                "ItemsView Concept",
                text_block("ItemsView combines a data source with layout and selection; the reactor templated list provides the equivalent data-driven model.")
                    .font_size(13.0),
                r#"ItemsView ItemsSource + Layout"#,
            ),
        ],
    )
}