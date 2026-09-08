use crate::controls::*;
use windows_reactor::*;

pub struct MapControlPage {
    status: String,
    token: String,
}

#[derive(Clone)]
pub enum Message {
    Token(String),
    PortoAlegre,
    Seattle,
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
            Message::Token(token) => self.token = token,
            Message::PortoAlegre => {
                self.status = String::from("Center: Porto Alegre");
            }
            Message::Seattle => {
                self.status = String::from("Center: Seattle");
            }
        }
    }

    fn view(&self, _: &(), context: &mut ViewContext<Self>) -> View {
        let map = |latitude: f64, longitude: f64| {
            MapControl::new()
                .center(latitude, longitude)
                .zoom_level(4.0)
                .map_service_token(self.token.clone())
                .height(320.0)
        };
        let center = if self.status == "Center: Seattle" {
            (47.6062, -122.3321)
        } else {
            (-30.034647, -51.217659)
        };
        page_content(
            "Map Control",
            "Display maps and location data with MapControl.",
            [KeyedView::new(
                "basic",
                sample_card(
                    "Map with Marker",
                    StackPanel::new().spacing(8.0).children((
                        map(center.0, center.1),
                        TextBox::new()
                            .placeholder_text("Map service token")
                            .text(self.token.clone())
                            .on_text_changed(context.callback(Message::Token)),
                        StackPanel::new()
                            .orientation(Orientation::Horizontal)
                            .spacing(8.0)
                            .children((
                                Button::new()
                                    .on_click(context.message(Message::PortoAlegre))
                                    .content("Porto Alegre"),
                                Button::new()
                                    .on_click(context.message(Message::Seattle))
                                    .content("Seattle"),
                                TextBlock::new().text(self.status.clone()).opacity(0.6),
                            )),
                    )),
                    "MapControl::new().center(lat, lon).zoom_level(4.0)",
                ),
            )],
        )
    }
}
