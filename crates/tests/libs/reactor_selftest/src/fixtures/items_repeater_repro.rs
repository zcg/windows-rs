use std::time::Duration;

use windows_reactor::*;

use crate::fixtures::reconciler::{FixtureFuture, cc};
use crate::harness::Harness;

pub fn items_repeater_repro(h: Harness) -> FixtureFuture {
    Box::pin(async move {
        let items = std::rc::Rc::new(
            (0..24)
                .map(|i| format!("Item {i}"))
                .collect::<Vec<String>>(),
        );
        h.mount(cc(move |cx| {
            let (tag, set_tag) = cx.use_state("other".to_string());
            let content: Element = if tag == "home" {
                let repeater: Element = items_repeater((*items).clone(), |item, _idx| {
                    let el: Element = border(text_block(item))
                        .padding(Thickness::uniform(10.0))
                        .corner_radius(4.0)
                        .background(ThemeRef::CardBackground)
                        .margin(Thickness::uniform(2.0))
                        .into();
                    el
                })
                .height(300.0)
                .width(460.0)
                .into();
                // Mimic gallery sample_card structure: border + padding + Expander
                let sample_area: Element = border(repeater)
                    .corner_radius(8.0)
                    .padding(Thickness::uniform(24.0))
                    .background(ThemeRef::SolidBackground)
                    .into();
                let source_area: Element = Expander::new(text_block("source").font_size(13.0))
                    .header("Source code")
                    .horizontal_alignment(HorizontalAlignment::Stretch)
                    .into();
                let inner: Element = vstack((sample_area, source_area)).spacing(0.0).into();
                let card: Element = border(inner)
                    .corner_radius(8.0)
                    .border_brush(ThemeRef::CardStroke)
                    .border_thickness(Thickness::uniform(1.0))
                    .background(ThemeRef::CardBackground)
                    .into();
                let page: Element = vstack((text_block("ItemsView").font_size(28.0), card))
                    .spacing(16.0)
                    .into();
                scroll_view(page).into()
            } else {
                text_block("other page").into()
            };
            let set_tag_nav = set_tag.clone();
            let nav = NavigationView::new(
                vec![
                    NavViewItem::new("Home").tag("home"),
                    NavViewItem::new("Other").tag("other"),
                ],
                content,
            )
            .selected_tag(&tag)
            .on_selection_changed(move |t: String| set_tag_nav.call(t));
            let go = { move || set_tag.call("home".to_string()) };
            vstack((button("switch").on_click(go), nav))
                .spacing(8.0)
                .into()
        }));
        let _ = h.activate();
        let ok0 = h
            .pump_until(Duration::from_secs(5), || {
                h.find_text("other page").is_some()
            })
            .await;
        h.check("ItemsRepeaterRepro_OtherPage", ok0);
        let clicked = h.click_button("switch");
        h.diag(&format!("click switch result: {clicked:?}"));
        let ok = h
            .pump_until(Duration::from_secs(5), || h.find_text("Item 0").is_some())
            .await;
        h.check("ItemsRepeaterRepro_NavItemsRealized", ok);
        h.diag(&format!(
            "other page still present: {}",
            h.find_text("other page").is_some()
        ));
        h.diag(&format!(
            "ItemsRepeater controls: {}",
            h.count_controls::<crate::bindings::ItemsRepeater>()
        ));
        h.diag(&h.dump_tree());
        let visible = if let Some(tb) = h.find_text("Item 0") {
            let w = tb
                .cast::<crate::bindings::IFrameworkElement>()
                .ok()
                .and_then(|fe| fe.ActualWidth().ok())
                .unwrap_or(0.0);
            let ht = tb
                .cast::<crate::bindings::IFrameworkElement>()
                .ok()
                .and_then(|fe| fe.ActualHeight().ok())
                .unwrap_or(0.0);
            h.diag(&format!("nav Item 0 actual size: {w}x{ht}"));
            w > 0.0 && ht > 0.0
        } else {
            false
        };
        h.check("ItemsRepeaterRepro_NavItem0Visible", visible);
    })
}
