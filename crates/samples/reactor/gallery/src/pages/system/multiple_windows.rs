use crate::controls::*;
use windows_reactor::*;

pub struct MultipleWindowsPage {
    opened: u32,
}

#[derive(Clone)]
pub enum Message {
    Open,
}

impl Component for MultipleWindowsPage {
    type Message = Message;
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self { opened: 0 }
    }

    fn update(&mut self, message: Message, context: &ComponentContext<Self>) {
        match message {
            Message::Open => {
                let number = self.opened + 1;
                if context.open_window::<CounterWindow>(number) {
                    self.opened = number;
                }
            }
        }
    }

    fn view(&self, _: &(), context: &mut ViewContext<Self>) -> View {
        page_content(
            "Multiple Windows",
            "Open independent windows, each hosting its own component.",
            [KeyedView::new(
                "basic",
                sample_card(
                    "Open a Window",
                    StackPanel::new().spacing(8.0).children((
                        TextBlock::new()
                            .text("Each opened window hosts its own independent counter.")
                            .opacity(0.6),
                        Button::new()
                            .on_click(context.message(Message::Open))
                            .content("Open counter window"),
                        TextBlock::new()
                            .text(format!("Windows opened: {}", self.opened))
                            .opacity(0.6),
                    )),
                ),
            )],
        )
    }
}

struct CounterWindow {
    count: i32,
}

#[derive(Clone, Copy)]
enum CounterMessage {
    Decrement,
    Increment,
}

impl Component for CounterWindow {
    type Message = CounterMessage;
    type Input = u32;

    fn create(_: &u32, _: &ComponentContext<Self>) -> Self {
        Self { count: 0 }
    }

    fn update(&mut self, message: CounterMessage, _: &ComponentContext<Self>) {
        match message {
            CounterMessage::Decrement => self.count -= 1,
            CounterMessage::Increment => self.count += 1,
        }
    }

    fn view(&self, number: &u32, context: &mut ViewContext<Self>) -> View {
        context.window_title(format!("Counter window #{number}"));
        context.window_visuals(WindowVisuals::new().client_size(320.0, 220.0));
        Border::new()
            .padding(24.0)
            .content(
                StackPanel::new().spacing(12.0).children((
                    TextBlock::new().text("Independent counter").font_size(20.0),
                    TextBlock::new()
                        .text(format!("Count: {}", self.count))
                        .font_size(28.0),
                    StackPanel::new()
                        .orientation(Orientation::Horizontal)
                        .spacing(8.0)
                        .children((
                            Button::new()
                                .on_click(context.message(CounterMessage::Decrement))
                                .content("-"),
                            Button::new()
                                .on_click(context.message(CounterMessage::Increment))
                                .content("+"),
                        )),
                )),
            )
            .into()
    }
}
