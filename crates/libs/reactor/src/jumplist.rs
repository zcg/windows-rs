//! Jump list (taskbar) customization through `Windows.UI.StartScreen.JumpList`.
//! Unpackaged processes have no app identity, so loading the jump list fails
//! there; the helpers surface that as an error the page displays.

use std::cell::RefCell;

use super::*;
use windows_future::{AsyncStatus, IAsyncAction};

thread_local! {
    static JUMPLIST_TIMER: RefCell<Option<(bindings::DispatcherQueueTimer, windows_core::EventRevoker)>> =
        const { RefCell::new(None) };
}

fn start_timer() -> Result<bindings::DispatcherQueueTimer> {
    let queue = bindings::DispatcherQueue::GetForCurrentThread()?;
    let timer = queue.CreateTimer()?;
    timer.SetInterval(TimeSpan { duration: 50_000 })?;
    timer.SetIsRepeating(true)?;
    Ok(timer)
}

/// Add two example tasks to the jump list and call `on_done` on the UI thread.
pub fn jumplist_add_tasks(on_done: impl Fn(Result<String>) + 'static) -> Result<()> {
    let operation = bindings::JumpList::LoadCurrentAsync()?;
    let timer = start_timer()?;
    let timer_for_tick = timer.clone();

    let revoker = timer.Tick(move |_, _| {
        if operation.Status() != Ok(AsyncStatus::Completed) {
            return;
        }
        timer_for_tick.Stop().ok();
        JUMPLIST_TIMER.with(|slot| *slot.borrow_mut() = None);
        let outcome = (|| -> Result<String> {
            let jump_list = operation.GetResults()?;
            let items = jump_list.Items()?;
            let compose = bindings::JumpListItem::CreateWithArguments("/compose", "New Message")?;
            compose.SetDescription("Compose a new message")?;
            let search = bindings::JumpListItem::CreateWithArguments("/search", "Search")?;
            search.SetDescription("Search for items")?;
            items.Append(&compose)?;
            items.Append(&search)?;
            let save = jump_list.SaveAsync()?;
            wait_action(save)?;
            Ok("Jump list tasks added.".to_string())
        })();
        on_done(outcome);
    })?;

    JUMPLIST_TIMER.with(|slot| *slot.borrow_mut() = Some((timer.clone(), revoker)));
    timer.Start()
}

/// Clear all items from the jump list and call `on_done` on the UI thread.
pub fn jumplist_clear(on_done: impl Fn(Result<String>) + 'static) -> Result<()> {
    let operation = bindings::JumpList::LoadCurrentAsync()?;
    let timer = start_timer()?;
    let timer_for_tick = timer.clone();

    let revoker = timer.Tick(move |_, _| {
        if operation.Status() != Ok(AsyncStatus::Completed) {
            return;
        }
        timer_for_tick.Stop().ok();
        JUMPLIST_TIMER.with(|slot| *slot.borrow_mut() = None);
        let outcome = (|| -> Result<String> {
            let jump_list = operation.GetResults()?;
            jump_list.Items()?.Clear()?;
            let save = jump_list.SaveAsync()?;
            wait_action(save)?;
            Ok("Jump list cleared.".to_string())
        })();
        on_done(outcome);
    })?;

    JUMPLIST_TIMER.with(|slot| *slot.borrow_mut() = Some((timer.clone(), revoker)));
    timer.Start()
}

/// Blocking wait for an `IAsyncAction`, called only after the jump list
/// operation has already completed on the UI thread.
fn wait_action(action: IAsyncAction) -> Result<()> {
    loop {
        match action.Status()? {
            AsyncStatus::Completed => return action.GetResults(),
            AsyncStatus::Error => return action.GetResults(),
            _ => std::thread::sleep(std::time::Duration::from_millis(10)),
        }
    }
}
