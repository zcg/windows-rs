use super::*;

/// `Microsoft.UI.Xaml.Controls.ToggleSplitButton`. A split button with a
/// primary action and a secondary dropdown action, plus a checked state.
#[derive(Clone, Default, Debug, PartialEq)]
pub struct ToggleSplitButton {
    pub key: Option<String>,
    pub modifiers: Modifiers,
    pub content: Option<String>,
    pub is_checked: bool,
    pub is_enabled: bool,
    pub on_click: Option<Callback<()>>,
    pub on_is_checked_changed: Option<Callback<bool>>,
}
impl ToggleSplitButton {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: Some(content.into()),
            is_enabled: true,
            ..Default::default()
        }
    }
    pub fn is_checked(mut self, v: bool) -> Self {
        self.is_checked = v;
        self
    }
    pub fn enabled(mut self, v: bool) -> Self {
        self.is_enabled = v;
        self
    }
    pub fn on_click(mut self, f: impl IntoUnitCallback) -> Self {
        self.on_click = Some(f.into_unit_callback());
        self
    }
    pub fn on_is_checked_changed(mut self, f: impl IntoCallback<bool>) -> Self {
        self.on_is_checked_changed = Some(f.into_callback());
        self
    }
}
impl Widget for ToggleSplitButton {
    widget_header!(ControlKind::ToggleSplitButton);
    fn bindings(&self) -> PropBindings {
        let mut out = generated::toggle_split_button_bindings(self);
        out.push(Binding::Event(
            Event::IsCheckedChanged,
            self.on_is_checked_changed
                .as_ref()
                .map(|cb| EventHandler::Bool(cb.clone())),
        ));
        out
    }
}
pub fn toggle_split_button(content: impl Into<String>) -> ToggleSplitButton {
    ToggleSplitButton::new(content)
}