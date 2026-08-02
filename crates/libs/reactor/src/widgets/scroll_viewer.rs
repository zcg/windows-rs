use super::*;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct ScrollViewer {
    pub key: Option<String>,
    pub modifiers: Modifiers,
    pub child: Box<Element>,
    pub horizontal_scroll_bar_visibility: ScrollBarVisibility,
    pub vertical_scroll_bar_visibility: ScrollBarVisibility,
    pub mounted: Option<Callback<Option<windows_core::IInspectable>>>,
}
impl Default for ScrollViewer {
    fn default() -> Self {
        Self {
            key: None,
            modifiers: Modifiers::default(),
            child: Box::new(Element::Empty),
            horizontal_scroll_bar_visibility: ScrollBarVisibility::Disabled,
            vertical_scroll_bar_visibility: ScrollBarVisibility::Auto,
            mounted: None,
        }
    }
}
impl ScrollViewer {
    pub fn new(child: impl Into<Element>) -> Self {
        Self {
            child: Box::new(child.into()),
            ..Default::default()
        }
    }
}

impl Widget for ScrollViewer {
    widget_header!(ControlKind::ScrollViewer);
    fn bindings(&self) -> PropBindings {
        generated::scroll_viewer_bindings(self)
    }
    fn children(&self) -> Children<'_> {
        Children::PositionalSingle(&self.child)
    }
    fn on_mounted_callback(&self) -> Option<&Callback<Option<windows_core::IInspectable>>> {
        self.mounted.as_ref()
    }
}

impl ScrollViewer {
    pub fn horizontal_scroll_bar_visibility(mut self, v: ScrollBarVisibility) -> Self {
        self.horizontal_scroll_bar_visibility = v;
        self
    }

    pub fn vertical_scroll_bar_visibility(mut self, v: ScrollBarVisibility) -> Self {
        self.vertical_scroll_bar_visibility = v;
        self
    }

    /// 订阅滚动变化：`(vertical_offset, scrollable_height)`（DIP）。
    /// 用于无限滚动分页：`offset + viewport >= scrollable` 时加载下一页。
    pub fn on_view_changed(mut self, f: impl Fn(f64, f64) + 'static) -> Self {
        let f = Rc::new(f);
        let prev = self.mounted.take();
        self.mounted = Some(Callback::new(
            move |native: Option<windows_core::IInspectable>| {
                if let Some(ref cb) = prev {
                    cb.invoke(native.clone());
                }
                let Some(native) = native else {
                    return;
                };
                if let Ok(sv) = native.cast::<bindings::IScrollViewer>() {
                    let f = f.clone();
                    if let Ok(revoker) = sv.ViewChanged(move |sender, _args| {
                        if let Some(s) = sender.as_ref()
                            && let Ok(sv) = s.cast::<bindings::IScrollViewer>()
                        {
                            let offset = sv.VerticalOffset().unwrap_or(0.0);
                            let scrollable = sv.ScrollableHeight().unwrap_or(0.0);
                            f(offset, scrollable);
                        }
                    }) {
                        let _ = revoker.into_token();
                    }
                }
            },
        ));
        self
    }
}

pub fn scroll_viewer(child: impl Into<Element>) -> ScrollViewer {
    ScrollViewer::new(child)
}
