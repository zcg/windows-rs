use crate::controls::*;
use windows_reactor::*;

pub fn system_backdrops_page(_: &(), cx: &mut RenderCx) -> Element {
    let (selected, set_selected) = cx.use_state(0_i32);

    let options: Vec<String> = ["Mica", "Mica Alt", "Acrylic", "None"]
        .into_iter()
        .map(String::from)
        .collect();

    page_content(
        "System Backdrops",
        "System backdrops apply Mica or Acrylic material to the window background.",
        vec![sample_card(
            "Backdrop Switcher",
            vstack((list_view(options, |item, _idx| {
                text_block(item.clone()).padding(Thickness::uniform(10.0))
            })
            .with_key_selector(|item| item.clone())
            .selected_index(selected)
            .on_selection_changed({
                let set_selected = set_selected;
                move |index: i32| {
                    let backdrop = match index {
                        0 => Some(Backdrop::Mica),
                        1 => Some(Backdrop::MicaAlt),
                        2 => Some(Backdrop::Acrylic),
                        _ => None,
                    };
                    set_backdrop(backdrop);
                    set_selected.call(index);
                }
            })
            .height(180.0),))
            .spacing(8.0),
            r#"set_backdrop(Some(Backdrop::Mica));
set_backdrop(Some(Backdrop::Acrylic));"#,
        )],
    )
}
