//! Reference implementation of `IElementFactory` for `ItemsView`, kept for
//! when the Windows App SDK 2.3.1 runtime crash (setting `ItemTemplate` on
//! `ItemsView` fails fast during layout, reproduced with an official
//! `DataTemplate`) is fixed or the runtime is upgraded.
//!
//! NOT wired into the backend: `items_view` currently leaves `ItemTemplate`
//! unset to avoid the crash. Re-enable by re-adding `mod element_factory;`
//! and the `SetItemTemplate` call in the `ItemsViewSource` prop handler.

use super::*;
use windows_core::*;

use crate::bindings::{
    ElementFactoryGetArgs, ElementFactoryRecycleArgs, IElementFactory, IElementFactory_Impl,
};

implement_decl! {
    impl ItemsViewElementFactory as pub ItemsViewElementFactory_Impl: [IElementFactory]
}

pub struct ItemsViewElementFactory;

impl IElementFactory_Impl for ItemsViewElementFactory_Impl {
    fn GetElement(&self, args: Ref<ElementFactoryGetArgs>) -> Result<UIElement> {
        let args = args.ok()?;
        let data = args.Data()?;
        let text = data
            .cast::<windows_reference::IReference<HSTRING>>()
            .ok()
            .and_then(|r| r.Value().ok())
            .map_or_else(|| "Item".to_string(), |value| value.to_string_lossy());

        let text_block = bindings::TextBlock::new()?;
        text_block.SetText(&text)?;
        text_block.SetFontSize(14.0)?;

        let border = bindings::Border::new()?;
        border.SetCornerRadius(bindings::CornerRadius {
            top_left: 4.0,
            top_right: 4.0,
            bottom_right: 4.0,
            bottom_left: 4.0,
        })?;
        border.SetPadding(Thickness {
            left: 12.0,
            top: 8.0,
            right: 12.0,
            bottom: 8.0,
        })?;
        border.SetChild(&text_block)?;
        border.cast()
    }

    fn RecycleElement(&self, _args: Ref<ElementFactoryRecycleArgs>) -> Result<()> {
        Ok(())
    }
}
