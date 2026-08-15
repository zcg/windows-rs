use crate::controls::*;
use windows_reactor::*;

pub fn popup_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "Popup",
        "A temporary overlay above the page content.",
        vec![
            sample_card(
                "Flyout-based Popup",
                vstack((
                    button("Show Popup")
                        .flyout("This overlay is shown via a Flyout - the reactor equivalent of a light-dismiss Popup."),
                    text_block("Flyout provides the same light-dismiss overlay behavior as Popup with simpler plumbing.")
                        .font_size(12.0)
                        .opacity(0.6),
                ))
                .spacing(8.0),
                r#"button("Show Popup").flyout("...") // light-dismiss overlay"#,
            ),
            sample_card(
                "Placement Control",
                hstack((
                    button("Flyout Top").flyout_with_placement("Placed above", FlyoutPlacementMode::Top),
                    button("Flyout Bottom").flyout_with_placement("Placed below", FlyoutPlacementMode::Bottom),
                ))
                .spacing(8.0),
                r#"flyout_with_placement("...", FlyoutPlacementMode::Top)"#,
            ),
        ],
    )
}
