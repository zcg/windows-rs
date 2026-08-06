use crate::controls::*;
use windows_reactor::*;

pub fn pull_to_refresh_page(_: &(), cx: &mut RenderCx) -> Element {
    let counter = cx.use_ref(0u32);
    let on_refresh = {
        let counter = counter.clone();
        move |_req: RefreshRequest| {
            // 同步刷新：回调返回后容器自动完成。异步场景可 req.deferral()。
            *counter.borrow_mut() += 1;
        }
    };

    let items: Vec<String> = (0..20)
        .map(|i| format!("Item {i} - pull down to refresh"))
        .collect();

    let list: Element = list_view(items.clone(), move |item, _| {
        text_block(item.clone()).padding(Thickness::uniform(8.0))
    })
    .height(320.0)
    .into();

    page_content(
        "Pull to Refresh",
        "RefreshContainer provides the standard pull-to-refresh experience for scrollable content.",
        vec![
            sample_card(
                "Basic Pull-to-Refresh",
                vstack((
                    RefreshContainer::new(
                        ScrollViewer::new(list)
                            .height(320.0)
                    )
                    .on_refresh_requested(on_refresh),
                    text_block(format!(
                        "Refreshed {} times",
                        *counter.borrow()
                    ))
                    .foreground(ThemeRef::SecondaryText),
                ))
                .spacing(8.0),
                r#"RefreshContainer::new(ScrollViewer::new(list_view(...))).on_refresh_requested(...)"#,
            ),
        ],
    )
}