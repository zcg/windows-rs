use crate::controls::*;
use windows_reactor::*;

pub fn templates_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "Templates",
        "Control templates define the visual structure of a control.",
        vec![
            sample_card(
                "Overview",
                vstack(vec![text_block("WinUI controls are lookless - their visuals come from ControlTemplate resources that can be restyled.").font_size(13.0).into(), text_block("Reactor widgets abstract common templates so apps stay declarative.").font_size(13.0).into()]).spacing(8.0),
                r#"ControlTemplate restyles the control visual tree"#,
            ),
        ],
    )
}