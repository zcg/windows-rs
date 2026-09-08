use crate::controls::*;
use windows::ApplicationModel::DataTransfer::{Clipboard, DataPackage};
use windows_reactor::*;

pub struct ClipboardPage {
    text: String,
    status: String,
}

#[derive(Clone)]
pub enum Message {
    Text(String),
    Copy,
    Paste,
}

fn copy_text(text: &str) -> bool {
    let Ok(package) = DataPackage::new() else {
        return false;
    };
    let text = windows::core::HSTRING::from(text);
    if package.SetText(&text).is_err() {
        return false;
    }
    if Clipboard::SetContent(&package).is_err() {
        return false;
    }
    Clipboard::Flush().is_ok()
}

fn paste_text() -> Option<String> {
    let view = Clipboard::GetContent().ok()?;
    let operation = view.GetTextAsync().ok()?;
    operation
        .GetResults()
        .ok()
        .map(|text| text.to_string_lossy())
}

impl Component for ClipboardPage {
    type Message = Message;
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self {
            text: String::from("Hello from Reactor Gallery!"),
            status: String::from("Clipboard ready"),
        }
    }

    fn update(&mut self, message: Message, _: &ComponentContext<Self>) {
        match message {
            Message::Text(text) => self.text = text,
            Message::Copy => {
                self.status = if copy_text(&self.text) {
                    String::from("Copied!")
                } else {
                    String::from("Copy failed")
                };
            }
            Message::Paste => match paste_text() {
                Some(text) => {
                    self.text = text;
                    self.status = String::from("Pasted!");
                }
                None => self.status = String::from("Clipboard has no text"),
            },
        }
    }

    fn view(&self, _: &(), context: &mut ViewContext<Self>) -> View {
        page_content(
            "Clipboard",
            "Read and write text to the system clipboard.",
            [KeyedView::new(
                "copy-paste",
                sample_card(
                    "Copy and Paste",
                    StackPanel::new().spacing(8.0).children((
                        TextBox::new()
                            .text(self.text.clone())
                            .on_text_changed(context.callback(Message::Text))
                            .width(360.0),
                        StackPanel::new()
                            .orientation(Orientation::Horizontal)
                            .spacing(8.0)
                            .children((
                                Button::new()
                                    .on_click(context.message(Message::Copy))
                                    .content("Copy to Clipboard"),
                                Button::new()
                                    .on_click(context.message(Message::Paste))
                                    .content("Paste from Clipboard"),
                            )),
                        TextBlock::new().text(self.status.clone()).opacity(0.6),
                    )),
                    "Clipboard::SetContent(&package); Clipboard::GetContent()",
                ),
            )],
        )
    }
}
