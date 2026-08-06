use super::*;

/// `Microsoft.UI.Xaml.Controls.AnnotatedScrollBar`. A scroll bar that shows
/// value labels alongside the thumb.
#[derive(Clone, Default, Debug, PartialEq)]
pub struct AnnotatedScrollBar {
    pub key: Option<String>,
    pub modifiers: Modifiers,
    pub labels: Vec<(f64, String)>,
    pub small_change: f64,
    pub mounted: Option<Callback<Option<windows_core::IInspectable>>>,
}
impl AnnotatedScrollBar {
    pub fn new() -> Self {
        Self {
            small_change: 1.0,
            ..Default::default()
        }
    }
    pub fn labels(mut self, labels: Vec<(f64, String)>) -> Self {
        self.labels = labels;
        self.rebuild_mounted();
        self
    }
    pub fn small_change(mut self, v: f64) -> Self {
        self.small_change = v;
        self.rebuild_mounted();
        self
    }
    fn rebuild_mounted(&mut self) {
        let small_change = self.small_change;
        self.mounted = Some(Callback::new(
            move |native: Option<windows_core::IInspectable>| {
                let Some(native) = native else { return };
                if let Ok(asb) = native.cast::<bindings::AnnotatedScrollBar>() {
                    let _ = asb.SetSmallChange(small_change);
                }
            },
        ));
    }
}

impl Widget for AnnotatedScrollBar {
    widget_header!(ControlKind::AnnotatedScrollBar);
    fn bindings(&self) -> PropBindings {
        Vec::new()
    }
    fn on_mounted_callback(&self) -> Option<&Callback<Option<windows_core::IInspectable>>> {
        self.mounted.as_ref()
    }
}

pub fn annotated_scroll_bar() -> AnnotatedScrollBar {
    AnnotatedScrollBar::new()
}
