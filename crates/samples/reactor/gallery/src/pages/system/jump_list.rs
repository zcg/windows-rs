use crate::controls::*;
use windows::UI::StartScreen::{JumpList, JumpListItem};
use windows_future::{AsyncStatus, IAsyncAction, IAsyncOperation};
use windows_reactor::*;

/// Blocks the calling thread until the async operation completes. Must run
/// off the UI thread: `GetResults` does not wait, so completion is polled.
fn wait_results<T>(operation: &IAsyncOperation<T>) -> windows::core::Result<()>
where
    T: windows::core::RuntimeType,
{
    for _ in 0..200 {
        match operation.Status()? {
            AsyncStatus::Completed => return Ok(()),
            AsyncStatus::Error => {
                let code = operation.ErrorCode().unwrap_or_default();
                return Err(code.into());
            }
            AsyncStatus::Canceled => {
                return Err(windows::core::Error::empty());
            }
            _ => std::thread::sleep(std::time::Duration::from_millis(50)),
        }
    }
    Err(windows::core::Error::empty())
}

fn wait_action(operation: &IAsyncAction) -> windows::core::Result<()> {
    for _ in 0..200 {
        match operation.Status()? {
            AsyncStatus::Completed => return Ok(()),
            AsyncStatus::Error => {
                let code = operation.ErrorCode().unwrap_or_default();
                return Err(code.into());
            }
            AsyncStatus::Canceled => {
                return Err(windows::core::Error::empty());
            }
            _ => std::thread::sleep(std::time::Duration::from_millis(50)),
        }
    }
    Err(windows::core::Error::empty())
}

pub struct JumpListPage {
    status: String,
    busy: bool,
}

#[derive(Clone)]
pub enum Message {
    Add,
    Clear,
    Done(String),
}

fn add_tasks() -> Result<String, windows::core::Error> {
    let load = JumpList::LoadCurrentAsync()?;
    wait_results(&load)?;
    let list = load.GetResults()?;
    let items = list.Items()?;
    items.Append(&JumpListItem::CreateWithArguments(
        &windows::core::HSTRING::from("/compose"),
        &windows::core::HSTRING::from("New Message"),
    )?)?;
    items.Append(&JumpListItem::CreateWithArguments(
        &windows::core::HSTRING::from("/calendar"),
        &windows::core::HSTRING::from("Calendar"),
    )?)?;
    let save = list.SaveAsync()?;
    wait_action(&save)?;
    save.GetResults()?;
    Ok(String::from("Jump list tasks added."))
}

fn clear_tasks() -> Result<String, windows::core::Error> {
    let load = JumpList::LoadCurrentAsync()?;
    wait_results(&load)?;
    let list = load.GetResults()?;
    list.Items()?.Clear()?;
    let save = list.SaveAsync()?;
    wait_action(&save)?;
    save.GetResults()?;
    Ok(String::from("Jump list cleared."))
}

impl Component for JumpListPage {
    type Message = Message;
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self {
            status: String::from(
                "Add tasks to the taskbar jump list. Packaged apps show them on right-click.",
            ),
            busy: false,
        }
    }

    fn update(&mut self, message: Message, context: &ComponentContext<Self>) {
        match message {
            Message::Add => {
                self.busy = true;
                self.status = String::from("Updating jump list...");
                context.spawn_background(|_| {
                    Message::Done(
                        add_tasks()
                            .unwrap_or_else(|error| format!("Cannot update jump list: {error}")),
                    )
                });
            }
            Message::Clear => {
                self.busy = true;
                self.status = String::from("Updating jump list...");
                context.spawn_background(|_| {
                    Message::Done(
                        clear_tasks()
                            .unwrap_or_else(|error| format!("Cannot update jump list: {error}")),
                    )
                });
            }
            Message::Done(status) => {
                self.busy = false;
                self.status = status;
            }
        }
    }

    fn view(&self, _: &(), context: &mut ViewContext<Self>) -> View {
        let busy = self.busy;
        page_content(
            "Jump List",
            "Add tasks and custom groups to the taskbar jump list.",
            [KeyedView::new(
                "tasks",
                sample_card(
                    "Jump List Tasks",
                    StackPanel::new().spacing(8.0).children((
                        StackPanel::new()
                            .orientation(Orientation::Horizontal)
                            .spacing(8.0)
                            .children((
                                Button::new()
                                    .on_click(context.message(Message::Add))
                                    .is_enabled(!busy)
                                    .content("Add tasks"),
                                Button::new()
                                    .on_click(context.message(Message::Clear))
                                    .is_enabled(!busy)
                                    .content("Clear tasks"),
                            )),
                        TextBlock::new().text(self.status.clone()).opacity(0.6),
                    )),
                ),
            )],
        )
    }
}
