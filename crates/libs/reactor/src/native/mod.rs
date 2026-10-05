use super::*;
use crate::core::*;

// ============================================================================
// 全局字号缩放
// ============================================================================
//
// 宿主想给整个 App 换一个字号系数（设置页的"字号"滑块），但 `TextBlock::font_size`
// 之类是**逐元素**声明的属性：应用里到处写着 `.font_size(14.0)`，一处一处改既
// 不现实也容易漏。WinUI 自己没有"默认字号"这样的全局开关可从宿主设置
// （`Control.FontSizeProperty` 的 statics 是 `pub(crate)`），所以这里提供
// **一个下发点的乘子**：所有字号在写进原生控件之前先乘以它。
//
// 消费点只有三处（`native/winui/generated.rs` 的 `TextBlockFontSize` /
// `TextBoxFontSize` / `RichTextBlockFontSize`）加一处 `TextBlockLineHeight`，
// 都在同一条 `set_property` 路径上 —— 改一处即全库生效。
//
// # 宿主要注意的：固定 DIP 布局不会跟着缩
//
// 这是这个开关的**代价**，也是它唯一真正的风险：宿主代码里凡是"按某个字号手算的
// 固定尺寸"（图标占位、行高、折行列数）不会自动跟着变。例如按"14px 中文一行
// 23 字"推出的折行宽度，在 scale=1.2 时就偏了，会重新出现孤字行。宿主要么让那些
// 推导也读 [`font_scale`]，要么像图标那样**显式除掉**它以保持不变
// （见 bili-rs 的 `coin_icon`：它要永远和 `SymbolIcon` 一样大）。

// 存 `f64` 的**位模式**，而不是 `AtomicF64` —— std 里**没有** `AtomicF64`
// （`core::sync::atomic` 只提供整数、bool、ptr；浮点原子是 nightly 的
// `floatAtomic`）。`AtomicU64` 在所有目标上都有，且 `to_bits`/`from_bits`
// 往返无损。
static FONT_SCALE_BITS: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(1.0f64.to_bits());

/// 当前全局字号缩放系数（`1.0` = 不缩放，即使用各元素自己声明的字号）。
///
/// 渲染路径每次下发字号属性都会读它，所以是无锁原子读。
#[inline]
pub fn font_scale() -> f64 {
    f64::from_bits(FONT_SCALE_BITS.load(std::sync::atomic::Ordering::Relaxed))
}

/// 设置全局字号缩放系数。
///
/// 返回 `false` 表示**非法输入，已保持原值不变**：`scale` 必须是有限正数。
/// 非法输入不能静默变成 1.0 —— 那会让"我调了字号但没反应"变成一个极难查的
/// 现象；而这里返回 `false`，调用方能立刻告诉用户。
///
/// # 什么时候调用
///
/// 宿主通常在收到"用户改了字号"时调用一次即可。系数是**全局状态**而不是
/// `WindowVisuals` 那样每帧声明的值，所以它**不会**自动把新值推给已经存在的
/// 控件 —— 要生效，宿主需要触发一次重渲染（改字号偏好本身就会引发重渲染）。
pub fn set_font_scale(scale: f64) -> bool {
    if !(scale.is_finite() && scale > 0.0) {
        return false;
    }
    FONT_SCALE_BITS.store(scale.to_bits(), std::sync::atomic::Ordering::Relaxed);
    true
}

pub(crate) enum FeedbackExpectation {
    Exact(EventPayload),
    Normalized { observation: Option<QueuedEvent> },
    Suppressed,
}

mod winui;
pub(crate) use winui::{NativeGrid, NativeHorizontalAlignment, NativeVerticalAlignment};

#[cfg(feature = "test")]
pub use winui::test::{
    LiveInputProbe, LiveInputProbeStage, application_corner_radius, schedule_live_test_exit,
    subscribe_live_rendering,
};
#[cfg(feature = "test")]
pub(crate) use winui::test::{finish_live_text_input_dispatch, subscribe_live_input_probe};
pub use winui::*;
