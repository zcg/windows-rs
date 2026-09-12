use crate::controls::*;
use windows_reactor::*;

pub struct MenuFlyoutPage {
    last_action: String,
    last_format: String,
    last_state: String,
}

impl Component for MenuFlyoutPage {
    type Message = (String, String);
    type Input = ();

    fn create(_input: &(), _context: &ComponentContext<Self>) -> Self {
        Self {
            last_action: "(none)".to_string(),
            last_format: "(none)".to_string(),
            last_state: "(none)".to_string(),
        }
    }

    fn update(&mut self, message: (String, String), _context: &ComponentContext<Self>) {
        let (which, value) = message;
        match which.as_str() {
            "action" => self.last_action = value,
            "format" => self.last_format = value,
            "state" => self.last_state = value,
            _ => {}
        }
    }

    fn view(&self, _input: &(), context: &mut ViewContext<Self>) -> View {
        page_content(
            "MenuFlyout",
            "A flyout that displays a list of menu commands.",
            [
                KeyedView::new(
                    "basic",
                    sample_card(
                        "Basic MenuFlyout",
                        StackPanel::new().spacing(8.0).children((
                            Button::new().content("Open Menu").menu(Menu::new(
                                [
                                    MenuItem::item("cut", "Cut"),
                                    MenuItem::item("copy", "Copy"),
                                    MenuItem::item("paste", "Paste"),
                                ],
                                context.callback(|label| ("action".to_string(), label)),
                            )),
                            TextBlock::new()
                                .text(format!("Last action: {}", self.last_action))
                                .opacity(0.6),
                        )),
                        r#"Button::new()
    .content("Open Menu")
    .menu(Menu::new(
        [
            MenuItem::item("cut", "Cut"),
            MenuItem::item("copy", "Copy"),
            MenuItem::item("paste", "Paste"),
        ],
        context.callback(|label| ("action".to_string(), label)),
    ))"#,
                    ),
                ),
                KeyedView::new(
                    "separators",
                    sample_card(
                        "MenuFlyout with Separators",
                        StackPanel::new().spacing(8.0).children((
                            Button::new().content("Format").menu(Menu::new(
                                [
                                    MenuItem::item("bold", "Bold"),
                                    MenuItem::item("italic", "Italic"),
                                    MenuItem::separator("separator"),
                                    MenuItem::item("underline", "Underline"),
                                    MenuItem::item("strikethrough", "Strikethrough"),
                                ],
                                context.callback(|label| ("format".to_string(), label)),
                            )),
                            TextBlock::new()
                                .text(format!("Last format: {}", self.last_format))
                                .opacity(0.6),
                        )),
                        r#"Button::new()
    .content("Format")
    .menu(Menu::new(
        [
            MenuItem::item("bold", "Bold"),
            MenuItem::item("italic", "Italic"),
            MenuItem::separator("separator"),
            MenuItem::item("underline", "Underline"),
            MenuItem::item("strikethrough", "Strikethrough"),
        ],
        context.callback(|label| ("format".to_string(), label)),
    ))"#,
                    ),
                ),
                KeyedView::new(
                    "state",
                    sample_card(
                        "Checkable and nested items",
                        StackPanel::new().spacing(8.0).children((
                            Button::new().content("View options").menu(Menu::new(
                                [
                                    MenuItem::toggle("wrap", "Word wrap", true),
                                    MenuItem::toggle("ruler", "Ruler", false),
                                    MenuItem::separator("state-separator"),
                                    MenuItem::radio("zoom-100", "Zoom 100%", "zoom", true),
                                    MenuItem::radio("zoom-125", "Zoom 125%", "zoom", false),
                                    MenuItem::radio("zoom-150", "Zoom 150%", "zoom", false),
                                    MenuItem::separator("nested-separator"),
                                    MenuItem::submenu(
                                        "theme",
                                        "Theme",
                                        [
                                            MenuItem::item("theme-light", "Light"),
                                            MenuItem::item("theme-dark", "Dark"),
                                        ],
                                    ),
                                ],
                                context.callback(|label| ("state".to_string(), label)),
                            )),
                            TextBlock::new()
                                .text(format!("Last state: {}", self.last_state))
                                .opacity(0.6),
                        )),
                        r#"MenuItem::toggle(key, label, checked)
MenuItem::radio(key, label, "zoom", checked)
MenuItem::submenu(key, label, items)"#,
                    ),
                ),
            ],
        )
    }
}
