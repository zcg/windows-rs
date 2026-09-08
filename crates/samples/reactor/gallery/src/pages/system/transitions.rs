use crate::controls::*;
use windows_reactor::*;

pub struct TransitionsPage {
    big: bool,
    visible: bool,
}

#[derive(Clone)]
pub enum Message {
    Big(bool),
    Visible(bool),
}

impl Component for TransitionsPage {
    type Message = Message;
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self {
            big: false,
            visible: true,
        }
    }

    fn update(&mut self, message: Message, _: &ComponentContext<Self>) {
        match message {
            Message::Big(value) => self.big = value,
            Message::Visible(value) => self.visible = value,
        }
    }

    fn view(&self, _: &(), context: &mut ViewContext<Self>) -> View {
        use std::time::Duration;
        page_content(
            "Transitions",
            "Implicit opacity and scale transitions animate property changes.",
            [KeyedView::new(
                "implicit",
                sample_card(
                    "Implicit Transitions",
                    StackPanel::new().spacing(8.0).children((
                        Border::new()
                            .background(ThemeBrush::Accent)
                            .opacity(if self.visible { 1.0 } else { 0.2 })
                            .opacity_transition(Duration::from_millis(400))
                            .scale(if self.big { 1.2 } else { 1.0 })
                            .scale_transition(Duration::from_millis(400))
                            .width(200.0)
                            .height(80.0)
                            .content(TextBlock::new().text("Animated box").opacity(0.8)),
                        ToggleSwitch::new()
                            .is_on(self.big)
                            .on_toggled(context.callback(Message::Big))
                            .slot(ToggleSwitchSlot::Header, "Big"),
                        ToggleSwitch::new()
                            .is_on(self.visible)
                            .on_toggled(context.callback(Message::Visible))
                            .slot(ToggleSwitchSlot::Header, "Visible"),
                    )),
                    "Border::new().opacity(v).opacity_transition(ms).scale(s).scale_transition(ms)",
                ),
            )],
        )
    }
}
