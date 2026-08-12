use crate::controls::*;
use windows_reactor::*;

pub fn storage_pickers_page(_: &(), cx: &mut RenderCx) -> Element {
    let (single, set_single) = cx.use_state(String::from("No file selected."));
    let (multi, set_multi) = cx.use_state(String::from("No files selected."));
    let (saved, set_saved) = cx.use_state(String::from("No destination selected."));
    let (folder, set_folder) = cx.use_state(String::from("No folder selected."));

    let pick_single = move || match active_window_id() {
        Some(window_id) => {
            let set_single_done = set_single.clone();
            let start_error = if let Err(err) =
                pick_single_file_async(window_id, &[".txt", ".md"], move |result| {
                    let text = match result {
                        Ok(Some(path)) => format!("Picked: {path}"),
                        Ok(None) => "No file selected.".into(),
                        Err(err) => format!("Error: {err}"),
                    };
                    set_single_done.call(text);
                }) {
                Some(format!("Error: {err}"))
            } else {
                None
            };
            if let Some(text) = start_error {
                set_single.call(text);
            }
        }
        None => set_single.call("No active window.".into()),
    };

    let pick_multi = move || match active_window_id() {
        Some(window_id) => {
            let set_multi_done = set_multi.clone();
            let start_error = if let Err(err) =
                pick_multiple_files_async(window_id, &[], move |result| {
                    let text = match result {
                        Ok(paths) if paths.is_empty() => "No files selected.".to_string(),
                        Ok(paths) => {
                            let list = paths
                                .iter()
                                .map(|p| format!("- {p}"))
                                .collect::<Vec<_>>()
                                .join("\n");
                            format!("Picked {} file(s):\n{list}", paths.len())
                        }
                        Err(err) => format!("Error: {err}"),
                    };
                    set_multi_done.call(text);
                }) {
                Some(format!("Error: {err}"))
            } else {
                None
            };
            if let Some(text) = start_error {
                set_multi.call(text);
            }
        }
        None => set_multi.call("No active window.".into()),
    };

    let pick_save = move || match active_window_id() {
        Some(window_id) => {
            let set_saved_done = set_saved.clone();
            let start_error = if let Err(err) = pick_save_file_async(window_id, move |result| {
                let text = match result {
                    Ok(Some(path)) => format!("Saving to: {path}"),
                    Ok(None) => "No destination selected.".into(),
                    Err(err) => format!("Error: {err}"),
                };
                set_saved_done.call(text);
            }) {
                Some(format!("Error: {err}"))
            } else {
                None
            };
            if let Some(text) = start_error {
                set_saved.call(text);
            }
        }
        None => set_saved.call("No active window.".into()),
    };

    let pick_folder = move || match active_window_id() {
        Some(window_id) => {
            let set_folder_done = set_folder.clone();
            let start_error = if let Err(err) = pick_folder_async(window_id, move |result| {
                let text = match result {
                    Ok(Some(path)) => format!("Picked: {path}"),
                    Ok(None) => "No folder selected.".into(),
                    Err(err) => format!("Error: {err}"),
                };
                set_folder_done.call(text);
            }) {
                Some(format!("Error: {err}"))
            } else {
                None
            };
            if let Some(text) = start_error {
                set_folder.call(text);
            }
        }
        None => set_folder.call("No active window.".into()),
    };

    page_content(
        "Storage Pickers",
        "Open, save, and folder pickers bound to the current AppWindow.",
        vec![
            sample_card(
                "Pick a Single File",
                vstack((
                    button("Pick a file").on_click(pick_single),
                    text_block(single).opacity(0.6).font_size(13.0),
                ))
                .spacing(8.0),
                r#"FileOpenPicker::CreateInstance(window_id)?;
picker.FileTypeFilter()?.Append(&HSTRING::from(".txt"))?;
picker.PickSingleFileAsync()? // poll Status() then GetResults()"#,
            ),
            sample_card(
                "Pick Multiple Files",
                vstack((
                    button("Pick files").on_click(pick_multi),
                    text_block(multi).opacity(0.6).font_size(13.0),
                ))
                .spacing(8.0),
                r#"let files = picker.PickMultipleFilesAsync()?.GetResults()?;
for i in 0..files.Size()? { files.GetAt(i)?.Path()? }"#,
            ),
            sample_card(
                "Save a File",
                vstack((
                    button("Pick a save destination").on_click(pick_save),
                    text_block(saved).opacity(0.6).font_size(13.0),
                ))
                .spacing(8.0),
                r#"let picker = FileSavePicker::CreateInstance(window_id)?;
picker.SetDefaultFileExtension(".txt")?;
picker.SetSuggestedFileName("gallery.txt")?;
picker.PickSaveFileAsync()?"#,
            ),
            sample_card(
                "Pick a Folder",
                vstack((
                    button("Pick a folder").on_click(pick_folder),
                    text_block(folder).opacity(0.6).font_size(13.0),
                ))
                .spacing(8.0),
                r#"let picker = FolderPicker::CreateInstance(window_id)?;
picker.PickSingleFolderAsync()? // -> PickFolderResult::Path()"#,
            ),
        ],
    )
}
