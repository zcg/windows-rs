//! Regression tests for the `ItemsView` widget: the reconciler must create
//! the control and emit the items source, layout, and selection-mode props
//! from the widget bindings.

use std::rc::Rc;

use test_reactor::{Op, RecordingBackend};
use windows_reactor::{
    ControlKind, Element, ItemsViewLayout, ItemsViewSelectionMode, Prop, PropValue, Reconciler,
    items_view,
};

#[test]
fn items_view_mounts_with_source_layout_and_selection() {
    let el: Element = items_view(["A", "B", "C"])
        .layout(ItemsViewLayout::Grid)
        .selection_mode(ItemsViewSelectionMode::Multiple)
        .into();
    let mut r = Reconciler::new(RecordingBackend::new());
    let id = r
        .reconcile(None, &el, None, Rc::new(|| {}))
        .expect("mount produced a control id");

    let mut saw_create = false;
    let mut saw_source = false;
    let mut saw_layout = false;
    let mut saw_selection = false;
    for op in &r.backend.ops {
        match op {
            Op::Create { kind, .. } => {
                if *kind == ControlKind::ItemsView {
                    saw_create = true;
                }
            }
            Op::SetProp {
                id: oid,
                prop,
                value,
            } if *oid == id => match (prop, value) {
                (Prop::ItemsViewSource, PropValue::StrList(items))
                    if items == &["A".to_string(), "B".to_string(), "C".to_string()] =>
                {
                    saw_source = true;
                }
                (Prop::ItemsViewLayout, PropValue::I32(1)) => saw_layout = true,
                (Prop::SelectionMode, PropValue::I32(2)) => saw_selection = true,
                _ => {}
            },
            _ => {}
        }
    }

    assert!(saw_create, "missing ControlKind::ItemsView create");
    assert!(saw_source, "missing ItemsViewSource prop");
    assert!(saw_layout, "missing ItemsViewLayout=1 (grid)");
    assert!(saw_selection, "missing SelectionMode=2 (multiple)");
}

#[test]
fn items_view_defaults_to_stack_and_single() {
    let el: Element = items_view(["A", "B"]).into();
    let mut r = Reconciler::new(RecordingBackend::new());
    let id = r
        .reconcile(None, &el, None, Rc::new(|| {}))
        .expect("mount produced a control id");

    let mut saw_layout = false;
    let mut saw_selection = false;
    for op in &r.backend.ops {
        if let Op::SetProp {
            id: oid,
            prop,
            value,
        } = op
        {
            if *oid != id {
                continue;
            }
            match (prop, value) {
                (Prop::ItemsViewLayout, PropValue::I32(0)) => saw_layout = true,
                (Prop::SelectionMode, PropValue::I32(1)) => saw_selection = true,
                _ => {}
            }
        }
    }
    assert!(saw_layout, "missing ItemsViewLayout=0 (stack)");
    assert!(saw_selection, "missing SelectionMode=1 (single)");
}
