use super::*;

/// Layout used by an [`ItemsView`].
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum ItemsViewLayout {
    /// Vertical stack of items.
    #[default]
    Stack,
    /// Uniform grid of items.
    Grid,
}

/// `Microsoft.UI.Xaml.Controls.ItemsView`. A data-driven list with a
/// flexible layout and item-template factory.
#[derive(Clone, Default, Debug, PartialEq)]
pub struct ItemsView {
    pub key: Option<String>,
    pub modifiers: Modifiers,
    pub items: Vec<String>,
    pub layout: ItemsViewLayout,
    pub selection_mode: ItemsViewSelectionMode,
    pub on_item_invoked: Option<Callback<String>>,
}

impl ItemsView {
    pub fn new(items: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            items: items.into_iter().map(Into::into).collect(),
            selection_mode: ItemsViewSelectionMode::Single,
            ..Default::default()
        }
    }

    pub fn layout(mut self, layout: ItemsViewLayout) -> Self {
        self.layout = layout;
        self
    }

    pub fn selection_mode(mut self, mode: ItemsViewSelectionMode) -> Self {
        self.selection_mode = mode;
        self
    }

    pub fn on_item_invoked(mut self, f: impl IntoCallback<String>) -> Self {
        self.on_item_invoked = Some(f.into_callback());
        self
    }
}

impl Widget for ItemsView {
    widget_header!(ControlKind::ItemsView);
    fn bindings(&self) -> PropBindings {
        let mut out = generated::items_view_bindings(self);
        // Layout must be applied before ItemsSource: ItemsView lays out as
        // soon as a source is set and fails without a layout.
        out.push(Binding::Prop(
            Prop::ItemsViewLayout,
            PropValue::I32(self.layout as i32),
        ));
        out.push(Binding::Prop(
            Prop::ItemsViewSource,
            PropValue::StrList(self.items.clone()),
        ));
        out
    }
}

pub fn items_view(items: impl IntoIterator<Item = impl Into<String>>) -> ItemsView {
    ItemsView::new(items)
}
