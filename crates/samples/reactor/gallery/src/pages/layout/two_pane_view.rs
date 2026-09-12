use crate::controls::*;
use windows_reactor::*;

pub struct TwoPaneViewPage;

impl Component for TwoPaneViewPage {
    type Message = ();
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self
    }

    fn view(&self, _: &(), _: &mut ViewContext<Self>) -> View {
        page_content(
            "TwoPaneView",
            "Lays out two panes side by side, or stacked when the window narrows.",
            [KeyedView::new(
                "basic",
                sample_card(
                    "Two panes",
                    TwoPaneView::new()
                        .pane1(pane(
                            "Pane 1",
                            "Stays visible; the split collapses to this pane first.",
                        ))
                        .pane2(pane(
                            "Pane 2",
                            "Folds away when the pane area gets too narrow.",
                        ))
                        .height(240.0),
                    "TwoPaneView::new().pane1(view).pane2(view)",
                ),
            )],
        )
    }
}

fn pane(title: &str, body: &str) -> View {
    Border::new()
        .background(ThemeBrush::CardBackground)
        .border_thickness(1.0)
        .border_brush(ThemeBrush::CardStroke)
        .corner_radius(6.0)
        .padding(16.0)
        .content(
            StackPanel::new().spacing(6.0).children((
                TextBlock::new()
                    .text(title)
                    .font_weight(FontWeight::SEMI_BOLD),
                TextBlock::new()
                    .text(body)
                    .opacity(0.7)
                    .text_wrapping(TextWrapping::Wrap),
            )),
        )
}
