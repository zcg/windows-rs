use crate::controls::*;
use windows_reactor::*;

pub struct SystemBackdropElementPage;

impl Component for SystemBackdropElementPage {
    type Message = ();
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self
    }

    fn view(&self, _: &(), _: &mut ViewContext<Self>) -> View {
        page_content(
            "SystemBackdropElement",
            "Applies a window backdrop material to a single element.",
            [
                KeyedView::new(
                    "basic",
                    sample_card(
                        "Backdrop element",
                        SystemBackdropElement::new().height(200.0),
                    ),
                ),
                KeyedView::new(
                    "backdrops",
                    sample_card(
                        "Backdrop materials",
                        InfoBar::new()
                            .severity(InfoBarSeverity::Informational)
                            .title("Element-scoped Mica and Acrylic")
                            .message(
                                "A window backdrop such as Mica or Acrylic is applied to the whole \
                                 window through WindowVisuals. SystemBackdropElement samples the \
                                 same material for one subtree, so a card can show the desktop \
                                 tint without changing the rest of the window.",
                            )
                            .is_open(true),
                    ),
                ),
            ],
        )
    }
}
