use crate::controls::*;
use windows_reactor::*;

pub fn capture_element_preview_page(_: &(), cx: &mut RenderCx) -> Element {
    let (status, set_status) = cx.use_state(String::from(
        "Click \"Start preview\" to show the first camera in the MediaPlayerElement.",
    ));
    let player = cx.use_ref(None::<UIElement>);

    let player_for_mount = player.clone();
    let preview = media_player_element("")
        .transport_controls(false)
        .auto_play(true)
        .height(300.0)
        .width(420.0)
        .on_mounted(move |native: Option<windows_core::IInspectable>| {
            if let Some(native) = native
                && let Ok(ui) = native.cast::<UIElement>()
            {
                player_for_mount.set(Some(ui));
            }
        });

    let start = move || {
        if let Some(ui) = player.borrow().as_ref() {
            set_status.call(start_camera_preview_into(ui));
        }
    };

    page_content(
        "Capture Element Preview",
        "Preview a live camera capture in a MediaPlayerElement.",
        vec![sample_card(
            "Live Camera Preview",
            vstack((
                preview,
                hstack((
                    button("Start preview").on_click(start),
                    text_block(status).opacity(0.6),
                ))
                .spacing(8.0),
                text_block(
                    "Uses MediaFrameSourceGroup + MediaCapture + MediaSource.CreateFromMediaFrameSource, the same approach as the official WinUI Gallery (WinUI 3 has no CaptureElement control). Camera access may require the Windows privacy setting.",
                )
                .font_size(12.0)
                .opacity(0.6),
            ))
            .spacing(8.0),
            r#"let groups = MediaFrameSourceGroup::FindAllAsync()?.GetResults()?;
let capture = MediaCapture::new()?;
capture.InitializeAsync()?.GetResults()?;
let source = MediaSource::CreateFromMediaFrameSource(
    capture.FrameSources().Lookup(source_id)?)?;
player.SetSource(&source)?;"#,
        )],
    )
}
