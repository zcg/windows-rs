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
pub use reference::{
    CompositionHostError, CompositionHostEvent, ElementObservation, ElementRef, FocusControl,
    FocusError, ImageSourceError, IntegrationError, ReferenceControl, ScrollOffsets,
    SwapChainPanelError, SwapChainPanelEvent, WebView2Error, WindowHandle, WindowRef,
};
pub use windows_time::{DateTime, TimeSpan};
