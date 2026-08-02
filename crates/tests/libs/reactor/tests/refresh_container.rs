use std::rc::Rc;

use test_reactor::{Op, RecordingBackend};
use windows_reactor::{
    ControlKind, Element, Event, Prop, PropValue, Reconciler, RefreshContainer,
    RefreshPullDirection, TextBlock,
};

fn mount(el: &Element) -> Reconciler<RecordingBackend> {
    let mut r = Reconciler::new(RecordingBackend::new());
    r.reconcile(None, el, None, Rc::new(|| {}));
    r
}

fn first_create(r: &Reconciler<RecordingBackend>) -> (ControlKind, windows_reactor::ControlId) {
    r.backend
        .ops
        .iter()
        .find_map(|op| match op {
            Op::Create { id, kind } => Some((*kind, *id)),
            _ => None,
        })
        .expect("no Create op")
}

#[test]
fn refresh_container_mounts_with_child_and_pull_direction() {
    let el: Element = RefreshContainer::new(TextBlock::new("list content"))
        .pull_direction(RefreshPullDirection::BottomToTop)
        .into();
    let r = mount(&el);
    let (kind, id) = first_create(&r);
    assert_eq!(kind, ControlKind::RefreshContainer);

    // PullDirection 按原始枚举值下发（BottomToTop == 3，winmd 权威值）。
    let saw_direction = r.backend.ops.iter().any(|op| {
        matches!(
            op,
            Op::SetProp {
                id: oid,
                prop: Prop::PullDirection,
                value: PropValue::I32(v),
            } if *oid == id && *v == RefreshPullDirection::BottomToTop.0
        )
    });
    assert!(saw_direction, "PullDirection not set");

    // 内容走 ContentControl 单 child 语义：append 到容器下。
    let appended = r
        .backend
        .ops
        .iter()
        .any(|op| matches!(op, Op::AppendChild { parent, .. } if *parent == id));
    assert!(appended, "child not appended");
    assert_eq!(r.backend.children_of(id).len(), 1);
}

#[test]
fn refresh_container_default_pull_direction_is_top_to_bottom() {
    let el: Element = RefreshContainer::new(TextBlock::new("x")).into();
    let r = mount(&el);
    let (_, id) = first_create(&r);
    let saw_direction = r.backend.ops.iter().any(|op| {
        matches!(
            op,
            Op::SetProp {
                id: oid,
                prop: Prop::PullDirection,
                value: PropValue::I32(v),
            } if *oid == id && *v == RefreshPullDirection::TopToBottom.0
        )
    });
    assert!(saw_direction, "default PullDirection should be TopToBottom");
}

#[test]
fn refresh_container_attaches_refresh_requested_event() {
    let el: Element = RefreshContainer::new(TextBlock::new("x"))
        .on_refresh_requested(|_req: windows_reactor::RefreshRequest| {})
        .into();
    let r = mount(&el);
    let (_, id) = first_create(&r);
    let attached = r.backend.ops.iter().any(|op| {
        matches!(
            op,
            Op::AttachEvent {
                id: oid,
                event: Event::RefreshRequested,
                ..
            } if *oid == id
        )
    });
    assert!(attached, "RefreshRequested event not attached");
}
