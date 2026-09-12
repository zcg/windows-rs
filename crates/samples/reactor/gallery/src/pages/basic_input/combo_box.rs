use crate::controls::*;
use windows_reactor::*;

pub struct ComboBoxPage {
    selected: Option<usize>,
    list: ElementRef<ComboBox>,
    offsets: Option<ScrollOffsets>,
}

#[derive(Clone)]
pub enum Message {
    Selected(Option<usize>),
    Offsets(ScrollOffsets),
}

impl Component for ComboBoxPage {
    type Message = Message;
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self {
            selected: None,
            list: ElementRef::new(),
            offsets: None,
        }
    }

    fn update(&mut self, message: Message, _: &ComponentContext<Self>) {
        match message {
            Message::Selected(selected) => self.selected = selected,
            Message::Offsets(offsets) => self.offsets = Some(offsets),
        }
    }

    fn view(&self, _: &(), context: &mut ViewContext<Self>) -> View {
        let colors = ["Red", "Green", "Blue", "Yellow"];
        let label = self
            .selected
            .and_then(|index| colors.get(index))
            .unwrap_or(&"(none)");
        let items = (0..50)
            .map(|index| format!("Item {index:02}"))
            .collect::<Vec<_>>();
        let offsets = self.offsets.map_or_else(
            || String::from("Open the list and scroll to report offsets"),
            |offsets| {
                format!(
                    "h={:.0} v={:.0} zoom={:.2}",
                    offsets.horizontal, offsets.vertical, offsets.zoom
                )
            },
        );
        let list = self.list.clone();
        let sender = context.sender();
        context.use_effect("list-offsets", (), move || {
            let observation = list.observe_scroll_offsets(move |offsets| {
                sender.send(Message::Offsets(offsets));
            });
            Some(Box::new(move || drop(observation)))
        });
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
                        "ComboBox::new().items_source(colors).on_selection_changed(handler)",
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
                        "ComboBox::new().items_source(items).is_editable(true)",
                    ),
                ),
                KeyedView::new(
                    "observed",
                    sample_card(
                        "Observed Drop-Down List",
                        StackPanel::new().spacing(8.0).children((
                            ComboBox::new()
                                .items_source(items)
                                .placeholder_text("Open and scroll")
                                .element_ref(&self.list)
                                .header("Items"),
                            TextBlock::new().text(offsets).opacity(0.6),
                        )),
                        "list.observe_scroll_offsets(handler)",
                    ),
                ),
            ],
        )
    }
}
