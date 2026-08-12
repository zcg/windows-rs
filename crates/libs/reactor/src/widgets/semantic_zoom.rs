use super::*;

/// `Microsoft.UI.Xaml.Controls.SemanticZoom`. Pairs a detailed zoomed-in
/// view with a compact zoomed-out overview of the same data. Both views
/// must be list-based controls that implement `ISemanticZoomInformation`
/// (e.g. `ListView` or `GridView`).
#[derive(Clone, Debug, PartialEq)]
pub struct SemanticZoom {
    pub key: Option<String>,
    pub modifiers: Modifiers,
    pub zoomed_in: Box<Element>,
    pub zoomed_out: Box<Element>,
    pub is_zoomed_in_view_active: bool,
}

impl Default for SemanticZoom {
    fn default() -> Self {
        Self {
            key: None,
            modifiers: Modifiers::default(),
            zoomed_in: Box::new(Element::Empty),
            zoomed_out: Box::new(Element::Empty),
            is_zoomed_in_view_active: true,
        }
    }
}

impl SemanticZoom {
    pub fn new(zoomed_in: impl Into<Element>) -> Self {
        Self {
            zoomed_in: Box::new(zoomed_in.into()),
            ..Default::default()
        }
    }

    pub fn zoomed_out(mut self, zoomed_out: impl Into<Element>) -> Self {
        self.zoomed_out = Box::new(zoomed_out.into());
        self
    }

    pub fn is_zoomed_in_view_active(mut self, active: bool) -> Self {
        self.is_zoomed_in_view_active = active;
        self
    }
}

impl Widget for SemanticZoom {
    widget_header!(ControlKind::SemanticZoom);
    fn bindings(&self) -> PropBindings {
        generated::semantic_zoom_bindings(self)
    }
    fn children(&self) -> Children<'_> {
        Children::PositionalSingle(&self.zoomed_in)
    }
    fn pane_element(&self) -> Option<&Element> {
        Some(&self.zoomed_out)
    }
}

pub fn semantic_zoom(zoomed_in: impl Into<Element>) -> SemanticZoom {
    SemanticZoom::new(zoomed_in)
}
