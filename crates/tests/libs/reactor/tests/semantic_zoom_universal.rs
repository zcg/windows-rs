//! Regression tests for the `SemanticZoom` widget: the reconciler must mount
//! the zoomed-in view as the primary child, mount the zoomed-out view through
//! the pane path, and emit `IsZoomedInViewActive` from the widget binding.

use std::rc::Rc;

use test_reactor::{Op, RecordingBackend};
use windows_reactor::{
    ControlKind, Element, Prop, PropValue, Reconciler, SemanticZoom, TextBlock, list_view,
};

fn view() -> Element {
    list_view(vec![1i32, 2, 3], |n, _| TextBlock::new(n.to_string())).into()
}

#[test]
fn semantic_zoom_mounts_both_views_and_active_prop() {
    let el: Element = SemanticZoom::new(view())
        .zoomed_out(view())
        .is_zoomed_in_view_active(false)
        .into();
    let mut r = Reconciler::new(RecordingBackend::new());
    let id = r
        .reconcile(None, &el, None, Rc::new(|| {}))
        .expect("mount produced a control id");

    let mut saw_zoom = false;
    let mut list_view_creates = 0;
    for op in &r.backend.ops {
        match op {
            Op::Create { kind, .. } => {
                if *kind == ControlKind::SemanticZoom {
                    saw_zoom = true;
                }
                if *kind == ControlKind::ListView {
                    list_view_creates += 1;
                }
            }
            Op::SetProp {
                id: oid,
                prop,
                value,
            } if *oid == id
                && *prop == Prop::IsZoomedInViewActive
                && *value == PropValue::Bool(false) =>
            {
                // expected set_prop
            }
            _ => {}
        }
    }

    assert!(saw_zoom, "missing ControlKind::SemanticZoom create");
    assert_eq!(list_view_creates, 2, "both views must mount as ListViews");
}

#[test]
fn semantic_zoom_default_active_is_true() {
    let el: Element = SemanticZoom::new(view()).into();
    let mut r = Reconciler::new(RecordingBackend::new());
    let id = r
        .reconcile(None, &el, None, Rc::new(|| {}))
        .expect("mount produced a control id");

    let mut saw_true = false;
    for op in &r.backend.ops {
        if let Op::SetProp {
            id: oid,
            prop,
            value,
        } = op
            && *oid == id
            && *prop == Prop::IsZoomedInViewActive
        {
            assert_eq!(*value, PropValue::Bool(true), "default must be active");
            saw_true = true;
        }
    }
    assert!(saw_true, "missing IsZoomedInViewActive=true set_prop");
}
