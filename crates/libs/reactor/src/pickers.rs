//! WinAppSDK storage pickers: file open / save and folder pickers, bound to
//! the active window via its `AppWindowId`.
//!
//! Pickers are asynchronous system dialogs. The helpers start the operation
//! and poll its status on a `DispatcherQueueTimer`, then invoke `on_done` on
//! the UI thread with the selected path (or `None` when cancelled). The UI
//! thread never blocks, so the in-process picker dialog keeps pumping.

use std::cell::RefCell;

use super::*;
use windows_future::AsyncStatus;

thread_local! {
    /// Holds the active picker timer + event revoker. Replacing the slot
    /// drops the revoker, which removes the previous tick handler.
    static PICKER_TIMER: RefCell<Option<(bindings::DispatcherQueueTimer, windows_core::EventRevoker)>> =
        const { RefCell::new(None) };
}

/// The `AppWindowId` of the first window opened on the current UI thread.
pub fn active_window_id() -> Option<WindowId> {
    with_active_host(|host| {
        let window = host.window().clone();
        let app_window = window.cast::<bindings::IWindow2>().ok()?.AppWindow().ok()?;
        app_window.Id().ok()
    })
    .flatten()
}

/// True when `error` is the empty sentinel WinRT returns for a cancelled
/// picker operation (a null result), as opposed to a real failure.
fn is_cancelled(error: &Error) -> bool {
    error.code() == windows_core::HRESULT(0)
}

/// Start a modal picker and call `on_done` on the UI thread when it closes.
/// `on_done` receives `Ok(Some(path))` on success, `Ok(None)` on cancel, or
/// `Err` on failure.
fn run_picker(
    operation: windows_future::IAsyncOperation<bindings::PickFileResult>,
    on_done: impl Fn(Result<Option<String>>) + 'static,
) -> Result<()> {
    let queue = bindings::DispatcherQueue::GetForCurrentThread()?;
    let timer = queue.CreateTimer()?;
    timer.SetInterval(TimeSpan { duration: 50_000 })?;
    timer.SetIsRepeating(true)?;

    let timer_for_tick = timer.clone();
    let revoker = timer.Tick(move |_, _| {
        let status = operation.Status();
        if status != Ok(AsyncStatus::Completed)
            && status != Ok(AsyncStatus::Canceled)
            && status != Ok(AsyncStatus::Error)
        {
            return;
        }
        timer_for_tick.Stop().ok();
        let result = match operation.GetResults() {
            Ok(file) => match file.Path() {
                Ok(path) => Ok(Some(path)),
                Err(error) => Err(error),
            },
            Err(error) if is_cancelled(&error) => Ok(None),
            Err(error) => Err(error),
        };
        PICKER_TIMER.with(|slot| *slot.borrow_mut() = None);
        on_done(result);
    })?;

    PICKER_TIMER.with(|slot| *slot.borrow_mut() = Some((timer.clone(), revoker)));
    timer.Start()
}

/// Pick a single file. `extensions` lists the allowed extensions, e.g.
/// `[".txt", ".md"]`; pass an empty slice for `*` (any file).
pub fn pick_single_file_async(
    window_id: WindowId,
    extensions: &[&str],
    on_done: impl Fn(Result<Option<String>>) + 'static,
) -> Result<()> {
    let picker = bindings::FileOpenPicker::CreateInstance(window_id)?;
    let filter = picker.FileTypeFilter()?;
    if extensions.is_empty() {
        filter.Append(&windows_core::HSTRING::from("*"))?;
    } else {
        for ext in extensions {
            filter.Append(&windows_core::HSTRING::from(*ext))?;
        }
    }
    run_picker(picker.PickSingleFileAsync()?, on_done)
}

/// Pick multiple files. `on_done` receives the selected paths (empty when
/// cancelled).
pub fn pick_multiple_files_async(
    window_id: WindowId,
    extensions: &[&str],
    on_done: impl Fn(Result<Vec<String>>) + 'static,
) -> Result<()> {
    let picker = bindings::FileOpenPicker::CreateInstance(window_id)?;
    let filter = picker.FileTypeFilter()?;
    if extensions.is_empty() {
        filter.Append(&windows_core::HSTRING::from("*"))?;
    } else {
        for ext in extensions {
            filter.Append(&windows_core::HSTRING::from(*ext))?;
        }
    }
    let operation = picker.PickMultipleFilesAsync()?;
    let queue = bindings::DispatcherQueue::GetForCurrentThread()?;
    let timer = queue.CreateTimer()?;
    timer.SetInterval(TimeSpan { duration: 50_000 })?;
    timer.SetIsRepeating(true)?;

    let timer_for_tick = timer.clone();
    let revoker = timer.Tick(move |_, _| {
        if operation.Status() != Ok(AsyncStatus::Completed) {
            return;
        }
        timer_for_tick.Stop().ok();
        let collect = (|| -> Result<Vec<String>> {
            let files = operation.GetResults()?;
            let mut paths = Vec::new();
            for i in 0..files.Size()? {
                paths.push(files.GetAt(i)?.Path()?);
            }
            Ok(paths)
        })();
        let result = match collect {
            Ok(paths) => Ok(paths),
            Err(error) if is_cancelled(&error) => Ok(Vec::new()),
            Err(error) => Err(error),
        };
        PICKER_TIMER.with(|slot| *slot.borrow_mut() = None);
        on_done(result);
    })?;

    PICKER_TIMER.with(|slot| *slot.borrow_mut() = Some((timer.clone(), revoker)));
    timer.Start()
}

/// Pick a destination for saving a file.
pub fn pick_save_file_async(
    window_id: WindowId,
    on_done: impl Fn(Result<Option<String>>) + 'static,
) -> Result<()> {
    let picker = bindings::FileSavePicker::CreateInstance(window_id)?;
    picker.SetDefaultFileExtension(".txt")?;
    picker.SetSuggestedFileName("gallery.txt")?;
    run_picker(picker.PickSaveFileAsync()?, on_done)
}

/// Pick a folder. `on_done` receives the chosen path, or `None` on cancel.
pub fn pick_folder_async(
    window_id: WindowId,
    on_done: impl Fn(Result<Option<String>>) + 'static,
) -> Result<()> {
    let picker = bindings::FolderPicker::CreateInstance(window_id)?;
    let operation = picker.PickSingleFolderAsync()?;
    let queue = bindings::DispatcherQueue::GetForCurrentThread()?;
    let timer = queue.CreateTimer()?;
    timer.SetInterval(TimeSpan { duration: 50_000 })?;
    timer.SetIsRepeating(true)?;

    let timer_for_tick = timer.clone();
    let revoker = timer.Tick(move |_, _| {
        if operation.Status() != Ok(AsyncStatus::Completed) {
            return;
        }
        timer_for_tick.Stop().ok();
        let result = match operation.GetResults() {
            Ok(folder) => match folder.Path() {
                Ok(path) => Ok(Some(path)),
                Err(error) => Err(error),
            },
            Err(error) if is_cancelled(&error) => Ok(None),
            Err(error) => Err(error),
        };
        PICKER_TIMER.with(|slot| *slot.borrow_mut() = None);
        on_done(result);
    })?;

    PICKER_TIMER.with(|slot| *slot.borrow_mut() = Some((timer.clone(), revoker)));
    timer.Start()
}
