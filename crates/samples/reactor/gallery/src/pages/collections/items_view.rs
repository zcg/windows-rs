use crate::controls::*;
use windows_reactor::*;

const FRUITS: [&str; 6] = ["Apple", "Banana", "Cherry", "Date", "Fig", "Grape"];

pub struct ItemsViewPage {
    invoked: String,
    selection: ItemsViewSelectionMode,
    layout: ItemsViewLayout,
}

#[derive(Clone)]
pub enum Message {
    Invoked(String),
    Single,
    Multiple,
    Stack,
    Grid,
}

impl Component for ItemsViewPage {
    type Message = Message;
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self {
            invoked: String::from("(none)"),
            selection: ItemsViewSelectionMode::Single,
            layout: ItemsViewLayout::Stack,
        }
    }

    fn update(&mut self, message: Message, _: &ComponentContext<Self>) {
        match message {
            Message::Invoked(item) => self.invoked = item,
            Message::Single => self.selection = ItemsViewSelectionMode::Single,
            Message::Multiple => self.selection = ItemsViewSelectionMode::Multiple,
            Message::Stack => self.layout = ItemsViewLayout::Stack,
            Message::Grid => self.layout = ItemsViewLayout::Grid,
        }
    }

    fn view(&self, _: &(), context: &mut ViewContext<Self>) -> View {
        page_content(
            "ItemsView",
            "A data-driven list with layout and selection.",
            [KeyedView::new(
                "basic",
                sample_card(
                    "Basic ItemsView",
                    StackPanel::new().spacing(8.0).children((
                        ItemsView::new()
                            .items_source(FRUITS)
                            .layout(self.layout)
                            .selection_mode(self.selection)
                            .on_item_invoked(context.callback(Message::Invoked))
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
                                Button::new()
                                    .on_click(context.message(Message::Stack))
                                    .content("Stack layout"),
                                Button::new()
                                    .on_click(context.message(Message::Grid))
                                    .content("Grid layout"),
                                TextBlock::new()
                                    .text(format!("Invoked: {}", self.invoked))
                                    .opacity(0.6),
                            )),
                    )),
                    "ItemsView::new().items_source(items).on_item_invoked(handler)",
                ),
            )],
        )
    }
}
