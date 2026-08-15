use crate::controls::*;
use windows_reactor::*;

pub fn items_repeater_page(_: &(), _cx: &mut RenderCx) -> Element {
    let items: Vec<String> = (0..20).map(|i| format!("Item {i}")).collect();

    page_content(
        "ItemsRepeater",
        "A data-driven control that repeats a template per item.",
        vec![sample_card(
            "ItemsRepeater (templated)",
            items_repeater(items, |item, _idx| {
                let el: Element = border(text_block(item.clone()))
                    .padding(Thickness::uniform(10.0))
                    .corner_radius(4.0)
                    .background(ThemeRef::CardBackground)
                    .margin(Thickness::uniform(2.0))
                    .into();
                el
            })
            .height(320.0)
            .width(320.0),
            r#"items_repeater(items, |item, _| border(text_block(item))...)"#,
        )],
    )
}
