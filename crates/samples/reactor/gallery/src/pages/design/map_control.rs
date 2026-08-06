use crate::controls::*;
use windows_reactor::*;

pub fn map_control_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "Map Control",
        "Display maps and location data.",
        vec![sample_card("Overview", text_block("MapControl requires the Windows.Services.Maps service and an authentication token.").font_size(13.0), r#"MapControl with MapServiceToken"#)],
    )
}