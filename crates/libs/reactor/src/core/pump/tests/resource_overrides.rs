use super::*;

#[test]
fn resource_overrides_mount_update_and_clear() {
    let initial = ResourceOverrides::new()
        .set("ButtonBackground", Color::rgb(178, 34, 34))
        .set("ButtonBorderThemeThickness", Thickness::uniform(0.0));
    let replacement = ResourceOverrides::new()
        .set("ButtonForeground", Color::rgb(255, 255, 255))
        .set("ControlCornerRadius", CornerRadius::uniform(8.0));
    let mut pump = Pump::new(RecordingRuntime::default());
    pump.mount(Button::new().resource_overrides(initial.clone()).into())
        .unwrap();
    let root = pump.root().unwrap();
    assert_eq!(
        pump.runtime()
            .node(root)
            .unwrap()
            .property(PropertyId::ButtonResources),
        Some(&PropertyValue::ResourceOverrides(initial))
    );

    pump.update(Button::new().resource_overrides(replacement.clone()).into())
        .unwrap();
    assert_eq!(
        pump.runtime()
            .node(root)
            .unwrap()
            .property(PropertyId::ButtonResources),
        Some(&PropertyValue::ResourceOverrides(replacement))
    );

    pump.update(Button::new().into()).unwrap();
    assert_eq!(
        pump.runtime()
            .node(root)
            .unwrap()
            .property(PropertyId::ButtonResources),
        None
    );
}

/// 形状 token 的便捷方法必须落成 WinUI 认识的资源键，否则应用级注入会静默失效。
#[test]
fn shape_tokens_use_winui_corner_radius_keys() {
    let resources = ResourceOverrides::new()
        .control_corner_radius(6.0)
        .overlay_corner_radius(10.0);
    assert_eq!(
        resources.values().collect::<Vec<_>>(),
        vec![
            (
                "ControlCornerRadius",
                &ResourceValue::CornerRadius(CornerRadius::uniform(6.0))
            ),
            (
                "OverlayCornerRadius",
                &ResourceValue::CornerRadius(CornerRadius::uniform(10.0))
            ),
        ]
    );
    // 单独设置一个 token 时只写入它自己的键。
    assert_eq!(
        ResourceOverrides::new()
            .control_corner_radius(8.0)
            .values()
            .collect::<Vec<_>>(),
        vec![(
            "ControlCornerRadius",
            &ResourceValue::CornerRadius(CornerRadius::uniform(8.0))
        )]
    );
}

#[test]
#[should_panic(expected = "resource override corner radius must be finite and non-negative")]
fn shape_tokens_reject_a_negative_radius() {
    // 便捷方法与通用 set 共用同一条校验路径，非法圆角依旧被拒绝。
    let _ = ResourceOverrides::new().overlay_corner_radius(-1.0);
}
