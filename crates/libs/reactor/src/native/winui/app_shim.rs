use super::*;
use windows_core::*;

implement_decl! {
    impl ReactorApplicationOverrides as pub ReactorApplicationOverrides_Impl:
        [IApplicationOverrides, IXamlMetadataProvider]
}

pub struct ReactorApplicationOverrides {
    controls_provider: RefCell<Option<XamlControlsXamlMetaDataProvider>>,
    on_launched: RefCell<Option<Box<dyn FnOnce() -> Result<()>>>>,
}

impl ReactorApplicationOverrides {
    fn new(on_launched: Box<dyn FnOnce() -> Result<()>>) -> Self {
        Self {
            controls_provider: RefCell::new(None),
            on_launched: RefCell::new(Some(on_launched)),
        }
    }

    fn provider(&self) -> Result<XamlControlsXamlMetaDataProvider> {
        if let Some(provider) = self.controls_provider.borrow().as_ref() {
            return Ok(provider.clone());
        }
        let provider = XamlControlsXamlMetaDataProvider::new()?;
        *self.controls_provider.borrow_mut() = Some(provider.clone());
        Ok(provider)
    }
}

impl IApplicationOverrides_Impl for ReactorApplicationOverrides_Impl {
    fn OnLaunched(&self, _args: Ref<LaunchActivatedEventArgs>) -> Result<()> {
        if let Some(on_launched) = self.on_launched.borrow_mut().take() {
            on_launched()?;
        }
        Ok(())
    }
}

impl IXamlMetadataProvider_Impl for ReactorApplicationOverrides_Impl {
    fn GetXamlType(&self, r#type: &TypeName) -> Result<IXamlType> {
        self.provider()?.GetXamlType(r#type)
    }

    fn GetXamlTypeByFullName(&self, full_name: &HSTRING) -> Result<IXamlType> {
        self.provider()?
            .GetXamlTypeByFullName(&full_name.to_string_lossy())
    }

    fn GetXmlnsDefinitions(&self) -> Result<Array<XmlnsDefinition>> {
        self.provider()?.GetXmlnsDefinitions()
    }
}

pub fn create_application(on_launched: Box<dyn FnOnce() -> Result<()>>) -> Result<Application> {
    Application::compose(ReactorApplicationOverrides::new(on_launched))
}

pub fn install_xaml_controls_resources(application: &Application) -> Result<()> {
    let controls = XamlControlsResources::new()?;
    let resources: ResourceDictionary = controls.cast()?;
    application
        .Resources()?
        .MergedDictionaries()?
        .Append(&resources)
}

/// 把宿主声明的应用级资源覆盖写进 `Application.Resources`。
///
/// 覆盖值写的是 `Application.Resources` **自身的条目**，其查找优先级高于它合并进来的
/// [`XamlControlsResources`]（WinUI 主题资源来自后者的合并字典）。因此同名键会覆盖
/// WinUI 默认值——例如 `ControlCornerRadius`（控件圆角，默认 4）与 `OverlayCornerRadius`
/// （浮层圆角，默认 8）——而没声明的键完全保持 WinUI 原样。
///
/// 调用时机必须是 [`install_xaml_controls_resources`] 之后、窗口创建之前：应用级资源
/// 只在启动时写入一次，启动后再写不会重新解析已经加载的控件模板。
pub fn install_theme_resources(
    application: &Application,
    resources: &ResourceOverrides,
) -> Result<()> {
    if resources.values().next().is_none() {
        return Ok(());
    }
    let map = application
        .Resources()?
        .cast::<windows_collections::IMap<IInspectable, IInspectable>>()?;
    for (key, value) in resources.values() {
        let key = windows_reference::IReference::from(key.as_ref());
        let value = resource_value(value)?;
        map.Insert(&key, &value)?;
    }
    Ok(())
}
