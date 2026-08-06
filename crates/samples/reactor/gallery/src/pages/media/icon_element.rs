use crate::controls::*;
use windows_reactor::*;

pub fn icon_element_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "IconElement",
        "Icons for buttons and command surfaces: SymbolIcon, FontIcon, and BitmapIcon.",
        vec![
            sample_card(
                "SymbolIcon",
                hstack((
                    button("Home").icon(Icon::symbol(Symbol::Home)),
                    button("Settings").icon(Icon::symbol(Symbol::Setting)),
                    button("Delete").icon(Icon::symbol(Symbol::Delete)),
                ))
                .spacing(8.0),
                r#"button("Home").icon(Icon::symbol(Symbol::Home))"#,
            ),
            sample_card(
                "FontIcon (Segoe Fluent Icons)",
                hstack((
                    button("Play").icon(Icon::font_family("\u{E768}", "Segoe Fluent Icons")),
                    button("Pause").icon(Icon::font_family("\u{E769}", "Segoe Fluent Icons")),
                ))
                .spacing(8.0),
                r#"Icon::font_family("\u{E768}", "Segoe Fluent Icons")"#,
            ),
            sample_card(
                "Icon on Button",
                button("Open").icon(Icon::symbol(Symbol::OpenFile)),
                r#"button("Open").icon(Icon::symbol(Symbol::OpenFile))"#,
            ),
        ],
    )
}