use crate::controls::*;
use windows_reactor::*;

pub fn binding_page(_: &(), cx: &mut RenderCx) -> Element {
    let (name, set_name) = cx.use_state(String::from("World"));

    page_content(
        "Binding",
        "Data binding connects UI properties to underlying data. Reactor achieves this with hooks and state.",
        vec![sample_card(
            "State-driven Binding",
            vstack((
                text_box(name.clone()).on_text_changed(move |v| set_name.call(v)),
                text_block(format!("Hello, {name}!")),
            ))
            .spacing(8.0),
            r#"let (name, set_name) = cx.use_state(String::from("World"));
text_box(name).on_text_changed(move |v| set_name.call(v));
text_block(format!("Hello, {name}!"))"#,
        )],
    )
}
