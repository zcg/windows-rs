use crate::controls::*;
use windows_reactor::*;

const GROUPS: [&str; 4] = ["Fruits", "Vegetables", "Grains", "Dairy"];

pub struct SemanticZoomPage {
    zoomed_in: bool,
}

#[derive(Clone)]
pub enum Message {
    Zoomed(bool),
}

impl Component for SemanticZoomPage {
    type Message = Message;
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self { zoomed_in: true }
    }

    fn update(&mut self, message: Message, _: &ComponentContext<Self>) {
        match message {
            Message::Zoomed(value) => self.zoomed_in = value,
        }
    }

    fn view(&self, _: &(), context: &mut ViewContext<Self>) -> View {
        let zoomed_in =
            ListView::new()
                .height(240.0)
                .items(["Apple", "Broccoli", "Rice"].into_iter().map(|item| {
                    DataItem::new(item, item).content(
                        ListViewItem::new()
                            .tag(item)
                            .content(TextBlock::new().text(item)),
                    )
                }));
        let zoomed_out = ListView::new()
            .height(240.0)
            .items(GROUPS.into_iter().map(|group| {
                DataItem::new(group, group).content(
                    ListViewItem::new()
                        .tag(group)
                        .content(TextBlock::new().text(group)),
                )
            }));
        page_content(
            "SemanticZoom",
            "Pairs a detailed zoomed-in view with a compact zoomed-out overview.",
            [KeyedView::new(
                "basic",
                sample_card(
                    "Basic SemanticZoom",
                    StackPanel::new().spacing(8.0).children((
                        SemanticZoom::new()
                            .is_zoomed_in_view_active(self.zoomed_in)
                            .zoomed_in_view(zoomed_in)
                            .zoomed_out_view(zoomed_out),
                        StackPanel::new()
                            .orientation(Orientation::Horizontal)
                            .spacing(8.0)
                            .children((
                                Button::new()
                                    .on_click(context.message(Message::Zoomed(true)))
                                    .content("Zoom in"),
                                Button::new()
                                    .on_click(context.message(Message::Zoomed(false)))
                                    .content("Zoom out"),
                            )),
                    )),
                ),
            )],
        )
    }
}
