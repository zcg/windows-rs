use super::*;

/// 下拉刷新容器。内容（`child`）通常是 `ScrollViewer` 包一个可滚动列表；
/// 用户从顶部（或 `pull_direction` 指定方向）下拉时触发
/// [`RefreshContainer::on_refresh_requested`]。
///
/// 默认视觉指示器（`RefreshVisualizer`）由 WinUI 自动创建，无需显式挂载。
#[derive(Clone, Debug, PartialEq)]
pub struct RefreshContainer {
    pub key: Option<String>,
    pub modifiers: Modifiers,
    pub child: Box<Element>,
    pub pull_direction: RefreshPullDirection,
    pub on_refresh_requested: Option<Callback<RefreshRequest>>,
}
impl Default for RefreshContainer {
    fn default() -> Self {
        Self {
            key: None,
            modifiers: Modifiers::default(),
            child: Box::new(Element::Empty),
            pull_direction: RefreshPullDirection::TopToBottom,
            on_refresh_requested: None,
        }
    }
}
impl RefreshContainer {
    pub fn new(child: impl Into<Element>) -> Self {
        Self {
            child: Box::new(child.into()),
            ..Default::default()
        }
    }
    /// 设置下拉方向（默认 `TopToBottom`，即顶部下拉）。
    pub fn pull_direction(mut self, v: RefreshPullDirection) -> Self {
        self.pull_direction = v;
        self
    }
    /// 订阅下拉刷新手势。回调收到 [`RefreshRequest`]：
    /// - 同步场景：回调内直接重新加载数据，返回后容器自动完成刷新。
    /// - 异步场景：调用 `req.deferral()` 获取并持有 [`RefreshDeferral`]，
    ///   在异步工作完成后（可跨线程）调用其 `complete()`。
    pub fn on_refresh_requested(mut self, f: impl IntoCallback<RefreshRequest>) -> Self {
        self.on_refresh_requested = Some(f.into_callback());
        self
    }
}

impl Widget for RefreshContainer {
    widget_header!(ControlKind::RefreshContainer);
    fn bindings(&self) -> PropBindings {
        // `generated::refresh_container_bindings` 只生成属性（PullDirection）；
        // RefreshRequested 事件带特殊 payload（RefreshRequest），不在 winui.toml
        // 声明，由这里手动接入（与 EventHandler::Refresh 配套）。
        let mut out = generated::refresh_container_bindings(self);
        out.push(Binding::Event(
            Event::RefreshRequested,
            self.on_refresh_requested
                .as_ref()
                .map(|cb| EventHandler::Refresh(cb.clone())),
        ));
        out
    }
    fn children(&self) -> Children<'_> {
        Children::PositionalSingle(&self.child)
    }
}

/// 下拉刷新回调入参，携带底层 `RefreshRequestedEventArgs`。
#[derive(Clone, Debug, PartialEq)]
pub struct RefreshRequest {
    pub(crate) args: bindings::RefreshRequestedEventArgs,
}

impl RefreshRequest {
    /// 获取刷新 deferral。获取并持有后，刷新视觉指示器保持"刷新中"，
    /// 直到调用 [`RefreshDeferral::complete`]；若回调内从不获取，
    /// WinUI 会在回调返回时自动完成本次刷新。
    pub fn deferral(&self) -> Option<RefreshDeferral> {
        self.args.GetDeferral().ok().map(RefreshDeferral)
    }
}

/// 异步刷新的完成句柄，包装 `Windows.Foundation.Deferral`。
/// 内部 COM 对象可跨线程（`Send + Sync`），适合在异步工作完成后
/// 回到 UI 线程（或由 dispatcher 调度）再调用 [`complete`](Self::complete)。
#[derive(Clone)]
pub struct RefreshDeferral(bindings::Deferral);

impl RefreshDeferral {
    /// 通知容器本次刷新已完成，视觉指示器收回。
    pub fn complete(&self) {
        let _ = self.0.Complete();
    }
}
