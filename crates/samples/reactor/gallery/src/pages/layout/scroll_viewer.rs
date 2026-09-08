use crate::controls::*;
use windows_reactor::*;

pub struct ScrollViewerPage {
    events: u32,
    viewer: ElementRef<ScrollViewer>,
    offsets: Option<ScrollOffsets>,
}

#[derive(Clone)]
pub enum Message {
    ViewChanged,
    Offsets(ScrollOffsets),
}

impl Component for ScrollViewerPage {
    type Message = Message;
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self {
            events: 0,
            viewer: ElementRef::new(),
            offsets: None,
        }
    }

    fn update(&mut self, message: Message, _: &ComponentContext<Self>) {
        match message {
            Message::ViewChanged => self.events += 1,
            Message::Offsets(offsets) => self.offsets = Some(offsets),
        }
    }

    fn view(&self, _: &(), context: &mut ViewContext<Self>) -> View {
        let long_text = (0..40)
            .map(|index| format!("Line {index} - scroll to fire ViewChanged"))
            .collect::<Vec<_>>()
            .join("\n");
        let offsets = self.offsets.map_or_else(
            || String::from("Scroll to report offsets"),
            |offsets| {
                format!(
                    "h={:.0} v={:.0} zoom={:.2}",
                    offsets.horizontal, offsets.vertical, offsets.zoom
                )
            },
        );
        let viewer = self.viewer.clone();
        let sender = context.sender();
        context.use_effect("scroll-offsets", (), move || {
            let observation = viewer.observe_scroll_offsets(move |offsets| {
                sender.send(Message::Offsets(offsets));
            });
            Some(Box::new(move || drop(observation)))
        });
        page_content(
            "ScrollViewer",
            "Scrolls content with configurable scroll bar visibility.",
            [
                KeyedView::new(
                    "basic",
                    sample_card(
                        "Basic ScrollViewer",
                        StackPanel::new().spacing(8.0).children((
                            ScrollViewer::new()
                                .height(220.0)
                                .on_view_changed(context.message(Message::ViewChanged))
                                .content(
                                    TextBlock::new()
                                        .text(long_text.clone())
                                        .text_wrapping(TextWrapping::Wrap),
                                ),
                            TextBlock::new()
                                .text(format!("ViewChanged fired {} times", self.events))
                                .opacity(0.6),
                        )),
                        "ScrollViewer::new().content(view).on_view_changed(handler)",
                    ),
                ),
                KeyedView::new(
                    "observed",
                    sample_card(
                        "Observed ScrollViewer",
                        StackPanel::new().spacing(8.0).children((
                            ScrollViewer::new()
                                .height(220.0)
                                .element_ref(&self.viewer)
                                .content(
                                    TextBlock::new()
                                        .text(long_text)
                                        .text_wrapping(TextWrapping::Wrap),
                                ),
                            TextBlock::new().text(offsets).opacity(0.6),
                        )),
                        "viewer.observe_scroll_offsets(handler)",
                    ),
                ),
            ],
        )
    }
}
