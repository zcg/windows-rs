use crate::controls::*;
use windows_reactor::*;

pub fn create_multiple_windows_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "Create Multiple Windows",
        "Open more than one window for an app.",
        vec![sample_card("Overview", text_block("Reactor supports multiple windows; each ReactorWindow registers its own host.").font_size(13.0), r#"ReactorWindow::new().run(root)"#)],
    )
}