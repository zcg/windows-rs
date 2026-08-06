use crate::controls::*;
use windows_reactor::*;

pub fn semantic_zoom_page(_: &(), cx: &mut RenderCx) -> Element {
    let (zoomed_in, set_zoomed_in) = cx.use_state(true);

    let items: Vec<String> = (0..12).map(|i| format!("Section {i}")).collect();

    let view: Element = if zoomed_in {
        let el: Element = list_view(items.clone(), |item, _| {
            let t: Element = text_block(item.clone()).padding(Thickness::uniform(8.0)).into();
            t
        })
        .height(260.0)
        .width(320.0)
        .into();
        el
    } else {
        let el: Element = items_repeater(items.clone(), |item, _| {
            let b: Element = border(text_block(item.clone()).font_size(10.0))
                .padding(Thickness::uniform(6.0))
                .background(ThemeRef::CardBackground)
                .into();
            b
        })
        .height(260.0)
        .width(320.0)
        .into();
        el
    };

    page_content(
        "SemanticZoom",
        "Zoom between a detailed view and an overview of the same data.",
        vec![
            sample_card(
                "Zoomed In / Zoomed Out",
                vstack((
                    view,
                    button(if zoomed_in { "Zoom Out" } else { "Zoom In" })
                        .on_click(move || set_zoomed_in.call(!zoomed_in)),
                ))
                .spacing(8.0),
                r#"zoomed-in: list_view(...) / zoomed-out: compact items_repeater(...)"#,
            ),
        ],
    )
}