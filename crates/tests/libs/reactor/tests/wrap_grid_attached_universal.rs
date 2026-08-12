//! Regression tests for the VariableSizedWrapGrid attached properties
//! `wrap_grid_row_span` / `wrap_grid_column_span`: the reconciler must emit
//! `AttachedWrapGridRowSpan` / `AttachedWrapGridColumnSpan` set_prop ops on
//! mount (when the span is greater than one) and on every diff that changes
//! the spans, including removal back to the default of one.

use std::rc::Rc;

use test_reactor::{Op, RecordingBackend};
use windows_reactor::{ControlId, Element, ElementExt, Prop, PropValue, Reconciler, TextBlock};

fn collect_spans(r: &Reconciler<RecordingBackend>, id: ControlId) -> (Option<i32>, Option<i32>) {
    let mut row = None;
    let mut column = None;
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
                (Prop::AttachedWrapGridRowSpan, PropValue::I32(v)) => row = Some(*v),
                (Prop::AttachedWrapGridColumnSpan, PropValue::I32(v)) => column = Some(*v),
                _ => {}
            }
        }
    }
    (row, column)
}

#[test]
fn wrap_grid_spans_emit_set_props_on_mount() {
    let el: Element = TextBlock::new("t")
        .wrap_grid_row_span(3)
        .wrap_grid_column_span(2)
        .into();
    let mut r = Reconciler::new(RecordingBackend::new());
    let id = r
        .reconcile(None, &el, None, Rc::new(|| {}))
        .expect("mount produced a control id");

    let (row, column) = collect_spans(&r, id);
    assert_eq!(row, Some(3), "missing AttachedWrapGridRowSpan=3");
    assert_eq!(column, Some(2), "missing AttachedWrapGridColumnSpan=2");
}

#[test]
fn wrap_grid_spans_default_to_one_when_unset() {
    let el: Element = TextBlock::new("t").into();
    let mut r = Reconciler::new(RecordingBackend::new());
    let id = r
        .reconcile(None, &el, None, Rc::new(|| {}))
        .expect("mount produced a control id");

    let (row, column) = collect_spans(&r, id);
    assert_eq!(row, None, "unset row span must not emit a set_prop");
    assert_eq!(column, None, "unset column span must not emit a set_prop");
}

#[test]
fn wrap_grid_span_diff_emits_new_values() {
    let old: Element = TextBlock::new("t")
        .wrap_grid_row_span(3)
        .wrap_grid_column_span(2)
        .into();
    let mut r = Reconciler::new(RecordingBackend::new());
    let id = r
        .reconcile(None, &old, None, Rc::new(|| {}))
        .expect("mount produced a control id");

    // Change the spans: 3 -> 4, 2 -> 1.
    let new: Element = TextBlock::new("t")
        .wrap_grid_row_span(4)
        .wrap_grid_column_span(1)
        .into();
    let _ = r.reconcile(Some(&old), &new, Some(id), Rc::new(|| {}));

    let (row, column) = collect_spans(&r, id);
    assert_eq!(row, Some(4), "diff must emit the new row span");
    assert_eq!(column, Some(1), "diff must emit the new column span");
}

#[test]
fn wrap_grid_span_removal_resets_to_one() {
    let old: Element = TextBlock::new("t")
        .wrap_grid_row_span(3)
        .wrap_grid_column_span(2)
        .into();
    let mut r = Reconciler::new(RecordingBackend::new());
    let id = r
        .reconcile(None, &old, None, Rc::new(|| {}))
        .expect("mount produced a control id");

    // Remove both spans entirely: the diff path must reset them to 1.
    let new: Element = TextBlock::new("t").into();
    let _ = r.reconcile(Some(&old), &new, Some(id), Rc::new(|| {}));

    let (row, column) = collect_spans(&r, id);
    assert_eq!(row, Some(1), "removal must reset the row span to 1");
    assert_eq!(column, Some(1), "removal must reset the column span to 1");
}
