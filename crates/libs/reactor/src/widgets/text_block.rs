use super::*;

#[derive(Clone, Default, Debug, PartialEq)]
pub struct TextBlock {
    pub key: Option<String>,
    pub modifiers: Modifiers,
    pub text: String,
    pub font_size: Option<f64>,
    pub font_weight: Option<u16>,
    pub font_family: Option<String>,
    pub max_lines: Option<i32>,
    pub text_trimming: Option<TextTrimming>,
    pub text_wrapping: TextWrapping,
    pub is_text_selection_enabled: bool,
}

/// 全局默认字体族（宿主设置；None = 用 XAML 默认）。
/// 只应用于 TextBlock（不设容器继承——避免个别字体整树渲染失败）。
static GLOBAL_FONT: std::sync::OnceLock<std::sync::Mutex<Option<String>>> =
    std::sync::OnceLock::new();

/// 设置全局默认字体族（之后创建/重渲染的 TextBlock 均应用）
pub fn set_default_font_family(name: impl Into<String>) {
    let cell = GLOBAL_FONT.get_or_init(|| std::sync::Mutex::new(None));
    *cell.lock().unwrap() = Some(name.into());
}

/// 清除全局默认字体族
pub fn clear_default_font_family() {
    if let Some(cell) = GLOBAL_FONT.get() {
        *cell.lock().unwrap() = None;
    }
}

fn default_font_family() -> Option<String> {
    let cell = GLOBAL_FONT.get()?;
    cell.lock().unwrap().clone()
}
impl TextBlock {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            text: content.into(),
            ..Self::default()
        }
    }
}

impl Widget for TextBlock {
    widget_header!(ControlKind::TextBlock);
    fn bindings(&self) -> PropBindings {
        let mut out = generated::text_block_bindings(self);
        if let Some(v) = self.font_size {
            out.push(Binding::Prop(Prop::FontSize, PropValue::F64(v)));
        }
        if let Some(v) = self.font_weight {
            out.push(Binding::Prop(Prop::FontWeight, PropValue::U16(v)));
        }
        if let Some(f) = &self.font_family {
            out.push(Binding::Prop(Prop::FontFamily, PropValue::Str(f.clone())));
        } else if let Some(f) = default_font_family() {
            // 全局默认字体（宿主经 set_default_font_family 设置）：未显式指定
            // 字体的 TextBlock 统一应用。不设到 NavigationView 等容器——
            // FontFamily 继承进内部模板会导致个别字体整树渲染失败（黑屏）。
            out.push(Binding::Prop(Prop::FontFamily, PropValue::Str(f)));
        }
        if let Some(v) = self.max_lines {
            out.push(Binding::Prop(Prop::MaxLines, PropValue::I32(v)));
        }
        if let Some(v) = self.text_trimming {
            out.push(Binding::Prop(
                Prop::TextTrimming,
                PropValue::I32(v.0),
            ));
        }
        out
    }
}

impl TextBlock {
    pub fn bold(mut self) -> Self {
        self.font_weight = Some(700);
        self
    }

    pub fn semibold(mut self) -> Self {
        self.font_weight = Some(600);
        self
    }

    pub fn font_weight(mut self, weight: u16) -> Self {
        self.font_weight = Some(weight);
        self
    }

    pub fn font_family(mut self, v: impl Into<String>) -> Self {
        self.font_family = Some(v.into());
        self
    }

    pub fn font_size(mut self, v: f64) -> Self {
        self.font_size = Some(v);
        self
    }

    pub fn wrap(mut self) -> Self {
        self.text_wrapping = TextWrapping::Wrap;
        self
    }

    pub fn max_lines(mut self, value: i32) -> Self {
        self.max_lines = Some(value);
        self
    }

    pub fn text_trimming(mut self, value: TextTrimming) -> Self {
        self.text_trimming = Some(value);
        self
    }

    pub fn selectable(mut self) -> Self {
        self.is_text_selection_enabled = true;
        self
    }
}

pub fn text_block(content: impl Into<String>) -> TextBlock {
    TextBlock::new(content)
}

// Type-ramp factories (WinUI 3 spec)

/// 28 px Semibold - page/section titles.
pub fn title(content: impl Into<String>) -> TextBlock {
    TextBlock::new(content).font_size(28.0).semibold()
}

/// 20 px Semibold - secondary headings.
pub fn subtitle(content: impl Into<String>) -> TextBlock {
    TextBlock::new(content).font_size(20.0).semibold()
}

/// 18 px Normal - larger body text.
pub fn body_large(content: impl Into<String>) -> TextBlock {
    TextBlock::new(content).font_size(18.0)
}

/// 14 px Semibold - emphasized body text.
pub fn body_strong(content: impl Into<String>) -> TextBlock {
    TextBlock::new(content).font_size(14.0).semibold()
}

/// 14 px Normal - standard body text.
pub fn body(content: impl Into<String>) -> TextBlock {
    TextBlock::new(content).font_size(14.0)
}

/// 12 px Normal - captions and secondary labels.
pub fn caption(content: impl Into<String>) -> TextBlock {
    TextBlock::new(content).font_size(12.0)
}
