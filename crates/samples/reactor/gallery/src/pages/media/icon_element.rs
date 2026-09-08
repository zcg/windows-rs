use crate::controls::*;
use windows_reactor::*;

pub struct IconElementPage;

impl Component for IconElementPage {
    type Message = ();
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self
    }

    fn view(&self, _: &(), _: &mut ViewContext<Self>) -> View {
        let row = |icon: Symbol| {
            StackPanel::new()
                .orientation(Orientation::Horizontal)
                .spacing(8.0)
                .children((
                    SymbolIcon::new().symbol(icon),
                    Button::new().content(SymbolIcon::new().symbol(icon)),
                ))
        };
        page_content(
            "IconElement",
            "Symbol, font, and path icons for buttons and command surfaces.",
            [
                KeyedView::new(
                    "symbol-icons",
                    sample_card(
                        "SymbolIcon",
                        StackPanel::new().spacing(8.0).children((
                            row(Symbol::Add),
                            row(Symbol::Accept),
                            row(Symbol::Delete),
                        )),
                        "SymbolIcon::new().symbol(Symbol::Add)",
                    ),
                ),
                KeyedView::new(
                    "font-path-icons",
                    sample_card(
                        "FontIcon and PathIcon",
                        StackPanel::new().spacing(8.0).children((
                            FontIcon::new().glyph("\u{E700}"),
                            PathIcon::new().data("M 0 0 L 10 10 Z"),
                        )),
                        "FontIcon::new().glyph(\"\\u{E700}\")",
                    ),
                ),
            ],
        )
    }
}
