use crate::controls::*;
use windows_reactor::*;

pub fn xaml_comp_interop_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "XAML/COM Interop",
        "Interop between XAML and classic COM APIs.",
        vec![sample_card("Overview", text_block("windows-rs exposes the underlying COM interfaces (ISwapChainPanelNative, IWindowNative, etc.) for interop.").font_size(13.0), r#"ISwapChainPanelNative / IWindowNative"#)],
    )
}