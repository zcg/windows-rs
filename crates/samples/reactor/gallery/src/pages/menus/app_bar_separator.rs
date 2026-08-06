use crate::controls::*;
use windows_reactor::*;

pub fn app_bar_separator_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "AppBarSeparator",
        "A vertical separator between AppBarButtons in an AppBar or CommandBar.",
        vec![
            sample_card(
                "Separators in a CommandBar",
                command_bar(vec![
                    CommandBarCommandDef::Button { label: "Add".into(), icon: Some(Symbol::Add.into()) },
                    CommandBarCommandDef::Button { label: "Edit".into(), icon: Some(Symbol::Edit.into()) },
                    CommandBarCommandDef::Separator,
                    CommandBarCommandDef::Button { label: "Delete".into(), icon: Some(Symbol::Delete.into()) },
                ]),
                r#"CommandBarCommandDef::Separator between button groups"#,
            ),
        ],
    )
}