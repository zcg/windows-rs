use super::bindings as raw;
use super::*;
use bindings::Application;
use bindings::*;
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

pub(super) fn create_application(
    on_launched: Box<dyn FnOnce() -> Result<()>>,
) -> Result<Application> {
    Application::compose(ReactorApplicationOverrides::new(on_launched))
}

pub(super) fn install_xaml_controls_resources(application: &Application) -> Result<()> {
    let controls = XamlControlsResources::new()?;
    let resources: ResourceDictionary = controls.cast()?;
    application
        .Resources()?
        .MergedDictionaries()?
        .Append(&resources)
}

/// Converts a [`ResourceValue`] into the boxed value a WinUI resource dictionary stores.
///
/// Element-level overrides ([`WinUiAdapter::set_resource_overrides`]) and application-level
/// overrides ([`install_theme_resources`]) share this one conversion path, so both agree.
///
/// `Color` / `Thickness` / `CornerRadius` exist in the crate declaration layer and in the raw
/// `bindings` output. These conversions need the latter, hence the explicit `raw::` qualifier
/// (the lints misreport the glob conflict).
#[allow(unused_qualifications)]
fn resource_value(value: &ResourceValue) -> Result<IInspectable> {
    Ok(match value {
        ResourceValue::Color(value) => {
            let brush = raw::SolidColorBrush::new()?;
            brush.SetColor(raw::Color {
                a: value.a,
                r: value.r,
                g: value.g,
                b: value.b,
            })?;
            brush.into()
        }
        ResourceValue::Thickness(value) => windows_reference::IReference::from(raw::Thickness {
            left: value.left,
            top: value.top,
            right: value.right,
            bottom: value.bottom,
        })
        .into(),
        ResourceValue::CornerRadius(value) => {
            windows_reference::IReference::from(raw::CornerRadius {
                top_left: value.top_left,
                top_right: value.top_right,
                bottom_right: value.bottom_right,
                bottom_left: value.bottom_left,
            })
            .into()
        }
    })
}

/// Writes the host-declared application-level resource overrides into `Application.Resources`.
///
/// The values are entries of `Application.Resources` itself, which WinUI resolves before the
/// merged [`XamlControlsResources`] dictionary that supplies the theme resources. A key that
/// collides with a WinUI default therefore wins - `ControlCornerRadius` (control corners,
/// default 4) and `OverlayCornerRadius` (flyout corners, default 8) are examples - while keys
/// that are not declared keep the WinUI value.
///
/// This must run after [`install_xaml_controls_resources`] and before the window is created:
/// application-level resources are written once at startup, and writing them later does not
/// re-resolve control templates that are already loaded.
pub fn install_theme_resources(
    application: &Application,
    resources: &ResourceOverrides,
) -> Result<()> {
    if resources.values().next().is_none() {
        return Ok(());
    }
    let map = application
        .Resources()?
        .cast::<IMap<IInspectable, IInspectable>>()?;
    for (key, value) in resources.values() {
        let key = windows_reference::IReference::from(key);
        let value = resource_value(value)?;
        map.Insert(&key, &value)?;
    }
    Ok(())
}
