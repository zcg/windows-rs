use crate::controls::*;
use windows_reactor::*;

pub fn parallax_view_page(_: &(), _cx: &mut RenderCx) -> Element {
    let foreground: Element = list_view(
        (0..30).map(|i| format!("Scrollable row {i}")).collect(),
        |item, _| {
            let t: Element = text_block(item.clone()).padding(Thickness::uniform(8.0)).into();
            t
        },
    )
    .height(260.0)
    .into();

    page_content(
        "ParallaxView",
        "Parallax scrolling links a background element to a scrolling source.",
        vec![
            sample_card(
                "Scrollable Content",
                vstack((
                    foreground,
                    text_block("ParallaxView moves a background element at a different rate than the scroll source. Reactor exposes scrolling through ScrollViewer and templated lists; attaching a ParallaxView source requires the ScrollController metadata.")
                        .font_size(12.0)
                        .opacity(0.6),
                ))
                .spacing(8.0),
                r#"ParallaxView Source + HorizontalShift // scroll-driven background"#,
            ),
        ],
    )
}