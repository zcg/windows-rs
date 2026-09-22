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
            [Keyed::new(
                "basic",
                sample_card(
                    "AppBar Separators",
                    CommandBar::new().primary_commands([
                        Keyed::new("new", AppBarButton::new().label("New")),
                        Keyed::new("sep", AppBarSeparator::new()),
                        Keyed::new("open", AppBarButton::new().label("Open")),
                    ]),
                    "AppBarSeparator::new()",
                ),
            )],
        )
    }
}
