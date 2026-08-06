use crate::controls::*;
use windows_reactor::*;

pub fn app_window_title_bar_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "AppWindow Title Bar",
        "Customize the title bar of an AppWindow.",
        vec![sample_card("Overview", text_block("Configure title bar colors, icon, and extensible content areas via AppWindowTitleBar.").font_size(13.0), r#"AppWindowTitleBar.PreferredHeightOption / ButtonBackgroundColor"#)],
    )
}