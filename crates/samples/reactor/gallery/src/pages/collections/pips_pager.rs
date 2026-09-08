use crate::controls::*;
use windows_reactor::*;

pub struct PipsPagerPage {
    page: i32,
}

#[derive(Clone)]
pub enum Message {
    Selected(i32),
    Previous,
    Next,
}

const PAGE_COUNT: i32 = 5;

impl Component for PipsPagerPage {
    type Message = Message;
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self { page: 0 }
    }

    fn update(&mut self, message: Message, _: &ComponentContext<Self>) {
        match message {
            Message::Selected(page) => self.page = page.clamp(0, PAGE_COUNT - 1),
            Message::Previous => self.page = (self.page - 1).max(0),
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
                            .number_of_pages(PAGE_COUNT)
                            .selected_page_index(self.page)
                            .max_visible_pips(5)
                            .on_selected_index_changed(context.callback(Message::Selected)),
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
                    "PipsPager::new().number_of_pages(n).selected_page_index(i)",
                ),
            )],
        )
    }
}
