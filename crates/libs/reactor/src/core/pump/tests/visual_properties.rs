use super::*;

fn visual_view(styled: bool) -> View {
    let border = Border::new();
    let text = TextBlock::new().text("Card");
    if styled {
        border
            .padding(Thickness::uniform(24.0))
            .border_thickness(Thickness::uniform(1.0))
            .corner_radius(CornerRadius::uniform(8.0))
            .content(text.font_size(28.0))
    } else {
        border.content(text)
    }
}

fn theme_view(background: Option<ThemeBrush>, border_brush: Option<ThemeBrush>) -> View {
    Border::new()
        .background_optional(background)
        .border_brush_optional(border_brush)
        .content(TextBlock::new().text("Card"))
}

fn brush_view(background: Option<Brush>) -> View {
    Border::new()
        .background_optional(background)
        .content(TextBlock::new().text("Card"))
}

#[test]
fn visual_values_reject_invalid_components_before_mount() {
    for invalid in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(
            std::panic::catch_unwind(|| Border::new().padding(Thickness::uniform(invalid)))
                .is_err()
        );
        assert!(
            std::panic::catch_unwind(|| {
                Border::new().corner_radius(CornerRadius::uniform(invalid))
            })
            .is_err()
        );
        assert!(std::panic::catch_unwind(|| TextBlock::new().font_size(invalid)).is_err());
        assert!(std::panic::catch_unwind(|| Border::new().scale(invalid)).is_err());
    }

    for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(
            std::panic::catch_unwind(|| Border::new().margin(Thickness::uniform(invalid))).is_err()
        );
    }

    let _ = Border::new()
        .margin(Thickness::uniform(-1.0))
        .padding(Thickness::uniform(0.0))
        .border_thickness(Thickness::uniform(0.0))
        .corner_radius(CornerRadius::uniform(0.0));
}

#[test]
fn implicit_visual_transitions_mount_update_and_clear() {
    let duration = std::time::Duration::from_secs(1);
    let mut pump = Pump::new(RecordingRuntime::default());
    pump.mount(
        Border::new()
            .opacity(1.0)
            .opacity_transition(duration)
            .scale(1.0)
            .scale_transition(duration)
            .translation(Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            })
            .translation_transition(duration)
            .into(),
    )
    .unwrap();
    let border = pump.root().unwrap();
    assert_eq!(
        pump.runtime()
            .node(border)
            .unwrap()
            .property(PropertyId::BorderOpacityTransition),
        Some(&PropertyValue::Duration(duration))
    );
    assert_eq!(
        pump.runtime()
            .node(border)
            .unwrap()
            .property(PropertyId::BorderScaleTransition),
        Some(&PropertyValue::Duration(duration))
    );
    assert_eq!(
        pump.runtime()
            .node(border)
            .unwrap()
            .property(PropertyId::BorderTranslation),
        Some(&PropertyValue::Vector3(Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0
        }))
    );
    assert_eq!(
        pump.runtime()
            .node(border)
            .unwrap()
            .property(PropertyId::BorderTranslationTransition),
        Some(&PropertyValue::Duration(duration))
    );

    pump.update(
        Border::new()
            .opacity(0.2)
            .opacity_transition(duration)
            .scale(1.3)
            .scale_transition(duration)
            .translation(Vector3 {
                x: 0.0,
                y: 32.0,
                z: 0.0,
            })
            .translation_transition(duration)
            .into(),
    )
    .unwrap();
    assert_eq!(
        pump.runtime()
            .node(border)
            .unwrap()
            .property(PropertyId::Opacity),
        Some(&PropertyValue::F64(0.2))
    );
    assert_eq!(
        pump.runtime()
            .node(border)
            .unwrap()
            .property(PropertyId::BorderScale),
        Some(&PropertyValue::F64(1.3))
    );
    assert_eq!(
        pump.runtime()
            .node(border)
            .unwrap()
            .property(PropertyId::BorderTranslation),
        Some(&PropertyValue::Vector3(Vector3 {
            x: 0.0,
            y: 32.0,
            z: 0.0
        }))
    );

    pump.update(Border::new().into()).unwrap();
    assert_eq!(
        pump.runtime()
            .node(border)
            .unwrap()
            .property(PropertyId::BorderOpacityTransition),
        None
    );
    assert_eq!(
        pump.runtime()
            .node(border)
            .unwrap()
            .property(PropertyId::BorderScaleTransition),
        None
    );
    assert_eq!(
        pump.runtime()
            .node(border)
            .unwrap()
            .property(PropertyId::BorderTranslation),
        None
    );
    assert_eq!(
        pump.runtime()
            .node(border)
            .unwrap()
            .property(PropertyId::BorderTranslationTransition),
        None
    );
}

#[test]
fn acrylic_brush_takes_the_plain_property_path_not_the_theme_style_one() {
    let acrylic = Acrylic::new(Color::rgb(0x1C, 0x1C, 0x22), 60);
    let mut pump = Pump::new(RecordingRuntime::default());
    pump.mount_view(Border::new().background(Brush::Acrylic(acrylic)).into())
        .unwrap();
    let border = pump.root().unwrap();

    assert_eq!(
        pump.runtime()
            .node(border)
            .unwrap()
            .property(PropertyId::BorderBackground),
        Some(&PropertyValue::Brush(Brush::Acrylic(acrylic)))
    );
    // 亚克力不是主题资源刷：主题样式保持为空，不能顺带下发主题资源。
    assert_eq!(
        pump.runtime().node(border).unwrap().theme_style(),
        ThemeStyle::default()
    );
}

#[test]
fn border_mount_records_struct_values_content_and_typography() {
    let mut pump = Pump::new(RecordingRuntime::default());
    pump.mount_view(visual_view(true)).unwrap();
    let border = pump.root().unwrap();
    let text = pump.runtime().node(border).unwrap().children()[0];

    assert_eq!(
        pump.runtime()
            .node(border)
            .unwrap()
            .property(PropertyId::BorderPadding),
        Some(&PropertyValue::Thickness(Thickness::uniform(24.0)))
    );
    assert_eq!(
        pump.runtime()
            .node(border)
            .unwrap()
            .property(PropertyId::BorderCornerRadius),
        Some(&PropertyValue::CornerRadius(CornerRadius::uniform(8.0)))
    );
    assert_eq!(
        pump.runtime()
            .node(text)
            .unwrap()
            .property(PropertyId::TextBlockFontSize),
        Some(&PropertyValue::F64(28.0))
    );
}

#[test]
fn border_preserves_independent_corner_radii() {
    let radius = CornerRadius::new(1.0, 2.0, 3.0, 4.0);
    let mut pump = Pump::new(RecordingRuntime::default());
    pump.mount_view(Border::new().corner_radius(radius.clone()).into())
        .unwrap();
    let border = pump.root().unwrap();

    assert_eq!(
        pump.runtime()
            .node(border)
            .unwrap()
            .property(PropertyId::BorderCornerRadius),
        Some(&PropertyValue::CornerRadius(radius))
    );
}

#[test]
fn visual_property_clear_and_no_op_use_the_shared_property_path() {
    let mut pump = Pump::new(RecordingRuntime::default());
    pump.mount_view(visual_view(true)).unwrap();
    let border = pump.root().unwrap();
    let text = pump.runtime().node(border).unwrap().children()[0];

    pump.update_view(visual_view(false)).unwrap();
    assert_eq!(
        pump.runtime()
            .node(border)
            .unwrap()
            .property(PropertyId::BorderPadding),
        None
    );
    assert_eq!(
        pump.runtime()
            .node(text)
            .unwrap()
            .property(PropertyId::TextBlockFontSize),
        None
    );
    let batches = pump.runtime().batches();

    pump.update_view(visual_view(false)).unwrap();
    assert_eq!(pump.runtime().batches(), batches);
}

#[test]
fn failed_visual_update_does_not_publish_struct_values() {
    let mut pump = Pump::new(RecordingRuntime::default());
    pump.mount_view(visual_view(true)).unwrap();
    let border = pump.root().unwrap();
    pump.runtime_mut().fail_at(0);

    assert!(matches!(
        pump.update_view(
            Border::new()
                .padding(Thickness::uniform(32.0))
                .content(TextBlock::new().text("Card"))
        ),
        Err(PumpError::NativeApplyFailed(_))
    ));
    assert_eq!(
        pump.tree
            .native(border)
            .properties
            .get(&PropertyId::BorderPadding),
        Some(&PropertyValue::Thickness(Thickness::uniform(24.0)))
    );
}

#[test]
fn theme_brushes_mount_as_one_grouped_style() {
    let mut pump = Pump::new(RecordingRuntime::default());
    pump.mount_view(theme_view(
        Some(ThemeBrush::CardBackground),
        Some(ThemeBrush::CardStroke),
    ))
    .unwrap();
    let border = pump.root().unwrap();
    let style = ThemeStyle::new([
        Some(ThemeBrush::CardBackground),
        Some(ThemeBrush::CardStroke),
        None,
        None,
    ]);

    assert_eq!(pump.runtime().node(border).unwrap().theme_style(), style);
    assert_eq!(
        pump.runtime()
            .commands()
            .iter()
            .flatten()
            .filter(|command| matches!(command, Command::SetThemeStyle { .. }))
            .count(),
        1
    );
    assert!(
        pump.runtime()
            .commands()
            .iter()
            .flatten()
            .all(|command| !matches!(
                command,
                Command::SetProperty {
                    property: PropertyId::BorderBackground | PropertyId::BorderBorderBrush,
                    ..
                }
            ))
    );
}

#[test]
fn theme_style_update_clear_and_no_op_are_transactional() {
    let mut pump = Pump::new(RecordingRuntime::default());
    pump.mount_view(theme_view(
        Some(ThemeBrush::CardBackground),
        Some(ThemeBrush::CardStroke),
    ))
    .unwrap();
    let border = pump.root().unwrap();

    pump.update_view(theme_view(
        Some(ThemeBrush::SolidBackground),
        Some(ThemeBrush::CardStroke),
    ))
    .unwrap();
    assert_eq!(
        pump.runtime().node(border).unwrap().theme_style(),
        ThemeStyle::new([
            Some(ThemeBrush::SolidBackground),
            Some(ThemeBrush::CardStroke),
            None,
            None,
        ])
    );

    pump.update_view(theme_view(None, None)).unwrap();
    assert_eq!(
        pump.runtime().node(border).unwrap().theme_style(),
        ThemeStyle::default()
    );
    let batches = pump.runtime().batches();
    pump.update_view(theme_view(None, None)).unwrap();
    assert_eq!(pump.runtime().batches(), batches);
}

#[test]
fn brush_transitions_switch_between_theme_property_and_inherited_values() {
    let mut pump = Pump::new(RecordingRuntime::default());
    pump.mount_view(brush_view(Some(ThemeBrush::CardBackground.into())))
        .unwrap();
    let border = pump.root().unwrap();
    let color = Color::rgb(20, 40, 60);

    pump.update_view(brush_view(Some(color.into()))).unwrap();
    assert_eq!(
        pump.runtime().node(border).unwrap().theme_style(),
        ThemeStyle::default()
    );
    assert_eq!(
        pump.runtime()
            .node(border)
            .unwrap()
            .property(PropertyId::BorderBackground),
        Some(&PropertyValue::Brush(Brush::Solid(color)))
    );
    assert!(
        pump.runtime()
            .commands()
            .last()
            .unwrap()
            .iter()
            .any(|command| matches!(
                command,
                Command::SetThemeStyle { style, .. } if style.is_empty()
            ))
    );

    let batches = pump.runtime().batches();
    pump.update_view(brush_view(Some(color.into()))).unwrap();
    assert_eq!(pump.runtime().batches(), batches);

    pump.update_view(brush_view(Some(ThemeBrush::SolidBackground.into())))
        .unwrap();
    assert_eq!(
        pump.runtime().node(border).unwrap().theme_style(),
        ThemeStyle::new([Some(ThemeBrush::SolidBackground), None, None, None])
    );
    assert_eq!(
        pump.runtime()
            .node(border)
            .unwrap()
            .property(PropertyId::BorderBackground),
        None
    );
    assert!(
        pump.runtime()
            .commands()
            .last()
            .unwrap()
            .iter()
            .any(|command| matches!(
                command,
                Command::ClearProperty {
                    property: PropertyId::BorderBackground,
                    ..
                }
            ))
    );

    pump.update_view(brush_view(None)).unwrap();
    assert_eq!(
        pump.runtime().node(border).unwrap().theme_style(),
        ThemeStyle::default()
    );
    assert_eq!(
        pump.runtime()
            .node(border)
            .unwrap()
            .property(PropertyId::BorderBackground),
        None
    );
}

#[test]
fn failed_theme_style_update_does_not_publish() {
    let mut pump = Pump::new(RecordingRuntime::default());
    pump.mount_view(theme_view(
        Some(ThemeBrush::CardBackground),
        Some(ThemeBrush::CardStroke),
    ))
    .unwrap();
    let border = pump.root().unwrap();
    let original = pump.tree.native(border).desired.theme_style();
    pump.runtime_mut().fail_at(0);

    assert!(matches!(
        pump.update_view(theme_view(
            Some(ThemeBrush::SolidBackground),
            Some(ThemeBrush::CardStroke),
        )),
        Err(PumpError::NativeApplyFailed(_))
    ));
    assert_eq!(pump.tree.native(border).desired.theme_style(), original);
    assert_eq!(pump.runtime().node(border).unwrap().theme_style(), original);
}

/// 虚拟化网格的列结构（与 `bili-rs` 的 `ui/virtual_grid::column_children` 同形）：
/// `[顶部 spacer] + 可见卡… + [底部 spacer]`。滚动 = spacer 高度变化 + 卡片增删。
///
/// 卡片用 `border_brush` 表达 hover 态（这正是 `bili-rs` 里 `feed_card` / `live_card`
/// 的写法）。
fn virtualized_column(top_h: f64, cards: &[(&str, bool)], bottom_h: f64) -> View {
    let accent = Color::rgb(0xFB, 0x72, 0x99);
    let normal = Color::rgb(0x1F, 0x1F, 0x24);
    let mut children: Vec<(String, View)> = Vec::new();
    if top_h > 0.5 {
        children.push((
            "col0_top".to_string(),
            Border::new().height(top_h).content(View::empty()),
        ));
    }
    for (key, hovered) in cards {
        children.push((
            (*key).to_string(),
            Border::new()
                .border_brush(if *hovered { accent } else { normal })
                .into(),
        ));
    }
    if bottom_h > 0.5 {
        children.push((
            "col0_bottom".to_string(),
            Border::new().height(bottom_h).content(View::empty()),
        ));
    }
    StackPanel::new().keyed_children(children)
}

fn keyed_child(pump: &Pump<RecordingRuntime>, parent: NodeId, key: &str) -> NodeId {
    pump.tree
        .children(parent)
        .iter()
        .copied()
        .find(|child| pump.tree.key(*child) == Some(&Key::from(key)))
        .unwrap_or_else(|| panic!("keyed child `{key}` not found"))
}

fn border_brush_of(pump: &Pump<RecordingRuntime>, node: NodeId) -> Option<&PropertyValue> {
    pump.runtime()
        .node(node)
        .and_then(|state| state.property(PropertyId::BorderBorderBrush))
}

/// 虚拟化滚动（spacer 高度变化 + 卡片增删）与 hover 状态**在同一拍**变化时，
/// 每张卡的 hover 视觉属性必须落在**它自己**身上：不会丢，更不会串到别的卡。
///
/// 这是"reconciler 对同 key 元素的 brush 更新在虚拟化滚动时不可靠"这类说法的
/// 直接对手实验。若那种说法成立，下面第二组断言（`b` 亮着、`c` 暗着）就会失败
/// ——那正是用户描述过的"所有卡都显示 hover 态"。
#[test]
fn keyed_cards_keep_their_own_hover_brush_across_virtualized_scroll() {
    let accent = Color::rgb(0xFB, 0x72, 0x99);
    let normal = Color::rgb(0x1F, 0x1F, 0x24);
    let brush = |color| PropertyValue::Brush(Brush::Solid(color));

    let mut pump = Pump::new(RecordingRuntime::default());
    // 列表顶端：没有顶部 spacer；`a` 被 hover、`b` 没有。
    pump.mount_view(virtualized_column(0.0, &[("a", true), ("b", false)], 100.0))
        .unwrap();
    let root = pump.root().unwrap();
    let a = keyed_child(&pump, root, "a");
    let b = keyed_child(&pump, root, "b");
    assert_eq!(border_brush_of(&pump, a), Some(&brush(accent)));
    assert_eq!(border_brush_of(&pump, b), Some(&brush(normal)));

    // 滚动一拍：顶部 spacer 长出来（`a` 滚出窗口被移除、`c` 滚进来），
    // 同时"指针此刻停在 `b` 上"——窗口滑动与 hover 变化发生在同一拍。
    pump.update_view(virtualized_column(120.0, &[("b", true), ("c", false)], 0.0))
        .unwrap();
    let b2 = keyed_child(&pump, root, "b");
    assert_eq!(
        b2, b,
        "同 key 的卡片必须复用同一个节点（而不是重新挂载），否则这个实验就没在考察复用路径"
    );
    assert_eq!(
        border_brush_of(&pump, b2),
        Some(&brush(accent)),
        "被 hover 的卡（key 不变、位置平移）必须拿到 accent 边框——相邻卡的插入/移除不能把它丢掉"
    );
    assert_eq!(
        border_brush_of(&pump, keyed_child(&pump, root, "c")),
        Some(&brush(normal)),
        "新进入窗口的卡必须是常态色——不能继承窗口里上一张卡的 hover 态"
    );

    // 再滚一拍：`b` 松开、`c` 被指到，仍然伴随一次窗口滑动，两边的属性各归各。
    pump.update_view(virtualized_column(80.0, &[("b", false), ("c", true)], 0.0))
        .unwrap();
    assert_eq!(border_brush_of(&pump, b), Some(&brush(normal)));
    assert_eq!(
        border_brush_of(&pump, keyed_child(&pump, root, "c")),
        Some(&brush(accent))
    );
}
