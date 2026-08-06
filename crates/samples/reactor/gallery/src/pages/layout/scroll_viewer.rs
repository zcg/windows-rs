use crate::controls::*;
use windows_reactor::*;

pub fn scroll_viewer_page(_: &(), _cx: &mut RenderCx) -> Element {
    let content: Element = vstack(
        (0..30).map(|i| {
            text_block(format!("Scrollable content item {i}"))
                .padding(Thickness::uniform(12.0))
                .into()
        }).collect::<Vec<Element>>(),
    )
    .spacing(4.0)
    .into();

    page_content(
        "ScrollViewer",
        "A ScrollViewer scrolls content with configurable scroll bar visibility and zoom.",
        vec![
            sample_card(
                "Content Inside ScrollViewer",
                ScrollViewer::new(content)
                    .vertical_scroll_bar_visibility(ScrollBarVisibility::Auto)
                    .height(280.0)
                    .width(360.0),
                r#"ScrollViewer::new(vstack(items)).vertical_scroll_bar_visibility(ScrollBarVisibility::Auto).height(280.0)"#,
            ),
            sample_card(
                "Hidden Scroll Bars",
                ScrollViewer::new(
                    vstack((0..10).map(|i| {
                        text_block(format!("Hidden bar item {i}")).padding(Thickness::uniform(10.0)).into()
                    }).collect::<Vec<Element>>()).spacing(4.0),
                )
                .vertical_scroll_bar_visibility(ScrollBarVisibility::Hidden)
                .height(200.0)
                .width(360.0),
                r#"ScrollViewer::new(...).vertical_scroll_bar_visibility(ScrollBarVisibility::Hidden)"#,
            ),
        ],
    )
}