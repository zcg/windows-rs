#![doc = include_str!("../readme.md")]

mod app;
mod core;
mod element;
mod generated;
mod native;
mod reference;
#[cfg(any(test, feature = "test"))]
pub mod test;

use generated::*;

pub use app::*;
pub use core::public::*;
pub use element::*;
pub use generated::public::*;
/// 系统能力查询（背景材质是否可渲染）。应用拿它决定"内容区要不要让材质透出来"，
/// 详见 [`backdrop_is_supported`]。
pub use native::backdrop_is_supported;
/// 窗口几何查询（显示器工作区、窗口当前落点），以及把工作区换算成 DIP 的助手。
///
/// 声明位置的正路是 [`WindowVisuals::client_position`]（记住上次的位置）或
/// [`WindowVisuals::centered`]（居中到工作区）；这几个函数给需要自己算坐标或存位置的宿主
/// 用，例如"工作区的 70% 作为初始尺寸"（[`primary_work_area_dip`]）和"关闭时记下窗口在哪"
/// （[`window_placement_dip`]）。
pub use native::{
    WorkArea, primary_work_area, primary_work_area_dip, window_placement_dip, work_area_for_window,
};
pub use reference::{
    CompositionHostError, CompositionHostEvent, ElementObservation, ElementRef, FocusControl,
    FocusError, ImageSourceError, IntegrationError, ReferenceControl, ScrollOffsets,
    SwapChainPanelError, SwapChainPanelEvent, WebView2Error, WindowHandle, WindowRef,
};
pub use windows_time::{DateTime, TimeSpan};
