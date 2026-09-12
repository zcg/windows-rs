use crate::controls::*;
use windows_reactor::*;

pub struct TextBlockPage {
    large: bool,
}

#[derive(Clone)]
pub enum Message {
    Large(bool),
}

impl Component for TextBlockPage {
    type Message = Message;
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self { large: false }
    }

    fn update(&mut self, message: Message, _: &ComponentContext<Self>) {
        match message {
            Message::Large(value) => self.large = value,
        }
    }

    fn view(&self, _: &(), context: &mut ViewContext<Self>) -> View {
        page_content(
            "TextBlock",
            "Displays read-only text with font styling.",
            [KeyedView::new(
                "styling",
                sample_card(
                    "Font Styling",
                    StackPanel::new().spacing(8.0).children((
                        TextBlock::new()
                            .text("Segoe UI, semibold")
                            .font_family("Segoe UI")
                            .font_weight(FontWeight::SEMI_BOLD)
                            .font_size(if self.large { 32.0 } else { 20.0 }),
                        TextBlock::new()
                            .text("Consolas monospace")
                            .font_family("Consolas")
                            .font_size(16.0),
                        ToggleSwitch::new()
                            .is_on(self.large)
                            .on_toggled(context.callback(Message::Large))
                            .header("Large"),
                    )),
                    "TextBlock::new().text(t).font_family(\"Segoe UI\").font_weight(FontWeight::SEMI_BOLD)",
                ),
            )],
        )
    }
}
