use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Slider {
    pub key: Option<String>,
    pub modifiers: Modifiers,
    pub value: f64,
    pub minimum: f64,
    pub maximum: f64,
    pub step: Option<f64>,
    pub on_value_changed: Option<Callback<f64>>,
    pub header: Option<String>,
    pub orientation: Orientation,
    pub is_enabled: bool,
    /// 原生 thumb tooltip（拖拽时跟随小圆点显示 Value 数字）。进度条需要
    /// 显示时间点，宿主禁用原生 + 叠加自绘 tooltip（跟随 thumb 定位）。
    pub is_thumb_tooltip_enabled: bool,
    /// 布局宽度变化回调（DIP）。宿主据此把自绘 tooltip 定位到 thumb 上方。
    pub on_size_changed: Option<Callback<f64>>,
}
impl Default for Slider {
    fn default() -> Self {
        Self {
            key: None,
            modifiers: Modifiers::default(),
            value: 0.0,
            minimum: 0.0,
            maximum: 100.0,
            step: None,
            on_value_changed: None,
            header: None,
            orientation: Orientation::Horizontal,
            is_enabled: true,
            is_thumb_tooltip_enabled: true,
            on_size_changed: None,
        }
    }
}
impl Slider {
    pub fn new(value: f64) -> Self {
        Self {
            value,
            ..Default::default()
        }
    }
    pub fn range(mut self, min: f64, max: f64) -> Self {
        self.minimum = min;
        self.maximum = max;
        self
    }
    pub fn step(mut self, v: f64) -> Self {
        self.step = Some(v);
        self
    }
    pub fn on_value_changed(mut self, f: impl IntoCallback<f64>) -> Self {
        self.on_value_changed = Some(f.into_callback());
        self
    }
    pub fn header(mut self, s: impl Into<String>) -> Self {
        self.header = Some(s.into());
        self
    }
    /// Switch to a vertical slider (`ISlider::Orientation = Vertical`).
    pub fn vertical(mut self) -> Self {
        self.orientation = Orientation::Vertical;
        self
    }
    /// Switch to a horizontal slider (default).
    pub fn horizontal(mut self) -> Self {
        self.orientation = Orientation::Horizontal;
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.is_enabled = enabled;
        self
    }
    /// Toggle the native thumb tooltip (the "1234" bubble that follows the
    /// thumb while dragging). Disable it when a custom timepoint tooltip is
    /// overlaid, to avoid showing both.
    pub fn thumb_tooltip_enabled(mut self, enabled: bool) -> Self {
        self.is_thumb_tooltip_enabled = enabled;
        self
    }
    /// Layout width callback (DIP), fired on every size change. Lets the host
    /// position an overlaid tooltip above the thumb: `x = value/max * width`.
    pub fn on_size_changed(mut self, f: impl IntoCallback<f64>) -> Self {
        self.on_size_changed = Some(f.into_callback());
        self
    }
}

impl Widget for Slider {
    widget_header!(ControlKind::Slider);
    fn bindings(&self) -> PropBindings {
        let mut out = generated::slider_bindings(self);
        if let Some(v) = self.step {
            out.push(Binding::Prop(Prop::Step, PropValue::F64(v)));
        }
        // 原生 thumb tooltip 开关 + 布局宽度回调（宿主自绘 tooltip 定位用）——
        // 未进 winui.toml，属手工绑定（与 TextBlock.font_size 同款惯例）。
        out.push(Binding::Prop(
            Prop::IsThumbToolTipEnabled,
            PropValue::Bool(self.is_thumb_tooltip_enabled),
        ));
        if let Some(cb) = &self.on_size_changed {
            out.push(Binding::Event(
                Event::SliderSizeChanged,
                Some(EventHandler::F64(cb.clone())),
            ));
        }
        out
    }
}
