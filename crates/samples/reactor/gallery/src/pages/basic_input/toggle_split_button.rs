use crate::controls::*;
use windows_reactor::*;

pub fn toggle_split_button_page(_: &(), cx: &mut RenderCx) -> Element {
    let (checked, set_checked) = cx.use_state(false);

    page_content(
        "ToggleSplitButton",
        "A split button that toggles between checked and unchecked states.",
        vec![sample_card(
            "Basic ToggleSplitButton",
            vstack((
                toggle_split_button("Toggle")
                    .is_checked(checked)
                    .on_is_checked_changed(move |v| set_checked.call(v)),
                text_block(format!("IsChecked: {checked}")).foreground(ThemeRef::SecondaryText),
            ))
            .spacing(8.0),
            r#"toggle_split_button("Toggle").is_checked(v).on_is_checked_changed(...)"#,
        )],
    )
}
