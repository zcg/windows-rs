use crate::controls::*;
use windows_reactor::*;

pub struct RefreshContainerPage {
    refreshes: u32,
}

#[derive(Clone)]
pub enum Message {
    Refreshed,
}

impl Component for RefreshContainerPage {
    type Message = Message;
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self { refreshes: 0 }
    }

    fn update(&mut self, message: Message, _: &ComponentContext<Self>) {
        match message {
            Message::Refreshed => self.refreshes += 1,
        }
    }

    fn view(&self, _: &(), context: &mut ViewContext<Self>) -> View {
        let list = ListView::new().height(280.0).items((0..20).map(|index| {
            let label = format!("Item {index} - pull down to refresh");
            KeyedView::new(
                label.clone(),
                ListViewItem::new().tag(label.clone()).content(
                    Border::new()
                        .padding(8.0)
                        .content(TextBlock::new().text(label)),
                ),
            )
        }));
        page_content(
            "RefreshContainer",
            "Pull down to refresh scrollable content.",
            [KeyedView::new(
                "basic",
                sample_card(
                    "Basic Pull-to-Refresh",
                    StackPanel::new().spacing(8.0).children((
                        RefreshContainer::new()
                            .pull_direction(RefreshPullDirection::TopToBottom)
                            .on_refresh_requested(context.callback(|deferral: RefreshDeferral| {
                                deferral.complete();
                                Message::Refreshed
                            }))
                            .content(ScrollViewer::new().height(300.0).content(list)),
                        TextBlock::new()
                            .text(format!("Refreshed {} times", self.refreshes))
                            .opacity(0.6),
                    )),
                    "RefreshContainer::new().content(list).on_refresh_requested(handler)",
                ),
            )],
        )
    }
}
