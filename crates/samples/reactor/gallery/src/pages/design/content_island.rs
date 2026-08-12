use crate::controls::*;
use windows_reactor::*;

pub fn content_island_page(_: &(), cx: &mut RenderCx) -> Element {
    let (status, set_status) = cx.use_state(String::from(
        "Click \"Load island\" to host a separate ContentIsland inside the rectangle.",
    ));
    let host = cx.use_ref(None::<UIElement>);
    let island = cx.use_ref(None::<IslandHandle>);

    let host_for_mount = host.clone();
    let host_shape = Shape::rectangle()
        .fill_rgb(235, 235, 235)
        .width(320.0)
        .height(200.0)
        .on_mounted(move |native: Option<windows_core::IInspectable>| {
            if let Some(native) = native
                && let Ok(ui) = native.cast::<UIElement>()
            {
                host_for_mount.set(Some(ui));
            }
        });

    let load = {
        let island = island.clone();
        let set_status = set_status.clone();
        move || {
            if let Some(ui) = host.borrow().as_ref() {
                match attach_island(
                    ui,
                    (320.0, 200.0),
                    Color {
                        a: 255,
                        r: 26,
                        g: 115,
                        b: 232,
                    },
                ) {
                    Ok(handle) => {
                        island.set(Some(handle));
                        set_status
                            .call("Island attached: separate content, rendered in-tree.".into());
                    }
                    Err(err) => set_status.call(format!("Error: {err}")),
                }
            }
        }
    };

    let remove = move || {
        if let Some(handle) = island.replace(None) {
            handle.close();
            set_status.call("Island removed.".into());
        }
    };

    page_content(
        "Content Islands",
        "Host separate content in the current XAML tree via ChildSiteLink.",
        vec![sample_card(
            "Host a ContentIsland",
            vstack((
                host_shape,
                hstack((
                    button("Load island").on_click(load),
                    button("Remove island").on_click(remove),
                ))
                .spacing(8.0),
                text_block(status).opacity(0.6),
            ))
            .spacing(8.0),
            r#"let xaml_root = host.XamlRoot()?;
let parent = xaml_root.cast::<IXamlRoot4>()?.ContentIsland()?;
let visual = ElementCompositionPreview::GetElementVisual(host)?;
let link = ChildSiteLink::Create(&parent, &visual.cast()?)?;
let island = ContentIsland::Create(&sprite_visual)?;
link.Connect(&island)?;"#,
        )],
    )
}
