use crate::controls::*;
use windows_reactor::*;

pub struct ItemsRepeaterScrollHostPage;

impl Component for ItemsRepeaterScrollHostPage {
    type Message = ();
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self
    }

    fn view(&self, _: &(), _: &mut ViewContext<Self>) -> View {
        page_content(
            "ItemsRepeaterScrollHost",
            "Bridges an ItemsRepeater to a ScrollViewer it does not own.",
            [
                KeyedView::new(
                    "basic",
                    sample_card(
                        "Repeater scroll host",
                        ItemsRepeaterScrollHost::new().height(160.0),
                        "ItemsRepeaterScrollHost::new()",
                    ),
                ),
                KeyedView::new(
                    "usage",
                    sample_card(
                        "When to use it",
                        InfoBar::new()
                            .severity(InfoBarSeverity::Informational)
                            .title("For pre-1809 scroll viewers")
                            .message(
                                "The host forwards the scroll offsets of a ScrollViewer that \
                                 ItemsRepeater does not create itself, so the repeater can \
                                 virtualize against it. Reactor's ItemsRepeater already owns a \
                                 ScrollViewer, so most pages never need this control.",
                            )
                            .is_open(true),
                        "ItemsRepeaterScrollHost::new()",
                    ),
                ),
            ],
        )
    }
}
