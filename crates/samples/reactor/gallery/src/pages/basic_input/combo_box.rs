use crate::controls::*;
use windows_reactor::*;

pub struct ComboBoxPage {
    selected: Option<usize>,
}

#[derive(Clone)]
pub enum Message {
    Selected(Option<usize>),
}

impl Component for ComboBoxPage {
    type Message = Message;
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self { selected: None }
    }

    fn update(&mut self, message: Message, _: &ComponentContext<Self>) {
        match message {
            Message::Selected(selected) => self.selected = selected,
        }
    }

    fn view(&self, _: &(), context: &mut ViewContext<Self>) -> View {
        let colors = ["Red", "Green", "Blue", "Yellow"];
        let label = self
            .selected
            .and_then(|index| colors.get(index))
            .unwrap_or(&"(none)");
        page_content(
            "ComboBox",
            "A drop-down list of items a user can select from.",
            [
                KeyedView::new(
                    "basic",
                    sample_card(
                        "Basic ComboBox",
                        StackPanel::new().spacing(8.0).children((
                            ComboBox::new()
                                .items_source(colors)
                                .selected_index(self.selected)
                                .placeholder_text("Pick a color")
                                .on_selection_changed(context.callback(Message::Selected))
                                .header("Color"),
                            TextBlock::new()
                                .text(format!("Selected: {label}"))
                                .opacity(0.6),
                        )),
                    ),
                ),
                KeyedView::new(
                    "editable",
                    sample_card(
                        "Editable ComboBox",
                        ComboBox::new()
                            .items_source(["Cat", "Dog", "Fox"])
                            .placeholder_text("Type or pick")
                            .is_editable(true)
                            .header("Animal"),
                    ),
                ),
            ],
        )
    }
}
