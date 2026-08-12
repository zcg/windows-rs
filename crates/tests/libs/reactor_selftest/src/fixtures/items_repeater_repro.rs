use windows_reactor::*;

use crate::fixtures::reconciler::{FixtureFuture, cc};
use crate::harness::Harness;

pub fn items_repeater_repro(h: Harness) -> FixtureFuture {
    Box::pin(async move {
        let items = std::rc::Rc::new((0..24).map(|i| format!("Item {i}")).collect::<Vec<String>>());
        h.mount(cc(move |_cx| {
            items_repeater((*items).clone(), |item, _idx| {
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
            .into()
        }));
        let _ = h.activate();
        let ok = h
            .render_until("items realized", |h| h.find_text("Item 0").is_some())
            .await;
        h.check("ItemsRepeaterRepro_ItemsRealized", ok);
    })
}
