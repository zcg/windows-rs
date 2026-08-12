use crate::controls::*;
use windows_reactor::*;

pub fn semantic_zoom_page(_: &(), cx: &mut RenderCx) -> Element {
    let (zoomed_in, set_zoomed_in) = cx.use_state(true);

    let items: Vec<String> = (0..12).map(|i| format!("Section {i}")).collect();

    let zoomed_in_view: Element = list_view(items.clone(), |item, _| {
        let t: Element = text_block(item.clone())
            .font_size(15.0)
            .padding(Thickness::uniform(10.0))
            .into();
        t
    })
    .height(260.0)
    .width(320.0)
    .into();

    let zoomed_out_view: Element = grid_view(items, |item, _| {
        let b: Element = border(text_block(item.clone()).font_size(10.0))
            .padding(Thickness::uniform(8.0))
            .background(ThemeRef::CardBackground)
            .corner_radius(4.0)
            .margin(Thickness::uniform(2.0))
            .into();
        b
    })
    .height(260.0)
    .width(320.0)
    .into();

    let zoom = semantic_zoom(zoomed_in_view)
        .zoomed_out(zoomed_out_view)
        .is_zoomed_in_view_active(zoomed_in);

    page_content(
        "SemanticZoom",
        "Zoom between a detailed view and an overview of the same data.",
        vec![sample_card(
            "Zoomed In / Zoomed Out",
            vstack((
                zoom,
                hstack((
                    button(if zoomed_in { "Zoom Out" } else { "Zoom In" })
                        .on_click(move || set_zoomed_in.call(!zoomed_in)),
                    text_block(if zoomed_in {
                        "Showing the detailed list."
                    } else {
                        "Showing the compact overview."
                    })
                    .opacity(0.6),
                ))
                .spacing(8.0),
            ))
            .spacing(8.0),
            r#"semantic_zoom(zoomed_in_list_view)
    .zoomed_out(zoomed_out_grid_view)
    .is_zoomed_in_view_active(zoomed_in)"#,
        )],
    )
}
