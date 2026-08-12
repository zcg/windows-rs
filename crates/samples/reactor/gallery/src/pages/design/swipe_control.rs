use crate::controls::*;
use windows_reactor::*;

pub fn swipe_control_page(_: &(), cx: &mut RenderCx) -> Element {
    let (status, set_status) = cx.use_state(String::from("Swipe a row to reveal actions"));

    let left_actions = vec![
        SwipeItemDef::new("Accept").on_invoked({
            let set_status = set_status.clone();
            move |label| set_status.call(format!("Action: {label}"))
        }),
        SwipeItemDef::new("Flag").on_invoked({
            let set_status = set_status.clone();
            move |label| set_status.call(format!("Action: {label}"))
        }),
    ];
    let right_actions = vec![
        SwipeItemDef::new("Delete").on_invoked({
            let set_status = set_status.clone();
            move |label| set_status.call(format!("Action: {label}"))
        }),
        SwipeItemDef::new("Archive").on_invoked({
            let set_status = set_status;
            move |label| set_status.call(format!("Action: {label}"))
        }),
    ];

    let row = |text: &str| -> Element {
        border(text_block(text))
            .padding(Thickness::uniform(16.0))
            .background(ThemeRef::SolidBackground)
            .width(360.0)
            .into()
    };

    page_content(
        "SwipeControl",
        "Reveal contextual actions on an item by swiping left or right.",
        vec![
            sample_card(
                "Swipe Right (Accept / Flag)",
                vstack((
                    swipe_control(row("Swipe Right to reveal actions"))
                        .left_items(left_actions)
                        .width(380.0),
                    text_block(&*status).opacity(0.6),
                ))
                .spacing(8.0),
                r#"swipe_control(row).left_items(vec![SwipeItemDef::new("Accept"), SwipeItemDef::new("Flag")])"#,
            ),
            sample_card(
                "Swipe Left (Delete / Archive)",
                vstack((
                    swipe_control(row("Swipe Left to reveal actions"))
                        .right_items(right_actions)
                        .width(380.0),
                ))
                .spacing(8.0),
                r#"swipe_control(row).right_items(vec![SwipeItemDef::new("Delete"), SwipeItemDef::new("Archive")])"#,
            ),
        ],
    )
}