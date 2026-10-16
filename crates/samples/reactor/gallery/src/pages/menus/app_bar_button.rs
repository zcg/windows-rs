use crate::controls::*;
use windows_reactor::*;

pub struct AppBarButtonPage {
    clicks: u32,
}

#[derive(Clone)]
pub enum Message {
    Clicked,
}

impl Component for AppBarButtonPage {
    type Message = Message;
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self { clicks: 0 }
    }

    fn update(&mut self, message: Message, _: &ComponentContext<Self>) {
        match message {
            Message::Clicked => self.clicks += 1,
        }
    }

    fn view(&self, _: &(), context: &mut ViewContext<Self>) -> View {
        page_content(
            "AppBarButton",
            "A button for command bars with a label and icon.",
            [KeyedView::new(
                "basic",
                sample_card(
                    "Basic AppBarButton",
                    StackPanel::new().spacing(8.0).children((
                        CommandBar::new()
                            .keyed_primary_commands([KeyedView::new(
                                "save",
                                AppBarButton::new()
                                    .label("Save")
                                    .on_click(context.message(Message::Clicked))
                                    .icon(Symbol::Save),
                            )])
                            .keyed_secondary_commands([KeyedView::new(
                                "delete",
                                AppBarButton::new()
                                    .label("Delete")
                                    .on_click(context.message(Message::Clicked)),
                            )]),
                        TextBlock::new()
                            .text(format!("Clicked {} times", self.clicks))
                            .opacity(0.6),
                    )),
                ),
            )],
        )
    }
}
