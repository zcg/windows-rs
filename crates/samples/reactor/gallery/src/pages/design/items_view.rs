use crate::controls::*;
use windows_reactor::*;

pub fn items_view_page(_: &(), _cx: &mut RenderCx) -> Element {
    let items: Vec<String> = (0..24).map(|i| format!("Item {i}")).collect();

    let list: Element = items_repeater(items, |item, _idx| {
        let el: Element = border(text_block(item.clone()))
            .padding(Thickness::uniform(10.0))
            .corner_radius(4.0)
            .background(ThemeRef::CardBackground)
            .margin(Thickness::uniform(2.0))
            .into();
        el
    })
    .height(300.0)
    .width(460.0)
    .into();

    page_content(
        "ItemsView",
        "A data-driven list with flexible layout and selection.",
        vec![
            sample_card(
                "Data-driven Items (ItemsRepeater-based)",
                list,
                r#"items_repeater(items, |item, _| {
    border(text_block(item)).background(ThemeRef::CardBackground)
})"#,
            ),
            sample_card(
                "Runtime Note",
                text_block(
                    "The native ItemsView control crashes in the Windows App SDK 2.3.1 runtime as soon as an ItemTemplate is set (reproduced with an official DataTemplate, not just the reactor factory). Until the runtime is fixed or upgraded, this page demonstrates the same data-driven model through ItemsRepeater, which ItemsView builds upon.",
                )
                .font_size(13.0),
                r#"ItemsView { ItemsSource, Layout, ItemTemplate } // runtime crash in 2.3.1"#,
            ),
        ],
    )
}
