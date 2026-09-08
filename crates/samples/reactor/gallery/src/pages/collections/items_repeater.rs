use crate::controls::*;
use windows_reactor::*;

pub struct ItemsRepeaterPage {
    count: usize,
}

#[derive(Clone)]
pub enum Message {
    More,
    Fewer,
}

impl Component for ItemsRepeaterPage {
    type Message = Message;
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self { count: 20 }
    }

    fn update(&mut self, message: Message, _: &ComponentContext<Self>) {
        match message {
            Message::More => self.count = (self.count + 10).min(1000),
            Message::Fewer => self.count = self.count.saturating_sub(10).max(1),
        }
    }

    fn view(&self, _: &(), context: &mut ViewContext<Self>) -> View {
        page_content(
            "ItemsRepeater",
            "Virtualizes large lists, realizing only visible items.",
            [KeyedView::new(
                "basic",
                sample_card(
                    "Virtualized List",
                    StackPanel::new().spacing(8.0).children((
                        ItemsRepeater::new()
                            .virtual_source(VirtualSource::new(
                                0,
                                self.count,
                                |index| index as u64,
                                |index| {
                                    Border::new()
                                        .padding(8.0)
                                        .content(TextBlock::new().text(format!("Row {index}")))
                                },
                            ))
                            .height(240.0),
                        StackPanel::new()
                            .orientation(Orientation::Horizontal)
                            .spacing(8.0)
                            .children((
                                Button::new()
                                    .on_click(context.message(Message::Fewer))
                                    .content("Fewer"),
                                Button::new()
                                    .on_click(context.message(Message::More))
                                    .content("More"),
                                TextBlock::new()
                                    .text(format!("{} rows", self.count))
                                    .opacity(0.6),
                            )),
                    )),
                    "ItemsRepeater::new().virtual_source(revision, len, key, view)",
                ),
            )],
        )
    }
}
