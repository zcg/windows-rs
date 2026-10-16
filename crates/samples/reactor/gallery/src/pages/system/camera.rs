use crate::controls::*;
use std::cell::RefCell;
use std::rc::Rc;
use windows::Media::Capture::Frames::MediaFrameSourceGroup;
use windows::Media::Capture::{
    MediaCapture, MediaCaptureInitializationSettings, MediaCaptureMemoryPreference,
    MediaCaptureSharingMode, StreamingCaptureMode,
};
use windows::Media::MediaProperties::ImageEncodingProperties;
use windows_core::HSTRING;
use windows_reactor::*;

pub struct CameraPage {
    /// Kept alive for the lifetime of the page: capture stops when the
    /// `MediaCapture` is released.
    capture: Rc<RefCell<Option<MediaCapture>>>,
    viewing: String,
    status: String,
    snapshot: Option<String>,
}

#[derive(Clone)]
pub enum Message {
    Ready(String),
    Failed(String),
    Capture,
}

/// Starts a shared camera capture session and returns the capture object and
/// the device name.
///
/// This mirrors the WinUI 3 Gallery camera page: a `MediaCapture` initialized in
/// shared-readonly video mode. Camera access follows the packaged app privacy
/// settings; the first use raises the per-app desktop prompt on Windows 11 builds
/// that have it.
fn start_capture() -> windows::core::Result<(MediaCapture, String)> {
    let groups = MediaFrameSourceGroup::FindAllAsync()?.join()?;
    if groups.Size()? == 0 {
        return Err(windows::core::Error::new(
            windows::core::HRESULT(0x80070490u32 as i32),
            "No camera devices found.",
        ));
    }

    let group = groups.GetAt(0)?;
    let name = group.DisplayName()?.to_string_lossy();

    let capture = MediaCapture::new()?;
    let settings = MediaCaptureInitializationSettings::new()?;
    settings.SetSourceGroup(&group)?;
    settings.SetSharingMode(MediaCaptureSharingMode::SharedReadOnly)?;
    settings.SetStreamingCaptureMode(StreamingCaptureMode::Video)?;
    settings.SetMemoryPreference(MediaCaptureMemoryPreference::Cpu)?;
    capture.InitializeWithSettingsAsync(&settings)?.join()?;

    Ok((capture, name))
}

/// Grabs one frame as a JPEG into the process temp directory and returns its path.
fn capture_photo(capture: &MediaCapture) -> windows::core::Result<String> {
    let path = std::env::temp_dir().join("gallery-camera-snapshot.jpg");
    std::fs::File::create(&path)?;
    let file = windows::Storage::StorageFile::GetFileFromPathAsync(&HSTRING::from(
        path.to_string_lossy().as_ref(),
    ))?
    .join()?;

    let format = ImageEncodingProperties::CreateJpeg()?;
    capture
        .CapturePhotoToStorageFileAsync(&format, &file)?
        .join()?;
    Ok(path.to_string_lossy().into_owned())
}

impl Component for CameraPage {
    type Message = Message;
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self {
            capture: Rc::new(RefCell::new(None)),
            viewing: String::from("Starting camera..."),
            status: String::from("Ready."),
            snapshot: None,
        }
    }

    fn update(&mut self, message: Message, _: &ComponentContext<Self>) {
        match message {
            Message::Ready(name) => self.viewing = format!("Viewing: {name}"),
            Message::Failed(error) => self.status = format!("Preview failed: {error}"),
            Message::Capture => {
                let guard = self.capture.borrow();
                let Some(capture) = guard.as_ref() else {
                    self.status = String::from("Camera is not running.");
                    return;
                };
                match capture_photo(capture) {
                    Ok(path) => {
                        self.status = format!("Captured: {path}");
                        self.snapshot = Some(path);
                    }
                    Err(error) => self.status = format!("Capture failed: {error}"),
                }
            }
        }
    }

    fn view(&self, _: &(), context: &mut ViewContext<Self>) -> View {
        let slot = Rc::clone(&self.capture);
        let sender = context.sender();
        context.use_effect("camera-preview", (), move || match start_capture() {
            Ok((capture, name)) => {
                *slot.borrow_mut() = Some(capture);
                _ = sender.send(Message::Ready(name));
                let slot = Rc::clone(&slot);
                Some(Box::new(move || {
                    *slot.borrow_mut() = None;
                }))
            }
            Err(error) => {
                _ = sender.send(Message::Failed(error.to_string()));
                None
            }
        });

        let snapshot: View = match self
            .snapshot
            .as_ref()
            .and_then(|path| Image::new().source_file(path).ok())
        {
            Some(image) => image.stretch(Stretch::Uniform).height(220.0).into(),
            None => TextBlock::new()
                .text("No snapshot yet.")
                .opacity(0.6)
                .into(),
        };

        page_content(
            "Camera",
            "Capture a photo from the camera with MediaCapture.",
            [
                KeyedView::new(
                    "preview",
                    sample_card(
                        "Capture Element / Camera",
                        StackPanel::new().spacing(8.0).children((
                            TextBlock::new().text(self.viewing.clone()),
                            StackPanel::new()
                                .orientation(Orientation::Horizontal)
                                .spacing(8.0)
                                .children((
                                    Button::new()
                                        .on_click(context.message(Message::Capture))
                                        .content("Capture photo"),
                                    TextBlock::new().text(self.status.clone()).opacity(0.6),
                                )),
                        )),
                    ),
                ),
                KeyedView::new("snapshot", sample_card("Snapshot", snapshot)),
                KeyedView::new(
                    "identity",
                    sample_card(
                        "Packaged Identity",
                        InfoBar::new()
                            .severity(InfoBarSeverity::Informational)
                            .title("Camera access follows the package identity")
                            .message(
                                "A packaged app resolves MediaCapture through the package \
                                 graph, so install the MSIX and launch from the Start Menu. \
                                 Windows 11 may raise a per-app camera prompt the first time; \
                                 the choice is remembered under Settings > Privacy & \
                                 security > Camera.",
                            )
                            .is_open(true),
                    ),
                ),
            ],
        )
    }
}
