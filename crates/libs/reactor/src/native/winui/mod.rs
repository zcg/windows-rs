use std::any::Any;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Write;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use super::*;
use crate::Brush as ReactorBrush;
use crate::Color as ReactorColor;
use crate::SwipeItem as SwipeItemDef;
use crate::VirtualKey as ReactorVirtualKey;
use windows_core::Interface;

windows_core::link!("kernel32.dll" "system" fn FindResourceW(module: *mut std::ffi::c_void, name: *const u16, resource_type: *const u16) -> *mut std::ffi::c_void);
windows_core::link!("kernel32.dll" "system" fn GetModuleHandleW(name: *const u16) -> *mut std::ffi::c_void);
windows_core::link!("kernel32.dll" "system" fn LoadResource(module: *mut std::ffi::c_void, resource: *mut std::ffi::c_void) -> *mut std::ffi::c_void);
windows_core::link!("kernel32.dll" "system" fn LockResource(resource: *mut std::ffi::c_void) -> *mut std::ffi::c_void);
windows_core::link!("kernel32.dll" "system" fn SizeofResource(module: *mut std::ffi::c_void, resource: *mut std::ffi::c_void) -> u32);

#[cfg_attr(not(feature = "test"), allow(dead_code))]
#[allow(
    clippy::missing_transmute_annotations,
    clippy::upper_case_acronyms,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals
)]
mod bindings;
pub use bindings::*;
pub(crate) use bindings::{
    Grid as NativeGrid, HorizontalAlignment as NativeHorizontalAlignment,
    VerticalAlignment as NativeVerticalAlignment,
};
mod app_shim;
pub use app_shim::*;
mod bootstrap;
mod content_dialog;
#[cfg(feature = "test")]
pub(crate) use content_dialog::LiveContentDialogState;
use content_dialog::{ContentDialogAction, ContentDialogScheduler};
mod element_factory;
#[allow(unused_qualifications)]
mod generated;
pub use generated::*;
mod framework;
mod grid;
mod window_geometry;
pub use window_geometry::{
    WorkArea, primary_work_area, primary_work_area_dip, window_placement_dip, work_area_for_window,
};
use window_geometry::{window_frame_size, work_area_of};
#[cfg(feature = "test")]
pub(crate) mod test;
#[cfg(feature = "test")]
pub(crate) use test::native_window_handle;

fn unit_event_handler<TSender, TArgs>(
    sink: EventSink,
    node: NodeId,
    event: EventId,
    revision: u32,
) -> impl Fn(windows_core::Ref<TSender>, windows_core::Ref<TArgs>)
where
    TSender: windows_core::RuntimeType + 'static,
    TArgs: windows_core::RuntimeType + 'static,
{
    move |_, _| sink.enqueue(node, event, revision, EventPayload::Unit)
}

fn drag_info_event_handler(
    sink: EventSink,
    node: NodeId,
    event: EventId,
    revision: u32,
) -> impl Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<DragEventArgs>) {
    move |_, args| {
        let result = args
            .as_ref()
            .ok_or_else(windows_core::Error::empty)
            .and_then(|args| {
                let data = args.DataView()?;
                let kind = if data.Contains("Shell IDList Array")? {
                    DragKind::StorageItems
                } else if data.Contains("Text")? {
                    DragKind::Text
                } else {
                    DragKind::Unsupported
                };
                let action = sink.drag_action(node, kind);
                args.SetAcceptedOperation(
                    action
                        .as_ref()
                        .map_or(DataPackageOperation::None, |action| {
                            native_drag_operation(action.operation)
                        }),
                )?;
                let ui = args.DragUIOverride()?;
                if let Some(caption) = action.as_ref().and_then(|action| action.caption.as_deref())
                {
                    ui.SetCaption(caption)?;
                    ui.SetIsCaptionVisible(true)?;
                } else {
                    ui.SetIsCaptionVisible(false)?;
                }
                Ok(action.map_or(DragKind::Unsupported, |_| kind))
            });
        match result {
            Ok(kind) => sink.enqueue(node, event, revision, EventPayload::DragKind(kind)),
            Err(error) => sink.error(node, event, revision, native_error(error)),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum PointerEventPhase {
    Plain,
    Press,
    Release,
}

fn pointer_event_handler(
    sink: EventSink,
    node: NodeId,
    event: EventId,
    revision: u32,
    element: UIElement,
    phase: PointerEventPhase,
) -> impl Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<PointerRoutedEventArgs>)
{
    move |_, args| {
        let mut info = PointerEventInfo::default();
        if let Some(args) = args.as_ref() {
            if let Ok(point) = args.GetCurrentPoint(&element) {
                if let Ok(position) = point.Position() {
                    info.x = f64::from(position.x);
                    info.y = f64::from(position.y);
                }
                if let Ok(properties) = point.Properties() {
                    info.is_left_button_pressed = properties.IsLeftButtonPressed().unwrap_or(false);
                    info.is_right_button_pressed =
                        properties.IsRightButtonPressed().unwrap_or(false);
                    info.is_middle_button_pressed =
                        properties.IsMiddleButtonPressed().unwrap_or(false);
                }
            }
            if let Ok(point) = args.GetCurrentPoint(None::<&UIElement>)
                && let Ok(position) = point.Position()
            {
                info.window_x = f64::from(position.x);
                info.window_y = f64::from(position.y);
            }
        }
        if phase == PointerEventPhase::Press {
            info.capture_succeeded = match sink.apply_pointer_press_policy(node, &element, args) {
                Ok(value) => value,
                Err(error) => {
                    sink.error(node, event, revision, error);
                    return;
                }
            };
        } else if phase == PointerEventPhase::Release
            && let Err(error) =
                sink.apply_pointer_release_policy(node, event, revision, &element, args)
        {
            sink.error(node, event, revision, error);
            return;
        }
        sink.enqueue(node, event, revision, EventPayload::PointerEventInfo(info));
    }
}

fn key_event_handler(
    sink: EventSink,
    node: NodeId,
    event: EventId,
    revision: u32,
) -> impl Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<KeyRoutedEventArgs>) {
    move |_, args| {
        let result = args
            .as_ref()
            .ok_or_else(windows_core::Error::empty)
            .and_then(key_event_info);
        match result {
            Ok(info) => {
                let handled = sink.route_key(node, event, revision, info);
                if let Some(args) = args.as_ref()
                    && let Err(error) = args.SetHandled(handled)
                {
                    sink.error(node, event, revision, native_error(error));
                }
            }
            Err(error) => sink.error(node, event, revision, native_error(error)),
        }
    }
}

enum RoutedEventAction {
    Unit,
    Focus(UIElement, bool),
    Password(IPasswordBox),
    ToggleSwitch(IToggleSwitch),
    RichEdit(IRichEditBox),
}

fn routed_event_handler(
    sink: EventSink,
    node: NodeId,
    event: EventId,
    revision: u32,
    action: RoutedEventAction,
) -> impl Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<RoutedEventArgs>) {
    move |_, args| match &action {
        RoutedEventAction::Unit => {
            sink.enqueue(node, event, revision, EventPayload::Unit);
        }
        RoutedEventAction::Focus(element, got_focus) => {
            let result = args
                .as_ref()
                .ok_or_else(windows_core::Error::empty)
                .and_then(|args| {
                    focus_event_info(
                        element,
                        args,
                        got_focus
                            .then(|| sink.take_pending_focus_state(node))
                            .flatten(),
                        *got_focus,
                    )
                });
            match result {
                Ok(info) => sink.enqueue(node, event, revision, EventPayload::FocusEventInfo(info)),
                Err(error) => sink.error(node, event, revision, native_error(error)),
            }
        }
        RoutedEventAction::Password(source) => match source.Password() {
            Ok(value) => sink.enqueue(node, event, revision, EventPayload::Str(value)),
            Err(error) => sink.error(node, event, revision, native_error(error)),
        },
        RoutedEventAction::ToggleSwitch(source) => match source.IsOn() {
            Ok(value) => sink.enqueue(node, event, revision, EventPayload::Bool(value)),
            Err(error) => sink.error(node, event, revision, native_error(error)),
        },
        RoutedEventAction::RichEdit(source) => {
            let value = source.Document().and_then(|document| {
                let mut value = windows_core::HSTRING::new();
                document
                    .GetText(TextGetOptions::UseLf, &mut value)
                    .map(|_| value)
            });
            match value {
                Ok(value) => {
                    sink.enqueue_rich_edit_text(node, event, revision, value.to_string_lossy());
                }
                Err(error) => sink.error(node, event, revision, native_error(error)),
            }
        }
    }
}

enum SelectionChangedAction {
    IndexSelector(ISelector),
    IndexRadioButtons(IRadioButtons),
    IndexPivot(IPivot),
    IndexTabView(ITabView),
    ListBox(ISelector),
}

fn selection_changed_handler(
    sink: EventSink,
    node: NodeId,
    event: EventId,
    revision: u32,
    action: SelectionChangedAction,
) -> impl Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<SelectionChangedEventArgs>)
{
    move |_, _| {
        if let SelectionChangedAction::ListBox(source) = &action {
            match source.SelectedItem() {
                Ok(item) => {
                    let selected = sink.selection_item(&item);
                    match selection_payload(selection_for_event(event).unwrap(), &item) {
                        Ok(tag) => sink.enqueue(
                            node,
                            event,
                            revision,
                            EventPayload::SelectionChange(SelectionChange {
                                item: selected,
                                tag,
                            }),
                        ),
                        Err(error) => sink.error(node, event, revision, error),
                    }
                }
                Err(error) if error.code().is_ok() => sink.enqueue(
                    node,
                    event,
                    revision,
                    EventPayload::SelectionChange(SelectionChange {
                        item: None,
                        tag: None,
                    }),
                ),
                Err(error) => sink.error(node, event, revision, native_error(error)),
            }
            return;
        }
        let value = match &action {
            SelectionChangedAction::IndexSelector(source) => source.SelectedIndex(),
            SelectionChangedAction::IndexRadioButtons(source) => source.SelectedIndex(),
            SelectionChangedAction::IndexPivot(source) => source.SelectedIndex(),
            SelectionChangedAction::IndexTabView(source) => source.SelectedIndex(),
            SelectionChangedAction::ListBox(_) => unreachable!(),
        };
        match value {
            Ok(value) => {
                let value = match selection_index(value) {
                    Ok(value) => value,
                    Err(error) => {
                        sink.error(node, event, revision, error);
                        return;
                    }
                };
                sink.enqueue(node, event, revision, EventPayload::SelectionIndex(value));
            }
            Err(error) => sink.error(node, event, revision, native_error(error)),
        }
    }
}

fn list_view_items_changed_handler(
    sink: EventSink,
    node: NodeId,
    event: EventId,
    revision: u32,
) -> impl Fn(windows_core::Ref<ListViewBase>, windows_core::Ref<DragItemsCompletedEventArgs>) {
    move |sender, _| {
        if let Some(sender) = sender.as_ref() {
            let result = sender
                .cast::<IItemsControl>()
                .and_then(|sender| sender.Items())
                .and_then(|items| {
                    let mut tags = Vec::with_capacity(items.Size()? as usize);
                    for index in 0..items.Size()? {
                        let tag = items
                            .GetAt(index)?
                            .cast::<IFrameworkElement>()?
                            .Tag()?
                            .cast::<windows_reference::IReference<windows_core::HSTRING>>()?
                            .Value()?;
                        tags.push(tag.to_string_lossy());
                    }
                    Ok(tags)
                });
            match result {
                Ok(value) => {
                    sink.enqueue(node, event, revision, EventPayload::StrList(Rc::new(value)));
                }
                Err(error) => sink.error(node, event, revision, native_error(error)),
            }
        }
    }
}

fn toggle_button_checked_handler(
    sink: EventSink,
    node: NodeId,
    event: EventId,
    revision: u32,
    source: IToggleButton,
) -> impl Fn(windows_core::Ref<DependencyObject>, windows_core::Ref<DependencyProperty>) {
    move |_, _| match source.IsChecked() {
        Ok(value) => sink.enqueue(node, event, revision, EventPayload::Bool(value)),
        Err(error) => sink.error(node, event, revision, native_error(error)),
    }
}

impl Handle {
    pub fn ui_element(&self) -> windows_core::Result<UIElement> {
        self.inspectable().cast()
    }

    pub fn dependency_object(&self) -> windows_core::Result<IDependencyObject> {
        self.inspectable().cast()
    }
}

#[inline]
fn set_content_control<T: Interface>(
    control: &T,
    child: Option<&UIElement>,
) -> Result<(), RuntimeError> {
    let control = control.cast::<IContentControl>().map_err(native_error)?;
    match child {
        Some(child) => control.SetContent(child).map_err(native_error),
        None => control
            .SetContent(None::<&windows_core::IInspectable>)
            .map_err(native_error),
    }
}

#[inline]
fn clear_value(
    handle: &Handle,
    property: impl FnOnce() -> windows_core::Result<DependencyProperty>,
) -> Result<(), RuntimeError> {
    handle
        .dependency_object()
        .map_err(native_error)?
        .ClearValue(&property().map_err(native_error)?)
        .map_err(native_error)
}

enum PropertyTarget<'a> {
    Framework(UIElement),
    Attached(UIElement),
    GridDefinitions(&'a Handle),
    Generated(&'a Handle),
}

/// 把 [`ResourceValue`] 转换成 WinUI 资源字典可以存放的 boxed 值。
///
/// 元素级覆盖（[`WinUiRuntime::set_resource_overrides`]）与应用级覆盖
/// （[`install_theme_resources`]）共用这一条转换路径，保证两处语义完全一致。
pub(crate) fn resource_value(
    value: &ResourceValue,
) -> windows_core::Result<windows_core::IInspectable> {
    Ok(match value {
        ResourceValue::Color(value) => {
            let brush = SolidColorBrush::new()?;
            brush.SetColor(bindings::Color {
                a: value.a,
                r: value.r,
                g: value.g,
                b: value.b,
            })?;
            brush.into()
        }
        ResourceValue::Thickness(value) => {
            windows_reference::IReference::from(bindings::Thickness {
                left: value.left(),
                top: value.top(),
                right: value.right(),
                bottom: value.bottom(),
            })
            .into()
        }
        ResourceValue::CornerRadius(value) => {
            windows_reference::IReference::from(bindings::CornerRadius {
                top_left: value.top_left(),
                top_right: value.top_right(),
                bottom_right: value.bottom_right(),
                bottom_left: value.bottom_left(),
            })
            .into()
        }
    })
}

impl<'a> PropertyTarget<'a> {
    fn resolve(
        runtime: &'a WinUiRuntime,
        node: NodeId,
        property: PropertyId,
    ) -> Result<Self, RuntimeError> {
        Ok(match property {
            PropertyId::Width
            | PropertyId::Height
            | PropertyId::MinWidth
            | PropertyId::MaxWidth
            | PropertyId::MinHeight
            | PropertyId::MaxHeight
            | PropertyId::Opacity
            | PropertyId::HorizontalAlignment
            | PropertyId::VerticalAlignment
            | PropertyId::Margin
            | PropertyId::Transitions => Self::Framework(runtime.ui_element(node)?),
            PropertyId::GridRow
            | PropertyId::GridColumn
            | PropertyId::GridRowSpan
            | PropertyId::GridColumnSpan
            | PropertyId::WrapGridRowSpan
            | PropertyId::WrapGridColumnSpan
            | PropertyId::RelativeAlignLeft
            | PropertyId::RelativeAlignTop
            | PropertyId::RelativeAlignRight
            | PropertyId::RelativeAlignBottom
            | PropertyId::RelativeAlignHorizontalCenter
            | PropertyId::RelativeAlignVerticalCenter
            | PropertyId::CanvasLeft
            | PropertyId::CanvasTop
            | PropertyId::AutomationName
            | PropertyId::AutomationId
            | PropertyId::AutomationHeadingLevel => Self::Attached(runtime.ui_element(node)?),
            PropertyId::GridRows | PropertyId::GridColumns => Self::GridDefinitions(
                runtime
                    .handles
                    .get(&node)
                    .ok_or(RuntimeError::MissingNode(node))?,
            ),
            _ => Self::Generated(
                runtime
                    .handles
                    .get(&node)
                    .ok_or(RuntimeError::MissingNode(node))?,
            ),
        })
    }

    fn set(&self, property: PropertyId, value: &PropertyValue) -> Result<(), RuntimeError> {
        match self {
            Self::Framework(element) => framework::set(element, property, value),
            Self::Attached(element) => grid::set_attached(element, property, value),
            Self::GridDefinitions(handle) => grid::set_definitions(handle, property, value),
            Self::Generated(handle) => set_property(handle, property, value),
        }
    }

    fn clear(&self, property: PropertyId) -> Result<(), RuntimeError> {
        match self {
            Self::Framework(element) => framework::clear(element, property),
            Self::Attached(element) => grid::clear_attached(element, property),
            Self::GridDefinitions(handle) => grid::clear_definitions(handle, property),
            Self::Generated(handle) => clear_property(handle, property),
        }
    }
}

pub enum NativeSubscription {
    Event {
        _revoker: windows_core::EventRevoker,
        revision: u32,
    },
    Property {
        object: DependencyObject,
        property: DependencyProperty,
        token: i64,
    },
}

#[derive(Clone, Copy, Default)]
struct PointerInteractionPolicy {
    capture: bool,
    focus_on_release: bool,
}

impl PointerInteractionPolicy {
    fn is_empty(self) -> bool {
        !self.capture && !self.focus_on_release
    }
}

impl Drop for NativeSubscription {
    fn drop(&mut self) {
        if let Self::Property {
            object,
            property,
            token,
        } = self
        {
            _ = object.UnregisterPropertyChangedCallback(&*property, *token);
        }
    }
}

#[derive(Default)]
pub struct WinUiRuntime {
    application: Option<(NodeId, Application)>,
    event_errors: Rc<RefCell<Vec<NativeWork<QueuedEventError>>>>,
    handles: HashMap<NodeId, Handle>,
    events: Rc<RefCell<Vec<NativeWork<QueuedEvent>>>>,
    host_events: Rc<RefCell<Vec<NativeWork<HostEvent>>>>,
    async_ingress: Arc<Mutex<AsyncIngressQueue>>,
    async_state: Rc<RefCell<AsyncIngressState>>,
    encoded_image_nodes: Rc<RefCell<HashSet<NodeId>>>,
    feedback: Rc<RefCell<HashMap<(NodeId, EventId), FeedbackExpectation>>>,
    rich_edit_feedback: Rc<RefCell<HashMap<NodeId, String>>>,
    controlled_collection_indices: HashMap<NodeId, i32>,
    content_dialogs: Rc<RefCell<ContentDialogScheduler>>,
    drop_policies: Rc<RefCell<HashMap<NodeId, DragDropPolicy>>>,
    flyouts: HashMap<NodeId, (bindings::Flyout, NodeId)>,
    owned_menus: HashMap<NodeId, NativeOwnedMenu>,
    pending_focus_states: Rc<RefCell<HashMap<NodeId, ElementFocusState>>>,
    pointer_policies: Rc<RefCell<HashMap<NodeId, PointerInteractionPolicy>>>,
    routed_callbacks: Rc<RefCell<HashMap<(NodeId, EventId), RoutedEventCallback>>>,
    resource_override_keys: HashMap<NodeId, HashSet<String>>,
    command_bar_flyouts: HashMap<NodeId, NativeCommandBarFlyout>,
    swipe_attachments: HashMap<NodeId, NativeSwipeItems>,
    identity: Rc<Cell<Option<WindowToken>>>,
    selection_items: Rc<RefCell<Vec<(NodeId, windows_core::IInspectable)>>>,
    selection_owners: HashMap<NodeId, (NodeId, SlotId)>,
    realizations: Rc<RefCell<Vec<NativeWork<RealizationRequest>>>>,
    retained_subtrees: HashMap<NodeId, NativeRetainedSubtree>,
    scheduler: Rc<RefCell<SchedulerState>>,
    image_decode_tickets: HashMap<NodeId, u64>,
    observation_subscriptions: HashMap<(NodeId, u64), ObservationSubscription>,
    subscriptions: HashMap<(NodeId, EventId), NativeSubscription>,
    /// `Border::scale` 的缩放原点订阅（见 [`WinUiRuntime::sync_scale_center`]）：
    /// `CenterPoint` 只能在元素测量之后建立，故按元素挂 `SizeChanged` 维持。
    scale_center_subscriptions: HashMap<NodeId, windows_core::EventRevoker>,
    theme_styles: HashMap<(MountedKind, ThemeStyle), Style>,
    virtuals: HashMap<NodeId, element_factory::VirtualHandle>,
    window_backdrops: Rc<RefCell<HashMap<NodeId, BackdropHost>>>,
    window_closed: Rc<Cell<bool>>,
    window_observations: HashMap<NodeId, WindowObservationFlags>,
    window_hosts: HashMap<NodeId, bindings::Grid>,
    window_observation_subscriptions: HashMap<NodeId, Vec<windows_core::EventRevoker>>,
    window_subscriptions: HashMap<NodeId, windows_core::EventRevoker>,
    window_title_bars: HashMap<NodeId, (NodeId, WindowTitleBarHeight)>,
    window_title_revisions: Rc<RefCell<HashMap<NodeId, u64>>>,
    window_visuals: HashMap<NodeId, WindowVisuals>,
    webview_initializations: Rc<RefCell<HashMap<NodeId, WebViewInitialization>>>,
    windows: HashMap<NodeId, Window>,
    pending_application: Option<Application>,
    #[cfg(feature = "test")]
    native_apply_times_us: Vec<f64>,
}

struct WebViewInitialization {
    _action: Option<windows_future::IAsyncAction>,
    _loaded: Option<windows_core::EventRevoker>,
    _initialized: windows_core::EventRevoker,
    completions: Vec<Callback<Result<windows_core::IUnknown, RuntimeError>>>,
}

fn complete_webview_initialization(
    initializations: &Rc<RefCell<HashMap<NodeId, WebViewInitialization>>>,
    node: NodeId,
    result: Result<windows_core::IUnknown, RuntimeError>,
) {
    let Some(initialization) = initializations.borrow_mut().remove(&node) else {
        return;
    };
    for completion in initialization.completions {
        _ = completion.call(result.clone());
    }
}

struct XamlRootScaleSubscriptions {
    _changed: Rc<RefCell<Option<windows_core::EventRevoker>>>,
    _loaded: windows_core::EventRevoker,
}

enum ObservationSubscription {
    SwapChainPanel {
        _rendering: windows_core::EventRevoker,
        _scale: windows_core::EventRevoker,
        _size: windows_core::EventRevoker,
    },
    ImageScale {
        _root: XamlRootScaleSubscriptions,
    },
    CompositionHost {
        _root: XamlRootScaleSubscriptions,
        _size: windows_core::EventRevoker,
    },
    ScrollOffsets {
        _anchor: Option<windows_core::EventRevoker>,
        _view: Rc<RefCell<Option<windows_core::EventRevoker>>>,
    },
}

struct NativeRetainedSubtree {
    nodes: Vec<NodeId>,
    parent: NodeId,
    slot: Option<SlotId>,
    ticket: u64,
    _timer: DispatcherQueueTimer,
    _tick: windows_core::EventRevoker,
}

#[derive(Default)]
struct AsyncIngressQueue {
    identity: Option<WindowToken>,
    completions: VecDeque<AsyncIngressCompletion>,
}

struct AsyncIngressCompletion {
    identity: WindowToken,
    ticket: u64,
    payload: Box<dyn Any + Send>,
}

#[derive(Default)]
struct AsyncIngressState {
    next_ticket: u64,
    pending: HashMap<u64, PendingAsync>,
}

struct PendingAsync {
    node: NodeId,
    cancel: Box<dyn FnOnce()>,
    finalize: Box<dyn FnOnce(&mut WinUiRuntime, Box<dyn Any + Send>)>,
}

#[derive(Clone)]
struct AsyncIngressSender<T> {
    identity: WindowToken,
    ticket: u64,
    ingress: Arc<Mutex<AsyncIngressQueue>>,
    wake: Arc<dyn Fn() -> bool + Send + Sync>,
    marker: PhantomData<fn(T)>,
}

impl<T: Send + 'static> AsyncIngressSender<T> {
    fn complete(self, payload: T) -> bool {
        let completion = AsyncIngressCompletion {
            identity: self.identity,
            ticket: self.ticket,
            payload: Box::new(payload),
        };
        self.enqueue(completion)
    }

    fn enqueue(self, completion: AsyncIngressCompletion) -> bool {
        {
            let mut ingress = self.ingress.lock().unwrap();
            if ingress.identity != Some(self.identity) {
                return false;
            }
            ingress.completions.push_back(completion);
        }
        if (self.wake)() {
            true
        } else {
            self.ingress
                .lock()
                .unwrap()
                .completions
                .retain(|completion| completion.ticket != self.ticket);
            false
        }
    }
}

struct NativeOwnedMenu {
    target: NodeId,
    kind: OwnedMenuKind,
    _flyout: Option<MenuFlyout>,
    _revokers: Vec<windows_core::EventRevoker>,
}

struct NativeCommandBarFlyout {
    target: NodeId,
    _flyout: bindings::CommandBarFlyout,
    _revokers: Vec<windows_core::EventRevoker>,
}

struct NativeSwipeItems {
    target: NodeId,
    _revokers: Vec<windows_core::EventRevoker>,
}

fn scroll_offsets_of(viewer: &bindings::ScrollViewer) -> ScrollOffsets {
    ScrollOffsets {
        horizontal: viewer.HorizontalOffset().unwrap_or(0.0),
        vertical: viewer.VerticalOffset().unwrap_or(0.0),
        zoom: viewer.ZoomFactor().unwrap_or(1.0),
        extent_width: viewer.ExtentWidth().unwrap_or(0.0),
        extent_height: viewer.ExtentHeight().unwrap_or(0.0),
        viewport_width: viewer.ViewportWidth().unwrap_or(0.0),
        viewport_height: viewer.ViewportHeight().unwrap_or(0.0),
    }
}

/// 以元素**当前的测量尺寸**为缩放原点（`Border::scale` 用）。
///
/// 未测量时（宽或高为 0）什么都不设：此时按 0 反算会把原点定在左上角，交给
/// [`WinUiRuntime::sync_scale_center`] 挂的 `SizeChanged` 在尺寸到位后补设。
fn apply_scale_center(
    element: &IUIElement,
    framework: &IFrameworkElement,
) -> Result<(), RuntimeError> {
    let width = framework.ActualWidth().map_err(native_error)? as f32;
    let height = framework.ActualHeight().map_err(native_error)? as f32;
    if width <= 0.0 || height <= 0.0 {
        return Ok(());
    }
    element
        .SetCenterPoint(Vector3 {
            x: width / 2.0,
            y: height / 2.0,
            z: 0.0,
        })
        .map_err(native_error)
}

fn emit_scroll_offsets(viewer: &bindings::ScrollViewer, callback: &Callback<ScrollOffsets>) {
    invoke_callback(callback, scroll_offsets_of(viewer));
}

fn subscribe_scroll_offsets(
    viewer: &bindings::ScrollViewer,
    callback: &Callback<ScrollOffsets>,
    late: &Rc<RefCell<Option<windows_core::EventRevoker>>>,
) -> Result<(), RuntimeError> {
    emit_scroll_offsets(viewer, callback);
    let scoped = viewer.clone();
    let callback = callback.clone();
    let revoker = viewer
        .ViewChanged(move |_, _| {
            emit_scroll_offsets(&scoped, &callback);
        })
        .map_err(native_error)?;
    *late.borrow_mut() = Some(revoker);
    Ok(())
}

/// Finds the drop-down list viewer of a combo box through its open popup.
///
/// Template part names are an implementation detail, so resolution walks the
/// visual tree of open popups instead of looking names up.
fn combo_box_scroll_viewer(
    control: &bindings::ComboBox,
) -> windows_core::Result<Option<bindings::ScrollViewer>> {
    let root = control.cast::<IUIElement>()?.XamlRoot()?;
    let popups = VisualTreeHelper::GetOpenPopupsForXamlRoot(&root)?;
    // The opening drop-down is the freshest popup; walk newest first.
    let count = popups.Size()?;
    for index in (0..count).rev() {
        let child: DependencyObject = popups.GetAt(index)?.Child()?.cast()?;
        if let Some(viewer) = find_scroll_viewer(&child) {
            return Ok(Some(viewer));
        }
    }
    Ok(None)
}

fn find_scroll_viewer(object: &DependencyObject) -> Option<bindings::ScrollViewer> {
    if let Ok(viewer) = object.cast::<bindings::ScrollViewer>() {
        return Some(viewer);
    }
    let count = VisualTreeHelper::GetChildrenCount(object).ok()?;
    for index in 0..count {
        let child = VisualTreeHelper::GetChild(object, index).ok()?;
        if let Some(viewer) = find_scroll_viewer(&child) {
            return Some(viewer);
        }
    }
    None
}

fn build_swipe_items(
    items: &[SwipeItemDef],
    owner: NodeId,
    revision: u32,
    sink: &EventSink,
    output: &windows_collections::IVector<bindings::SwipeItem>,
    revokers: &mut Vec<windows_core::EventRevoker>,
) -> Result<(), RuntimeError> {
    for item in items {
        let native = bindings::SwipeItem::new().map_err(native_error)?;
        native.SetText(&item.text).map_err(native_error)?;
        if let Some(ReactorBrush::Solid(color)) = item.background {
            let brush = SolidColorBrush::new().map_err(native_error)?;
            brush
                .SetColor(bindings::Color {
                    a: color.a,
                    r: color.r,
                    g: color.g,
                    b: color.b,
                })
                .map_err(native_error)?;
            native.SetBackground(&brush).map_err(native_error)?;
        }
        if let Some(ReactorBrush::Solid(color)) = item.foreground {
            let brush = SolidColorBrush::new().map_err(native_error)?;
            brush
                .SetColor(bindings::Color {
                    a: color.a,
                    r: color.r,
                    g: color.g,
                    b: color.b,
                })
                .map_err(native_error)?;
            native.SetForeground(&brush).map_err(native_error)?;
        }
        let text = item.text.clone();
        let sink = sink.clone();
        revokers.push(
            native
                .Invoked(move |_, _| {
                    sink.enqueue(
                        owner,
                        EventId::OwnedSwipeInvoked,
                        revision,
                        EventPayload::String(text.clone()),
                    );
                })
                .map_err(native_error)?,
        );
        output.Append(&native).map_err(native_error)?;
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct WindowVisualChanges {
    backdrop: bool,
    backdrop_tuning: bool,
    border: bool,
    client_size: bool,
    constraints: bool,
    corner: bool,
    icon: bool,
    placement: bool,
    presenter: bool,
    theme: bool,
}

/// Whether `property` is a control's element-level `Resources` overrides property.
///
/// The generated dispatch cannot apply this property on its own: `FrameworkElement.Resources`
/// has no setter, so the generated arm only ever answers `RuntimeError::UnsupportedKind`.
/// [`WinUiRuntime::set_resource_overrides`] and [`WinUiRuntime::clear_resource_overrides`] do
/// the real work, and they are generic — they take any node's `IFrameworkElement.Resources`
/// dictionary — so this predicate is the only thing standing between the generator's output and
/// a control that opted into the `resource_overrides` adapter.
///
/// Keep the list in sync with the `resource_overrides` `OneOf` allow-list in
/// `crates/tools/reactor/src/schema.rs`. Adding a control to `winui.toml` without adding it
/// here compiles and generates, but every `resource_overrides` write on that control fails at
/// runtime with `UnsupportedKind`.
fn is_resource_overrides_property(property: PropertyId) -> bool {
    matches!(
        property,
        PropertyId::ButtonResources
            | PropertyId::CheckBoxResources
            | PropertyId::ComboBoxResources
            | PropertyId::NavigationViewResources
            | PropertyId::NumberBoxResources
            | PropertyId::ProgressBarResources
            | PropertyId::SliderResources
            | PropertyId::TextBoxResources
            | PropertyId::ToggleButtonResources
            | PropertyId::ToggleSwitchResources
    )
}

fn set_resource_style(
    element: &IFrameworkElement,
    resource: Option<&str>,
) -> Result<(), RuntimeError> {
    let Some(resource) = resource else {
        return element.SetStyle(None::<&Style>).map_err(native_error);
    };
    // A style is decoration: one missing resource key must not take the frame down.
    //
    // The distinction is not academic. Failing here surfaces as
    // `PumpError::NativeApplyFailed`, and that path runs `publish.rs::fail_stop`: the frame is
    // dropped and the pump is poisoned, so every control's properties in that frame go
    // unapplied and the window stops updating altogether — not just the one control that
    // asked for a missing style. A single wrong resource name therefore reads as "the entire
    // window stopped updating".
    //
    // The keys themselves still have to be right, and they drift with the SDK. `Subtle` is
    // the cautionary example: it maps to `"SubtleButtonStyle"`, which only exists from
    // WindowsAppSDK 1.8 onwards (this workspace pins 2.4.0 → WinUI 2.3.6, where the key is
    // defined at `generic.xaml` line 27717). Verifying that by hand is error-prone — note
    // the lookup command recorded in `winui.toml`: `Select-String -SimpleMatch` leaves
    // `MatchInfo.Matches` empty, so `... | % { $_.Matches.Count }` returns 0 for *every*
    // file and silently "proves" that a key which is present is missing.
    match lookup_resource_style(resource) {
        Ok(style) => element.SetStyle(&style).map_err(native_error),
        Err(error) => {
            // Keep a trace — this is still something the application should fix — but do
            // not make the frame pay for it. Falling back to "no explicit style" leaves the
            // control on its own default template, which is a working button.
            eprintln!("windows-reactor: style resource '{resource}' is unavailable: {error:?}");
            element.SetStyle(None::<&Style>).map_err(native_error)
        }
    }
}

pub(crate) fn validate_native_image_uri(value: &str) -> windows_core::Result<()> {
    Uri::CreateUri(value).map(drop)
}

pub(crate) fn validate_native_uri(value: &str) -> windows_core::Result<()> {
    Uri::CreateUri(value).map(drop)
}

fn lookup_resource_style(resource: &str) -> Result<Style, RuntimeError> {
    let resources = Application::Current()
        .and_then(|application| application.Resources())
        .map_err(native_error)?;
    let resources = resources
        .cast::<windows_collections::IMap<windows_core::IInspectable, windows_core::IInspectable>>()
        .map_err(native_error)?;
    let key = windows_reference::IReference::from(windows_core::HSTRING::from(resource));
    let style = resources
        .Lookup(&key)
        .and_then(|style| style.cast::<Style>())
        .map_err(native_error)?;
    Ok(style)
}

fn build_command_bar_element(
    command: &CommandBarCommand,
    owner: NodeId,
    revision: u32,
    sink: &EventSink,
) -> Result<(ICommandBarElement, Option<windows_core::EventRevoker>), RuntimeError> {
    match command {
        CommandBarCommand::Button {
            label,
            icon,
            enabled,
            ..
        } => {
            let button = bindings::AppBarButton::new().map_err(native_error)?;
            button.SetLabel(label).map_err(native_error)?;
            button
                .cast::<IControl>()
                .and_then(|control| control.SetIsEnabled(*enabled))
                .map_err(native_error)?;
            if let Some(icon) = icon {
                let icon_element = bindings::SymbolIcon::new().map_err(native_error)?;
                set_property(
                    &Handle::SymbolIcon(icon_element.clone()),
                    PropertyId::SymbolIconSymbol,
                    &PropertyValue::Symbol(*icon),
                )?;
                button.SetIcon(&icon_element).map_err(native_error)?;
            }
            let label = label.clone();
            let sink = sink.clone();
            let revoker = button
                .cast::<IButtonBase>()
                .and_then(|button| {
                    button.Click(move |_, _| {
                        sink.enqueue(
                            owner,
                            EventId::OwnedCommandInvoked,
                            revision,
                            EventPayload::String(label.clone()),
                        );
                    })
                })
                .map_err(native_error)?;
            Ok((button.cast().map_err(native_error)?, Some(revoker)))
        }
        CommandBarCommand::Separator { .. } => Ok((
            bindings::AppBarSeparator::new()
                .and_then(|separator| separator.cast())
                .map_err(native_error)?,
            None,
        )),
    }
}

pub(crate) fn build_native_menu_items(
    items: &[MenuItem],
    output: &windows_collections::IVector<MenuFlyoutItemBase>,
    revokers: &mut Vec<windows_core::EventRevoker>,
    invoke: &Rc<dyn Fn(String)>,
) -> windows_core::Result<()> {
    for item in items {
        let native: MenuFlyoutItemBase = match item {
            MenuItem::Item { label, enabled, .. } => {
                let item = MenuFlyoutItem::new()?;
                item.SetText(label)?;
                item.cast::<IControl>()
                    .and_then(|control| control.SetIsEnabled(*enabled))?;
                let label = label.clone();
                let invoke = Rc::clone(invoke);
                revokers.push(item.Click(move |_, _| invoke(label.clone()))?);
                item.cast()?
            }
            MenuItem::Separator { .. } => {
                MenuFlyoutSeparator::new().and_then(|separator| separator.cast())?
            }
            MenuItem::Radio {
                label,
                group,
                checked,
                ..
            } => {
                let item = RadioMenuFlyoutItem::new()?;
                let text: IMenuFlyoutItem = item.cast()?;
                text.SetText(label)?;
                let state: IRadioMenuFlyoutItem = item.cast()?;
                state.SetGroupName(group)?;
                state.SetIsChecked(*checked)?;
                let label = label.clone();
                let invoke = Rc::clone(invoke);
                revokers.push(text.Click(move |_, _| invoke(label.clone()))?);
                item.cast()?
            }
            MenuItem::Toggle { label, checked, .. } => {
                let item = ToggleMenuFlyoutItem::new()?;
                let text: IMenuFlyoutItem = item.cast()?;
                text.SetText(label)?;
                let state: IToggleMenuFlyoutItem = item.cast()?;
                state.SetIsChecked(*checked)?;
                let label = label.clone();
                let invoke = Rc::clone(invoke);
                revokers.push(text.Click(move |_, _| invoke(label.clone()))?);
                item.cast()?
            }
            MenuItem::Submenu { label, items, .. } => {
                let submenu = MenuFlyoutSubItem::new()?;
                submenu.SetText(label)?;
                build_native_menu_items(items, &submenu.Items()?, revokers, invoke)?;
                submenu.cast()?
            }
        };
        output.Append(&native)?;
    }
    Ok(())
}

fn set_rich_edit_text(control: &bindings::RichEditBox, value: &str) -> Result<bool, RuntimeError> {
    let read_only = control.IsReadOnly().map_err(native_error)?;
    let document = control.Document().map_err(native_error)?;
    let mut current = windows_core::HSTRING::new();
    document
        .GetText(TextGetOptions::UseLf, &mut current)
        .map_err(native_error)?;
    if current == value {
        return Ok(false);
    }
    if read_only {
        control.SetIsReadOnly(false).map_err(native_error)?;
    }
    let write = document
        .SetText(TextSetOptions::None, value)
        .map_err(native_error);
    let restore = if read_only {
        control.SetIsReadOnly(true).map_err(native_error)
    } else {
        Ok(())
    };
    write.and(restore).map(|_| true)
}

/// Applies a `"latitude,longitude"` center to a `MapControl`: moves the
/// camera and attaches a marker layer at the point, mirroring the previous
/// hand-written backend. A fresh marker layer is appended on every apply.
fn set_map_center(control: &bindings::MapControl, value: &str) -> Result<(), RuntimeError> {
    let (latitude, longitude) = value
        .split_once(',')
        .and_then(|(latitude, longitude)| {
            Some((
                latitude.trim().parse::<f64>().ok()?,
                longitude.trim().parse::<f64>().ok()?,
            ))
        })
        .ok_or(RuntimeError::UnsupportedKind)?;
    let position = BasicGeoposition {
        latitude,
        longitude,
        altitude: 0.0,
    };
    let point = Geopoint::Create(position).map_err(native_error)?;
    control.SetCenter(&point).map_err(native_error)?;
    let icon = MapIcon::new().map_err(native_error)?;
    icon.SetLocation(&point).map_err(native_error)?;
    let layer = MapElementsLayer::new().map_err(native_error)?;
    layer
        .MapElements()
        .map_err(native_error)?
        .Append(&icon)
        .map_err(native_error)?;
    control
        .Layers()
        .map_err(native_error)?
        .Append(&layer)
        .map_err(native_error)?;
    Ok(())
}

/// Applies a decode width to the current bitmap source of an `Image`:
/// downsamples at decode time (physical pixels) and re-sets the URI source
/// to retrigger the decode. A width of zero resets to natural-size decoding.
/// Missing sources, non-bitmap sources, and negative widths are a no-op.
fn apply_image_decode_width(control: &bindings::Image, width: i32) -> Result<(), RuntimeError> {
    if width < 0 {
        return Ok(());
    }
    let Ok(source) = control.Source() else {
        return Ok(());
    };
    let Ok(bitmap) = source.cast::<BitmapImage>() else {
        return Ok(());
    };
    bitmap
        .SetDecodePixelType(DecodePixelType::Physical)
        .map_err(native_error)?;
    bitmap.SetDecodePixelWidth(width).map_err(native_error)?;
    let uri = bitmap.UriSource().map_err(native_error)?;
    bitmap.SetUriSource(&uri).map_err(native_error)?;
    Ok(())
}

fn build_tree_node(definition: &TreeNode) -> Result<TreeViewNode, RuntimeError> {
    let node = TreeViewNode::new().map_err(native_error)?;
    let content: windows_core::IInspectable =
        windows_reference::IReference::from(windows_core::HSTRING::from(&definition.text)).into();
    node.SetContent(&content).map_err(native_error)?;
    node.SetIsExpanded(definition.expanded)
        .map_err(native_error)?;
    let children = node.Children().map_err(native_error)?;
    for child in &definition.children {
        let child = build_tree_node(child)?;
        children.Append(&child).map_err(native_error)?;
    }
    Ok(node)
}

fn window_visual_changes(
    previous: Option<WindowVisuals>,
    next: WindowVisuals,
) -> WindowVisualChanges {
    WindowVisualChanges {
        backdrop: previous.is_none_or(|previous| previous.backdrop != next.backdrop),
        backdrop_tuning: previous
            .is_none_or(|previous| previous.backdrop_tuning != next.backdrop_tuning),
        border: previous.is_none_or(|previous| previous.border != next.border),
        client_size: previous.is_none_or(|previous| previous.client_size != next.client_size),
        constraints: previous.is_none_or(|previous| previous.constraints != next.constraints),
        corner: previous.is_none_or(|previous| previous.corner != next.corner),
        icon: next.icon.is_some() && previous.is_none_or(|previous| previous.icon != next.icon),
        placement: previous.is_none_or(|previous| previous.placement != next.placement),
        presenter: previous.is_none_or(|previous| previous.presenter != next.presenter),
        theme: previous.is_none_or(|previous| previous.theme != next.theme),
    }
}

/// Maps the window theme onto the material's own theme.
///
/// The material is a composition layer of its own: it does not inherit
/// `RequestedTheme` from the XAML content, so a theme change has to reach it
/// through [`SystemBackdropConfiguration`] as well.
fn backdrop_theme(theme: WindowTheme) -> SystemBackdropTheme {
    match theme {
        WindowTheme::Light => SystemBackdropTheme::Light,
        WindowTheme::Dark => SystemBackdropTheme::Dark,
        // `Default` 让框架跟着内容根的 `RequestedTheme` 走（`apply_window_theme`
        // 把它设成 `ElementTheme::Default`）。系统明暗因此不需要这个库再轮询
        // 一次注册表——问一次系统的是 `SystemBackdropTheme::Default` 自己。
        WindowTheme::System => SystemBackdropTheme::Default,
    }
}

// ---------------------------------------------------------------------------
// 为什么这里重新用 Composition controllers
// ---------------------------------------------------------------------------
//
// XAML 的 `MicaBackdrop` / `DesktopAcrylicBackdrop` 在 WinUI 内部就是这两组
// controller 的薄封装：它们同样调 `SetKind` / `SetTintColor` / `SetTintOpacity`
// / `SetLuminosityOpacity`，同样把一个 `SystemBackdropConfiguration` 装到窗口
// 上。所以"控制器路径"不是另一条渲染管线，只是同一个管线的内层——而它的参数
// 是可下发的，XAML 那层没有对外的旋钮。放弃它等于放弃
// `WindowBackdropTuning` 的全部四项。
//
// 决策注释点名的那三个"手搓"部件，逐个处理如下。
//
// (1) **主题传播。** 材质是不继承 `RequestedTheme` 的独立合成层，明暗切换必须
//     单独下发一次。两个入口：
//     - 主题变了 → [`BackdropHost::apply_theme`] 就地改 configuration。configuration
//       是**常驻**的（随 controller 装上、随 controller 释放），所以这一下只是
//       重画，不重建任何东西。
//     - 参数变了 → 调用方重算 tint/luminosity 后通过 `backdrop_tuning` 传进来，
//       走 [`apply_window_backdrop`] 的就地下发。库**不猜**调用方的插值曲线，
//       那是调用方自己的数据来源；库负责的是"改了就一定落到已装的材质上"。
//
// (2) **input-activation。** 失焦窗口的材质要走失焦外观，这是这条路径上唯一
//     必须自己接的一根线：`Window::Activated` → `configuration.SetIsInputActive`。
//     这不算"手搓框架已有的东西"——XAML 那侧之所以不用写，是因为它在 C++ 里
//     订阅了同一个事件，这里只是把订阅搬到了能看见的表面上。`EventRevoker`
//     存在 [`BackdropHost`] 里，跟 controller 同生共死。
//
// (3) **材质自身的 opacity 默认值。** 做法是**根本不下发默认值**：
//     [`apply_material_tuning`] 对每个 setter 都只在调用方给了 `Some(_)` 时才调用，
//     `None` 就让 controller 自带的默认值留着（与 XAML 侧内部同一套：Mica/Acrylic
//     的 TintOpacity 0.8、LuminosityOpacity 1.0）。
//     决策注释里"深色下一块平板"的真正来源是**自己挑了默认值**——早期版本在
//     `Solid` 上写死 `TintOpacity(1.0)` + `LuminosityOpacity(0.0)`，那是把模糊层
//     和亮度层一起压掉，不是 controller 路径本身的问题。`Solid` 现在画在内容根上，
//     根本不装 controller，那条路也就不会回来了。

/// The material setters that [`IMicaController`] and
/// [`IDesktopAcrylicController`] have in common.
///
/// Both WinRT interfaces declare the same four setters with the same signatures,
/// but the generated bindings are two unrelated types. This trait exists so the
/// rule that matters — *forward a value only when the caller supplied one* — is
/// written down once, in [`apply_material_tuning`], instead of twice.
trait MaterialTuning {
    fn set_tint_opacity(&self, value: f32) -> windows_core::Result<()>;
    fn set_luminosity_opacity(&self, value: f32) -> windows_core::Result<()>;
    fn set_tint_color(&self, value: bindings::Color) -> windows_core::Result<()>;
    fn set_fallback_color(&self, value: bindings::Color) -> windows_core::Result<()>;
}

impl MaterialTuning for IMicaController {
    fn set_tint_opacity(&self, value: f32) -> windows_core::Result<()> {
        Self::SetTintOpacity(self, value)
    }

    fn set_luminosity_opacity(&self, value: f32) -> windows_core::Result<()> {
        Self::SetLuminosityOpacity(self, value)
    }

    fn set_tint_color(&self, value: bindings::Color) -> windows_core::Result<()> {
        Self::SetTintColor(self, value)
    }

    fn set_fallback_color(&self, value: bindings::Color) -> windows_core::Result<()> {
        Self::SetFallbackColor(self, value)
    }
}

impl MaterialTuning for IDesktopAcrylicController {
    fn set_tint_opacity(&self, value: f32) -> windows_core::Result<()> {
        Self::SetTintOpacity(self, value)
    }

    fn set_luminosity_opacity(&self, value: f32) -> windows_core::Result<()> {
        Self::SetLuminosityOpacity(self, value)
    }

    fn set_tint_color(&self, value: bindings::Color) -> windows_core::Result<()> {
        Self::SetTintColor(self, value)
    }

    fn set_fallback_color(&self, value: bindings::Color) -> windows_core::Result<()> {
        Self::SetFallbackColor(self, value)
    }
}

/// Forwards `tuning` onto the material's own setters.
///
/// **Only `Some(_)` fields are pushed.** A `None` field must leave the controller's
/// own default alone rather than be replaced by a value this crate made up — that
/// is the whole of note (3) above: `None` means "the system default", and the
/// system default is exactly what the XAML backdrop types would have used.
///
/// `SetKind` is deliberately *not* applied here. It has to run before the tint
/// setters (setting the kind resets them to that kind's defaults) and it only
/// changes when the declared material changes, so it belongs to
/// [`create_backdrop_controller`].
fn apply_material_tuning<T: MaterialTuning + ?Sized>(
    controller: &T,
    tuning: WindowBackdropTuning,
) -> Result<(), RuntimeError> {
    if let Some(value) = tuning.tint_opacity {
        controller
            .set_tint_opacity(value as f32)
            .map_err(native_error)?;
    }
    if let Some(value) = tuning.luminosity_opacity {
        controller
            .set_luminosity_opacity(value as f32)
            .map_err(native_error)?;
    }
    if let Some(color) = tuning.tint_color {
        controller
            .set_tint_color(native_color(color))
            .map_err(native_error)?;
    }
    if let Some(color) = tuning.fallback_color {
        controller
            .set_fallback_color(native_color(color))
            .map_err(native_error)?;
    }
    Ok(())
}

/// An installed material controller, typed so parameter changes can be pushed
/// onto it without rebuilding the composition layer.
///
/// Holding a typed interface here is not redundant with
/// [`BackdropHost::controller`]: the `IUnknown` field documents the lifetime
/// rule ("releasing the controller tears the material out"), this one exists to
/// make [`BackdropMaterial::apply_tuning`] callable.
enum BackdropMaterial {
    Mica(IMicaController),
    Acrylic(IDesktopAcrylicController),
}

impl BackdropMaterial {
    /// Selects the material's `Kind`.
    ///
    /// `SetKind` lives on a late-added interface (`IMicaController2` /
    /// `IDesktopAcrylicController3`), so the cast can fail on older systems. That
    /// is **not** fatal: the material still renders, it just keeps the base kind.
    /// Failing the whole call instead would trade one missing distinction for a
    /// window with no background at all, so the failure is logged and swallowed.
    ///
    /// Must run before the tint setters — `SetKind` resets them to the defaults of
    /// the kind it selects.
    fn set_kind(&self, backdrop: WindowBackdrop) -> Result<(), RuntimeError> {
        let result = match self {
            Self::Mica(controller) => controller
                .cast::<IMicaController2>()
                .and_then(|kind| kind.SetKind(mica_kind(backdrop))),
            Self::Acrylic(controller) => controller
                .cast::<IDesktopAcrylicController3>()
                .and_then(|kind| kind.SetKind(acrylic_kind(backdrop))),
        };
        if let Err(error) = result {
            eprintln!(
                "windows-reactor: backdrop `SetKind` rejected: {error}; \
                 the material falls back to its base kind"
            );
        }
        Ok(())
    }

    /// Pushes new parameters onto the already-installed material.
    fn apply_tuning(&self, tuning: WindowBackdropTuning) -> Result<(), RuntimeError> {
        match self {
            Self::Mica(controller) => apply_material_tuning(controller, tuning),
            Self::Acrylic(controller) => apply_material_tuning(controller, tuning),
        }
    }
}

fn mica_kind(backdrop: WindowBackdrop) -> MicaKind {
    match backdrop {
        WindowBackdrop::MicaAlt => MicaKind::BaseAlt,
        _ => MicaKind::Base,
    }
}

fn acrylic_kind(backdrop: WindowBackdrop) -> DesktopAcrylicKind {
    match backdrop {
        WindowBackdrop::AcrylicThin => DesktopAcrylicKind::Thin,
        _ => DesktopAcrylicKind::Base,
    }
}

/// A freshly built controller, before it is attached to a window.
struct BackdropController {
    /// Keeps the controller object alive; releasing it tears the material out.
    controller: windows_core::IUnknown,
    targets: ISystemBackdropControllerWithTargets,
    /// Typed view of `controller`, used to push later parameter changes in place.
    material: BackdropMaterial,
}

/// Live backdrop state for one window.
///
/// The controller has to outlive the call that installs it — dropping it
/// detaches the material — so [`BackdropHost`] keeps it for the window's
/// lifetime and pushes parameter changes onto the installed controller rather
/// than building a new one. The theme is the exception: it lives on
/// `SystemBackdropConfiguration`, which is also installed for the window's lifetime.
struct BackdropHost {
    /// Keeps the controller object alive; releasing it tears the material out.
    controller: Option<windows_core::IUnknown>,
    targets: Option<ISystemBackdropControllerWithTargets>,
    activated: Option<windows_core::EventRevoker>,
    /// Live material configuration. `SetTheme` on it repaints the installed material without
    /// rebuilding the controller.
    configuration: Option<SystemBackdropConfiguration>,
    /// Typed controller, so a `backdrop_tuning` change does not rebuild the layer.
    material: Option<BackdropMaterial>,
    /// Which material `material` holds. `None` when no material is installed.
    backdrop: Option<WindowBackdrop>,
}

impl BackdropHost {
    /// Re-pushes the material's theme onto the live configuration.
    ///
    /// This is note (1) of the block comment above: the material does not inherit
    /// `RequestedTheme` from the content root, so the window's theme change has to
    /// reach it separately. Repainting is all it takes — the configuration stays
    /// installed for the window's lifetime.
    fn apply_theme(&self, theme: WindowTheme) -> Result<(), RuntimeError> {
        let Some(configuration) = self.configuration.as_ref() else {
            return Ok(());
        };
        configuration
            .SetTheme(backdrop_theme(theme))
            .map_err(native_error)
    }

    /// Releases the COM references **while the XAML runtime is still alive**.
    ///
    /// The timing is the whole point. On the process-exit path WinUI has already
    /// been torn down, and `Release`-ing a backdrop controller there dereferences
    /// a dead runtime — measured as an access violation with `IUnknown::drop` on
    /// the stack. Every path that can run during a live session goes through here;
    /// [`Drop`] deliberately leaks instead.
    fn release(&mut self) {
        if let Some(activated) = self.activated.take() {
            activated.into_token();
        }
        self.configuration = None;
        self.material = None;
        self.backdrop = None;
        self.targets = None;
        self.controller = None;
    }
}

impl Drop for BackdropHost {
    fn drop(&mut self) {
        // Reached only on the exit path, where releasing would fault (see
        // `release`). A process on its way out has no use for accurate COM
        // reference counts, and leaking beats crashing on the way out.
        std::mem::forget(self.activated.take());
        std::mem::forget(self.configuration.take());
        std::mem::forget(self.material.take());
        std::mem::forget(self.targets.take());
        std::mem::forget(self.controller.take());
    }
}

/// Installs (or removes) the window material.
///
/// Returns `Ok(None)` when the window should carry no *system* material: the
/// caller asked for [`WindowBackdrop::None`] or [`WindowBackdrop::Solid`] (both
/// paint the content root instead), or this system cannot render the requested
/// material (in which case [`WindowBackdropTuning::fallback_color`] is painted
/// in its place). Otherwise it returns the [`BackdropHost`] holding the
/// controller that has to stay alive.
fn apply_window_backdrop(
    window: &Window,
    visuals: WindowVisuals,
    previous: Option<BackdropHost>,
) -> Result<Option<BackdropHost>, RuntimeError> {
    let window_2 = window.cast::<IWindow2>().map_err(native_error)?;

    // The XAML `SystemBackdrop` type and a Composition controller both drive the
    // same surface; holding both paints the material twice. Clearing it up front
    // also means `None` / `Solid` do not have to remember to clear it themselves.
    window_2
        .SetSystemBackdrop(None::<&SystemBackdrop>)
        .map_err(native_error)?;

    // Wipe whatever layer is on the content root: the `Solid` colour of a previous
    // declaration, or the `fallback_color` painted for a material this system
    // could not render. Leaving either behind would cover the material chosen next.
    clear_content_backdrop(window)?;

    // The window object is itself the composition target a material attaches to.
    // Only the material path needs it, so a failure here is not fatal yet — it is
    // reported below, once we know a material was actually requested.
    let target = window.cast::<ICompositionSupportsSystemBackdrop>().ok();

    // Parameters alone do not need a new controller: the installed material is a
    // live composition target, so pushing the new values onto it repaints without
    // detaching and re-attaching the layer. Rebuilding here would make every frame
    // of a slider drag flicker. A changed material, or no previous material, still
    // goes through the full path below.
    let mut previous = previous;
    if let Some(host) = previous.as_ref()
        && host.backdrop == Some(visuals.backdrop)
        && let Some(material) = host.material.as_ref()
    {
        // A failure here is released rather than left to `Drop`, which deliberately
        // leaks: this runs inside a live session, where the COM references *can*
        // go, and a dropped-on-error host would take the material with it anyway.
        if let Err(error) = material.apply_tuning(visuals.backdrop_tuning) {
            if let Some(mut host) = previous.take() {
                host.release();
            }
            return Err(error);
        }
        return Ok(previous);
    }

    if let Some(previous) = previous.take() {
        // Detach the outgoing target *before* releasing: the window's composition
        // layer outlives the controller handle, and leaving the target attached
        // keeps it rendering into a released controller.
        if let (Some(targets), Some(target)) = (previous.targets.as_ref(), target.as_ref()) {
            let _ = targets.RemoveSystemBackdropTarget(target);
        }
        let mut previous = previous;
        // Released here rather than left to `Drop`, which deliberately leaks: this
        // runs inside a live UI session, so the COM references can go immediately
        // instead of accumulating one per rebuilt material.
        previous.release();
    }

    // `None` and `Solid` paint the content root rather than a system material.
    if matches!(
        visuals.backdrop,
        WindowBackdrop::None | WindowBackdrop::Solid
    ) {
        apply_solid_backdrop(window, visuals, visuals.backdrop == WindowBackdrop::Solid)?;
        return Ok(None);
    }

    let Some(target) = target else {
        eprintln!(
            "windows-reactor: this window is not a system backdrop target; \
             painting the fallback colour instead of {:?}",
            visuals.backdrop
        );
        apply_backdrop_fallback(window, visuals.backdrop_tuning)?;
        return Ok(None);
    };

    let Some(built) = create_backdrop_controller(visuals.backdrop, visuals.backdrop_tuning)? else {
        // This system cannot render the requested material at all.
        apply_backdrop_fallback(window, visuals.backdrop_tuning)?;
        return Ok(None);
    };
    let BackdropController {
        controller,
        targets,
        material,
    } = built;

    let configuration = SystemBackdropConfiguration::new().map_err(native_error)?;
    configuration
        .SetTheme(backdrop_theme(visuals.theme))
        .map_err(native_error)?;
    // Note (2): an inactive window renders its material in the lost-focus
    // appearance. `Activated` is the only feed for that flag on this path.
    configuration.SetIsInputActive(true).map_err(native_error)?;

    targets
        .AddSystemBackdropTarget(&target)
        .map_err(native_error)?;
    targets
        .SetSystemBackdropConfiguration(&configuration)
        .map_err(native_error)?;

    let live_configuration = configuration.clone();
    let activated = window
        .Activated(move |_, args| {
            if let Some(args) = args.as_ref()
                && let Ok(state) = args.WindowActivationState()
            {
                let _ = live_configuration
                    .SetIsInputActive(state != WindowActivationState::Deactivated);
            }
        })
        .map_err(native_error)?;

    Ok(Some(BackdropHost {
        controller: Some(controller),
        targets: Some(targets),
        activated: Some(activated),
        configuration: Some(configuration),
        material: Some(material),
        backdrop: Some(visuals.backdrop),
    }))
}

/// Builds the Composition controller for `backdrop` and forwards `tuning` onto it.
///
/// `Ok(None)` means this system cannot render the material; the caller then paints
/// [`WindowBackdropTuning::fallback_color`] instead. The capability probe uses the
/// same predicate as the public [`backdrop_is_supported`] — and swallows a failing
/// probe as "unsupported" rather than failing the whole visuals pass, because the
/// degradation to a flat colour is strictly better than an error that leaves the
/// window with nothing painted underneath its content.
fn create_backdrop_controller(
    backdrop: WindowBackdrop,
    tuning: WindowBackdropTuning,
) -> Result<Option<BackdropController>, RuntimeError> {
    let (controller, material): (windows_core::IUnknown, BackdropMaterial) = match backdrop {
        WindowBackdrop::Mica | WindowBackdrop::MicaAlt => {
            if !MicaController::IsSupported().unwrap_or(false) {
                return Ok(None);
            }
            let controller = MicaController::new().map_err(native_error)?;
            let mica: IMicaController = controller.cast().map_err(native_error)?;
            let material = BackdropMaterial::Mica(mica.clone());
            // `Kind` before the tint setters: selecting a kind resets them.
            material.set_kind(backdrop)?;
            apply_material_tuning(&mica, tuning)?;
            (mica.cast().map_err(native_error)?, material)
        }
        WindowBackdrop::Acrylic | WindowBackdrop::AcrylicThin => {
            if !DesktopAcrylicController::IsSupported().unwrap_or(false) {
                return Ok(None);
            }
            let controller = DesktopAcrylicController::new().map_err(native_error)?;
            let acrylic: IDesktopAcrylicController = controller.cast().map_err(native_error)?;
            let material = BackdropMaterial::Acrylic(acrylic.clone());
            material.set_kind(backdrop)?;
            apply_material_tuning(&acrylic, tuning)?;
            (acrylic.cast().map_err(native_error)?, material)
        }
        // `None` and `Solid` paint the content root; they never build a controller.
        WindowBackdrop::None | WindowBackdrop::Solid => return Ok(None),
    };

    let targets: ISystemBackdropControllerWithTargets = controller.cast().map_err(native_error)?;
    Ok(Some(BackdropController {
        controller,
        targets,
        material,
    }))
}

/// 系统渲染不了请求的材质时，用 `fallback_color` 铺一层纯色。
///
/// `WindowBackdropTuning::fallback_color` 的文档承诺它会在"the system cannot render
/// the requested material"时被画出来。装不上材质有三条路——控制器的
/// `IsSupported()` 为假、窗口拿不到 backdrop target、材质本身建不出来——三条都
/// 收敛到这里。
///
/// 铺在窗口内容根上（`Window.Content` 的 `Panel` 背景）：材质画在窗口最底层，
/// 内容根的不透明背景正好遮住整块客户区，观感就是一块纯色底。只在这条早退
/// 路径上写，材质可用时绝不碰应用自己的背景；换走材质时由
/// [`clear_content_backdrop`] 擦掉。
fn apply_backdrop_fallback(
    window: &Window,
    tuning: WindowBackdropTuning,
) -> Result<(), RuntimeError> {
    let Some(color) = tuning.fallback_color else {
        // 调用方没提供回落色 → 保持"什么都不画"，底色由应用自己负责。
        return Ok(());
    };
    // 全不透明，不套 `tint_opacity`：`fallback_color` 的契约是"顶替整块材质"，
    // 半透明的话露出来的是窗口自己的底色，而不是这块回落色。
    set_content_background(window, Some(color))
}

/// 当前系统能否渲染指定的背景材质。
///
/// 应用需要它来回答一个容易漏掉的耦合问题：**内容区要不要让材质透出来**。
/// 材质不可用时窗口底层只有 [`WindowBackdropTuning::fallback_color`] 那一层纯色
/// （甚至什么都没有），此时把内容调成透明得到的是"整块看不见东西"，而不是"没有材质效果"。
///
/// 会调用系统 API，**不要在每帧渲染里调**——启动时或材质变更时各查一次即可。
pub fn backdrop_is_supported(backdrop: WindowBackdrop) -> bool {
    match backdrop {
        WindowBackdrop::Mica | WindowBackdrop::MicaAlt => {
            MicaController::IsSupported().unwrap_or(false)
        }
        WindowBackdrop::Acrylic | WindowBackdrop::AcrylicThin | WindowBackdrop::Solid => {
            DesktopAcrylicController::IsSupported().unwrap_or(false)
        }
        // "不画材质"永远可用——它本来就不需要系统能力。
        WindowBackdrop::None => true,
    }
}

/// 把 [`WindowBackdrop::Solid`] 画成内容根上的一层不透明纯色。
///
/// 纯色不采样桌面，因此不需要材质的模糊层与亮度层——之前在
/// `DesktopAcrylicController` 上用 `TintOpacity(1.0)` 压住那些层，
/// 在深色主题下会画出一块亮板。画在内容根上则由主题直接决定底色。
///
/// 这一层是**不属于**应用自己的内容背景，所以换走 Solid 时必须由
/// [`clear_content_backdrop`] 擦掉，否则它会永久盖住之后选择的任何材质。
fn apply_solid_backdrop(
    window: &Window,
    visuals: WindowVisuals,
    use_tuning: bool,
) -> Result<(), RuntimeError> {
    let color = match (use_tuning, visuals.backdrop_tuning.tint_color) {
        (true, Some(color)) => color,
        _ => solid_backdrop_default(visuals.theme),
    };
    let opacity = if use_tuning {
        visuals.backdrop_tuning.tint_opacity.unwrap_or(1.0)
    } else {
        1.0
    };
    set_content_background(window, Some(apply_opacity(color, opacity)))
}

/// 擦掉 [`apply_solid_backdrop`] 写在内容根上的那层纯色。
fn clear_content_backdrop(window: &Window) -> Result<(), RuntimeError> {
    set_content_background(window, None)
}

fn set_content_background(
    window: &Window,
    color: Option<ReactorColor>,
) -> Result<(), RuntimeError> {
    // 内容根不是 Panel（或窗口还没挂内容）时静默跳过：这里失败不该阻断
    // 整个视觉应用流程。
    let Ok(content) = window.Content() else {
        return Ok(());
    };
    let Ok(panel) = content.cast::<Panel>() else {
        return Ok(());
    };
    match color {
        Some(color) => {
            let brush = SolidColorBrush::new().map_err(native_error)?;
            brush.SetColor(native_color(color)).map_err(native_error)?;
            panel.SetBackground(&brush).map_err(native_error)?;
        }
        None => {
            panel
                .SetBackground(None::<&bindings::Brush>)
                .map_err(native_error)?;
        }
    }
    Ok(())
}

/// 未指定 [`WindowBackdropTuning::tint_color`] 时的纯色底，随主题切换。
fn solid_backdrop_default(theme: WindowTheme) -> ReactorColor {
    match theme {
        WindowTheme::Light => ReactorColor {
            a: 255,
            r: 0xF3,
            g: 0xF3,
            b: 0xF3,
        },
        _ => ReactorColor {
            a: 255,
            r: 0x20,
            g: 0x20,
            b: 0x20,
        },
    }
}

fn apply_opacity(color: ReactorColor, opacity: f64) -> ReactorColor {
    ReactorColor {
        a: (f64::from(color.a) * opacity.clamp(0.0, 1.0)) as u8,
        ..color
    }
}

fn native_color(color: ReactorColor) -> bindings::Color {
    bindings::Color {
        a: color.a,
        r: color.r,
        g: color.g,
        b: color.b,
    }
}

/// Applies the window outline and corner rounding through DWM.
///
/// Both are recorded on the window itself (`DWMWINDOWATTRIBUTE` is window state,
/// not per-frame state), so they survive until the next change.
/// `SetWindowPos` 的 `uFlags`。只用到这三个：保留尺寸、保留层序、不激活——
/// 一个"恢复上次窗口位置"的声明不该顺手改变尺寸、层级或焦点。
///
/// 定义在这里而不是 `bindings.rs`：那个文件整体由 `windows_bindgen` 重写
/// （输入是 `crates/tools/reactor/src/bindings.txt`），手写的定义会在下次重新
/// 生成时消失。
const SWP_NOSIZE: u32 = 0x0001;
const SWP_NOZORDER: u32 = 0x0004;
const SWP_NOACTIVATE: u32 = 0x0010;

fn apply_window_frame(hwnd: HWND, visuals: WindowVisuals) -> Result<(), RuntimeError> {
    let corner = match visuals.corner {
        WindowCorner::Default => None,
        WindowCorner::DoNotRound => Some(DWMWCP_DONOTROUND),
        WindowCorner::RoundSmall => Some(DWMWCP_ROUNDSMALL),
        WindowCorner::Round => Some(DWMWCP_ROUND),
    };
    let border = match visuals.border {
        WindowBorder::Default => None,
        // `DWMWA_COLOR_NONE` is DWM's documented "no outline" sentinel. An
        // application drawing its own outline wants exactly this: the system
        // outline does not share geometry with a self-drawn one, and the two
        // together read as a doubled hairline around the corners.
        WindowBorder::Hidden => Some(0xFFFF_FFFE_u32),
        WindowBorder::Color(color) => Some(
            // DWM takes a `COLORREF`, whose high byte must be zero: an alpha
            // there is rejected outright (`E_INVALIDARG`) rather than ignored, so
            // the system outline cannot be drawn partially transparent. An
            // application that wants a soft hairline draws its own and asks for
            // `Hidden` here.
            (u32::from(color.r) << 16) | (u32::from(color.g) << 8) | u32::from(color.b),
        ),
    };

    unsafe {
        if let Some(corner) = corner {
            let value = corner;
            // A rejection here is not fatal: corner rounding is decoration, and
            // DWM answers `E_INVALIDARG` on any system that predates the
            // attribute. Losing it must not take the window down.
            if let Err(error) = DwmSetWindowAttribute(
                hwnd,
                DWMWA_WINDOW_CORNER_PREFERENCE as u32,
                std::ptr::from_ref(&value).cast(),
                size_of_val(&value) as u32,
            )
            .ok()
            {
                eprintln!("windows-reactor: DWMWA_WINDOW_CORNER_PREFERENCE rejected: {error}");
            }
        }
        if let Some(border) = border {
            let value = border;
            if let Err(error) = DwmSetWindowAttribute(
                hwnd,
                DWMWA_BORDER_COLOR as u32,
                std::ptr::from_ref(&value).cast(),
                size_of_val(&value) as u32,
            )
            .ok()
            {
                eprintln!("windows-reactor: DWMWA_BORDER_COLOR rejected: {error}");
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod window_visual_tests {
    use super::*;

    #[test]
    fn theme_change_does_not_reapply_client_size_or_backdrop() {
        let previous = WindowVisuals::new()
            .theme(WindowTheme::System)
            .backdrop(WindowBackdrop::Mica)
            .client_size(1400.0, 900.0)
            .centered();
        let next = WindowVisuals::new()
            .theme(WindowTheme::Dark)
            .backdrop(WindowBackdrop::Mica)
            .client_size(1400.0, 900.0)
            .centered();

        assert_eq!(
            window_visual_changes(Some(previous), next),
            WindowVisualChanges {
                backdrop: false,
                backdrop_tuning: false,
                border: false,
                client_size: false,
                constraints: false,
                corner: false,
                icon: false,
                placement: false,
                presenter: false,
                theme: true,
            }
        );
    }

    #[test]
    fn acrylic_thin_and_acrylic_request_distinct_kinds() {
        // `AcrylicThin` used to resolve to the same XAML backdrop as `Acrylic`,
        // so the thin preset did not exist pixel-wise. It has to stay distinct.
        assert_ne!(
            acrylic_kind(WindowBackdrop::AcrylicThin),
            acrylic_kind(WindowBackdrop::Acrylic)
        );
        assert_eq!(
            acrylic_kind(WindowBackdrop::Acrylic),
            DesktopAcrylicKind::Base
        );
        assert_eq!(
            acrylic_kind(WindowBackdrop::AcrylicThin),
            DesktopAcrylicKind::Thin
        );
        assert_eq!(mica_kind(WindowBackdrop::Mica), MicaKind::Base);
        assert_eq!(mica_kind(WindowBackdrop::MicaAlt), MicaKind::BaseAlt);
    }

    #[test]
    fn constraints_change_independently_and_can_be_cleared() {
        let constrained =
            WindowVisuals::new()
                .client_size(800.0, 600.0)
                .constraints(WindowConstraints {
                    min_width: Some(400.0),
                    ..Default::default()
                });
        let resized =
            WindowVisuals::new()
                .client_size(900.0, 700.0)
                .constraints(WindowConstraints {
                    min_width: Some(400.0),
                    ..Default::default()
                });
        let cleared = WindowVisuals::new().client_size(900.0, 700.0);

        assert_eq!(
            window_visual_changes(Some(constrained), resized),
            WindowVisualChanges {
                backdrop: false,
                backdrop_tuning: false,
                border: false,
                client_size: true,
                constraints: false,
                corner: false,
                icon: false,
                placement: false,
                presenter: false,
                theme: false,
            }
        );
        assert_eq!(
            window_visual_changes(Some(resized), cleared),
            WindowVisualChanges {
                backdrop: false,
                backdrop_tuning: false,
                border: false,
                client_size: false,
                constraints: true,
                corner: false,
                icon: false,
                placement: false,
                presenter: false,
                theme: false,
            }
        );
    }

    /// 位置只在声明值变化时下发一次：每帧重声明同一个位置（居中或坐标）都不会跟用户拖动
    /// 窗口较劲；改成另一种位置声明时则必须重新下发。
    #[test]
    fn placement_applies_only_when_the_declaration_changes() {
        let centered = WindowVisuals::new().client_size(900.0, 700.0).centered();
        let position = WindowVisuals::new()
            .client_size(900.0, 700.0)
            .client_position(120.0, 80.0);
        let moved = WindowVisuals::new()
            .client_size(900.0, 700.0)
            .client_position(300.0, 200.0);
        let silent = WindowVisuals::new().client_size(900.0, 700.0);

        let placement = |previous, next| window_visual_changes(Some(previous), next).placement;
        assert!(placement(WindowVisuals::default(), centered));
        assert!(!placement(centered, centered));
        assert!(placement(centered, position));
        assert!(placement(position, moved));
        assert!(placement(position, silent));
        assert!(!placement(silent, silent));
    }

    /// 位置与居中互斥：两者写同一个声明，后调用的那个生效。否则"既给了坐标又要求居中"的
    /// 声明没有任何合理语义。
    #[test]
    fn later_placement_declaration_wins() {
        let centered_last = WindowVisuals::new().client_position(10.0, 20.0).centered();
        let position_last = WindowVisuals::new().centered().client_position(10.0, 20.0);

        assert_eq!(centered_last.placement, WindowPlacement::Centered);
        assert_eq!(
            position_last.placement,
            WindowPlacement::Position { x: 10.0, y: 20.0 }
        );
    }
}

impl WinUiRuntime {
    pub fn with_application(application: Application) -> Self {
        Self {
            pending_application: Some(application),
            ..Default::default()
        }
    }

    #[cfg(feature = "test")]
    pub fn live_select_list_box_item(&self, node: NodeId, index: u32) -> Result<(), RuntimeError> {
        let Some(handle @ Handle::ListBox(_)) = self.handles.get(&node) else {
            return Err(RuntimeError::UnsupportedKind);
        };
        let item = slot_collection(handle, SlotId::ListBoxItems)?.GetAt(index)?;
        set_selected_item(
            handle,
            selection_for_slot(SlotId::ListBoxItems).unwrap(),
            &item,
        )
    }

    #[cfg(feature = "test")]
    pub fn live_range_value(&self, node: NodeId) -> Result<f64, RuntimeError> {
        match self.handles.get(&node) {
            Some(Handle::NumberBox(control)) => control.Value().map_err(native_error),
            Some(Handle::Slider(control)) => control
                .cast::<IRangeBase>()
                .and_then(|control| control.Value())
                .map_err(native_error),
            _ => Err(RuntimeError::UnsupportedKind),
        }
    }

    #[cfg(feature = "test")]
    pub fn live_checked_value(&self, node: NodeId) -> Result<bool, RuntimeError> {
        match self.handles.get(&node) {
            Some(Handle::CheckBox(control)) => control
                .cast::<IToggleButton>()
                .and_then(|control| control.IsChecked())
                .map_err(native_error),
            Some(Handle::ToggleButton(control)) => control.IsChecked().map_err(native_error),
            _ => Err(RuntimeError::UnsupportedKind),
        }
    }

    #[cfg(feature = "test")]
    pub fn live_set_checked(&self, node: NodeId, value: bool) -> Result<(), RuntimeError> {
        let property = match self.handles.get(&node).map(Handle::kind) {
            Some(MountedKind::CheckBox) => PropertyId::CheckBoxIsChecked,
            Some(MountedKind::ToggleButton) => PropertyId::ToggleButtonIsChecked,
            _ => return Err(RuntimeError::UnsupportedKind),
        };
        self.live_write_property(node, property, &PropertyValue::Bool(value))
    }

    #[cfg(feature = "test")]
    fn live_write_property(
        &self,
        node: NodeId,
        property: PropertyId,
        value: &PropertyValue,
    ) -> Result<(), RuntimeError> {
        let handle = self
            .handles
            .get(&node)
            .ok_or(RuntimeError::MissingNode(node))?;
        set_property(handle, property, value)
    }

    #[cfg(feature = "test")]
    pub(crate) fn live_write_test_property(
        &self,
        node: NodeId,
        property: PropertyId,
        value: &PropertyValue,
    ) -> Result<(), RuntimeError> {
        self.live_write_property(node, property, value)
    }

    #[cfg(feature = "test")]
    pub(crate) fn live_set_test_date(
        &self,
        node: NodeId,
        value: DateTime,
    ) -> Result<(), RuntimeError> {
        match self.handles.get(&node) {
            Some(Handle::DatePicker(control)) => {
                control.SetSelectedDate(Some(value)).map_err(native_error)
            }
            Some(Handle::CalendarDatePicker(control)) => {
                control.SetDate(Some(value)).map_err(native_error)
            }
            _ => Err(RuntimeError::UnsupportedKind),
        }
    }

    #[cfg(feature = "test")]
    pub(crate) fn live_set_test_time(
        &self,
        node: NodeId,
        value: TimeSpan,
    ) -> Result<(), RuntimeError> {
        let Some(Handle::TimePicker(control)) = self.handles.get(&node) else {
            return Err(RuntimeError::UnsupportedKind);
        };
        control.SetSelectedTime(Some(value)).map_err(native_error)
    }

    #[cfg(feature = "test")]
    pub fn take_live_native_apply_times(&mut self) -> Vec<f64> {
        std::mem::take(&mut self.native_apply_times_us)
    }

    #[cfg(feature = "test")]
    pub fn clear_live_native_apply_times(&mut self) {
        self.native_apply_times_us.clear();
    }

    #[cfg(feature = "test")]
    pub fn live_event_subscription_count(&self) -> usize {
        self.subscriptions.len() + self.content_dialogs.borrow().subscription_count()
    }

    #[cfg(feature = "test")]
    pub(crate) fn live_content_dialog_states(&self) -> Vec<LiveContentDialogState> {
        self.content_dialogs.borrow().states()
    }

    #[cfg(feature = "test")]
    pub(crate) fn live_hide_content_dialog(&self, node: NodeId) -> Result<(), RuntimeError> {
        let dialog = self.content_dialogs.borrow().dialog(node)?;
        dialog.Hide().map_err(native_error)
    }

    #[cfg(feature = "test")]
    pub fn live_bring_virtual_index(&self, index: usize) -> Result<(), RuntimeError> {
        let virtual_handle = self
            .virtuals
            .values()
            .next()
            .ok_or(RuntimeError::UnsupportedKind)?;
        let index = i32::try_from(index).map_err(|_| RuntimeError::IndexOutOfBounds)?;
        virtual_handle
            .repeater
            .GetOrCreateElement(index)
            .and_then(|element| element.StartBringIntoView())
            .map_err(native_error)
    }

    #[cfg(feature = "test")]
    pub fn live_virtual_shell_counts(&self) -> Result<(usize, usize), RuntimeError> {
        let mut live = 0;
        let mut retired = 0;
        for handle in self.virtuals.values() {
            let counts = handle.shell_counts();
            live += counts.0;
            retired += counts.1;
        }
        if self.virtuals.is_empty() {
            Err(RuntimeError::UnsupportedKind)
        } else {
            Ok((live, retired))
        }
    }

    fn apply_window_visuals(
        &mut self,
        node: NodeId,
        visuals: WindowVisuals,
    ) -> Result<(), RuntimeError> {
        let window = self
            .windows
            .get(&node)
            .ok_or(RuntimeError::MissingNode(node))?;
        let window_2 = window.cast::<IWindow2>().map_err(native_error)?;
        let changes = window_visual_changes(self.window_visuals.get(&node).copied(), visuals);
        let mut hwnd: Option<HWND> = None;
        let mut window_handle = || -> Result<HWND, RuntimeError> {
            if let Some(handle) = hwnd {
                return Ok(handle);
            }
            let mut value = std::ptr::null_mut();
            unsafe {
                window
                    .cast::<IWindowNative>()
                    .map_err(native_error)?
                    .WindowHandle(&mut value)
                    .ok()
                    .map_err(native_error)?;
            }
            // `IWindowNative::WindowHandle` hands back a raw `c_void*`, while the 2.5.1 bindings
            // type `HWND` as `*mut HWND__`. Both name the same OS handle.
            let handle: HWND = value.cast();
            hwnd = Some(handle);
            Ok(handle)
        };

        if changes.backdrop || changes.backdrop_tuning {
            let backdrop_window = self
                .windows
                .get(&node)
                .cloned()
                .ok_or(RuntimeError::MissingNode(node))?;
            let previous = self.window_backdrops.borrow_mut().remove(&node);
            if let Some(host) = apply_window_backdrop(&backdrop_window, visuals, previous)? {
                self.window_backdrops.borrow_mut().insert(node, host);
            }
        }

        if changes.corner || changes.border {
            apply_window_frame(window_handle()?, visuals)?;
        }

        if changes.theme {
            let title_bar = window_2
                .AppWindow()
                .and_then(|window| window.TitleBar())
                .and_then(|title_bar| title_bar.cast::<IAppWindowTitleBar3>())
                .map_err(native_error)?;
            title_bar
                .SetPreferredTheme(match visuals.theme {
                    WindowTheme::System => TitleBarTheme::UseDefaultAppMode,
                    WindowTheme::Light => TitleBarTheme::Light,
                    WindowTheme::Dark => TitleBarTheme::Dark,
                })
                .map_err(native_error)?;

            if let Some(host) = self.window_hosts.get(&node) {
                Self::apply_window_theme(host, visuals.theme)?;
            }

            // `None` and `Solid` are painted on the content root rather than by a
            // system material, so nothing else repaints them when the theme
            // changes; without this they would keep the previous theme's colour.
            if matches!(
                visuals.backdrop,
                WindowBackdrop::None | WindowBackdrop::Solid
            ) {
                let backdrop_window = self
                    .windows
                    .get(&node)
                    .cloned()
                    .ok_or(RuntimeError::MissingNode(node))?;
                apply_solid_backdrop(
                    &backdrop_window,
                    visuals,
                    visuals.backdrop == WindowBackdrop::Solid,
                )?;
            }

            // A material is its own composition layer and does not inherit
            // `RequestedTheme` from the content root, so the window's theme change
            // has to reach it as well — otherwise the material keeps the old
            // theme while everything else in the window has already repainted.
            // The configuration is installed for the window's lifetime, so this
            // repaints the material instead of rebuilding the controller.
            if let Some(host) = self.window_backdrops.borrow().get(&node) {
                host.apply_theme(visuals.theme)?;
            }
        }

        if changes.icon
            && let Some(path) = visuals.icon
        {
            window_2
                .AppWindow()
                .and_then(|window| window.SetIcon(path))
                .map_err(native_error)?;
        }

        if changes.constraints {
            let app_window = window_2.AppWindow().map_err(native_error)?;
            let presenter = app_window
                .Presenter()
                .and_then(|presenter| presenter.cast::<IOverlappedPresenter3>())
                .map_err(native_error)?;
            let (min_width, min_height, max_width, max_height) =
                if let Some(constraints) = visuals.constraints {
                    let hwnd = window_handle()?;
                    let dpi = unsafe { GetDpiForWindow(hwnd.cast()) }.max(96);
                    let pixels = |dips: f64| (dips * f64::from(dpi) / 96.0).round() as i32;
                    let client_window = app_window.cast::<IAppWindow2>().map_err(native_error)?;
                    let outer = app_window.Size().map_err(native_error)?;
                    let inner = client_window.ClientSize().map_err(native_error)?;
                    let non_client_width = outer.width.saturating_sub(inner.width);
                    let non_client_height = outer.height.saturating_sub(inner.height);
                    (
                        constraints
                            .min_width
                            .map(|value| pixels(value).saturating_add(non_client_width)),
                        constraints
                            .min_height
                            .map(|value| pixels(value).saturating_add(non_client_height)),
                        constraints
                            .max_width
                            .map(|value| pixels(value).saturating_add(non_client_width)),
                        constraints
                            .max_height
                            .map(|value| pixels(value).saturating_add(non_client_height)),
                    )
                } else {
                    (None, None, None, None)
                };
            presenter
                .SetPreferredMinimumWidth(min_width)
                .and_then(|()| presenter.SetPreferredMinimumHeight(min_height))
                .and_then(|()| presenter.SetPreferredMaximumWidth(max_width))
                .and_then(|()| presenter.SetPreferredMaximumHeight(max_height))
                .map_err(native_error)?;
        }

        if changes.presenter {
            let app_window = window_2.AppWindow().map_err(native_error)?;
            let kind = match visuals.presenter {
                WindowPresenter::Default => AppWindowPresenterKind::Default,
                WindowPresenter::FullScreen => AppWindowPresenterKind::FullScreen,
            };
            app_window.SetPresenterByKind(kind).map_err(native_error)?;
        }

        if changes.client_size
            && let Some((width, height)) = visuals.client_size
        {
            let hwnd = window_handle()?;
            let dpi = unsafe { GetDpiForWindow(hwnd.cast()) }.max(96);
            let pixels = |dips: f64| (dips * f64::from(dpi) / 96.0).round() as i32;
            window_2
                .AppWindow()
                .and_then(|window| window.cast::<IAppWindow2>())
                .and_then(|window| {
                    window.ResizeClient(SizeInt32 {
                        width: pixels(width),
                        height: pixels(height),
                    })
                })
                .map_err(native_error)?;
        }
        if changes.placement {
            // 位置放在尺寸之后应用：`.centered()` 要按**最终**的外框尺寸算居中，先
            // `ResizeClient` 再移动才不会用旧尺寸算出一个偏掉的坐标。
            match visuals.placement {
                WindowPlacement::Unspecified => {}
                WindowPlacement::Position { x, y } => {
                    // 位置走 Win32 `SetWindowPos` 而不是 `AppWindow.Move`：绑定只生成了
                    // `IAppWindow2` 的尺寸路径（`ClientSize`/`ResizeClient`），位置槽在
                    // vtable 里还是匿名占位（`MoveInZOrder*`），手搓接口布局的代价远大于
                    // 收益。`SetWindowPos` 本来就是窗口放置的标准入口，`AppWindow` 也是从
                    // 同一层读回几何信息，两者不会打架。
                    //
                    // `SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE`：只移动。声明"窗口在
                    // 哪儿"不该顺手改尺寸、改 Z 序或抢走焦点。
                    let hwnd = window_handle()?;
                    let dpi = unsafe { GetDpiForWindow(hwnd) }.max(96);
                    let pixels = |dips: f64| (dips * f64::from(dpi) / 96.0).round() as i32;
                    unsafe {
                        _ = SetWindowPos(
                            hwnd,
                            std::ptr::null_mut(),
                            pixels(x),
                            pixels(y),
                            0,
                            0,
                            SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
                        );
                    }
                }
                WindowPlacement::Centered => {
                    // 居中在物理像素层面算：工作区（`GetMonitorInfoW` 的 `rcWork`，已经排除
                    // 任务栏）和窗口外框（`GetWindowRect`）本来就是同一套单位，中间不过
                    // DPI，也就不存在"用错缩放比例"这一类错误。窗口当前在哪块显示器上，
                    // 就居中到哪块显示器。
                    let hwnd = window_handle()?;
                    if let (Some(area), Some((width, height))) =
                        (work_area_of(hwnd), window_frame_size(hwnd))
                    {
                        let (x, y) = area.centered_origin(width, height);
                        unsafe {
                            _ = SetWindowPos(
                                hwnd,
                                std::ptr::null_mut(),
                                x as i32,
                                y as i32,
                                0,
                                0,
                                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
                            );
                        }
                    }
                }
            }
        }
        self.window_visuals.insert(node, visuals);
        Ok(())
    }

    fn clear_window_title_bar(&mut self, node: NodeId) -> Result<(), RuntimeError> {
        let window = self
            .windows
            .get(&node)
            .ok_or(RuntimeError::MissingNode(node))?;
        window
            .SetTitleBar(None::<&UIElement>)
            .and_then(|()| window.SetExtendsContentIntoTitleBar(false))
            .map_err(native_error)?;
        window
            .cast::<IWindow2>()
            .and_then(|window| window.AppWindow())
            .and_then(|window| window.TitleBar())
            .and_then(|title_bar| title_bar.cast::<IAppWindowTitleBar2>())
            .and_then(|title_bar| {
                title_bar.SetPreferredHeightOption(TitleBarHeightOption::Standard)
            })
            .map_err(native_error)?;
        self.window_title_bars.remove(&node);
        Ok(())
    }

    fn apply_window_title_bar(
        &mut self,
        node: NodeId,
        title_bar: NodeId,
        height: WindowTitleBarHeight,
    ) -> Result<(), RuntimeError> {
        let window = self
            .windows
            .get(&node)
            .ok_or(RuntimeError::MissingNode(node))?;
        let title_bar_element = self.ui_element(title_bar)?;
        title_bar_element
            .SetIsTabStop(false)
            .and_then(|()| window.SetExtendsContentIntoTitleBar(true))
            .and_then(|()| window.SetTitleBar(&title_bar_element))
            .map_err(native_error)?;
        let height_option = match height {
            WindowTitleBarHeight::Standard => TitleBarHeightOption::Standard,
            WindowTitleBarHeight::Tall => TitleBarHeightOption::Tall,
        };
        window
            .cast::<IWindow2>()
            .and_then(|window| window.AppWindow())
            .and_then(|window| window.TitleBar())
            .and_then(|title_bar| title_bar.cast::<IAppWindowTitleBar2>())
            .and_then(|title_bar| title_bar.SetPreferredHeightOption(height_option))
            .map_err(native_error)?;
        self.window_title_bars.insert(node, (title_bar, height));
        Ok(())
    }

    fn set_window_title(&mut self, node: NodeId, title: &str) -> Result<(), RuntimeError> {
        let window = self
            .windows
            .get(&node)
            .ok_or(RuntimeError::MissingNode(node))?;
        let revision = {
            let mut revisions = self.window_title_revisions.borrow_mut();
            let revision = revisions
                .get_mut(&node)
                .ok_or(RuntimeError::MissingNode(node))?;
            *revision = revision.wrapping_add(1);
            *revision
        };
        window.SetTitle(title).map_err(native_error)?;
        if !self.window_title_bars.contains_key(&node) {
            return Ok(());
        }

        let window = window.clone();
        let title = title.to_string();
        let title_revisions = Rc::clone(&self.window_title_revisions);
        let identity = Rc::clone(&self.identity);
        let window_identity = identity.get().unwrap();
        let sink = self.event_sink()?;
        let handler = DispatcherQueueHandler::new(move || {
            if identity.get() != Some(window_identity)
                || title_revisions.borrow().get(&node) != Some(&revision)
            {
                return;
            }
            if let Err(error) = window.SetTitle(&title).map_err(native_error) {
                sink.enqueue_host(HostEvent::Error(error));
            }
        });
        let accepted = DispatcherQueue::GetForCurrentThread()
            .and_then(|dispatcher| {
                dispatcher.TryEnqueueWithPriority(DispatcherQueuePriority::Low, &handler)
            })
            .map_err(native_error)?;
        if accepted {
            Ok(())
        } else {
            Err(RuntimeError::DispatcherRejected)
        }
    }

    fn apply_theme_style(&mut self, node: NodeId, style: ThemeStyle) -> Result<(), RuntimeError> {
        let handle = self
            .handles
            .get(&node)
            .ok_or(RuntimeError::MissingNode(node))?;
        let framework_element = handle
            .ui_element()
            .and_then(|element| element.cast::<IFrameworkElement>())
            .map_err(native_error)?;
        if style.is_empty() {
            return framework_element
                .SetStyle(None::<&Style>)
                .map_err(native_error);
        }

        let kind = handle.kind();
        let compiled = if let Some(compiled) = self.theme_styles.get(&(kind, style)) {
            compiled.clone()
        } else {
            let (target, properties) =
                theme_style_info(kind).ok_or(RuntimeError::UnsupportedKind)?;
            let mut xaml = format!(
                "<Style xmlns='http://schemas.microsoft.com/winfx/2006/xaml/presentation' \
                 TargetType='{target}'>"
            );
            for (property, brush) in properties.iter().zip(style.values()) {
                if let Some(brush) = brush {
                    write!(
                        xaml,
                        "<Setter Property='{property}' Value='{{ThemeResource {}}}'/>",
                        brush.resource_key()
                    )
                    .unwrap();
                }
            }
            xaml.push_str("</Style>");
            // Same reasoning as `set_resource_style`: a style is decoration, and one
            // unresolvable `{ThemeResource …}` key must not take the whole frame down. The
            // brush keys come from `ThemeBrush::resource_key`, so a typo — or an SDK that
            // renamed one — would otherwise surface as "the entire window stopped updating"
            // rather than "this control lost its theme style".
            //
            // Nothing is cached on failure: the next publication retries, and a transient
            // failure must not be remembered as "this style is unavailable".
            let Ok(compiled) = XamlReader::Load(&xaml).and_then(|value| value.cast::<Style>())
            else {
                eprintln!("windows-reactor: theme style could not be compiled: {xaml}");
                return framework_element
                    .SetStyle(None::<&Style>)
                    .map_err(native_error);
            };
            self.theme_styles.insert((kind, style), compiled.clone());
            compiled
        };
        framework_element.SetStyle(&compiled).map_err(native_error)
    }

    fn apply_window_theme(host: &bindings::Grid, theme: WindowTheme) -> Result<(), RuntimeError> {
        host.cast::<FrameworkElement>()
            .and_then(|root| {
                root.SetRequestedTheme(match theme {
                    WindowTheme::System => ElementTheme::Default,
                    WindowTheme::Light => ElementTheme::Light,
                    WindowTheme::Dark => ElementTheme::Dark,
                })
            })
            .map_err(native_error)
    }

    fn set_resource_overrides(
        &mut self,
        node: NodeId,
        resources: &ResourceOverrides,
    ) -> Result<(), RuntimeError> {
        let dictionary = self
            .ui_element(node)?
            .cast::<IFrameworkElement>()
            .and_then(|element| element.Resources())
            .map_err(native_error)?;
        let map = dictionary
            .cast::<
                windows_collections::IMap<
                    windows_core::IInspectable,
                    windows_core::IInspectable,
                >,
            >()
            .map_err(native_error)?;
        let desired_keys = resources
            .values()
            .map(|(key, _)| key.to_string())
            .collect::<HashSet<_>>();

        if let Some(previous_keys) = self.resource_override_keys.get(&node) {
            for key in previous_keys.difference(&desired_keys) {
                let key = windows_reference::IReference::from(key.as_str());
                if map.HasKey(&key).map_err(native_error)? {
                    map.Remove(&key).map_err(native_error)?;
                }
            }
        }

        for (key, value) in resources.values() {
            let key = windows_reference::IReference::from(key);
            let value = resource_value(value).map_err(native_error)?;
            map.Insert(&key, &value).map_err(native_error)?;
        }

        if desired_keys.is_empty() {
            self.resource_override_keys.remove(&node);
        } else {
            self.resource_override_keys.insert(node, desired_keys);
        }
        Ok(())
    }

    fn set_window_observations(
        &mut self,
        node: NodeId,
        observations: WindowObservationFlags,
    ) -> Result<(), RuntimeError> {
        self.window_observations.insert(node, observations);
        self.window_observation_subscriptions.remove(&node);
        if observations == WindowObservationFlags::default() {
            return Ok(());
        }
        let source = self
            .window_hosts
            .get(&node)
            .ok_or(RuntimeError::MissingNode(node))?
            .cast::<IFrameworkElement>()
            .map_err(native_error)?;
        let sink = self.event_sink()?;
        let mut subscriptions = Vec::new();
        if let Some(observation) = observations.window_size {
            let sink = sink.clone();
            subscriptions.push(
                source
                    .SizeChanged(move |_, args| {
                        let result = args
                            .as_ref()
                            .ok_or_else(windows_core::Error::empty)
                            .and_then(|args| args.NewSize());
                        match result {
                            Ok(value) => sink.enqueue_host(HostEvent::WindowSize {
                                observation,
                                size: WindowSize {
                                    width: f64::from(value.width),
                                    height: f64::from(value.height),
                                },
                            }),
                            Err(error) => {
                                sink.enqueue_host(HostEvent::ObservationError {
                                    observation,
                                    error: native_error(error),
                                });
                            }
                        }
                    })
                    .map_err(native_error)?,
            );
        }
        if let Some(observation) = observations.color_scheme {
            let changed_sink = sink.clone();
            subscriptions.push(
                source
                    .ActualThemeChanged(move |sender, _| {
                        let result = sender
                            .as_ref()
                            .ok_or_else(windows_core::Error::empty)
                            .and_then(|sender| sender.cast::<IFrameworkElement>())
                            .and_then(|sender| sender.ActualTheme());
                        match result {
                            Ok(ElementTheme::Dark) => {
                                changed_sink.enqueue_host(HostEvent::ColorScheme {
                                    observation,
                                    scheme: ColorScheme::Dark,
                                });
                            }
                            Ok(_) => {
                                changed_sink.enqueue_host(HostEvent::ColorScheme {
                                    observation,
                                    scheme: ColorScheme::Light,
                                });
                            }
                            Err(error) => {
                                changed_sink.enqueue_host(HostEvent::ObservationError {
                                    observation,
                                    error: native_error(error),
                                });
                            }
                        }
                    })
                    .map_err(native_error)?,
            );
            let scheme = match source.ActualTheme().map_err(native_error)? {
                ElementTheme::Dark => ColorScheme::Dark,
                _ => ColorScheme::Light,
            };
            sink.enqueue_host(HostEvent::ColorScheme {
                observation,
                scheme,
            });
        }
        self.window_observation_subscriptions
            .insert(node, subscriptions);
        Ok(())
    }

    fn clear_resource_overrides(&mut self, node: NodeId) -> Result<(), RuntimeError> {
        let Some(keys) = self.resource_override_keys.remove(&node) else {
            return Ok(());
        };
        let dictionary = self
            .ui_element(node)?
            .cast::<IFrameworkElement>()
            .and_then(|element| element.Resources())
            .map_err(native_error)?;
        let map = dictionary
            .cast::<
                windows_collections::IMap<
                    windows_core::IInspectable,
                    windows_core::IInspectable,
                >,
            >()
            .map_err(native_error)?;
        for key in keys {
            let key = windows_reference::IReference::from(key.as_str());
            if map.HasKey(&key).map_err(native_error)? {
                map.Remove(&key).map_err(native_error)?;
            }
        }
        Ok(())
    }

    fn set_key_accelerators(
        &self,
        node: NodeId,
        accelerators: &KeyAccelerators,
    ) -> Result<(), RuntimeError> {
        let element = self
            .ui_element(node)?
            .cast::<IUIElement>()
            .map_err(native_error)?;
        let values = element.KeyboardAccelerators().map_err(native_error)?;
        values.Clear().map_err(native_error)?;
        element
            .SetKeyboardAcceleratorPlacementMode(KeyboardAcceleratorPlacementMode::Hidden)
            .map_err(native_error)?;

        for accelerator in &accelerators.values {
            let value = KeyboardAccelerator::new().map_err(native_error)?;
            value
                .SetKey(match accelerator.key {
                    AcceleratorKey::Left => bindings::VirtualKey::Left,
                    AcceleratorKey::Up => bindings::VirtualKey::Up,
                    AcceleratorKey::Right => bindings::VirtualKey::Right,
                    AcceleratorKey::Down => bindings::VirtualKey::Down,
                    AcceleratorKey::Space => bindings::VirtualKey::Space,
                    AcceleratorKey::N => bindings::VirtualKey::N,
                    AcceleratorKey::P => bindings::VirtualKey::P,
                    AcceleratorKey::R => bindings::VirtualKey::R,
                    AcceleratorKey::NumberPad0 => bindings::VirtualKey::NumberPad0,
                    AcceleratorKey::NumberPad1 => bindings::VirtualKey::NumberPad1,
                    AcceleratorKey::NumberPad2 => bindings::VirtualKey::NumberPad2,
                    AcceleratorKey::NumberPad3 => bindings::VirtualKey::NumberPad3,
                    AcceleratorKey::NumberPad4 => bindings::VirtualKey::NumberPad4,
                    AcceleratorKey::NumberPad5 => bindings::VirtualKey::NumberPad5,
                    AcceleratorKey::NumberPad6 => bindings::VirtualKey::NumberPad6,
                    AcceleratorKey::NumberPad7 => bindings::VirtualKey::NumberPad7,
                    AcceleratorKey::NumberPad8 => bindings::VirtualKey::NumberPad8,
                    AcceleratorKey::NumberPad9 => bindings::VirtualKey::NumberPad9,
                    AcceleratorKey::Divide => bindings::VirtualKey::Divide,
                    AcceleratorKey::Multiply => bindings::VirtualKey::Multiply,
                    AcceleratorKey::Subtract => bindings::VirtualKey::Subtract,
                    AcceleratorKey::Add => bindings::VirtualKey::Add,
                    AcceleratorKey::Decimal => bindings::VirtualKey::Decimal,
                    AcceleratorKey::Enter => bindings::VirtualKey::Enter,
                })
                .map_err(native_error)?;
            value
                .SetModifiers(match accelerator.modifiers {
                    AcceleratorModifiers::None => VirtualKeyModifiers::None,
                    AcceleratorModifiers::Control => VirtualKeyModifiers::Control,
                })
                .map_err(native_error)?;
            let callback = accelerator.callback.clone();
            value
                .Invoked(move |_, args| {
                    if let Some(args) = args.as_ref() {
                        _ = args.SetHandled(true);
                    }
                    _ = callback.call(());
                })
                .map(windows_core::EventRevoker::into_token)
                .map_err(native_error)?;
            values.Append(&value).map_err(native_error)?;
        }
        Ok(())
    }

    fn set_pointer_policy(
        &self,
        node: NodeId,
        property: PropertyId,
        value: &PropertyValue,
    ) -> Result<(), RuntimeError> {
        let PropertyValue::Bool(value) = value else {
            return Err(RuntimeError::UnsupportedKind);
        };
        let element = self.ui_element(node)?;
        let mut policy = self
            .pointer_policies
            .borrow()
            .get(&node)
            .copied()
            .unwrap_or_default();
        match property {
            PropertyId::BorderCapturePointerOnPress => {
                policy.capture = *value;
                if !value {
                    element.ReleasePointerCaptures().map_err(native_error)?;
                }
            }
            PropertyId::BorderFocusOnPointerRelease => policy.focus_on_release = *value,
            _ => return Err(RuntimeError::UnsupportedKind),
        }
        if policy.is_empty() {
            self.pointer_policies.borrow_mut().remove(&node);
        } else {
            self.pointer_policies.borrow_mut().insert(node, policy);
        }
        Ok(())
    }

    fn clear_pointer_policy(&self, node: NodeId, property: PropertyId) -> Result<(), RuntimeError> {
        self.set_pointer_policy(node, property, &PropertyValue::Bool(false))
    }

    fn apply_one(&mut self, command: &Command) -> Result<(), RuntimeError> {
        match command {
            Command::CreateApplication { node } => {
                if self.contains(*node) || self.application.is_some() {
                    return Err(RuntimeError::DuplicateNode(*node));
                }
                let application = self
                    .pending_application
                    .take()
                    .ok_or(RuntimeError::MissingApplication)?;
                self.application = Some((*node, application));
            }
            Command::CreateWindow { node } => {
                if self.contains(*node) {
                    return Err(RuntimeError::DuplicateNode(*node));
                }
                let window = Window::new().map_err(native_error)?;
                let host = bindings::Grid::new().map_err(native_error)?;
                window
                    .SetContent(&host.cast::<UIElement>().map_err(native_error)?)
                    .map_err(native_error)?;
                let closed = Rc::clone(&self.window_closed);
                let identity = self.identity.get().unwrap();
                let backdrops = Rc::clone(&self.window_backdrops);
                let closed_node = *node;
                let subscription = window
                    .Closed(move |_, _| {
                        // Release this window's backdrop controller here, while the
                        // XAML runtime is still alive.
                        //
                        // It has to be *this* window's: `native_window_closed` fires
                        // for any window, and draining the whole table there tears the
                        // material out from under every other open window — whose
                        // composition layer then dereferences a released controller
                        // (measured: access violation with `SerializeEffectDescription`
                        // on the stack). It also cannot wait for `WinUiRuntime` to be
                        // dropped: by then WinUI is gone and `Release` faults.
                        if let Some(mut host) = backdrops.borrow_mut().remove(&closed_node) {
                            host.release();
                        }
                        closed.set(true);
                        dispatch_window_closed(identity);
                    })
                    .map_err(native_error)?;
                self.window_subscriptions.insert(*node, subscription);
                self.window_hosts.insert(*node, host);
                self.window_title_revisions.borrow_mut().insert(*node, 0);
                self.windows.insert(*node, window);
            }
            Command::ActivateWindow { node } => {
                self.windows
                    .get(node)
                    .ok_or(RuntimeError::MissingNode(*node))?
                    .Activate()
                    .map_err(native_error)?;
            }
            Command::RequestWindowActivation { node } => {
                let window = self
                    .windows
                    .get(node)
                    .ok_or(RuntimeError::MissingNode(*node))?;
                window.Activate().map_err(native_error)?;
                if let Ok(native) = window.cast::<IWindowNative>() {
                    let mut hwnd = std::ptr::null_mut();
                    unsafe {
                        if native.WindowHandle(&mut hwnd).is_ok() {
                            if IsIconic(hwnd.cast()).as_bool() {
                                _ = ShowWindow(hwnd.cast(), SW_RESTORE);
                            }
                            _ = SetForegroundWindow(hwnd.cast());
                        }
                    }
                }
            }
            Command::CloseWindow { node } => {
                self.windows
                    .get(node)
                    .ok_or(RuntimeError::MissingNode(*node))?
                    .Close()
                    .map_err(native_error)?;
            }
            Command::SetWindowTitle { node, title } => {
                self.set_window_title(*node, title)?;
            }
            Command::ClearWindowTitleBar { node } => {
                self.clear_window_title_bar(*node)?;
            }
            Command::SetWindowTitleBar {
                node,
                title_bar,
                height,
            } => {
                self.apply_window_title_bar(*node, *title_bar, *height)?;
            }
            Command::SetWindowVisuals { node, visuals } => {
                self.apply_window_visuals(*node, *visuals)?;
            }
            Command::SetWindowObservations { node, observations } => {
                self.set_window_observations(*node, *observations)?;
            }
            Command::SetThemeStyle { node, style } => {
                self.apply_theme_style(*node, *style)?;
            }
            Command::Create { node, kind } => {
                if self.contains(*node) {
                    return Err(RuntimeError::DuplicateNode(*node));
                }
                let handle = Handle::create(*kind)?;
                if let Handle::ContentDialog(dialog) = &handle {
                    self.content_dialogs
                        .borrow_mut()
                        .create(*node, dialog.clone());
                }
                self.handles.insert(*node, handle);
            }
            Command::CreateVirtualCollection {
                node,
                item_count,
                source_revision,
            } => {
                if self.contains(*node) {
                    return Err(RuntimeError::DuplicateNode(*node));
                }
                let handle = element_factory::VirtualHandle::create(
                    self.identity.get().unwrap(),
                    *node,
                    *item_count,
                    *source_revision,
                    Rc::clone(&self.realizations),
                    self.event_sink()?,
                )
                .map_err(native_error)?;
                self.virtuals.insert(*node, handle);
            }
            Command::ResetVirtualCollection {
                node,
                item_count,
                source_revision,
            } => {
                self.virtuals
                    .get(node)
                    .ok_or(RuntimeError::MissingNode(*node))?
                    .reset(*item_count, *source_revision)
                    .map_err(native_error)?;
            }
            Command::AttachRealized {
                collection,
                container,
                child,
            } => {
                let child = self.ui_element(*child)?;
                self.virtuals
                    .get(collection)
                    .ok_or(RuntimeError::MissingNode(*collection))?
                    .set_content(*container, Some(&child))
                    .map_err(native_error)?;
            }
            Command::DetachRealized {
                collection,
                container,
                ..
            } => {
                self.virtuals
                    .get(collection)
                    .ok_or(RuntimeError::MissingNode(*collection))?
                    .clear_content(*container)
                    .map_err(native_error)?;
            }
            Command::AcknowledgeRecycle {
                collection,
                container,
            } => {
                self.virtuals
                    .get(collection)
                    .ok_or(RuntimeError::MissingNode(*collection))?
                    .acknowledge_recycle(*container)
                    .map_err(native_error)?;
            }
            Command::Destroy { node } => {
                if !self.contains(*node) {
                    return Err(RuntimeError::MissingNode(*node));
                }
                self.cancel_retirements_for_node(*node)?;
                if self.flyouts.contains_key(node)
                    || self.flyouts.values().any(|(_, content)| content == node)
                    || self.owned_menus.values().any(|menu| menu.target == *node)
                    || self
                        .command_bar_flyouts
                        .values()
                        .any(|flyout| flyout.target == *node)
                    || self
                        .swipe_attachments
                        .values()
                        .any(|swipes| swipes.target == *node)
                {
                    return Err(RuntimeError::StillParented(*node));
                }
                if self
                    .window_title_bars
                    .values()
                    .any(|(title_bar, _)| title_bar == node)
                {
                    return Err(RuntimeError::StillParented(*node));
                }
                return self.remove_node_state(*node);
            }
            Command::RetireSubtree {
                root,
                nodes,
                parent,
                slot,
                transition,
            } => {
                self.start_retirement(*root, nodes.clone(), *parent, *slot, *transition)?;
            }

            Command::Focus { node, completion } => {
                let result = self.ui_element(*node).and_then(|element| {
                    self.pending_focus_states
                        .borrow_mut()
                        .insert(*node, ElementFocusState::Programmatic);
                    match element
                        .Focus(FocusState::Programmatic)
                        .map_err(native_error)
                    {
                        Ok(true) => {
                            let pending = Rc::clone(&self.pending_focus_states);
                            let focus_node = *node;
                            let cleanup = DispatcherQueueHandler::new(move || {
                                pending.borrow_mut().remove(&focus_node);
                            });
                            let accepted = DispatcherQueue::GetForCurrentThread()
                                .and_then(|dispatcher| {
                                    dispatcher.TryEnqueueWithPriority(
                                        DispatcherQueuePriority::Low,
                                        &cleanup,
                                    )
                                })
                                .map_err(native_error)?;
                            if accepted {
                                Ok(true)
                            } else {
                                self.pending_focus_states.borrow_mut().remove(node);
                                Err(RuntimeError::DispatcherRejected)
                            }
                        }
                        Ok(false) => {
                            self.pending_focus_states.borrow_mut().remove(node);
                            Ok(false)
                        }
                        Err(error) => {
                            self.pending_focus_states.borrow_mut().remove(node);
                            Err(error)
                        }
                    }
                });
                _ = completion.call(result);
            }
            Command::InitializeWebView2 { node, completion } => {
                let Some(Handle::WebView2(control)) = self.handles.get(node) else {
                    _ = completion.call(Err(if self.contains(*node) {
                        RuntimeError::UnsupportedKind
                    } else {
                        RuntimeError::MissingNode(*node)
                    }));
                    return Ok(());
                };
                if let Ok(core) = control.CoreWebView2() {
                    let result = Ok(core.into());
                    let pending = {
                        let mut initializations = self.webview_initializations.borrow_mut();
                        if let Some(initialization) = initializations.get_mut(node) {
                            initialization.completions.push(completion.clone());
                            true
                        } else {
                            false
                        }
                    };
                    if pending {
                        complete_webview_initialization(
                            &self.webview_initializations,
                            *node,
                            result,
                        );
                    } else {
                        _ = completion.call(result);
                    }
                    return Ok(());
                }
                if let Some(initialization) =
                    self.webview_initializations.borrow_mut().get_mut(node)
                {
                    initialization.completions.push(completion.clone());
                    return Ok(());
                }
                let initializations = Rc::clone(&self.webview_initializations);
                let initialized_node = *node;
                let revoker = match control.CoreWebView2Initialized(move |sender, _| {
                    let result = sender
                        .as_ref()
                        .ok_or_else(windows_core::Error::empty)
                        .and_then(|control| control.CoreWebView2())
                        .map(Into::into)
                        .map_err(native_error);
                    complete_webview_initialization(&initializations, initialized_node, result);
                }) {
                    Ok(revoker) => revoker,
                    Err(error) => {
                        _ = completion.call(Err(native_error(error)));
                        return Ok(());
                    }
                };
                let element = match control.cast::<IFrameworkElement>() {
                    Ok(element) => element,
                    Err(error) => {
                        _ = completion.call(Err(native_error(error)));
                        return Ok(());
                    }
                };
                let mut action = None;
                let loaded = if element.IsLoaded().unwrap_or(false) {
                    match control.EnsureCoreWebView2Async() {
                        Ok(pending) => action = Some(pending),
                        Err(error) => {
                            _ = completion.call(Err(native_error(error)));
                            return Ok(());
                        }
                    }
                    None
                } else {
                    let initializations = Rc::clone(&self.webview_initializations);
                    let loaded_node = *node;
                    let control = control.clone();
                    match element.Loaded(move |_, _| match control.EnsureCoreWebView2Async() {
                        Ok(value) => {
                            if let Some(initialization) =
                                initializations.borrow_mut().get_mut(&loaded_node)
                            {
                                initialization._action = Some(value);
                            }
                        }
                        Err(error) => complete_webview_initialization(
                            &initializations,
                            loaded_node,
                            Err(native_error(error)),
                        ),
                    }) {
                        Ok(loaded) => Some(loaded),
                        Err(error) => {
                            _ = completion.call(Err(native_error(error)));
                            return Ok(());
                        }
                    }
                };
                self.webview_initializations.borrow_mut().insert(
                    *node,
                    WebViewInitialization {
                        _action: action,
                        _loaded: loaded,
                        _initialized: revoker,
                        completions: vec![completion.clone()],
                    },
                );
            }
            Command::ObserveSwapChainPanel {
                node,
                observation,
                binding,
                callback,
            } => {
                let Some(Handle::SwapChainPanel(control)) = self.handles.get(node) else {
                    return Err(if self.contains(*node) {
                        RuntimeError::UnsupportedKind
                    } else {
                        RuntimeError::MissingNode(*node)
                    });
                };
                let binding = *binding;
                let element = control.cast::<IFrameworkElement>().map_err(native_error)?;
                let emit_metrics = |width: f64, height: f64| {
                    let event = SwapChainPanelEvent::Metrics {
                        binding,
                        width,
                        height,
                        scale_x: control.CompositionScaleX().unwrap_or(1.0),
                        scale_y: control.CompositionScaleY().unwrap_or(1.0),
                    };
                    invoke_callback(callback, event);
                };
                emit_metrics(
                    element.ActualWidth().unwrap_or(0.0),
                    element.ActualHeight().unwrap_or(0.0),
                );

                let size_callback = callback.clone();
                let size_control = control.clone();
                let size = element
                    .SizeChanged(move |_, args| {
                        if let Some(args) = args.as_ref()
                            && let Ok(value) = args.NewSize()
                        {
                            invoke_callback(
                                &size_callback,
                                SwapChainPanelEvent::Metrics {
                                    binding,
                                    width: f64::from(value.width),
                                    height: f64::from(value.height),
                                    scale_x: size_control.CompositionScaleX().unwrap_or(1.0),
                                    scale_y: size_control.CompositionScaleY().unwrap_or(1.0),
                                },
                            );
                        }
                    })
                    .map_err(native_error)?;
                let scale_callback = callback.clone();
                let scale_element = element;
                let scale = control
                    .CompositionScaleChanged(move |sender, _| {
                        if let Some(sender) = sender.as_ref() {
                            invoke_callback(
                                &scale_callback,
                                SwapChainPanelEvent::Metrics {
                                    binding,
                                    width: scale_element.ActualWidth().unwrap_or(0.0),
                                    height: scale_element.ActualHeight().unwrap_or(0.0),
                                    scale_x: sender.CompositionScaleX().unwrap_or(1.0),
                                    scale_y: sender.CompositionScaleY().unwrap_or(1.0),
                                },
                            );
                        }
                    })
                    .map_err(native_error)?;
                let rendering_callback = callback.clone();
                let rendering = CompositionTarget::Rendering(move |_, _| {
                    invoke_callback(&rendering_callback, SwapChainPanelEvent::Rendering);
                })
                .map_err(native_error)?;
                self.observation_subscriptions.insert(
                    (*node, *observation),
                    ObservationSubscription::SwapChainPanel {
                        _rendering: rendering,
                        _scale: scale,
                        _size: size,
                    },
                );
            }
            Command::RequestSwapChainPanelFrame { node, completion } => {
                let result = match self.handles.get(node) {
                    Some(Handle::SwapChainPanel(_)) => {
                        let completion = completion.clone();
                        let handler = DispatcherQueueHandler::new(move || {
                            _ = completion.call(Ok(()));
                        });
                        DispatcherQueue::GetForCurrentThread()
                            .map_err(native_error)
                            .and_then(|dispatcher| {
                                dispatcher
                                    .TryEnqueueWithPriority(
                                        DispatcherQueuePriority::Normal,
                                        &handler,
                                    )
                                    .map_err(native_error)
                            })
                            .and_then(|accepted| {
                                accepted.then_some(()).ok_or(RuntimeError::SchedulerClosed)
                            })
                    }
                    Some(_) => Err(RuntimeError::UnsupportedKind),
                    None => Err(RuntimeError::MissingNode(*node)),
                };
                if let Err(error) = result {
                    _ = completion.call(Err(error));
                }
            }
            Command::SetSwapChain {
                node,
                swap_chain,
                completion,
            } => {
                let result = match self.handles.get(node) {
                    Some(Handle::SwapChainPanel(control)) => control
                        .cast::<ISwapChainPanelNative>()
                        .map_err(native_error)
                        .and_then(|native| {
                            let raw = swap_chain
                                .as_ref()
                                .map_or(std::ptr::null_mut(), Interface::as_raw);
                            unsafe { native.SetSwapChain(raw).ok().map_err(native_error) }
                        }),
                    Some(_) => Err(RuntimeError::UnsupportedKind),
                    None => Err(RuntimeError::MissingNode(*node)),
                };
                _ = completion.call(result);
            }
            Command::SetNativeImageSource {
                node,
                source,
                completion,
            } => {
                let control = match self.handles.get(node) {
                    Some(Handle::Image(control)) => Ok(control.clone()),
                    Some(_) => Err(RuntimeError::UnsupportedKind),
                    None => Err(RuntimeError::MissingNode(*node)),
                };
                let source = source
                    .as_ref()
                    .map(|source| source.cast::<ImageSource>().map_err(native_error))
                    .transpose();
                let result = control.and_then(|control| {
                    source.and_then(|source| {
                        self.release_encoded_image_source(*node);
                        control.SetSource(source.as_ref()).map_err(native_error)
                    })
                });
                _ = completion.call(result);
            }
            Command::SetMediaSource {
                node,
                source,
                completion,
            } => {
                let control = match self.handles.get(node) {
                    Some(Handle::MediaPlayerElement(control)) => Ok(control.clone()),
                    Some(_) => Err(RuntimeError::UnsupportedKind),
                    None => Err(RuntimeError::MissingNode(*node)),
                };
                let source = source
                    .as_ref()
                    .map(|source| source.cast::<IMediaPlaybackSource>().map_err(native_error))
                    .transpose();
                let result = control.and_then(|control| {
                    source
                        .and_then(|source| control.SetSource(source.as_ref()).map_err(native_error))
                });
                _ = completion.call(result);
            }
            Command::ObserveImageScale {
                node,
                observation,
                callback,
            } => {
                let control = match self.handles.get(node) {
                    Some(Handle::Image(control)) => control,
                    Some(_) => return Err(RuntimeError::UnsupportedKind),
                    None => return Err(RuntimeError::MissingNode(*node)),
                };
                let element = control.cast::<UIElement>().map_err(native_error)?;
                let framework = control.cast::<IFrameworkElement>().map_err(native_error)?;
                let sink = self.event_sink()?;
                let scale_callback = callback.clone();
                let subscriptions =
                    subscribe_xaml_root_scale(&element, &framework, &sink, move |scale| {
                        invoke_callback(&scale_callback, scale);
                    })?;
                self.observation_subscriptions.insert(
                    (*node, *observation),
                    ObservationSubscription::ImageScale {
                        _root: subscriptions,
                    },
                );
            }
            Command::ObserveCompositionHost {
                node,
                observation,
                callback,
            } => {
                let control = match self.handles.get(node) {
                    Some(Handle::Grid(control)) => control,
                    Some(_) => return Err(RuntimeError::UnsupportedKind),
                    None => return Err(RuntimeError::MissingNode(*node)),
                };
                let element = control.cast::<UIElement>().map_err(native_error)?;
                let framework = control.cast::<IFrameworkElement>().map_err(native_error)?;
                let visual =
                    ElementCompositionPreview::GetElementVisual(&element).map_err(native_error)?;
                let compositor = visual
                    .cast::<ICompositionObject>()
                    .map_err(native_error)?
                    .Compositor()
                    .map_err(native_error)?;
                invoke_callback(
                    callback,
                    CompositionHostEvent::Ready {
                        compositor: compositor.into(),
                        width: framework.ActualWidth().unwrap_or(0.0),
                        height: framework.ActualHeight().unwrap_or(0.0),
                        scale: xaml_scale(&element)?,
                    },
                );

                let size_callback = callback.clone();
                let size_element = element.clone();
                let sink = self.event_sink()?;
                let size_sink = sink.clone();
                let size = framework
                    .SizeChanged(move |_, args| {
                        if let Some(args) = args.as_ref()
                            && let Ok(value) = args.NewSize()
                        {
                            let scale = match xaml_scale(&size_element) {
                                Ok(scale) => scale,
                                Err(error) => {
                                    size_sink.enqueue_host(HostEvent::Error(error));
                                    return;
                                }
                            };
                            invoke_callback(
                                &size_callback,
                                CompositionHostEvent::Metrics {
                                    width: f64::from(value.width),
                                    height: f64::from(value.height),
                                    scale,
                                },
                            );
                        }
                    })
                    .map_err(native_error)?;

                let scale_callback = callback.clone();
                let scale_framework = framework.clone();
                let root = subscribe_xaml_root_scale(&element, &framework, &sink, move |scale| {
                    invoke_callback(
                        &scale_callback,
                        CompositionHostEvent::Metrics {
                            width: scale_framework.ActualWidth().unwrap_or(0.0),
                            height: scale_framework.ActualHeight().unwrap_or(0.0),
                            scale,
                        },
                    );
                })?;
                self.observation_subscriptions.insert(
                    (*node, *observation),
                    ObservationSubscription::CompositionHost {
                        _root: root,
                        _size: size,
                    },
                );
            }
            Command::ObserveScrollOffsets {
                node,
                observation,
                callback,
            } => {
                let late: Rc<RefCell<Option<windows_core::EventRevoker>>> =
                    Rc::new(RefCell::new(None));
                let anchor = match self.handles.get(node) {
                    Some(Handle::ScrollViewer(control)) => {
                        subscribe_scroll_offsets(control, callback, &late)?;
                        None
                    }
                    Some(Handle::ComboBox(control)) => {
                        // Resolve now in case the template is ready; the open
                        // hook below re-resolves on every drop-down, which also
                        // covers lazily materialized popup content.
                        if let Ok(viewer) = combo_box_scroll_viewer(control)
                            && let Some(viewer) = viewer
                        {
                            subscribe_scroll_offsets(&viewer, callback, &late)?;
                        }
                        let opened_control = control.clone();
                        let opened_callback = callback.clone();
                        let opened_late = Rc::clone(&late);
                        let scoped = opened_control.clone();
                        let opened = opened_control
                            .DropDownOpened(move |_, _| {
                                if let Ok(Some(viewer)) = combo_box_scroll_viewer(&scoped) {
                                    let _ = subscribe_scroll_offsets(
                                        &viewer,
                                        &opened_callback,
                                        &opened_late,
                                    );
                                }
                            })
                            .map_err(native_error)?;
                        Some(opened)
                    }
                    Some(_) => {
                        return Err(if self.contains(*node) {
                            RuntimeError::UnsupportedKind
                        } else {
                            RuntimeError::MissingNode(*node)
                        });
                    }
                    None => return Err(RuntimeError::MissingNode(*node)),
                };
                self.observation_subscriptions.insert(
                    (*node, *observation),
                    ObservationSubscription::ScrollOffsets {
                        _anchor: anchor,
                        _view: late,
                    },
                );
            }
            Command::RevokeObservation { node, observation } => {
                self.observation_subscriptions
                    .remove(&(*node, *observation));
            }
            Command::SetCompositionChildVisual {
                node,
                visual,
                completion,
            } => {
                let result = match self.handles.get(node) {
                    Some(Handle::Grid(control)) => control
                        .cast::<UIElement>()
                        .map_err(native_error)
                        .and_then(|element| {
                            let visual = visual
                                .as_ref()
                                .map(|visual| visual.cast::<Visual>().map_err(native_error))
                                .transpose()?;
                            ElementCompositionPreview::SetElementChildVisual(
                                &element,
                                visual.as_ref(),
                            )
                            .map_err(native_error)
                        }),
                    Some(_) => Err(RuntimeError::UnsupportedKind),
                    None => Err(RuntimeError::MissingNode(*node)),
                };
                _ = completion.call(result);
            }
            Command::SetProperty {
                node,
                property,
                value,
            } => {
                if matches!(
                    property,
                    PropertyId::ImageSource | PropertyId::ImageIconSource
                ) {
                    self.release_encoded_image_source(*node);
                    if let PropertyValue::EncodedImage(value) = value {
                        self.encoded_image_nodes.borrow_mut().insert(*node);
                        let handle = self
                            .handles
                            .get(node)
                            .ok_or(RuntimeError::MissingNode(*node))?;
                        clear_property(handle, *property)?;
                        if let Err(error) = self.start_encoded_image(*node, value) {
                            self.report_image_decode_error(error);
                        }
                        return Ok(());
                    }
                }
                if matches!(
                    property,
                    PropertyId::ButtonKeyboardAccelerators | PropertyId::GridKeyboardAccelerators
                ) {
                    return match value {
                        PropertyValue::KeyAccelerators(value) => {
                            self.set_key_accelerators(*node, value)
                        }
                        _ => Err(RuntimeError::UnsupportedKind),
                    };
                }
                if is_resource_overrides_property(*property) {
                    return match value {
                        PropertyValue::ResourceOverrides(value) => {
                            self.set_resource_overrides(*node, value)
                        }
                        _ => Err(RuntimeError::UnsupportedKind),
                    };
                }
                if *property == PropertyId::BorderScale {
                    self.sync_scale_center(*node)?;
                }
                let target = PropertyTarget::resolve(self, *node, *property)?;
                let feedback = expected_feedback(*property, Some(value));
                let selection_owner = self
                    .selection_owners
                    .get(node)
                    .copied()
                    .filter(|(_, slot)| selection_for_item_property(*property, *slot).is_some());
                let feedback_event = feedback.as_ref().map(|(event, _)| *event);
                if let Some((event, expectation)) = feedback {
                    self.feedback
                        .borrow_mut()
                        .insert((*node, event), expectation);
                }
                let (result, observation) = self.with_selection_suppressed(selection_owner, || {
                    let result = if matches!(
                        property,
                        PropertyId::BorderCapturePointerOnPress
                            | PropertyId::BorderFocusOnPointerRelease
                    ) {
                        self.set_pointer_policy(*node, *property, value)
                    } else if *property == PropertyId::BorderAllowDrop {
                        match value {
                            PropertyValue::DragDropPolicy(policy) => self
                                .ui_element(*node)
                                .and_then(|element| {
                                    element
                                        .SetAllowDrop(
                                            policy.accepts(DragKind::StorageItems)
                                                || policy.accepts(DragKind::Text),
                                        )
                                        .map_err(native_error)
                                })
                                .map(|_| {
                                    self.drop_policies
                                        .borrow_mut()
                                        .insert(*node, policy.clone());
                                }),
                            _ => Err(RuntimeError::UnsupportedKind),
                        }
                    } else if *property == PropertyId::RichEditBoxDocument {
                        self.set_rich_edit_text(*node, value)
                    } else {
                        target.set(*property, value)
                    };
                    let observation =
                        feedback_event.and_then(|event| {
                            self.feedback.borrow_mut().remove(&(*node, event)).and_then(
                                |expectation| match expectation {
                                    FeedbackExpectation::Normalized { observation } => observation,
                                    FeedbackExpectation::Exact(_)
                                    | FeedbackExpectation::Suppressed => None,
                                },
                            )
                        });
                    (result, observation)
                });
                result?;
                if controlled_collection_for_property(*property).is_some()
                    && let PropertyValue::I32(value) = value
                {
                    self.controlled_collection_indices.insert(*node, *value);
                }
                if let Some(observation) = observation {
                    self.events.borrow_mut().push(NativeWork {
                        identity: self.identity.get().unwrap(),
                        work: observation,
                    });
                    self.schedule_dispatch()?;
                }
            }
            Command::ClearProperty { node, property } => {
                if matches!(
                    property,
                    PropertyId::ImageSource | PropertyId::ImageIconSource
                ) {
                    self.release_encoded_image_source(*node);
                }
                if matches!(
                    property,
                    PropertyId::ButtonKeyboardAccelerators | PropertyId::GridKeyboardAccelerators
                ) {
                    return self.set_key_accelerators(*node, &KeyAccelerators::default());
                }
                if is_resource_overrides_property(*property) {
                    return self.clear_resource_overrides(*node);
                }
                let target = PropertyTarget::resolve(self, *node, *property)?;
                let feedback = expected_feedback(*property, None);
                let selection_owner = self
                    .selection_owners
                    .get(node)
                    .copied()
                    .filter(|(_, slot)| selection_for_item_property(*property, *slot).is_some());
                let feedback_event = feedback.as_ref().map(|(event, _)| *event);
                if let Some((event, expectation)) = feedback {
                    self.feedback
                        .borrow_mut()
                        .insert((*node, event), expectation);
                }
                let (result, observation) = self.with_selection_suppressed(selection_owner, || {
                    let result = (|| {
                        if matches!(
                            property,
                            PropertyId::BorderCapturePointerOnPress
                                | PropertyId::BorderFocusOnPointerRelease
                        ) {
                            self.clear_pointer_policy(*node, *property)
                        } else if *property == PropertyId::BorderAllowDrop {
                            self.ui_element(*node)?
                                .SetAllowDrop(false)
                                .map_err(native_error)?;
                            self.drop_policies.borrow_mut().remove(node);
                            Ok(())
                        } else if *property == PropertyId::RichEditBoxDocument {
                            self.set_rich_edit_text(*node, &PropertyValue::Str(String::new()))
                        } else {
                            target.clear(*property)
                        }
                    })();
                    let observation =
                        feedback_event.and_then(|event| {
                            self.feedback.borrow_mut().remove(&(*node, event)).and_then(
                                |expectation| match expectation {
                                    FeedbackExpectation::Normalized { observation } => observation,
                                    FeedbackExpectation::Exact(_)
                                    | FeedbackExpectation::Suppressed => None,
                                },
                            )
                        });
                    (result, observation)
                });
                result?;
                if controlled_collection_for_property(*property).is_some() {
                    self.controlled_collection_indices.remove(node);
                }
                if let Some(observation) = observation {
                    self.events.borrow_mut().push(NativeWork {
                        identity: self.identity.get().unwrap(),
                        work: observation,
                    });
                    self.schedule_dispatch()?;
                }
            }
            Command::SubscribeEvent {
                node,
                event,
                revision,
            } => {
                let content_dialog_closed = *event == EventId::ContentDialogClosed
                    && self.content_dialogs.borrow().contains(*node);
                if self.subscriptions.contains_key(&(*node, *event))
                    || content_dialog_closed
                        && self.content_dialogs.borrow().has_subscription(*node)
                {
                    return Err(RuntimeError::DuplicateEvent(*node, *event));
                }
                let handle = self
                    .handles
                    .get(node)
                    .ok_or(RuntimeError::MissingNode(*node))?;
                let sink = self.event_sink()?;
                let revoker = subscribe_event(handle, *node, *event, *revision, sink)?;
                if content_dialog_closed {
                    self.content_dialogs
                        .borrow_mut()
                        .subscribe(*node, *revision, revoker)?;
                } else {
                    self.subscriptions.insert((*node, *event), revoker);
                }
            }
            Command::UnsubscribeEvent { node, event } => {
                if *event == EventId::ContentDialogClosed
                    && self
                        .content_dialogs
                        .borrow_mut()
                        .unsubscribe(*node, *event)?
                {
                    return Ok(());
                }
                let revoker = self
                    .subscriptions
                    .remove(&(*node, *event))
                    .ok_or(RuntimeError::MissingSubscription(*node, *event))?;
                drop(revoker);
            }
            Command::SetSlot {
                parent,
                slot,
                child,
            } => {
                let child = child.map(|child| self.ui_element(child)).transpose()?;
                let handle = self
                    .handles
                    .get(parent)
                    .ok_or(RuntimeError::MissingNode(*parent))?;
                set_slot(handle, *slot, child.as_ref())?;
            }
            Command::SetTooltip {
                target,
                tooltip,
                placement,
            } => {
                let target = self
                    .ui_element(*target)?
                    .cast::<DependencyObject>()
                    .map_err(native_error)?;
                if let Some(tooltip) = tooltip {
                    ToolTipService::SetToolTip(&target, &self.ui_element(*tooltip)?)
                        .map_err(native_error)?;
                } else {
                    ToolTipService::SetToolTip(&target, None::<&windows_core::IInspectable>)
                        .map_err(native_error)?;
                }
                ToolTipService::SetPlacement(
                    &target,
                    match placement {
                        TooltipPlacement::Top => PlacementMode::Top,
                        TooltipPlacement::Bottom => PlacementMode::Bottom,
                        TooltipPlacement::Left => PlacementMode::Left,
                        TooltipPlacement::Right => PlacementMode::Right,
                        TooltipPlacement::Mouse => PlacementMode::Mouse,
                    },
                )
                .map_err(native_error)?;
            }
            Command::SetFlyout {
                target,
                content,
                placement,
            } => {
                let target_handle = self
                    .handles
                    .get(target)
                    .ok_or(RuntimeError::MissingNode(*target))?;
                if let Some(content) = content {
                    if self
                        .flyouts
                        .iter()
                        .any(|(owner, (_, owned))| owner != target && owned == content)
                    {
                        return Err(RuntimeError::AlreadyParented(*content));
                    }
                    let content_element = self.ui_element(*content)?;
                    let flyout = if let Some((flyout, previous_content)) = self.flyouts.get(target)
                    {
                        if previous_content != content {
                            flyout
                                .SetContent(None::<&UIElement>)
                                .map_err(native_error)?;
                        }
                        flyout.clone()
                    } else {
                        let flyout = bindings::Flyout::new().map_err(native_error)?;
                        match &target_handle {
                            Handle::Button(target) => target.SetFlyout(&flyout),
                            Handle::SplitButton(target) => target.SetFlyout(&flyout),
                            _ => return Err(RuntimeError::UnsupportedKind),
                        }
                        .map_err(native_error)?;
                        flyout
                    };
                    flyout.SetContent(&content_element).map_err(native_error)?;
                    flyout
                        .cast::<IFlyoutBase>()
                        .and_then(|flyout| {
                            flyout.SetPlacement(match placement {
                                FlyoutPlacement::Top => FlyoutPlacementMode::Top,
                                FlyoutPlacement::Bottom => FlyoutPlacementMode::Bottom,
                                FlyoutPlacement::Left => FlyoutPlacementMode::Left,
                                FlyoutPlacement::Right => FlyoutPlacementMode::Right,
                                FlyoutPlacement::Full => FlyoutPlacementMode::Full,
                                FlyoutPlacement::TopEdgeAlignedLeft => {
                                    FlyoutPlacementMode::TopEdgeAlignedLeft
                                }
                                FlyoutPlacement::TopEdgeAlignedRight => {
                                    FlyoutPlacementMode::TopEdgeAlignedRight
                                }
                                FlyoutPlacement::BottomEdgeAlignedLeft => {
                                    FlyoutPlacementMode::BottomEdgeAlignedLeft
                                }
                                FlyoutPlacement::BottomEdgeAlignedRight => {
                                    FlyoutPlacementMode::BottomEdgeAlignedRight
                                }
                                FlyoutPlacement::LeftEdgeAlignedTop => {
                                    FlyoutPlacementMode::LeftEdgeAlignedTop
                                }
                                FlyoutPlacement::LeftEdgeAlignedBottom => {
                                    FlyoutPlacementMode::LeftEdgeAlignedBottom
                                }
                                FlyoutPlacement::RightEdgeAlignedTop => {
                                    FlyoutPlacementMode::RightEdgeAlignedTop
                                }
                                FlyoutPlacement::RightEdgeAlignedBottom => {
                                    FlyoutPlacementMode::RightEdgeAlignedBottom
                                }
                                FlyoutPlacement::Auto => FlyoutPlacementMode::Auto,
                            })
                        })
                        .map_err(native_error)?;
                    self.flyouts.insert(*target, (flyout, *content));
                } else {
                    if let Some((flyout, _)) = self.flyouts.get(target) {
                        flyout
                            .SetContent(None::<&UIElement>)
                            .map_err(native_error)?;
                    }
                    match &target_handle {
                        Handle::Button(target) => target.SetFlyout(None::<&FlyoutBase>),
                        Handle::SplitButton(target) => target.SetFlyout(None::<&FlyoutBase>),
                        _ => return Err(RuntimeError::UnsupportedKind),
                    }
                    .map_err(native_error)?;
                    self.flyouts.remove(target);
                }
            }
            Command::SetOwnedMenu {
                owner,
                target,
                kind,
                items,
                revision,
            } => {
                if let Some(previous) = self.owned_menus.remove(owner) {
                    match previous.kind {
                        OwnedMenuKind::ButtonFlyout | OwnedMenuKind::DropDownButtonFlyout => {
                            let button = self
                                .ui_element(previous.target)?
                                .cast::<IButton>()
                                .map_err(native_error)?;
                            button
                                .SetFlyout(None::<&FlyoutBase>)
                                .map_err(native_error)?;
                        }
                        OwnedMenuKind::MenuBarItem => {
                            let Some(Handle::MenuBarItem(item)) =
                                self.handles.get(&previous.target)
                            else {
                                return Err(RuntimeError::UnsupportedKind);
                            };
                            item.Items()
                                .and_then(|items| items.Clear())
                                .map_err(native_error)?;
                        }
                    }
                }
                let Some(items) = items else {
                    return Ok(());
                };
                let sink = self.event_sink()?;
                let event_owner = *owner;
                let event_revision = *revision;
                let invoke: Rc<dyn Fn(String)> = Rc::new(move |label| {
                    sink.enqueue(
                        event_owner,
                        EventId::OwnedMenuItemInvoked,
                        event_revision,
                        EventPayload::String(label),
                    );
                });
                let mut revokers = Vec::new();
                let flyout = match kind {
                    OwnedMenuKind::ButtonFlyout | OwnedMenuKind::DropDownButtonFlyout => {
                        let flyout = MenuFlyout::new().map_err(native_error)?;
                        build_native_menu_items(
                            items,
                            &flyout.Items().map_err(native_error)?,
                            &mut revokers,
                            &invoke,
                        )
                        .map_err(native_error)?;
                        self.ui_element(*target)?
                            .cast::<IButton>()
                            .and_then(|button| button.SetFlyout(&flyout))
                            .map_err(native_error)?;
                        Some(flyout)
                    }
                    OwnedMenuKind::MenuBarItem => {
                        let Some(Handle::MenuBarItem(item)) = self.handles.get(target) else {
                            return Err(RuntimeError::UnsupportedKind);
                        };
                        build_native_menu_items(
                            items,
                            &item.Items().map_err(native_error)?,
                            &mut revokers,
                            &invoke,
                        )
                        .map_err(native_error)?;
                        None
                    }
                };
                self.owned_menus.insert(
                    *owner,
                    NativeOwnedMenu {
                        target: *target,
                        kind: *kind,
                        _flyout: flyout,
                        _revokers: revokers,
                    },
                );
            }
            Command::SetCommandBarFlyout {
                owner,
                target,
                primary,
                secondary,
                revision,
            } => {
                if let Some(previous) = self.command_bar_flyouts.remove(owner) {
                    self.ui_element(previous.target)?
                        .cast::<IButton>()
                        .and_then(|button| button.SetFlyout(None::<&FlyoutBase>))
                        .map_err(native_error)?;
                }
                let Some(primary) = primary else {
                    return Ok(());
                };
                let sink = self.event_sink()?;
                let flyout = bindings::CommandBarFlyout::new().map_err(native_error)?;
                let primary_items = flyout.PrimaryCommands().map_err(native_error)?;
                let secondary_items = flyout.SecondaryCommands().map_err(native_error)?;
                let mut revokers = Vec::new();
                for command in primary {
                    let (command, revoker) =
                        build_command_bar_element(command, *owner, *revision, &sink)?;
                    primary_items.Append(&command).map_err(native_error)?;
                    revokers.extend(revoker);
                }
                for command in secondary {
                    let (command, revoker) =
                        build_command_bar_element(command, *owner, *revision, &sink)?;
                    secondary_items.Append(&command).map_err(native_error)?;
                    revokers.extend(revoker);
                }
                self.ui_element(*target)?
                    .cast::<IButton>()
                    .and_then(|button| button.SetFlyout(&flyout))
                    .map_err(native_error)?;
                self.command_bar_flyouts.insert(
                    *owner,
                    NativeCommandBarFlyout {
                        target: *target,
                        _flyout: flyout,
                        _revokers: revokers,
                    },
                );
            }
            Command::SetTreeViewNodes { target, nodes } => {
                let Some(Handle::TreeView(tree)) = self.handles.get(target) else {
                    return Err(RuntimeError::UnsupportedKind);
                };
                let roots = tree.RootNodes().map_err(native_error)?;
                roots.Clear().map_err(native_error)?;
                for node in nodes {
                    let node = build_tree_node(node)?;
                    roots.Append(&node).map_err(native_error)?;
                }
            }
            Command::SetSwipeItems {
                owner,
                target,
                left,
                right,
                revision,
            } => {
                self.swipe_attachments.remove(owner);
                let Some(left) = left else {
                    return Ok(());
                };
                let sink = self.event_sink()?;
                let Some(Handle::SwipeControl(control)) = self.handles.get(target) else {
                    return Err(RuntimeError::UnsupportedKind);
                };
                // A fresh SwipeControl reports null for its item collections, so
                // assign populated collections instead of mutating in place.
                let left_items = bindings::SwipeItems::new().map_err(native_error)?;
                let left_vector = left_items
                    .cast::<windows_collections::IVector<bindings::SwipeItem>>()
                    .map_err(native_error)?;
                let right_items = bindings::SwipeItems::new().map_err(native_error)?;
                let right_vector = right_items
                    .cast::<windows_collections::IVector<bindings::SwipeItem>>()
                    .map_err(native_error)?;
                let mut revokers = Vec::new();
                build_swipe_items(left, *owner, *revision, &sink, &left_vector, &mut revokers)?;
                build_swipe_items(
                    right,
                    *owner,
                    *revision,
                    &sink,
                    &right_vector,
                    &mut revokers,
                )?;
                control.SetLeftItems(&left_items).map_err(native_error)?;
                control.SetRightItems(&right_items).map_err(native_error)?;
                self.swipe_attachments.insert(
                    *owner,
                    NativeSwipeItems {
                        target: *target,
                        _revokers: revokers,
                    },
                );
            }
            Command::SetContentDialogOpen { node, owner, open } => {
                let owner = if *open {
                    Some(self.ui_element(*owner)?)
                } else {
                    None
                };
                let xaml_root = match owner.as_ref().map(|owner| owner.XamlRoot()).transpose() {
                    Ok(root) => root,
                    Err(error) if error.code().is_ok() => None,
                    Err(error) => return Err(native_error(error)),
                };
                let action = self
                    .content_dialogs
                    .borrow_mut()
                    .set_open(*node, *open, xaml_root)?;
                match action {
                    ContentDialogAction::None => {}
                    ContentDialogAction::WaitForRoot(generation) => {
                        let sink = self.event_sink()?;
                        let loaded_owner = owner.unwrap();
                        let owner = loaded_owner
                            .cast::<IFrameworkElement>()
                            .map_err(native_error)?;
                        let node_id = *node;
                        let loaded = owner
                            .Loaded(move |_, _| match loaded_owner.XamlRoot() {
                                Ok(root) => {
                                    if let Err(error) =
                                        sink.content_dialog_root_ready(node_id, generation, root)
                                    {
                                        sink.error(node_id, EventId::ContentDialogClosed, 0, error);
                                    }
                                }
                                Err(error) if error.code().is_ok() => {}
                                Err(error) => sink.error(
                                    node_id,
                                    EventId::ContentDialogClosed,
                                    0,
                                    native_error(error),
                                ),
                            })
                            .map_err(native_error)?;
                        self.content_dialogs
                            .borrow_mut()
                            .set_root_loaded(*node, loaded)?;
                    }
                    ContentDialogAction::Hide(dialog) => {
                        dialog.Hide().map_err(native_error)?;
                    }
                }
            }
            Command::InsertChild {
                parent,
                slot,
                child,
                index,
            } => match slot {
                Some(slot) => self.insert_slot_child(*parent, *slot, *child, *index)?,
                None => self.insert_child(*parent, *child, *index)?,
            },
            Command::RemoveChild {
                parent,
                slot,
                child,
            } => match slot {
                Some(slot) => self.remove_slot_child(*parent, *slot, *child)?,
                None => self.remove_child(*parent, *child)?,
            },
            Command::SynchronizeChildren {
                parent,
                slot,
                children,
            } => match slot {
                Some(slot) => self.synchronize_slot_children(*parent, *slot, children)?,
                None => self.synchronize_children(*parent, children)?,
            },
            Command::MoveChild {
                parent,
                slot,
                child,
                index,
            } => match slot {
                Some(slot) => self.move_slot_child(*parent, *slot, *child, *index)?,
                None => self.move_child(*parent, *child, *index)?,
            },
            Command::SetRoutedCallback {
                node,
                event,
                callback,
            } => {
                let mut callbacks = self.routed_callbacks.borrow_mut();
                if let Some(callback) = callback {
                    callbacks.insert((*node, *event), callback.clone());
                } else {
                    callbacks.remove(&(*node, *event));
                }
            }
        }
        Ok(())
    }

    fn contains(&self, node: NodeId) -> bool {
        self.handles.contains_key(&node)
            || self.virtuals.contains_key(&node)
            || self.windows.contains_key(&node)
            || self
                .application
                .as_ref()
                .is_some_and(|(application, _)| *application == node)
    }

    fn window_children(
        &self,
        node: NodeId,
    ) -> Result<Option<(&bindings::Grid, UIElementCollection)>, RuntimeError> {
        if !self.windows.contains_key(&node) {
            return Ok(None);
        }
        let host = self
            .window_hosts
            .get(&node)
            .ok_or(RuntimeError::MissingNode(node))?;
        let children = host
            .cast::<IPanel>()
            .and_then(|host| host.Children())
            .map_err(native_error)?;
        Ok(Some((host, children)))
    }

    fn insert_child(
        &mut self,
        parent: NodeId,
        child: NodeId,
        index: usize,
    ) -> Result<(), RuntimeError> {
        let parent_id = parent;
        let child = self.ui_element(child)?;
        if self.windows.contains_key(&parent) {
            if index != 0 {
                return Err(RuntimeError::IndexOutOfBounds);
            }
            let (host, children) = self.window_children(parent)?.unwrap();
            children.Append(&child).map_err(native_error)?;
            if let Some(visuals) = self.window_visuals.get(&parent) {
                Self::apply_window_theme(host, visuals.theme)?;
            }
            if let Some(observations) = self.window_observations.get(&parent).copied() {
                self.set_window_observations(parent, observations)?;
            }
            Ok(())
        } else {
            let parent = self
                .handles
                .get(&parent)
                .ok_or(RuntimeError::MissingNode(parent))?;
            if parent.is_content() {
                if index != 0 {
                    return Err(RuntimeError::IndexOutOfBounds);
                }
                set_content(parent, Some(&child))
            } else if let Some(children) = parent.child_collection()? {
                let retained = self.retained_identities(parent_id, None)?;
                let index = if retained.is_empty() {
                    index
                } else {
                    let current = (0..children.Size().map_err(native_error)?)
                        .map(|index| {
                            children
                                .GetAt(index)
                                .map_err(native_error)
                                .and_then(|child| com_identity(&child))
                        })
                        .collect::<Result<Vec<_>, RuntimeError>>()?;
                    physical_retained_index(&current, &retained, index)?
                };
                children
                    .InsertAt(index32(index)?, &child)
                    .map_err(native_error)
            } else {
                Err(RuntimeError::UnsupportedKind)
            }
        }
    }

    fn insert_slot_child(
        &mut self,
        parent: NodeId,
        slot: SlotId,
        child: NodeId,
        index: usize,
    ) -> Result<(), RuntimeError> {
        let parent_id = parent;
        let child_id = child;
        let parent = self
            .handles
            .get(&parent)
            .ok_or(RuntimeError::MissingNode(parent))?;
        let collection = slot_collection(parent, slot)?;
        let child: windows_core::IInspectable = self.ui_element(child)?.into();
        let selection = selection_for_slot(slot);
        let selected = selection
            .map(|selection| selected_item(parent, selection))
            .transpose()?
            .flatten();
        let retained = self.retained_identities(parent_id, Some(slot))?;
        let current = (0..collection.Size()?)
            .map(|index| {
                let child = collection.GetAt(index)?;
                com_identity(&child)
            })
            .collect::<Result<Vec<_>, RuntimeError>>()?;
        let index = physical_retained_index(&current, &retained, index)?;
        let result = self.with_controlled_collection_preserved(parent_id, parent, slot, || {
            self.with_selection_suppressed(selection.map(|_| (parent_id, slot)), || {
                collection.InsertAt(index32(index)?, &child)?;
                if let Some(selection) = selection {
                    if selection_item_is_selected(selection, &child)? {
                        set_selected_item(parent, selection, &child)?;
                    } else if let Some(selected) = selected.as_ref() {
                        set_selected_item(parent, selection, selected)?;
                    }
                }
                Ok(())
            })
        });
        if result.is_ok() && selection.is_some() {
            self.selection_owners.insert(child_id, (parent_id, slot));
            self.selection_items.borrow_mut().push((child_id, child));
        }
        result
    }

    fn remove_slot_child(
        &mut self,
        parent: NodeId,
        slot: SlotId,
        child: NodeId,
    ) -> Result<(), RuntimeError> {
        let child_id = child;
        let parent_id = parent;
        let parent = self
            .handles
            .get(&parent)
            .ok_or(RuntimeError::MissingNode(parent))?;
        let collection = slot_collection(parent, slot)?;
        let child: windows_core::IInspectable = self.ui_element(child)?.into();
        let selection = selection_for_slot(slot);
        let selected = selection
            .map(|selection| selected_item(parent, selection))
            .transpose()?
            .flatten();
        let result = self.with_controlled_collection_preserved(parent_id, parent, slot, || {
            self.with_selection_suppressed(selection.map(|_| (parent_id, slot)), || {
                let index = inspectable_child_index(&collection, child_id, &child)?;
                collection.RemoveAt(index)?;
                if let Some(selection) = selection
                    && let Some(selected) = selected.as_ref()
                    && selected != &child
                {
                    set_selected_item(parent, selection, selected)?;
                }
                Ok(())
            })
        });
        if result.is_ok() {
            self.selection_owners.remove(&child_id);
            self.selection_items
                .borrow_mut()
                .retain(|(node, _)| *node != child_id);
        }
        result
    }

    fn move_slot_child(
        &mut self,
        parent: NodeId,
        slot: SlotId,
        child: NodeId,
        index: usize,
    ) -> Result<(), RuntimeError> {
        let child_id = child;
        let parent_id = parent;
        let parent = self
            .handles
            .get(&parent)
            .ok_or(RuntimeError::MissingNode(parent))?;
        let selection = selection_for_slot(slot);
        let selected = selection
            .map(|selection| selected_item(parent, selection))
            .transpose()?
            .flatten();
        let collection = slot_collection(parent, slot)?;
        let child: windows_core::IInspectable = self.ui_element(child)?.into();
        let child_selected = selection
            .map(|selection| selection_item_is_selected(selection, &child))
            .transpose()?
            .unwrap_or(false);
        let child_identity = com_identity(&child)?;
        let retained = self.retained_identities(parent_id, Some(slot))?;
        let current = (0..collection.Size()?)
            .map(|index| {
                let item = collection.GetAt(index)?;
                com_identity(&item)
            })
            .collect::<Result<Vec<_>, RuntimeError>>()?
            .into_iter()
            .filter(|identity| *identity != child_identity)
            .collect::<Vec<_>>();
        let index = physical_retained_index(&current, &retained, index)?;
        self.with_controlled_collection_preserved(parent_id, parent, slot, || {
            self.with_selection_suppressed(selection.map(|_| (parent_id, slot)), || {
                let from = inspectable_child_index(&collection, child_id, &child)?;
                collection.RemoveAt(from)?;
                collection.InsertAt(index32(index)?, &child)?;
                if let Some(selection) = selection {
                    if child_selected {
                        set_selected_item(parent, selection, &child)?;
                    } else if let Some(selected) = selected.as_ref() {
                        set_selected_item(parent, selection, selected)?;
                    }
                }
                Ok(())
            })
        })
    }

    fn synchronize_slot_children(
        &mut self,
        parent: NodeId,
        slot: SlotId,
        children: &[NodeId],
    ) -> Result<(), RuntimeError> {
        let parent_id = parent;
        let parent = self
            .handles
            .get(&parent)
            .ok_or(RuntimeError::MissingNode(parent))?;
        let selection = selection_for_slot(slot);
        let selected = selection
            .map(|selection| selected_item(parent, selection))
            .transpose()?
            .flatten();
        let collection = slot_collection(parent, slot)?;
        let desired = children
            .iter()
            .map(|child| {
                let item: windows_core::IInspectable = self.ui_element(*child)?.into();
                Ok((*child, com_identity(&item)?, item))
            })
            .collect::<Result<Vec<_>, RuntimeError>>()?;
        let current = (0..collection.Size()?)
            .map(|index| {
                let item = collection.GetAt(index)?;
                Ok((com_identity(&item)?, item))
            })
            .collect::<Result<Vec<_>, RuntimeError>>()?;
        let current_ids = current
            .iter()
            .map(|(identity, _)| *identity)
            .collect::<Vec<_>>();
        let desired_ids = desired
            .iter()
            .map(|(_, identity, _)| *identity)
            .collect::<Vec<_>>();
        let retiring = self.retained_identities(parent_id, Some(slot))?;
        let target_ids = merge_retained_identities(&current_ids, &desired_ids, &retiring);
        let target = target_ids
            .iter()
            .map(|identity| {
                desired
                    .iter()
                    .find(|(_, candidate, _)| candidate == identity)
                    .map(|(_, _, item)| (*identity, item.clone()))
                    .or_else(|| {
                        current
                            .iter()
                            .find(|(candidate, _)| candidate == identity)
                            .map(|(_, item)| (*identity, item.clone()))
                    })
                    .unwrap()
            })
            .collect::<Vec<_>>();
        let retained = retained_subsequence(&current_ids, &target_ids);
        let restore_selection = selected.as_ref().is_some_and(|selected| {
            let selected_in_current = current.iter().any(|(_, item)| item == selected);
            !selected_in_current
                || current
                    .iter()
                    .any(|(identity, item)| !retained.contains(identity) && item == selected)
                    && desired.iter().any(|(_, _, item)| item == selected)
        });

        let result = self.with_controlled_collection_preserved(parent_id, parent, slot, || {
            self.with_selection_suppressed(selection.map(|_| (parent_id, slot)), || {
                for (index, (identity, _)) in current.iter().enumerate().rev() {
                    if !retained.contains(identity) {
                        collection.RemoveAt(index32(index)?)?;
                    }
                }
                for (index, (identity, child)) in target.iter().enumerate() {
                    if !retained.contains(identity) {
                        collection.InsertAt(index32(index)?, child)?;
                    }
                }
                if restore_selection && let Some(selected) = selected.as_ref() {
                    set_selected_item(parent, selection.unwrap(), selected)?;
                } else {
                    for (_, _, child) in &desired {
                        if let Some(selection) = selection
                            && selection_item_is_selected(selection, child)?
                        {
                            set_selected_item(parent, selection, child)?;
                            break;
                        }
                    }
                }
                Ok(())
            })
        });
        if result.is_ok() && selection.is_some() {
            let desired_nodes = children.iter().copied().collect::<HashSet<_>>();
            self.selection_items.borrow_mut().retain(|(child, _)| {
                self.selection_owners.get(child) != Some(&(parent_id, slot))
                    || desired_nodes.contains(child)
            });
            self.selection_owners.retain(|child, owner| {
                *owner != (parent_id, slot) || desired_nodes.contains(child)
            });
            let mut known = self
                .selection_items
                .borrow()
                .iter()
                .map(|(node, _)| *node)
                .collect::<HashSet<_>>();
            for (child, _, item) in desired {
                self.selection_owners.insert(child, (parent_id, slot));
                if known.insert(child) {
                    self.selection_items.borrow_mut().push((child, item));
                }
            }
        }
        result
    }

    fn remove_child(&mut self, parent: NodeId, child: NodeId) -> Result<(), RuntimeError> {
        let child_id = child;
        let child = self.ui_element(child)?;
        if let Some((_, children)) = self.window_children(parent)? {
            children.Clear().map_err(native_error)?;
            Ok(())
        } else {
            let parent = self
                .handles
                .get(&parent)
                .ok_or(RuntimeError::MissingNode(parent))?;
            if parent.is_content() {
                set_content(parent, None)
            } else if let Some(children) = parent.child_collection()? {
                let index = child_index(&children, child_id, &child)?;
                children.RemoveAt(index).map_err(native_error)
            } else {
                Err(RuntimeError::UnsupportedKind)
            }
        }
    }

    fn reset_children(&mut self, parent: NodeId) -> Result<(), RuntimeError> {
        if let Some((_, children)) = self.window_children(parent)? {
            children.Clear().map_err(native_error)?;
            Ok(())
        } else {
            let parent = self
                .handles
                .get(&parent)
                .ok_or(RuntimeError::MissingNode(parent))?;
            if parent.is_content() {
                set_content(parent, None)
            } else if let Some(children) = parent.child_collection()? {
                children.Clear().map_err(native_error)
            } else {
                Err(RuntimeError::UnsupportedKind)
            }
        }
    }

    fn synchronize_children(
        &mut self,
        parent: NodeId,
        children: &[NodeId],
    ) -> Result<(), RuntimeError> {
        if self.windows.contains_key(&parent) {
            return match children {
                [] => self.reset_children(parent),
                [child] => {
                    self.reset_children(parent)?;
                    self.insert_child(parent, *child, 0)
                }
                _ => Err(RuntimeError::IndexOutOfBounds),
            };
        }
        if self.handles.get(&parent).is_some_and(Handle::is_content) {
            return match children {
                [] => self.reset_children(parent),
                [child] => self.insert_child(parent, *child, 0),
                _ => Err(RuntimeError::IndexOutOfBounds),
            };
        }

        let parent_handle = self
            .handles
            .get(&parent)
            .ok_or(RuntimeError::MissingNode(parent))?;
        let collection = parent_handle
            .child_collection()?
            .ok_or(RuntimeError::UnsupportedKind)?;
        let desired = children
            .iter()
            .map(|child| {
                let item = self.ui_element(*child)?;
                Ok((com_identity(&item)?, item))
            })
            .collect::<Result<Vec<_>, RuntimeError>>()?;
        let current = (0..collection.Size().map_err(native_error)?)
            .map(|index| {
                let item = collection.GetAt(index).map_err(native_error)?;
                Ok((com_identity(&item)?, item))
            })
            .collect::<Result<Vec<_>, RuntimeError>>()?;
        let current_ids = current
            .iter()
            .map(|(identity, _)| *identity)
            .collect::<Vec<_>>();
        let desired_ids = desired
            .iter()
            .map(|(identity, _)| *identity)
            .collect::<Vec<_>>();
        let retiring = self.retained_identities(parent, None)?;
        let target_ids = merge_retained_identities(&current_ids, &desired_ids, &retiring);
        let target = target_ids
            .iter()
            .map(|identity| {
                desired
                    .iter()
                    .find(|(candidate, _)| candidate == identity)
                    .or_else(|| current.iter().find(|(candidate, _)| candidate == identity))
                    .cloned()
                    .unwrap()
            })
            .collect::<Vec<_>>();
        let retained = retained_subsequence(&current_ids, &target_ids);
        for (index, (identity, _)) in current.iter().enumerate().rev() {
            if !retained.contains(identity) {
                collection.RemoveAt(index32(index)?).map_err(native_error)?;
            }
        }
        for (index, (identity, child)) in target.iter().enumerate() {
            if !retained.contains(identity) {
                collection
                    .InsertAt(index32(index)?, child)
                    .map_err(native_error)?;
            }
        }
        Ok(())
    }

    fn move_child(&self, parent: NodeId, child: NodeId, index: usize) -> Result<(), RuntimeError> {
        let parent_id = parent;
        let child_id = child;
        let child = self.ui_element(child)?;
        if self.windows.contains_key(&parent) {
            if index != 0 {
                return Err(RuntimeError::IndexOutOfBounds);
            }
            Ok(())
        } else {
            let parent = self
                .handles
                .get(&parent)
                .ok_or(RuntimeError::MissingNode(parent))?;
            if parent.is_content() {
                if index == 0 {
                    Ok(())
                } else {
                    Err(RuntimeError::IndexOutOfBounds)
                }
            } else if let Some(children) = parent.child_collection()? {
                let from = child_index(&children, child_id, &child)?;
                let retained = self.retained_identities(parent_id, None)?;
                let index = if retained.is_empty() {
                    index
                } else {
                    let child_identity = com_identity(&child)?;
                    let current = (0..children.Size().map_err(native_error)?)
                        .map(|current| {
                            children
                                .GetAt(current)
                                .map_err(native_error)
                                .and_then(|item| com_identity(&item))
                        })
                        .collect::<Result<Vec<_>, RuntimeError>>()?
                        .into_iter()
                        .filter(|identity| *identity != child_identity)
                        .collect::<Vec<_>>();
                    physical_retained_index(&current, &retained, index)?
                };
                let index = index32(index)?;
                children
                    .cast::<IUIElementCollection>()
                    .and_then(|children| children.Move(from, index))
                    .map_err(native_error)
            } else {
                Err(RuntimeError::UnsupportedKind)
            }
        }
    }

    fn ui_element(&self, node: NodeId) -> Result<UIElement, RuntimeError> {
        if let Some(handle) = self.handles.get(&node) {
            handle.ui_element().map_err(native_error)
        } else if let Some(handle) = self.virtuals.get(&node) {
            handle.ui_element().map_err(native_error)
        } else {
            Err(RuntimeError::MissingNode(node))
        }
    }

    fn retained_identities(
        &self,
        parent: NodeId,
        slot: Option<SlotId>,
    ) -> Result<HashSet<usize>, RuntimeError> {
        self.retained_subtrees
            .iter()
            .filter(|(_, retained)| retained.parent == parent && retained.slot == slot)
            .map(|(root, _)| self.ui_element(*root).and_then(|root| com_identity(&root)))
            .collect()
    }

    fn event_sink(&self) -> Result<EventSink, RuntimeError> {
        let dispatcher = DispatcherQueue::GetForCurrentThread().map_err(native_error)?;
        let identity = self.identity.get().unwrap();
        Ok(EventSink {
            queue: Rc::clone(&self.events),
            errors: Rc::clone(&self.event_errors),
            host_events: Rc::clone(&self.host_events),
            async_ingress: Arc::clone(&self.async_ingress),
            async_state: Rc::clone(&self.async_state),
            drop_policies: Rc::clone(&self.drop_policies),
            encoded_image_nodes: Rc::clone(&self.encoded_image_nodes),
            feedback: Rc::clone(&self.feedback),
            rich_edit_feedback: Rc::clone(&self.rich_edit_feedback),
            content_dialogs: Rc::clone(&self.content_dialogs),
            pending_focus_states: Rc::clone(&self.pending_focus_states),
            pointer_policies: Rc::clone(&self.pointer_policies),
            routed_callbacks: Rc::clone(&self.routed_callbacks),
            selection_items: Rc::clone(&self.selection_items),
            dispatcher,
            identity,
            current_identity: Rc::clone(&self.identity),
            scheduler: Rc::clone(&self.scheduler),
        })
    }

    pub fn schedule_dispatch(&self) -> Result<(), RuntimeError> {
        self.event_sink()?.request(WorkPriority::Low)
    }

    pub fn close_scheduler(&self) {
        self.scheduler.borrow_mut().close();
    }

    fn set_rich_edit_text(&self, node: NodeId, value: &PropertyValue) -> Result<(), RuntimeError> {
        let PropertyValue::Str(value) = value else {
            return Err(RuntimeError::UnsupportedKind);
        };
        let Some(Handle::RichEditBox(control)) = self.handles.get(&node) else {
            return Err(RuntimeError::MissingNode(node));
        };
        if set_rich_edit_text(control, value)? {
            self.rich_edit_feedback
                .borrow_mut()
                .insert(node, value.clone());
        }
        Ok(())
    }

    fn with_controlled_collection_preserved(
        &self,
        owner: NodeId,
        handle: &Handle,
        slot: SlotId,
        apply: impl FnOnce() -> Result<(), RuntimeError>,
    ) -> Result<(), RuntimeError> {
        let Some(collection) = controlled_collection_for_slot(slot) else {
            return apply();
        };
        self.feedback
            .borrow_mut()
            .insert((owner, collection.event), FeedbackExpectation::Suppressed);
        let result = apply().and_then(|()| {
            let Some(index) = self.controlled_collection_indices.get(&owner).copied() else {
                return Ok(());
            };
            if index >= 0 {
                let size = slot_collection(handle, slot)?.Size()?;
                if u32::try_from(index).unwrap() >= size {
                    return Ok(());
                }
            }
            set_property(handle, collection.property, &PropertyValue::I32(index))
        });
        self.feedback
            .borrow_mut()
            .remove(&(owner, collection.event));
        result
    }

    fn with_selection_suppressed<T>(
        &self,
        owner: Option<(NodeId, SlotId)>,
        apply: impl FnOnce() -> T,
    ) -> T {
        let selection = if let Some((owner, slot)) = owner
            && let Some(selection) = selection_for_slot(slot)
        {
            self.feedback
                .borrow_mut()
                .insert((owner, selection.event), FeedbackExpectation::Suppressed);
            Some((owner, selection.event))
        } else {
            None
        };
        let result = apply();
        if let Some(selection) = selection {
            self.feedback.borrow_mut().remove(&selection);
        }
        result
    }
}

#[derive(Clone)]
pub struct EventSink {
    queue: Rc<RefCell<Vec<NativeWork<QueuedEvent>>>>,
    errors: Rc<RefCell<Vec<NativeWork<QueuedEventError>>>>,
    host_events: Rc<RefCell<Vec<NativeWork<HostEvent>>>>,
    async_ingress: Arc<Mutex<AsyncIngressQueue>>,
    async_state: Rc<RefCell<AsyncIngressState>>,
    drop_policies: Rc<RefCell<HashMap<NodeId, DragDropPolicy>>>,
    encoded_image_nodes: Rc<RefCell<HashSet<NodeId>>>,
    feedback: Rc<RefCell<HashMap<(NodeId, EventId), FeedbackExpectation>>>,
    rich_edit_feedback: Rc<RefCell<HashMap<NodeId, String>>>,
    content_dialogs: Rc<RefCell<ContentDialogScheduler>>,
    pending_focus_states: Rc<RefCell<HashMap<NodeId, ElementFocusState>>>,
    pointer_policies: Rc<RefCell<HashMap<NodeId, PointerInteractionPolicy>>>,
    routed_callbacks: Rc<RefCell<HashMap<(NodeId, EventId), RoutedEventCallback>>>,
    selection_items: Rc<RefCell<Vec<(NodeId, windows_core::IInspectable)>>>,
    dispatcher: DispatcherQueue,
    identity: WindowToken,
    current_identity: Rc<Cell<Option<WindowToken>>>,
    scheduler: Rc<RefCell<SchedulerState>>,
}

impl EventSink {
    fn enqueue_host(&self, event: HostEvent) {
        self.host_events.borrow_mut().push(NativeWork {
            identity: self.identity,
            work: event,
        });
        self.schedule();
    }

    fn drag_action(&self, node: NodeId, kind: DragKind) -> Option<DragDropAction> {
        self.drop_policies
            .borrow()
            .get(&node)
            .and_then(|policy| match kind {
                DragKind::StorageItems => policy.storage_items.clone(),
                DragKind::Text => policy.text.clone(),
                DragKind::Unsupported => None,
            })
    }

    fn begin_async_event(
        &self,
        node: NodeId,
        event: EventId,
        revision: u32,
        deferral: DragOperationDeferral,
    ) -> AsyncIngressSender<Result<DroppedData, RuntimeError>> {
        let ticket = {
            let mut state = self.async_state.borrow_mut();
            assert_ne!(state.next_ticket, u64::MAX, "async event ticket exhausted");
            let ticket = state.next_ticket;
            state.next_ticket += 1;
            let cancellation = deferral.clone();
            state.pending.insert(
                ticket,
                PendingAsync {
                    node,
                    cancel: Box::new(move || {
                        _ = cancellation.Complete();
                    }),
                    finalize: Box::new(move |runtime, payload| {
                        let result = *payload
                            .downcast::<Result<DroppedData, RuntimeError>>()
                            .unwrap();
                        let result = deferral.Complete().map_err(native_error).and(result);
                        let identity = runtime.identity.get().unwrap();
                        match result {
                            Ok(value) => runtime.events.borrow_mut().push(NativeWork {
                                identity,
                                work: QueuedEvent::new(
                                    node,
                                    event,
                                    revision,
                                    EventPayload::DroppedData(value),
                                ),
                            }),
                            Err(error) => runtime.event_errors.borrow_mut().push(NativeWork {
                                identity,
                                work: QueuedEventError {
                                    node,
                                    event,
                                    revision,
                                    error,
                                },
                            }),
                        }
                    }),
                },
            );
            ticket
        };
        AsyncIngressSender {
            identity: self.identity,
            ticket,
            ingress: Arc::clone(&self.async_ingress),
            wake: background_dispatch_waker(self.dispatcher.clone(), self.identity),
            marker: PhantomData,
        }
    }

    pub fn apply_pointer_press_policy(
        &self,
        node: NodeId,
        element: &UIElement,
        args: windows_core::InRef<'_, PointerRoutedEventArgs>,
    ) -> Result<bool, RuntimeError> {
        let Some(policy) = self.pointer_policies.borrow().get(&node).copied() else {
            return Ok(false);
        };
        if !policy.capture {
            return Ok(false);
        }
        let Some(args) = args.as_ref() else {
            return Ok(false);
        };
        let pointer = args.Pointer().map_err(native_error)?;
        element.CapturePointer(&pointer).map_err(native_error)
    }

    pub fn apply_pointer_release_policy(
        &self,
        node: NodeId,
        event: EventId,
        revision: u32,
        element: &UIElement,
        args: windows_core::InRef<'_, PointerRoutedEventArgs>,
    ) -> Result<(), RuntimeError> {
        let Some(policy) = self.pointer_policies.borrow().get(&node).copied() else {
            return Ok(());
        };
        if policy.capture
            && let Some(args) = args.as_ref()
        {
            let pointer = args.Pointer().map_err(native_error)?;
            element
                .ReleasePointerCapture(&pointer)
                .map_err(native_error)?;
        }
        if policy.focus_on_release {
            let sink = self.clone();
            let element = element.clone();
            let handler = DispatcherQueueHandler::new(move || {
                if sink.current_identity.get() != Some(sink.identity)
                    || !sink
                        .pointer_policies
                        .borrow()
                        .get(&node)
                        .is_some_and(|policy| policy.focus_on_release)
                {
                    return;
                }
                sink.pending_focus_states
                    .borrow_mut()
                    .insert(node, ElementFocusState::Pointer);
                match element.Focus(FocusState::Pointer).map_err(native_error) {
                    Ok(true) => {
                        let pending = Rc::clone(&sink.pending_focus_states);
                        let cleanup = DispatcherQueueHandler::new(move || {
                            pending.borrow_mut().remove(&node);
                        });
                        match sink
                            .dispatcher
                            .TryEnqueueWithPriority(DispatcherQueuePriority::Low, &cleanup)
                            .map_err(native_error)
                        {
                            Ok(true) => {}
                            Ok(false) => {
                                sink.pending_focus_states.borrow_mut().remove(&node);
                                sink.error(node, event, revision, RuntimeError::DispatcherRejected);
                            }
                            Err(error) => {
                                sink.pending_focus_states.borrow_mut().remove(&node);
                                sink.error(node, event, revision, error);
                            }
                        }
                    }
                    Ok(false) => {
                        sink.pending_focus_states.borrow_mut().remove(&node);
                    }
                    Err(error) => {
                        sink.pending_focus_states.borrow_mut().remove(&node);
                        sink.error(node, event, revision, error);
                    }
                }
            });
            let accepted = self
                .dispatcher
                .TryEnqueueWithPriority(DispatcherQueuePriority::Normal, &handler)
                .map_err(native_error)?;
            if !accepted {
                return Err(RuntimeError::DispatcherRejected);
            }
        }
        Ok(())
    }

    pub fn take_pending_focus_state(&self, node: NodeId) -> Option<ElementFocusState> {
        self.pending_focus_states.borrow_mut().remove(&node)
    }

    fn content_dialog_root_ready(
        &self,
        node: NodeId,
        generation: u64,
        xaml_root: XamlRoot,
    ) -> Result<(), RuntimeError> {
        self.content_dialogs
            .borrow_mut()
            .root_ready(node, generation, xaml_root)
    }

    pub fn content_dialog_closed(
        &self,
        node: NodeId,
        _revision: u32,
    ) -> Result<bool, RuntimeError> {
        content_dialog::closed(&self.content_dialogs, self, node)
    }

    pub fn selection_item(&self, selected: &windows_core::IInspectable) -> Option<NodeId> {
        self.selection_items
            .borrow()
            .iter()
            .find_map(|(node, item)| (item == selected).then_some(*node))
    }

    pub fn enqueue_rich_edit_text(
        &self,
        node: NodeId,
        event: EventId,
        revision: u32,
        value: String,
    ) {
        let mut feedback = self.rich_edit_feedback.borrow_mut();
        let suppress = feedback
            .get(&node)
            .is_some_and(|expected| expected == &value);
        if !suppress {
            feedback.remove(&node);
        }
        drop(feedback);
        if !suppress {
            self.enqueue(node, event, revision, EventPayload::Str(value));
        }
    }

    pub fn enqueue(&self, node: NodeId, event: EventId, revision: u32, payload: EventPayload) {
        if self.encoded_image_nodes.borrow().contains(&node)
            && matches!(event, EventId::ImageImageOpened | EventId::ImageImageFailed)
        {
            return;
        }
        {
            let mut feedback = self.feedback.borrow_mut();
            if let Some(expected) = feedback.get_mut(&(node, event)) {
                match expected {
                    // Keep the expectation active until the setter returns so every synchronous
                    // echo from the same native mutation is covered.
                    FeedbackExpectation::Exact(expected) if expected == &payload => return,
                    FeedbackExpectation::Normalized { observation } => {
                        *observation =
                            Some(QueuedEvent::observation(node, event, revision, payload));
                        return;
                    }
                    FeedbackExpectation::Suppressed => return,
                    FeedbackExpectation::Exact(_) => {}
                }
            }
        }
        self.queue.borrow_mut().push(NativeWork {
            identity: self.identity,
            work: QueuedEvent::new(node, event, revision, payload),
        });
        self.schedule();
    }

    pub fn route_key(
        &self,
        node: NodeId,
        event: EventId,
        revision: u32,
        value: KeyEventInfo,
    ) -> bool {
        self.route(
            node,
            event,
            revision,
            EventPayload::KeyEventInfo(value),
            |callback| callback.key(value),
        )
    }

    pub fn route_character(
        &self,
        node: NodeId,
        event: EventId,
        revision: u32,
        value: CharacterEventInfo,
    ) -> bool {
        self.route(
            node,
            event,
            revision,
            EventPayload::CharacterEventInfo(value),
            |callback| callback.character(value),
        )
    }

    fn route(
        &self,
        node: NodeId,
        event: EventId,
        revision: u32,
        payload: EventPayload,
        invoke: impl FnOnce(&RoutedEventCallback) -> Option<RoutedDispatch>,
    ) -> bool {
        if self.current_identity.get() != Some(self.identity) {
            return false;
        }
        let callback = self.routed_callbacks.borrow().get(&(node, event)).cloned();
        let Some(callback) = callback else {
            return false;
        };
        let dispatch =
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| invoke(&callback))) {
                Ok(Some(dispatch)) => dispatch,
                Ok(None) => return false,
                Err(_) => {
                    // 同上：路由事件回调 panic → 静默 abort 会抹掉现场。
                    eprintln!(
                        "windows-reactor fatal: routed event callback panicked \
                         (node={node:?} event={event:?} rev={revision}), aborting"
                    );
                    std::process::abort();
                }
            };
        if let Some(message) = dispatch.message {
            self.queue.borrow_mut().push(NativeWork {
                identity: self.identity,
                work: QueuedEvent::routed(
                    node,
                    event,
                    revision,
                    payload,
                    message,
                    dispatch.handled,
                ),
            });
            self.schedule();
        }
        dispatch.handled
    }

    pub fn observe(&self, node: NodeId, event: EventId, revision: u32, payload: EventPayload) {
        self.queue.borrow_mut().push(NativeWork {
            identity: self.identity,
            work: QueuedEvent::observation(node, event, revision, payload),
        });
        self.schedule();
    }

    pub fn enqueue_or_observe(
        &self,
        node: NodeId,
        event: EventId,
        revision: u32,
        payload: EventPayload,
        invoke_callback: bool,
    ) {
        if invoke_callback {
            self.enqueue(node, event, revision, payload);
        } else {
            self.observe(node, event, revision, payload);
        }
    }

    pub fn error(&self, node: NodeId, event: EventId, revision: u32, error: RuntimeError) {
        self.errors.borrow_mut().push(NativeWork {
            identity: self.identity,
            work: QueuedEventError {
                node,
                event,
                revision,
                error,
            },
        });
        self.schedule();
    }

    pub fn wake(&self) {
        self.schedule();
    }

    fn schedule(&self) {
        match self.request(WorkPriority::Normal) {
            Ok(()) | Err(RuntimeError::SchedulerClosed) => {}
            Err(error) => fail_native_scheduler(error),
        }
    }

    fn request(&self, priority: WorkPriority) -> Result<(), RuntimeError> {
        let action = self.scheduler.borrow_mut().request(priority);
        Self::perform(
            action,
            self.identity,
            &self.current_identity,
            &self.scheduler,
            &self.dispatcher,
        )
    }

    fn perform(
        action: ScheduleAction,
        identity: WindowToken,
        current_identity: &Rc<Cell<Option<WindowToken>>>,
        scheduler: &Rc<RefCell<SchedulerState>>,
        dispatcher: &DispatcherQueue,
    ) -> Result<(), RuntimeError> {
        let ScheduleAction::Enqueue(ticket) = action else {
            return match action {
                ScheduleAction::Closed => Err(RuntimeError::SchedulerClosed),
                _ => Ok(()),
            };
        };
        let current_identity_capture = Rc::clone(current_identity);
        let scheduler_capture = Rc::clone(scheduler);
        let dispatcher_capture = dispatcher.clone();
        let handler = DispatcherQueueHandler::new(move || {
            if current_identity_capture.get() != Some(identity) {
                let action = {
                    let mut scheduler = scheduler_capture.borrow_mut();
                    if !scheduler.begin_dispatch(ticket) {
                        return;
                    }
                    _ = scheduler.request(WorkPriority::Normal);
                    scheduler.finish_dispatch()
                };
                if let Some(identity) = current_identity_capture.get()
                    && let Err(error) = Self::perform(
                        action,
                        identity,
                        &current_identity_capture,
                        &scheduler_capture,
                        &dispatcher_capture,
                    )
                    && error != RuntimeError::SchedulerClosed
                {
                    fail_native_scheduler(error);
                }
                return;
            }
            if !scheduler_capture.borrow_mut().begin_dispatch(ticket) {
                return;
            }
            dispatch_native_events(identity);
            let action = scheduler_capture.borrow_mut().finish_dispatch();
            if let Some(identity) = current_identity_capture.get()
                && let Err(error) = Self::perform(
                    action,
                    identity,
                    &current_identity_capture,
                    &scheduler_capture,
                    &dispatcher_capture,
                )
                && error != RuntimeError::SchedulerClosed
            {
                fail_native_scheduler(error);
            }
        });
        let priority = match ticket.priority {
            WorkPriority::Low => DispatcherQueuePriority::Low,
            WorkPriority::Normal => DispatcherQueuePriority::Normal,
        };
        match dispatcher.TryEnqueueWithPriority(priority, &handler) {
            Ok(true) => Ok(()),
            Ok(false) => {
                scheduler.borrow_mut().enqueue_failed(ticket);
                Err(RuntimeError::DispatcherRejected)
            }
            Err(error) => {
                scheduler.borrow_mut().enqueue_failed(ticket);
                Err(native_error(error))
            }
        }
    }
}

fn com_identity(value: &impl Interface) -> Result<usize, RuntimeError> {
    value
        .cast::<windows_core::IUnknown>()
        .map(|identity| identity.as_raw() as usize)
        .map_err(native_error)
}

fn physical_retained_index(
    current: &[usize],
    retained: &HashSet<usize>,
    semantic_index: usize,
) -> Result<usize, RuntimeError> {
    let mut live = 0;
    for (physical, identity) in current.iter().enumerate() {
        if retained.contains(identity) {
            continue;
        }
        if live == semantic_index {
            return Ok(physical);
        }
        live += 1;
    }
    if live == semantic_index {
        Ok(current.len())
    } else {
        Err(RuntimeError::IndexOutOfBounds)
    }
}

fn merge_retained_identities(
    current: &[usize],
    desired: &[usize],
    retained: &HashSet<usize>,
) -> Vec<usize> {
    let desired_set = desired.iter().copied().collect::<HashSet<_>>();
    let mut before = HashMap::<usize, Vec<usize>>::new();
    let mut trailing = Vec::new();
    for (index, identity) in current.iter().copied().enumerate() {
        if !retained.contains(&identity) {
            continue;
        }
        if let Some(successor) = current[index + 1..]
            .iter()
            .copied()
            .find(|candidate| desired_set.contains(candidate))
        {
            before.entry(successor).or_default().push(identity);
        } else {
            trailing.push(identity);
        }
    }
    let mut merged = Vec::with_capacity(desired.len() + retained.len());
    for identity in desired.iter().copied() {
        if let Some(retained) = before.remove(&identity) {
            merged.extend(retained);
        }
        merged.push(identity);
    }
    merged.extend(trailing);
    merged
}

fn retained_subsequence(current: &[usize], desired: &[usize]) -> HashSet<usize> {
    let positions = current
        .iter()
        .copied()
        .enumerate()
        .map(|(index, identity)| (identity, index))
        .collect::<HashMap<_, _>>();
    let sequence = desired
        .iter()
        .filter_map(|identity| {
            positions
                .get(identity)
                .map(|position| (*identity, *position))
        })
        .collect::<Vec<_>>();
    let mut tails = Vec::<usize>::new();
    let mut previous = vec![None; sequence.len()];
    for (index, (_, position)) in sequence.iter().enumerate() {
        let slot = tails.partition_point(|tail| sequence[*tail].1 < *position);
        if slot > 0 {
            previous[index] = Some(tails[slot - 1]);
        }
        if slot == tails.len() {
            tails.push(index);
        } else {
            tails[slot] = index;
        }
    }
    let mut retained = HashSet::new();
    let Some(mut index) = tails.last().copied() else {
        return retained;
    };
    loop {
        retained.insert(sequence[index].0);
        let Some(next) = previous[index] else {
            break;
        };
        index = next;
    }
    retained
}

fn child_index(
    children: &UIElementCollection,
    child_id: NodeId,
    child: &UIElement,
) -> Result<u32, RuntimeError> {
    let mut index = 0;
    children
        .IndexOf(child, &mut index)
        .map_err(native_error)?
        .then_some(index)
        .ok_or(RuntimeError::ChildNotFound(child_id))
}

#[cfg(test)]
mod tests;

fn inspectable_child_index(
    children: &SlotCollection,
    child_id: NodeId,
    child: &windows_core::IInspectable,
) -> Result<u32, RuntimeError> {
    let size = children.Size()?;
    (0..size)
        .find(|index| children.GetAt(*index).as_ref() == Ok(child))
        .ok_or(RuntimeError::ChildNotFound(child_id))
}

fn index32(index: usize) -> Result<u32, RuntimeError> {
    index.try_into().map_err(|_| RuntimeError::IndexOutOfBounds)
}

fn native_selection_index(value: Option<usize>) -> Result<i32, RuntimeError> {
    value.map_or(Ok(-1), |value| {
        value.try_into().map_err(|_| RuntimeError::IndexOutOfBounds)
    })
}

/// FontFamily objects cached by name: a FontFamily is a WinRT COM object, and
/// rebuilding one per text control on every render would churn allocations on
/// text-heavy pages. The cache only runs on the UI thread; the mutex guards
/// insertion.
fn cached_font_family(name: &str) -> windows_core::Result<FontFamily> {
    static CACHE: std::sync::OnceLock<Mutex<HashMap<String, FontFamily>>> =
        std::sync::OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = cache.lock().unwrap();
    if let Some(family) = guard.get(name) {
        return Ok(family.clone());
    }
    let family = FontFamily::CreateInstanceWithName(name)?;
    guard.insert(name.to_string(), family.clone());
    Ok(family)
}

fn selection_index(value: i32) -> Result<Option<usize>, RuntimeError> {
    match value {
        -1 => Ok(None),
        0.. => Ok(Some(value as usize)),
        _ => Err(RuntimeError::IndexOutOfBounds),
    }
}

fn native_number_box_value(value: Option<f64>) -> f64 {
    value.unwrap_or(f64::NAN)
}

fn number_box_value(value: f64) -> Option<f64> {
    (!value.is_nan()).then_some(value)
}

fn native_rating_value(value: Option<f64>) -> f64 {
    value.unwrap_or(-1.0)
}

fn rating_value(value: f64) -> Option<f64> {
    (value != -1.0).then_some(value)
}

fn is_internal_detach(command: &Command, destroyed: &HashSet<NodeId>) -> bool {
    match command {
        Command::RemoveChild { parent, child, .. } => {
            destroyed.contains(parent) && destroyed.contains(child)
        }
        Command::SetSlot {
            parent,
            child: None,
            ..
        } => destroyed.contains(parent),
        _ => false,
    }
}

impl NativeRuntime for WinUiRuntime {
    fn apply(&mut self, commands: &[Command]) -> Result<(), NativeApplyError> {
        #[cfg(feature = "test")]
        let started = std::time::Instant::now();
        // Detach only the external edge of a subtree so WinUI cannot run deferred callbacks
        // against a control whose internal children have already been removed.
        let destroyed = commands
            .iter()
            .filter_map(|command| match command {
                Command::Destroy { node } => Some(*node),
                _ => None,
            })
            .collect::<HashSet<_>>();
        for (index, command) in commands.iter().enumerate() {
            if is_internal_detach(command, &destroyed) {
                continue;
            }
            if let Err(error) = self.apply_one(command) {
                self.routed_callbacks.borrow_mut().clear();
                eprintln!("windows-reactor failed command {index}: {command:?}: {error:?}");
                return Err(NativeApplyError {
                    command: index,
                    error,
                });
            }
        }
        #[cfg(feature = "test")]
        self.native_apply_times_us
            .push(started.elapsed().as_secs_f64() * 1_000_000.0);
        Ok(())
    }

    fn open_windows(&mut self, roots: Vec<View>) -> Result<(), RuntimeError> {
        open_live_windows(roots)
    }

    fn window_handle(&self, node: NodeId) -> Result<isize, RuntimeError> {
        let window = self
            .windows
            .get(&node)
            .ok_or(RuntimeError::MissingNode(node))?;
        let mut hwnd = std::ptr::null_mut();
        unsafe {
            window
                .cast::<IWindowNative>()
                .map_err(native_error)?
                .WindowHandle(&mut hwnd)
                .ok()
                .map_err(native_error)?;
        }
        Ok(hwnd as isize)
    }

    fn reset(&mut self) {
        for root in self.retained_subtrees.keys().copied().collect::<Vec<_>>() {
            _ = self.cancel_retirement(root);
        }
        self.clear_async_ingress();
        for node in self
            .webview_initializations
            .borrow()
            .keys()
            .copied()
            .collect::<Vec<_>>()
        {
            complete_webview_initialization(
                &self.webview_initializations,
                node,
                Err(RuntimeError::MissingNode(node)),
            );
        }
        self.subscriptions.clear();
        self.encoded_image_nodes.borrow_mut().clear();
        self.image_decode_tickets.clear();
        self.observation_subscriptions.clear();
        self.scale_center_subscriptions.clear();
        self.owned_menus.clear();
        self.command_bar_flyouts.clear();
        self.swipe_attachments.clear();
        content_dialog::reset(&self.content_dialogs);
        for (target, (flyout, _)) in self.flyouts.drain() {
            _ = flyout.SetContent(None::<&UIElement>);
            match self.handles.get(&target) {
                Some(Handle::Button(button)) => {
                    _ = button.SetFlyout(None::<&FlyoutBase>);
                }
                Some(Handle::SplitButton(button)) => {
                    _ = button.SetFlyout(None::<&FlyoutBase>);
                }
                _ => {}
            }
        }
        if self.window_closed.get() {
            for (_, subscription) in self.window_subscriptions.drain() {
                subscription.into_token();
            }
        } else {
            self.window_subscriptions.clear();
            for window in self.windows.values() {
                _ = window.Close();
            }
        }
        self.windows.clear();
        self.window_observations.clear();
        self.window_hosts.clear();
        self.window_observation_subscriptions.clear();
        self.window_title_bars.clear();
        self.window_title_revisions.borrow_mut().clear();
        self.window_visuals.clear();
        self.theme_styles.clear();
        self.handles.clear();
        self.virtuals.clear();
        self.application = None;
        self.pending_application = None;
        self.event_errors.borrow_mut().clear();
        self.events.borrow_mut().clear();
        self.host_events.borrow_mut().clear();
        self.feedback.borrow_mut().clear();
        self.rich_edit_feedback.borrow_mut().clear();
        self.drop_policies.borrow_mut().clear();
        self.pending_focus_states.borrow_mut().clear();
        self.pointer_policies.borrow_mut().clear();
        self.routed_callbacks.borrow_mut().clear();
        self.resource_override_keys.clear();
        self.controlled_collection_indices.clear();
        self.selection_owners.clear();
        self.selection_items.borrow_mut().clear();
        self.realizations.borrow_mut().clear();
        self.window_closed.set(false);
    }

    fn native_window_closed(&mut self) {
        for root in self.retained_subtrees.keys().copied().collect::<Vec<_>>() {
            _ = self.cancel_retirement(root);
        }
        self.clear_async_ingress();
        self.window_observation_subscriptions.clear();
        self.window_title_revisions.borrow_mut().clear();
        for (_, subscription) in self.window_subscriptions.drain() {
            subscription.into_token();
        }
    }

    fn component_waker(&self) -> Option<Rc<dyn Fn()>> {
        let sink = self.event_sink().ok()?;
        Some(Rc::new(move || sink.wake()))
    }

    fn component_background_waker(&self) -> Option<Arc<dyn Fn() -> bool + Send + Sync>> {
        let dispatcher = DispatcherQueue::GetForCurrentThread().ok()?;
        let identity = self.identity.get()?;
        Some(background_dispatch_waker(dispatcher, identity))
    }

    fn set_identity(&mut self, identity: WindowToken) {
        if self
            .identity
            .get()
            .is_none_or(|current| current != identity)
        {
            self.scheduler.borrow_mut().open();
        }
        self.identity.set(Some(identity));
        let mut ingress = self.async_ingress.lock().unwrap();
        ingress.identity = Some(identity);
        ingress.completions.clear();
    }

    fn drain_events(&mut self) -> Vec<NativeWork<QueuedEvent>> {
        self.flush_async_ingress();
        self.events.borrow_mut().drain(..).collect()
    }

    fn drain_event_errors(&mut self) -> Vec<NativeWork<QueuedEventError>> {
        self.event_errors.borrow_mut().drain(..).collect()
    }

    fn drain_host_events(&mut self) -> Vec<NativeWork<HostEvent>> {
        self.host_events.borrow_mut().drain(..).collect()
    }

    fn drain_realizations(&mut self) -> Vec<NativeWork<RealizationRequest>> {
        self.realizations.borrow_mut().drain(..).collect()
    }
}

impl WinUiRuntime {
    fn register_async_completion<T: Send + 'static>(
        &mut self,
        node: NodeId,
        cancel: impl FnOnce() + 'static,
        finalize: impl FnOnce(&mut Self, T) + 'static,
    ) -> Result<(u64, AsyncIngressSender<T>), RuntimeError> {
        let identity = self
            .identity
            .get()
            .ok_or(RuntimeError::MissingApplication)?;
        let ticket = {
            let mut state = self.async_state.borrow_mut();
            if state.next_ticket == u64::MAX {
                return Err(RuntimeError::UnsupportedKind);
            }
            let ticket = state.next_ticket;
            state.next_ticket += 1;
            state.pending.insert(
                ticket,
                PendingAsync {
                    node,
                    cancel: Box::new(cancel),
                    finalize: Box::new(move |runtime, payload| {
                        let payload = payload.downcast::<T>().unwrap();
                        finalize(runtime, *payload);
                    }),
                },
            );
            ticket
        };
        let dispatcher = DispatcherQueue::GetForCurrentThread().map_err(native_error)?;
        Ok((
            ticket,
            AsyncIngressSender {
                identity,
                ticket,
                ingress: Arc::clone(&self.async_ingress),
                wake: background_dispatch_waker(dispatcher, identity),
                marker: PhantomData,
            },
        ))
    }

    fn start_encoded_image(
        &mut self,
        node: NodeId,
        image: &EncodedImage,
    ) -> Result<(), RuntimeError> {
        match self.handles.get(&node) {
            Some(Handle::Image(_) | Handle::ImageIcon(_)) => {}
            Some(_) => return Err(RuntimeError::UnsupportedKind),
            None => return Err(RuntimeError::MissingNode(node)),
        }

        let stream = InMemoryRandomAccessStream::new().map_err(native_error)?;
        let output = stream.GetOutputStreamAt(0).map_err(native_error)?;
        let writer = DataWriter::CreateDataWriter(&output).map_err(native_error)?;
        writer.WriteBytes(image.as_bytes()).map_err(native_error)?;
        let operation = writer.StoreAsync().map_err(native_error)?;
        let cancel_operation = operation.clone();
        let (ticket, sender) = self.register_async_completion(
            node,
            move || {
                _ = cancel_operation.Cancel();
            },
            move |runtime, result: Result<u32, RuntimeError>| {
                runtime.image_decode_tickets.remove(&node);
                if let Err(error) = result {
                    runtime.report_image_decode_error(error);
                    return;
                }
                if let Err(error) = runtime.start_bitmap_decode(node, stream, writer) {
                    runtime.report_image_decode_error(error);
                }
            },
        )?;
        self.image_decode_tickets.insert(node, ticket);
        if let Err(error) = operation.when(move |result| {
            _ = sender.complete(result.map_err(native_error));
        }) {
            self.cancel_image_decode(node);
            return Err(native_error(error));
        }
        Ok(())
    }

    fn start_bitmap_decode(
        &mut self,
        node: NodeId,
        stream: InMemoryRandomAccessStream,
        writer: DataWriter,
    ) -> Result<(), RuntimeError> {
        _ = writer.DetachStream().map_err(native_error)?;
        stream.Seek(0).map_err(native_error)?;
        let bitmap = BitmapImage::new().map_err(native_error)?;
        let operation = bitmap
            .cast::<IBitmapSource>()
            .map_err(native_error)?
            .SetSourceAsync(&stream)
            .map_err(native_error)?;
        let cancel_operation = operation.clone();
        let (ticket, sender) = self.register_async_completion(
            node,
            move || {
                _ = cancel_operation.Cancel();
            },
            move |runtime, result: Result<(), RuntimeError>| {
                runtime.image_decode_tickets.remove(&node);
                match result {
                    Ok(()) => {
                        if let Err(error) = runtime.set_decoded_image(node, &bitmap) {
                            runtime.report_image_decode_error(error);
                        } else {
                            runtime.enqueue_image_decode_event(node, EventId::ImageImageOpened);
                        }
                    }
                    Err(_) => {
                        runtime.enqueue_image_decode_event(node, EventId::ImageImageFailed);
                    }
                }
                drop(stream);
            },
        )?;
        self.image_decode_tickets.insert(node, ticket);
        if let Err(error) = operation.when(move |result| {
            _ = sender.complete(result.map_err(native_error));
        }) {
            self.cancel_image_decode(node);
            return Err(native_error(error));
        }
        Ok(())
    }

    fn set_decoded_image(&self, node: NodeId, bitmap: &BitmapImage) -> Result<(), RuntimeError> {
        let source = bitmap.cast::<ImageSource>().map_err(native_error)?;
        match self.handles.get(&node) {
            Some(Handle::Image(control)) => control.SetSource(&source).map_err(native_error),
            Some(Handle::ImageIcon(control)) => control.SetSource(&source).map_err(native_error),
            Some(_) => Err(RuntimeError::UnsupportedKind),
            None => Err(RuntimeError::MissingNode(node)),
        }
    }

    fn cancel_image_decode(&mut self, node: NodeId) {
        let Some(ticket) = self.image_decode_tickets.remove(&node) else {
            return;
        };
        let pending = self.async_state.borrow_mut().pending.remove(&ticket);
        if let Some(pending) = pending {
            (pending.cancel)();
        }
    }

    fn release_encoded_image_source(&mut self, node: NodeId) {
        self.cancel_image_decode(node);
        self.purge_image_events(node);
        self.encoded_image_nodes.borrow_mut().remove(&node);
    }

    fn report_image_decode_error(&mut self, error: RuntimeError) {
        let Some(identity) = self.identity.get() else {
            return;
        };
        self.host_events.borrow_mut().push(NativeWork {
            identity,
            work: HostEvent::Error(error),
        });
    }

    fn enqueue_image_decode_event(&mut self, node: NodeId, event: EventId) -> bool {
        let Some(NativeSubscription::Event { revision, .. }) =
            self.subscriptions.get(&(node, event))
        else {
            return false;
        };
        self.events.borrow_mut().push(NativeWork {
            identity: self.identity.get().unwrap(),
            work: QueuedEvent::new(node, event, *revision, EventPayload::Unit),
        });
        true
    }

    fn purge_image_events(&mut self, node: NodeId) {
        self.events.borrow_mut().retain(|event| {
            event.work.node != node
                || !matches!(
                    event.work.event,
                    EventId::ImageImageOpened | EventId::ImageImageFailed
                )
        });
    }

    fn start_retirement(
        &mut self,
        root: NodeId,
        nodes: Vec<NodeId>,
        parent: NodeId,
        slot: Option<SlotId>,
        transition: ExitTransition,
    ) -> Result<(), RuntimeError> {
        if self.retained_subtrees.contains_key(&root)
            || nodes.iter().any(|node| !self.contains(*node))
        {
            return Err(RuntimeError::MissingNode(root));
        }
        for node in &nodes {
            self.cancel_image_decode(*node);
            self.purge_image_events(*node);
        }
        let duration =
            TimeSpan::try_from(transition.duration()).map_err(|_| RuntimeError::UnsupportedKind)?;
        let root_element = self.ui_element(root)?;
        let opacity_transition = ScalarTransition::new().map_err(native_error)?;
        opacity_transition
            .SetDuration(duration)
            .map_err(native_error)?;
        root_element
            .SetOpacityTransition(&opacity_transition)
            .map_err(native_error)?;

        let timer = DispatcherQueue::GetForCurrentThread()
            .and_then(|dispatcher| dispatcher.CreateTimer())
            .map_err(native_error)?;
        timer.SetInterval(duration).map_err(native_error)?;
        timer.SetIsRepeating(false).map_err(native_error)?;
        let identity = self
            .identity
            .get()
            .ok_or(RuntimeError::MissingApplication)?;
        let (ticket, sender) = self.register_async_completion(
            root,
            || {},
            move |runtime, ()| {
                if let Err(error) = runtime.finish_retirement(root) {
                    runtime.host_events.borrow_mut().push(NativeWork {
                        identity,
                        work: HostEvent::Error(error),
                    });
                }
            },
        )?;
        let sender = Rc::new(RefCell::new(Some(sender)));
        let tick_sender = Rc::clone(&sender);
        let tick = timer
            .Tick(move |_, _| {
                if let Some(sender) = tick_sender.borrow_mut().take() {
                    _ = sender.complete(());
                }
            })
            .map_err(native_error)?;
        self.retained_subtrees.insert(
            root,
            NativeRetainedSubtree {
                nodes,
                parent,
                slot,
                ticket,
                _timer: timer.clone(),
                _tick: tick,
            },
        );
        if let Err(error) = root_element.SetOpacity(0.0).map_err(native_error) {
            self.cancel_retirement(root)?;
            return Err(error);
        }
        if let Err(error) = timer.Start().map_err(native_error) {
            self.cancel_retirement(root)?;
            return Err(error);
        }
        Ok(())
    }

    fn finish_retirement(&mut self, root: NodeId) -> Result<(), RuntimeError> {
        let Some(retained) = self.retained_subtrees.remove(&root) else {
            return Ok(());
        };
        retained._timer.Stop().map_err(native_error)?;
        match retained.slot {
            Some(slot) => self.remove_slot_child(retained.parent, slot, root)?,
            None => self.remove_child(retained.parent, root)?,
        }
        for node in retained.nodes {
            self.remove_node_state(node)?;
        }
        Ok(())
    }

    fn cancel_retirement(&mut self, root: NodeId) -> Result<(), RuntimeError> {
        let ticket = self
            .retained_subtrees
            .get(&root)
            .map(|retained| retained.ticket);
        let pending =
            ticket.and_then(|ticket| self.async_state.borrow_mut().pending.remove(&ticket));
        if let Some(pending) = pending {
            (pending.cancel)();
        }
        self.finish_retirement(root)
    }

    fn cancel_retirements_for_node(&mut self, node: NodeId) -> Result<(), RuntimeError> {
        let roots = self
            .retained_subtrees
            .iter()
            .filter_map(|(root, retained)| {
                (retained.parent == node || retained.nodes.contains(&node)).then_some(*root)
            })
            .collect::<Vec<_>>();
        for root in roots {
            self.cancel_retirement(root)?;
        }
        Ok(())
    }

    /// `Border::scale` 的缩放原点：`CenterPoint` 是**像素**坐标，只能由元素测量后的
    /// `ActualWidth/Height` 反算。下发 `BorderScale` 时元素可能刚挂载、尚未测量
    /// （尺寸读到 0，原点会落到左上角——以"已激活"状态挂载的元素整体偏向右下）；
    /// 元素尺寸之后还可能变（窗口缩放、瀑布流列宽变化），原点也必须跟上。
    ///
    /// 这里按元素挂一个 `SizeChanged`：下发时能读到尺寸就立即设一次，之后每次
    /// 尺寸变化按 `NewSize` 重设。订阅按元素去重，随 [`Self::remove_node_state`]
    /// 一起释放。
    fn sync_scale_center(&mut self, node: NodeId) -> Result<(), RuntimeError> {
        if self.scale_center_subscriptions.contains_key(&node) {
            return Ok(());
        }
        let Some(Handle::Border(control)) = self.handles.get(&node) else {
            return Ok(());
        };
        let element = control.cast::<IUIElement>().map_err(native_error)?;
        let framework = control.cast::<IFrameworkElement>().map_err(native_error)?;
        apply_scale_center(&element, &framework)?;
        // 闭包**不捕获元素**：`SizeChanged` 的 sender 就是元素本身，捕获它会让
        // "元素 → 事件源 → 闭包 → 元素" 形成 COM 引用环，元素永不释放。
        let revoker = framework
            .SizeChanged(move |sender, args| {
                let (Some(sender), Some(args)) = (sender.as_ref(), args.as_ref()) else {
                    return;
                };
                let Ok(size) = args.NewSize() else {
                    return;
                };
                let Ok(element) = sender.cast::<IUIElement>() else {
                    return;
                };
                let _ = element.SetCenterPoint(Vector3 {
                    x: size.width / 2.0,
                    y: size.height / 2.0,
                    z: 0.0,
                });
            })
            .map_err(native_error)?;
        self.scale_center_subscriptions.insert(node, revoker);
        Ok(())
    }

    fn remove_node_state(&mut self, node: NodeId) -> Result<(), RuntimeError> {
        self.release_encoded_image_source(node);
        self.feedback
            .borrow_mut()
            .retain(|(feedback_node, _), _| *feedback_node != node);
        self.rich_edit_feedback.borrow_mut().remove(&node);
        self.subscriptions
            .retain(|(subscription_node, _), _| *subscription_node != node);
        self.cancel_async_for_node(node);
        self.window_subscriptions.remove(&node);
        self.window_observation_subscriptions.remove(&node);
        self.scale_center_subscriptions.remove(&node);
        self.window_observations.remove(&node);
        self.window_hosts.remove(&node);
        self.window_title_bars.remove(&node);
        self.window_title_revisions.borrow_mut().remove(&node);
        self.window_visuals.remove(&node);
        complete_webview_initialization(
            &self.webview_initializations,
            node,
            Err(RuntimeError::MissingNode(node)),
        );
        self.controlled_collection_indices.remove(&node);
        self.observation_subscriptions
            .retain(|(subscription_node, _), _| *subscription_node != node);
        self.drop_policies.borrow_mut().remove(&node);
        self.pending_focus_states.borrow_mut().remove(&node);
        self.pointer_policies.borrow_mut().remove(&node);
        self.routed_callbacks
            .borrow_mut()
            .retain(|(callback_node, _), _| *callback_node != node);
        if self.resource_override_keys.contains_key(&node) {
            self.clear_resource_overrides(node)?;
        }
        content_dialog::retire(&self.content_dialogs, node);
        self.selection_owners
            .retain(|child, (parent, _)| *child != node && *parent != node);
        self.selection_items
            .borrow_mut()
            .retain(|(child, _)| *child != node);
        if self.handles.remove(&node).is_some()
            || self.virtuals.remove(&node).is_some()
            || self.windows.remove(&node).is_some()
        {
            return Ok(());
        }
        if self
            .application
            .as_ref()
            .is_some_and(|(application, _)| *application == node)
        {
            self.application = None;
            return Ok(());
        }
        Err(RuntimeError::MissingNode(node))
    }

    fn cancel_async_for_node(&mut self, node: NodeId) {
        let mut state = self.async_state.borrow_mut();
        let tickets = state
            .pending
            .iter()
            .filter_map(|(ticket, pending)| (pending.node == node).then_some(*ticket))
            .collect::<Vec<_>>();
        let cancellations = tickets
            .into_iter()
            .filter_map(|ticket| state.pending.remove(&ticket))
            .map(|pending| pending.cancel)
            .collect::<Vec<_>>();
        drop(state);
        for cancel in cancellations {
            cancel();
        }
    }

    fn clear_async_ingress(&mut self) {
        {
            let mut ingress = self.async_ingress.lock().unwrap();
            ingress.identity = None;
            ingress.completions.clear();
        }
        let pending = self
            .async_state
            .borrow_mut()
            .pending
            .drain()
            .map(|(_, pending)| pending)
            .collect::<Vec<_>>();
        for pending in pending {
            (pending.cancel)();
        }
        self.image_decode_tickets.clear();
        self.encoded_image_nodes.borrow_mut().clear();
    }

    fn flush_async_ingress(&mut self) {
        let completions: Vec<_> = self
            .async_ingress
            .lock()
            .unwrap()
            .completions
            .drain(..)
            .collect();
        for completion in completions {
            if self.identity.get() != Some(completion.identity) {
                continue;
            }
            let Some(pending) = self
                .async_state
                .borrow_mut()
                .pending
                .remove(&completion.ticket)
            else {
                continue;
            };
            (pending.finalize)(self, completion.payload);
        }
    }

    #[cfg(feature = "test")]
    pub(crate) fn live_window(&self) -> Result<Window, RuntimeError> {
        self.windows
            .values()
            .next()
            .cloned()
            .ok_or(RuntimeError::MissingApplication)
    }
}

fn background_dispatch_waker(
    dispatcher: DispatcherQueue,
    identity: WindowToken,
) -> Arc<dyn Fn() -> bool + Send + Sync> {
    let queued = Arc::new(AtomicBool::new(false));
    Arc::new(move || {
        if queued.swap(true, Ordering::AcqRel) {
            return true;
        }

        let handler_queued = Arc::clone(&queued);
        let handler = DispatcherQueueHandler::new(move || {
            handler_queued.store(false, Ordering::Release);
            dispatch_native_events(identity);
        });
        if matches!(
            dispatcher.TryEnqueueWithPriority(DispatcherQueuePriority::Normal, &handler),
            Ok(true)
        ) {
            true
        } else {
            queued.store(false, Ordering::Release);
            false
        }
    })
}

fn native_drag_operation(operation: DragDropOperation) -> DataPackageOperation {
    match operation {
        DragDropOperation::Copy => DataPackageOperation::Copy,
        DragDropOperation::Move => DataPackageOperation::Move,
        DragDropOperation::Link => DataPackageOperation::Link,
    }
}

fn input_modifiers() -> windows_core::Result<InputModifiers> {
    let mut keys = [0u8; 256];
    if !unsafe { GetKeyboardState(keys.as_mut_ptr()) }.as_bool() {
        return Err(windows_core::Error::from_thread());
    }
    let mut modifiers = InputModifiers::NONE;
    if keys[0x10] & 0x80 != 0 {
        modifiers |= InputModifiers::SHIFT;
    }
    if keys[0x11] & 0x80 != 0 {
        modifiers |= InputModifiers::CONTROL;
    }
    if keys[0x12] & 0x80 != 0 {
        modifiers |= InputModifiers::ALT;
    }
    if keys[0x5b] & 0x80 != 0 || keys[0x5c] & 0x80 != 0 {
        modifiers |= InputModifiers::WINDOWS;
    }
    Ok(modifiers)
}

fn physical_key_status(value: CorePhysicalKeyStatus) -> PhysicalKeyStatus {
    PhysicalKeyStatus {
        repeat_count: value.repeat_count,
        scan_code: value.scan_code,
        is_extended: value.is_extended_key,
        is_menu_down: value.is_menu_key_down,
        was_down: value.was_key_down,
        is_released: value.is_key_released,
    }
}

fn key_event_info(args: &KeyRoutedEventArgs) -> windows_core::Result<KeyEventInfo> {
    Ok(KeyEventInfo {
        key: ReactorVirtualKey(args.Key()?.0 as u32),
        original_key: ReactorVirtualKey(args.OriginalKey()?.0 as u32),
        status: physical_key_status(args.KeyStatus()?),
        modifiers: input_modifiers()?,
    })
}

fn character_event_info(
    args: &CharacterReceivedRoutedEventArgs,
) -> windows_core::Result<CharacterEventInfo> {
    Ok(CharacterEventInfo {
        character: args.Character()?,
        status: physical_key_status(args.KeyStatus()?),
        modifiers: input_modifiers()?,
    })
}

fn focus_event_info(
    element: &UIElement,
    args: &RoutedEventArgs,
    pending_focus_state: Option<ElementFocusState>,
    got_focus: bool,
) -> windows_core::Result<FocusEventInfo> {
    let original = args.OriginalSource()?;
    let element_identity: &windows_core::IUnknown = element.into();
    let original_identity: &windows_core::IUnknown = (&original).into();
    let is_direct = element_identity == original_identity;
    let state = if got_focus {
        // FocusState can still describe the previous state while GotFocus is being raised.
        if let Some(state) = pending_focus_state {
            state
        } else {
            match element.FocusState()? {
                FocusState::Pointer => ElementFocusState::Pointer,
                FocusState::Keyboard => ElementFocusState::Keyboard,
                FocusState::Programmatic => ElementFocusState::Programmatic,
                _ => ElementFocusState::Unfocused,
            }
        }
    } else {
        ElementFocusState::Unfocused
    };
    Ok(FocusEventInfo { state, is_direct })
}

fn native_error(error: windows_core::Error) -> RuntimeError {
    RuntimeError::Native(error.code().0)
}

fn invoke_callback<T>(callback: &Callback<T>, value: T) {
    if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| callback.call(value))).is_err() {
        // 静默 abort 会让现场彻底消失（进程直接 __fastfail，WER 只记
        // StackHash12/E_FAIL，既没有 Rust panic 日志也没有访问违规转储）。
        // 留一行 stderr 才有机会事后定位。
        eprintln!("windows-reactor fatal: callback panicked, aborting");
        std::process::abort();
    }
}

fn subscribe_xaml_root_scale<F>(
    element: &UIElement,
    framework: &IFrameworkElement,
    sink: &EventSink,
    callback: F,
) -> Result<XamlRootScaleSubscriptions, RuntimeError>
where
    F: Clone + Fn(f64) + 'static,
{
    let changed = Rc::new(RefCell::new(None));
    observe_xaml_root_scale(element, &changed, sink, callback.clone())?;
    let loaded_element = element.clone();
    let loaded_changed = changed.clone();
    let loaded_sink = sink.clone();
    let loaded = framework
        .Loaded(move |_, _| {
            if let Err(error) = observe_xaml_root_scale(
                &loaded_element,
                &loaded_changed,
                &loaded_sink,
                callback.clone(),
            ) {
                loaded_sink.enqueue_host(HostEvent::Error(error));
            }
        })
        .map_err(native_error)?;
    Ok(XamlRootScaleSubscriptions {
        _changed: changed,
        _loaded: loaded,
    })
}

fn observe_xaml_root_scale<F>(
    element: &UIElement,
    changed: &Rc<RefCell<Option<windows_core::EventRevoker>>>,
    sink: &EventSink,
    callback: F,
) -> Result<(), RuntimeError>
where
    F: Fn(f64) + 'static,
{
    let root = match element.XamlRoot() {
        Ok(root) => root,
        Err(error) if error.code().is_ok() => return Ok(()),
        Err(error) => return Err(native_error(error)),
    };
    let scale = root.RasterizationScale().map_err(native_error)?;
    callback(scale);
    let changed_sink = sink.clone();
    let revoker = root
        .Changed(move |sender, _| {
            if let Some(sender) = sender.as_ref() {
                match sender.RasterizationScale() {
                    Ok(scale) => callback(scale),
                    Err(error) => {
                        changed_sink.enqueue_host(HostEvent::Error(native_error(error)));
                    }
                }
            }
        })
        .map_err(native_error)?;
    changed.replace(Some(revoker));
    Ok(())
}

fn xaml_scale(element: &UIElement) -> Result<f64, RuntimeError> {
    match element.XamlRoot() {
        Ok(root) => root.RasterizationScale().map_err(native_error),
        Err(error) if error.code().is_ok() => Ok(1.0),
        Err(error) => Err(native_error(error)),
    }
}

pub fn bootstrap_runtime() -> windows_core::Result<()> {
    if self_contained_manifest_present() && self_contained_runtime_present() {
        return Ok(());
    }

    // The self-contained marker with missing local files means the executable
    // is deployed framework-dependent (for example a packaged MSIX layout,
    // where self-contained runtime files cannot be shipped): resolve the
    // Windows App Runtime through the package graph instead.
    bootstrap::bootstrap()
}

fn self_contained_runtime_present() -> bool {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.to_path_buf()))
        .is_some_and(|parent| parent.join("Microsoft.WindowsAppRuntime.dll").is_file())
}

#[allow(clippy::manual_dangling_ptr)] // FindResource uses low pointer values for resource ordinals.
fn self_contained_manifest_present() -> bool {
    const SELF_CONTAINED_MARKER: &str = "windows-reactor-self-contained";
    let marker = SELF_CONTAINED_MARKER.as_bytes();

    unsafe {
        let module = GetModuleHandleW(std::ptr::null());
        if module.is_null() {
            return false;
        }
        let resource = FindResourceW(module, 1usize as *const u16, 24usize as *const u16);
        if resource.is_null() {
            return false;
        }
        let size = SizeofResource(module, resource) as usize;
        let loaded = LoadResource(module, resource);
        if loaded.is_null() {
            return false;
        }
        let data = LockResource(loaded).cast::<u8>();
        if data.is_null() {
            return false;
        }
        std::slice::from_raw_parts(data, size)
            .windows(marker.len())
            .any(|window| window == marker)
    }
}

pub fn initialize_ui_thread() -> windows_core::Result<()> {
    unsafe {
        _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }
    let result = unsafe { CoInitializeEx(std::ptr::null(), COINIT_APARTMENTTHREADED as u32) };
    if result == RPC_E_CHANGED_MODE {
        return Err(windows_core::Error::new(
            RPC_E_CHANGED_MODE,
            "WinUI requires an STA thread",
        ));
    }
    result.ok()
}

pub fn exit_application() -> windows_core::Result<()> {
    let result = Application::Current().and_then(|application| application.Exit());
    if result.is_err() {
        unsafe {
            PostQuitMessage(0);
        }
    }
    result
}

pub fn exit_ui_thread() {
    // 落盘：调用方的上下文（哪个 PumpError / 哪个窗口）才是关键信息，而这里
    // 只知道"要退出了"。写一行让事后能把两侧对上。
    {
        use std::io::Write;
        let path = std::env::temp_dir().join("windows-reactor-fault.log");
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            let _ = writeln!(file, "exit_ui_thread 被调用");
            let _ = file.flush();
        }
    }
    let handler = DispatcherQueueHandler::new(|| {
        _ = exit_application();
    });
    let queued = DispatcherQueue::GetForCurrentThread().and_then(|dispatcher| {
        dispatcher.TryEnqueueWithPriority(DispatcherQueuePriority::High, &handler)
    });
    if !matches!(queued, Ok(true)) {
        unsafe {
            PostQuitMessage(0);
        }
    }
}
