use crate::controls::*;
use windows_reactor::*;

const FRUITS: [&str; 6] = ["Apple", "Banana", "Cherry", "Date", "Fig", "Grape"];

pub struct ItemsViewPage {
    invoked: String,
    selection: ItemsViewSelectionMode,
}

#[derive(Clone)]
pub enum Message {
    Invoked(String),
    Single,
    Multiple,
}

impl Component for ItemsViewPage {
    type Message = Message;
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self {
            invoked: String::from("(none)"),
            selection: ItemsViewSelectionMode::Single,
        }
    }

    fn update(&mut self, message: Message, _: &ComponentContext<Self>) {
        match message {
            Message::Invoked(item) => self.invoked = item,
            Message::Single => self.selection = ItemsViewSelectionMode::Single,
            Message::Multiple => self.selection = ItemsViewSelectionMode::Multiple,
        }
    }

    fn view(&self, _: &(), context: &mut ViewContext<Self>) -> View {
        page_content(
            "ItemsView",
            "A data-driven list with selection.",
            [KeyedView::new(
                "basic",
                sample_card(
                    "Basic ItemsView",
                    StackPanel::new().spacing(8.0).children((
                        ItemsView::new()
                            .items_source(FRUITS)
                            .selection_mode(self.selection)
                            .on_item_invoked(context.callback(|value: std::rc::Rc<str>| {
                                Message::Invoked(value.to_string())
                            }))
                            .height(220.0),
                        StackPanel::new()
                            .orientation(Orientation::Horizontal)
                            .spacing(8.0)
                            .children((
                                Button::new()
                                    .on_click(context.message(Message::Single))
                                    .content("Single"),
                                Button::new()
                                    .on_click(context.message(Message::Multiple))
                                    .content("Multiple"),
                                TextBlock::new()
                                    .text(format!("Invoked: {}", self.invoked))
                                    .opacity(0.6),
                            )),
                    )),
                ),
            )],
        )
    }
}
