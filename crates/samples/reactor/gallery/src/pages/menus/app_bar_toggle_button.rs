use crate::controls::*;
use windows_reactor::*;

pub struct AppBarToggleButtonPage {
    checked: bool,
}

#[derive(Clone)]
pub enum Message {
    Clicked,
}

impl Component for AppBarToggleButtonPage {
    type Message = Message;
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self { checked: false }
    }

    fn update(&mut self, message: Message, _: &ComponentContext<Self>) {
        match message {
            Message::Clicked => self.checked = !self.checked,
        }
    }

    fn view(&self, _: &(), context: &mut ViewContext<Self>) -> View {
        page_content(
            "AppBarToggleButton",
            "A command-bar toggle button with a checked state.",
            [KeyedView::new(
                "basic",
                sample_card(
                    "Basic AppBarToggleButton",
                    StackPanel::new().spacing(8.0).children((
                        CommandBar::new().keyed_primary_commands([KeyedView::new(
                            "shuffle",
                            AppBarToggleButton::new()
                                .label("Shuffle")
                                .is_checked(self.checked)
                                .on_click(context.message(Message::Clicked))
                                .icon(Symbol::Shuffle),
                        )]),
                        TextBlock::new()
                            .text(if self.checked {
                                "Shuffle is on."
                            } else {
                                "Shuffle is off."
                            })
                            .opacity(0.6),
                    )),
                    "AppBarToggleButton::new().label(\"Shuffle\").is_checked(value)",
                ),
            )],
        )
    }
}
