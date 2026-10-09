use crate::controls::*;
use windows_reactor::*;

pub struct ItemContainerPage;

impl Component for ItemContainerPage {
    type Message = ();
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self
    }

    fn view(&self, _: &(), _: &mut ViewContext<Self>) -> View {
        page_content(
            "ItemContainer",
            "Wraps an item with the selection and layout behavior of a repeater.",
            [
                KeyedView::new(
                    "basic",
                    sample_card(
                        "Item container",
                        ItemContainer::new().width(220.0).child(
                            Border::new().padding(12.0).content(
                                TextBlock::new()
                                    .text("Item content")
                                    .font_weight(FontWeight::SEMI_BOLD),
                            ),
                        ),
                        "ItemContainer::new().child(view)",
                    ),
                ),
                KeyedView::new(
                    "usage",
                    sample_card(
                        "Where it is used",
                        InfoBar::new()
                            .severity(InfoBarSeverity::Informational)
                            .title("Repeater item host")
                            .message(
                                "An ItemsRepeater realizes its items inside an ItemContainer. \
                                 The container supplies the selection states, so a virtualized \
                                 list keeps one item container per realized item instead of one \
                                 per bound object.",
                            )
                            .is_open(true),
                        "ItemContainer::new()",
                    ),
                ),
            ],
        )
    }
}
