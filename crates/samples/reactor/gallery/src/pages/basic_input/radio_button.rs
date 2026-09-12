use crate::controls::*;
use windows_reactor::*;

pub struct RadioButtonPage {
    first: bool,
    second: bool,
}

#[derive(Clone)]
pub enum Message {
    First(bool),
    Second(bool),
}

impl Component for RadioButtonPage {
    type Message = Message;
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self {
            first: true,
            second: false,
        }
    }

    fn update(&mut self, message: Message, _: &ComponentContext<Self>) {
        match message {
            Message::First(checked) => self.first = checked,
            Message::Second(checked) => self.second = checked,
        }
    }

    fn view(&self, _: &(), context: &mut ViewContext<Self>) -> View {
        page_content(
            "RadioButton",
            "Select one option from a group.",
            [
                KeyedView::new(
                    "basic",
                    sample_card(
                        "Grouped buttons",
                        StackPanel::new().spacing(8.0).children((
                            RadioButton::new()
                                .group_name("drive")
                                .is_checked(self.first)
                                .on_checked(context.callback(Message::First))
                                .content("Automatic"),
                            RadioButton::new()
                                .group_name("drive")
                                .is_checked(self.second)
                                .on_checked(context.callback(Message::Second))
                                .content("Manual"),
                            TextBlock::new()
                                .text(format!(
                                    "Automatic: {}, Manual: {}",
                                    self.first, self.second
                                ))
                                .opacity(0.6),
                        )),
                        "RadioButton::new().group_name(group).is_checked(flag).on_checked(handler)",
                    ),
                ),
                KeyedView::new(
                    "disabled",
                    sample_card(
                        "Disabled button",
                        RadioButton::new()
                            .group_name("unavailable")
                            .is_enabled(false)
                            .content("Not available"),
                        "RadioButton::new().is_enabled(false)",
                    ),
                ),
            ],
        )
    }
}
