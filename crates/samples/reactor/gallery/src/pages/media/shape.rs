use crate::controls::*;
use windows_reactor::*;

pub struct ShapePage {
    filled: bool,
}

#[derive(Clone)]
pub enum Message {
    Filled(bool),
}

impl Component for ShapePage {
    type Message = Message;
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self { filled: true }
    }

    fn update(&mut self, message: Message, _: &ComponentContext<Self>) {
        match message {
            Message::Filled(value) => self.filled = value,
        }
    }

    fn view(&self, _: &(), context: &mut ViewContext<Self>) -> View {
        let fill = if self.filled {
            Brush::from(Color::rgb(0, 120, 215))
        } else {
            Brush::from(Color::transparent())
        };
        let stroke = Brush::from(Color::rgb(0, 120, 215));
        page_content(
            "Shape",
            "Draws lines, rectangles, and ellipses with fill and stroke.",
            [KeyedView::new(
                "basic",
                sample_card(
                    "Basic Shapes",
                    StackPanel::new().spacing(8.0).children((
                        StackPanel::new()
                            .orientation(Orientation::Horizontal)
                            .spacing(16.0)
                            .children((
                                Line::new()
                                    .x1(0.0)
                                    .y1(0.0)
                                    .x2(120.0)
                                    .y2(80.0)
                                    .stroke(stroke)
                                    .stroke_thickness(2.0),
                                Rectangle::new()
                                    .fill(fill)
                                    .stroke(stroke)
                                    .stroke_thickness(2.0)
                                    .width(120.0)
                                    .height(80.0),
                                Ellipse::new()
                                    .fill(fill)
                                    .stroke(stroke)
                                    .stroke_thickness(2.0)
                                    .width(120.0)
                                    .height(80.0),
                            )),
                        ToggleSwitch::new()
                            .is_on(self.filled)
                            .on_toggled(context.callback(Message::Filled))
                            .header("Filled"),
                    )),
                ),
            )],
        )
    }
}
