use crate::controls::*;
use windows_reactor::*;

pub fn custom_xaml_conditionals_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "Custom XAML Conditionals",
        "Feature-flag conditional markup.",
        vec![sample_card("Custom XAML Conditionals", text_block("XAML conditionals (custom:Conditional) select markup by compile-time flags in C# projects.").font_size(13.0), r#"custom:Conditional for feature-flag markup"#), sample_card("Reactor", text_block("Reactor uses plain Rust if/else to include or omit elements.").font_size(13.0), r#"if cfg!(feature = "x") { element }"#)],
    )
}
