use crate::controls::*;
use windows_reactor::*;

pub struct PipsPagerPage {
    page: usize,
}

#[derive(Clone)]
pub enum Message {
    Selected,
    Previous,
    Next,
}

const PAGE_COUNT: usize = 5;

impl Component for PipsPagerPage {
    type Message = Message;
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self { page: 0 }
    }

    fn update(&mut self, message: Message, _: &ComponentContext<Self>) {
        match message {
            Message::Selected => {}
            Message::Previous => self.page = self.page.saturating_sub(1),
            Message::Next => self.page = (self.page + 1).min(PAGE_COUNT - 1),
        }
    }

    fn view(&self, _: &(), context: &mut ViewContext<Self>) -> View {
        page_content(
            "PipsPager",
            "A horizontal row of pips representing pages of content.",
            [KeyedView::new(
                "basic",
                sample_card(
                    "Basic PipsPager",
                    StackPanel::new().spacing(8.0).children((
                        PipsPager::new()
                            .number_of_pages(PAGE_COUNT as i32)
                            .selected_page_index(self.page as i32)
                            .max_visible_pips(5)
                            .on_selected_index_changed(context.callback(|_| Message::Selected)),
                        StackPanel::new()
                            .orientation(Orientation::Horizontal)
                            .spacing(8.0)
                            .children((
                                Button::new()
                                    .on_click(context.message(Message::Previous))
                                    .content("Previous"),
                                Button::new()
                                    .on_click(context.message(Message::Next))
                                    .content("Next"),
                                TextBlock::new()
                                    .text(format!("Page {} of {PAGE_COUNT}", self.page + 1))
                                    .opacity(0.6),
                            )),
                    )),
                ),
            )],
        )
    }
}
