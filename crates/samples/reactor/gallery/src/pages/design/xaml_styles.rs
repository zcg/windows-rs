use crate::controls::*;
use windows_reactor::*;

pub fn xaml_styles_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "XAML Styles",
        "Styles group property values that can be applied to controls.",
        vec![
            sample_card(
                "Overview",
                vstack(vec![text_block("A Style targets a control type and sets property values.").font_size(13.0).into(), text_block("Styles promote consistency across an app.").font_size(13.0).into()]).spacing(8.0),
                r#"Style TargetType=Button sets padding, background, corner radius"#,
            ),
        ],
    )
}