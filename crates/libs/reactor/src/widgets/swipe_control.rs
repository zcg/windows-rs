use super::*;

/// A single swipe action definition (text label + optional invoked callback).
#[derive(Clone, Debug, PartialEq)]
pub struct SwipeItemDef {
    pub text: String,
    pub on_invoked: Option<Callback<String>>,
}

impl SwipeItemDef {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            on_invoked: None,
        }
    }
    pub fn on_invoked(mut self, f: impl IntoCallback<String>) -> Self {
        self.on_invoked = Some(f.into_callback());
        self
    }
}

/// `Microsoft.UI.Xaml.Controls.SwipeControl`. Wraps content and reveals
/// contextual actions on a swipe gesture.
#[derive(Clone, Debug, PartialEq)]
pub struct SwipeControl {
    pub key: Option<String>,
    pub modifiers: Modifiers,
    pub child: Box<Element>,
    pub left_items: Vec<SwipeItemDef>,
    pub right_items: Vec<SwipeItemDef>,
    pub mounted: Option<Callback<Option<windows_core::IInspectable>>>,
}
impl Default for SwipeControl {
    fn default() -> Self {
        Self {
            key: None,
            modifiers: Modifiers::default(),
            child: Box::new(Element::Empty),
            left_items: Vec::new(),
            right_items: Vec::new(),
            mounted: None,
        }
    }
}
impl SwipeControl {
    pub fn new(child: impl Into<Element>) -> Self {
        let mut this = Self {
            child: Box::new(child.into()),
            ..Default::default()
        };
        let left = this.left_items.clone();
        let right = this.right_items.clone();
        this.mounted = Some(Callback::new(
            move |native: Option<windows_core::IInspectable>| {
                let Some(native) = native else { return };
                if let Ok(sc) = native.cast::<bindings::SwipeControl>() {
                    // 左/右动作在挂载时创建并挂到原生控件。
                    if let Some(items) = build_swipe_items(&left) {
                        let _ = sc.SetLeftItems(&items);
                    }
                    if let Some(items) = build_swipe_items(&right) {
                        let _ = sc.SetRightItems(&items);
                    }
                }
            },
        ));
        this
    }
    pub fn left_items(mut self, items: Vec<SwipeItemDef>) -> Self {
        self.left_items = items;
        self.rebuild_mounted();
        self
    }
    pub fn right_items(mut self, items: Vec<SwipeItemDef>) -> Self {
        self.right_items = items;
        self.rebuild_mounted();
        self
    }
    fn rebuild_mounted(&mut self) {
        let left = self.left_items.clone();
        let right = self.right_items.clone();
        self.mounted = Some(Callback::new(
            move |native: Option<windows_core::IInspectable>| {
                let Some(native) = native else { return };
                if let Ok(sc) = native.cast::<bindings::SwipeControl>() {
                    if let Some(items) = build_swipe_items(&left) {
                        let _ = sc.SetLeftItems(&items);
                    }
                    if let Some(items) = build_swipe_items(&right) {
                        let _ = sc.SetRightItems(&items);
                    }
                }
            },
        ));
    }
}

fn build_swipe_items(defs: &[SwipeItemDef]) -> Option<bindings::SwipeItems> {
    let items = bindings::SwipeItems::new().ok()?;
    for def in defs {
        let item = bindings::SwipeItem::new().ok()?;
        item.SetText(def.text.as_str()).ok()?;
        if let Some(cb) = &def.on_invoked {
            let cb = cb.clone();
            let label = def.text.clone();
            item.Invoked(move |_s, _a| cb.invoke(label.clone())).ok();
        }
        if let Ok(vec) = items.cast::<windows_collections::IVector<bindings::SwipeItem>>() {
            vec.Append(&item).ok()?;
        }
    }
    Some(items)
}

impl Widget for SwipeControl {
    widget_header!(ControlKind::SwipeControl);
    fn children(&self) -> Children<'_> {
        Children::PositionalSingle(&self.child)
    }
    fn bindings(&self) -> PropBindings {
        Vec::new()
    }
    fn on_mounted_callback(&self) -> Option<&Callback<Option<windows_core::IInspectable>>> {
        self.mounted.as_ref()
    }
}

pub fn swipe_control(child: impl Into<Element>) -> SwipeControl {
    SwipeControl::new(child)
}
