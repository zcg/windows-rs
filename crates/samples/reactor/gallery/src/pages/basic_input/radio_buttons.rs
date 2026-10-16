use crate::controls::*;
use windows_reactor::*;

pub struct RadioButtonsPage {
    selected: Option<usize>,
}

impl Component for RadioButtonsPage {
    type Message = Option<usize>;
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self { selected: Some(0) }
    }

    fn update(&mut self, selected: Option<usize>, _: &ComponentContext<Self>) {
        self.selected = selected;
    }

    fn view(&self, _: &(), context: &mut ViewContext<Self>) -> View {
        let options = ["Option A", "Option B", "Option C"];
        let label = self
            .selected
            .and_then(|index| options.get(index))
            .unwrap_or(&"?");
        page_content(
            "RadioButtons",
            "Presents a list of options and keeps one selected.",
            [
                KeyedView::new(
                    "basic",
                    sample_card(
                        "Bound options",
                        StackPanel::new().spacing(8.0).children((
                            RadioButtons::new()
                                .items_source(options)
                                .selected_index(self.selected)
                                .on_selection_changed(context.forward())
                                .header("Pick one"),
                            TextBlock::new()
                                .text(format!("Selected: {label}"))
                                .opacity(0.6),
                        )),
                    ),
                ),
                KeyedView::new(
                    "columns",
                    sample_card(
                        "Columns and header",
                        RadioButtons::new()
                            .items_source(["Small", "Medium", "Large", "Extra Large"])
                            .selected_index(1)
                            .max_columns(2)
                            .header("T-shirt size"),
                    ),
                ),
            ],
        )
    }
}
