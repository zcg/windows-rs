use crate::controls::*;
use windows_reactor::*;

pub struct ParallaxViewPage;

impl Component for ParallaxViewPage {
    type Message = ();
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self
    }

    fn view(&self, _: &(), _: &mut ViewContext<Self>) -> View {
        let backdrop = Border::new()
            .background(ThemeBrush::Accent)
            .height(200.0)
            .content(TextBlock::new().text("Parallax background").font_size(32.0));
        let foreground = ListView::new().height(300.0).items((0..30).map(|index| {
            let label = format!("Row {index}");
            KeyedView::new(
                label.clone(),
                ListViewItem::new().tag(label.clone()).content(
                    Border::new()
                        .padding(8.0)
                        .content(TextBlock::new().text(label)),
                ),
            )
        }));
        page_content(
            "ParallaxView",
            "Links a background element to a scrolling source.",
            [KeyedView::new(
                "basic",
                sample_card(
                    "Basic ParallaxView",
                    ParallaxView::new()
                        .horizontal_shift(100.0)
                        .child(backdrop)
                        .source(foreground),
                    "ParallaxView::new().child(backdrop).source(list)",
                ),
            )],
        )
    }
}
