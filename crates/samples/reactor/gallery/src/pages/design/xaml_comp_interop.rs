use crate::controls::*;
use windows_reactor::*;

pub fn xaml_comp_interop_page(_: &(), cx: &mut RenderCx) -> Element {
    let (scale, set_scale) = cx.use_state(1.0_f64);
    let (status, set_status) = cx.use_state(String::from("Ready."));
    let expression_target = cx.use_ref(None::<UIElement>);
    let keyframe_target = cx.use_ref(None::<UIElement>);

    let expression_shape = Shape::rectangle()
        .fill_rgb(26, 115, 232)
        .width(160.0)
        .height(100.0)
        .on_mounted({
            let expression_target = expression_target.clone();
            move |native: Option<windows_core::IInspectable>| {
                if let Some(native) = native
                    && let Ok(ui) = native.cast::<UIElement>()
                {
                    expression_target.set(Some(ui));
                }
            }
        });

    let keyframe_shape = Shape::ellipse()
        .fill_rgb(155, 79, 150)
        .width(120.0)
        .height(120.0)
        .on_mounted({
            let keyframe_target = keyframe_target.clone();
            move |native: Option<windows_core::IInspectable>| {
                if let Some(native) = native
                    && let Ok(ui) = native.cast::<UIElement>()
                {
                    keyframe_target.set(Some(ui));
                }
            }
        });

    let drive_expression = {
        let set_status = set_status.clone();
        move |value: f64| {
            if let Some(ui) = expression_target.borrow().as_ref() {
                let scale = value as f32;
                match start_expression_animation(
                    ui,
                    "Scale",
                    "Vector3(scale, scale, 1)",
                    &[("scale", scale)],
                ) {
                    Ok(()) => set_status.call(format!("Expression animation: scale = {scale:.2}")),
                    Err(err) => set_status.call(format!("Error: {err}")),
                }
            }
        }
    };

    let run_keyframes = move || {
        if let Some(ui) = keyframe_target.borrow().as_ref() {
            match start_scale_keyframes(
                ui,
                &[(0.0, 1.0), (0.3, 1.6), (0.6, 0.85), (1.0, 1.0)],
                std::time::Duration::from_millis(600),
            ) {
                Ok(()) => set_status.call("Key-frame animation started.".into()),
                Err(err) => set_status.call(format!("Error: {err}")),
            }
        }
    };

    page_content(
        "XAML/COM Interop",
        "Drive XAML elements with Composition expression and key-frame animations.",
        vec![
            sample_card(
                "Expression Animation",
                vstack((
                    expression_shape,
                    Slider::new(scale)
                        .range(0.5, 2.0)
                        .step(0.05)
                        .header("Scale")
                        .on_value_changed(move |value| {
                            set_scale.call(value);
                            drive_expression(value);
                        }),
                    text_block(format!("Scale: {scale:.2}")).opacity(0.6),
                ))
                .spacing(8.0),
                r#"let anim = compositor.CreateExpressionAnimation()?;
anim.SetExpression("Vector3(scale, scale, 1)")?;
anim.SetScalarParameter("scale", scale)?;
anim.SetTarget("Scale")?;
element.StartAnimation(&anim.cast()?)?;"#,
            ),
            sample_card(
                "Key-frame Animation",
                vstack((
                    keyframe_shape,
                    button("Animate scale").on_click(run_keyframes),
                ))
                .spacing(8.0),
                r#"let anim = compositor.CreateVector3KeyFrameAnimation()?;
anim.InsertKeyFrame(0.3, Vector3 { x: 1.6, y: 1.6, z: 1.0 })?;
anim.SetDuration(600ms)?; anim.SetTarget("Scale")?;
element.StartAnimation(&anim.cast()?)?;"#,
            ),
            sample_card(
                "Status",
                text_block(status).opacity(0.6),
                r#"start_expression_animation(element, "Scale", "Vector3(scale, scale, 1)", &[("scale", 1.5)])
start_scale_keyframes(element, &[(0.0, 1.0), (0.3, 1.6), (0.6, 0.85), (1.0, 1.0)], 600ms)"#,
            ),
        ],
    )
}
