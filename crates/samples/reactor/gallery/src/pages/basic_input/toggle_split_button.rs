use crate::controls::*;
use windows_reactor::*;

pub struct ToggleSplitButtonPage {
    checked: bool,
    clicks: u32,
}

#[derive(Clone)]
pub enum Message {
    Checked(bool),
    Clicked,
}

impl Component for ToggleSplitButtonPage {
    type Message = Message;
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self {
            checked: false,
            clicks: 0,
        }
    }

    fn update(&mut self, message: Message, _: &ComponentContext<Self>) {
        match message {
            Message::Checked(value) => self.checked = value,
            Message::Clicked => self.clicks += 1,
        }
    }

    fn view(&self, _: &(), context: &mut ViewContext<Self>) -> View {
        page_content(
            "ToggleSplitButton",
            "A split button with a checked state.",
            [KeyedView::new(
                "basic",
                sample_card(
                    "Basic ToggleSplitButton",
                    StackPanel::new().spacing(8.0).children((
                        ToggleSplitButton::new()
                            .is_checked(self.checked)
                            .on_is_checked_changed(context.callback(Message::Checked))
                            .on_click(context.message(Message::Clicked))
                            .content("Options"),
                        TextBlock::new()
                            .text(format!(
                                "Checked: {}, clicks: {}",
                                self.checked, self.clicks
                            ))
                            .opacity(0.6),
                    )),
                    "ToggleSplitButton::new().content(label).is_checked(value)",
                ),
            )],
        )
    }
}
