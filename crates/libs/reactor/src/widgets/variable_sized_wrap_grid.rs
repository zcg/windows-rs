use super::*;

/// `Microsoft.UI.Xaml.Controls.VariableSizedWrapGrid`. Arranges items in a
/// grid where items can span multiple rows and columns.
#[derive(Clone, Default, Debug, PartialEq)]
pub struct VariableSizedWrapGrid {
    pub key: Option<String>,
    pub modifiers: Modifiers,
    pub children: Vec<Element>,
    pub item_height: f64,
    pub item_width: f64,
    pub maximum_rows_or_columns: i32,
    pub orientation: Orientation,
}
impl VariableSizedWrapGrid {
    pub fn new() -> Self {
        Self {
            item_height: 44.0,
            item_width: 44.0,
            maximum_rows_or_columns: 0,
            orientation: Orientation::Horizontal,
            ..Default::default()
        }
    }
    pub fn item_height(mut self, v: f64) -> Self {
        self.item_height = v;
        self
    }
    pub fn item_width(mut self, v: f64) -> Self {
        self.item_width = v;
        self
    }
    pub fn maximum_rows_or_columns(mut self, v: i32) -> Self {
        self.maximum_rows_or_columns = v;
        self
    }
    pub fn orientation(mut self, v: Orientation) -> Self {
        self.orientation = v;
        self
    }
    pub fn child(mut self, el: impl Into<Element>) -> Self {
        self.children.push(el.into());
        self
    }
    pub fn children(mut self, els: impl IntoIterator<Item = Element>) -> Self {
        self.children.extend(els);
        self
    }
}

/// Attached property for children of [`VariableSizedWrapGrid`]. Set via
/// [`Element::wrap_grid_row_span`] / [`Element::wrap_grid_column_span`].
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WrapGridSpan {
    pub row_span: i32,
    pub column_span: i32,
}

impl Default for WrapGridSpan {
    fn default() -> Self {
        Self {
            row_span: 1,
            column_span: 1,
        }
    }
}

impl Widget for VariableSizedWrapGrid {
    widget_header!(ControlKind::VariableSizedWrapGrid);
    fn children(&self) -> Children<'_> {
        Children::Keyed(&self.children)
    }
    fn bindings(&self) -> PropBindings {
        generated::variable_sized_wrap_grid_bindings(self)
    }
}
pub fn variable_sized_wrap_grid() -> VariableSizedWrapGrid {
    VariableSizedWrapGrid::new()
}
