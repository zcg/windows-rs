use crate::controls::*;
use windows_reactor::*;

pub fn xaml_resources_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "XAML Resources",
        "Resources are reusable objects such as brushes, styles, and templates.",
        vec![
            sample_card(
                "Overview",
                vstack(vec![text_block("Resources live in ResourceDictionary objects and are looked up by key.").font_size(13.0).into(), text_block("Reactor exposes resources through the resources modifier.").font_size(13.0).into()]).spacing(8.0),
                r#"resources: ResourceDictionary with brushes, styles, templates"#,
            ),
        ],
    )
}