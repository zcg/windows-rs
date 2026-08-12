//! Content islands: host a separate composition-backed `ContentIsland` inside
//! the current XAML tree through a `ChildSiteLink` attached to a placement
//! element's composition visual.

use super::*;

/// A live island connection. Call [`IslandHandle::close`] to tear it down.
pub struct IslandHandle {
    link: bindings::ChildSiteLink,
    island: bindings::ContentIsland,
}

impl IslandHandle {
    /// Close the child site link and the island.
    pub fn close(&self) {
        if !self
            .link
            .cast::<bindings::IClosableNotifier>()
            .and_then(|v| v.IsClosed())
            .unwrap_or(true)
        {
            self.link
                .cast::<bindings::IClosable>()
                .ok()
                .map(|v| v.Close());
        }
        if !self
            .island
            .cast::<bindings::IClosableNotifier>()
            .and_then(|v| v.IsClosed())
            .unwrap_or(true)
        {
            self.island
                .cast::<bindings::IClosable>()
                .ok()
                .map(|v| v.Close());
        }
    }
}

/// Attach a composition-backed content island to `host` (a XAML element such
/// as a `Rectangle`). The island is a separate `ContentIsland` connected to
/// the host's composition visual; it renders `size`-sized content in
/// `color`.
pub fn attach_island(host: &UIElement, size: (f32, f32), color: Color) -> Result<IslandHandle> {
    let xaml_root = host.XamlRoot()?;
    let parent_island = xaml_root.cast::<bindings::IXamlRoot4>()?.ContentIsland()?;

    let placement: bindings::Visual = bindings::ElementCompositionPreview::GetElementVisual(host)?;
    let placement_container: bindings::ContainerVisual = placement.cast()?;
    let link = bindings::ChildSiteLink::Create(&parent_island, &placement_container)?;

    let compositor = placement
        .cast::<bindings::ICompositionObject>()?
        .Compositor()?;
    let sprite = compositor.CreateSpriteVisual()?;
    let brush = compositor.CreateColorBrushWithColor(color)?;
    sprite.SetBrush(&brush)?;
    sprite
        .cast::<bindings::IVisual>()?
        .SetSize(bindings::Vector2 {
            x: size.0,
            y: size.1,
        })?;

    let island = bindings::ContentIsland::Create(&sprite)?;
    link.SetActualSize(bindings::Vector2 {
        x: size.0,
        y: size.1,
    })?;
    link.SetLocalToParentTransformMatrix(bindings::Matrix4x4 {
        m11: 1.0,
        m22: 1.0,
        m33: 1.0,
        m44: 1.0,
        ..Default::default()
    })?;
    link.Connect(&island)?;

    Ok(IslandHandle { link, island })
}
