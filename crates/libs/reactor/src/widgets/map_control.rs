use super::*;

/// `Microsoft.UI.Xaml.Controls.MapControl`. Displays a map centered on
/// `center` (latitude, longitude) with a marker, an optional map service
/// token, and interactive pan/zoom controls.
#[derive(Clone, Debug, PartialEq)]
pub struct MapControl {
    pub key: Option<String>,
    pub modifiers: Modifiers,
    pub center: (f64, f64),
    pub zoom: f64,
    pub map_service_token: Option<String>,
    pub interactive: bool,
}

impl Default for MapControl {
    fn default() -> Self {
        Self {
            key: None,
            modifiers: Modifiers::default(),
            center: (0.0, 0.0),
            zoom: 1.0,
            map_service_token: None,
            interactive: true,
        }
    }
}

impl MapControl {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn center(mut self, lat: f64, lon: f64) -> Self {
        self.center = (lat, lon);
        self
    }

    pub fn zoom_level(mut self, zoom: f64) -> Self {
        self.zoom = zoom;
        self
    }

    pub fn map_service_token(mut self, token: impl Into<String>) -> Self {
        self.map_service_token = Some(token.into());
        self
    }

    pub fn interactive(mut self, v: bool) -> Self {
        self.interactive = v;
        self
    }
}

impl Widget for MapControl {
    widget_header!(ControlKind::MapControl);
    fn bindings(&self) -> PropBindings {
        let mut out = Vec::with_capacity(4);
        out.push(Binding::Prop(
            Prop::MapCenter,
            PropValue::MapCenter(self.center.0, self.center.1),
        ));
        out.push(Binding::Prop(Prop::MapZoom, PropValue::F64(self.zoom)));
        if let Some(token) = self.map_service_token.to_prop_str() {
            out.push(Binding::Prop(Prop::MapServiceToken, token));
        }
        out.push(Binding::Prop(
            Prop::MapInteractive,
            PropValue::Bool(self.interactive),
        ));
        out
    }
}

pub fn map_control() -> MapControl {
    MapControl::new()
}
