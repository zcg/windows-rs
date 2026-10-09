use crate::controls::*;
use windows_reactor::*;

pub struct MapControlPage {
    status: String,
    token: String,
}

#[derive(Clone)]
pub enum Message {
    Token(String),
}

impl Component for MapControlPage {
    type Message = Message;
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self {
            status: String::from("Enter a map service token to load map tiles."),
            token: String::new(),
        }
    }

    fn update(&mut self, message: Message, _: &ComponentContext<Self>) {
        match message {
            Message::Token(token) => {
                self.token = token;
                self.status = String::from("Map service token updated.");
            }
        }
    }

    fn view(&self, _: &(), context: &mut ViewContext<Self>) -> View {
        page_content(
            "Map Control",
            "Display maps and location data with MapControl.",
            [KeyedView::new(
                "basic",
                sample_card(
                    "Map with Token",
                    StackPanel::new().spacing(8.0).children((
                        MapControl::new()
                            .zoom_level(4.0)
                            .map_service_token(self.token.clone())
                            .height(320.0),
                        TextBox::new(self.token.clone())
                            .placeholder_text("Map service token")
                            .on_text_changed(context.callback(|value: std::rc::Rc<str>| {
                                Message::Token(value.to_string())
                            })),
                        TextBlock::new().text(self.status.clone()).opacity(0.6),
                    )),
                    "MapControl::new().zoom_level(4.0).map_service_token(token)",
                ),
            )],
        )
    }
}
