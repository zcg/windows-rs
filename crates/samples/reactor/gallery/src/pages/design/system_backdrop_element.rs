use crate::controls::*;
use windows_reactor::*;

pub fn system_backdrop_element_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "SystemBackdropElement",
        "Apply backdrop material to an element.",
        vec![sample_card("Overview", text_block("SystemBackdrop brushes (MicaBackdrop, DesktopAcrylicBackdrop) apply materials to window roots.").font_size(13.0), r#"MicaBackdrop / DesktopAcrylicBackdrop"#)],
    )
}
