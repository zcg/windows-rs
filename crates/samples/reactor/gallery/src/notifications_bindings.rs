#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppNotification(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    AppNotification,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl AppNotification {
    pub(crate) fn CreateInstance(payload: &str) -> windows_core::Result<Self> {
        Self::IAppNotificationFactory(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateInstance)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(&windows_core::HSTRING::from(payload)),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    fn IAppNotificationFactory<
        R,
        F: FnOnce(&IAppNotificationFactory) -> windows_core::Result<R>,
    >(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<AppNotification, IAppNotificationFactory> =
            windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for AppNotification {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IAppNotification>();
}
unsafe impl windows_core::Interface for AppNotification {
    type Vtable = <IAppNotification as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IAppNotification as windows_core::Interface>::IID;
}
impl core::ops::Deref for AppNotification {
    type Target = IAppNotification;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for AppNotification {
    const NAME: &'static str = "Microsoft.Windows.AppNotifications.AppNotification";
}
unsafe impl Send for AppNotification {}
unsafe impl Sync for AppNotification {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppNotificationManager(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    AppNotificationManager,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl AppNotificationManager {
    pub(crate) fn Default() -> windows_core::Result<Self> {
        Self::IAppNotificationManagerStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Default)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub(crate) fn IsSupported() -> windows_core::Result<bool> {
        Self::IAppNotificationManagerStatics2(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).IsSupported)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        })
    }
    fn IAppNotificationManagerStatics<
        R,
        F: FnOnce(&IAppNotificationManagerStatics) -> windows_core::Result<R>,
    >(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<
            AppNotificationManager,
            IAppNotificationManagerStatics,
        > = windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
    fn IAppNotificationManagerStatics2<
        R,
        F: FnOnce(&IAppNotificationManagerStatics2) -> windows_core::Result<R>,
    >(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<
            AppNotificationManager,
            IAppNotificationManagerStatics2,
        > = windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for AppNotificationManager {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IAppNotificationManager>();
}
unsafe impl windows_core::Interface for AppNotificationManager {
    type Vtable = <IAppNotificationManager as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IAppNotificationManager as windows_core::Interface>::IID;
}
impl core::ops::Deref for AppNotificationManager {
    type Target = IAppNotificationManager;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for AppNotificationManager {
    const NAME: &'static str = "Microsoft.Windows.AppNotifications.AppNotificationManager";
}
unsafe impl Send for AppNotificationManager {}
unsafe impl Sync for AppNotificationManager {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BadgeNotificationGlyph(pub i32);
impl BadgeNotificationGlyph {
    pub const None: Self = Self(0);
    pub const Activity: Self = Self(1);
    pub const Alarm: Self = Self(2);
    pub const Alert: Self = Self(3);
    pub const Attention: Self = Self(4);
    pub const Available: Self = Self(5);
    pub const Away: Self = Self(6);
    pub const Busy: Self = Self(7);
    pub const Error: Self = Self(8);
    pub const NewMessage: Self = Self(9);
    pub const Paused: Self = Self(10);
    pub const Playing: Self = Self(11);
    pub const Unavailable: Self = Self(12);
}
impl windows_core::imp::TypeKind for BadgeNotificationGlyph {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for BadgeNotificationGlyph {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"enum(Microsoft.Windows.BadgeNotifications.BadgeNotificationGlyph;i4)",
    );
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BadgeNotificationManager(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    BadgeNotificationManager,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl BadgeNotificationManager {
    pub(crate) fn Current() -> windows_core::Result<Self> {
        Self::IBadgeNotificationManagerStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Current)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    fn IBadgeNotificationManagerStatics<
        R,
        F: FnOnce(&IBadgeNotificationManagerStatics) -> windows_core::Result<R>,
    >(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<
            BadgeNotificationManager,
            IBadgeNotificationManagerStatics,
        > = windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for BadgeNotificationManager {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IBadgeNotificationManager>();
}
unsafe impl windows_core::Interface for BadgeNotificationManager {
    type Vtable = <IBadgeNotificationManager as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IBadgeNotificationManager as windows_core::Interface>::IID;
}
impl core::ops::Deref for BadgeNotificationManager {
    type Target = IBadgeNotificationManager;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for BadgeNotificationManager {
    const NAME: &'static str = "Microsoft.Windows.BadgeNotifications.BadgeNotificationManager";
}
unsafe impl Send for BadgeNotificationManager {}
unsafe impl Sync for BadgeNotificationManager {}
windows_core::imp::define_interface!(
    IAppNotification,
    IAppNotification_Vtbl,
    0x373a6917_4116_5657_936a_15f99afdd667
);
impl windows_core::RuntimeType for IAppNotification {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IAppNotification_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IAppNotificationFactory,
    IAppNotificationFactory_Vtbl,
    0x9ffee485_184a_5c65_87a9_c1d94469dbe7
);
impl windows_core::RuntimeType for IAppNotificationFactory {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IAppNotificationFactory_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub CreateInstance: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IAppNotificationManager,
    IAppNotificationManager_Vtbl,
    0x55129688_b4bd_550b_ae6b_c24061954d91
);
impl windows_core::RuntimeType for IAppNotificationManager {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IAppNotificationManager {
    pub(crate) fn Register(&self) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).Register)(windows_core::Interface::as_raw(self))
                .ok()
        }
    }
    pub(crate) fn Show<P0>(&self, notification: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<AppNotification>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).Show)(
                windows_core::Interface::as_raw(self),
                notification.param().abi(),
            )
            .ok()
        }
    }
}
#[repr(C)]
pub struct IAppNotificationManager_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Register: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    Unregister: usize,
    UnregisterAll: usize,
    NotificationInvoked: usize,
    RemoveNotificationInvoked: usize,
    pub Show: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IAppNotificationManagerStatics,
    IAppNotificationManagerStatics_Vtbl,
    0x6cfc0d8d_84a3_5592_b4c6_e3e7e7c680e4
);
impl windows_core::RuntimeType for IAppNotificationManagerStatics {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IAppNotificationManagerStatics_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Default: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IAppNotificationManagerStatics2,
    IAppNotificationManagerStatics2_Vtbl,
    0x6eb42a35_e82f_5732_98f1_129705602f2e
);
impl windows_core::RuntimeType for IAppNotificationManagerStatics2 {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IAppNotificationManagerStatics2_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub IsSupported:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IBadgeNotificationManager,
    IBadgeNotificationManager_Vtbl,
    0x11cb6e8f_11ca_53f8_80f6_5330d44ba908
);
impl windows_core::RuntimeType for IBadgeNotificationManager {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IBadgeNotificationManager {
    pub(crate) fn SetBadgeAsCount(&self, notificationcount: u32) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetBadgeAsCount)(
                windows_core::Interface::as_raw(self),
                notificationcount,
            )
            .ok()
        }
    }
    pub(crate) fn SetBadgeAsGlyph(
        &self,
        glyphvalue: BadgeNotificationGlyph,
    ) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetBadgeAsGlyph)(
                windows_core::Interface::as_raw(self),
                glyphvalue,
            )
            .ok()
        }
    }
    pub(crate) fn ClearBadge(&self) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).ClearBadge)(windows_core::Interface::as_raw(
                self,
            ))
            .ok()
        }
    }
}
#[repr(C)]
pub struct IBadgeNotificationManager_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub SetBadgeAsCount:
        unsafe extern "system" fn(*mut core::ffi::c_void, u32) -> windows_core::HRESULT,
    pub SetBadgeAsGlyph: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        BadgeNotificationGlyph,
    ) -> windows_core::HRESULT,
    pub ClearBadge: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IBadgeNotificationManagerStatics,
    IBadgeNotificationManagerStatics_Vtbl,
    0xa6e71616_7c9f_5d22_ad1c_f4ab874087b5
);
impl windows_core::RuntimeType for IBadgeNotificationManagerStatics {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IBadgeNotificationManagerStatics_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Current: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
