//! Live camera preview through `MediaCapture` + `MediaFrameSourceGroup`,
//! displayed on a `MediaPlayerElement` (the same approach the official WinUI
//! Gallery uses; WinUI 3 has no `CaptureElement` control).
//!
//! The async capture steps are polled with a short busy wait; preview startup
//! may take a second or two and briefly occupies the UI thread. Camera access
//! requires the Windows privacy setting to allow camera use for this app.

use std::time::Duration;

use super::*;
use windows_future::{AsyncStatus, IAsyncAction, IAsyncOperation};

fn new_media_capture() -> Result<bindings::MediaCapture> {
    static SHARED: windows_core::imp::FactoryCache<
        bindings::MediaCapture,
        windows_core::imp::IGenericFactory,
    > = windows_core::imp::FactoryCache::new();
    SHARED.call(|f| f.ActivateInstance::<bindings::MediaCapture>())
}

fn new_capture_settings() -> Result<bindings::MediaCaptureInitializationSettings> {
    static SHARED: windows_core::imp::FactoryCache<
        bindings::MediaCaptureInitializationSettings,
        windows_core::imp::IGenericFactory,
    > = windows_core::imp::FactoryCache::new();
    SHARED.call(|f| f.ActivateInstance::<bindings::MediaCaptureInitializationSettings>())
}

fn wait_action(action: IAsyncAction) -> Result<()> {
    loop {
        match action.Status()? {
            AsyncStatus::Completed | AsyncStatus::Error => return action.GetResults(),
            _ => std::thread::sleep(Duration::from_millis(10)),
        }
    }
}

fn wait_operation<T: windows_core::RuntimeType + 'static>(
    operation: IAsyncOperation<T>,
) -> Result<T> {
    loop {
        match operation.Status()? {
            AsyncStatus::Completed | AsyncStatus::Error => return operation.GetResults(),
            _ => std::thread::sleep(Duration::from_millis(10)),
        }
    }
}

/// Start the first available camera's preview and show it on the given
/// `MediaPlayerElement` (passed as its native `UIElement`). Returns a status
/// string: `Viewing: <device>` on success, or a diagnostic listing every
/// initialization combination's error on failure.
pub fn start_camera_preview_into(host: &UIElement) -> String {
    let operation = match bindings::MediaFrameSourceGroup::FindAllAsync() {
        Ok(op) => op,
        Err(error) => return format!("Camera error (FindAllAsync): {error}"),
    };
    let groups = match wait_operation(operation) {
        Ok(groups) => groups,
        Err(error) => return format!("Camera error (FindAllAsync wait): {error}"),
    };
    let group_count = match groups.Size() {
        Ok(n) => n,
        Err(error) => return format!("Camera error (Size): {error}"),
    };
    if group_count == 0 {
        return "No camera devices found.".to_string();
    }
    let group = match groups.GetAt(0) {
        Ok(g) => g,
        Err(error) => return format!("Camera error (GetAt): {error}"),
    };
    let name = match group.DisplayName() {
        Ok(n) => n,
        Err(error) => return format!("Camera error (DisplayName): {error}"),
    };
    let infos = match group.SourceInfos() {
        Ok(i) => i,
        Err(error) => return format!("Camera error (SourceInfos): {error}"),
    };
    // Prefer a color video-preview source: a camera group can expose
    // infrared/depth/metadata sources first, and using one of those as the
    // MediaSource shows a black frame.
    let source_count = match infos.Size() {
        Ok(n) => n,
        Err(error) => return format!("Camera error (SourceInfo Size): {error}"),
    };
    let mut source_id = String::new();
    let mut picked_kind = String::new();
    let mut kinds = Vec::new();
    for i in 0..source_count {
        let info = match infos.GetAt(i) {
            Ok(info) => info,
            Err(error) => return format!("Camera error (SourceInfo GetAt {i}): {error}"),
        };
        let id = match info.Id() {
            Ok(id) => id,
            Err(error) => return format!("Camera error (Id {i}): {error}"),
        };
        let kind = match info.MediaStreamType() {
            Ok(kind) => kind,
            Err(error) => return format!("Camera error (MediaStreamType {i}): {error}"),
        };
        kinds.push(format!("{i}:{}", kind.0));
        if kind == bindings::MediaStreamType::VideoPreview {
            source_id = id;
            picked_kind = format!("video-preview");
            break;
        }
    }
    if source_id.is_empty() {
        // No video-preview source; fall back to the first source.
        source_id = match infos.GetAt(0) {
            Ok(info) => match info.Id() {
                Ok(id) => id,
                Err(error) => return format!("Camera error (Id fallback): {error}"),
            },
            Err(error) => return format!("Camera error (SourceInfo fallback): {error}"),
        };
        picked_kind = "first".to_string();
    }
    let _ = picked_kind;
    let _ = kinds;

    // MediaCapture can only be initialized once per instance, so each
    // settings combination gets a fresh instance. Try the official
    // shared-read-only/Cpu combo first, then fall back to exclusive control
    // and automatic memory preference for devices/drivers that reject them.
    let combos = [
        (
            bindings::MediaCaptureSharingMode::SharedReadOnly,
            bindings::MediaCaptureMemoryPreference::Cpu,
        ),
        (
            bindings::MediaCaptureSharingMode::ExclusiveControl,
            bindings::MediaCaptureMemoryPreference::Cpu,
        ),
        (
            bindings::MediaCaptureSharingMode::ExclusiveControl,
            bindings::MediaCaptureMemoryPreference::Auto,
        ),
    ];
    let mut errors = Vec::new();
    let mut capture = None;
    for (sharing, memory) in combos {
        let candidate = match new_media_capture() {
            Ok(c) => c,
            Err(error) => {
                errors.push(format!("create:{error:?}"));
                continue;
            }
        };
        let settings = match new_capture_settings() {
            Ok(s) => s,
            Err(error) => {
                errors.push(format!("settings:{error:?}"));
                continue;
            }
        };
        let settings5 = match settings.cast::<bindings::IMediaCaptureInitializationSettings5>() {
            Ok(s) => s,
            Err(error) => {
                errors.push(format!("cast:{error:?}"));
                continue;
            }
        };
        if let Err(error) = settings5.SetSourceGroup(&group) {
            errors.push(format!("sourcegroup:{error:?}"));
            continue;
        }
        if let Err(error) = settings.SetStreamingCaptureMode(bindings::StreamingCaptureMode::Video)
        {
            errors.push(format!("mode:{error:?}"));
            continue;
        }
        if let Err(error) = settings5.SetSharingMode(sharing) {
            errors.push(format!("sharing:{error:?}"));
            continue;
        }
        if let Err(error) = settings5.SetMemoryPreference(memory) {
            errors.push(format!("memory:{error:?}"));
            continue;
        }
        let init_operation = match candidate.InitializeWithSettingsAsync(&settings) {
            Ok(op) => op,
            Err(error) => {
                errors.push(format!("init-start:{error:?}"));
                continue;
            }
        };
        match wait_action(init_operation) {
            Ok(()) => {
                capture = Some(candidate);
                break;
            }
            Err(error) => errors.push(format!("init:{error:?}")),
        }
    }
    let Some(capture) = capture else {
        let denied = errors.iter().any(|e| e.contains("80070005"));
        return if denied {
            format!(
                "Camera access denied. Enable camera access in Windows Settings > Privacy & security > Camera, then try again. (devices: {group_count}, {name})"
            )
        } else {
            format!(
                "Camera error. devices: {group_count}, {name} | {}",
                errors.join(" | ")
            )
        };
    };

    let capture5 = match capture.cast::<bindings::IMediaCapture5>() {
        Ok(c) => c,
        Err(error) => return format!("Camera error (cast5): {error}"),
    };
    let sources = match capture5.FrameSources() {
        Ok(s) => s,
        Err(error) => return format!("Camera error (FrameSources): {error}"),
    };
    let frame_source = match sources.Lookup(&windows_core::HSTRING::from(source_id)) {
        Ok(f) => f,
        Err(error) => return format!("Camera error (Lookup): {error}"),
    };
    let media_source = match bindings::MediaSource::CreateFromMediaFrameSource(&frame_source) {
        Ok(m) => m,
        Err(error) => return format!("Camera error (CreateFromMediaFrameSource): {error}"),
    };

    let element: bindings::MediaPlayerElement = match host.cast() {
        Ok(e) => e,
        Err(error) => return format!("Camera error (cast): {error}"),
    };
    match element.SetSource(&media_source) {
        Ok(()) => format!("Viewing: {name}"),
        Err(error) => format!("Camera error (SetSource): {error}"),
    }
}
