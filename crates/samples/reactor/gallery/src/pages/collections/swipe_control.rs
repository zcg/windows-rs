use crate::controls::*;
use windows_reactor::*;

const MESSAGES: [(&str, &str); 4] = [
    ("Team Standup", "Daily sync moved to 9:30 in room 4B."),
    (
        "Build Server",
        "Nightly build 2413 succeeded with no warnings.",
    ),
    (
        "Lunch Group",
        "Trying the new place across the street today?",
    ),
    ("Weekend Plans", "Hiking on Saturday if the weather holds."),
];

pub struct SwipeControlPage;

impl Component for SwipeControlPage {
    type Message = ();
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self
    }

    fn view(&self, _: &(), _: &mut ViewContext<Self>) -> View {
        let row = |(sender, subject): &(&str, &str)| {
            Border::new()
                .padding(12.0)
                .content(StackPanel::new().spacing(2.0).children((
                    TextBlock::new().text(*sender),
                    TextBlock::new().text(*subject).opacity(0.6),
                )))
        };
        page_content(
            "SwipeControl",
            "Hosts swipe interactions for an item in a list.",
            [KeyedView::new(
                "inbox",
                sample_card(
                    "Inbox",
                    ListView::new().items(MESSAGES.iter().map(|message| {
                        let (sender, _) = *message;
                        DataItem::new(sender, sender)
                            .content(SwipeControl::new().content(row(message)))
                    })),
                ),
            )],
        )
    }
}
