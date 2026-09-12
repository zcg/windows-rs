use super::*;
use std::cell::RefCell;
use std::rc::Rc;

fn swiped(content: &str) -> View {
    SwipeControl::new()
        .content(TextBlock::new().text(content))
        .into()
}

fn swipes(on_invoked: impl IntoPayloadCallback<String>) -> SwipeItems {
    SwipeItems::new(
        [SwipeItem::new("delete", "Delete")],
        [SwipeItem::new("archive", "Archive")],
        on_invoked,
    )
}

#[test]
fn swipe_control_mounts_items_and_routes_text() {
    let invoked = Rc::new(RefCell::new(Vec::new()));
    let capture = Rc::clone(&invoked);
    let mut pump = Pump::new(RecordingRuntime::default());
    pump.mount_view(
        swiped("Swipe me").swipe_items(swipes(move |text| capture.borrow_mut().push(text))),
    )
    .unwrap();

    let owner = pump.root().unwrap();
    let (attached, left, right, _) = pump.runtime().swipe_attachments(owner).unwrap();
    assert_eq!(
        *attached,
        Pump::<RecordingRuntime>::native_root(&pump.tree, pump.root().unwrap()).unwrap()
    );
    assert_eq!(left, &[SwipeItem::new("delete", "Delete")]);
    assert_eq!(right, &[SwipeItem::new("archive", "Archive")]);

    pump.runtime_mut().queue_swipe_invoked(owner, "Archive");
    assert_eq!(pump.dispatch_events(), Ok(1));
    assert_eq!(&*invoked.borrow(), &["Archive"]);
}

#[test]
fn swipe_items_update_replaces_lists() {
    let mut pump = Pump::new(RecordingRuntime::default());
    pump.mount_view(swiped("Swipe me").swipe_items(swipes(|_| {})))
        .unwrap();

    let owner = pump.root().unwrap();
    pump.update_view(swiped("Swipe me").swipe_items(SwipeItems::new(
        [SwipeItem::new("pin", "Pin")],
        [],
        |_| {},
    )))
    .unwrap();

    let (_, left, right, _) = pump.runtime().swipe_attachments(owner).unwrap();
    assert_eq!(left, &[SwipeItem::new("pin", "Pin")]);
    assert!(right.is_empty());
}

#[test]
fn swipe_items_removed_clears_attachment() {
    let mut pump = Pump::new(RecordingRuntime::default());
    pump.mount_view(swiped("Swipe me").swipe_items(swipes(|_| {})))
        .unwrap();

    let owner = pump.root().unwrap();
    pump.update_view(swiped("Swipe me")).unwrap();

    assert!(pump.runtime().swipe_attachments(owner).is_none());
}

#[test]
fn swipe_items_reject_duplicate_keys() {
    let mut pump = Pump::new(RecordingRuntime::default());
    let result = pump.mount_view(swiped("Swipe me").swipe_items(SwipeItems::new(
        [SwipeItem::new("same", "One"), SwipeItem::new("same", "Two")],
        [],
        |_| {},
    )));
    assert!(matches!(result, Err(PumpError::DuplicateKey(_))));
}
