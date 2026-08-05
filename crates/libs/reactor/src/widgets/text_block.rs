use super::*;

#[derive(Clone, Default, Debug, PartialEq)]
pub struct TextBlock {
    pub key: Option<String>,
    pub modifiers: Modifiers,
    pub text: String,
    pub font_size: Option<f64>,
    pub font_weight: Option<u16>,
    pub text_wrapping: TextWrapping,
    pub is_text_selection_enabled: bool,
    pub max_lines: Option<u32>,
    pub text_trimming: Option<i32>,
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
        if let Some(v) = self.max_lines {
            out.push(Binding::Prop(Prop::MaxLines, PropValue::I32(v as i32)));
        }
        if let Some(v) = self.text_trimming {
            out.push(Binding::Prop(Prop::TextTrimming, PropValue::I32(v)));
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

    pub fn font_size(mut self, v: f64) -> Self {
        self.font_size = Some(v);
        self
    }

    pub fn wrap(mut self) -> Self {
        self.text_wrapping = TextWrapping::Wrap;
        self
    }

    /// 限制最大显示行数（WinUI `MaxLines`）。与 `wrap` 组合实现"截断到 N 行"，
    /// 保证卡片等高布局（PiliPlus `maxLines: 2` 等价）。
    pub fn max_lines(mut self, v: u32) -> Self {
        self.max_lines = Some(v);
        self
    }

    /// 超出 `max_lines` 后的裁剪方式（WinUI `TextTrimming`）：
    /// 0=None 1=CharacterEllipsis 2=WordEllipsis 3=Clip。与 `max_lines` 组合实现
    /// "截断到 N 行 + 省略号"（PiliPlus `overflow: .ellipsis` 等价）。
    pub fn text_trimming(mut self, v: i32) -> Self {
        self.text_trimming = Some(v);
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
