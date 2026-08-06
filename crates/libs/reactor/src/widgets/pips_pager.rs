use super::*;

#[derive(Clone, Default, Debug, PartialEq)]
pub struct PipsPager {
    pub key: Option<String>,
    pub modifiers: Modifiers,
    pub number_of_pages: i32,
    pub selected_page_index: i32,
    pub max_visible_pips: i32,
    pub on_selected_index_changed: Option<Callback<i32>>,
}
impl PipsPager {
    pub fn new(number_of_pages: i32) -> Self {
        Self {
            number_of_pages,
            max_visible_pips: 5,
            ..Default::default()
        }
    }
    pub fn selected_page_index(mut self, i: i32) -> Self {
        self.selected_page_index = i;
        self
    }
    pub fn max_visible_pips(mut self, n: i32) -> Self {
        self.max_visible_pips = n;
        self
    }
    pub fn on_selected_index_changed(mut self, f: impl IntoCallback<i32>) -> Self {
        self.on_selected_index_changed = Some(f.into_callback());
        self
    }
}
impl Widget for PipsPager {
    widget_header!(ControlKind::PipsPager);
    fn bindings(&self) -> PropBindings {
        let mut out = generated::pips_pager_bindings(self);
        out.push(Binding::Prop(
            Prop::NumberOfPages,
            PropValue::I32(self.number_of_pages),
        ));
        out.push(Binding::Prop(
            Prop::MaxVisiblePips,
            PropValue::I32(self.max_visible_pips),
        ));
        out.push(Binding::Event(
            Event::SelectedIndexChanged,
            self.on_selected_index_changed
                .as_ref()
                .map(|cb| EventHandler::I32(cb.clone())),
        ));
        out
    }
}
pub fn pips_pager(number_of_pages: i32) -> PipsPager {
    PipsPager::new(number_of_pages)
}