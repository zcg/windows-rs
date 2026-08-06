use crate::controls::*;
use windows_reactor::*;

pub fn radial_gradient_brush_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "RadialGradientBrush",
        "A radial gradient brush fills a shape with colors radiating from a center.",
        vec![
            sample_card(
                "Overview",
                text_block("RadialGradientBrush is available in the WinUI binding set; reactor exposes solid brushes today.").font_size(13.0),
                r#"RadialGradientBrush GradientOrigin, RadiusX, RadiusY"#,
            ),
        ],
    )
}