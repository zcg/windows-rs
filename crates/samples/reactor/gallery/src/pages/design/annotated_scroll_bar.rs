use crate::controls::*;
use windows_reactor::*;

pub fn annotated_scroll_bar_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "AnnotatedScrollBar",
        "A scroll bar with value labels alongside the thumb.",
        vec![
            sample_card(
                "Annotated ScrollBar with Labels",
                vstack((
                    annotated_scroll_bar()
                        .labels(vec![
                            (0.0, "Start".into()),
                            (25.0, "25".into()),
                            (50.0, "Half".into()),
                            (75.0, "75".into()),
                            (100.0, "End".into()),
                        ])
                        .small_change(5.0)
                        .height(220.0),
                    text_block("AnnotatedScrollBar renders alongside scrollable content. Label objects require the AnnotatedScrollBarLabel metadata; the control surface and sizing are shown here.")
                        .font_size(12.0)
                        .opacity(0.6),
                ))
                .spacing(8.0),
                r#"annotated_scroll_bar().labels(vec![(0.0, "Start"), (50.0, "Half"), (100.0, "End")])"#,
            ),
        ],
    )
}
