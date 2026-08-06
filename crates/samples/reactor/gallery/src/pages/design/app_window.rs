use crate::controls::*;
use windows_reactor::*;

pub fn app_window_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "AppWindow",
        "AppWindow provides a lightweight window with independent configuration.",
        vec![sample_card("Overview", text_block("AppWindow separates the window from the Application frame, enabling multiple independent windows.").font_size(13.0), r#"AppWindow.Create() / Show()"#)],
    )
}