use super::*;

#[test]
fn semantic_zoom_views_mount_with_framework_sizes() {
    let mut pump = Pump::new(RecordingRuntime::default());
    pump.mount_view(
        SemanticZoom::new()
            .is_zoomed_in_view_active(true)
            .zoomed_in_view(ListView::new().height(240.0))
            .zoomed_out_view(ListView::new().height(120.0))
            .into(),
    )
    .unwrap();

    let batches = pump.runtime().commands();
    let heights: Vec<f64> = batches
        .iter()
        .flatten()
        .filter_map(|command| match command {
            Command::SetProperty {
                property: PropertyId::Height,
                value: PropertyValue::F64(height),
                ..
            } => Some(*height),
            _ => None,
        })
        .collect();
    assert!(
        heights.contains(&240.0) && heights.contains(&120.0),
        "slot children must receive their framework sizes, got {heights:?}"
    );

    let batch = batches
        .iter()
        .flatten()
        .filter_map(|command| match command {
            Command::SetSlot { slot, .. } => Some(*slot),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(
        batch.contains(&SlotId::SemanticZoomZoomedInView)
            && batch.contains(&SlotId::SemanticZoomZoomedOutView),
        "semantic zoom views must be attached through their slots, got {batch:?}"
    );
}
