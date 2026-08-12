//! Element sound playback through `Microsoft.UI.Xaml.ElementSoundPlayer`.

use super::*;

/// Play a control sound kind.
pub fn play_element_sound(kind: ElementSoundKind) -> Result<()> {
    bindings::ElementSoundPlayer::Play(kind)
}

/// Turn control sounds on or off.
pub fn set_element_sound_state(on: bool) -> Result<()> {
    bindings::ElementSoundPlayer::SetState(if on {
        ElementSoundPlayerState::On
    } else {
        ElementSoundPlayerState::Off
    })
}

/// Toggle spatial audio mode for control sounds.
pub fn set_element_spatial_audio(on: bool) -> Result<()> {
    bindings::ElementSoundPlayer::SetSpatialAudioMode(if on {
        ElementSpatialAudioMode::On
    } else {
        ElementSpatialAudioMode::Off
    })
}
