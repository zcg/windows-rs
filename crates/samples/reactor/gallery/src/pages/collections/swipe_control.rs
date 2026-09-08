use crate::controls::*;
use windows_reactor::*;

const DELETE_RED: Color = Color::rgb(197, 15, 31);
const PIN_GREEN: Color = Color::rgb(16, 124, 16);
const FLAG_BLUE: Color = Color::rgb(0, 90, 158);
const WHITE: Color = Color::rgb(255, 255, 255);

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

pub struct SwipeControlPage {
    status: String,
}

#[derive(Clone)]
pub enum Message {
    Invoked(String),
}

impl Component for SwipeControlPage {
    type Message = Message;
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self {
            status: String::from("Swipe a row to reveal actions"),
        }
    }

    fn update(&mut self, message: Message, _: &ComponentContext<Self>) {
        match message {
            Message::Invoked(text) => self.status = format!("Action: {text}"),
        }
    }

    fn view(&self, _: &(), context: &mut ViewContext<Self>) -> View {
        let row = |(sender, subject): &(&str, &str)| {
            Border::new()
                .padding(12.0)
                .content(StackPanel::new().spacing(2.0).children((
                    TextBlock::new().text(*sender),
                    TextBlock::new().text(*subject).opacity(0.6),
                )))
        };
        let swipes = || {
            SwipeItems::new(
                [
                    SwipeItem::new("pin", "Pin")
                        .background(PIN_GREEN)
                        .foreground(WHITE),
                    SwipeItem::new("flag", "Flag")
                        .background(FLAG_BLUE)
                        .foreground(WHITE),
                ],
                [
                    SwipeItem::new("delete", "Delete")
                        .background(DELETE_RED)
                        .foreground(WHITE),
                    SwipeItem::new("archive", "Archive"),
                ],
                context.callback(Message::Invoked),
            )
        };
        page_content(
            "SwipeControl",
            "Reveal contextual actions on an item by swiping left or right.",
            [KeyedView::new(
                "inbox",
                sample_card(
                    "Inbox",
                    StackPanel::new().spacing(8.0).children((
                        ListView::new().collection_slot(
                            ListViewSlot::Items,
                            MESSAGES.iter().map(|message| {
                                let (sender, _) = *message;
                                KeyedView::new(
                                    sender,
                                    ListViewItem::new().content(
                                        SwipeControl::new()
                                            .slots([SlotView::new(
                                                SwipeControlSlot::Content,
                                                row(message),
                                            )])
                                            .swipe_items(swipes()),
                                    ),
                                )
                            }),
                        ),
                        TextBlock::new().text(self.status.clone()).opacity(0.6),
                    )),
                    "SwipeControl::new().swipe_items(SwipeItems::new(left, right, handler))",
                ),
            )],
        )
    }
}
