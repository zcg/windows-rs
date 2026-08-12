//! Composition animations started on XAML elements: expression animations
//! and vector key-frame animations, driven by the thread's compositor.

use super::*;

/// Get the compositor for the current UI thread.
pub fn compositor_for_current_thread() -> Result<bindings::Compositor> {
    bindings::CompositionTarget::GetCompositorForCurrentThread()
}

/// Start an expression animation on `element` targeting `property` (e.g.
/// `"Scale"`). `expression` may reference the scalar parameters in `scalars`
/// by name, as in `"Vector3(scale, scale, 1)"` with `[("scale", 1.5)]`.
pub fn start_expression_animation(
    element: &UIElement,
    property: &str,
    expression: &str,
    scalars: &[(&str, f32)],
) -> Result<()> {
    let compositor = compositor_for_current_thread()?;
    let animation = compositor.CreateExpressionAnimation()?;
    animation.SetExpression(expression)?;
    let base: bindings::ICompositionAnimation = animation.cast()?;
    for (key, value) in scalars {
        base.SetScalarParameter(key, *value)?;
    }
    let target: bindings::ICompositionAnimation2 = animation.cast()?;
    target.SetTarget(property)?;
    let base_animation: bindings::ICompositionAnimationBase = animation.cast()?;
    element.StartAnimation(&base_animation)
}

/// Start a key-frame animation on the `"Scale"` property of `element`.
/// `keyframes` maps normalized progress (`0.0..=1.0`) to scale factors.
pub fn start_scale_keyframes(
    element: &UIElement,
    keyframes: &[(f32, f32)],
    duration: std::time::Duration,
) -> Result<()> {
    let compositor = compositor_for_current_thread()?;
    let animation = compositor.CreateVector3KeyFrameAnimation()?;
    let frames: bindings::IVector3KeyFrameAnimation = animation.cast()?;
    for (progress, scale) in keyframes {
        frames.InsertKeyFrame(
            *progress,
            bindings::Vector3 {
                x: *scale,
                y: *scale,
                z: 1.0,
            },
        )?;
    }
    let timed: bindings::IKeyFrameAnimation = animation.cast()?;
    timed.SetDuration(TimeSpan::try_from(duration).unwrap_or(TimeSpan::MAX))?;
    let target: bindings::ICompositionAnimation2 = animation.cast()?;
    target.SetTarget("Scale")?;
    let base_animation: bindings::ICompositionAnimationBase = animation.cast()?;
    element.StartAnimation(&base_animation)
}
