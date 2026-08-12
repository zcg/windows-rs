use crate::controls::*;
use windows_reactor::*;

pub fn sound_page(_: &(), cx: &mut RenderCx) -> Element {
    let (enabled, set_enabled) = cx.use_state(false);
    let (spatial, set_spatial) = cx.use_state(false);

    let sound_button = |kind: ElementSoundKind, label: &'static str| {
        button(label).on_click(move || {
            let _ = play_element_sound(kind);
        })
    };

    page_content(
        "Sound",
        "Play WinUI element sounds for control interactions.",
        vec![
            sample_card(
                "Play Control Sounds",
                vstack((
                    text_block("Plays the WinUI control sound for the selected interaction (requires the element sound player to be enabled).")
                        .font_size(12.0)
                        .opacity(0.6),
                    hstack((
                        sound_button(ElementSoundKind::Invoke, "Invoke"),
                        sound_button(ElementSoundKind::Show, "Show"),
                        sound_button(ElementSoundKind::Hide, "Hide"),
                        sound_button(ElementSoundKind::MoveNext, "MoveNext"),
                        sound_button(ElementSoundKind::GoBack, "GoBack"),
                    ))
                    .spacing(8.0),
                    ToggleSwitch::new(enabled)
                        .header("Element sound player")
                        .on_toggled({
                            let set_enabled = set_enabled.clone();
                            move |v| {
                                let _ = set_element_sound_state(v);
                                set_enabled.call(v);
                            }
                        }),
                    CheckBox::new(spatial)
                        .content("Spatial audio")
                        .enabled(enabled)
                        .on_checked({
                            let set_spatial = set_spatial.clone();
                            move |v| {
                                let _ = set_element_spatial_audio(v);
                                set_spatial.call(v);
                            }
                        }),
                    text_block(if enabled {
                        "Sounds enabled."
                    } else {
                        "Enable the player above, then play a sound."
                    })
                    .opacity(0.6),
                ))
                .spacing(8.0),
                r#"ElementSoundPlayer::Play(ElementSoundKind::Invoke)?;
ElementSoundPlayer::SetState(ElementSoundPlayerState::On)?;"#,
            ),
        ],
    )
}
