use crate::controls::*;
use windows_reactor::*;

pub struct FramePage;

impl Component for FramePage {
    type Message = ();
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self
    }

    fn view(&self, _: &(), _: &mut ViewContext<Self>) -> View {
        page_content(
            "Frame",
            "A navigation host that keeps a journal of visited content.",
            [
                KeyedView::new(
                    "basic",
                    sample_card(
                        "Frame content",
                        Frame::new().height(180.0).content(
                            Border::new().padding(20.0).content(
                                StackPanel::new().spacing(6.0).children((
                                    TextBlock::new()
                                        .text("Frame content")
                                        .font_size(18.0)
                                        .font_weight(FontWeight::SEMI_BOLD),
                                    TextBlock::new()
                                        .text(
                                            "A Frame renders whatever content is assigned to \
                                             it and records each assignment in its back stack.",
                                        )
                                        .opacity(0.7)
                                        .text_wrapping(TextWrapping::Wrap),
                                )),
                            ),
                        ),
                        "Frame::new().content(view)",
                    ),
                ),
                KeyedView::new(
                    "navigation",
                    sample_card(
                        "Navigation model",
                        InfoBar::new()
                            .severity(InfoBarSeverity::Informational)
                            .title("Frame navigation in a Reactor app")
                            .message(
                                "Frame navigates between XAML page types and keeps them alive in \
                                 a journal. A Reactor app swaps components through its own router \
                                 instead, so the gallery uses Frame only where a page type is \
                                 supplied by the platform, such as hosted XAML content.",
                            )
                            .is_open(true),
                        "Frame::new()",
                    ),
                ),
            ],
        )
    }
}
