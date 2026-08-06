use super::*;

/// `Microsoft.UI.Xaml.Controls.ParallaxView`. Links a background element to a
/// scrolling source; the background shifts at a different rate.
#[derive(Clone, Default, Debug, PartialEq)]
pub struct ParallaxView {
    pub key: Option<String>,
    pub modifiers: Modifiers,
    pub child: Box<Element>,
    pub horizontal_shift: f64,
}
impl ParallaxView {
    pub fn new(child: impl Into<Element>) -> Self {
        Self {
            child: Box::new(child.into()),
            horizontal_shift: 100.0,
            ..Default::default()
        }
    }
    pub fn horizontal_shift(mut self, v: f64) -> Self {
        self.horizontal_shift = v;
        self
    }
}
impl Widget for ParallaxView {
    widget_header!(ControlKind::ParallaxView);
    fn children(&self) -> Children<'_> {
        Children::PositionalSingle(&self.child)
    }
    fn bindings(&self) -> PropBindings {
        Vec::new()
    }
}
pub fn parallax_view(child: impl Into<Element>) -> ParallaxView {
    ParallaxView::new(child)
}
