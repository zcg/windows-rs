use super::*;

/// `Microsoft.UI.Xaml.Controls.MediaPlayerElement`. Hosts a MediaPlayer for
/// audio/video playback with optional transport controls.
#[derive(Clone, Default, Debug, PartialEq)]
pub struct MediaPlayerElement {
    pub key: Option<String>,
    pub modifiers: Modifiers,
    pub source: Option<String>,
    pub are_transport_controls_enabled: bool,
    pub auto_play: bool,
}
impl MediaPlayerElement {
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: Some(source.into()),
            are_transport_controls_enabled: true,
            auto_play: false,
            ..Default::default()
        }
    }
    pub fn transport_controls(mut self, v: bool) -> Self {
        self.are_transport_controls_enabled = v;
        self
    }
    pub fn auto_play(mut self, v: bool) -> Self {
        self.auto_play = v;
        self
    }
}
impl Widget for MediaPlayerElement {
    widget_header!(ControlKind::MediaPlayerElement);
    fn bindings(&self) -> PropBindings {
        generated::media_player_element_bindings(self)
    }
}
pub fn media_player_element(source: impl Into<String>) -> MediaPlayerElement {
    MediaPlayerElement::new(source)
}
