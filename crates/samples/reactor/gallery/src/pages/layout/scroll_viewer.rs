use crate::controls::*;
use windows_reactor::*;

pub struct ScrollViewerPage;

impl Component for ScrollViewerPage {
    type Message = ();
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self
    }

    fn view(&self, _: &(), _: &mut ViewContext<Self>) -> View {
        let long_text = (0..40)
            .map(|index| format!("Line {index} - scroll to view more"))
            .collect::<Vec<_>>()
            .join("\n");
        page_content(
            "ScrollViewer",
            "Scrolls content with configurable scroll bar visibility.",
            [
                KeyedView::new(
                    "basic",
                    sample_card(
                        "Basic ScrollViewer",
                        ScrollViewer::new().height(220.0).content(
                            TextBlock::new()
                                .text(long_text.clone())
                                .text_wrapping(TextWrapping::Wrap),
                        ),
                        "ScrollViewer::new().content(view)",
                    ),
                ),
                KeyedView::new(
                    "scroll-bars",
                    sample_card(
                        "Scroll Bar Visibility",
                        ScrollViewer::new()
                            .height(220.0)
                            .horizontal_scroll_bar_visibility(ScrollBarVisibility::Auto)
                            .vertical_scroll_bar_visibility(ScrollBarVisibility::Auto)
                            .content(
                                TextBlock::new()
                                    .text(long_text)
                                    .text_wrapping(TextWrapping::Wrap),
                            ),
                        r#"ScrollViewer::new()
    .horizontal_scroll_bar_visibility(ScrollBarVisibility::Auto)
    .vertical_scroll_bar_visibility(ScrollBarVisibility::Auto)"#,
                    ),
                ),
            ],
        )
    }
}
