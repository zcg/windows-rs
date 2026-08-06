use crate::controls::*;
use windows_reactor::*;

pub fn content_island_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "Content Islands",
        "Host XAML content inside a Win32 HWND.",
        vec![sample_card("Overview", text_block("ContentIsland hosts WinUI content in classic desktop windows via DesktopWindowXamlSource.").font_size(13.0), r#"DesktopWindowXamlSource"#)],
    )
}