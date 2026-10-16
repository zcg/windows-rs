use crate::controls::*;
use windows_reactor::*;

pub struct AppBarElementContainerPage;

impl Component for AppBarElementContainerPage {
    type Message = ();
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self
    }

    fn view(&self, _: &(), _: &mut ViewContext<Self>) -> View {
        page_content(
            "AppBarElementContainer",
            "Hosts a regular control inside a command bar.",
            [
                KeyedView::new(
                    "basic",
                    sample_card(
                        "Slider hosted in an app bar",
                        AppBarElementContainer::new().height(56.0).content(
                            StackPanel::new()
                                .orientation(Orientation::Horizontal)
                                .spacing(12.0)
                                .children((
                                    SymbolIcon::new().symbol(Symbol::Zoom),
                                    Slider::new().width(200.0).value(40.0),
                                )),
                        ),
                    ),
                ),
                KeyedView::new(
                    "usage",
                    sample_card(
                        "Why a container is needed",
                        InfoBar::new()
                            .severity(InfoBarSeverity::Informational)
                            .title("Command bars accept commands, not arbitrary controls")
                            .message(
                                "A CommandBar only lays out AppBarButton, AppBarToggleButton, \
                                 AppBarSeparator, and AppBarElementContainer. Wrap a Slider, \
                                 ComboBox, or TextBox in an AppBarElementContainer to place it \
                                 on the bar while keeping the command-bar measurements.",
                            )
                            .is_open(true),
                    ),
                ),
            ],
        )
    }
}
