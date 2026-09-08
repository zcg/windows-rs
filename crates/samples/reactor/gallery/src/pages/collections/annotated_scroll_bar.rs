use crate::controls::*;
use windows_reactor::*;

pub struct AnnotatedScrollBarPage {
    small_change: f64,
}

#[derive(Clone)]
pub enum Message {
    Small(f64),
}

impl Component for AnnotatedScrollBarPage {
    type Message = Message;
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self { small_change: 1.0 }
    }

    fn update(&mut self, message: Message, _: &ComponentContext<Self>) {
        match message {
            Message::Small(value) => self.small_change = value,
        }
    }

    fn view(&self, _: &(), context: &mut ViewContext<Self>) -> View {
        page_content(
            "AnnotatedScrollBar",
            "A scroll bar that shows value labels alongside the thumb.",
            [KeyedView::new(
                "basic",
                sample_card(
                    "Small Change",
                    StackPanel::new().spacing(8.0).children((
                        AnnotatedScrollBar::new()
                            .small_change(self.small_change)
                            .height(240.0),
                        StackPanel::new()
                            .orientation(Orientation::Horizontal)
                            .spacing(8.0)
                            .children((
                                Button::new()
                                    .on_click(context.message(Message::Small(1.0)))
                                    .content("1.0"),
                                Button::new()
                                    .on_click(context.message(Message::Small(5.0)))
                                    .content("5.0"),
                                TextBlock::new()
                                    .text(format!("Small change: {}", self.small_change))
                                    .opacity(0.6),
                            )),
                    )),
                    "AnnotatedScrollBar::new().small_change(value)",
                ),
            )],
        )
    }
}
