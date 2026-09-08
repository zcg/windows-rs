use crate::controls::*;
use windows::Storage::Pickers::FileOpenPicker;
use windows::Win32::shobjidl_core::IInitializeWithWindow;
use windows::Win32::winuser::GetActiveWindow;
use windows::core::{HSTRING, Interface};
use windows_reactor::*;

pub struct StoragePickersPage {
    status: String,
}

#[derive(Clone)]
pub enum Message {
    Pick,
}

fn pick_single_file() -> Result<String, windows::core::Error> {
    let window = unsafe { GetActiveWindow() };
    if window.0.is_null() {
        return Ok(String::from("No active window."));
    }
    let picker = FileOpenPicker::new()?;
    unsafe {
        picker
            .cast::<IInitializeWithWindow>()?
            .Initialize(window)
            .ok()?;
    }
    let filter = picker.FileTypeFilter()?;
    filter.Append(&HSTRING::from(".txt"))?;
    filter.Append(&HSTRING::from(".md"))?;
    match picker.PickSingleFileAsync()?.GetResults() {
        Ok(file) => Ok(format!("Picked: {}", file.Path()?.to_string_lossy())),
        Err(_) => Ok(String::from("No file selected.")),
    }
}

impl Component for StoragePickersPage {
    type Message = Message;
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self {
            status: String::from("No file selected."),
        }
    }

    fn update(&mut self, message: Message, _: &ComponentContext<Self>) {
        match message {
            Message::Pick => {
                self.status = pick_single_file().unwrap_or_else(|error| format!("Error: {error}"));
            }
        }
    }

    fn view(&self, _: &(), context: &mut ViewContext<Self>) -> View {
        page_content(
            "Storage Pickers",
            "Pick files with the system file picker.",
            [KeyedView::new(
                "single",
                sample_card(
                    "Pick a Single File",
                    StackPanel::new().spacing(8.0).children((
                        Button::new()
                            .on_click(context.message(Message::Pick))
                            .content("Pick a .txt or .md file"),
                        TextBlock::new().text(self.status.clone()).opacity(0.6),
                    )),
                    "FileOpenPicker::new()?.PickSingleFileAsync()?",
                ),
            )],
        )
    }
}
