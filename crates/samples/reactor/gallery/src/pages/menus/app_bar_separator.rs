use crate::controls::*;
use windows_reactor::*;

pub struct AppBarSeparatorPage;

impl Component for AppBarSeparatorPage {
    type Message = ();
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self
    }

    fn view(&self, _: &(), _: &mut ViewContext<Self>) -> View {
        page_content(
            "AppBarSeparator",
            "A separator between groups of command-bar buttons.",
            [KeyedView::new(
                "basic",
                sample_card(
                    "AppBar Separators",
                    CommandBar::new().keyed_primary_commands([
                        KeyedView::new("new", AppBarButton::new().label("New")),
                        KeyedView::new("sep", AppBarSeparator::new()),
                        KeyedView::new("open", AppBarButton::new().label("Open")),
                    ]),
                    "AppBarSeparator::new()",
                ),
            )],
        )
    }
}
