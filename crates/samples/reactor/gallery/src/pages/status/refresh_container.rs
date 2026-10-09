use crate::controls::*;
use windows_reactor::*;

pub struct RefreshContainerPage;

impl Component for RefreshContainerPage {
    type Message = ();
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self
    }

    fn view(&self, _: &(), _: &mut ViewContext<Self>) -> View {
        let list = ListView::new().height(280.0).items((0..20).map(|index| {
            let label = format!("Item {index} - pull down to refresh");
            DataItem::new(label.clone(), label.clone()).content(
                Border::new()
                    .padding(8.0)
                    .content(TextBlock::new().text(label)),
            )
        }));
        page_content(
            "RefreshContainer",
            "Pull down to refresh scrollable content.",
            [KeyedView::new(
                "basic",
                sample_card(
                    "Basic Pull-to-Refresh",
                    RefreshContainer::new()
                        .pull_direction(RefreshPullDirection::TopToBottom)
                        .content(ScrollViewer::new().height(300.0).content(list)),
                    "RefreshContainer::new().content(list).pull_direction(RefreshPullDirection::TopToBottom)",
                ),
            )],
        )
    }
}
