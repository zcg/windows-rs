use crate::controls::*;
use windows_reactor::*;

/// A reusable counter component: a value with increment/decrement buttons.
fn counter_component(cx: &mut RenderCx, label: &str) -> Element {
    let counter = cx.use_ref(0i32);
    let label = String::from(label);
    hstack((
        text_block(label).font_size(14.0),
        button("-").on_click({
            let counter = counter.clone();
            move || { *counter.borrow_mut() -= 1; }
        }),
        text_block("0").font_size(16.0).bold().width(48.0),
        button("+").on_click({
            let counter = counter.clone();
            move || { *counter.borrow_mut() += 1; }
        }),
    ))
    .spacing(8.0)
    .vertical_alignment(VerticalAlignment::Center)
    .into()
}

pub fn custom_user_controls_page(_: &(), cx: &mut RenderCx) -> Element {
    page_content(
        "Custom User Controls",
        "Compose reusable custom controls that wrap state and render logic.",
        vec![
            sample_card(
                "Reusable Counter Component",
                vstack((
                    counter_component(cx, "Count:"),
                    text_block("A custom component encapsulates state (use_ref) and renders buttons + value.").font_size(12.0).opacity(0.6),
                ))
                .spacing(8.0),
                r#"fn counter_component(cx: &mut RenderCx) -> Element {
    let counter = cx.use_ref(0i32);
    hstack((button("-"), text_block("0"), button("+")))
}"#,
            ),
            sample_card(
                "Composition over Inheritance",
                vstack((
                    counter_component(cx, "Tickets:"),
                    text_block("Reactor components compose built-in widgets into reusable pieces - the equivalent of custom controls in C#.").font_size(12.0).opacity(0.6),
                ))
                .spacing(8.0),
                r#"component(fn, props) / plain functions returning Element"#,
            ),
        ],
    )
}